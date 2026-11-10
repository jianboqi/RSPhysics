# Contributing to RSPhysics

RSPhysics is under active development. Read the
[README](README.md), [Model API RFC](docs/architecture/model-api-v0.1.md), and
[scientific conventions](docs/architecture/scientific-conventions.md) first.
Use English for maintained core documentation, contribution descriptions, and
new developer-facing explanations. Website localization is tracked separately.

## 📘 Before submitting

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python tooling/check-models/check_models.py
```

Start a new model from `templates/rust-model`. Define typed inputs/outputs,
units, domains, original sources/licenses, reproducible references, and explicit
scientific, implementation, and data versions. Do not label an incomplete model
or engineering fixture as scientifically validated.

Catalog generation is not implemented. Add an official model explicitly to
`rsphysics-catalog`, its Cargo dependencies, and the single catalog inventory.
Consistency checks reject inventory-only additions. Run the local checks before
submitting a pull request.

Numerical changes require a reference error report. Refactoring must not silently
change scientific behavior. Original code is Apache-2.0; preserve third-party code
and data terms. Additional Apache permission for SMAC is pending. State the source
and permission scope of contributions; see [licensing](docs/licensing.md).

## Continuous integration

The Checks workflow validates Rust code, catalog/resource integrity, and source
archive installation. The Wheels workflow builds platform-specific Python
packages and runs the numerical suite against each installed Wheel. It uploads
tested artifacts to the workflow run; see [installation](docs/installation.md).
Neither workflow requires MATLAB/R, disposable reference caches, or website
source. Publishing to PyPI is a separate release step.
