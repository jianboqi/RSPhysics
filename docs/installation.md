# Installation

RSPhysics requires Python 3.11 or later and NumPy.

## PyPI

```sh
python -m pip install rsphysics
```

Compatible binary wheels install without a Rust compiler. To include Matplotlib
for the plotting examples:

```sh
python -m pip install "rsphysics[examples]"
```

Prebuilt packages cover CPython 3.11–3.15 on Windows x86_64, Linux x86_64/ARM64,
and macOS Intel/Apple Silicon. Other platforms can [build from source](build-from-source.md).

## Prebuilt wheels

For a development build, download `rsphysics-distributions` from the **Artifacts** section of a successful
[Wheels run](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)
and extract it. GitHub requires signing in to download workflow artifacts.

From the extracted `dist` directory:

```sh
python -m pip install numpy
python -m pip install --no-index --no-deps --find-links . rsphysics
```

Pip selects the wheel matching the current Python interpreter and platform.

## Check the installation

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d()
print(leaf['result']['reflectance'].shape)  # (2101,)
```

Continue with [Basic usage](usage.md), or [build from source](build-from-source.md).
