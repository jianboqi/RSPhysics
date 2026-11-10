# SORT-L numerical reference

Frozen complete 400–2400 nm spectra from the supplied Python model.
Inputs, expected values, source fingerprints, and the resource SHA-256 are kept together.

The directional suite covers no roughness and Hapke single/multifacet cases, geometry changes, dry limits, and thin/thick films. Numerical agreement is distinct from validation against measurements.

Run `cargo test -p rsphysics-sort` and `python -m pytest bindings/python/tests/test_sort.py`.
