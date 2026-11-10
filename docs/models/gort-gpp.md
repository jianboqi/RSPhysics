# GORT-GPP

GORT-GPP combines canopy geometry/radiation with sunlit and shaded leaf
photosynthesis. The supplied study examines leaf age and canopy structure in
a subtropical evergreen Chinese fir forest.[^wu] This adaptation selects the
provided `Main4PSN` path with **mvp=1, flagSeason=0**, evaluated at one time step.

## Implemented formulation

Ellipsoidal crown geometry and leaf area define between-/within-crown gaps and
openness. The original finite crown-count approximation and five-order
background-scattering treatment partition direct/diffuse PAR. The selected
two-leaf photosynthetic path allocates Vcmax using a canopy-top young-leaf weight,
then solves the provided Farquhar/Ball–Berry quadratics for each leaf class.

```text
An,canopy = An,sunlit × LAIsunlit + An,shaded × LAIshaded
```

**The exposed carbon flux is net assimilation**, including the selected source
path's respiration treatment. It is not directly GPP, GEP, or NEE. Negative
values are retained. Canopy flux uses ground area; individual leaf-class fluxes
use that class's leaf area. PAR uses μmol photons m⁻² s⁻¹, VPD uses Pa, and the
temperature input uses °C.

## Python example

```python
import rsphysics as rsp

result = rsp.ecosystem.gort_gpp(
    leaf_area_index=4.31, solar_zenith_deg=40,
    direct_par_umol_m2_s=1200, diffuse_par_umol_m2_s=200,
)
net_assimilation = result['result']['net_assimilation_umol_m2_s']

table = rsp.parameter_grid('ecosystem.gort-gpp', {'young_leaf_fraction': [0., 0.4, 0.8]})
batch = rsp.run_array('ecosystem.gort-gpp', table, shared_axes=True)
```

Use `describe` for the input contract and `parameters_for` for matrix columns.
Outputs include leaf-class areas/assimilation/PAR, background absorption,
openness, and both gap curves at the original 0–89° sampling.

## Scope and verification

Explicit solar zenith and incident direct/diffuse PAR replace site/calendar/Excel
preprocessing. The library does not require or distribute station observations.
Annual drivers, evapotranspiration, daily aggregation, seasonal Vcmax tables,
Beer/CI alternatives, and generic model coupling are outside this adaptation.

Original `PI=3.1415926`, MAXCROWNS=10, MAXORDER=5, openness quadrature, and
the two distinct angular lookup conventions are retained. The main photosynthetic
path rounds degrees into a one-based array without the radiation routine's +1.
Its failure at index zero is avoided by the supported 1–80° input domain.
Original unused/empty fields are not promoted to scientific outputs.

32 original MATLAB R2026a cases cover ordinary inputs, leaf-age weights,
direct/diffuse-only illumination, and zero PAR. Source functions are unchanged;
the runner extracts the selected pointwise path without Excel/plotting I/O.
See the [reference directory](../../benchmarks/reference/gort-gpp). Matching that
implementation does not constitute independent validation of the model physics.

The project maintainer has confirmed permission to publish the supplied source
and adaptations. See the
[source record](../../crates/models/ecosystem/gort-gpp/LICENSE-ORIGIN.md).

[^wu]: Wu et al. (2021), [Effects of leaf age and canopy structure on gross ecosystem production in a subtropical evergreen Chinese fir forest](https://doi.org/10.1016/j.agrformet.2021.108618), Agricultural and Forest Meteorology 310, 108618. The supplied paper documents the scientific context; reference fixtures pin the supplied code separately.
