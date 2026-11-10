# RSPhysics

**A scientific computing library for remote sensing physics.**

[![Checks](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/ci.yml)
[![Wheels](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml/badge.svg)](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)

Physical models connect remote sensing measurements to the properties and
processes of the Earth system. They provide a basis for interpreting observations,
testing hypotheses, and designing retrieval methods. Yet their implementations
are scattered across languages, software packages, and research code, making them
difficult to discover, use, and extend together.

RSPhysics brings these models into a shared scientific computing library. Its
aim is to make established remote sensing models more accessible, support their
use in research and education, and develop consistent conventions for model
implementation, interfaces, and validation.

## Approach

RSPhysics provides a common framework while preserving each model's physical
assumptions, parameters, and domain of applicability.

- **Shared implementations.** Scientific solvers are implemented in Rust and
  exposed through Python, providing a common numerical foundation for scripts,
  batch simulations, and applications.
- **Explicit physical meaning.** Model interfaces describe inputs, outputs,
  units, and valid domains, so a consistent API also carries meaningful
  scientific definitions.
- **Traceable validation.** Implementations are checked against reference code
  or independent calculations, with documented sources, versions, and numerical
  tolerances.

The library includes models for vegetation optics, surface directional
reflectance, microwave vegetation response, and atmospheric correction, alongside
spectral and radiometric tools. Coverage grows through documented, validated
contributions.

## Getting started

Start with the [installation guide](docs/installation.md). The
[Wheels workflow](https://github.com/jianboqi/RSPhysics/actions/workflows/wheels.yml)
provides tested builds for Windows, Linux, and macOS after each successful run.

The Python interface supports model-specific functions and NumPy arrays:

```python
import rsphysics as rsp

leaf = rsp.vegetation.prospect_d(chlorophyll=45, water=0.015)

wavelengths = leaf["result"]["wavelengths_nm"]
reflectance = leaf["result"]["reflectance"]
transmittance = leaf["result"]["transmittance"]
```

See the [Python guide](bindings/python/README.md) for installation, model
interfaces, and batch computation. The [examples](bindings/python/examples)
demonstrate complete workflows, including spectral simulation and sensor-band
calculations.

## Documentation

- [Installation and prebuilt wheels](docs/installation.md)
- [Documentation index](docs/README.md)
- [Spectral, sensor, and radiometric tools](docs/python-infrastructure.md)
- [Fundamental physics tools](docs/basic-physics.md)
- [Model interface specification](docs/architecture/model-api-v0.1.md)
- [Reference benchmarks](benchmarks/reference/README.md)

## Contributing

Contributions of models, reference cases, scientific documentation, and numerical
improvements are welcome. The [contribution guide](CONTRIBUTING.md) describes the
model template, interface conventions, and validation requirements. Contributions
should preserve scientific attribution and document the scope and limitations of
each implementation.

## License

Project-original code is licensed under [Apache-2.0](LICENSE). Included model
implementations and resources retain applicable third-party licenses, including
MIT and GPLv3; see [license details](docs/licensing.md) and
[third-party notices](bindings/python/python/rsphysics/THIRD_PARTY_NOTICES.md).
