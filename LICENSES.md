# Component licenses

Project-original RSPhysics code is licensed under [Apache-2.0](LICENSE).
Included model implementations and scientific resources retain their own terms.

| Component | Declaration |
|---|---|
| Original numerical tools, interfaces, and infrastructure | Apache-2.0 |
| PROSPECT-D, PROSAIL-D and shared optical 4SAIL transport | MIT notices retained |
| 4SAIL-TIR thermal extension | Apache-2.0; shared MIT optical transport attribution retained |
| Ross–Li, tau–omega, and water-cloud implementations | Apache-2.0 |
| FvCB leaf and single-big-leaf implementations | Apache-2.0 |
| Independent RPV and SOILSPECT implementations | Apache-2.0 |
| GSV numerical implementation / supplied LESS spectral vectors | Apache-2.0 / GPL-3.0 terms; see component attribution |
| BSM implementation and coefficient table | GPL-3.0-only; Wout Verhoef / Peiqi Yang attribution retained |
| SMAC component | GPL-3.0-only |
| Preserved 6SV2.1 Fortran distribution | Original authorship retained; retrieved archive has no standalone license file; see [component record](components/6sv2.1/README.md) |
| Supplied L-MEB and GORT-GPP components | LicenseRef-User-Supplied-Research-Code; publication authorized by the project maintainer |
| Coefficients, spectra, and sensor response data | Resource-specific terms and attribution |

The Python distribution records the combined component terms. Full notices and
attribution accompany the [package](bindings/python/python/rsphysics/THIRD_PARTY_NOTICES.md).
The project license does not replace third-party conditions. See
[NOTICE](NOTICE) and the [rights-contact statement](RIGHTS-STATEMENT.md).

SORT and SORT-L retain the author's academic/non-commercial usage terms and the
confirmed public distribution authorization; see [component rights](crates/models/soil/sort/LICENSE-ORIGIN.md) and its [MIT roughness notice](crates/models/soil/sort/THIRD_PARTY_NOTICES.md).


## I²EM rough-surface scattering

Scientific implementation: Adrian Fung. Reference: IEEE-GRSS Remote Sensing
Code Library, DOI 10.21982/M8BC7Z, and Ulaby & Long (2014), section 10-3.9.
The maintainer confirmed authorization for public distribution of the adapted
implementation. This component is identified as
`LicenseRef-I2EM-Authorized-Adaptation`; it is not represented as an Apache-2.0
relicensing of the original MATLAB program. See
`crates/models/microwave/i2em/LICENSE.component`. Original MATLAB source is
not bundled. RSPhysics-authored interfaces and packaging are Apache-2.0.


## FLUSPECT-CX

The scientific implementation and optical data retain GPL-3.0 terms and author
attribution. See the component NOTICE.md / packaged FLUSPECT_ORIGIN.md and
FLUSPECT_GPL-3.0.txt. Original project code remains Apache-2.0.

GOMS retains the maintainer-confirmed distribution authorization recorded in
[its component terms](crates/models/vegetation/goms/LICENSE.component), identified
as `LicenseRef-GOMS-Authorized-Adaptation`.
