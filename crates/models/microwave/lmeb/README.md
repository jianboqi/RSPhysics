# L-MEB

Adapted supplied tauw1i soil forward branch, fixed 1.41 GHz. Includes Dobson soil permittivity, frozen-soil/dry-sand branches, effective soil temperature, roughness/polarization mixing, directional vegetation opacity and prescribed L-band sky model. No water/ice-surface branch, higher-frequency Wang branch or inversion. The sky air temperature is supplied separately exactly as para(3) in the reference. No external model coupling. Publication of the supplied source and adaptations is authorized by the project maintainer; see LICENSE-ORIGIN.md for component terms.

The scientific solver is implemented in Rust. Numerical references are generated
by MATLAB R2026a from the supplied, unchanged scientific functions. See
`benchmarks/reference/lmeb` for pinned inputs, outputs, and source checksums.
See [source permissions](LICENSE-ORIGIN.md) before redistribution.
