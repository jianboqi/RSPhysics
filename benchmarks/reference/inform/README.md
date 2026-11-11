# INFORM MATLAB reference

The [EUFAR teaching archive](https://www.eufar.net/documents/6406/) contains the MATLAB INFORM implementation and its historical PROSPECT 3.01 coefficient table. Keep the extracted files and generated CSVs in an ignored directory such as `target/inform-reference`.

`matlab_reference.m` runs six cases at 421 wavelengths using the teaching implementation and a synthetic soil spectrum. It runs from temporary copies of the MATLAB files. The copies restore the forward scattering coefficient in the SAIL direct-beam equation and bound the crown–shadow intersection by the two individual cover fractions. The tests compare the resulting forest, soil, understory and dense-crown reflectance, the crown and ground weights, both directional crown transmittances, crown cover, and both layers' leaf optics.

In MATLAB, add this folder to the path and run `matlab_reference(source_dir, output_dir)`, with the full paths to the extracted `INFORM_V5` directory and the desired output directory. In PowerShell run the full comparison with:

```powershell
$env:INFORM_REFERENCE_DIR = (Resolve-Path 'target/inform-reference').Path
cargo test -p rsphysics-inform --test reference -- --include-ignored
```

The regular Rust test uses nine wavelength samples per case and supplies the reference leaf spectra directly. The full comparison supplies the PROSPECT 3.01 coefficients to Rust and tests the leaf solver as well.
