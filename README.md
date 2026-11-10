# RSPhysics

**A scientific computing library for remote sensing physics.**

[![Checks](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml)
[![Wheels](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)

RSPhysics brings remote sensing physical models into a shared scientific
computing library. It makes models scattered across research software easier to
use, compare, and extend through consistent interfaces and physical conventions.
Rust implementations and Python/NumPy interfaces support research, education,
and application development.

## Installation

PyPI installation, available after the first package release:

```sh
pip install rsphysics
```

For current builds, install a [prebuilt wheel](docs/installation.md).

## Quick start

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)
wavelengths = leaf["result"]["wavelengths_nm"]
reflectance = leaf["result"]["reflectance"]
```

## Documentation

[Documentation](docs/README.md) covers basic usage, physical tools, and individual
models with parameter definitions, examples, authors, and references.

## Build from source

See [Build from source](docs/build-from-source.md).

## License

Project-original code is licensed under [Apache-2.0](LICENSE). Included models
and resources retain their applicable [component licenses](LICENSES.md).
