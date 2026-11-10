# Installation

RSPhysics provides Python bindings to its Rust scientific kernels. NumPy is the
Python runtime dependency; a compatible prebuilt Wheel does not require a Rust
compiler on the user's machine.

## Prebuilt wheels

The [Wheels workflow](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)
builds and tests CPython-specific Wheels for:

| Platform | Architecture | Python versions |
|---|---|---|
| Linux, manylinux2014 | x86_64, ARM64 | CPython 3.11–3.15 |
| Windows | x86_64 | CPython 3.11–3.15 |
| macOS 11+ | Intel, Apple Silicon | CPython 3.11–3.15 |

After a successful workflow run, download the `rsphysics-wheels` artifact from
the run's **Artifacts** section and extract it. GitHub requires signing in to
download workflow artifacts. The bundle includes `SHA256SUMS` for its Wheels.

Install from the extracted directory; pip selects the matching Wheel:

```sh
python -m pip install numpy
python -m pip install --no-index --no-deps --find-links ./wheelhouse rsphysics
```

Adjust `./wheelhouse` to the directory containing the extracted `.whl` files.
Use the Python interpreter/environment in which you intend to run RSPhysics.
PyPy, free-threaded Python, Windows ARM64, and musllinux are not currently in
the build matrix.

## Build from source

Source builds require the pinned Rust toolchain, a native C/C++ linker, and
Python 3.11 or later. From the repository root:

```sh
python -m pip install "maturin==1.16.0" numpy
python -m maturin build --manifest-path bindings/python/Cargo.toml --release --locked --out dist --interpreter python
python -m pip install --no-index --no-deps --find-links dist rsphysics
```

The [Checks workflow](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml)
also builds an sdist and installs it in an independent build environment. Runtime
and regression checks do not require original MATLAB/R installations, disposable
reference caches, or website source.

## Check the installation

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)
print(leaf['result']['reflectance'].shape)  # (2101,)
```

See the [Python guide](../bindings/python/README.md) for model calls and batches,
and [license details](licensing.md) for component-specific source/data terms.
GitHub Wheel artifacts are development builds; this workflow does not publish
to PyPI or create a tagged release automatically.
