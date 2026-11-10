# RPV directional reflectance

The Rahman–Pinty–Verstraete model describes the directional reflectance of a
surface with an amplitude, a modified Minnaert angular shape, a Henyey–Greenstein
phase function and a shadow-hiding hotspot term. It applies to vegetation and
other land surfaces as well as soil.

Available in the current source tree; install [from source](../build-from-source.md).
The published PyPI 0.1.0 wheels do not contain this model.

```python
import rsphysics as rsp

surface = rsp.surface.rpv(
    rho0=0.2, minnaert_k=0.7, phase_asymmetry=-0.2,
    solar_zenith_deg=30, view_zenith_deg=20, relative_azimuth_deg=0,
)
brf = surface["result"]["brf"]
brdf = surface["result"]["brdf"]
```

`BRF = rho0*M(k)*FHG(Theta)*H(rhoc)`, where:

- `M = [mu_s*mu_v*(mu_s+mu_v)]**(k-1)`.
- `FHG = (1-Theta²)/(1+Theta²+2*Theta*cos(g))**1.5`.
- `H = 1+(1-rhoc)/(1+G)`; `G` is the horizontal tangent-direction distance.

All angles are degrees. Relative azimuth 0° is backward/same solar azimuth,
and 180° is forward. Negative `phase_asymmetry` favours backward scattering;
positive values favour forward scattering. `minnaert_k<1` produces a bowl-shaped
response and `k>1` a bell-shaped response. The supported geometry excludes the
horizon (zenith 0–85°).

`hotspot_mode="original_three_parameter"` uses `rhoc=rho0`; `hotspot_rho` is
inactive in this mode. `"independent_four_parameter"` uses the supplied
`hotspot_rho`, with 1 disabling hotspot enhancement. This exposes both forms
without changing the meaning of the original three parameters.

Outputs include BRF, BRDF=BRF/π (sr⁻¹), the three multiplicative factors, a
171-point principal plane and a 648-point hemispherical direction grid. Positive
signed principal angles use relative azimuth 0°, negative angles use 180°.

```python
import rsphysics as rsp

p = rsp.parameter_grid("surface.rpv", {"phase_asymmetry": [-0.3, 0, 0.3]})
batch = rsp.surface.rpv_batch(p, hotspot_mode="independent_four_parameter",
                            shared_axes=True)
assert batch["result"]["brf"].shape == (3,)
```

This is a single-band semi-empirical model. It does not synthesize a spectrum or
infer coefficients. `rho0` is not a hemispherical albedo, BRF may exceed one,
and the parameter limits do not enforce hemispherical energy conservation.

Authors: H. Rahman, Bernard Pinty and Michel M. Verstraete (1993).
*Coupled surface-atmosphere reflectance (CSAR) model: 2. Semiempirical surface
model usable with NOAA AVHRR data*. JGR 98(D11), 20791–20801.
[Paper](https://doi.org/10.1029/93JD02023).
See also the [JRC RAMI definition](https://rami-benchmark.jrc.ec.europa.eu/_www/definition.php).
