# SOILSPECT equation reference

generate.py independently evaluates Jacquemoud, Baret & Hanocq (1992) and
Baret, Jacquemoud & Hanocq (1993), without importing RSPhysics. It covers
all four phase terms, albedo0/0.75/1, zero/narrow/broad shadow hotspots,
nadir, exact backscatter and grazing configurations. Full principal-plane
curves are retained. Python/SciPy flux quadrature uses96x192 points; Rust
returns a32x64 or64x128 integral and exposes the order difference.

The authors' prosail R soilspect routine is a test-only equation cross-check
for selected BRF, not an executed R reference. Its hemispherical formulas
omit hotspot and primed phase terms and contain removable singularities;
they are not used as golden flux outputs. RSPhysics integrates the entire
BRF expression, including those terms. Integrals are numerical approximations,
with test tolerance informed by convergence; BRF tolerance2e-12.
References: https://doi.org/10.1016/0034-4257(92)90072-R and
https://doi.org/10.1016/0273-1177(93)90560-X.
