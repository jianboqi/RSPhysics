# PROSPECT-D author-R reference benchmarks

The author's [installation documentation](https://jbferet.gitlab.io/prospect/) points to the maintained `jbferet/prospect` repository. This suite pins **v1.7.0**, commit `4649d5006ddf6bce609951765a88ba2498fcd8ab`, rather than current master.

Downloaded R source, coefficient copies, provenance, and the reference runtime are isolated in **`.reference-cache/prospect-d/`**, excluded from version control and disposable. `upstream-provenance.json` retains per-file URLs, sizes, and SHA-256. Original files preserve author and FLUSPECT attribution.

After validation, approximately 289.6 MiB of reference cache was removed; regressions still passed. The prepare/generate commands below can recreate it.

The historical release declares `Version: 1.7.0` and `License: MIT` in DESCRIPTION. The v2.0.1 version inspected on 2026-11-09 declares `GPL-3 + file LICENSE`. Do not apply the older MIT declaration to newer source. The v1.7.0 tree has no separate LICENSE file; review retained third-party attribution and any additional coefficient restrictions before public distribution. Provenance registration does not establish completed permissions review.

v1.7.0 supports D/PRO; this suite explicitly selects D and every parameter, with PROT=CBC=0. Node/WebR 0.6.0 runs R 4.6.0 and expint 0.2.1, executing the original `PROSPECT` and `calctav` scientific bodies. Only the `define_Input_PROSPECT` normalization helper is replaced by a complete-input-frame passthrough to avoid requiring the full R package namespace. Windows Worker adaptation changes JS file-loading paths only, not R/math; this is not the product WASM binding.

`inputs.json` contains 32 deterministic cases; `expected.json` preserves 2,101-band reflectance/transmittance generated before Rust implementation. Coverage includes typical/boundary cases, zero/near-zero absorption, incidence angles, and 18 seeded samples.

Acceptance: `abs_error <= 2e-10 + 2e-8 * abs(reference)`. This measures implementation agreement, not accuracy against observations. Local Rust Debug maximum absolute error was approximately 4.8e-9; Python Release approximately 3.9e-15, with per-case metrics in `validation-report.json`. Conditioning-dependent Debug/Release differences meet the predetermined tolerance; outputs are not clipped to hide differences. Remote cross-platform validation remains pending.

Daily tests use retained runtime coefficients, inputs, and expected outputs, without R or reference caches. The retained few megabytes are offline regression evidence, not an original installation.

Offline verification:

```sh
python benchmarks/reference/prospect-d/verify_upstream.py
```

Regeneration requires network access and writes references only into the shared cache:

```sh
python tooling/reference-runner/prepare.py
node tooling/reference-runner/generate.mjs
```

`fetch_upstream.py` can separately retrieve the pinned original sources. The generator requires the recorded expint version, never silently accepting dependency upgrades. Regeneration is explicit, outside daily CI; fixture changes need source/difference review.

Cleanup, preview before removal:

```sh
python tooling/reference-runner/clean.py
python tooling/reference-runner/clean.py --remove
```

Cleanup preserves required `crates/models/vegetation/prospect-d/data/coefficients.tsv`, inputs/outputs/provenance, and the Python environment. Rust/Python tests remain runnable afterward.
