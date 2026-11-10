# PROSPECT-D source and data provenance

Scientific references: https://doi.org/10.1016/j.rse.2017.03.004 and
https://doi.org/10.21105/joss.06027 . Rust equations are adapted from the authors'
R package `prospect` v1.7.0, commit 4649d5006ddf6bce609951765a88ba2498fcd8ab:
https://github.com/jbferet/prospect/tree/4649d5006ddf6bce609951765a88ba2498fcd8ab .
That release declares MIT in DESCRIPTION; newer releases may have other terms.

Original source: Copyright 2020/04 Jean-Baptiste FERET; programmers Jean-Baptiste
FERET and Florian de Boissieu. The source credits FLUSPECT optimizations by Wout
Verhoef, Christiaan van der Tol and Joris Timmermans (2007 / January 2011).

Changes (2026-11-09): Rust adaptation, typed parameters, explicit input envelope,
finite/energy checks, independent E1 numerical routine and cached interface
calculations. Python does not reimplement equations. The reference harness
bypasses only package-dependent input normalization with a complete D parameter
frame, PROT=CBC=0; scientific PROSPECT/calctav function bodies are unchanged.

`data/coefficients.tsv` is byte-for-byte upstream `data-raw/dataSpec_PRO.txt`:
SHA-256 cb2da8cef652ef90e666aef302689936d4ddf817c7908484a3fd06e1467fd3a5.
This is required runtime data, not disposable reference installation data.
The full ten-column file is preserved; D uses the six SAC_CHL through SAC_LMA
columns, not the protein/CBC columns. No separate data license was found in
the inspected files. This provenance record does not claim project-level
redistribution review is complete. The upstream MIT permission terms for the
adapted model are reproduced in LICENSE. Other project-original infrastructure is licensed Apache-2.0.
