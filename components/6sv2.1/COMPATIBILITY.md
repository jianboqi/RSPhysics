# Compatibility corrections

The files under `upstream/` remain byte-identical to the original distribution.
`tooling/sixs/build.py` copies them into the build directory and applies the
following correction there. `--original` builds the distribution unchanged.
Neither path uses fast-math or changes default REAL/DOUBLE PRECISION kinds.

## Initialize the unused zero-direction scattering history

In `OS.f`, `OSSURF.f`, and `ISO.f`, the primary and successive scattering
assignments skip direction index zero, but later history, extrapolation, sum,
and convergence loops read `in(0)`, `inm1(0)`, and `inm2(0)`. `OSPOL.f` has
the corresponding uninitialized `in(:,0)`, `qn(:,0)`, and `un(:,0)` values.
The correction explicitly initializes these unused entries to zero at the
start of each subroutine call.

Direction zero is the incident-sun slot rather than a quadrature direction.
Its undefined history must not contribute to the convergence tests. Native
GNU builds can happen to obtain zeros from fresh memory; a Flang WASM build
exposed nonzero values. A diagnostic copy of the original program traced the
resulting extra Fourier orders in the original MODIS BRDF example. All
equations, tolerances, quadrature dimensions, tables, and physical directions
are retained.

Build reports list whether this correction was applied. Numerical comparison
must include both original and corrected native executables and the WASM
executables; original printed precision limits comparisons of text reports.
