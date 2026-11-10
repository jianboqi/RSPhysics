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


`full-inputs.json` / `full-expected.json` add 18 unmodified MATLAB reference
cases for x-power and x-exponential roughness, exponents 1.2, 1.5 and 2, and
incidence angles 10, 30 and 60 degrees. Rust evaluates the x-power transform
through its positive Laplace integral, avoiding separate Bessel-K/Gamma overflow.
The x-exponential Hankel transform retains the reference upper bound of nine.
An exponent greater than two does not define a positive-definite isotropic
x-exponential covariance and is rejected. Nonconverged integrals are errors.

The extended oracle tightens only the original `quad` tolerance to 1e-12;
`full-original-tolerance.json` retains its default-tolerance outputs. This
separates default MATLAB integration error from a difference in the equations.
Use `verify_i2em_source.py --extended` to reproduce the extended reference.
