"""Fresnel tools for one smooth planar isotropic nonmagnetic interface.

Incident medium must be real, positive and lossless. Transmitted n+i*k is
passive (n,k>=0, not both zero), using exp(-i*omega*t). The angle is in degrees
from the surface normal, [0,90). All inputs broadcast using NumPy rules; Rust
computes coefficients while releasing the GIL. Invalid computed elements abort
the call with a flat element index. Scalars produce zero-dimensional arrays.
"""
from . import _native
from ._api import _call
from ._numeric import real_array, complex_array, broadcast

def fresnel(incidence_angle_deg,n_transmitted,*,n_incident=1.):
    """Complex amplitudes and dimensionless s/p/unpolarized power R and interface T.

    n_transmitted may be complex. s is perpendicular to the incidence plane
    (H for a horizontal surface); p lies in the plane (V). Amplitude r_p follows
    the Byrnes electric-field convention: at normal incidence r_p=-r_s, while
    their power reflectances are equal. Total internal reflection retains phase.

    interface_transmittance is Poynting flux entering medium 2 at its boundary,
    NOT throughput of a finite absorbing layer. No roughness, multilayer,
    scattering, emission or absorptive incident medium is modeled. Unpolarized
    power means an equal incoherent mixture; amplitudes are not averaged.
    """
    theta,ni,nt=broadcast(real_array(incidence_angle_deg),real_array(n_incident),complex_array(n_transmitted))
    return _call(_native.fresnel_array,theta,ni,nt,False)

def fresnel_permittivity(incidence_angle_deg,relative_permittivity,*,permittivity_incident=1.):
    """Fresnel from relative permittivity, with mu_r=1; refractive indices in Rust.

    Incident epsilon is positive real. Transmitted epsilon may have negative
    real part but must be nonzero and Im>=0 for exp(-i*omega*t). Data using the
    opposite time convention must be explicitly conjugated by the caller.
    No dielectric mixing or soil-moisture model is assumed.
    """
    theta,ei,et=broadcast(real_array(incidence_angle_deg),real_array(permittivity_incident),complex_array(relative_permittivity,name='relative_permittivity'))
    return _call(_native.fresnel_array,theta,ei,et,True)
