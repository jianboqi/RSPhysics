<p align="center">
  <img src="docs/assets/rsphysics-logo.png" alt="RSPhysics" width="460">
</p>

# RSPhysics

**A scientific computing library for remote sensing physics.**

[![Checks](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml)
[![Wheels](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)

RSPhysics brings remote sensing physical models into a shared scientific
computing library. It makes models scattered across research software easier to
use, compare, and extend through consistent interfaces and physical conventions.
Native numerical kernels and Python/NumPy interfaces support research, education,
and application development.

## Installation

```sh
pip install rsphysics
```

See [Installation](docs/installation.md) for supported platforms and prebuilt wheels.

## Quick start

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)
wavelengths = leaf["result"]["wavelengths_nm"]
reflectance = leaf["result"]["reflectance"]
```

## Models

| Category | Model | Main purpose |
|---|---|---|
| Vegetation optics | [FLUSPECT-CX](docs/models/fluspect-cx.md) | Leaf optics and fluorescence transfer |
| Vegetation optics | [PROSPECT-D](docs/models/prospect-d.md) | Leaf reflectance and transmittance |
| Vegetation optics | [PROSAIL-D](docs/models/prosail-d.md) | Canopy reflectance |
| Vegetation optics | [GOMS](docs/models/goms.md) | Discrete-crown reflectance and mutual shadowing |
| Thermal infrared | [4SAIL-TIR](docs/models/foursail-tir.md) | Four-component canopy thermal radiance and brightness temperature |
| Surface reflectance | [Ross–Li](docs/models/ross-li.md) | Directional reflectance and BRDF |
| Surface reflectance | [RPV](docs/models/rpv.md) | Semi-empirical directional reflectance and hotspot |
| Passive microwave | [Tau–omega](docs/models/tau-omega.md) | Soil–vegetation brightness temperature |
| Passive microwave | [L-MEB](docs/models/lmeb.md) | L-band dual-polarization emission |
| Active microwave | [I²EM](docs/models/i2em.md) | Rough bare-surface HH/VV radar backscatter |
| Active microwave | [Water-cloud model](docs/models/water-cloud.md) | Vegetation and soil radar backscatter |
| Soil optics | [SORT](docs/models/sort.md) | Dry and wet soil directional reflectance |
| Soil optics | [SORT-L](docs/models/sort-l.md) | Lambertian dry and wet soil reflectance |
| Soil optics | [SOILSPECT](docs/models/soilspect.md) | Single/multiple scattering and hemispherical reflectance |
| Soil optics | [BSM](docs/models/bsm.md) | Brightness, shape and water-film moisture effects |
| Soil optics | [GSV](docs/models/gsv.md) | GSV1/GSV3 hyperspectral soil spectra |
| Atmosphere | [SMAC](docs/models/smac.md) | Band-wise atmospheric correction |
| Atmosphere | [6SV2.1](docs/models/sixsv21.md) | Vector atmospheric radiative transfer and correction |
| Ecosystem processes | [GORT-GPP](docs/models/gort-gpp.md) | Canopy photosynthesis and net carbon assimilation |
| Ecosystem processes | [FvCB Leaf](docs/models/fvcb.md) | C₃ leaf biochemistry and alternative stomatal formulations |
| Ecosystem processes | [FvCB Big Leaf](docs/models/fvcb-big-leaf.md) | Homogeneous canopy carbon and water-vapour fluxes |

FvCB, RPV, SOILSPECT, BSM, GSV and 6SV2.1 currently require a source installation and are not included in
the published PyPI 0.1.0 wheels.

## Documentation

[Documentation](docs/README.md) covers basic usage, physical tools, and individual
models with parameter definitions, examples, authors, and references.

## Build from source

See [Build from source](docs/build-from-source.md).

## License

Project-original code is licensed under [Apache-2.0](LICENSE). Included models
and resources retain their applicable [component licenses](LICENSES.md).

Model authors and rights holders are welcome to [contact the maintainers](https://github.com/jianboqi/RSPhysics/issues)
with any concerns about inclusion, attribution, or licensing. We will review each
request and make appropriate corrections or remove the affected material where necessary.
