# Building and validating 6SV2.1

The production interfaces call libraries compiled from the preserved Fortran source.
No Rust numerical replacement, precision promotion, or fast-math is used.

## Direct native library

GNU Fortran 10+ and Python are required to build. Installing a prebuilt wheel
requires neither a compiler nor a separate 6S program. Both original entry
points are embedded in the extension as direct C ABI libraries.

```sh
python tooling/sixs/build.py verify
python tooling/sixs/library.py native --output target/sixs-library
python tooling/sixs/validate_library.py --library-dir target/sixs-library
```

Use `FC` or `--fc PATH` to select GNU Fortran. Native Python builds invoke the
same recipe automatically. The build implementation resides in the source-only
Cargo component under `components/6sv2.1/build_tools`, so source archives are
self-contained. The CLI files here are thin entry points.

`library-build-info.json` records compiler flags, original archive hash,
compatibility corrections, and state-reset storage. Source files in `upstream/`
are verified before each build. Windows bundles a statically linked runtime;
Linux bundles shared GNU runtimes. Required license texts accompany the package.

## Original executables for reference validation

Original executables are retained only as numerical oracles and performance
baselines. Production Python and browser execution does not launch them.

```sh
python tooling/sixs/build.py verify
python tooling/sixs/build.py native --output target/sixs-native
python tooling/sixs/build.py native --original --output target/sixs-original
python tooling/sixs/generate_reference.py --corrected target/sixs-native
```

Use `--fc PATH` or `FC` to select the compiler executable. Compiler warnings are
retained in per-source `.log` files. `build-info.json` records the source archive
hash, compiler, flags, programs, and applied compatibility corrections.
See [compatibility corrections](../../components/6sv2.1/COMPATIBILITY.md).

## WebAssembly

The pinned image supplies Flang 23.1.4, its WASM Fortran runtime, and Emscripten
5.0.7. The same Fortran source and original object list are used for both targets.

```sh
docker build -f tooling/sixs/Dockerfile.library-wasm -t rsphysics-sixs-library-wasm .
docker run --rm --mount type=bind,source="$PWD",target=/workspace rsphysics-sixs-library-wasm
```

The product exports `rsp_fortran_run`, with Emscripten's `ccall` and `FS` helpers.
Each calculation starts a fresh module in a Web Worker. Configuration is validated
by the public Rust adapter; original input/output files use the module's MEMFS.

The separate executable-WASM smoke baseline remains available:

```sh
docker build -f tooling/sixs/Dockerfile.wasm -t rsphysics-sixs-wasm .
docker run --rm --mount type=bind,source="$PWD",target=/workspace rsphysics-sixs-wasm
node tooling/sixs/run-wasm.mjs target/sixs-wasm benchmarks/reference/6sv2.1/inputs.json target/sixs-wasm-results.json
python tooling/sixs/verify.py target/sixs-original target/sixs-native --wasm-results target/sixs-wasm-results.json
```

The Node smoke runner preserves the reference program's initialization and STOP
behavior; it is not the product's direct-library transport.

## Performance

```sh
python tooling/sixs/benchmark.py target/sixs-original --output target/sixs-performance.json
```

The benchmark measures process launch, computation, and report capture, with
warm-up runs and isolated working directories. It also compares serial and
parallel execution of identical independent tasks. Container startup is outside
the timing region. Results depend on inputs, compiler, hardware, and concurrent
load; a batch throughput increase is not a single-calculation speedup.

Builds and disposable diagnostic copies are under `target/`. Downloaded source
archives and portable toolchains are under `.reference-cache/6s-intake/`.
