# 6SV2.1 build-portability benchmarks

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
