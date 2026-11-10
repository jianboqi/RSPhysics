# Installation

RSPhysics requires Python 3.11 or later and NumPy.

## PyPI

The package has not yet been published to PyPI. Once available, install it with:

```sh
pip install rsphysics
```

## Prebuilt wheels

Download `rsphysics-wheels` from the **Artifacts** section of a successful
[Wheels run](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)
and extract it. GitHub requires signing in to download workflow artifacts.

From the directory containing the extracted Wheels:

```sh
python -m pip install numpy
python -m pip install --no-index --no-deps --find-links . rsphysics
```

Pip selects the Wheel matching the current Python interpreter and platform.
Prebuilt packages cover CPython 3.11–3.15 on Windows x86_64, Linux x86_64/ARM64,
and macOS Intel/Apple Silicon. No Rust compiler is needed for a compatible Wheel.

## Check the installation

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d()
print(leaf['result']['reflectance'].shape)  # (2101,)
```

Continue with [Basic usage](usage.md), or [build from source](build-from-source.md).
