# FLUSPECT-CX: leaf optics and fluorescence

FLUSPECT-CX predicts leaf reflectance, transmittance and chlorophyll fluorescence
transfer from leaf structure and biochemical composition. It extends FLUSPECT-B
with a violaxanthin–zeaxanthin transition parameter to describe xanthophyll-related
changes in visible reflectance.

The model was developed by **Nastassia Vilfan, Christiaan van der Tol, Wout
Verhoef and colleagues**. This interface uses PROSPECT-D optical coefficients
and the SCOPE light spectral grid. Photosystems I and II are combined.

## Basic use

```python
from rsphysics.vegetation import fluspect_cx

simulation = fluspect_cx(
    n=1.5, chlorophyll=40, carotenoids=8, anthocyanins=0,
    water=0.01, dry_matter=0.009,
    xanthophyll_state=0.5, fluorescence_efficiency=0.01,
)
leaf = simulation["result"]
assert leaf["reflectance"].shape == (2001,)
assert leaf["fluorescence_backward"].shape == (53, 71)
```

| Parameter | Unit | Meaning |
| --- | --- | --- |
| `n` | 1 | Leaf mesophyll structure parameter, 1–5 |
| `chlorophyll` | µg cm⁻² | Chlorophyll a+b |
| `carotenoids` | µg cm⁻² | Total carotenoids |
| `anthocyanins` | µg cm⁻² | Anthocyanins |
| `brown_pigments` | 1 | Brown/senescent material |
| `water` | g cm⁻² | Equivalent water thickness |
| `dry_matter` | g cm⁻² | Dry matter content |
| `xanthophyll_state` | 1 | Violaxanthin-to-zeaxanthin state, 0–1 |
| `fluorescence_efficiency` | 1 | Combined-photosystem fluorescence efficiency |

Zero fluorescence efficiency produces zero fluorescence matrices while leaving
reflectance and transmittance unchanged. Efficiency scales the matrices linearly.
Defaults illustrate the model and are not a species-specific calibration.

## Principles and output conventions

Leaf absorption coefficients and interface transmission first determine the
optical response. FLUSPECT removes the leaf–air interfaces, derives absorption
and scattering coefficients for the mesophyll, and calculates fluorescence using
15 doubling steps. The interfaces are then added back to obtain the leaf response.

Reflectance and transmittance use **400–2400 nm at 1 nm intervals**. Illumination
transmission follows the fixed 59° convention in this formulation. This is not
an additional configurable solar/view geometry.

`fluorescence_backward` describes emission toward the illuminated side;
`fluorescence_forward` describes emission toward the opposite side. Each is a
matrix with **emission wavelength in rows** and **excitation wavelength in columns**:

- `emission_wavelengths_nm`: 640, 644, …, 848 nm — 53 rows.
- `excitation_wavelengths_nm`: 400, 405, …, 750 nm — 71 columns.

The matrices are **transfer responses**, not absolute canopy SIF or directional
radiance. They already include the **5 nm excitation-bin integration weight**.
For a separately specified compatible spectral irradiance vector, use a matrix
product without multiplying by 5 again. Illumination, atmospheric transmission,
and canopy escape are not assumed or calculated here.

`chlorophyll_absorption_fraction` and `carotenoid_absorption_fraction` are the
relative contributions to bulk absorption. They are not the leaf absorptance
`1 − reflectance − transmittance`.

## NumPy batches

```python
import rsphysics as rsp
from rsphysics.vegetation import fluspect_cx_batch

table = rsp.parameter_grid(
    "vegetation.fluspect-cx", {"chlorophyll": [20., 40., 60.]}
)
batch = fluspect_cx_batch(table, shared_axes=True)
assert batch["result"]["reflectance"].shape == (3, 2001)
assert batch["result"]["fluorescence_backward"].shape == (3, 53, 71)
assert batch["coordinates"]["excitation_wavelengths_nm"].shape == (71,)
```

## Scope

This is a leaf-scale, combined-photosystem forward model. It does not separate
PSI/PSII, implement the PRO protein/carbon-based coefficient set, calculate
photosynthetic regulation or energy balance, or perform canopy/atmosphere
coupling or inversion. The conservative zero-absorption case uses the analytic
Stokes limit to avoid an ill-conditioned floating-point division.

## References

- Vilfan, N., van der Tol, C., Muller, O., Rascher, U. & Verhoef, W. (2016).
  *Fluspect-B: A model for leaf fluorescence, reflectance and transmittance
  spectra*. Remote Sensing of Environment, 186, 596–615.
  [DOI: 10.1016/j.rse.2016.09.017](https://doi.org/10.1016/j.rse.2016.09.017).
- Vilfan, N., van der Tol, C., Yang, P., Wyber, R., Malenovský, Z., Robinson,
  S. A. & Verhoef, W. (2018). *Extending Fluspect to simulate xanthophyll driven
  leaf reflectance dynamics*. Remote Sensing of Environment, 211, 345–356.
  [DOI: 10.1016/j.rse.2018.04.012](https://doi.org/10.1016/j.rse.2018.04.012).
