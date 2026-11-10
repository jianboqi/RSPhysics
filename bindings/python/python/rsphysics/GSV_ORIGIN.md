# GSV component attribution

Scientific model: Chongya Jiang and Hongliang Fang (2019),
*GSV: a general model for hyperspectral soil reflectance simulation*,
International Journal of Applied Earth Observation and Geoinformation 83, 101932.
https://doi.org/10.1016/j.jag.2019.101932

The numerical implementation is newly written RSPhysics code (Apache-2.0).
The two unmodified basis tables, DryVec.txt and SMVec.txt, are supplied with
the LESS pyLessSDK GSVSoil module. The source distribution declares GPL v3
at its repository root; those resources retain these component terms.
No separate permission or public-domain status is inferred for the tables.
The full GPL v3 text accompanies this component as LICENSE.resources.

Changes from the supplied Python interface: finite/domain checks, explicit
GSV1/GSV3 selection, empty wavelength request means the original 211-sample
axis, out-of-range requests are errors, and algebraic values outside [0,1]
are reported without clipping. Both original interpolation formulas are
retained numerically, including the 2500 nm endpoint. No model coupling.
