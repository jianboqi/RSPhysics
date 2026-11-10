# FLUSPECT-CX: leaf optics and fluorescence

FLUSPECT-CX predicts leaf reflectance, transmittance and chlorophyll fluorescence
transfer from leaf structure and biochemical composition. It extends FLUSPECT-B
with a violaxanthin–zeaxanthin transition parameter to describe xanthophyll-related
changes in visible reflectance.

The model was developed by **Nastassia Vilfan, Christiaan van der Tol, Wout
Verhoef and colleagues**. This interface uses PROSPECT-D optical coefficients
with a full 1 nm fluorescence grid by default. An optional light grid matches the SCOPE 5/4 nm calculation. Photosystems I and II are combined.

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
assert leaf["fluorescence_backward"].shape == (211, 351)
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
| `xanthophyll_state` | 1 | Violaxanthin-to-zeaxanthin state 0–1, or −999 for fixed carotenoid absorption |
| `fluorescence_efficiency` | 1 | Combined-photosystem fluorescence efficiency |
| `protein` | g cm⁻² | PRO protein content |
| `carbon_based_constituents` | g cm⁻² | PRO carbon-based constituent content |
| `optical_coefficients` | — | `prospect_d` or `prospect_pro` |
| `spectral_resolution` | — | `full_1nm` (default) or `light_5x4` |

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

- Full grid: emission 640–850 nm, 211 rows; excitation 400–750 nm, 351 columns. Both grids are 1 nm.
- Light grid: emission 640–848 nm / 4 nm, 53 rows; excitation 400–750 nm / 5 nm, 71 columns.

The matrices are **photon-domain transfer responses**, not absolute canopy SIF
or directional radiance. They include the selected **excitation-bin width**
(1 or 5 nm); do not multiply by this width a second time. To apply a measured
energy irradiance spectrum, convert through photon flux. The Rust-backed helper
implements `F_energy[f] = sum(M[f,e] * E_energy[e] * lambda_exc[e]/lambda_em[f])`.
Planck's constant and the speed of light cancel in this wavelength ratio.

```python
import numpy as np
from rsphysics.vegetation import fluspect_fluorescence

incident = np.ones(351)  # W m^-2 nm^-1 on the full excitation grid
emitted = fluspect_fluorescence(incident)
assert emitted["backward_w_m2_nm"].shape == (211,)
```

PRO mode accepts the original protein and carbon-based constituent inputs.
Avoid double counting those constituents in the total dry matter term.

```python
pro_leaf = fluspect_cx(
    optical_coefficients="prospect_pro", dry_matter=0,
    protein=0.001, carbon_based_constituents=0.008,
)
assert pro_leaf["result"]["fluorescence_forward"].shape == (211, 351)
```

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
assert batch["result"]["fluorescence_backward"].shape == (3, 211, 351)
assert batch["coordinates"]["excitation_wavelengths_nm"].shape == (351,)
```

## Scope

This is a leaf-scale, combined-photosystem forward model. It does not separate
PSI/PSII, calculate
photosynthetic regulation or energy balance, or perform canopy/atmosphere
coupling or inversion. The conservative zero-absorption case uses the analytic
Stokes limit to avoid an ill-conditioned floating-point division. The negative
641 nm emission-coefficient calibration residue in the D table is set to zero
when computing emission. Raw coefficient values and unmodified negative-response
rows are retained separately for verification.

## References

- Vilfan, N., van der Tol, C., Muller, O., Rascher, U. & Verhoef, W. (2016).
  *Fluspect-B: A model for leaf fluorescence, reflectance and transmittance
  spectra*. Remote Sensing of Environment, 186, 596–615.
  [DOI: 10.1016/j.rse.2016.09.017](https://doi.org/10.1016/j.rse.2016.09.017).
- Vilfan, N., van der Tol, C., Yang, P., Wyber, R., Malenovský, Z., Robinson,
  S. A. & Verhoef, W. (2018). *Extending Fluspect to simulate xanthophyll driven
  leaf reflectance dynamics*. Remote Sensing of Environment, 211, 345–356.
  [DOI: 10.1016/j.rse.2018.04.012](https://doi.org/10.1016/j.rse.2018.04.012).
