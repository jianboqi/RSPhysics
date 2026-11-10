# Public Python bindings and product boundaries

The public PyO3/maturin/NumPy bindings live in `python/`. They provide generic and
model-specific calls, native NumPy batches, and locally verified Wheel/source
installation for the compiled model catalog. The package has not been uploaded to PyPI.

Product wasm-bindgen bindings, Worker, website, and future Tauri integration live
in the sibling `RSPhysics-app` project. Website source is intended to be public;
product validation remains separate from this library. See
[ADR 0004](../docs/decisions/0004-apache-project-license.md) for current licensing.

All interfaces reuse the Rust scientific solvers. The product owns WASM builds,
browser execution, and browser reference checks. The public library does not
require WASM CI or browser support for model acceptance. Other users may evaluate
and compile public Rust code to WASM subject to the applicable licenses.
