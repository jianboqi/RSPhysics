# Ross–Li

Ross–Li describes directional surface reflectance as a weighted sum of isotropic, volumetric, and geometric scattering kernels.

## How it works

The supported formulation is RossThick–LiSparseReciprocal with fixed h/b=2 and b/r=1: `BRF = f_iso + f_vol*K_vol + f_geo*K_geo`. Weights refer to one spectral band.

## Usage

```python
import rsphysics as rsp

surface = rsp.surface.ross_li(
    f_iso=0.2, f_vol=0.1, f_geo=0.02,
    solar_zenith_deg=30., view_zenith_deg=20., relative_azimuth_deg=60.)
brf = surface['result']['brf']
brdf = surface['result']['brdf']
```

Model ID: `surface.ross-li`. Use the same ID with `rsp.run`, `rsp.run_array`, or
`rsp.parameter_grid`; see [Basic usage](../usage.md).

## Parameters

| Parameter | Default | Unit | Supported range |
|---|---|---|---|
| `f_geo` | `0.02` | 1 | 0.0–1.0 |
| `f_iso` | `0.2` | 1 | 0.0–1.0 |
| `f_vol` | `0.1` | 1 | 0.0–1.0 |
| `relative_azimuth_deg` | `0` | deg | 0.0–180.0 |
| `solar_zenith_deg` | `30` | deg | 0.0–85.0 |
| `view_zenith_deg` | `20` | deg | 0.0–85.0 |

## Outputs

| Field | Meaning |
|---|---|
| `brf / brdf` | Directional BRF and BRDF; BRDF = BRF/π, sr⁻¹ |
| `k_iso / k_vol / k_geo` | Kernel values |
| `principal_brf / hemisphere_brf` | Principal-plane and hemisphere responses |
| `negative_sample_count` | Number of negative BRF samples |

## Scope

This is a single-band angular model, not a reflectance spectrum. Nonnegative kernel weights do not guarantee physical reflectance at every angle; negative BRF is reported without clipping. No hotspot enhancement or albedo integration is applied.

## Authors and references

The Ross–Li kernel framework combines Ross volumetric and Li geometric scattering concepts. The MODIS semiempirical formulation is described by Wolfgang Lucht, Crystal Schaaf, and Alan Strahler.

- [Lucht, Schaaf & Strahler (2000). An algorithm for the retrieval of albedo from space using semiempirical BRDF models. IEEE TGRS.](https://doi.org/10.1109/36.841980)
