# Building 6SV2.1

The two original programs are compiled from the preserved Fortran source.
No Rust numerical replacement, precision promotion, or fast-math is used.

## Native

A GNU Fortran compiler is required to build, but not to run the resulting
executables. GNU Fortran 13.3 (Linux) and 16.2 (Windows) have been tested.

```sh
python tooling/sixs/build.py verify
python tooling/sixs/build.py native --output target/sixs-native
python tooling/sixs/build.py native --original --output target/sixs-original
```

Use `--fc PATH` or `FC` to select the compiler executable. Compiler warnings are
retained in per-source `.log` files. `build-info.json` records the source archive
hash, compiler, flags, programs, and applied compatibility corrections.
See [compatibility corrections](../../components/6sv2.1/COMPATIBILITY.md).

## WebAssembly

The pinned image supplies Flang 23.1.4, its WASM Fortran runtime, and Emscripten
5.0.7. The same Fortran source and original object list are used for both targets.

```sh
docker build -f tooling/sixs/Dockerfile.wasm -t rsphysics-sixs-wasm .
docker run --rm --mount type=bind,source="$PWD",target=/workspace rsphysics-sixs-wasm
node tooling/sixs/run-wasm.mjs target/sixs-wasm benchmarks/reference/6sv2.1/inputs.json target/sixs-wasm-results.json
python tooling/sixs/verify.py target/sixs-original target/sixs-native --wasm-results target/sixs-wasm-results.json
```

The Node runner creates a fresh module per calculation, preserving the original
program's initialization and STOP behavior. Browser integration uses a worker
and must also preserve original auxiliary-file input and output.

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
