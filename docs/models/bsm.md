# BSM soil reflectance

The Brightness–Shape–Moisture model simulates isotropic dry and wet soil
reflectance from brightness, spectral shape and moisture. It combines three dry
spectral vectors and computes a Poisson ensemble of water-film thicknesses,
including water absorption and repeated water–soil interface reflections.

Available in the current source tree; install [from source](../build-from-source.md).
The published PyPI 0.1.0 wheels do not contain this model.

```python
import rsphysics as rsp

soil = rsp.soil.bsm(
    soil_brightness=0.5,
    shape_latitude_deg=-10,
    shape_longitude_deg=100,
    soil_moisture_percent=25,
    moisture_capacity_percent=25,
    water_film_thickness_cm=0.015,
)
dry = soil["result"]["dry_reflectance"]
wet = soil["result"]["wet_reflectance"]
```

The axis is 400–2400 nm at 1 nm spacing, with 2001 samples. The shape coordinates
convert to dry-vector weights as `B*sin(lat)`, `B*cos(lat)*sin(lon)` and
`B*cos(lat)*cos(lon)`. Typical shape ranges in the author example are −30–30°
latitude and 80–120° longitude. Supported numeric ranges are wider, but a
combination producing dry reflectance outside [0,1] is rejected.

Moisture uses **percentage points**: `25` means 25%, not 0.25. The expected film
count is `mu=max(0,(SMp-5)/SMC)`; `SMC` defaults to 25. At moisture ≤5%, the wet
output equals the dry output. Elementary water-film thickness is in **cm**,
consistent with the bundled water absorption coefficients in cm⁻¹. Zero film
thickness removes absorption, but wet interfaces can still change reflectance.

`poisson_mode="complete"` sums the published distribution until omitted
probability is below 1e−14. `"legacy_six_films"` reproduces the finite 0–6-film
approximation. The latter is not renormalized; at mu=2 it omits about 0.45% of
probability. Outputs expose `mean_film_count`, `dry_area_fraction`,
`poisson_probability_sum` and `omitted_probability`.

```python
import rsphysics as rsp

p = rsp.parameter_grid("soil.bsm", {"soil_moisture_percent": [0, 20, 50]})
batch = rsp.soil.bsm_batch(p)
assert batch["result"]["wet_reflectance"].shape == (3, 2001)
```

This is an isotropic optical soil model. It does not compute directional BRDF,
retrieve moisture, or call a vegetation/atmosphere model.

Model development: Wout Verhoef and Peiqi Yang. Full model description:
Yang, P., van der Tol, C., Yin, T. & Verhoef, W. (2020). *The SPART model: A
soil-plant-atmosphere radiative transfer model for satellite measurements in the
solar spectrum*. RSE 247, 111870. [Paper](https://doi.org/10.1016/j.rse.2020.111870).
