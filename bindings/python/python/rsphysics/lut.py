"""Sampling, construction and inversion of numerical lookup tables."""

from collections.abc import Mapping
from copy import deepcopy
from dataclasses import asdict, dataclass, field
from functools import cached_property
from hashlib import sha256
from itertools import product
from math import prod
from numbers import Real
import json
import operator
import os
from pathlib import Path
from tempfile import NamedTemporaryFile
from zipfile import BadZipFile

import numpy as np

from . import _native
from ._api import ModelError, _call, _dumps, describe, run_batch
from ._arrays import parameters_for, run_array
from ._numeric import real_array
from .sensors import Sensor


def _invalid(message):
    return ModelError("invalid_parameter", message)


def _integer(value, name, minimum=1):
    if isinstance(value, (bool, np.bool_)):
        raise _invalid(f"{name} must be an integer >= {minimum}")
    try:
        result = operator.index(value)
    except TypeError as error:
        raise _invalid(f"{name} must be an integer >= {minimum}") from error
    if result < minimum:
        raise _invalid(f"{name} must be an integer >= {minimum}")
    return result


def _names(values):
    if isinstance(values, str):
        raise _invalid("names must be a sequence of strings")
    try:
        names = tuple(values)
    except TypeError as error:
        raise _invalid("names must be a sequence of strings") from error
    if not names or any(not isinstance(name, str) or not name.strip() for name in names):
        raise _invalid("names must be nonempty strings")
    if len(set(names)) != len(names):
        raise _invalid("names must be unique")
    return names


def _mapping(value, name):
    if not isinstance(value, Mapping) or any(not isinstance(key, str) for key in value):
        raise _invalid(f"{name} must be a mapping with string keys")
    return value


def _json_value(value):
    if isinstance(value, Mapping):
        return {key: _json_value(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_json_value(item) for item in value]
    if isinstance(value, np.ndarray):
        return value.tolist()
    return value


def _record(value, name):
    """Copy plain, finite JSON metadata, including nested NumPy inputs."""
    return json.loads(_dumps(_json_value(_mapping(value, name))))


def _finite_array(value, *, ndim, name):
    array = real_array(value, ndim=ndim, name=name)
    if not np.isfinite(array).all():
        raise _invalid(f"{name} must contain only finite values")
    return array


@dataclass(frozen=True, eq=False)
class Samples:
    """Ordered parameter names, an (n, parameters) matrix, and sampling metadata."""

    names: tuple[str, ...]
    values: np.ndarray
    design: dict = field(default_factory=dict)

    def __post_init__(self):
        names = _names(self.names)
        values = _finite_array(self.values, ndim=2, name="parameter samples")
        if values.shape[0] == 0 or values.shape[1] != len(names):
            raise _invalid("parameter samples must have rows and one column per name")
        design = _record(self.design, "sampling metadata")
        if "bounds" in design:
            bounds = _finite_array(design["bounds"], ndim=2, name="sampling bounds")
            if (bounds.shape != (len(names), 2) or np.any(bounds[:, 0] > bounds[:, 1])
                    or np.any(values < bounds[:, 0]) or np.any(values > bounds[:, 1])):
                raise _invalid("sampling bounds must contain every parameter value")
        values = values.copy(order="C")
        values.flags.writeable = False
        object.__setattr__(self, "names", names)
        object.__setattr__(self, "values", values)
        object.__setattr__(self, "design", design)

    @classmethod
    def from_columns(cls, parameters):
        """Create samples from a mapping of parameter names to equal-length columns."""
        names = _names(_mapping(parameters, "parameters"))
        columns = [_finite_array(parameters[name], ndim=1, name=name) for name in names]
        if len({column.size for column in columns}) != 1:
            raise _invalid("parameter columns must have equal lengths")
        return cls(names, np.column_stack(columns), {"method": "external"})


def sample_grid(levels, *, max_samples=1_000_000):
    """Enumerate the Cartesian product of explicit, unique parameter levels."""
    names = _names(_mapping(levels, "grid levels"))
    columns = [_finite_array(levels[name], ndim=1, name=name) for name in names]
    if any(not column.size or np.unique(column).size != column.size for column in columns):
        raise _invalid("each grid parameter needs nonempty, unique levels")
    count = prod(column.size for column in columns)
    limit = _integer(max_samples, "max_samples")
    if count > limit:
        raise _invalid(f"grid has {count} samples, exceeding max_samples={limit}")
    values = np.fromiter(
        (value for row in product(*columns) for value in row),
        dtype=np.float64, count=count * len(names),
    ).reshape(count, len(names))
    bounds = [[column.min(), column.max()] for column in columns]
    return Samples(names, values, {"method": "grid", "levels": dict(zip(names, columns)), "bounds": bounds})


def sample_lhs(ranges, n_samples, *, seed=None, log_parameters=()):
    """Draw a Latin hypercube with one random draw per marginal stratum."""
    names = _names(_mapping(ranges, "sampling ranges"))
    n = _integer(n_samples, "n_samples")
    seed = np.random.SeedSequence().entropy if seed is None else _integer(seed, "seed", 0)
    log_parameters = (log_parameters,) if isinstance(log_parameters, str) else tuple(log_parameters)
    if any(name not in names for name in log_parameters):
        raise _invalid("log_parameters must name sampled parameters")
    bounds = _finite_array([ranges[name] for name in names], ndim=2, name="sampling ranges")
    if bounds.shape != (len(names), 2) or np.any(bounds[:, 0] >= bounds[:, 1]):
        raise _invalid("each range must contain an increasing lower and upper bound")
    rng = np.random.Generator(np.random.PCG64(seed))
    values = np.empty((n, len(names)))
    for i, (name, (low, high)) in enumerate(zip(names, bounds)):
        u = (rng.permutation(n) + rng.random(n)) / n
        if name in log_parameters:
            if low <= 0:
                raise _invalid(f"log-sampled parameter {name} must have a positive range")
            values[:, i] = np.exp((1 - u) * np.log(low) + u * np.log(high))
        else:
            values[:, i] = (1 - u) * low + u * high
        np.clip(values[:, i], low, high, out=values[:, i])
    design = {"method": "latin_hypercube", "seed": seed, "rng": "PCG64",
              "bounds": bounds, "log_parameters": log_parameters}
    return Samples(names, values, _record(design, "sampling metadata"))


@dataclass(frozen=True)
class Channel:
    """A named observable. Use unit='1' for quantities without a unit."""

    name: str
    quantity: str
    unit: str = "1"

    def __post_init__(self):
        if any(not isinstance(x, str) or not x.strip() for x in (self.name, self.quantity, self.unit)):
            raise _invalid("channel name, quantity and unit must be nonempty strings")


def _channels(channels):
    try:
        result = tuple(channels)
    except TypeError as error:
        raise _invalid("channels must be a sequence of Channel objects") from error
    if not result or any(not isinstance(channel, Channel) for channel in result):
        raise _invalid("channels must be a nonempty sequence of Channel objects")
    _names(channel.name for channel in result)
    return result


@dataclass(frozen=True, eq=False)
class LookupTable:
    """Sampled parameter rows and their simulated observables in channel order."""

    samples: Samples
    simulated: np.ndarray
    channels: tuple[Channel, ...]
    context: dict = field(default_factory=dict)
    metadata: dict = field(default_factory=dict)

    def __post_init__(self):
        if not isinstance(self.samples, Samples):
            raise _invalid("samples must be a Samples object")
        channels = _channels(self.channels)
        simulated = _finite_array(self.simulated, ndim=2, name="simulated observations")
        if simulated.shape != (len(self.samples.values), len(channels)):
            raise _invalid("simulated observations must have one row per sample and one column per channel")
        # The Rust matcher keeps a snapshot. An immutable NumPy buffer keeps
        # returned residuals in sync with that snapshot across calls.
        simulated = np.frombuffer(simulated.tobytes(order="C"), dtype=np.float64).reshape(simulated.shape)
        object.__setattr__(self, "simulated", simulated)
        object.__setattr__(self, "channels", channels)
        object.__setattr__(self, "context", _record(self.context, "observation context"))
        object.__setattr__(self, "metadata", _record(self.metadata, "LUT metadata"))

    @classmethod
    def from_arrays(cls, parameters, simulated, channels, *, context=None):
        """Import a model-independent LUT from named parameter columns or Samples."""
        samples = parameters if isinstance(parameters, Samples) else Samples.from_columns(parameters)
        return cls(samples, simulated, channels, context if context is not None else {})

    @cached_property
    def _matcher(self):
        return _call(_native.LutMatcher, self.simulated)

    def match(self, observations, *, channels, sigma=None, mask=None, k=1,
              max_score=None, boundary_fraction=0.05, context=None, workers=1,
              return_residuals=True):
        """Return exact top-k matches with signed residuals and quality indicators."""
        if _channels(channels) != self.channels:
            raise _invalid("observation channel names, quantities, units and order must match the LUT")
        observed_context = {} if context is None else _record(context, "observation context")
        if observed_context != self.context:
            raise _invalid("observation context must match the LUT")
        if sigma is None and len({channel.unit for channel in self.channels}) > 1:
            raise _invalid("provide channel standard deviations when comparing different units")
        k = _integer(k, "k")
        workers = _integer(workers, "workers")
        if not isinstance(return_residuals, (bool, np.bool_)):
            raise _invalid("return_residuals must be boolean")
        if k > len(self.simulated):
            raise _invalid("k cannot exceed the number of LUT samples")
        observations = real_array(observations, name="observations")
        if observations.ndim == 1:
            observations = observations[None, :]
        if observations.ndim != 2 or observations.shape[1] != len(self.channels):
            raise _invalid("observations must have one column per LUT channel")
        shape = observations.shape
        deviations = (np.empty((0, 0)) if sigma is None
                      else _query_matrix(real_array(sigma, name="sigma"), shape, "sigma"))
        if mask is None:
            valid = np.empty((0, 0), dtype=bool)
        else:
            valid = _query_matrix(_input_array(mask, "mask", "b"), shape, "mask")
        fraction = _scalar(boundary_fraction, "boundary_fraction", maximum=0.5)
        threshold = None if max_score is None else _scalar(max_score, "max_score")
        matches = _call(self._matcher.search, observations, deviations, valid, k, workers)
        indices, scores = matches["indices"], matches["scores"]
        candidates = self.samples.values[indices]
        residuals = None
        if return_residuals:
            with np.errstate(over="ignore", invalid="ignore"):
                residuals = self.simulated[indices] - observations[:, None, :]
            if valid.size:
                np.copyto(residuals, np.nan, where=~valid[:, None, :])
        bounds = self.samples.design.get("bounds")
        if bounds is None:
            low, high = self.samples.values.min(axis=0), self.samples.values.max(axis=0)
        else:
            low, high = np.asarray(bounds).T
        lower_edge = (1 - fraction) * low + fraction * high
        upper_edge = fraction * low + (1 - fraction) * high
        near_boundary = (low < high) & ((candidates[:, 0] <= lower_edge) | (candidates[:, 0] >= upper_edge))
        return {
            "parameter_names": self.samples.names,
            "indices": indices,
            "scores": scores,
            "parameters": candidates,
            "residuals": residuals,
            "near_boundary": near_boundary,
            "poor_fit": None if threshold is None else scores[:, 0] > threshold,
        }

    def iter_match(self, observations, *, channels, sigma=None, mask=None,
                   pixel_mask=None, k=1, max_score=None, boundary_fraction=0.05,
                   context=None, workers=1, block_size=65_536, return_residuals=False):
        """Yield matches for blocks of a (pixels, channels) array or a band-last raster.

        Blocks follow row-major pixel order. The returned flat pixel indices
        identify matched pixels. Nonfinite and masked channels are excluded.
        Pixels without a usable channel and pixels excluded by pixel_mask are skipped.
        NumPy memory maps and noncontiguous array views are read one block at a time.
        """
        image = _input_array(observations, "observations", "fiu")
        if image.ndim not in (2, 3) or image.shape[-1] != len(self.channels):
            raise _invalid("observations must be shaped (pixels, channels) or (height, width, channels)")
        block_size = _integer(block_size, "block_size")
        channels = _channels(channels)
        shape = image.shape
        count = prod(shape[:-1])
        pixels = (None if pixel_mask is None
                  else _broadcast(_input_array(pixel_mask, "pixel_mask", "b"), shape[:-1], "pixel_mask"))
        valid_bands = (None if mask is None
                       else _broadcast(_input_array(mask, "mask", "b"), shape, "mask"))
        uncertainty = None if sigma is None else _input_array(sigma, "sigma", "fiu")
        shared_sigma = uncertainty is not None and uncertainty.ndim <= 1
        if uncertainty is not None and not shared_sigma:
            uncertainty = _broadcast(uncertainty, shape, "sigma")

        for start in range(0, count, block_size):
            stop = min(start + block_size, count)
            flat_indices = np.arange(start, stop, dtype=np.intp)
            block = slice(start, stop) if image.ndim == 2 else np.divmod(flat_indices, shape[1])
            values = image[block]
            valid = np.isfinite(values)
            if valid_bands is not None:
                valid &= valid_bands[block]
            usable = valid.any(axis=1)
            if pixels is not None:
                usable &= pixels[block]
            deviation = (uncertainty if shared_sigma or uncertainty is None
                         else uncertainty[block][usable])
            # An all-valid block can use the unmasked native search.
            valid = valid[usable]
            block_mask = None if valid.all() else valid
            matched = self.match(
                values[usable], channels=channels, sigma=deviation, mask=block_mask,
                k=k, max_score=max_score, boundary_fraction=boundary_fraction,
                context=context, workers=workers, return_residuals=return_residuals,
            )
            yield {"start": start, "stop": stop, "pixel_indices": flat_indices[usable], **matched}

    def save(self, path):
        """Write a versioned NumPy archive, replacing the destination atomically."""
        path = Path(path)
        header = _dumps({"format_version": 1, "parameter_names": self.samples.names,
                         "sampling": self.samples.design, "channels": [asdict(ch) for ch in self.channels],
                         "context": self.context, "metadata": self.metadata})
        temporary = None
        try:
            with NamedTemporaryFile(dir=path.parent, suffix=".npz", delete=False) as stream:
                temporary = Path(stream.name)
                np.savez(stream, parameters=self.samples.values, simulated=self.simulated, header=header)
            os.replace(temporary, path)
        finally:
            if temporary is not None:
                temporary.unlink(missing_ok=True)

    @classmethod
    def load(cls, path):
        """Load a validated LUT archive containing arrays and JSON metadata."""
        try:
            with np.load(path, allow_pickle=False) as data:
                header = json.loads(data["header"].item())
                if header.get("format_version") != 1:
                    raise _invalid("unsupported LUT archive version")
                samples = Samples(header["parameter_names"], data["parameters"], header["sampling"])
                channels = tuple(Channel(**item) for item in header["channels"])
                return cls(samples, data["simulated"], channels, header["context"], header["metadata"])
        except ModelError:
            raise
        except (AttributeError, BadZipFile, KeyError, TypeError, ValueError) as error:
            raise _invalid("invalid LUT archive") from error


def _broadcast(value, shape, name):
    try:
        return np.broadcast_to(value, shape)
    except ValueError as error:
        raise _invalid(f"{name} must broadcast to the observation shape") from error


def _query_matrix(value, shape, name):
    target = (1, shape[1]) if value.ndim < 2 or value.shape[0] == 1 else shape
    return _broadcast(value, target, name)


def _input_array(value, name, kinds):
    try:
        array = np.asarray(value)
    except (TypeError, ValueError) as error:
        raise _invalid(f"{name} must be a rectangular array") from error
    if array.dtype.kind not in kinds:
        kind = "boolean" if kinds == "b" else "real numeric"
        raise _invalid(f"{name} must contain {kind} values")
    return array


def _scalar(value, name, maximum=np.inf):
    array = _finite_array(value, ndim=0, name=name)
    if not 0 <= array <= maximum:
        raise _invalid(f"{name} must be nonnegative and at most {maximum}")
    return float(array)


def _model_input(fixed, paths, values):
    row = deepcopy(fixed)
    for path, value in zip(paths, values):
        destination = row
        for key in path[:-1]:
            destination = destination.setdefault(key, {})
            if not isinstance(destination, dict):
                raise _invalid(f"sampled parameter {'.'.join(path)} conflicts with a fixed value")
        if path[-1] in destination:
            raise _invalid(f"sampled parameter {'.'.join(path)} is already specified")
        destination[path[-1]] = float(value)
    return row


def _numeric_defaults(model_id, properties, names, fixed):
    """Use the native batch adapter where all parameters fit its numeric columns."""
    try:
        columns = parameters_for(model_id)
    except ModelError as error:
        if error.kind != "unknown_model":
            raise
        return None
    if not (set(names) | set(fixed)).issubset(columns):
        return None
    defaults = []
    for name in columns:
        value = fixed.get(name, properties[name].get("default"))
        if name not in fixed and isinstance(value, list) and len(value) == 1:
            value = value[0]
        if not isinstance(value, Real) or isinstance(value, bool):
            return None
        defaults.append(value)
    indices = [columns.index(name) for name in names]
    return np.array(defaults), indices


def _features(result, outputs, sensor, rows=None):
    if callable(outputs):
        return _finite_array(outputs(result), ndim=1, name="projected observations")
    try:
        values = [real_array(result[name], name=name) for name in outputs]
    except KeyError as error:
        raise _invalid(f"model result has no output {error.args[0]!r}") from error
    if sensor is not None:
        if "wavelengths_nm" not in result or values[0].ndim != (1 if rows is None else 2):
            raise _invalid("sensor integration requires a spectral vector and wavelengths_nm")
        return sensor.convolve(result["wavelengths_nm"], values[0])["values"]
    if rows is None:
        if any(value.ndim > 1 for value in values):
            raise _invalid("multidimensional outputs need a projection function")
        return np.concatenate([np.atleast_1d(value) for value in values])
    if any(value.ndim not in (1, 2) or len(value) != rows for value in values):
        raise _invalid("batched outputs must have one scalar or vector per sample")
    return np.column_stack(values)


def build(model_id, samples, *, output, channels, fixed=None, context=None,
          sensor=None, constraint=None, batch_size=128):
    """Build a LUT from an RSPhysics model and sampled parameter rows.

    Output is a field name, an ordered sequence of field names, or a callable
    mapping a single model result to a feature vector. A Sensor integrates a
    single spectral field. A constraint receives the supplied fixed and sampled
    inputs for each row and returns a boolean. The table records exclusions.
    """
    if not isinstance(samples, Samples):
        raise _invalid("samples must be a Samples object")
    channels = _channels(channels)
    batch_size = _integer(batch_size, "batch_size")
    outputs = output if callable(output) else _names((output,) if isinstance(output, str) else output)
    if sensor is not None:
        if not isinstance(sensor, Sensor) or callable(outputs) or len(outputs) != 1:
            raise _invalid("sensor integration requires a Sensor and one spectral output field")
        if tuple(channel.name for channel in channels) != sensor.band_ids:
            raise _invalid("channel names and order must match the selected sensor bands")
    if constraint is not None and not callable(constraint):
        raise _invalid("constraint must be callable")
    fixed = {} if fixed is None else _record(fixed, "fixed parameters")
    context = {} if context is None else _record(context, "observation context")
    descriptor = describe(model_id)
    properties = descriptor["input_schema"]["properties"]
    paths = [name.split(".") for name in samples.names]
    if any(not all(path) or path[0] not in properties for path in paths):
        raise _invalid("each sampled parameter must name a model input")
    _model_input(fixed, paths, samples.values[0])

    attempted = len(samples.values)
    if constraint is not None:
        keep = np.empty(attempted, dtype=bool)
        for i, values in enumerate(samples.values):
            accepted = constraint(_model_input(fixed, paths, values))
            if not isinstance(accepted, (bool, np.bool_)):
                raise _invalid(f"constraint must return a boolean at sample {i}")
            keep[i] = accepted
        if not keep.any():
            raise _invalid(f"constraint excluded all {attempted} samples")
        if not keep.all():
            samples = Samples(samples.names, samples.values[keep], samples.design)

    count = len(samples.values)
    simulated = np.empty((count, len(channels)))
    numeric = None if callable(outputs) else _numeric_defaults(model_id, properties, samples.names, fixed)
    for start in range(0, count, batch_size):
        block = samples.values[start:start + batch_size]
        if numeric is not None:
            defaults, indices = numeric
            matrix = np.broadcast_to(defaults, (len(block), len(defaults))).copy()
            matrix[:, indices] = block
            result = run_array(model_id, matrix)
            values = _features(result["result"], outputs, sensor, len(block))
            manifest = result["manifests"][0]
        else:
            rows = [_model_input(fixed, paths, sample) for sample in block]
            results = run_batch(model_id, rows)
            vectors = [_features(item["result"], outputs, sensor) for item in results]
            if any(vector.shape != (len(channels),) for vector in vectors):
                raise _invalid(f"model outputs must match the channel definition in batch starting at {start}")
            values = np.stack(vectors)
            manifest = results[0]["manifest"]
        if values.shape != (len(block), len(channels)) or not np.isfinite(values).all():
            raise _invalid(f"model outputs must be finite and match the channels in batch starting at {start}")
        simulated[start:start + len(block)] = values

    provenance = {key: manifest[key] for key in (
        "model_id", "scientific_version", "implementation_version", "api_version", "data_hashes", "build",
    ) if key in manifest}
    metadata = {
        "model": provenance,
        "fixed": fixed,
        "outputs": "projection" if callable(outputs) else outputs,
        "attempted_samples": attempted,
        "accepted_samples": count,
        "rejected_samples": attempted - count,
    }
    if sensor is not None:
        profile = json.dumps(sensor.metadata, sort_keys=True, separators=(",", ":"), allow_nan=False)
        metadata["sensor"] = {"id": sensor.id, "version": sensor.version,
                              "bands": sensor.band_ids, "weighting": "response",
                              "profile_sha256": sha256(profile.encode()).hexdigest()}
    return LookupTable(samples, simulated, channels, context, metadata)
