# Ross–Li implementation and reference provenance

The Rust solver is an independent implementation of published equations:

- JRC RAMI, parametric Ross–Li reciprocal model:
  https://rami-benchmark.jrc.ec.europa.eu/_www/definition.php
- MODIS BRDF/Albedo ATBD (1999), standard RossThick–LiSparseReciprocal:
  https://modis.gsfc.nasa.gov/data/atbd/atbd_mod09.pdf
- Lucht, Schaaf & Strahler (2000), An algorithm for the retrieval of albedo
  from space using semiempirical BRDF models:
  https://doi.org/10.1109/36.841980

Copyright 2026 RSPhysics contributors. This crate does not declare an MIT
license inherited from other model crates. The project owner's original
infrastructure license is still pending (LicenseRef-RSPhysics-Pending), and
publish=false remains in effect. No third-party C source is shipped or linked.

Original numerical verification uses unchanged AMBRALS C forward functions
from public mirror arthurelmes/ambrals_forward at commit
b43d9171ea9f6dc76854cac34037e2af7903d02e. The MODIS science-team FTP archive
was not reachable during intake; provenance does not assert authenticated
NASA operational source or identity with that unavailable archive. No explicit
redistribution license was found; downloads and compiler products are only in
the ignored disposable reference cache. URLs, SHA-256 and original precision
are recorded under benchmarks/reference/ross-li. The retained ABI driver is
new test infrastructure and contains no copied original scientific functions.

Scientific configuration: h/b=2, b/r=1, no hotspot correction. f64 equations,
clamped roundoff-sensitive inverse trig inputs, stable hotspot distance
identity, typed scalar input, raw negative-response diagnostics, and explicitly
defined principal-plane/hemisphere sampling. No runtime datasets, inversion,
spectral interpolation or general model coupling.
