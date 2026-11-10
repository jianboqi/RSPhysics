# Independent microwave references

Two independent forward models, each with 32 deterministic cases and 81-point
zenith sensitivity sweeps. `tau-omega` uses separate soil/canopy temperatures,
nadir optical depth and directly supplied effective soil power reflectivity.
`water-cloud` fixes V1=V2=VWC and directly supplied bare-soil sigma0 in linear power.
All non-angle inputs, including soil response, stay fixed in each sweep.

τ–ω reference: NASA SMAP ATBD Revision B (September 14, 2015), §2.1 Eq.1,
independently evaluated using mpmath 1.3.0 at 80 decimal digits. This is an official
equation reference, **not** an executed original SMAP operational code oracle.

WCM reference: the published Attema–Ulaby water-cloud formulation and its
VWC parameterization, independently evaluated at 80 digits. In addition, the
developer's original `WaterCloudCanopy` class from SenSE commit
`f9bde397c32f8759f7a410cf3d55febac9034aef` is executed unchanged (AST-selected class
with original NumPy methods/decorator; other models are not imported).
Its vegetation contribution and two-way transmission are compared at the selected
angle and all sweep angles. It is not claimed to be the original 1978 author code.
No SenSE soil model or soil moisture conversion enters RSPhysics.

Boundary cases include bare soil, transparent and opaque canopy, weak optical
depth (1e-15), zero signal, signals above one, extreme angles/temperatures and
seeded random parameters. Rust uses exp_m1 for small canopy attenuation. The
agreement tolerance is 2e-12 absolute + 2e-12 relative, including the original
NumPy canopy implementation. Zero WCM output remains zero; dB is presentation
only and zero has no finite dB value. These are implementation tests, not empirical
validation of illustrative coefficients or retrieval accuracy.

## Regeneration

```sh
python benchmarks/reference/microwave/prepare.py
python -m pip install mpmath==1.3.0 numpy
python benchmarks/reference/microwave/generate.py
cargo test --workspace --locked
python -m pytest bindings/python/tests/test_microwave.py -q
python benchmarks/reference/microwave/report.py
python tooling/reference-runner/clean.py --model tau-omega --remove
python tooling/reference-runner/clean.py --model water-cloud --remove
```

Downloads go only to `.reference-cache/{tau-omega,water-cloud}/upstream`. Published
expected values, input cases, source hashes and validation reports remain under
`benchmarks/reference/{tau-omega,water-cloud}` after cleanup. Daily tests need
neither the downloads nor mpmath/SenSE. Official documents and GPL-declared SenSE
source are test-only, not linked, redistributed or bundled in the wheel/sdist.
Private WASM/Worker and web regressions live exclusively in RSPhysics-app.
