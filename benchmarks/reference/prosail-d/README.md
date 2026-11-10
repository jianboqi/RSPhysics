# PROSAIL-D author reference

Configuration: PROSPECT-D + 4SAIL, original 13-bin Campbell distribution,
direct solar illumination, original dry/wet soil blend. Fixed author source:
`jbferet/prosail` v3.1.1, commit `46549958295b605ac1e211a693d4b2287c3a5a13`.
Leaf source and coefficients use the separately recorded `prospect` v1.7.0.
All scientific function bodies are executed unchanged in R 4.6.0 with
expint 0.2.1 via WebR 0.6.0. Only the leaf package-dependent input normalizer
is bypassed using a fully specified D parameter frame (PROT=CBC=0).

`expected.json` contains 24 actual original R runs and one **separately labelled
analytical bare-soil limit**. The original fourSAIL has undefined hotspot
arithmetic at LAI=0, so this case is not presented as author-R output.
105,050 output values are checked at 2101 wavelengths, with tolerance
`abs <= 2e-10 + 2e-8 * abs(reference)`. BRF is original `rsot`; directional-
hemispherical reflectance is `rsdt`. These are not R/T and are not added for
an energy check. BRF can exceed one. No diffuse illumination mixing is used.

Rebuild (requires network; no installed desktop R required):

```sh
python tooling/reference-runner/prepare.py
python benchmarks/reference/prosail-d/fetch_upstream.py
python tooling/reference-runner/make_prosail_cases.py
node tooling/reference-runner/generate-prosail.mjs
python benchmarks/reference/prosail-d/verify_upstream.py --require-cache
cargo test -p rsphysics-prosail-d -- --nocapture
python tooling/reference-runner/report.py --model prosail-d
```

Original downloads live only in `.reference-cache/prosail-d`; the shared R
runtime is in `.reference-cache/prospect-d`. `clean.py --remove` verifies and
removes both model caches. It preserves required runtime soil/leaf tables,
inputs, expected spectra, provenance and validation reports. Normal Cargo,
Python and private browser regression tests use these offline artifacts.

Native Debug maximum absolute error: 4.662936703e-15 on this Windows machine.
Installed Python Release metrics are in `validation-report.json`. Browser
metrics belong to the independent private RSPhysics-app project, whose
public-model capabilities remain unchanged (`wasm=false` here).
