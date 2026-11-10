
## SMAC

Scientific reference: H. Rahman and G. Dedieu (1994), SMAC: a simplified method
for the atmospheric correction of satellite measurements in the solar spectrum,
International Journal of Remote Sensing 15(1), 123-143.
https://doi.org/10.1080/01431169408954055

Reference/adapted implementation: Olivier Hagolle, CNES, from the original SMAC
C routine. The pinned ESA Sen2Like SMAC subdirectory explicitly declares GPL v3
in copyright.txt, independently of Sen2Like's top-level Apache license:
https://github.com/senbox-org/sen2like/tree/6dd3f0da82a7461b03a2914df56d305070e50d6d/sen2like/sen2like/atmcor/smac
Rust adaptation, typed contracts, compressed coefficient registry, provenance
checks and array interface: RSPhysics contributors, 2026-11-10.
The SMAC crate is GPL-3.0-only. Full GPLv3 terms accompany the source in its
LICENSE and the Python distribution in SMAC_GPL-3.0.txt. These GPLv3 terms
are specific to this component; the preceding MIT terms do not apply to SMAC.

Acknowledge CNES for calculating the sensor coefficients. Sentinel-2A/B/C
continental and Landsat-8 OLI coefficients are from that pinned Sen2Like tree.
Legacy coefficients are from Olivier Hagolle's author repository, commit
77bf73dd1f2a2b6a33a97b8997da469210f9ce3e, which has no separate coefficient-data
license file. Original-file text, source URLs, SHA-256 and band column indices
are retained losslessly in one compressed resource. No public-domain status
is inferred, no precision is reduced, and no missing sensor is substituted.
The project remains a local verification build; license compatibility and
corresponding-source arrangements for a combined public/private distribution
must be settled before formal distribution.
