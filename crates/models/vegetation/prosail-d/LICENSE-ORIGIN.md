# PROSAIL-D forward solver and soil provenance

Forward canopy equations adapted from Jean-Baptiste Féret's official `prosail`
v3.1.1 commit `46549958295b605ac1e211a693d4b2287c3a5a13`:
https://github.com/jbferet/prosail/tree/46549958295b605ac1e211a693d4b2287c3a5a13 .
This fixed DESCRIPTION declares `MIT + file LICENSE`; LICENSE identifies
Copyright 2026 Jean-Baptiste Féret. `volscatt` credits Wout Verhoef, April 2001,
for CROMA. Newer releases may declare GPL; they were not used for this port.
Only the forward `fourSAIL`, Campbell, volscatt and jfunc1/2/3 equations are
adapted. No upstream package, GIS or inversion dependencies are distributed.

The leaf solver delegates to RSPhysics PROSPECT-D, whose distinct original
v1.7.0 source, coefficients and notices remain in the sibling model crate.
This is one scientific model, not generic runtime model orchestration.

`data/soil.tsv` preserves all bytes of the author's
`data-raw/dataSpec_SOIL_ATM.txt`, SHA-256
`7754d3ddb5d3b1f6c3c23cfd0a7c09d7a75be83a1da75c10ba04e71461c30b74`.
Only dry/wet soil columns are used. Direct/diffuse light columns are not used.
No separate data license was found. Required runtime data is retained;
temporary original sources and R/WebR installations are disposable.
Project-level redistribution/license review is not claimed complete.

Changes (2026-11-09): Rust scalar forward solver, reusable geometry,
flat typed input, embedded data hash checks, explicit BRF/hemispherical
reflectance semantics, native/Python reference tests. Original 13 Campbell
bins and 20-step hotspot integration retained. BRF is never clipped to one.
LAI=0 uses the analytical Lambertian bare-soil limit; the pinned R fourSAIL
routine produces undefined arithmetic there. The reference fixture clearly
labels that case separately from 24 actual author-R runs. Conservative leaf
scattering (water=dry matter=0 for positive LAI) is excluded in this version.
Numerically coincident hotspot integration endpoints use their continuous
limit, and roundoff-sized negative geometric radicands are limited to zero.

Complete source URLs and SHA-256 values are retained in
`benchmarks/reference/prosail-d/upstream-provenance.json`.
