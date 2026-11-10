"""Binding validation and buffer conversion, without scientific calculations."""
import numpy as np
from ._api import ModelError

def real_array(value, *, ndim=None, name='values'):
    try:
        array = np.asarray(value)
    except (TypeError, ValueError) as error:
        raise ModelError('invalid_parameter', f'{name} must be a rectangular real numeric array') from error
    if array.dtype.kind not in 'fiu' or (ndim is not None and array.ndim != ndim):
        raise ModelError('invalid_parameter', f'{name} must be a real numeric array' + (f' with {ndim} dimensions' if ndim is not None else ''))
    return np.asarray(array, dtype=np.float64)

def axis(value, name='wavelengths_nm'):
    array = real_array(value, ndim=1, name=name)
    if not array.size:
        raise ModelError('invalid_parameter', f'{name} cannot be empty')
    return array

def broadcast(*arrays):
    try:
        return np.broadcast_arrays(*arrays)
    except ValueError as error:
        raise ModelError('invalid_parameter', 'input array shapes cannot be broadcast together') from error

def complex_array(value, *, name='index'):
    try:
        array=np.asarray(value)
    except (TypeError,ValueError) as error:
        raise ModelError('invalid_parameter',f'{name} must be a rectangular real or complex numeric array') from error
    if array.dtype.kind not in 'fciu':
        raise ModelError('invalid_parameter',f'{name} must be a real or complex numeric array')
    return np.asarray(array,dtype=np.complex128)
