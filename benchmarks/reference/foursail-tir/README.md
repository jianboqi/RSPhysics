# 4SAIL thermal infrared references

The implementation reference solves the transfer ODE in Verhoef, Jia, Xiao
and Su (2007), Eqs. (1)–(5), using independent SciPy matrix exponentials.
Diffuse fluxes satisfy an explicit mixed soil boundary condition; a separate
augmented state integrates scattered radiance in the viewing direction.

Geometry is evaluated by the author's MIT R reference at commit
`46549958295b605ac1e211a693d4b2287c3a5a13`. Its original 20-segment
exponential hotspot quadrature defines the joint-gap numerical convention.
The thermal oracle does not call the Rust solver or translate its analytic
transport solution. Original source files, the paper PDF and WebR runtime
are disposable and are not bundled.

The pinned suite contains 28 cases, 819 default angular directions and 61
default wavelengths. It covers four independent surface temperatures, sky and
direct irradiation, both LIDFs, exact/disabled hotspots, bare/thin/dense
canopies, black/reflective limits, and wavelength/temperature boundaries.
Independent physical checks also enforce isothermal equilibrium, Kirchhoff's
law, the bare-soil solution and radiative component closure. These are
implementation checks, not field validation of the canopy approximation.

Run offline integrity checks:

```sh
python benchmarks/reference/foursail-tir/verify.py
```

With NumPy and SciPy installed, regenerate to a temporary file from the pinned
original-R geometry:

```sh
python benchmarks/reference/foursail-tir/generate_reference.py --geometry benchmarks/reference/foursail-tir/geometry.json.zlib --out target/foursail-tir-reference.json
```

To rerun the original R geometry as well, provide independently obtained source
files matching `provenance.json` and WebR 0.6.0:

```sh
node tooling/reference-runner/verify_foursail_source.mjs --source .reference-cache/foursail-tir --webr .reference-cache/foursail-tir/webr/node_modules/webr/dist --python .venv/Scripts/python.exe
```

Reference: [public NLR-TP-2007-242 report](https://reports.nlr.nl/server/api/core/bitstreams/ecd3960d-4c3c-4a76-9bd6-186044eb0862/content).
