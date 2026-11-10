# SMAC original coefficients

`fetch.py` retrieves fixed original materials into `.reference-cache/smac/upstream`.
`prepare.py` losslessly packs them into one zlib resource in the public model crate.
`verify.py` audits that resource, all original-file hashes, 49-value layouts and
the original-reference fixture offline. Python standard library only.

331 source files / 367 coefficient sets. Multi-band Sentinel-2 columns share
one original text. No coefficient rounding or sensor substitution. Runtime
decompresses once in Rust with a size limit and validates the archive and every
original text checksum. `atmosphere.smac_coefficient_source(id)` exports the
exact original text plus source URL, SHA and zero-based band column when relevant.

Sources are pinned by `source-manifest.json`. Preserve CNES acknowledgements,
SMAC's GPLv3 notice and the original paper citation. The old author repository
does not contain a separate coefficient-data license; do not mark it public domain.
See [scope and license notes](../../docs/smac.md).

Cleanup: `python tooling/reference-runner/clean.py --model smac --remove`.
The runtime resource and offline fixtures are retained. No data download occurs
when importing or running the installed library or browser model.
