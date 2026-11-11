# 6SV2.1

Original Second Simulation of a Satellite Signal in the Solar Spectrum,
vector version 2.1, by E. Vermote and collaborators.

`upstream/` preserves every regular file from the original distribution,
including its two main programs, coefficient tables, and example input.
`source-manifest.json` records the archive and individual file SHA-256 hashes.
The scientific source is not translated to Rust.

Source: <https://salsa.umd.edu/files/6S/6sV2.1.tar>.
Archive SHA-256:
`42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1`.

The retrieved archive contains no standalone license file. Its original
authorship and terms are not replaced by the license of the RSPhysics adapters.

Both `sixsV2.1` and `sixsV2.1-lutaero` are retained. Original input cards and
auxiliary files provide access to the original program options. A successful
build alone does not establish numerical agreement across compilers.

`build_tools/` contains the source-package build implementation. The native
and WASM products expose a direct C entry point around the retained Fortran
kernel. See [compatibility corrections](COMPATIBILITY.md),
[build and validation commands](../../../../../tooling/sixs/README.md), and the
[Python model interface](../../../../../docs/models/sixsv21.md).
