# RSPhysics

**A scientific computing library for remote sensing physics.**

[![Checks](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml)
[![Wheels](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)

RSPhysics brings remote sensing physical models into a shared scientific
computing library. It makes models scattered across research software easier to
use, compare, and extend through consistent interfaces and physical conventions.
Rust implementations and Python/NumPy interfaces support research, education,
and application development.

## Installation

PyPI installation, available after the first package release:

```sh
pip install rsphysics
```

For current builds, install a [prebuilt wheel](docs/installation.md).

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
| Vegetation optics | [PROSPECT-D](docs/models/prospect-d.md) | Leaf reflectance and transmittance |
| Vegetation optics | [PROSAIL-D](docs/models/prosail-d.md) | Canopy reflectance |
| Surface reflectance | [Ross–Li](docs/models/ross-li.md) | Directional reflectance and BRDF |
| Passive microwave | [Tau–omega](docs/models/tau-omega.md) | Soil–vegetation brightness temperature |
| Passive microwave | [L-MEB](docs/models/lmeb.md) | L-band dual-polarization emission |
| Active microwave | [Water-cloud model](docs/models/water-cloud.md) | Vegetation and soil radar backscatter |
| Atmosphere | [SMAC](docs/models/smac.md) | Band-wise atmospheric correction |
| Ecosystem processes | [GORT-GPP](docs/models/gort-gpp.md) | Canopy photosynthesis and net carbon assimilation |

## Documentation

[Documentation](docs/README.md) covers basic usage, physical tools, and individual
models with parameter definitions, examples, authors, and references.

## Build from source

See [Build from source](docs/build-from-source.md).

## License

Project-original code is licensed under [Apache-2.0](LICENSE). Included models
and resources retain their applicable [component licenses](LICENSES.md).
