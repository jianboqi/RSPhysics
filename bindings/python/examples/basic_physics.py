"""Verify independent Fresnel and Planck building blocks after pip installation.

No imagery, sensor coefficients, model coupling or optional plotting libraries.
Run: python bindings/python/examples/basic_physics.py
"""
import json
import numpy as np
import rsphysics as rsp

def main():
    angle=np.array([0.,30.,45.,60.,80.]);glass=rsp.optics.fresnel(angle,1.5)
    assert np.allclose(glass['reflectance_s'][0],.04,atol=1e-14)
    assert np.allclose(glass['reflectance_s']+glass['interface_transmittance_s'],1.,atol=1e-14)
    brewster=rsp.optics.fresnel(np.degrees(np.arctan(1.5)),1.5)
    assert brewster['reflectance_p']<1e-28
    tir=rsp.optics.fresnel(60.,1.,n_incident=1.5)
    assert tir['total_internal_reflection'] and np.isclose(tir['reflectance_unpolarized'],1.)
    absorber=rsp.optics.fresnel_permittivity(40.,6.+.5j)
    wavelength=np.array([8000.,10000.,12000.,14000.]);unit='W m^-2 sr^-1 um^-1'
    radiance=rsp.radiometry.planck_radiance(wavelength,300.,radiance_unit=unit)
    recovered=rsp.radiometry.inverse_planck(wavelength,radiance,radiance_unit=unit)
    assert np.allclose(recovered,300.,rtol=2e-12)
    nu=1.4e9;microwave=float(rsp.radiometry.planck_frequency_radiance(nu,300.))
    assert np.isclose(rsp.radiometry.inverse_planck_frequency(nu,microwave),300.,rtol=2e-12)
    print(json.dumps(dict(
        fresnel=dict(incidence_angle_deg=angle.tolist(),glass_reflectance_s=glass['reflectance_s'].tolist(),glass_reflectance_p=glass['reflectance_p'].tolist(),brewster_reflectance_p=float(brewster['reflectance_p']),tir_reflectance=float(tir['reflectance_unpolarized']),passive_permittivity_reflectance=float(absorber['reflectance_unpolarized'])),
        planck=dict(wavelength_nm=wavelength.tolist(),temperature_k=300.,radiance_unit=unit,radiance=radiance.tolist(),inverse_temperature_k=recovered.tolist(),microwave_frequency_hz=nu,microwave_radiance_per_hz=microwave),
        scope='Independent basic operators: one smooth interface and vacuum blackbody radiance. No emissivity/atmosphere or model coupling.'
    ),indent=2,allow_nan=False))
if __name__=='__main__':main()
