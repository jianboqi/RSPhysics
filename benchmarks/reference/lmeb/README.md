# L-MEB supplied-MATLAB references

32 deterministic input cases are stored in `inputs.json`. `expected.json` was
generated with MATLAB R2026a 26.1.0.3203278 before the Rust solver. It includes
both polarization angle curves at all 81 samples.
Original scientific functions are unchanged. The runner selects
the fixed 1.41 GHz soil forward branch of tauw1i.

## Restore and regenerate

Import the same local source files using the recorded per-file size/SHA-256:

```sh
python tooling/reference-runner/import_supplied.py lmeb --source "PATH/LMEB.rar"
matlab -batch "run('benchmarks/reference/lmeb/generate.m')"
```

All original sources live in `.reference-cache/lmeb`. The importer rejects
different bytes or redirected source/cache paths. No station spreadsheets are
needed or included. Publication permission was confirmed by the project maintainer on 2026-11-10;
see `upstream-provenance.json` and the model's `LICENSE-ORIGIN.md`.

## Numerical checks

Native Rust, Python, and product WASM compare to these original outputs with
`abs_error <= 2e-9 + 2e-9 * abs(reference)`. This predetermined implementation
tolerance does not establish accuracy against observations. Python and browser
reports record actual errors. Runtime algorithms do not call MATLAB.

```sh
cargo test -p rsphysics-lmeb --locked
python benchmarks/reference/report_supplied.py
python tooling/reference-runner/clean.py --model lmeb
python tooling/reference-runner/clean.py --model lmeb --remove
```

Cleanup preserves reference fixtures, provenance, Rust source, and Python runtime.
Daily tests do not require the original sources or MATLAB.


## Complete forward reference

40 cases cover Dobson/Wang frequencies, fresh/saline water and pure ice. Only the undefined water emission temperature is corrected.

`full-provenance.json` records the changes and fixture hashes. Recompute using
`python tooling/reference-runner/verify_extended_matlab.py --model lmeb --source PATH`.
