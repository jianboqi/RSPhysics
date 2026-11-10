# ADR 0005: Ross–Li dual references and angular outputs

Status: implemented and locally verified at the owner's request, 2026-11-09.

The third independent model is `surface.ross-li`, fixed RossThick–LiSparseReciprocal,
h/b=2 and b/r=1, without hotspot enhancement, inversion, or generic coupling.
The core targets native/Python/pip; RSPhysics-app declares its own WASM capability/evidence.

The scientific team's original FTP download timed out. Use an immutable public
AMBRALS C mirror and per-file SHA-256 without claiming an authenticated NASA
operational version. Original float functions remain unchanged. Independently
evaluate published JRC/MODIS equations at 80 digits and apply separate C/f64
tolerances. Rust implements the equations independently and uses a stable
equivalent distance near hotspots; original C is not linked into products.
Its redistribution license is unclear, so original source stays disposable.
The original project license was pending at this stage, later resolved for
original code by the Apache decision.

Six inputs are three same-band weights and sun/view geometry. No wavelength
axis is invented. Outputs include scalar BRF/BRDF/kernels and 171 principal-plane /
648 hemisphere samples with correct dimensions. Negative BRF is retained/counted;
negative kernels may be valid. Arbitrary nonnegative weights do not guarantee
physical reflectance across the hemisphere.

Worker transport changes from spectral-array-only to number/Float64Array, retaining
scalars. Renderers follow output types; Rust supplies science and UI supplies
plots, selection, and export. Map azimuth is clockwise from solar azimuth 0°;
negative principal-plane angles use forward φ180°, positive angles backward φ0°.
All default weights are illustrative.
