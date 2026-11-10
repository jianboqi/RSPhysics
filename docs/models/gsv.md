# GSV soil spectra

GSV describes hyperspectral soil reflectance with a small set of empirical
spectral-vector coefficients. GSV3 combines three dry-soil vectors and a moisture
vector. GSV1 uses only the first dry vector and the moisture vector.

Available in the current source tree; install [from source](../build-from-source.md).
The published PyPI 0.1.0 wheels do not contain this model.

```python
import rsphysics as rsp

soil = rsp.soil.gsv(c1=0.4, c2=-0.01, c3=0.04, csm=-0.13)
wavelengths = soil["result"]["wavelengths_nm"]
reflectance = soil["result"]["reflectance"]

# Either variant accepts explicit wavelengths, interpolated in Rust.
sampled = rsp.soil.gsv(variant="gsv1", c1=0.4, csm=-0.13,
                       wavelengths_nm=[400, 405.5, 850, 2500])
```

The default axis is 400–2500 nm at 10 nm spacing (211 samples). `None` or an
empty wavelength list selects that axis. Explicit requests retain their order
and may include repeated wavelengths. Linear interpolation is supported within
the domain; extrapolation is rejected.

The equation is `R = c1*G1 + c2*G2 + c3*G3 + csm*GSM`. GSV1 omits the second
and third terms, so `c2` and `c3` have no effect in that mode. `csm=0` represents
dry soil; negative values add the empirical moisture effect. The supported
moisture-coefficient interval is −0.6 to 0. **It is not volumetric soil moisture.**
The dry coefficients are finite algebraic weights, supported from −10 to 10;
their accepted numeric range does not establish physical validity.

Outputs include `dry_reflectance`, `moisture_contribution`, final `reflectance`,
and `outside_unit_interval_count`. Values outside [0,1] are retained and counted,
so unsuitable coefficient combinations remain visible rather than being clipped.

```python
import numpy as np
import rsphysics as rsp

# Columns: c1, c2, c3, csm. One shared wavelength request for the batch.
p = np.array([[0.4, -0.01, 0.04, -0.13], [0.7, 0, 0, 0]])
batch = rsp.soil.gsv_batch(p, wavelengths_nm=[400, 850, 1650, 2500])
assert batch["result"]["reflectance"].shape == (2, 4)
```

GSV synthesizes spectra; it does not compute angular reflectance, radiative
transport through a water film, or soil composition. No other model is invoked.

Authors: Chongya Jiang and Hongliang Fang.

Jiang, C. & Fang, H. (2019). *GSV: a general model for hyperspectral soil
reflectance simulation*. International Journal of Applied Earth Observation and
Geoinformation 83, 101932. [Paper](https://doi.org/10.1016/j.jag.2019.101932).
