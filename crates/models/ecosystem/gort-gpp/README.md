# GORT-GPP

Pointwise GORT radiation and sunlit/shaded Farquhar-Ball-Berry photosynthesis path from supplied Main4PSN. Explicit incident direct/diffuse PAR and solar zenith replace site/time/Excel preprocessing. Fixed MAXCROWNS=10, MAXORDER=5, original PI=3.1415926 and original angular lookup conventions retained. Output is net assimilation, not gross primary productivity or NEE. No annual station driver, ET, seasonal Vcmax tables, Beer/CI alternative modes or generic model coupling. Publication of the supplied source and adaptations is authorized by the project maintainer; see LICENSE-ORIGIN.md for component terms.

The scientific solver is implemented in Rust. Numerical references are generated
by MATLAB R2026a from the supplied, unchanged scientific functions. See
`benchmarks/reference/gort-gpp` for pinned inputs, outputs, and source checksums.
See [source permissions](LICENSE-ORIGIN.md) before redistribution.
