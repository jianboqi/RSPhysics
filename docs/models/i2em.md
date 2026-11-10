# I²EM: rough-surface radar backscatter

I²EM computes the normalized radar cross section of a single-scale random
rough surface from its dielectric properties, height statistics and radar
geometry. This interface provides monostatic HH and VV co-polarization for
isotropic exponential or Gaussian height autocorrelation. It is useful for
studying bare-soil radar response without an empirical soil-moisture conversion.

The scientific model is attributed to **Adrian Fung**. The reference description
appears in **Fawwaz T. Ulaby and David G. Long (2014)**, section 10-3.9.

## Basic use

```python
from rsphysics.microwave import i2em

simulation = i2em(
    frequency_ghz=5.3,
    permittivity_real=10.0,
    permittivity_loss=1.0,
    rms_height_m=0.005,
    correlation_length_m=0.06,
    observation_zenith_deg=30.0,
    correlation_function="exponential",
)
result = simulation["result"]
print(result["sigma0_vv_db"], result["sigma0_hh_db"])
# Approximately -9.7968 dB and -11.6216 dB.
```

| Parameter | Unit | Meaning |
| --- | --- | --- |
| `frequency_ghz` | GHz | Radar frequency |
| `permittivity_real` | 1 | Real relative permittivity |
| `permittivity_loss` | 1 | Nonnegative dielectric loss, ε = ε′ − i ε″ |
| `rms_height_m` | m | Surface RMS height s |
| `correlation_length_m` | m | Correlation length L |
| `observation_zenith_deg` | degrees | Incidence angle from the surface normal |
| `correlation_function` | — | `exponential` or `gaussian` |

Height and correlation length are in **meters**, not centimeters. Permittivity
is supplied directly; this model does not calculate it from soil moisture.

## Principle and outputs

The surface spectrum is obtained from powers of the height autocorrelation:
exponential `ρ(r) = exp(−r/L)` or Gaussian `ρ(r) = exp(−(r/L)²)`. I²EM combines
this spectrum with polarization-dependent dielectric terms, a roughness
transition of the reflection coefficient, and a shadowing correction to
calculate the backscatter series.

`sigma0_vv` and `sigma0_hh` are **linear power** quantities. Their dB forms are
`sigma0_vv_db` and `sigma0_hh_db`, defined as `10 log10(σ⁰)`. They are neither
electric-field amplitudes nor optical reflectances; σ⁰ can exceed one.
`ks` and `height_length_ratio` report dimensionless roughness and s/L.

## NumPy batches and parameter sweeps

```python
import numpy as np
import rsphysics as rsp
from rsphysics.microwave import i2em_batch, I2EM_PARAMETERS

table = rsp.parameter_grid(
    "microwave.i2em", {"observation_zenith_deg": [20., 30., 40., 50.]}
)
batch = i2em_batch(table, correlation_function="gaussian")
assert table.shape == (4, len(I2EM_PARAMETERS))
assert batch["result"]["sigma0_vv_db"].shape == (4,)

# The common API accepts the same selector and supports chunked evaluation.
chunks = list(rsp.iter_array(
    "microwave.i2em", table, chunk_size=2,
    fixed={"correlation_function": "gaussian"},
))
np.testing.assert_array_equal(
    np.concatenate([c["result"]["sigma0_vv_db"] for c in chunks]),
    batch["result"]["sigma0_vv_db"],
)
```

## Applicability

The reference constrains **ks < 1** and **s/L ≤ 0.25**. Both restrictions are
checked for every input row. The implementation accepts incidence angles
**1–80°**; exact normal and grazing incidence are excluded. These conditions
do not guarantee accuracy for every measured surface: the single-scale,
isotropic covariance assumption must also be appropriate.

The selected numerical formulation uses `k = 2π f / 0.30` in m⁻¹ with f in GHz
and an incident-angle offset of **0.01 rad** in the incident trigonometric
terms, while retaining the specified scattering angle. These conventions
are part of this formulation and affect numerical comparisons.

Extremely weak responses that underflow floating-point arithmetic return a
domain error; no arbitrary dB floor is applied. HV polarization, bistatic
geometry, x-power/x-exponential covariance, vegetation scattering, inversion
and model coupling are outside this interface.

## References

- Fung, A. (2016). *I2EM Backscattering from Single-Scale Random Surface*.
  IEEE-GRSS Remote Sensing Code Library.
  [DOI: 10.21982/M8BC7Z](https://doi.org/10.21982/M8BC7Z).
- Ulaby, F. T. & Long, D. G. (2014). *Microwave Radar and Radiometric Remote
  Sensing*. University of Michigan Press, section 10-3.9.
  ISBN 978-0-472-11935-6.
  [Book resources](https://ulabyweb.eecs.umich.edu/mrs/).
