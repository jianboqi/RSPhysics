import json
from collections.abc import Mapping
from typing import Any

import numpy as np
from . import _native


class ModelError(ValueError):
    """A classified scientific/model API error; inspect ``kind`` and ``message``."""
    def __init__(self, kind: str, message: str):
        self.kind = kind
        self.message = message
        super().__init__(f"{kind}: {message}")


def _call(function, *args):
    try:
        return function(*args)
    except (ValueError, RuntimeError, TypeError) as error:
        try:
            record = json.loads(str(error))
            kind, message = record["kind"], record["message"]
        except (ValueError, TypeError, KeyError):
            kind = "internal" if isinstance(error, RuntimeError) else "invalid_parameter"
            message = str(error)
        raise ModelError(kind, message) from error


def _dumps(value):
    def convert(item):
        if isinstance(item, np.generic):
            return item.item()
        raise TypeError(f"cannot serialize {type(item).__name__}")
    try:
        return json.dumps(value, allow_nan=False, default=convert)
    except (TypeError, ValueError) as error:
        raise ModelError("invalid_parameter", str(error)) from error


def _numpy_outputs(value):
    if isinstance(value, dict):
        return {key: _numpy_outputs(item) for key, item in value.items()}
    if isinstance(value, list):
        try:
            array = np.asarray(value)
        except ValueError:
            return value
        if array.dtype.kind in "fiu":
            return array
    return value


def list_models() -> list[dict[str, Any]]:
    """Descriptors of models compiled into this installed wheel."""
    return json.loads(_call(_native.list_models_json))


def describe(model_id: str) -> dict[str, Any]:
    """Return the scientific metadata and generated input/output JSON Schemas."""
    return json.loads(_call(_native.describe_json, model_id))


def run(model_id: str, params: Mapping[str, Any] | None = None) -> dict[str, Any]:
    """Run one model; return ``result`` (numeric arrays are NumPy) and ``manifest``."""
    if params is not None and not isinstance(params, Mapping):
        raise ModelError("invalid_parameter", "params must be a mapping")
    envelope = json.loads(_call(_native.run_json, model_id, _dumps(dict(params) if params is not None else {})))
    envelope["result"] = _numpy_outputs(envelope["result"])
    return envelope


def run_batch(model_id: str, parameter_table) -> list[dict[str, Any]]:
    """Ordered, fail-fast batch of parameter mappings; errors identify a zero-based row."""
    rows = []
    try:
        iterator = iter(parameter_table)
    except TypeError as error:
        raise ModelError("invalid_parameter", "parameter_table must be an iterable of mappings") from error
    for row in iterator:
        if not isinstance(row, Mapping):
            raise ModelError("invalid_parameter", "each batch row must be a mapping")
        rows.append(dict(row))
    envelopes = json.loads(_call(_native.run_batch_json, model_id, _dumps(rows)))
    for envelope in envelopes:
        envelope["result"] = _numpy_outputs(envelope["result"])
    return envelopes
