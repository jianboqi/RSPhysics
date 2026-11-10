# Basic physics independent reference

`generate.py` evaluates direct published Fresnel/Snell and Planck equations at
80 decimal digits using mpmath 1.3.0, without importing RSPhysics or reading its
outputs. It records 32 optical and 64 thermal cases and a fixture SHA-256.

Fresnel reference uses complex transmitted angles, E-field r/t and the separate
s/p Poynting-flux factors. Production uses normalized normal-wavevectors and
admittance ratios. Planck reference uses direct exponentials; production uses
logarithmic branches and expm1/log1p. Exact SI defining h/c/k values are used.

Sources and conventions: [NIST SI](https://www.nist.gov/pml/special-publication-330/sp-330-section-2),
[CIE Planck definition](https://cie.co.at/eilvterm/17-24-005),
[Byrnes 1603.02720v5](https://arxiv.org/abs/1603.02720v5).
No third-party program or runtime data is downloaded or redistributed.

```sh
python benchmarks/reference/basic-physics/generate.py
python benchmarks/reference/basic-physics/verify.py
python -m pytest bindings/python/tests/test_basic_physics.py -q
python benchmarks/reference/basic-physics/report.py
```

Only regeneration requires `mpmath==1.3.0`; ordinary tests are offline. See
[API and physical scope](../../../docs/basic-physics.md). The reference verifies
equation implementations, not observational calibration or model coupling.
