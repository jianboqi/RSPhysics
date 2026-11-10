"""Remote sensing physics with one shared Rust scientific implementation."""
from ._native import __version__
from ._api import ModelError, describe, list_models, run, run_batch
from ._arrays import run_array, iter_array, parameters_for, parameter_grid
from . import vegetation, surface, microwave, spectral, radiometry, sensors, optics, atmosphere, ecosystem, soil, thermal, sixs

__all__ = ["__version__", "ModelError", "describe", "list_models", "run", "run_batch", "run_array", "iter_array", "parameters_for", "parameter_grid", "vegetation", "surface", "microwave", "spectral", "radiometry", "sensors", "optics", "atmosphere", "ecosystem", "soil", "thermal", "sixs"]
