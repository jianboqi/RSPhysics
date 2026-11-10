# RSPhysics

A scientific computing library for remote sensing physics, with Python/NumPy
interfaces to physical models and numerical tools.

## Installation

```sh
pip install rsphysics
```

Python 3.11 or later is required. NumPy is installed automatically. Compatible
prebuilt wheels run without a Rust compiler on Windows, Linux and macOS.

## Usage

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)
reflectance = leaf['result']['reflectance']
```

See [Documentation](https://github.com/jianboqi/RSPhysics/blob/main/docs/README.md)
for basic usage, physical tools, and model guides, or
[Build from source](https://github.com/jianboqi/RSPhysics/blob/main/docs/build-from-source.md).

For plotting examples, install the optional dependencies with
`pip install "rsphysics[examples]"`.

## License

Project-original code is Apache-2.0. Included models and resources retain
their component-specific terms; see the packaged third-party notices.
