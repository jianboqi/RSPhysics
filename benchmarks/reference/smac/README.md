# SMAC original reference

Reference code: unmodified `smac.py` from ESA Sen2Like commit
`6dd3f0da82a7461b03a2914df56d305070e50d6d`, authored by Olivier Hagolle / CNES.
Its SHA-256 is retained in `expected.json`. The source is disposable, located
under `.reference-cache/smac/upstream` with all original coefficient files.

`generate.py` imports the original Python 3-compatible implementation. A Python
return-frame profile reads intermediate results without editing any equations.
Original verbose console output is captured, not changed. No Rust or installed
RSPhysics code is imported. NumPy is the only nonstandard reference dependency.

839 cases: default, both directions for all 367 coefficient sets, 96 seeded
random scenarios and 8 geometry/AOT/zero-gas boundaries. Two additional complete
21-point AOT curves hold all other inputs fixed. Eight physical outputs per
case are compared. Absolute tolerance 2e-11 plus relative tolerance 2e-10 covers
platform-dependent cancellation in legacy aerosol expressions, without clipping
negative results or hiding strongly absorbing bands.

Rust/Python/private Chromium WASM compare against the same retained fixture;
round-trip and resource identity tests supplement it. This is implementation
agreement, not observational validation or a guarantee of SMAC retrieval accuracy.

Regeneration: fetch, prepare, generate as documented in `docs/smac.md`.
Offline integrity: `python tooling/smac-data/verify.py`.
Error report: `.venv/Scripts/python.exe benchmarks/reference/smac/report.py`.
