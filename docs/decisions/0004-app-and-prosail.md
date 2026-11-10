# ADR 0004: Separate RSPhysics-app and PROSAIL-D

Status: adopted and locally verified at the owner's request, 2026-11-09.

Create sibling `RSPhysics-app` (singular), initially as a private product starting
point. First establish real WASM, Worker, and website template, then add the
second independent model, PROSAIL-D. Public library acceptance remains
native/Python/pip; public model metadata remains `wasm=false`, with no public
WASM CI. The product owns its lockfile, capability declarations, and browser evidence.
Later source visibility/licensing decisions are separate ADRs.

PROSAIL-D fixes PROSPECT-D + author 4SAIL, Campbell 13 bins, and direct illumination.
Register `vegetation.prosail-d` as an independent model that internally reuses the
PROSPECT-D solver. This adds no generic model graph, chaining, inversion, or coupling.
Product parameter forms, legends, and exports follow model outputs. BRF and
directional–hemispherical reflectance are not leaf R/T energy terms.

Acquire author prospect v1.7.0 and the MIT-declared forward code from prosail
v3.1.1 commit `46549958295b605ac1e211a693d4b2287c3a5a13`. Original downloads/R runtimes
use `.reference-cache/{prospect-d,prosail-d}` and are cleaned after verification.
Retain required coefficients, inputs, offline references, source hashes, and reports.

Neither local project has a remote commit. Use sibling Cargo path dependencies,
record actual core fingerprints and null Git revisions, then pin a full public
SHA after remote setup. Website/PyPI are unpublished and Tauri is unimplemented;
cross-platform and distribution-rights work remain release conditions.
