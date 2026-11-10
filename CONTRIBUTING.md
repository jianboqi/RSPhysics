# Contributing

Contributions of models, scientific examples, documentation, and numerical
improvements are welcome. Use English for documentation and contribution descriptions.

For a model contribution, include its physical assumptions, parameter/output units,
scientific authors and references, appropriate source/data permissions, and
numerical validation. Start with [the model template](templates/rust-model/README.md)
and use existing model crates as examples.

Run the relevant checks before submitting a pull request:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
python tooling/check-models/check_models.py
python -m pytest bindings/python/tests -q
```

Explain changes to numerical behavior and provide reference comparisons when
equations or numerical methods change. Preserve scientific attribution and
applicable [component licenses](LICENSES.md).
