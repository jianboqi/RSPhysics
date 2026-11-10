# 6SV2.1 reference validation

## Direct-library suite

`full-reference.json.zlib` stores 330 complete reports and auxiliary products
from GNU Fortran 16.2 executables compiled from the original source with the
[documented compatibility corrections](../../../components/6sv2.1/COMPATIBILITY.md).
It includes the oracle build recipe. `manifest.json` pins the compressed fixture
hash, suite identifier, and case count; `cases.py` defines the inputs.

Coverage includes both original main programs, all 198 built-in sensor bands,
all atmospheric profile modes, satellite geometry modes, aerosol mixtures and
layers, four Mie distribution families and eight save/reload cases, target/sensor
altitudes, spectral modes, all 12 surface BRDF selectors, heterogeneous surfaces,
surface polarization, and radiance/reflectance correction.

The installed Python API is checked against every case. The direct Linux C ABI
validator checks 145,597 numerical values across reports and auxiliary files.
The application validates the same 330 cases through a browser Web Worker.
Signal tolerances distinguish reflectance from radiance; remaining report/file
comparisons account for single-precision arithmetic and printed rounding.
These tests establish implementation agreement over this suite, not exhaustive
input-space coverage or independent validation of 6SV's physical assumptions.

## Preserved smoke and performance baselines

`inputs.json` contains an initial three-case suite: a monochromatic Lambertian
surface, a uniform 400–900 nm response, and the original distribution's MODIS
BRDF example. Both `sixsV2.1` and `sixsV2.1-lutaero` are exercised.

Run the source-preservation, original/corrected native, and WASM checks using
[the build tools](../../../tooling/sixs/README.md). The smoke comparison checks
all numerical fields in the printed reports, including the LUT program's
angular coordinate diagnostics. It is limited by the original output precision;
it is not an exhaustive test of atmospheric profiles, Mie options, sensor bands,
surface modes, correction modes, or auxiliary-file round trips.

`smoke-validation.json` records the tested builds and differences. Separate
tolerances are used for report quantities (1.01e-5 absolute in this initial
suite) and single-precision LUT angles (1e-4 degree). These smoke tolerances
must not substitute for quantity-specific validation of additional cases.

`performance-linux.json` records an initial GNU Fortran 13.3, `-O2` baseline on
a local 32-logical-CPU WSL2 environment. Each single-run statistic follows a
warm-up and seven repetitions. The parallel comparison runs 32 independent
monochromatic calculations on eight workers. Process launch and report capture
are included; container startup is excluded. This report does not attribute
time to individual numerical routines.

`performance-windows.json` is a second baseline using the tracked benchmark
script and the original GNU Fortran 16.2 Windows executables. It includes all
seven sample timings per case and the serial/eight-worker comparison.
