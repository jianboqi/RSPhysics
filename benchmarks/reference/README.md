# Original-reference benchmarks

These suites document numerical agreement with identified original implementations
or independent published-equation references. Original downloads, source,
coefficient copies, and reference runtimes are isolated in `.reference-cache`;
they may be deleted after acceptance and are not part of version control.

Retain offline evidence and required runtime data:

- `prospect-d/inputs.json`: deterministic inputs with all parameters and units.
- `prospect-d/expected.json`: original outputs generated independently of Rust.
- `prospect-d/upstream-provenance.json`: versions, source URLs, file hashes.
- `prospect-d/validation-report.json`: Rust/Python error metrics.
- Model crate coefficients: required runtime resources, verified before use.

Never generate golden outputs with the new Rust implementation and call them
original references. See the [PROSPECT guide](prospect-d/README.md) for generation
and cleanup; subsequent models have their own reference directories.

The supplied [L-MEB](lmeb/README.md) and [GORT-GPP](gort-gpp/README.md) suites
include unchanged-function MATLAB runners, pinned source checksums, and full
angular reference arrays. Their source permissions and implemented subsets are
documented separately from numerical validation.
