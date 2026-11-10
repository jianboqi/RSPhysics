# FvCB leaf photosynthesis

The Farquhar–von Caemmerer–Berry model describes steady-state C₃ leaf
photosynthesis. This implementation separates biochemical limitations from
the stomatal formulation and temperature response, so the assumptions used in
each calculation are explicit.

These models require the current [source installation](../build-from-source.md).
They are not included in the published PyPI 0.1.0 wheels.

## Usage

```python
import rsphysics as rsp

leaf = rsp.ecosystem.fvcb(
    absorbed_par_umol_m2_s=1200.,
    leaf_temperature_c=25.,
    co2_surface_umol_mol=400.,
    vapour_pressure_deficit_kpa=1.,
    vcmax25_umol_m2_s=80., jmax25_umol_m2_s=160.,
    stomatal_model='medlyn',
)
result = leaf['result']
assimilation = result['net_assimilation_umol_m2_s']
ci = result['intercellular_co2_umol_mol']
gsw = result['stomatal_conductance_water_mol_m2_s']
assert abs(result['co2_flux_residual_umol_m2_s']) < 1e-8
```

Model ID: `ecosystem.fvcb`. Fluxes are per unit **leaf area**. The separate
[single-big-leaf model](fvcb-big-leaf.md) reports fluxes per ground area.

## Biochemistry

At prescribed leaf temperature, let `G` denote Γ*, `K = Kc(1 + O/Ko)`, and
`J` the electron transport rate. The two potential net-assimilation branches are

`Ac = Vcmax (Ci − G)/(Ci + K) − Rd`,

`Aj = (J/4) (Ci − G)/(Ci + 2G) − Rd`.

FvCB limits **carboxylation** before applying the photorespiratory factor
`1 − G/Ci`. Above Γ*, this is the minimum of the two net branches. Below Γ*,
the sign reverses and the larger net branch corresponds to the smaller
carboxylation rate. The analytic `Ci=0` limit is supported. Zero light or zero
capacity therefore does not create a photorespiratory flux without its required
electron transport or enzyme capacity. Negative net assimilation is retained.

Electron transport uses the non-rectangular hyperbola
`theta J² − (alpha Iabs + Jmax)J + alpha Iabs Jmax = 0`. The smaller root is
evaluated in a stable form, including `theta=0`, `theta=1` and zero light/capacity.
`alpha` is electrons per **absorbed** PAR photon, not CO₂ quantum yield.

With `tpu_limitation='simple'`, the additional upper bound is `3 Tp − Rd`.
`tpu_umol_m2_s` is supplied at the current temperature, rather than at 25 °C.
This simple TPU option does not model photorespiratory carbon export.

## Fixed Ci and stomatal solutions

| Setting | Meaning |
|---|---|
| `gas_exchange_mode='prescribed_ci'` | Evaluate biochemistry at the supplied `intercellular_co2_umol_mol` |
| `gas_exchange_mode='stomatal'` | Solve `An = (gsw/1.6)(Cs − Ci)` together with the chosen stomatal equation |
| `stomatal_model='ball_berry'` | `gsw = g0w + mBB An hs/Cs` |
| `stomatal_model='leuning'` | `gsw = g0w + 1.6 a1 An/[(Cs − Gamma)(1 + Ds/D0)]` |
| `stomatal_model='medlyn'` | `gsw = g0w + 1.6(1 + g1/sqrt(Ds_eff)) An/Cs`, following the 2012 correction |

The photosynthetic enhancement of conductance is applied only for positive net
assimilation. For nonpositive assimilation, conductance equals `g0w`. A zero
intercept is supported when an open-stomata, positive-assimilation solution
exists; otherwise the solver reports the absent/nonunique gas-exchange solution.

The three slope parameters have distinct definitions. Ball–Berry uses
`ball_berry_slope` on a **water-vapour** basis, Leuning uses
`leuning_slope_co2` on a **CO₂** basis, and Medlyn uses
`medlyn_slope_kpa_sqrt` in kPa^0.5. They are not interchangeable `g1` values.

For Leuning, `leuning_compensation='net_compensation'` uses the current
biochemical zero-net-assimilation point, including day respiration:

`Gamma = max[(Vcmax G + Rd K)/(Vcmax − Rd), G(J + 8Rd)/(J − 4Rd)]`.

If no positive-assimilation branch exists, its stomatal enhancement is inactive.
`leuning_compensation='gamma_star'` instead uses Γ* without day respiration.
The selected convention is recorded with the other parameters.

`Ds` is the **leaf-interior to leaf-surface** vapour-pressure deficit, in kPa.
The imposed leaf-surface humidity is `hs = 1 − Ds/es(Tleaf)` and must be
nonnegative. The FAO-56 saturation-vapour-pressure expression is used. Medlyn's
square root uses `Ds_eff = max(Ds, medlyn_vpd_floor_kpa)`; transpiration uses the
actual deficit. The output water-vapour flux is `gsw Ds/P` in molar units,
reported as mmol H₂O m^-2 s^-1. No atmospheric/aerodynamic or leaf boundary-layer
resistance is added.

```python
fixed_ci = rsp.ecosystem.fvcb(
    gas_exchange_mode='prescribed_ci', intercellular_co2_umol_mol=280.,
)
assert fixed_ci['result']['intercellular_co2_umol_mol'] == 280.
assert fixed_ci['result']['stomatal_conductance_water_mol_m2_s'] is None

for scheme in ['ball_berry', 'leuning', 'medlyn']:
    simulation = rsp.ecosystem.fvcb(stomatal_model=scheme)
```

## Temperature and pressure conventions

`Vcmax25`, `Jmax25` and `respiration25` are independent capacities at 25 °C.
`temperature_response` selects Arrhenius or peaked Arrhenius for Vcmax/Jmax.
Day respiration always uses Arrhenius. Activation energies, deactivation
energies and entropy factors can be supplied explicitly; deactivation must
exceed activation in the peaked formulation. Both formulations are normalized
at 298.15 K. Defaults demonstrate the interface and are not a universal species
calibration.

The Ci-based kinetic preset follows the Bernacchi temperature functions,
using the standard-atmosphere molar convention: at 25 °C and 101325 Pa,
`Kc=404.9 µmol/mol`, `Ko=278.4 mmol/mol`, `Gamma*=42.75 µmol/mol` at
`O2=210 mmol/mol`. Kc and Ko are converted through their reference partial
pressures when total pressure changes. Γ* scales with oxygen mole fraction;
it is not the net CO₂ compensation point. This pressure convention is explicit
because published implementations use different reference-pressure conventions.
The supported temperature range is 10–40 °C.

## Parameters and outputs

Use `rsp.describe('ecosystem.fvcb')['input_schema']` for all defaults, units
and supported ranges. Parameters are organized into imposed environment,
biochemical capacities/light response, stomatal coefficients and temperature
response. The [complete workflow](../../bindings/python/examples/fvcb_workflow.py)
compares the three stomatal formulations and exports an A–Ci curve.

| Output | Meaning |
|---|---|
| `net_assimilation_umol_m2_s` | Net assimilation including day respiration; not GPP or NEE |
| `assimilation_before_day_respiration_umol_m2_s` | Photorespiratory net carbon assimilation before mitochondrial respiration; not gross carboxylation |
| `rubisco_limited_assimilation_umol_m2_s`, `electron_limited_assimilation_umol_m2_s` | Potential **net** branches, each already includes subtraction of Rd |
| `tpu_limited_assimilation_umol_m2_s` | Potential `3Tp−Rd`, active only when `tpu_limitation_enabled` is true |
| `intercellular_co2_umol_mol` | Prescribed or solved Ci |
| `stomatal_conductance_water_mol_m2_s`, `stomatal_conductance_co2_mol_m2_s` | Water and CO₂ conductances, with ratio 1.6 |
| `transpiration_mmol_m2_s` | Molar water-vapour flux to the imposed leaf surface |
| `vcmax_umol_m2_s`, `jmax_umol_m2_s`, `electron_transport_umol_m2_s`, `day_respiration_umol_m2_s` | Current-temperature biochemical capacities and rates |
| `co2_flux_residual_umol_m2_s` | `An−gsc(Cs−Ci)`; null in prescribed-Ci mode |
| `limitation_code` | 0 Rubisco, 1 electron transport, 2 TPU, 3 co-limitation, 4 no canopy |
| `ci_*` | 61-point biochemical A–Ci experiment at fixed absorbed PAR |
| `light_*` | 51-point light experiment holding the selected Ci fixed; no stomatal re-solution |

## NumPy batches

```python
table = rsp.parameter_grid('ecosystem.fvcb', {
    'intercellular_co2_umol_mol': [200., 300., 400.],
})
batch = rsp.ecosystem.fvcb_batch(
    table, gas_exchange_mode='prescribed_ci', shared_axes=True,
)
assert batch['result']['net_assimilation_umol_m2_s'].shape == (3,)
assert batch['result']['ci_net_assimilation'].shape == (3, 61)
assert batch['coordinates']['ci_axis_umol_mol'].shape == (61,)
```

There are 26 numeric columns in `FVCB_PARAMETERS` order. Formulation selectors
are fixed per batch and work with the common `run_array` / `iter_array` API.
Scalar calls use `None` for uncomputed conductance. Native NumPy batches use
NaN with `gas_exchange_computed` and `intercellular_co2_defined` flags; missing
values are not zero physical fluxes.

## Scope

Steady-state C₃ biochemistry with prescribed leaf temperature, absorbed PAR and
leaf-surface conditions. Infinite mesophyll and boundary-layer conductance are
assumed. It does not solve leaf energy balance, drought hydraulics, C₄ pathways
or inverse parameter estimation. A finite-mesophyll-conductance extension needs
compatible chloroplast-based kinetics rather than an unchecked substitution of
the apparent Ci-based parameters.

## Authors and references

- Farquhar, G. D., von Caemmerer, S. & Berry, J. A. (1980). [A biochemical model of photosynthetic CO₂ assimilation in leaves of C₃ species.](https://doi.org/10.1007/BF00386231)
- von Caemmerer, S., Berry, J. A. & Farquhar, G. D. (2025). [A perspective on the original C₃ biochemical model.](https://doi.org/10.1007/s00425-025-04834-7)
- Bernacchi, C. J. et al. (2001). [Improved temperature response functions for models of Rubisco-limited photosynthesis.](https://doi.org/10.1111/j.1365-3040.2001.00668.x)
- Medlyn, B. E. et al. (2002). [Temperature response of parameters of a biochemically based model of photosynthesis. II.](https://doi.org/10.1046/j.1365-3040.2002.00891.x)
- Ball, J. T., Woodrow, I. E. & Berry, J. A. (1987). [A model predicting stomatal conductance.](https://doi.org/10.1007/978-94-017-0519-6_48)
- Leuning, R. (1995). [A critical appraisal of a combined stomatal–photosynthesis model for C₃ plants.](https://doi.org/10.1111/j.1365-3040.1995.tb00370.x)
- Medlyn, B. E. et al. (2011). [Reconciling optimal and empirical approaches to stomatal conductance.](https://doi.org/10.1111/j.1365-2486.2010.02375.x) See the [2012 correction](https://doi.org/10.1111/j.1365-2486.2012.02790.x).
- Bonan, G. B. et al. (2011). [Improving canopy processes in CLM4.](https://doi.org/10.1029/2010JG001593) Standard-atmosphere kinetic convention.
- Allen, R. G. et al. (1998). [FAO-56 meteorological expressions.](https://www.fao.org/4/x0490e/x0490e07.htm)
