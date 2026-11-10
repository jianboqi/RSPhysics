# PROSPECT reference intake requirements

Officially available materials are acquired independently at the project owner's
request. The author's website led to the author-maintained repository; pinned
v1.7.0 source/coefficients were acquired for the first candidate, PROSPECT-D.
See the [reference guide](../../benchmarks/reference/prospect-d/README.md).
No user-supplied source is currently required for that model.

Record the following for a golden reference. Request user help only when official
materials are genuinely missing or inaccessible. Preserve original directories
and names; do not first rewrite equations or coefficient formats.

1. Scientific version: identify PROSPECT-5/D explicitly; do not mix versions' equations/data.
2. Complete implementation: source, version/commit/archive, build/run procedure, dependencies, license, authors, and paper. A local path is acceptable.
3. Coefficients/data: original spectral table, wavelength grid, format, units, source, version, and redistribution terms; identify embedded data too.
4. Reproducible cases: at least one complete input/output with meanings/units. If outputs are absent, provide a runnable original and invocation instructions so references can be generated.
5. Scientific acceptance: trusted tests, known issues, domains, and maintainer/reviewer contacts. Set tolerances from evidence rather than a universal arbitrary threshold.

Keep original materials separate from the Rust port and record file hashes.
Use pinned generation scripts and typical, boundary, and invalid inputs. Files
with unverified distribution rights are not cleared by a provenance record.

The pinned official R scientific functions have now generated 32 complete spectra
and passed Rust/Python agreement checks. Downloads/R runtime moved to disposable
`.reference-cache/prospect-d/`. Retain provenance, required runtime coefficients,
inputs/expected spectra, and error reports. This is agreement with the specified
reference implementation, not accuracy against all real observations.
