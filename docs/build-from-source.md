# Build from source

Install Python 3.11+, Rust through rustup, a native C/C++ build toolchain, and
GNU Fortran 10 or newer for the bundled 6SV2.1 kernel.
The repository's `rust-toolchain.toml` selects the required Rust version.

```sh
git clone https://github.com/jianboqi/RSPhysics.git
cd RSPhysics
python -m pip install "maturin==1.16.0" numpy
python -m maturin build --manifest-path bindings/python/Cargo.toml --release --locked --auditwheel repair --out dist --interpreter python
python -m pip install --no-index --no-deps --find-links dist rsphysics
```

On Windows, use the Visual Studio C++ Build Tools. On macOS, install the Xcode
Command Line Tools. On Linux, install the compiler/linker packages supplied by
your distribution.

Set `FC` to the GNU Fortran executable if it is not named `gfortran` on `PATH`.
On Windows, GNU Fortran builds the embedded DLL while Visual Studio builds the
Rust/Python extension. The resulting wheel includes the required Fortran runtime;
users installing a wheel do not need a compiler or a separate 6S program.

## Run the checks

```sh
cargo test --workspace --locked
python -m pip install pytest
python -m pytest bindings/python/tests -q
```

See [Basic usage](usage.md) for model calls and NumPy batches.
