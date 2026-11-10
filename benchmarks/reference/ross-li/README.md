# RossThick–LiSparseReciprocal reference

Fixed configuration: RossThick volume kernel + LiSparseReciprocal geometry
kernel, h/b=2 and b/r=1, no hotspot enhancement. One band, no spectral model,
inversion, albedo integration or coupling. Relative azimuth0 is the sun's
azimuth/backscatter; 180 is opposite/forward. BRF is dimensionless; BRDF=BRF/pi
has unit sr^-1. Weights and raw kernels are not interchangeable physical inputs.

Two independently generated references are retained in expected.json:

1. **Original AMBRALS C functions**, unchanged. The original science-team
   tools page links `ftp://rsftp.eeos.umb.edu/data01/Website/brdf_forward.tar.gz`,
   which timed out at intake. We obtained ambralsfor.c/h and readme.txt from
   the public mirror `arthurelmes/ambrals_forward`, fixed commit
   `b43d9171ea9f6dc76854cac34037e2af7903d02e`. Source comments trace to
   ambmodels.c.reciprocal. This is not asserted to be an authenticated NASA
   operational checkout or byte-verified copy of the inaccessible archive.
   File URLs and hashes are in upstream-provenance.json.
2. **Published JRC/MODIS equations**, independently evaluated using mpmath
   1.3.0 at 80 decimal digits. This does not call the Rust solver or convert it
   into reference code. It supplies the tighter reference for f64 arithmetic.

The original C uses float inputs, intermediates and return values. Its source
is compiled directly with a thin ABI driver, without changing the functions
to doubles or changing the scientific equations. Source C is only in the
disposable cache and is never linked into the library or browser product.
Windows reference run: MSVC 2022, /O2 /fp:precise; POSIX runner uses cc -O2.
The DLL hash and platform are recorded; compiler-generated binaries are not
portable reproducibility identifiers across operating systems.

32 input cases include nadir, overhead sun, exact/near hotspot, forward,
orthogonal, high zeniths, isotropic/black limits, nonphysical coefficients,
BRF above one, and 16 seeded samples. Each includes a selected geometry,
171 principal-plane directions and 648 hemisphere directions. Original
comparisons: 26,304 values; f64 comparisons: 26,368 (also BRDF and Kiso).

Original float tolerance: `abs <= 1e-4 + 2e-5 * abs(original)`.
High-precision tolerance: `abs <= 2e-12 + 2e-12 * abs(reference)`.
The two reference versions differ by at most 1.793342252e-4 in this suite,
within the original tolerance. Native/Python/browser f64 maximum absolute
error vs 80-digit reference is about 1.27898e-13. The near-hotspot Rust distance
uses the algebraically equivalent `(tan(s)-tan(v))² + 4 tan(s)tan(v)sin²(phi/2)`
to avoid double-precision cancellation.

Rebuild (network only needed for prepare, plus a C compiler and mpmath):

```sh
python -m pip install mpmath==1.3.0
python benchmarks/reference/ross-li/prepare.py
python benchmarks/reference/ross-li/generate.py
python benchmarks/reference/ross-li/verify_upstream.py --require-cache
cargo test -p rsphysics-ross-li -- --nocapture
python benchmarks/reference/ross-li/report.py
```

`report.py` uses the installed rsphysics Python package. Daily checks use
offline fixtures, not the compiler, original C or mpmath. Source/runtime/builds
are disposable under `.reference-cache/ross-li`. The common
`tooling/reference-runner/clean.py --remove` removes this cache too.
No explicit redistribution license was found for the original reference C;
it is excluded from distribution. RSPhysics independently implements the
published equations and retains the original project's pending license status.
No runtime coefficients or external datasets are required by this model.
