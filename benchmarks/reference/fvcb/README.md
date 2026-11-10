# FvCB reference equations

`generate.py` implements the published equations independently in Python, using
SciPy's Brent root solver. It does not import RSPhysics or another model's
source. `reference.json` contains 92 leaf/canopy cases and complete diagnostic
curves for both default models. Runtime and offline checks need only the
committed fixture, not SciPy.

The reference fixes the conventions documented in the
[leaf](../../../docs/models/fvcb.md) and
[single-big-leaf](../../../docs/models/fvcb-big-leaf.md) guides:

- Ci-based Bernacchi kinetics with a stated standard-atmosphere pressure basis.
- Limitation of carboxylation before photorespiratory conversion, including the
  sign change below Γ* and the analytic zero-Ci limit.
- Fixed-Ci, Ball–Berry, Leuning (both compensation conventions), and corrected
  Medlyn water-vapour conductance formulations.
- Arrhenius / peaked Arrhenius, optional simple TPU, residual-conductance and
  temperature/pressure/oxygen limits.
- Ground-area capacity integration with zero/near-zero extinction and LAI,
  and uniform-canopy/leaf scaling consistency.

The Rust implementation uses a rationalized electron-transport quadratic and
bracketed bisection. Python reference calculations use the ordinary quadratic
and Brent roots. Invariants separately test gas-flux closure, the 1.6
conductance conversion, dark respiration and homogeneous area scaling.

```sh
python benchmarks/reference/fvcb/generate.py
python benchmarks/reference/fvcb/verify.py
```

Updating reference equations requires reviewing their scientific conventions
and refreshing the hashes in `source-manifest.json`.
