# FvCB single-big-leaf canopy

This model applies [FvCB leaf biochemistry and the selected stomatal equation](fvcb.md)
to a single homogeneous canopy leaf. It uses prescribed absorbed canopy PAR and
an exponential biochemical-capacity profile. Temperature and leaf-surface
conditions are uniform. Fluxes are reported per unit **horizontal ground area**.

It requires the current [source installation](../build-from-source.md), and is
not included in the published PyPI 0.1.0 wheels.

## Usage

```python
import rsphysics as rsp

canopy = rsp.ecosystem.fvcb_big_leaf(
    leaf_area_index=3.,
    absorbed_par_ground_umol_m2_s=1200.,
    nitrogen_extinction_coefficient=0.3,
    vcmax25_umol_m2_s=80., jmax25_umol_m2_s=160.,
    stomatal_model='medlyn',
)
result = canopy['result']
net_ground_assimilation = result['net_assimilation_umol_m2_s']
```

Model ID: `ecosystem.fvcb-big-leaf`.

## Scaling and parameters

| Parameter | Unit | Meaning |
|---|---|---|
| `leaf_area_index` | m² leaf/m² ground | One-sided LAI, 0–12 |
| `absorbed_par_ground_umol_m2_s` | µmol photons/m² ground/s | Canopy absorbed PAR, supplied directly |
| `nitrogen_extinction_coefficient` | 1 | Exponential capacity decline with cumulative LAI; 0–3 |
| `vcmax25_umol_m2_s`, `jmax25_umol_m2_s`, `respiration25_umol_m2_s`, `tpu_umol_m2_s` | µmol/m² leaf/s | Capacity of top-canopy leaves before integration |
| Other parameters | See leaf guide | Shared kinetic, stomatal and temperature conventions |

For cumulative leaf area `l`, each biochemical capacity follows
`capacity(l)=capacity_top exp(−kN l)`. Its integral is

`F = [1−exp(−kN LAI)]/kN`, with the exact limit `F=LAI` when `kN=0`.

Vcmax, Jmax, day respiration and optional TPU are multiplied by `F`. Residual
water conductance is multiplied by total LAI. The integrated capacities and
**absorbed ground-area PAR** are then used in the same biochemical and
stomatal equations. CO₂ concentrations and stomatal slopes are not multiplied
by LAI. This also defines a useful consistency limit: uniform capacity and
`APARground = APARleaf*LAI` give the leaf flux multiplied by LAI.

LAI zero requires zero absorbed canopy PAR. Carbon/water fluxes are zero and
Ci is undefined, represented as `None` in scalar calls and NaN with
`intercellular_co2_defined=0` in NumPy batches.

## Outputs and NumPy batches

The [leaf output table](fvcb.md#parameters-and-outputs) applies, with all rates
and conductances on a ground-area basis. `capacity_area_factor` reports `F`;
`leaf_area_factor` reports LAI. The diagnostic light coordinate is also per
ground area and holds the selected Ci fixed.

```python
table = rsp.parameter_grid('ecosystem.fvcb-big-leaf', {
    'leaf_area_index': [1., 3., 5.],
})
batch = rsp.ecosystem.fvcb_big_leaf_batch(table, shared_axes=True)
assert batch['result']['net_assimilation_umol_m2_s'].shape == (3,)
```

There are 28 numeric columns in `FVCB_BIG_LEAF_PARAMETERS` order. All
formulation selectors are fixed per batch, including the gas-exchange,
stomatal, temperature, Leuning-compensation and TPU choices.

## Scope

This is a single-big-leaf approximation. It does not resolve sunlit/shaded
populations, layered light environments or temperature gradients, and is **not**
the de Pury–Farquhar 1997 sun/shade model. Nonlinear photosynthetic light
responses mean averaging a heterogeneous canopy can introduce systematic
errors. The model makes that scaling assumption explicit; it does not treat
the result as a universally accurate canopy GPP estimate.

Absorbed PAR is supplied directly. No atmospheric, optical radiation-transfer,
leaf energy-balance or soil-water model is invoked.

## Authors and references

The biochemical model is due to Graham D. Farquhar, Susanne von Caemmerer and
Joseph A. Berry. Stomatal and kinetic references are listed in the
[leaf guide](fvcb.md#authors-and-references). The single-big-leaf capacity
integration is the explicit scaling used here.

- de Pury, D. G. G. & Farquhar, G. D. (1997). [Simple scaling of photosynthesis from leaves to canopies without the errors of big-leaf models.](https://doi.org/10.1111/j.1365-3040.1997.00094.x) Comparison and limitations of canopy scaling.
