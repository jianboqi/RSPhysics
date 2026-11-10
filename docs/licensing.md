# Licensing

RSPhysics uses Apache-2.0 for project-original infrastructure, numerical tools,
bindings, CLI, and independently original Ross–Li, tau–omega, and WCM models.
Included adaptations and resources retain their own notices and conditions.

| Component | Current terms or status |
|---|---|
| Project-original code | [Apache-2.0](../LICENSE) |
| Pinned PROSPECT-D / PROSAIL-D adaptations | Original MIT notices retained |
| SMAC adaptation | GPLv3 notice from the pinned Sen2Like reference retained; additional Apache permission pending |
| Supplied L-MEB / GORT-GPP adaptations | Publication permission confirmed by the project maintainer; component-specific terms retained |
| Coefficients, spectra, sensor responses | Resource-specific provenance and distribution terms |

## Combined builds

The local Python and website/WASM builds include these components. Python
metadata records `Apache-2.0 AND MIT AND GPL-3.0-only AND
LicenseRef-User-Supplied-Research-Code`. The LicenseRef identifies the supplied-code components, whose publication the
project maintainer authorized on 2026-11-10. Original source notices are retained.
The combined build is not an Apache-only distribution; applicable third-party
conditions and resource-specific terms continue to apply.

The root LICENSE does not relicense third-party source, adaptations, or data.
See the packaged [third-party notices](../bindings/python/python/rsphysics/THIRD_PARTY_NOTICES.md),
[supplied-model source record](../bindings/python/python/rsphysics/SUPPLIED_MODELS_ORIGIN.md),
and [rights-contact statement](../RIGHTS-STATEMENT.md).

## SMAC provenance

No explicit LICENSE was found in the pinned author repository. The particular
Sen2Like copy used for reference contains an explicit GPLv3 notice in its SMAC
subdirectory, independently of Sen2Like's top-level license. That notice is
retained without extending it to every SMAC mathematical method or implementation.

The pinned author and Sen2Like functions also have a documented
[technical source comparison](licensing/smac-source-comparison.json). Similarity
of computational statements does not establish ownership or additional permission.
An [authorization inquiry template](smac-authorization-request.md) is available;
no inquiry has been sent by this project tooling.

## Permission records

When permission is obtained, retain the written grant, covered source versions,
rights-holder authority, code/data scope, and downstream distribution terms.
Update component and package metadata while preserving required attribution.
The library and website have independent engineering/release responsibilities;
that boundary does not remove obligations attached to their included components.
