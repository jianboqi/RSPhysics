# Third-party scientific source and data

PROSPECT-D equations and optical coefficients reference the authors' prospect
R package v1.7.0, commit 4649d5006ddf6bce609951765a88ba2498fcd8ab:
https://github.com/jbferet/prospect/tree/4649d5006ddf6bce609951765a88ba2498fcd8ab .
The pinned release declares MIT; newer releases may have different terms.
Copyright 2020/04 Jean-Baptiste FERET. Programmers Jean-Baptiste FERET and
Florian de Boissieu. Original source also credits FLUSPECT optimizations by
Wout Verhoef, Christiaan van der Tol and Joris Timmermans (2007 / January 2011).
Rust adaptation: RSPhysics contributors, 2026-11-09.

Scientific references: https://doi.org/10.1016/j.rse.2017.03.004 and
https://doi.org/10.21105/joss.06027 . Coefficients SHA-256:
cb2da8cef652ef90e666aef302689936d4ddf817c7908484a3fd06e1467fd3a5 .

PROSAIL-D canopy equations and soil data reference the author's `prosail` v3.1.1,
commit 46549958295b605ac1e211a693d4b2287c3a5a13:
https://github.com/jbferet/prosail/tree/46549958295b605ac1e211a693d4b2287c3a5a13 .
That fixed DESCRIPTION declares MIT + file LICENSE; Copyright 2026
Jean-Baptiste Féret. volscatt credits Wout Verhoef, April 2001, CROMA.
Only forward fourSAIL/Campbell/volscatt/jfunc equations are adapted; newer
GPL releases and package/inversion/GIS dependencies were not used.
Original soil table SHA-256:
7754d3ddb5d3b1f6c3c23cfd0a7c09d7a75be83a1da75c10ba04e71461c30b74 .
Leaf optics reuse the separately attributed PROSPECT-D solver above.
No separate data license was found; redistribution review is still pending.
Rust adaptation: RSPhysics contributors, 2026-11-09. Changes include typed
contracts, SHA checks, analytical LAI=0 bare-soil limit, explicit BRF semantics,
roundoff guards and continuous hotspot integration endpoint limits.

## MIT permission terms for the adapted forward models

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

This notice preserves upstream attribution; it does not assign a license to
other original RSPhysics infrastructure.

## Ross–Li

The surface.ross-li solver independently implements the published
RossThick–LiSparseReciprocal equations, h/b=2, b/r=1:
https://rami-benchmark.jrc.ec.europa.eu/_www/definition.php and
https://doi.org/10.1109/36.841980 . Copyright 2026 RSPhysics contributors;
original implementation is licensed Apache-2.0. The MIT notices above do not apply
to this newly written model or other original infrastructure.
Original AMBRALS C functions from public mirror arthurelmes/ambrals_forward,
commit b43d9171ea9f6dc76854cac34037e2af7903d02e, were used as a test-only
reference. No explicit redistribution license was found for those files;
they are not included in the package or linked into its scientific solver.
No external runtime data is needed by this model.

## Microwave τ–ω and WCM
Independent implementations of published equations. The original implementations are licensed Apache-2.0. NASA ATBD and GPL-declared SenSE source are test-only references in the disposable cache, never linked or distributed. No external runtime data or pretrained calibration is included.

## Official sensor response data
USGS Landsat and NASA OBPG MODIS tabulations are separately attributed scientific data. Each bundled profile carries original URL/SHA-256, version/snapshot, band scope and explicit preparation notes. No ESA SentiWiki workbook or response arrays are redistributed; its Sentinel-2 v5 registry entries contain identification and checksums only, and require local acquisition/import under provider terms. These data acknowledgements do not grant a license to third-party material or change the project's original-code Apache-2.0 license.

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
The component source and full license notices accompany the distribution.

## Project-original code

Project-original numerical tools, independent Ross-Li/tau-omega/water-cloud
implementations, interfaces, and tooling are Apache-2.0. The MIT and GPLv3 terms
above retain their component-specific scope. The root project license does not
relicense third-party code or data. See NOTICE and LICENSES.md.

## Supplied L-MEB and GORT-GPP adaptations

These two local model adaptations use user-supplied MATLAB research code without
an explicit redistribution license in the supplied source sets. They are recorded
as LicenseRef-User-Supplied-Research-Code, not relabeled Apache-2.0. See
SUPPLIED_MODELS_ORIGIN.md for attribution, scientific references, and terms.
The project maintainer confirmed publication permission on 2026-11-10. Original
MATLAB source, papers, and station spreadsheets are not bundled.

I2EM: Adrian Fung, IEEE-GRSS RSCL DOI 10.21982/M8BC7Z; Ulaby & Long (2014).
Adapted implementation distributed with maintainer-confirmed authorization.
Original MATLAB source is not included. See I2EM_COMPONENT_LICENSE.txt.


## FLUSPECT-CX

The scientific implementation and optical data retain GPL-3.0 terms and author
attribution. See the component NOTICE.md / packaged FLUSPECT_ORIGIN.md and
FLUSPECT_GPL-3.0.txt. Original project code remains Apache-2.0.

GOMS: Li and Strahler (1992), DOI 10.1109/36.134078.
Component distribution authorization is recorded in GOMS_COMPONENT_LICENSE.txt.

## Shared 4SAIL transport and 4SAIL-TIR

The shared optical transport in rsphysics-numerics/src/sail.rs retains the
MIT attribution to Jean-Baptiste Féret (2026), pinned prosail commit
46549958295b605ac1e211a693d4b2287c3a5a13, and the MIT terms above.
The new thermal source implementation follows Verhoef, Jia, Xiao and Su (2007)
and is RSPhysics-original Apache-2.0 code. No external optical coefficient
table or third-party thermal source implementation is bundled for this model.


## GSV and BSM soil spectra

GSV: Jiang & Fang(2019), DOI10.1016/j.jag.2019.101932. New numerical code is
Apache-2.0; supplied LESS basis resources retain GPL v3. See GSV_ORIGIN.md.
BSM: Wout Verhoef / Peiqi Yang, DOI10.1016/j.rse.2020.111870. Adapted solver
and coefficients retain GPL-3.0-only; see BSM_ORIGIN.md. Full GPL v3 terms
are included in SMAC_GPL-3.0.txt. RPV and SOILSPECT are newly written
implementations of published equations, licensed Apache-2.0.

## 6SV2.1

The original Fortran kernel and its runtime notices are recorded in [SIXSV21_ORIGIN.md](SIXSV21_ORIGIN.md). Source builds retain the complete original distribution and document build-copy corrections separately.
