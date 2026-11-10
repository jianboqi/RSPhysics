# BSM original reference

Ten full 2001-sample dry/wet spectra were generated with the unmodified
author BSM.m in MATLAB R2026a, pinned at commit
ff6d8b5fb4ae0093ffdf25f91353176a11dc394a. They cover the three author example
moistures, dry/5% threshold, zero film absorption thickness, shape and
capacity changes, and a large Poisson mean exposing truncation loss.
A separate elementary Poisson PMF helper replaces the unavailable MATLAB Statistics Toolbox dependency; BSM.m itself is unchanged. The fixture validates explicit legacy_six_films mode at2e-10 absolute.

Complete-mode tests independently sum the published Poisson series with
SciPy probabilities and numerical Fresnel integration. They verify omitted
mass and explain the difference from the author's k=0..6 approximation.
Original downloads and MATLAB runners are disposable in .reference-cache.
