# I²EM reference cases

Twenty monostatic HH/VV cases computed with the original IEEE-GRSS MATLAB
program (DOI [10.21982/M8BC7Z](https://doi.org/10.21982/M8BC7Z)) using MATLAB
R2026a. Cases cover exponential and Gaussian correlation functions, L/C/X-band
frequencies, lossless and absorbing media, incidence angles 1–80°, and roughness
near the stated `ks < 1` and `s/L <= 0.25` limits.

`expected.json` contains rows `[sigma0_vv_db, sigma0_hh_db]` in `inputs.json`
order. The comparison tolerance is 2e-9 dB. Linear power is checked against
`10**(dB/10)`. Provenance and content hashes are recorded separately.

Run `python benchmarks/reference/i2em/verify.py` for offline fixture integrity.
To recompute with a separately downloaded original source and MATLAB, run
`python tooling/reference-runner/verify_i2em_source.py --source PATH_TO_M_FILE`.
The runner creates its temporary extracted co-polarization routine beside the
reference source and leaves the original source unchanged. It retains the
original arithmetic, including unused Fresnel integrations, the approximate
wave number and the 0.01 rad incident-angle offset; it does not call the HV
solver. Original MATLAB code is not bundled with RSPhysics.
