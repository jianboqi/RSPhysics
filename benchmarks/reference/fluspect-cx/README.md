# FLUSPECT-CX reference spectra and matrices

Twelve cases computed by the unmodified author implementation in SCOPE
revision `f4d07b33828f82c572cb1965cf82b1db4579c5b4`, using MATLAB R2026a.
The selected coefficient table is `Optipar2017_ProspectD.mat`; protein and
carbon-based constituent inputs are zero.

The original light calculation uses 15 doubling steps, an excitation grid
400–750 nm at 5 nm intervals, and an emission grid 640–848 nm at 4 nm intervals.
Both fluorescence matrices include the original 5 nm excitation integration
weight. `expected.json.zlib` contains all optical spectra and both full matrices
in `inputs.json` order. Compression is lossless.

Cases cover leaf structure, pigments, water/dry matter, xanthophyll state,
zero/doubled fluorescence efficiency, and a conservative zero-absorption limit.
The standard tolerance is 2e-10 absolute plus 2e-9 relative. The conservative
case allows 2e-8 absolute because the original floating-point Stokes evaluation
is ill-conditioned at zero absorption; Rust evaluates its analytic limit.
When fluorescence efficiency is zero, the reference wrapper supplies zero
matrices for fields omitted by the original function.

`provenance.json` records source and data checksums. The original source and
MATLAB binary coefficient file are obtained separately, outside the package.
Run `python benchmarks/reference/fluspect-cx/verify.py` for offline checks.

Recompute with separately obtained pinned source/data files using
`python tooling/reference-runner/verify_fluspect_source.py --source DIRECTORY`.


Eight additional cases in `full-inputs.json` and `full-expected.json.zlib` cover
the full 1 nm fluorescence grid, D/PRO coefficients, protein/CBC, fixed-carotenoid
state -999 and zero fluorescence efficiency. On common inputs the full-grid
SCOPE wrapper agrees exactly with the standalone author full-grid routine.

The D table has a negative emission residue at 641 nm (-4.5368895016696e-6).
The solver and full-grid wrapper set it to zero for physical emission;
`negative-emission-evidence.json.zlib` retains original negative matrix rows.
Raw coefficient bytes remain unchanged. Matrices act on photon spectra;
energy-flux conversion is tested independently using lambda_exc/lambda_em.
