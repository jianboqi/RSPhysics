import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[3]
IDS={'gsv':'soil.gsv','bsm':'soil.bsm','rpv':'surface.rpv','soilspect':'soil.soilspect'}
CASES=[]
for name,files in [('gsv',['reference.json']),('bsm',['reference.json','complete.json']),('rpv',['reference.json','precision.json']),('soilspect',['reference.json'])]:
    for file in files:
        for case in json.loads((ROOT/f'benchmarks/reference/{name}/{file}').read_text())['cases']:
            CASES.append((name,file,case))

@pytest.mark.parametrize('name,file,case',CASES,ids=[f'{n}-{f}-{c["name"]}' for n,f,c in CASES])
def test_independent_spectra_and_angular_reference(name,file,case):
    out=rsp.run(IDS[name],case['parameters'])['result']
    for key,expected in case['expected'].items():
        tol=2e-5 if 'hemispherical' in key else 1.1e-6 if name=='rpv' and file=='reference.json' else 2e-10 if name=='bsm' else 2e-12
        np.testing.assert_allclose(out[key],expected,atol=tol,rtol=0)
    for key in ['principal_brf','hemisphere_brf']:
        if key in case:np.testing.assert_allclose(out[key],case[key],atol=2e-11,rtol=0)

@pytest.mark.parametrize('name',IDS)
def test_scalar_numpy_strided_empty_and_selectors(name):
    id=IDS[name];describe=rsp.describe(id);cols=rsp.parameters_for(id)
    p={key:describe['input_schema']['properties'][key]['default'] for key in cols}
    row=np.array([p[k] for k in cols]);params=np.vstack([row,row]);backing=np.zeros((2,len(cols)*2));backing[:,::2]=params
    out=rsp.run_array(id,backing[:,::2],shared_axes=True);expected=rsp.run(id,p)['result']
    for key,v in out['result'].items():
        np.testing.assert_allclose(v[0],expected[key],atol=2e-12,rtol=0)
    assert len(out['manifests'])==2
    empty=rsp.run_array(id,np.empty((0,len(cols))),shared_axes=True)
    assert not empty['manifests']
    with pytest.raises(rsp.ModelError):rsp.run_array(id,np.zeros((2,len(cols)+1)))
    params[1,0]=np.nan
    with pytest.raises(rsp.ModelError,match='row 1'):rsp.run_array(id,params)
    with pytest.raises(rsp.ModelError):rsp.run(id,{'unexpected':1})

def test_gsv_original_modes_arbitrary_requests_and_domains():
    wl=[2500,400,405,405,999.25]
    a=rsp.soil.gsv(variant='gsv1',wavelengths_nm=wl)['result']
    b=rsp.soil.gsv(variant='gsv1',c2=3,c3=-2,wavelengths_nm=wl)['result']
    for key in a:np.testing.assert_array_equal(a[key],b[key])
    np.testing.assert_array_equal(a['wavelengths_nm'],wl)
    c=rsp.soil.gsv(variant='gsv3',c2=0,c3=0,wavelengths_nm=wl)['result']
    np.testing.assert_array_equal(a['reflectance'],c['reflectance'])
    np.testing.assert_allclose(np.asarray(a['dry_reflectance'])+a['moisture_contribution'],a['reflectance'],atol=1e-15)
    assert rsp.soil.gsv(c1=10,csm=0)['result']['outside_unit_interval_count']>0
    for wl in [[399.999],[2500.001],[float('nan')]]:
        with pytest.raises(rsp.ModelError):rsp.soil.gsv(wavelengths_nm=wl)
    with pytest.raises(rsp.ModelError):rsp.soil.gsv(csm=.01)
    rows=np.array([[.4,-.01,.04,-.13],[.7,0,0,0]])
    b=rsp.soil.gsv_batch(rows,wavelengths_nm=[400.,405.,2500.],variant='gsv1')
    assert b['result']['reflectance'].shape==(2,3)
    np.testing.assert_array_equal(b['result']['wavelengths_nm'],[400,405,2500])

def test_bsm_dry_threshold_poisson_mass_and_complete_correction():
    a=rsp.soil.bsm(soil_moisture_percent=0)['result'];b=rsp.soil.bsm(soil_moisture_percent=5)['result']
    for key in a:np.testing.assert_array_equal(a[key],b[key])
    np.testing.assert_array_equal(a['dry_reflectance'],a['wet_reflectance'])
    p=dict(soil_moisture_percent=55,moisture_capacity_percent=25)
    complete=rsp.soil.bsm(**p)['result'];legacy=rsp.soil.bsm(**p,poisson_mode='legacy_six_films')['result']
    assert complete['mean_film_count']==2
    assert 0<=complete['omitted_probability']<1e-14
    assert .004<legacy['omitted_probability']<.005
    assert np.all(np.asarray(complete['wet_reflectance'])>=legacy['wet_reflectance'])
    np.testing.assert_allclose(complete['dry_area_fraction'],np.exp(-2),atol=1e-15)
    with pytest.raises(rsp.ModelError):rsp.soil.bsm(moisture_capacity_percent=0)
    with pytest.raises(rsp.ModelError):rsp.soil.bsm(shape_longitude_deg=270)

def test_rpv_limits_reciprocity_and_four_parameter_hotspot():
    a=rsp.surface.rpv(solar_zenith_deg=50,view_zenith_deg=30)['result']
    b=rsp.surface.rpv(solar_zenith_deg=30,view_zenith_deg=50)['result']
    assert abs(a['brf']-b['brf'])<1e-14
    c=rsp.surface.rpv(hotspot_mode='independent_four_parameter',hotspot_rho=.2)['result']
    d=rsp.surface.rpv()['result']
    for key in c:np.testing.assert_array_equal(c[key],d[key])
    for z,a in [(0,0),(50,180),(85,90)]:
        out=rsp.surface.rpv(rho0=.3,minnaert_k=1,phase_asymmetry=0,hotspot_mode='independent_four_parameter',hotspot_rho=1,view_zenith_deg=z,relative_azimuth_deg=a)['result']
        np.testing.assert_allclose(out['hemisphere_brf'],.3,atol=2e-15)
    with pytest.raises(rsp.ModelError):rsp.surface.rpv(phase_asymmetry=-1)

def test_soilspect_all_phase_terms_full_flux_and_endpoint_stability():
    black=rsp.soil.soilspect(single_scattering_albedo=0)['result']
    assert black['brf']==black['bihemispherical_reflectance']==0
    for w in [.75,1.]:
        out=rsp.soil.soilspect(single_scattering_albedo=w,solar_zenith_deg=0,view_zenith_deg=0)['result']
        assert all(np.all(np.isfinite(v)) for v in out.values())
    a=rsp.soil.soilspect()['result'];b=rsp.soil.soilspect(phase_b_prime=.2,phase_c_prime=.2)['result']
    assert a['bihemispherical_reflectance']!=b['bihemispherical_reflectance']
    c=rsp.soil.soilspect(roughness_h=1)['result']
    assert c['directional_hemispherical_reflectance']>a['directional_hemispherical_reflectance']
    out=rsp.soil.soilspect(quadrature='refined')['result']
    assert abs(out['bihemispherical_reflectance']-a['bihemispherical_reflectance'])<=out['integration_error_estimate']+1e-15
    assert abs(out['brf']-out['single_scattering_brf']-out['multiple_scattering_brf'])<1e-15
