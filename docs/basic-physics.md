# Fresnel and Planck building blocks

Numerical equations live in `rsphysics-numerics::{optics,thermal}`. Python exposes
NumPy broadcasting, native float64/complex128 buffers and GIL-free computation.
No scientific model is coupled, and no atmosphere, rough-surface model or
multilayer solver is added. The existing five model IDs and batch APIs stay
unchanged.

## Fresnel reflection

```python
import numpy as np
import rsphysics as rsp

angles = np.array([0., 30., 45., 60., 80.])
glass = rsp.optics.fresnel(angles, 1.5, n_incident=1.)
print(glass['reflectance_s'], glass['reflectance_p'])

# Lossy material, n + i*k, using exp(-i*omega*t).
absorber = rsp.optics.fresnel(40., 2.5 + 0.3j)
# Alternative input: relative permittivity, assuming mu_r=1.
epsilon = rsp.optics.fresnel_permittivity(40., 6.0 + 0.5j)

# Total internal reflection from glass into air.
tir = rsp.optics.fresnel(60., 1., n_incident=1.5)
print(tir['total_internal_reflection'], tir['amplitude_r_s'])
```

Scope: a smooth planar interface between homogeneous, isotropic, nonmagnetic
media. Incident refractive index is **positive real and lossless**. Transmitted
index is `n+i*k`, with finite `n>=0`, `k>=0`, not both zero. Negative-index/gain
media and an absorbing incident medium are outside the implementation. The
incident angle is in degrees **from the interface normal**, `0<=theta<90`.
The branch of the transmitted normal wavevector is forward/decaying for this
passive convention. Extremely large index ratios/intermediate admittances that
cannot be represented in f64 produce an error rather than a nonfinite result.

For relative-permittivity inputs, incident epsilon is positive real and
transmitted epsilon is nonzero with `Im(epsilon)>=0`; negative real epsilon is
allowed. Refractive-index conversion occurs in Rust with relative permeability
fixed at one. Microwave data that use `exp(+i*omega*t)` often write absorption
with a **negative** imaginary part; conjugate explicitly to the selected
convention. The function does not guess or change the supplied sign.

All inputs broadcast according to NumPy rules. For example, angles `(4,)` and
indices `(3,1)` yield `(3,4)` outputs. Scalars yield zero-dimensional arrays;
empty broadcast arrays are supported. Invalid computed points abort the call
and report a zero-based **flat element index**, rather than silently masking.

| Result field | Meaning |
| --- | --- |
| `amplitude_r_s`, `amplitude_r_p` | Complex electric-field reflection amplitudes, complex128 |
| `reflectance_s`, `reflectance_p` | Dimensionless power reflectances, `abs(r)**2` |
| `reflectance_unpolarized` | Equal incoherent average `(R_s+R_p)/2` |
| `interface_transmittance_s`, `interface_transmittance_p` | Dimensionless normal Poynting flux entering medium 2 at its boundary |
| `interface_transmittance_unpolarized` | Equal incoherent average of those fluxes |
| `total_internal_reflection` | Boolean for the lossless transmitted-index evanescent branch |

`s` is perpendicular to the incidence plane; `p` lies in that plane. For a
horizontal interface they correspond to H and V. At normal incidence the power
reflectances are equal. The chosen reflected-field p basis follows Byrnes:
`r_p=-r_s` at normal incidence. Opposite amplitude signs do not mean different
normal-incidence reflected powers. Evanescent/TIR coefficients retain phase.
Amplitudes are not averaged to produce the unpolarized result.

With the lossless incident medium assumed here, `R+T=1` up to f64 rounding.
For an absorbing second medium, **T is flux entering at the interface**, not
power surviving through a finite slab and not a bulk absorption fraction. No
slab thickness, roughness, multiple reflection or emissivity conversion is
inferred. Small rounding excursions are retained rather than clipped.

Implementation uses normalized normal-wavevector/admittance expressions and
scaled divisions, with the cosine term retained at matched/grazing incidence.
The independent 80-digit reference instead uses complex Snell angles, standard
electric-field r/t expressions and explicit Poynting-flux factors.

Physical/formula source: Steven J. Byrnes,
[Multilayer optical calculations, arXiv:1603.02720v5](https://arxiv.org/abs/1603.02720v5),
single-interface equations and sections on transmitted power, complex branches
and amplitude sign conventions. Only the single-interface physics is used;
the reference does not imply that multilayer functionality is implemented.

## Planck radiation and inverse

```python
r = rsp.radiometry
wavelength_nm = np.array([8000., 10000., 12000., 14000.])
unit = 'W m^-2 sr^-1 um^-1'
L = r.planck_radiance(wavelength_nm, 300., radiance_unit=unit)
T = r.inverse_planck(wavelength_nm, L, radiance_unit=unit)

# Broadcast a temperature grid across wavelengths.
temperatures = np.array([250., 300., 350.])[:, None]
spectra = r.planck_radiance(wavelength_nm, temperatures)  # (3,4), per nm

Bnu = r.planck_frequency_radiance(1.4e9, 300.)           # per Hz
Tnu = r.inverse_planck_frequency(1.4e9, Bnu)             # K
```

These functions describe an **ideal blackbody in vacuum**, with both
polarizations included:

`B_lambda = 2*h*c^2 / [lambda^5 * (exp(h*c/(lambda*k*T))-1)]`

`B_nu = 2*h*nu^3 / [c^2 * (exp(h*nu/(k*T))-1)]`

| Function | Inputs | Output |
| --- | --- | --- |
| `planck_radiance` | vacuum wavelength in **nm**, temperature in K | Wavelength spectral radiance, selected density unit |
| `inverse_planck` | wavelength in nm, matching wavelength spectral radiance | Monochromatic brightness temperature, K |
| `planck_frequency_radiance` | frequency in **Hz**, temperature in K | `W m^-2 sr^-1 Hz^-1` |
| `inverse_planck_frequency` | frequency in Hz, frequency spectral radiance | Monochromatic brightness temperature, K |

Wavelength input always stays in nm, even when output density is per um.
`radiance_unit` explicitly selects one of `W m^-2 sr^-1 nm^-1`,
`W m^-2 sr^-1 um^-1`, or `W m^-2 sr^-1 m^-1`. All results are broadcast NumPy
float64 arrays, including zero-dimensional scalar results. These functions use
strict per-computed-element errors; the separate calibration functions continue
to return their `values/valid/flags/unit` envelopes.

Radiance is per steradian; for an ideal Lambertian blackbody, spectral exitance
is **pi times** spectral radiance. Frequency and wavelength densities are not
equal numbers: `B_nu = B_lambda(per m) * lambda_m**2/c`. Wavelength density per
nm must first be multiplied by `1e9` for that expression.

Coordinates must be positive and finite; temperature and radiance must be
nonnegative and finite. `T=0` returns zero radiance, and inverse zero radiance
returns the limiting temperature 0 K. An extreme Wien tail may underflow to
zero in float64, in which case its original positive temperature cannot be
recovered. Actual output overflow raises `ModelError`. Logarithmic evaluation,
`expm1`/`log1p` and asymptotic branches avoid naive exponential overflow and
Rayleigh–Jeans cancellation.

Constants are exported from the Rust kernel as `radiometry.PLANCK_CONSTANT`,
`SPEED_OF_LIGHT`, and `BOLTZMANN_CONSTANT`. Their SI defining values are
`6.62607015e-34 J s`, `299792458 m/s`, and `1.380649e-23 J/K` respectively
([NIST SI definitions](https://www.nist.gov/pml/special-publication-330/sp-330-section-2)).
Planck's wavelength/radiance convention follows
[CIE e-ILV, Planck's law](https://cie.co.at/eilvterm/17-24-005).

The inverse gives **monochromatic brightness temperature**, not a fitted
sensor-band temperature, emissivity-corrected surface temperature or LST.
The existing `radiance_to_brightness_temperature(...,k1=...,k2=...)` remains the
separate product/band-specific calibration path. No atmosphere, emissivity,
sensor response or model-to-model data connection is applied automatically.

## Validation and runnable example

```sh
python benchmarks/reference/basic-physics/verify.py
python -m pytest bindings/python/tests/test_basic_physics.py -q
python benchmarks/reference/basic-physics/report.py
python bindings/python/examples/basic_physics.py
```

The fixture contains 32 Fresnel cases, 32 wavelength-Planck cases and 32
frequency-Planck cases, evaluated independently at **80 decimal digits** with
mpmath 1.3.0. Regenerate using `benchmarks/reference/basic-physics/generate.py`;
normal tests do not require mpmath or a network. These are independently
evaluated published equations, not original vendor software output.

Fresnel tests use mixed tolerance `2e-12 + 2e-11*abs(reference)`; thermal tests
use `2e-12` relative with exact zero/underflow comparisons. Near a critical
angle, sensitivity of the square root increases and index/angle floating-point
rounding matters. Additional physical checks cover normal incidence, Brewster
reflection, TIR phase, matched grazing media, power conservation, permittivity
equivalence, density-unit/Jacobian conversion, Wien's peak, Rayleigh–Jeans and
Stefan–Boltzmann limits. The Stefan–Boltzmann check numerically integrates a
wide log-spaced spectrum, so it has its own discretization tolerance.

Measured errors are recorded in
`benchmarks/reference/basic-physics/validation-report.json`. Validation applies
to these operators and stated domains; it is not empirical validation of a
surface material or a satellite product. The example prints independently
verified interface coefficients and blackbody spectra without coupling models.
