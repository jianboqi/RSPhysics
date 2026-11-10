# BSM attribution and changes

Scientific model: Wout Verhoef (soilwat v1.0, September2012), Peiqi Yang
(BSM v2.0, January2020). Reference: Yang, van der Tol, Yin & Verhoef(2020),
The SPART model, Remote Sensing of Environment247,111870.
https://doi.org/10.1016/j.rse.2020.111870

Author repository: https://github.com/peiqiyang/BSM
Pinned snapshot: ff6d8b5fb4ae0093ffdf25f91353176a11dc394a.
BSM_coef.dat is bundled unchanged. This component and its resources retain
the author's GPL-3.0-only declaration; see LICENSE. Original MATLAB is not
bundled or required at runtime. Rust adaptation: RSPhysics contributors.

All six numeric inputs are exposed. The default complete Poisson sum follows
Eq15 of the paper to omitted probability<1e-14; an explicit legacy six-film
mode reproduces the author's finite sum. At mu=2, the latter discards about
0.45% of probability and is intentionally not renormalized. An independent
flux quadrature evaluates dielectric cone transmission in the shared Rust
numerical library. Domain, finite and resource SHA checks are added.
