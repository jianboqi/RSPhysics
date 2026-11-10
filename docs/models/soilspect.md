# SOILSPECT soil reflectance

SOILSPECT is a Hapke-derived optical soil model that separates single and
multiple scattering. Inputs are single-scattering albedo for a selected band,
four phase-function coefficients, shadow-hiding width and illumination/view
geometry. The same equations can be evaluated independently for a supplied
single-scattering-albedo spectrum through NumPy batches.

Available in the current source tree; install [from source](../build-from-source.md).
The published PyPI 0.1.0 wheels do not contain this model.

```python
import rsphysics as rsp

soil = rsp.soil.soilspect(
    single_scattering_albedo=0.6, roughness_h=0.1,
    phase_b=0.3, phase_c=0.1, phase_b_prime=0.1, phase_c_prime=0.05,
    solar_zenith_deg=30, view_zenith_deg=20, relative_azimuth_deg=0,
    quadrature="refined",
)
brf = soil["result"]["brf"]
dhr = soil["result"]["directional_hemispherical_reflectance"]
```

`BRF = w/[4*(mu_s+mu_v)] * [(1+B)*P + H_s*H_v-1]`.
The phase function includes all four coefficients:
`P=1+b*cos(g)+c*P2(cos(g))+b_prime*cos(g_prime)+c_prime*P2(cos(g_prime))`.
The hotspot function is `B=1/(1+tan(g/2)/h)` and
`H(mu)=(1+2*mu)/(1+2*mu*sqrt(1-w))`. `h=0` uses the zero-width hotspot limit.

Albedo `w` is **not soil reflectance**. It must be supplied externally; this
model does not estimate it from moisture, texture or mineral fractions.
The four phase coefficients are all active. Negative/nonphysical algebraic
responses remain visible through `negative_sample_count`; BRF may exceed one.

Angles are in degrees. Relative azimuth 0° is backward/hotspot and 180° forward.
The principal-plane and hemisphere grids use the same conventions as
[RPV](rpv.md). Selected outputs include `single_scattering_brf`,
`multiple_scattering_brf`, total `brf` and `brdf=brf/pi`.

Both `directional_hemispherical_reflectance` and
`bihemispherical_reflectance` integrate the **complete** BRF expression,
including hotspot and all four phase terms. `quadrature="standard"` returns a
32×64 Gaussian angular integral; `"refined"` uses 64×128. Both orders are
evaluated, and `integration_error_estimate` reports their maximum absolute
difference across the two flux quantities. It is a convergence diagnostic,
not a rigorous bound. Narrow hotspots require particular attention to this
diagnostic. Albedos 0, 0.75 and 1 are evaluated without removable singularities
from closed-form integral approximations.

```python
import rsphysics as rsp

p = rsp.parameter_grid("soil.soilspect", {"single_scattering_albedo": [0.2, 0.5, 0.8]})
batch = rsp.soil.soilspect_batch(p, quadrature="refined", shared_axes=True)
assert batch["result"]["brf"].shape == (3,)
```

Authors: Stéphane Jacquemoud, Frédéric Baret and Jean-François Hanocq.

- Jacquemoud, S., Baret, F. & Hanocq, J.-F. (1992). *Modeling spectral and
  bidirectional soil reflectance*. RSE 41, 123–132.
  [Paper](https://doi.org/10.1016/0034-4257(92)90072-R).
- Baret, F., Jacquemoud, S. & Hanocq, J.-F. (1993). *About the soil line concept
  in remote sensing*. Advances in Space Research 13(5), 281–284.
  [Paper](https://doi.org/10.1016/0273-1177(93)90560-X).
