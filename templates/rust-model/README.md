# Rust model template

This is a compilable engineering template without a scientific solver. It does
not enter the official scientific catalog.

1. Copy it to `crates/models/<domain>/<name>`, rename the crate, and add it to the root workspace.
2. Set the ID, scientific/implementation versions, authors, references, license, and domain in `model.toml`.
3. Define typed inputs/outputs using shared spectral axes and units; derive boundary schemas with Serde/Schemars.
4. Implement `validate_input`, `run`, and `validate_output`. Direct Rust calls must also enforce domains.
5. Retain traceable original-reference cases before writing the solver; check shapes and finite values.
6. Record the permissions for source, data, and dependencies in `LICENSE-ORIGIN.md`.
7. Run `python tooling/check-models/check_models.py` and `cargo test --workspace`.

See existing model crates for Python interfaces and numerical reference tests.
Declare only the capabilities that the contribution implements and validates.
