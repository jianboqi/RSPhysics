# RSPhysics

A scientific computing library for remote sensing physics, with Python/NumPy
interfaces to physical models and numerical tools.

## Installation

After the first PyPI release:

```sh
pip install rsphysics
```

Current packages are available as [prebuilt wheels](https://github.com/jianboqi/RSPhysics/blob/main/docs/installation.md).

## Usage

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)
reflectance = leaf['result']['reflectance']
```

See [Documentation](https://github.com/jianboqi/RSPhysics/blob/main/docs/README.md)
for basic usage, physical tools, and model guides, or
[Build from source](https://github.com/jianboqi/RSPhysics/blob/main/docs/build-from-source.md).

## License

Project-original code is Apache-2.0. Included models and resources retain
their component-specific terms; see the packaged third-party notices.
