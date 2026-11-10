# PROSPECT-D

One Rust solver serves native CLI and Python. The fixed grid is 400–2500 nm
at 1 nm spacing; outputs are leaf reflectance and transmittance. Parameters are
in `src/parameters.rs`; units/domains are in `model.toml` and the Python guide.

The D parameterization (PROT=CBC=0) uses the author's pinned `prospect` R v1.7.0.
Reference functions, inputs, and 32 spectra are documented in
`benchmarks/reference/prospect-d` at the repository root. Rust compares R/T at
every wavelength and checks finite values, shape, and energy.
`reference-validated` applies only to this implementation version and suite.

```rust
use rsphysics_prospect_d::{Input, Model};
fn main() -> Result<(), rsphysics_model_api::ModelError> {
    let model = Model::new()?;
    let result = model.simulate(&Input { chlorophyll: 45.0, ..Input::default() })?;
    println!("{} spectral samples", result.reflectance.len());
    Ok(())
}
```

Coefficients are embedded with `include_str!`; construction checks their SHA-256.
No download or original R installation is required. `data/coefficients.tsv` is a
runtime dependency and must not be cleaned as reference cache. Authors, license,
data source, and modifications are recorded in `LICENSE-ORIGIN.md`.
