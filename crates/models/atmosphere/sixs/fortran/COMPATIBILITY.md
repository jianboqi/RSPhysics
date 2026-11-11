# Compatibility corrections

The files under `upstream/` remain byte-identical to the original distribution.
`tooling/sixs/build.py` copies them into the build directory and applies the
following corrections there. `--original` builds the distribution unchanged.
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

## Initialize degree-zero polarization harmonics

`KERNELPOL.f` sums from degree zero, but only initializes `rsl` and `tsl` from
degree one. The missing degree-zero spin-2 harmonics are mathematically zero.
Reading undefined values can produce NaNs even when their expansion coefficient
is zero. Explicitly zeroing those entries removes a Linux/GNU shared-library
failure. A GNU build with signaling-NaN initialization and floating-point traps
located the read in the kernel summation.

## Restrict the full surface-boundary calculation to populated BRDF branches

The new `rosur` boundary is populated by the Rahman, MODIS, and
Ross–Li–Maignan FOS routines (BRDF IDs 8, 10, 11). Calling `OSSURF` with that
array for the other BRDF branches reads an undefined boundary. Those branches
retain the hemispherical-albedo treatment and use `albbrdf` for the diffuse
boundary, rather than deriving it from an uncomputed full-boundary signal.

The LUT main also lacks the standard main's BRDF-11 selection branch. Its build
copy receives that exact branch and calls the same shared routines.

## Guard spectral interpolation endpoints

The original `DISCOM.f` conditions access `wldis(l+1)` at `l=20` and
`wldis(l-1)` at `l=1`. Fortran does not require short-circuit evaluation of
`.and.`. Nested bounds conditions retain the selection logic while avoiding
out-of-bounds reads.

## Restore Kuusk integration control flow

`AKTOOL.f:GMD92` contains an incomplete conversion of assigned GOTOs: return
labels 71–73 are assigned, assignment 74 is commented, and three returns are
hardcoded to 74. This corrupts the integration state and can crash the original
2.1 executable. The build copy restores selection among 71–74 and assignment
74, following the assigned-GOTO control flow preserved in the original 6S 1.1
distribution. Its equations, quadrature, and optical data are retained.

## Correct polarization interpolation and cutoff initialization

`INTERP.f` initializes `coefl` inside a block skipped for ground observations,
then uses it to interpolate aerosol Q/U phase functions. It is now initialized
after the spectral bracket is selected. The assignment to undeclared `ruaer0`
is corrected to the output `ruaero` in the linear U interpolation branch.

`IAPITOOLS.f:solve` leaves `Q0d(k)` and `xI0t` undefined when its exponential
cutoff is reached. They are initialized to zero before the cutoff test. The
last row of the convergence history `xIf` is also initialized. This removes
compiler/storage-dependent Iaquinta–Pinty albedos and non-finite LUT signals.

The LUT initialization diagnostic uses the reserved `rm(mu)` slot before it
is populated. That slot now uses the selected viewing cosine `xmuv` in the
diagnostic, without changing the subsequent solver or LUT construction.

## Preserve wavelength rounding in native builds

GNU Fortran builds use `-ffp-contract=off` for the solver and executable
reference builds. In the Kuusk BRDF branch, the single-precision wavelength
expression can round to opposite sides of the 550 nm midpoint depending on
whether the multiply and add are fused. This selects different entries in the
leaf optical table. Separate rounding preserves the reference table selection
on targets with fused multiply-add instructions.

## Direct-library adaptation

`library.py` makes additional structural changes in build copies only:

- Convert each `PROGRAM` to a callable subroutine and retain its SAVE lifetime.
- Expose `rsp_fortran_run(directory)` through `ISO_C_BINDING`.
- Relocate standard input/output to units 45/46, preserving the host's streams.
- Scope auxiliary paths to the calculation directory, check input I/O, and
  convert original STOP paths into returned errors.
- Check fixed array counts when reading raw layer, Mie, and particle inputs.
- Allocate Mie scratch arrays through the series order selected by the original
  criterion, retaining their lower bounds and 10-million-order maximum. This
  avoids reserving 1.12 GB for every module or overflowing Flang's wasm32 stack.
  Local allocatables are released when the routine returns. Numerical
  statements, stopping criteria, and supported series capacity are unchanged.

Windows/Linux wrappers snapshot model-owned initialized sections and reset
COMMON/BSS between calls. The Fortran runtime is excluded. macOS loads a unique
library image for each calculation. WASM starts a fresh module/linear memory.
These strategies preserve independent original-program initialization.

Build reports record all applied corrections. The original/corrected executable
smoke suite and the 330-case direct-library suite serve different purposes:
the former retains the unmodified baseline, while the latter covers the usable
original solver with these documented corrections. Comparison is limited by
the original formatted output precision; it is not an independent validation
of the underlying physical model.
