import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp
SUITE=json.loads((Path(__file__).resolve().parents[3]/'benchmarks/reference/basic-physics/expected.json').read_text())

@pytest.mark.parametrize('case',SUITE['fresnel'],ids=[c['name'] for c in SUITE['fresnel']])
def test_fresnel_independent_complex_snell_reference(case):
    p=case['parameters'];a=rsp.optics.fresnel(p['angle_deg'],complex(p['n_real'],p['k']),n_incident=p['n_incident'])
    for key,expected in case['expected'].items():
        e=complex(*expected) if key.startswith('amplitude') else expected
        np.testing.assert_allclose(a[key],e,rtol=2e-11,atol=2e-12,err_msg=case['name']+' '+key)
    for pol in ['s','p','unpolarized']:
        np.testing.assert_allclose(a['reflectance_'+pol]+a['interface_transmittance_'+pol],1.,rtol=0,atol=3e-14)

@pytest.mark.parametrize('case',SUITE['planck_wavelength'])
def test_planck_wavelength_direct_80_digit_reference(case):
    c=case;actual=rsp.radiometry.planck_radiance(c['wavelength_nm'],c['temperature_k'])
    np.testing.assert_allclose(actual,c['radiance_per_nm'],rtol=2e-12,atol=0)
    temperature=rsp.radiometry.inverse_planck(c['wavelength_nm'],c['radiance_per_nm'])
    np.testing.assert_allclose(temperature,c['inverse_temperature_k'],rtol=2e-12,atol=0)

@pytest.mark.parametrize('case',SUITE['planck_frequency'])
def test_planck_frequency_direct_80_digit_reference(case):
    c=case;actual=rsp.radiometry.planck_frequency_radiance(c['frequency_hz'],c['temperature_k'])
    np.testing.assert_allclose(actual,c['radiance_per_hz'],rtol=2e-12,atol=0)
    temperature=rsp.radiometry.inverse_planck_frequency(c['frequency_hz'],c['radiance_per_hz'])
    np.testing.assert_allclose(temperature,c['inverse_temperature_k'],rtol=2e-12,atol=0)

def test_optical_limits_sign_polarization_permittivity_and_broadcasting():
    normal=rsp.optics.fresnel(0,1.5)
    np.testing.assert_allclose(normal['reflectance_s'],.04,atol=1e-15)
    np.testing.assert_allclose(normal['amplitude_r_s'],-normal['amplitude_r_p'],atol=1e-15)
    assert rsp.optics.fresnel(np.degrees(np.arctan(1.5)),1.5)['reflectance_p']<1e-29
    angles=np.array([0,30,60,89.9]);n=np.array([1.33,1.5,2+.3j])[:,None]
    a=rsp.optics.fresnel(angles,n);b=rsp.optics.fresnel_permittivity(angles,n*n)
    assert a['reflectance_s'].shape==(3,4);assert a['amplitude_r_s'].dtype==np.complex128
    for key in a:np.testing.assert_allclose(a[key],b[key],atol=1e-14,rtol=1e-13)
    tir=rsp.optics.fresnel([30,60],1.,n_incident=1.5)
    np.testing.assert_array_equal(tir['total_internal_reflection'],[False,True]);assert abs(tir['amplitude_r_s'][1].imag)>0
    np.testing.assert_allclose(tir['interface_transmittance_s'][1],0,atol=1e-15)
    assert rsp.optics.fresnel([],1.5)['reflectance_s'].shape==(0,)
    assert np.asarray(normal['reflectance_s']).shape==()

def test_thermal_units_density_jacobian_broadcasting_and_stefan_boltzmann():
    r=rsp.radiometry;wave=np.array([500,1000,10000,20000.]);t=np.array([250.,300.,1000.])[:,None]
    per_nm=r.planck_radiance(wave,t);per_um=r.planck_radiance(wave,t,radiance_unit='W m^-2 sr^-1 um^-1')
    np.testing.assert_allclose(per_um,per_nm*1000,rtol=3e-14)
    np.testing.assert_allclose(r.inverse_planck(wave,per_um,radiance_unit='W m^-2 sr^-1 um^-1'),np.broadcast_to(t,(3,4)),rtol=3e-14)
    lam=wave*1e-9;nu=r.SPEED_OF_LIGHT/lam
    # Two independent f64 evaluations plus reciprocal-coordinate rounding in a
    # Wien tail; use the same accuracy scale as the high-precision references.
    np.testing.assert_allclose(r.planck_frequency_radiance(nu,t),per_nm*1e9*lam**2/r.SPEED_OF_LIGHT,rtol=2e-12)
    # Numerical integral over log wavelength, independent of production Planck algebra.
    w=np.geomspace(10,1e9,20000);exitance=np.pi*rsp.spectral.integrate(w,r.planck_radiance(w,300.))
    sigma=2*np.pi**5*r.BOLTZMANN_CONSTANT**4/(15*r.SPEED_OF_LIGHT**2*r.PLANCK_CONSTANT**3)
    np.testing.assert_allclose(exitance,sigma*300**4,rtol=2e-7)
    peak=2.897771955185e6/300
    radiances=r.planck_radiance(peak*np.array([.999,1,1.001]),300)
    assert radiances[1]>radiances[0] and radiances[1]>radiances[2]
    assert r.planck_radiance(np.empty((0,1)),[200,300]).shape==(0,2)

@pytest.mark.parametrize('args',[(90,1.5,1),(-1,1.5,1),(40,1.5,0),(40,1.5,-1),(40,1.5-.1j,1),(40,0,1),(40,complex(np.nan,1),1)])
def test_invalid_fresnel_inputs(args):
    with pytest.raises(rsp.ModelError):rsp.optics.fresnel(args[0],args[1],n_incident=args[2])

def test_invalid_inputs_and_scientific_output_overflow():
    for x,t in [(0,300),(-1,300),(1000,-1),(np.nan,300),(1000,np.inf)]:
        with pytest.raises(rsp.ModelError):rsp.radiometry.planck_radiance(x,t)
    with pytest.raises(rsp.ModelError):rsp.radiometry.inverse_planck(10000,-1)
    with pytest.raises(rsp.ModelError):rsp.radiometry.planck_radiance(10000,300,radiance_unit='per guess')
    with pytest.raises(rsp.ModelError):rsp.radiometry.planck_radiance([500,600],[200,300,400])
    with pytest.raises(rsp.ModelError):rsp.optics.fresnel([0,20],['bad','index'])
    with pytest.raises(rsp.ModelError):rsp.optics.fresnel_permittivity(40,3-.1j)
    with pytest.raises(rsp.ModelError):rsp.radiometry.planck_radiance(1e-100,1e200)
    with pytest.raises(rsp.ModelError,match='element 1'):rsp.radiometry.planck_radiance([500,0],300)
