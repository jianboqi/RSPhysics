# Scientific conventions v0.1

- `SpectralAxis` is a nonempty, positive, finite, strictly increasing wavelength array in nm. Nonuniform sampling and a single wavelength are valid; resampling is never implicit.
- `Angle` stores/serializes radians, exposes `from_degrees`, and rejects NaN/infinity.
- `Geometry` solar/view zeniths are in [0, π/2]; relative azimuth is an unsigned angle in [0, π], with 0 denoting the same azimuth. Models needing directed azimuth, lower hemispheres, or other coordinates must define appropriate types and conversions.
- Spectral radiance uses W m^-2 sr^-1 nm^-1; spectral irradiance W m^-2 nm^-1; BRDF sr^-1; BRF, reflectance, transmittance, and hemispherical albedo use 1; brightness temperature uses K.
- Do not automatically exchange BRDF/BRF or treat arbitrary dimensionless arrays as reflectance. A universal [0,1] bound does not apply to every dimensionless radiometric quantity.
- `interpolate_linear` checks axis/value lengths, finite values, and query bounds; extrapolation is rejected. Implicit interpolation must not hide differences in model spectral domains.

`DomainError` describes an invalid foundational scientific type or numerical
primitive. Model boundaries map it to an appropriate `ModelError`. Model-specific
spectral shapes, anisotropy, and energy constraints are checked by each model.
