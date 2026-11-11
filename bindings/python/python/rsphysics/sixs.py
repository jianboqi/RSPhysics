"""6SV2.1 through a direct Fortran C ABI; no 6S executable is launched.

All configuration is validated/rendered in the public Rust adapter. Parameters
use micrometres, degrees, km, hPa, g/cm2 and atm-cm, as named. The original
mixed-precision kernel, full formatted report and auxiliary files are retained.
"""

from __future__ import annotations
from copy import deepcopy
from concurrent.futures import ProcessPoolExecutor
from datetime import date, timedelta
from enum import IntEnum
from functools import cache
import math
import json
import re
from collections.abc import Mapping
import numpy as np
from . import _native, spectral, sensors
from ._sixs_observations import (
    aeronet_aot,
    aeronet_inversion,
    aster_samples,
    bas_levels,
    radiosonde_levels,
    solar_angles,
    utc_datetime,
    wyoming_levels,
    wyoming_url_levels,
)
from ._sixs_plots import plot_angular_grid, plot_principal_plane, plot_spectrum
from ._api import (
    ModelError,
    _call,
    _numpy_outputs,
    run as _run,
    run_batch as _run_batch,
)

MODEL_ID = "atmosphere.sixsv21"
REFRACTIVE_WAVELENGTHS_UM = np.array(
    [
        0.350,
        0.400,
        0.412,
        0.443,
        0.470,
        0.488,
        0.515,
        0.550,
        0.590,
        0.633,
        0.670,
        0.694,
        0.760,
        0.860,
        1.240,
        1.536,
        1.650,
        1.950,
        2.250,
        3.750,
    ]
)
BRDF_MODELS = {
    "measured": 0,
    "hapke": 1,
    "verstraete": 2,
    "roujean": 3,
    "walthall": 4,
    "minnaert": 5,
    "ocean": 6,
    "iaquinta_pinty": 7,
    "rahman": 8,
    "kuusk": 9,
    "modis": 10,
    "ross_li_maignan": 11,
}


@cache
def _bands():
    entries = json.loads(_call(_native.sixs_resources_json))["bands"]
    for entry in entries:
        label = re.sub(r"[^A-Z0-9]+", "_", entry["name"].upper()).strip("_")
        label = re.sub(r"_(\d[A-Z0-9]*)$", r"_B\1", label)
        if entry["sensor"] in {"tm", "mss", "etm"}:
            label = "LANDSAT_" + label
        entry["key"] = label
    return entries


def bands():
    """6S band IDs, symbolic names, sensor families and integration bounds in µm."""
    return deepcopy(_bands())


Band = IntEnum("Band", {entry["key"]: entry["id"] for entry in _bands()})


def standard_levels(profile=6):
    """Copy a standard profile as (34,5) NumPy rows: km, hPa, K, g/m3, g/m3.

    The original final 99999-km sentinel row is retained. Profile IDs are 1..6.
    """
    try:
        return np.array(
            json.loads(_call(_native.sixs_resources_json))["profiles"][str(profile)][
                "levels"
            ],
            dtype=float,
        )
    except KeyError as error:
        raise ModelError(
            "invalid_parameter", "standard profile must be 1..6"
        ) from error


def _plain(value):
    if isinstance(value, SixS):
        return value.to_dict()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, Mapping):
        return {key: _plain(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_plain(item) for item in value]
    return value


def prepare(configuration=None):
    """Validate/default configuration and export the exact original input cards."""
    try:
        payload = json.dumps(_plain(configuration or {}), allow_nan=False)
    except (TypeError, ValueError) as error:
        raise ModelError("invalid_parameter", str(error)) from error
    return json.loads(_call(_native.sixs_prepare_json, payload))


def run(configuration=None, **sections):
    """Run one direct library calculation; return the standard result/manifest.

    Sections may be passed as mappings, e.g. ``run(spectrum=Spectrum.band(43))``.
    No compiler or separate 6S installation is required by a prebuilt wheel.
    """
    parameters = _plain(configuration or {})
    if not isinstance(parameters, dict):
        raise ModelError("invalid_parameter", "configuration must be a mapping or SixS")
    parameters.update(_plain(sections))
    return _run(MODEL_ID, parameters)


def run_cards(input_cards, *, files=None, program="standard"):
    """Run original input cards and named auxiliary files through the same C ABI.

    Advanced input preserves original options; it bypasses structured physical
    validation. ``files`` maps simple ASCII filenames to original ASCII contents.
    """
    return run(input_cards=input_cards, files=files or {}, program=program)


def parse_report(report, *, files=None):
    """Parse an existing original report without executing the model."""
    return _numpy_outputs(
        json.loads(
            _call(_native.sixs_parse_report_json, report, json.dumps(files or {}))
        )
    )


def _batch_item(parameters):
    try:
        return run(parameters), None
    except ModelError as error:
        # Exception reconstruction must not depend on ModelError's constructor
        # signature in spawned workers.
        return None, (error.kind, str(error))


def run_batch(configurations, *, workers=1):
    """Ordered, fail-fast batch. Parallel workers each call the direct library.

    For ``workers>1`` use Python's usual ``if __name__ == '__main__'`` guard
    in scripts. No 6S executable is invoked. Native COMMON state is reset and
    calls within a process are serialized; worker processes permit parallelism.
    """
    if isinstance(workers, bool) or not isinstance(workers, int) or workers < 1:
        raise ModelError("invalid_parameter", "workers must be a positive integer")
    rows = [_plain(row) for row in configurations]
    if workers == 1:
        return _run_batch(MODEL_ID, rows)
    for index, row in enumerate(rows):
        try:
            prepare(row)
        except ModelError as error:
            raise ModelError(error.kind, f"batch row {index}: {error}") from error
    results = []
    with ProcessPoolExecutor(max_workers=workers) as pool:
        iterator = pool.map(_batch_item, rows)
        for index in range(len(rows)):
            try:
                result, failure = next(iterator)
                if failure is not None:
                    raise ModelError(failure[0], failure[1])
                results.append(result)
            except Exception as error:
                raise ModelError(
                    getattr(error, "kind", "internal"), f"batch row {index}: {error}"
                ) from error
    return results


def spectrum(wavelengths_um, configuration=None, *, output=None, workers=1):
    """Monochromatic spectral sweep; return NumPy signals and full ordered runs."""
    wavelengths = np.asarray(wavelengths_um, dtype=float)
    if wavelengths.ndim != 1 or not np.isfinite(wavelengths).all():
        raise ModelError(
            "invalid_parameter", "wavelengths_um must be a finite one-dimensional array"
        )
    base = _plain(configuration or {})
    if base.get("input_cards"):
        raise ModelError(
            "invalid_parameter", "Wavelength sweep requires structured inputs"
        )
    rows = []
    for wavelength in wavelengths:
        row = base.copy()
        row["spectrum"] = Spectrum.monochromatic(float(wavelength))
        rows.append(row)
    runs = run_batch(rows, workers=workers)
    return {
        "wavelengths_um": wavelengths.copy(),
        **_scan_outputs(runs, output),
    }


def extract_output(runs, field):
    """Collect a dotted numeric result field; array indices are zero-based."""
    if not isinstance(field, str) or not field:
        raise ModelError("invalid_parameter", "A numeric result field name is required")
    values = []
    try:
        for run in runs:
            value = run["result"]
            for segment in field.split("."):
                value = value[segment if isinstance(value, Mapping) else int(segment)]
            array = np.asarray(value)
            if array.dtype.kind not in "iufO" or isinstance(value, Mapping):
                raise ValueError
            values.append(np.asarray(value, dtype=float))
        return np.asarray(values, dtype=float)
    except (KeyError, IndexError, ValueError, TypeError) as error:
        raise ModelError(
            "invalid_parameter", f"Unknown or non-numeric 6S output: {field}"
        ) from error


def _scan_outputs(runs, output=None, *, shape=None):
    keys = ("apparent_reflectance", "apparent_radiance_w_m2_sr_um")
    values = {key: extract_output(runs, key) for key in keys}
    if output is not None:
        values["values"] = extract_output(runs, output)
    if shape is not None:
        values = {
            key: array.reshape(*shape, *array.shape[1:])
            for key, array in values.items()
        }
    return dict(values, runs=runs)


def _spectral_range(lower, upper, step):
    if not math.isfinite(step) or step <= 0:
        raise ModelError(
            "invalid_parameter", "Spectral spacing must be finite and positive"
        )
    count = math.floor((upper - lower) / step + 1e-10) + 1
    return np.minimum(lower + step * np.arange(count), upper)


def vnir(configuration=None, *, spacing_um=0.005, output=None, workers=1):
    """Monochromatic sweep from 0.40 to 1.40 µm."""
    return spectrum(
        _spectral_range(0.4, 1.4, spacing_um),
        configuration,
        output=output,
        workers=workers,
    )


def full_spectrum(configuration=None, *, spacing_um=0.01, output=None, workers=1):
    """Monochromatic sweep across the 6SV2.1 range of 0.25 to 4.00 µm."""
    return spectrum(
        _spectral_range(0.25, 4, spacing_um),
        configuration,
        output=output,
        workers=workers,
    )


def sensor_bands(sensor, configuration=None, *, band_ids=None, output=None, workers=1):
    """Sweep a built-in 6S sensor family or a registry sensor ID.

    Use ``bands()`` for built-in family names and ``sensors.list_sensors()`` for
    the registry. Use numbers or symbolic names for 6S families, and band IDs
    such as "B2" for registry profiles. Requested bands are returned in order.
    """
    original = [band for band in _bands() if band["sensor"] == sensor]
    if original:
        selected = {band["id"]: band for band in original}
        if isinstance(band_ids, (str, int, np.integer)):
            band_ids = [band_ids]
        ids = (
            list(selected)
            if band_ids is None
            else [Spectrum.band(b)["band"] for b in band_ids]
        )
        try:
            names = [selected[band_id]["name"] for band_id in ids]
        except KeyError as error:
            raise ModelError(
                "invalid_parameter", f"Unknown 6S band for {sensor}: {error.args[0]}"
            ) from error
        windows = [Spectrum.band(band_id) for band_id in ids]
        centres = [(selected[i]["lower_um"] + selected[i]["upper_um"]) / 2 for i in ids]
    else:
        profile = sensors.load(sensor)
        if band_ids is not None:
            profile = profile.select(band_ids)
        ids = list(profile.band_ids)
        names = ids.copy()
        windows = [
            Spectrum.from_samples(b.wavelengths_nm / 1000, b.response)
            for b in profile.bands
        ]
        centres = [(w["lower_um"] + w["upper_um"]) / 2 for w in windows]
    base = _plain(configuration or {})
    if not ids or base.get("input_cards"):
        raise ModelError(
            "invalid_parameter",
            "Sensor sweep needs at least one band and structured inputs",
        )
    runs = run_batch(
        [dict(base, spectrum=window) for window in windows], workers=workers
    )
    return {
        "band_ids": ids,
        "band_names": names,
        "centre_wavelengths_um": np.array(centres),
        **_scan_outputs(runs, output),
    }


def angles(
    view_zenith_deg, view_azimuth_deg, configuration=None, *, output=None, workers=1
):
    """Sweep paired view angles in degrees; a scalar broadcasts across an array."""
    try:
        zenith, azimuth = np.broadcast_arrays(
            np.atleast_1d(np.asarray(view_zenith_deg, dtype=float)),
            np.atleast_1d(np.asarray(view_azimuth_deg, dtype=float)),
        )
    except (TypeError, ValueError) as error:
        raise ModelError(
            "invalid_parameter", "View angles must be broadcastable numeric arrays"
        ) from error
    if zenith.ndim != 1 or not zenith.size or not np.isfinite([zenith, azimuth]).all():
        raise ModelError(
            "invalid_parameter",
            "View angles must be nonempty finite one-dimensional arrays",
        )
    return {
        "view_zenith_deg": zenith.copy(),
        "view_azimuth_deg": azimuth.copy(),
        **_scan_outputs(
            _run_angle_pairs(zenith, azimuth, configuration, "view", workers), output
        ),
    }


def _run_angle_pairs(zenith, azimuth, configuration, direction, workers):
    base = _plain(configuration or {})
    geometry = base.get("geometry", {})
    if base.get("input_cards") or geometry.get("mode", 0) != 0:
        raise ModelError(
            "invalid_parameter", "Angle sweep requires structured user geometry"
        )
    rows = [
        dict(
            base,
            geometry=dict(
                geometry,
                **{
                    f"{direction}_zenith_deg": float(z),
                    f"{direction}_azimuth_deg": float(a),
                },
            ),
        )
        for z, a in zip(zenith, azimuth)
    ]
    return run_batch(rows, workers=workers)


def angular_grid(
    zenith_deg,
    azimuth_deg,
    configuration=None,
    *,
    direction="view",
    output=None,
    workers=1,
):
    """Sweep a solar or view grid with output axes (zenith, azimuth, ...)."""
    if direction not in ("solar", "view"):
        raise ModelError("invalid_parameter", "direction must be solar or view")
    axes = [np.asarray(axis, dtype=float) for axis in (zenith_deg, azimuth_deg)]
    if any(
        axis.ndim != 1
        or not axis.size
        or not np.isfinite(axis).all()
        or np.any(np.diff(axis) <= 0)
        for axis in axes
    ):
        raise ModelError(
            "invalid_parameter",
            "Angle grid axes must be increasing finite one-dimensional arrays",
        )
    zenith, azimuth = axes
    grid_z, grid_a = np.meshgrid(zenith, azimuth, indexing="ij")
    runs = _run_angle_pairs(
        grid_z.ravel(), grid_a.ravel(), configuration, direction, workers
    )
    return {
        "zenith_deg": zenith.copy(),
        "azimuth_deg": azimuth.copy(),
        "direction": direction,
        **_scan_outputs(runs, output, shape=grid_z.shape),
    }


def principal_plane(
    signed_view_zenith_deg, configuration=None, *, output=None, workers=1
):
    """Sweep the solar principal plane; negative angles face away from the sun."""
    base = _plain(configuration or {})
    azimuth = base.get("geometry", {}).get("solar_azimuth_deg", 0.0)
    signed = np.atleast_1d(np.asarray(signed_view_zenith_deg, dtype=float))
    result = angles(
        np.abs(signed),
        (azimuth + np.where(signed < 0, 180, 0)) % 360,
        base,
        output=output,
        workers=workers,
    )
    return dict(result, signed_view_zenith_deg=signed.copy())


def azimuth_scan(
    view_zenith_deg, configuration=None, *, step_deg=5.0, output=None, workers=1
):
    """Sweep azimuth from 0° to below 360° at a fixed view zenith."""
    if not math.isfinite(step_deg) or not 0 < step_deg <= 360:
        raise ModelError("invalid_parameter", "step_deg must be in (0, 360]")
    return angles(
        view_zenith_deg,
        np.arange(0, 360, step_deg),
        configuration,
        output=output,
        workers=workers,
    )


class SixS:
    """Reusable independent configuration; each run resets the original kernel."""

    def __init__(self, configuration=None, **sections):
        self.configuration = deepcopy(_plain(configuration or {}))
        self.configuration.update(_plain(sections))

    def to_dict(self):
        return deepcopy(self.configuration)

    def prepare(self):
        return prepare(self.configuration)

    def input_cards(self):
        return self.prepare()["prepared"]["input_cards"]

    def run(self, **sections):
        return run(self.configuration, **sections)


class Geometry:
    @staticmethod
    def user(
        solar_zenith_deg=30.0,
        solar_azimuth_deg=0.0,
        view_zenith_deg=20.0,
        view_azimuth_deg=90.0,
        month=7,
        day=1,
        *,
        max_scattering_order=None,
    ):
        return dict(
            mode=0,
            solar_zenith_deg=solar_zenith_deg,
            solar_azimuth_deg=solar_azimuth_deg,
            view_zenith_deg=view_zenith_deg,
            view_azimuth_deg=view_azimuth_deg,
            month=month,
            day=day,
            max_scattering_order=max_scattering_order,
        )

    @staticmethod
    def satellite(mode, **parameters):
        return dict(mode=mode, **parameters)

    @staticmethod
    def from_time_and_location(
        latitude_deg,
        longitude_deg,
        when,
        *,
        view_zenith_deg=0.0,
        view_azimuth_deg=0.0,
        max_scattering_order=None,
    ):
        """Approximate geometric solar angles for an ISO 8601 or aware datetime."""
        utc = utc_datetime(when)
        zenith, azimuth = solar_angles(latitude_deg, longitude_deg, utc)
        if zenith > 89:
            raise ModelError(
                "invalid_parameter",
                "Solar zenith is outside the daytime 6S range (0–89°)",
            )
        return Geometry.user(
            zenith,
            azimuth,
            view_zenith_deg,
            view_azimuth_deg,
            utc.month,
            utc.day,
            max_scattering_order=max_scattering_order,
        )


class Atmosphere:
    @staticmethod
    def standard(profile=2):
        return {"profile": profile}

    @staticmethod
    def water_ozone(water_vapour_g_cm2, ozone_atm_cm):
        return dict(
            profile=8, water_vapour_g_cm2=water_vapour_g_cm2, ozone_atm_cm=ozone_atm_cm
        )

    @staticmethod
    def radiosonde(levels):
        return dict(profile=7, levels=_plain(levels))

    @staticmethod
    def from_latitude_and_date(latitude_deg, when):
        """Select a climatological 6S profile using latitude and hemisphere season.

        Tropical below 23.5° absolute latitude, midlatitude below 60°, and
        subarctic at higher latitudes. Northern summer is May–October.
        """
        if isinstance(when, str):
            try:
                when = date.fromisoformat(when)
            except ValueError as error:
                raise ModelError(
                    "invalid_parameter", "Expected an ISO calendar date"
                ) from error
        if not isinstance(when, date) or not -90 <= latitude_deg <= 90:
            raise ModelError(
                "invalid_parameter", "Expected a date and latitude in [-90, 90]"
            )
        if abs(latitude_deg) < 23.5:
            return Atmosphere.standard(1)
        summer = (5 <= when.month <= 10) == (latitude_deg > 0)
        return Atmosphere.standard(
            (2 if abs(latitude_deg) < 60 else 4) + int(not summer)
        )

    @staticmethod
    def from_radiosonde(levels, *, fallback_profile=6):
        """Map measured [km,hPa,K,water g/m³,optional ozone g/m³] to 34 levels."""
        return Atmosphere.radiosonde(
            radiosonde_levels(levels, standard_levels(fallback_profile))
        )

    @staticmethod
    def from_wyoming(path, *, fallback_profile=6):
        """Read a downloaded University of Wyoming TEXT:LIST or TEXT:CSV sounding."""
        return Atmosphere.from_radiosonde(
            wyoming_levels(path), fallback_profile=fallback_profile
        )

    @staticmethod
    def from_wyoming_url(url, *, fallback_profile=6, timeout=20):
        """Fetch one University of Wyoming TEXT:LIST or TEXT:CSV sounding."""
        return Atmosphere.from_radiosonde(
            wyoming_url_levels(url, timeout), fallback_profile=fallback_profile
        )

    @staticmethod
    def from_bas(path, *, fallback_profile=6):
        """Read the legacy BAS radiosonde table (hPa, m, °C, dew point °C)."""
        return Atmosphere.from_radiosonde(
            bas_levels(path), fallback_profile=fallback_profile
        )


def _aeronet_time(when, max_age):
    if max_age is not None and (
        not isinstance(max_age, timedelta) or max_age < timedelta()
    ):
        raise ModelError(
            "invalid_parameter", "max_age must be a nonnegative timedelta or None"
        )
    return utc_datetime(when)


class Aerosol:
    @staticmethod
    def predefined(model=1, *, aot550=0.2, visibility_km=None):
        return dict(model=model, aot550=aot550, visibility_km=visibility_km)

    @staticmethod
    def from_aeronet(path, when, *, model=1, max_age=timedelta(hours=1)):
        """Use AERONET AOD at 550 nm or interpolate between bracketing channels.

        Takes a downloaded all-points CSV and a timezone-aware observation
        time. The selected 6S aerosol type controls optical properties.
        """
        if model not in (1, 2, 3, 5, 6, 7):
            raise ModelError(
                "invalid_parameter", "AERONET AOD requires a predefined aerosol model"
            )
        aot = aeronet_aot(path, _aeronet_time(when, max_age), max_age)
        return Aerosol.predefined(model if aot > 0 else 0, aot550=aot)

    @staticmethod
    def from_aeronet_inversion(path, when, *, max_age=timedelta(hours=1)):
        """Configure Mie aerosol from a single-site V3 ALL or V2 Combined file.

        The refractive index is interpolated to the 20 6S wavelengths with
        constant endpoints outside the measured wavelength interval.
        """
        aot, size_distribution, real, imaginary = aeronet_inversion(
            path, _aeronet_time(when, max_age), max_age, REFRACTIVE_WAVELENGTHS_UM
        )
        return Aerosol.sunphotometer(
            size_distribution,
            aot550=aot,
            refractive_real=real,
            refractive_imag=imaginary,
        )

    @staticmethod
    def mixture(components, **loading):
        return dict(model=4, components=_plain(components), **loading)

    @staticmethod
    def layered(layers, **optical_properties):
        """Layer thickness and AOT with one optical model shared by the layers."""
        return dict(model=-1, layers=_plain(layers), **_plain(optical_properties))

    @staticmethod
    def lognormal(modes, *, radius_min_um=0.01, radius_max_um=1.0, **options):
        return dict(
            model=8,
            modes=_plain(modes),
            radius_min_um=radius_min_um,
            radius_max_um=radius_max_um,
            **options,
        )

    @staticmethod
    def gamma(parameters, *, radius_min_um=0.01, radius_max_um=1.0, **options):
        return dict(
            model=9,
            gamma=_plain(parameters),
            radius_min_um=radius_min_um,
            radius_max_um=radius_max_um,
            **options,
        )

    @staticmethod
    def junge(exponent=3.5, *, radius_min_um=0.01, radius_max_um=1.0, **options):
        return dict(
            model=10,
            junge_exponent=exponent,
            radius_min_um=radius_min_um,
            radius_max_um=radius_max_um,
            **options,
        )

    @staticmethod
    def sunphotometer(distribution, **options):
        return dict(model=11, sunphotometer=_plain(distribution), **options)

    @staticmethod
    def from_mie(filename, **loading):
        return dict(model=12, mie_file=filename, **loading)

    @staticmethod
    def mode(
        mean_radius_um=0.1,
        sigma=1.5,
        fraction=1.0,
        *,
        refractive_real=None,
        refractive_imag=None,
    ):
        return dict(
            mean_radius_um=mean_radius_um,
            sigma=sigma,
            fraction=fraction,
            refractive_real=_plain(
                refractive_real if refractive_real is not None else [1.5] * 20
            ),
            refractive_imag=_plain(
                refractive_imag if refractive_imag is not None else [0.01] * 20
            ),
        )


class Altitudes:
    @staticmethod
    def satellite(*, target_altitude_km=0.0, target_pressure_hpa=None):
        return dict(
            target_altitude_km=target_altitude_km,
            target_pressure_hpa=target_pressure_hpa,
            sensor_height_km=None,
        )

    @staticmethod
    def ground(**target):
        return dict(sensor_height_km=0.0, **target)

    @staticmethod
    def aircraft(
        height_km,
        *,
        path_water_vapour_g_cm2=-1.0,
        path_ozone_atm_cm=-1.0,
        path_aot550=-1.0,
        **target,
    ):
        return dict(
            sensor_height_km=height_km,
            path_water_vapour_g_cm2=path_water_vapour_g_cm2,
            path_ozone_atm_cm=path_ozone_atm_cm,
            path_aot550=path_aot550,
            **target,
        )


class Spectrum:
    @staticmethod
    def monochromatic(wavelength_um=0.55):
        return dict(mode="monochromatic", wavelength_um=wavelength_um)

    @staticmethod
    def band(band):
        if isinstance(band, str):
            try:
                band = Band[band.upper()].value
            except KeyError as error:
                raise ModelError(
                    "invalid_parameter", f"Unknown 6S band: {band}"
                ) from error
        return dict(mode="band", band=band)

    @staticmethod
    def uniform(lower_um, upper_um):
        return dict(mode="uniform", lower_um=lower_um, upper_um=upper_um)

    @staticmethod
    def step(lower_um, upper_um):
        return dict(mode="step", lower_um=lower_um, upper_um=upper_um)

    @staticmethod
    def response(lower_um, upper_um, response):
        return dict(
            mode="response",
            lower_um=lower_um,
            upper_um=upper_um,
            response=_plain(response),
        )

    @staticmethod
    def from_samples(wavelengths_um, response):
        """Resample a response curve to the 6S grid; zero outside the table."""
        x = np.asarray(wavelengths_um, dtype=float) * 1000
        response = np.asarray(response, dtype=float)
        if (
            x.ndim != 1
            or x.size < 2
            or x.shape != response.shape
            or not np.isfinite(x).all()
            or not np.isfinite(response).all()
            or np.any(np.diff(x) <= 0)
            or np.any(response < 0)
            or x[0] < 250
            or x[-1] > 4000
        ):
            raise ModelError(
                "invalid_parameter",
                "Expected increasing wavelengths in [0.25, 4] µm and a matching nonnegative response",
            )
        lo = math.floor((x[0] - 250) / 2.5)
        hi = math.ceil((x[-1] - 250) / 2.5)
        grid = 250 + 2.5 * np.arange(lo, hi + 1)
        inside = (grid >= x[0] - 1e-9) & (grid <= x[-1] + 1e-9)
        values = np.zeros(grid.size)
        if np.any(inside):
            values[inside] = spectral.resample(
                x, response, np.clip(grid[inside], x[0], x[-1])
            )
        if not np.any(values > 0):
            raise ModelError(
                "invalid_parameter", "Response has no positive sample on the 6S grid"
            )
        return Spectrum.response(grid[0] / 1000, grid[-1] / 1000, values)


def _spectrum_grid_nm(window):
    mode = window.get("mode", "monochromatic")
    if mode == "band":
        band = next(
            (item for item in bands() if item["id"] == window.get("band")), None
        )
        if band is None:
            raise ModelError("invalid_parameter", "Unknown built-in 6S band")
        lo, hi = band["lower_um"], band["upper_um"]
    elif mode == "monochromatic":
        lo = hi = window.get("wavelength_um", 0.55)
    elif mode in ("step", "uniform", "response"):
        lo, hi = window["lower_um"], window["upper_um"]
    else:
        raise ModelError("invalid_parameter", "Unknown 6S spectrum mode")
    if not 0.25 <= lo <= hi <= 4:
        raise ModelError(
            "invalid_parameter", "6S spectral bounds must be in [0.25, 4] µm"
        )
    lo, hi = [math.floor((bound - 0.25) / 0.0025 + 0.5) for bound in (lo, hi)]
    return 250 + 2.5 * np.arange(lo, hi + 1)


class Surface:
    @staticmethod
    def lambertian(reflectance=0.2):
        return dict(mode="lambertian", target=dict(spectrum=0, value=reflectance))

    @staticmethod
    def predefined(spectrum=1):
        return dict(mode="lambertian", target=dict(spectrum=spectrum))

    @staticmethod
    def spectrum(lower_um, upper_um, reflectance):
        return dict(
            mode="lambertian",
            target=dict(
                spectrum=-1,
                lower_um=lower_um,
                upper_um=upper_um,
                values=_plain(reflectance),
            ),
        )

    @staticmethod
    def sampled_reflectance(wavelengths_um, reflectance, spectrum):
        """Resample a ground spectrum for a 6S band without extrapolation.

        Pass the returned mapping to ``Surface.heterogeneous`` as a target or
        background, or set it as the target of a uniform Lambertian surface.
        """
        grid = _spectrum_grid_nm(spectrum)
        x = np.asarray(wavelengths_um, dtype=float) * 1000
        y = np.asarray(reflectance, dtype=float)
        if y.ndim != 1 or not np.isfinite(y).all() or np.any((y < 0) | (y > 1)):
            raise ModelError(
                "invalid_parameter",
                "Ground reflectance must be a finite one-dimensional array in [0, 1]",
            )
        values = spectral.resample(x, y, grid)
        return dict(
            spectrum=-1,
            lower_um=grid[0] / 1000,
            upper_um=grid[-1] / 1000,
            values=values.tolist(),
        )

    @staticmethod
    def sampled_curve(wavelengths_um, reflectance):
        """Reflectance input resampled separately for each simulated band."""
        return dict(
            spectrum=-1,
            wavelengths_um=_plain(wavelengths_um),
            values=_plain(reflectance),
        )

    @staticmethod
    def from_samples(wavelengths_um, reflectance):
        """Uniform Lambertian surface defined by arbitrary spectral samples."""
        return dict(
            mode="lambertian", target=Surface.sampled_curve(wavelengths_um, reflectance)
        )

    @staticmethod
    def from_aster(path):
        """Read an ASTER/ECOSTRESS reflectance file as a Lambertian surface."""
        return Surface.from_samples(*aster_samples(path))

    @staticmethod
    def heterogeneous(target, background, *, radius_km=1.0):
        def select(value):
            return (
                dict(spectrum=0, value=value) if np.isscalar(value) else _plain(value)
            )

        return dict(
            mode="heterogeneous",
            target=select(target),
            background=select(background),
            radius_km=radius_km,
        )

    @staticmethod
    def brdf(model, parameters):
        return dict(
            mode="brdf",
            brdf_model=BRDF_MODELS[model] if isinstance(model, str) else model,
            parameters=_plain(parameters),
        )

    @staticmethod
    def measured(sun_table, view_table, *, albedo, directional_reflectance):
        return dict(
            mode="brdf",
            brdf_model=0,
            sun_table=_plain(sun_table),
            view_table=_plain(view_table),
            albedo=albedo,
            directional_reflectance=directional_reflectance,
        )

    @staticmethod
    def hapke(single_scattering_albedo, asymmetry, hotspot_amplitude, hotspot_width):
        return Surface.brdf(
            "hapke",
            [single_scattering_albedo, asymmetry, hotspot_amplitude, hotspot_width],
        )

    @staticmethod
    def roujean(isotropic, geometric, volumetric):
        return Surface.brdf("roujean", [isotropic, geometric, volumetric])

    @staticmethod
    def walthall(quadratic_product, quadratic_sum, azimuth_term, albedo):
        return Surface.brdf(
            "walthall", [quadratic_product, quadratic_sum, azimuth_term, albedo]
        )

    @staticmethod
    def minnaert(k, albedo):
        return Surface.brdf("minnaert", [k, albedo])

    @staticmethod
    def rahman(rho0, asymmetry, k):
        return Surface.brdf("rahman", [rho0, asymmetry, k])

    @staticmethod
    def iaquinta_pinty(
        leaf_angle,
        hotspot,
        lai,
        hotspot_size,
        leaf_reflectance,
        leaf_transmittance,
        soil_albedo,
    ):
        return Surface.brdf(
            "iaquinta_pinty",
            [
                leaf_angle,
                hotspot,
                lai,
                hotspot_size,
                leaf_reflectance,
                leaf_transmittance,
                soil_albedo,
            ],
        )

    @staticmethod
    def verstraete(
        extinction,
        phase_function,
        multiple_scattering,
        leaf_area_density,
        sunfleck_radius,
        orientation_a,
        orientation_b,
        single_scattering_albedo,
        phase_a=0.0,
        phase_b=0.0,
    ):
        return Surface.brdf(
            "verstraete",
            [
                extinction,
                phase_function,
                multiple_scattering,
                leaf_area_density,
                sunfleck_radius,
                orientation_a,
                orientation_b,
                single_scattering_albedo,
                phase_a,
                phase_b,
            ],
        )

    @staticmethod
    def kuusk(
        lai,
        lad_eccentricity,
        lad_angle_deg,
        relative_leaf_size,
        chlorophyll_ug_cm2,
        water_thickness_cm,
        internal_layers,
        refractive_index_ratio,
        soil_price_weight,
    ):
        return Surface.brdf(
            "kuusk",
            [
                lai,
                lad_eccentricity,
                lad_angle_deg,
                relative_leaf_size,
                chlorophyll_ug_cm2,
                water_thickness_cm,
                internal_layers,
                refractive_index_ratio,
                soil_price_weight,
            ],
        )

    @staticmethod
    def ross_li_maignan(isotropic, volumetric, geometric):
        return Surface.brdf("ross_li_maignan", [isotropic, volumetric, geometric])

    @staticmethod
    def modis(isotropic, volumetric, geometric):
        return Surface.brdf("modis", [isotropic, volumetric, geometric])

    @staticmethod
    def ocean(
        wind_speed_m_s=5.0, wind_azimuth_deg=0.0, salinity_ppt=34.3, pigment_mg_m3=0.1
    ):
        return Surface.brdf(
            "ocean", [wind_speed_m_s, wind_azimuth_deg, salinity_ppt, pigment_mg_m3]
        )


class Correction:
    @staticmethod
    def none():
        return dict(mode="none")

    @staticmethod
    def from_radiance(value):
        return dict(mode="radiance", value=value)

    @staticmethod
    def from_reflectance(value):
        return dict(mode="reflectance", value=value)


class Polarization:
    @staticmethod
    def stokes(q, u):
        return dict(mode="stokes", q=q, u=u)

    @staticmethod
    def vegetation(fraction):
        return dict(mode="vegetation", vegetation_fraction=fraction)

    @staticmethod
    def ocean(wind_speed_m_s=5.0, wind_azimuth_deg=0.0):
        return dict(
            mode="ocean",
            wind_speed_m_s=wind_speed_m_s,
            wind_azimuth_deg=wind_azimuth_deg,
        )
