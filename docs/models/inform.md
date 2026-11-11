# INFORM

The Invertible Forest Reflectance Model (INFORM) estimates the directional reflectance of a forest stand. Clement Atzberger and Martin Schlerf combined FLIM crown and shadow geometry with SAIL radiative transfer in the crowns and understory.

The model first computes the reflectance of the understory over a Lambertian soil. It then computes dense-crown reflectance and the effective crown transmittance in the solar and viewing directions. The forest spectrum is a weighted sum of dense-crown and understory reflectance. The weights depend on the four areas formed by intersecting projected crowns and their shadows. The Lambertian equivalent used for feedback into the crown and the effective transmittances used as mixing weights are bounded to [0, 1].

## Run in Python

```python
from rsphysics.vegetation import inform

out = inform(
    stem_density_per_ha=650,
    tree_height_m=20,
    crown_diameter_m=4.5,
    crown_lai=7,
    understory_lai=0.6,
    solar_zenith_deg=30,
    view_zenith_deg=15,
    relative_azimuth_deg=60,
    diffuse_fraction=0.1,
    crown_leaf={"chlorophyll": 60, "water": 0.025},
    understory_leaf={"chlorophyll": 40},
)["result"]

wavelength = out["wavelengths_nm"]
forest = out["reflectance"]
understory = out["understory_reflectance"]
view_transmission = out["crown_transmittance_view"]
```

Leaf area indices are in m²/m². Crown LAI is defined for a single crown. The default leaf spectra are computed with PROSPECT-D from 400 to 2500 nm at 1 nm spacing. The solar and viewing azimuths coincide at a relative azimuth of zero.

The result includes the soil and dense-crown spectra, leaf reflectance and transmittance for both layers, solar and viewing crown transmission, and the two spectral weights in the forest reflectance equation. The four crown and gap fractions are wavelength-independent. Directional reflectance factors can exceed one.

## Use measured spectra

Supply a common, strictly increasing wavelength grid and a soil spectrum. The crown and understory leaf spectra are optional and independent. A layer without measured optics uses PROSPECT-D interpolated to the supplied grid.

```python
out = inform(spectra={
    "wavelengths_nm": [550, 680, 850, 1650],
    "soil_reflectance": [0.12, 0.15, 0.22, 0.30],
    "crown_leaf": {
        "reflectance": [0.11, 0.04, 0.46, 0.25],
        "transmittance": [0.09, 0.03, 0.43, 0.18],
    },
})["result"]
```

For PROSPECT 3.01, supply `spectra["prospect_301_coefficients"]` with arrays named `refractive_index`, `albino_absorption`, `chlorophyll_absorption`, `water_absorption`, and `dry_matter_absorption` on the same grid. Leaf structure, chlorophyll, water, and dry matter are taken from each layer's parameters. PROSPECT 3.01 uses a 60° incidence cone. The historical coefficient table is supplied by the caller.

## Run a parameter grid

```python
import rsphysics as rsp

params = rsp.parameter_grid("vegetation.inform", {"view_zenith_deg": [0, 15, 30, 45]})
batch = rsp.vegetation.inform_batch(params)
assert batch["result"]["reflectance"].shape == (4, 2101)
```

The batch shares one wavelength axis and can take fixed `crown_leaf`, `understory_leaf`, and `spectra` inputs. Use `rsp.run_batch` with per-run dictionaries if leaf parameters or spectral grids vary among runs.

## Applicability

INFORM assumes randomly distributed tree crowns, homogeneous leaf layers within the crowns and understory, and Lambertian soil. Use measured soil spectra for wavelength-dependent backgrounds. The implementation provides forward calculations.

## References

- Atzberger, C. (2000). Development of an invertible forest reflectance model: the INFOR-model. In *A Decade of Trans-European Remote Sensing Cooperation*, pp. 39–44.
- Schlerf, M., & Atzberger, C. (2006). Inversion of a forest reflectance model to estimate structural canopy variables from hyperspectral remote sensing data. *Remote Sensing of Environment*, 100(3), 281–294. [DOI](https://doi.org/10.1016/j.rse.2005.10.006)
- Yuan, H., Ma, R., Atzberger, C., Li, F., Loiselle, S. A., & Luo, J. (2015). Estimating forest fAPAR from multispectral Landsat-8 data using the invertible forest reflectance model INFORM. *Remote Sensing*, 7(6), 7425–7446. [DOI](https://doi.org/10.3390/rs70607425)
