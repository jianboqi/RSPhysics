# ADR 0003: PROSPECT-D reference and disposable environment

Date: 2026-11-09. Status: adopted.

Acquire official materials independently. The first agreement reference is
PROSPECT-D in the author's pinned prospect v1.7.0. Do not mix source, data, or
licenses from the current main branch.

Downloads, coefficient copies, Node/WebR/R environment, and scratch outputs use
ignored `.reference-cache/prospect-d/`. Remove them with
`tooling/reference-runner/clean.py --remove`. This environment executes official R;
it is not a public WASM product or compatibility promise.

Execute the original PROSPECT/calctav functions independently with complete D
inputs, bypassing only package-dependent input normalization. Function bodies
remain unchanged; R expint does not call Rust. Generate 32×2,101-band expected
spectra before writing the Rust solver.

Retain provenance/file hashes, generators, inputs, expected spectra, and reports
for offline regression. The required 226 KB runtime coefficient table remains in
the crate with byte-hash verification; it is not disposable cache.

Predetermined tolerance is `abs_error <= 2e-10 + 2e-8 * abs(reference)`: absolute
error covers weak signals; relative error accommodates near-zero-absorption
Stokes conditioning. Expected outputs are not generated backwards or changed
to pass tests, and clipping does not hide errors. Tolerance describes numerical
agreement, not leaf observational accuracy. Debug/Release must both pass, with
their actual errors reported separately.

Local acceptance covers Windows/CPython 3.13. Configured public CI will cover
other systems/Python versions after remote setup. Provenance is not release permission.
