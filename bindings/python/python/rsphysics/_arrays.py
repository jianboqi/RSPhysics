"""Unified NumPy model batches. Scientific values never traverse JSON.

Inputs are copied in Rust before releasing the GIL, so another Python thread
cannot mutate the array while computation is reading it. Outputs own their
Rust-produced NumPy buffers. Only small provenance manifests use JSON.
"""
import itertools
import json
import operator
import re
from collections.abc import Mapping
import numpy as np
from . import _native
from ._api import ModelError, _call, describe, _dumps
from ._numeric import real_array

def parameters_for(model_id):
    """Ordered matrix column names for a compiled model's native NumPy adapter."""
    return tuple(_call(_native.array_parameters, model_id))

def run_array(model_id, parameters, *, shared_axes=False, fixed=None):
    """Compute real (batch,parameter) matrix using the shared typed Rust collector.

    Default layout preserves dedicated *_batch interfaces: wavelength axes are
    shared, scalar outputs are (batch,), other outputs are (batch,sample).
    shared_axes=True moves ALL fixed coordinate arrays into 'coordinates' as
    one-dimensional arrays, without repeating them for each simulation.
    Result arrays are float64; integer diagnostic counts remain int64. Empty,
    strided and non-native-endian inputs are supported. No implicit parameter
    broadcasting. An invalid row aborts the whole call and identifies the row.
    """
    if not isinstance(shared_axes,(bool,np.bool_)):
        raise ModelError('invalid_parameter','shared_axes must be boolean')
    array=real_array(parameters,ndim=2,name='parameters')
    if fixed is not None and not isinstance(fixed,Mapping):
        raise ModelError('invalid_parameter','fixed must be a mapping of model selectors')
    out=_call(_native.model_batch_native,model_id,array,bool(shared_axes),_dumps(dict(fixed or {})))
    out['manifests']=json.loads(out.pop('manifests_json'))
    return out

def iter_array(model_id, parameters, *, chunk_size=256, shared_axes=False, fixed=None):
    """Yield bounded result chunks with row_offset, preserving input row order.

    The parameter matrix itself is retained; output memory is bounded by a chunk
    if the caller consumes/discards chunks. Earlier yielded chunks remain valid
    if a later chunk fails; error row indices refer to the FULL input matrix.
    Empty matrices yield one correctly shaped empty result. Holding all yielded
    chunks naturally retains all outputs. No internal parallel thread pool.
    """
    if isinstance(chunk_size,(bool,np.bool_)):
        raise ModelError('invalid_parameter','chunk_size must be a positive integer')
    try:size=operator.index(chunk_size)
    except TypeError as e:raise ModelError('invalid_parameter','chunk_size must be a positive integer') from e
    if size<=0:raise ModelError('invalid_parameter','chunk_size must be a positive integer')
    array=real_array(parameters,ndim=2,name='parameters')
    for offset in range(0,max(1,len(array)),size):
        try:out=run_array(model_id,array[offset:offset+size],shared_axes=shared_axes,fixed=fixed)
        except ModelError as error:
            message=re.sub(r'batch row (\d+)',lambda m:'batch row '+str(offset+int(m.group(1))),error.message)
            raise ModelError(error.kind,message) from error
        out['row_offset']=offset
        yield out

def parameter_grid(model_id, varying, *, base=None, max_samples=1_000_000):
    """Small Cartesian parameter grid in parameters_for() column order.

    Unspecified fields use current model defaults, then base overrides. This is
    table construction, not inversion, sampling inference or model coupling.
    Run with run_array or iter_array. max_samples guards accidental huge grids.
    """
    if not isinstance(varying,Mapping) or base is not None and not isinstance(base,Mapping):
        raise ModelError('invalid_parameter','varying and base must be mappings')
    if isinstance(max_samples,(bool,np.bool_)):
        raise ModelError('invalid_parameter','max_samples must be a positive integer')
    try:limit=operator.index(max_samples)
    except TypeError as e:raise ModelError('invalid_parameter','max_samples must be a positive integer') from e
    if limit<=0:raise ModelError('invalid_parameter','max_samples must be positive')
    columns=parameters_for(model_id);unknown=(set(varying)|set(base or {}))-set(columns)
    if unknown:raise ModelError('invalid_parameter','unknown parameters: '+', '.join(sorted(unknown)))
    schema=describe(model_id)['input_schema']['properties']
    defaults={key:schema[key]['default'] for key in columns};defaults.update(base or {})
    for key,value in list(defaults.items()):
        if isinstance(value,(list,tuple)):
            if len(value)!=1:raise ModelError("invalid_parameter","numeric parameter grids require single-band coefficients; use run_batch for multiband mappings")
            defaults[key]=value[0]
    choices=[real_array(varying[key],ndim=1,name=key) if key in varying else real_array([defaults[key]],ndim=1,name=key) for key in columns]
    count=1
    for choice in choices:count*=len(choice)
    if count>limit:raise ModelError('invalid_parameter',f'grid has {count} samples, exceeding max_samples={limit}')
    return np.asarray(list(itertools.product(*choices)),dtype=np.float64).reshape(count,len(columns))
