# Implementation and validation history

## Supplied L-MEB and GORT-GPP forward models · 2026-11-10

Added `microwave.lmeb` (supplied 1.41 GHz soil-forward path) and
`ecosystem.gort-gpp` (Main4PSN mvp=1, flagSeason=0 pointwise photosynthesis).
Each has 32 original MATLAB R2026a reference cases, typed Rust, Runtime/CLI,
dedicated Python/NumPy interfaces, and separate product WASM integration.
GORT-GPP explicitly exposes net assimilation; annual station processing, ET,
other modes, and generic coupling are outside this adaptation. L-MEB excludes
water/high-frequency/inversion paths. Model guides document exact scope.

Local checks passed 45 Rust tests, fmt/Clippy, nine manifests/eight scientific
models, and 1,190 installed-Wheel Python tests including optional local ESA data
(ordinary offline CI skips that one optional test). A Wheel independently rebuilt
offline from the sdist also passed all 1,190 tests. Against original MATLAB,
L-MEB checked 5,248 values with maximum Python/WASM absolute error 1.70530e-13;
GORT-GPP checked 6,080 values with maximum Python error 4.54747e-13 and WASM
error 2.27374e-13. These are implementation comparisons, not observational validation.

The product passed 22 development and eight production browser tests, including
introductions, forms, response plots, exports, saved reruns, and responsive layout.
Long mobile parameter labels received a layout fix and targeted/production rechecks.
Original source caches were removed after verification; all 1,190 Python tests
passed again without them. Fixtures and provenance remain. Supplied-code
redistribution permission is unresolved; component/package records retain that
status. No remote publication or deployment was performed.

## 📘 2026-11-10 · Original-code Apache license and rights contact

Original infrastructure, independently original models, and website code now use Apache-2.0, with complete LICENSE, NOTICE,
a rights-contact statement, and a website license entry. Existing MIT notices and the GPLv3 notice in the pinned SMAC reference
remain; all six models stay included. Additional Apache permission for SMAC awaits rights-holder confirmation. An inquiry
exists only as an unsent local draft. A technical comparison of pinned author/Sen2Like sources records highly similar core
computational statements; it does not determine ownership or change the recorded reference source.

This round passed fmt, 41 Rust tests, and consistency checks for seven manifests/six scientific models. The rebuilt/reinstalled
Wheel passed 1,182 Python tests; website type checks, WASM/static builds, and six production-browser tests also passed.
Wheel/sdist retain Apache/MIT/GPL provenance and rights statements. Python metadata accurately declares
Apache-2.0 AND MIT AND GPL-3.0-only. Temporary source-comparison downloads were removed.
No remote hosting or public release has occurred. The combined package is not an authorized Apache-only distribution.

## 📘 2026-11-10 · Independent SMAC integration

The sixth scientific model, `atmosphere.smac`, adds pinned references, public Rust, Runtime/CLI, dedicated Python and NumPy batch interfaces. The separate RSPhysics-app consumes the same kernel through WASM, with category/detail/run pages and a 21-point AOT sensitivity curve. String selections for both directions survive parameters, exports, and history reruns. No generic model coupling was introduced.

367 coefficient entries derive from 331 pinned original files. One 82,152-byte zlib archive preserves exact source text, index, URLs, and SHA-256, reducing 219,290 source bytes by about 62.5%. Coverage includes Sentinel-2A/B/C, Landsat-4/5/7/8, legacy MODIS, and other sensors in the author's repository. No Landsat-9 substitution or relabeling of legacy MODIS as a newer Terra/Aqua response version is performed.

Local checks passed 41 Rust and 1,182 Python tests, including the optional local ESA source-table test; ordinary offline CI runs 1,181 and skips one. Across 839 original SMAC cases and two complete AOT curves, 6,754 installed-Wheel values had maximum absolute difference 1.77636e-15. Seven manifests/six scientific models, fmt, Clippy, and offline coefficient checks passed. Product tests passed 18 development and six static-production browser cases; final form-order/mobile changes received targeted rechecks. Chromium WASM had the same maximum difference, 1.77636e-15. Both the final Wheel and a Wheel rebuilt offline from extracted sdist in an isolated directory passed all 1,182 Python tests. Packages retain complete GPLv3 licensing and exclude temporary original source, loose raw dat files, and product code.

SMAC GPLv3 notices/full text and CNES coefficient attribution remain. Subsequent policy uses Apache-2.0 for original code while retaining all six models and original third-party licenses; additional SMAC Apache permission is pending. See [ADR 0004](decisions/0004-apache-project-license.md), [licensing](licensing.md), the source comparison, and the inquiry draft. The website offers a rights-contact entry; no public hosting/release has occurred. Approximately 0.6 MiB of `.reference-cache/smac` was removed. Complete Python tests, examples, and offline archive verification passed after cleanup. See [SMAC scope](smac.md).

---


## 📘 Earlier stage · Fresnel and Planck foundations

Public `rsphysics-numerics` added `optics`/`thermal`: passive nonmagnetic single-interface Fresnel s/p power and complex amplitudes, interface Poynting flux, total internal reflection, permittivity input, and vacuum wavelength/frequency Planck radiance/inverses. Python `optics`/`radiometry` offer NumPy broadcasting, float64/complex128 buffers, and GIL release. The five models then present retained their equations without new coupling.

Definitions specify exact SI h/c/k, exp(-iωt), n+iκ, incidence from the normal, lossless incident media, interface flux versus slab transmittance, spectral density per m/nm/um/Hz, radiance versus exitance, and monochromatic brightness temperature versus product K1/K2 conversion. Log/expm1/log1p formulations handle extreme thermal ranges; true overflow raises, and Wien-tail underflow returns explicitly noninvertible zero.

Stage acceptance passed 37 Rust and 339 installed-Wheel Python tests including local ESA data, fmt/Clippy, and six manifests/five scientific models. The 106 added Python tests included independent 80-digit references for 32 Fresnel, 32 wavelength Planck, and 32 frequency Planck cases, plus Brewster/TIR/energy, spectral Jacobians, Wien/Rayleigh–Jeans/Stefan–Boltzmann limits. Ordinary offline CI ran 338 and skipped one local-data test.

Maximum Fresnel absolute difference: 2.1407e-13. Wavelength Planck forward/inverse maximum relative differences: 1.2901e-13 / 1.3589e-13; frequency: 1.6664e-13 / 9.2371e-14. These are independent high-precision evaluations of published equations, not vendor-software runs. Both final and independently sdist-rebuilt Wheels passed all 339 tests. See [physics tools](basic-physics.md) and `bindings/python/examples/basic_physics.py`. Subsequent entries retain earlier historical stages.

## 📘 Earlier stage · Scientific computing infrastructure

Public Rust added spectral interpolation/integration/response convolution and basic radiometric calibration. Python added `spectral`, `sensors`, `radiometry`, and common `run_array`/`iter_array`/`parameter_grid`. The five dedicated batch interfaces used one typed NumPy collector, bypassing JSON for scientific arrays. Default parameter columns/result structures remained compatible; shared coordinates and chunks were optional. No generic coupling or inversion was introduced.

The official-response registry contained 12 profiles and 112 curves: Sentinel-2 A/B/C MSI v5; Landsat 4/5 TM, 7 ETM+, 8/9 OLI, 8/9 TIRS; and Terra/Aqua MODIS, using NASA OBPG's 16 reflective bands. Nine profiles/73 curves were bundled. ESA tables/responses were excluded and required explicit local download/import. Records include raw/processed hashes, ranges, negative-response clipping, and source sorting. The user's standard cache contained S2 A/B/C plus verified source tables; temporary project source downloads were removed. Ordinary offline tests do not require them.

Windows acceptance passed 33 Rust and 233 installed-Wheel Python tests including actual ESA v5 import, fmt/Clippy, six manifests/five scientific models, and metadata/resource hashes for 12 profiles. Ordinary CI ran 232 and skipped one local-table test without downloading ESA. Spectral kernels matched 32 independent rational-polynomial integration references. PROSAIL-to-satellite-band and chunked parameter-sweep examples ran. The independently buildable source package excluded raw tables, ESA samples, caches, and product code.

A local 64-row, three-repeat median batch comparison and profile checks are recorded in `benchmarks/infrastructure/validation-report.json`; these are local evidence only. API/physical/calibration conventions are in [Python infrastructure](python-infrastructure.md). Earlier-stage history follows.

## 📘 2026-11-09 · Five independent models

Added `microwave.tau-omega`, SMAP ATBD zero-order equation 1, and `microwave.water-cloud`, V1=V2=VWC. Each provides independent Rust, Runtime, dedicated/batch Python, and installable packages. The product added WASM/Worker, microwave categories, introduction pages, contribution breakdowns, fixed-input angular sensitivity, CSV/JSON, and reruns. The homepage explains remote sensing physics/modeling without automatically simulating. Public CI gained no WASM requirement.

Stage acceptance passed 28 Rust and 163 installed-Wheel Python tests, fmt/Clippy, six manifests/five models, 16 development-browser and five production-browser tests. Each microwave model had 32 high-precision cases and an 81-point angle curve. Python/Chromium maximum absolute errors were 8.5265e−14 K for tau–omega and 3.5527e−15 for WCM, including sweeps. WCM also checked pinned SenSE vegetation source; tau–omega used official-equation references, not an original operational software run. Reports remain in model reference directories. Final and independently rebuilt sdist Wheels each passed all 163 tests, excluding temporary reference/product code.

Soil response is directly supplied and coefficients require external calibration; defaults are illustrative. No soil-moisture conversion, inversion, or generic coupling was added. Original downloads were isolated in `.reference-cache` and removed after acceptance, while offline cases/provenance remain. Earlier history follows.

Updated: 2026-11-09.

## 📘 Earlier stage · Third model, Ross–Li

Added `surface.ross-li`, fixed RossThick–LiSparseReciprocal, h/b=2, b/r=1, without hotspot enhancement, inversion, or coupling. Six scalar inputs produce single-band BRF/BRDF, three kernels, a 171-point principal plane, a 648-point hemisphere, and negative-BRF diagnostics. No optical coefficient data or artificial wavelength axis is used.

References used original AMBRALS float C functions from a pinned public mirror and independent 80-digit mpmath 1.3.0 JRC/MODIS equations. Across 32 cases, 26,304 original and 26,368 high-precision values were checked. Native/Python/WASM maximum absolute difference from high precision was approximately 1.27898e-13; original C passed its separate single-precision mixed tolerance. The science-team FTP timed out; the mirror was not claimed to be authenticated operational NASA source. Original C had no explicit redistribution license and stayed temporary; Rust independently implemented published equations. Original-project licensing was undecided at this historical stage.

Local acceptance passed 24 Rust and 97 Python tests, fmt/Clippy, four manifests/three models, plus 12 development and three static-production browser tests. Website additions included surface/directional-reflectance classification, introductions, angular plots, map selection, negative-reflectance notices, CSV/JSON, and reruns. A homepage and ordered navigation—home, catalog, model run, history—avoided automatic selection/computation and supported direct model links/refresh. Science stayed in the public kernel; the product retained WASM capabilities/browser evidence, while public descriptors stayed wasm=false.

Wheel and independently built source-package installations passed numerical checks. Sources, inputs, offline spectra/angular arrays, and reports remain in `benchmarks/reference`. The shared cleaner can remove all three original environments. Cross-platform CI, PyPI, remote repositories, and formal publication remained pending.

The then-current library/product boundary followed v0.3 and ADR 0002. Public M1 accepted native/Python/pip workflows; products owned WASM/browser/Tauri acceptance.

| Work | Historical stage status | Evidence or remaining requirement |
|---|---|---|
| Workspace/toolchain/lockfile | Implemented | fmt, Clippy, workspace tests |
| Physical types/numerics | Implemented | Spectral, angular, geometry-domain, interpolation checks |
| Model API v0.1 RFC | Draft implemented | Typed contracts, schemas, errors, capabilities, evidence |
| Runtime/provenance | Initial implementation | Defaults, backend restrictions, output validation, batch errors |
| Template/manifest checker | Initial implementation | Template compilation, Cargo/metadata/catalog consistency |
| Native CLI | Implemented | Discovery, details, manifest validation, JSON execution |
| CI | Configured; remote execution pending | Windows/Linux/macOS × Python 3.11/3.13; Rust/installed-Wheel checks; no general WASM check |
| Asset/rights inventory | Official reference acquired; review incomplete | Author v1.7.0 pin, original source/coefficients, MIT package declaration, SHA-256 |
| PROSPECT golden reference | Established | Original pinned R functions via R 4.6.0/expint 0.2.1; 32×2101×2 values |
| Rust PROSPECT | Local reference agreement | Typed model/schema, resource hashes, finite/energy/shape checks |
| PyO3/NumPy/Wheel | Implemented and installed locally | Dedicated/common calls, Rust batches, NumPy buffers, Wheel/sdist installation |
| PROSAIL-D Rust/Python | Locally validated | PROSPECT-D + 4SAIL/Campbell-13; 24 author-R cases plus labelled analytical bare-soil limit; 105,050 values |
| wasm-bindgen/Worker | Separate local product implemented | Actual two-model WASM/Worker; no public WASM requirement |
| Website/experiments | Separate local product implemented | Forms, plots, citations, exports, reruns, cancellation/recovery, responsive layout, static-build checks |
| Backend agreement | Local native/Python/Chromium validated | Other OS/browser/Tauri checks pending; no deployment |
| Catalog generation/features | Deferred | Two explicit model registrations; compiled/manifest consistency checked |

M0/M1 original references, Rust kernel, and local Python/pip workflows were completed, with a second model and product integration. Formal release still required cross-platform CI/permissions review. Official source/resources were acquired autonomously; no user-supplied materials were needed. `.reference-cache/{prospect-d,prosail-d}` isolated original downloads/shared R; cleanup does not affect daily tests.

## 📘 Earlier stage · Second model and local website acceptance

Windows/Rust 1.97.0/CPython 3.13/NumPy 2.5.4 passed 21 Rust and 63 Python tests, fmt/Clippy, and consistency for three manifests/two models. PROSAIL maximum original-reference absolute errors were native Debug 4.66294e-15, Python Release 2.66454e-15, Chromium WASM 3.96905e-15; bare soil was separately labelled.

The product passed six development and one static-production test, covering 239,514 two-model WASM reference values, real forms/exports, offline execution, history, cancellation, and responsive interaction. Hotspot BRF above 1 was not clipped; canopy pages did not mislabel results as leaf R/T absorptance. Product evidence/screenshots are in sibling `RSPhysics-app/docs`. Neither project had a Git remote; product metadata recorded public-source SHA-256 and null Git revision. No PyPI publication, remote creation, or deployment occurred.

Wheel and source-package rebuild/install checks passed. Formal source/resource rights review and cross-platform CI remained incomplete. Cleanup removed approximately 240.5 MiB of original two-model sources/downloads/shared R, preserving runtime resources and offline evidence. Afterward, provenance/resource checks, PROSAIL Rust references, and 63 Python tests passed again.

## 📘 Earlier stage · PROSPECT-D acceptance

Windows x86_64/Rust 1.97.0/CPython 3.13/NumPy 2.5.4 passed 19 Rust and 36 Python tests, formatting, Clippy, metadata, and source hashes. Installed Wheel and offline sdist build passed the same Python references. Packaging excluded original caches, Node dependencies, and product source. Isolated sdist builds preserved matching metadata/evidence. Actual Debug/Release errors are in benchmark notes/reports. Cross-platform CI was configured but not remotely executed.

After acceptance, approximately 289.6 MiB of original reference cache was removed. Source/resource/fixture checks, 36 Python tests, and Rust references passed afterward, establishing independence from R/original installations. Prepare/generate scripts can rebuild the shared cache when needed.

When local incremental-cache hard links are unavailable, Rust copies files instead; this does not affect these test results.

## 📘 Historical repository-boundary checks

Added v0.3 and ADR 0002; updated README, Model API, and Python binding targets. No Rust crates changed and no product source needed moving.

An independent workspace in `target/external-consumer-smoke` consumed Runtime/Catalog/Model API through local paths, compiled, and printed `external workspace: API 0.1, models 0`. This established compatibility of workspace metadata/internal dependencies with independent consumption. No actual remote existed, so real Git URL/SHA fetching and private-CI integration remained pending organization decisions.
