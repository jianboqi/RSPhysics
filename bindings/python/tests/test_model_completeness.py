"""Complete forward modes, independent source references, and physical invariants."""
from pathlib import Path
import json,zlib
import numpy as np
import pytest
import rsphysics as rsp
ROOT=Path(__file__).resolve().parents[3]/'benchmarks/reference'
def read(model,name):return json.loads((ROOT/model/name).read_text(encoding='utf-8'))

@pytest.mark.parametrize('model',['lmeb','gort-gpp'])
def test_complete_matlab_modes_through_scalar_and_native_numpy(model):
    cases=read(model,'full-inputs.json');expected=read(model,'full-expected.json')
    if model=='gort-gpp':cases=[c['parameters'] for c in cases];expected=expected['cases']
    id=('ecosystem.' if model=='gort-gpp' else 'microwave.')+model
    defaults=rsp.run(id)['manifest']['parameters'];cols=rsp.parameters_for(id)
    for p,want in zip(cases,expected,strict=True):
        o=rsp.run(id,p);fixed={k:v for k,v in p.items() if isinstance(v,str)}
        batch=rsp.run_array(id,np.array([[(defaults|p)[k] for k in cols]]),fixed=fixed)
        assert batch['manifests'][0]==o['manifest']
        for key,value in want.items():
            if key=='name':continue
            np.testing.assert_allclose(o['result'][key],value,rtol=2e-9,atol=2e-9)
            np.testing.assert_allclose(batch['result'][key][0],value,rtol=2e-9,atol=2e-9)

def test_all_nine_4sail_outputs_and_distributions():
    cases=json.loads(zlib.decompress((ROOT/'prosail-d/full-expected.json.zlib').read_bytes()))['cases'];cols=rsp.parameters_for('vegetation.prosail-d')
    defaults=rsp.run('vegetation.prosail-d')['manifest']['parameters']
    for c in cases:
        p=defaults|c['parameters'];o=rsp.vegetation.prosail_d(**p)['result']
        batch=rsp.vegetation.prosail_d_batch(np.array([[p[k] for k in cols]]),leaf_angle_distribution=p['leaf_angle_distribution'])['result']
        for key,v in c['result'].items():
            np.testing.assert_allclose(o[key],v,rtol=2e-8,atol=2e-10)
            np.testing.assert_allclose(batch[key][0],v,rtol=2e-8,atol=2e-10)
    conservative=rsp.vegetation.prosail_d(chlorophyll=0,carotenoids=0,water=0,dry_matter=0)['result']
    np.testing.assert_allclose(conservative['direct_absorptance'],0,atol=2e-12)
    np.testing.assert_allclose(conservative['diffuse_absorptance'],0,atol=2e-12)

@pytest.mark.parametrize('index',range(31))
def test_goms_complete_c_reference(index):
    cases=read('goms','inputs.json');want=json.loads(zlib.decompress((ROOT/'goms/expected.json.zlib').read_bytes()))[index]
    p=cases[index];o=rsp.run('vegetation.goms',p)['result']
    named=rsp.vegetation.goms(**p)['result']
    for key,value in want.items():
        np.testing.assert_allclose(o[key],value,rtol=2e-9,atol=2e-10)
        np.testing.assert_array_equal(named[key],o[key])

def test_goms_numpy_and_band_vectors():
    id='vegetation.goms';grid=rsp.parameter_grid(id,{'view_zenith_deg':[0,30,60]})
    assert grid.shape==(3,13);assert rsp.parameters_for(id)==rsp.vegetation.GOMS_PARAMETERS
    o=rsp.vegetation.goms_batch(grid,shared_axes=True)
    assert o['result']['principal_brf'].shape==(3,1,171)
    assert o['result']['hemisphere_brf'].shape==(3,1,648)
    assert o['result']['invalid_weight_count'].dtype==np.int64
    assert o['coordinates']['principal_view_zenith_deg'].shape==(171,)
    empty=rsp.vegetation.goms_batch(grid[:0]);assert empty['result']['principal_brf'].shape==(0,1,171)
    mult=rsp.vegetation.goms(sunlit_ground_reflectance=[.1,.2],sunlit_crown_reflectance=[.5,.6],shaded_reflectance=.04)['result']
    assert mult['brf'].shape==(2,);assert mult['principal_brf'].shape==(2,171)
    for p in [dict(sunlit_ground_reflectance=[]),dict(sunlit_ground_reflectance=[1.2]),dict(sunlit_ground_reflectance=[.1,.2],sunlit_crown_reflectance=[.3,.4,.5]),dict(sunlit_ground_reflectance=[.1]*11)]:
        with pytest.raises(rsp.ModelError):rsp.vegetation.goms(**p)

def test_goms_multiband_native_buffers_and_strides(monkeypatch):
    from rsphysics import _native
    def forbidden(*args):raise AssertionError('multiband numerical results traversed JSON transport')
    monkeypatch.setattr(_native,'run_batch_json',forbidden)
    id='vegetation.goms';cols=rsp.parameters_for(id);geometry_cols=rsp.vegetation.GOMS_GEOMETRY_PARAMETERS
    grid=rsp.parameter_grid(id,{'view_zenith_deg':[10.,40.]})
    geometry=grid[:,[cols.index(k) for k in geometry_cols]]
    optics=np.array([[[.1,.3,.02],[.2,.5,.04]],[[.15,.4,.01],[.3,.6,.05]]])
    out=rsp.vegetation.goms_multiband_batch(geometry[:,::-1][:,::-1],optics[:,::-1],shared_axes=True)
    assert out['result']['brf'].shape==(2,2)
    assert out['result']['principal_brf'].shape==(2,2,171)
    for i in range(2):
        params=dict(zip(geometry_cols,geometry[i]));coefs=optics[i,::-1]
        scalar=rsp.vegetation.goms(**params,sunlit_ground_reflectance=coefs[:,0],sunlit_crown_reflectance=coefs[:,1],shaded_reflectance=coefs[:,2])
        assert out['manifests'][i]==scalar['manifest']
        for key,value in out['result'].items():np.testing.assert_allclose(value[i],scalar['result'][key],atol=2e-13,rtol=2e-13)
    empty=rsp.vegetation.goms_multiband_batch(geometry[:0],optics[:0]);assert empty['result']['hemisphere_brf'].shape==(0,2,648)
    for g,o in [(geometry[:,:9],optics),(geometry,optics[:1]),(geometry,optics[:,:,:2]),(geometry,np.zeros((2,11,3)))]:
        with pytest.raises(rsp.ModelError):rsp.vegetation.goms_multiband_batch(g,o)

def test_all_model_string_selectors_have_direct_schema_types():
    for model in rsp.list_models():
        schema=model['input_schema']['properties']
        for key,s in schema.items():
            if isinstance(s.get('default'),str):
                assert s['type']=='string',(model['metadata']['id'],key,s)
                assert s['default'] in s['enum']

def test_all_i2em_correlations_and_shape_parameter():
    cols=rsp.parameters_for('microwave.i2em')
    for p,want in zip(read('i2em','full-inputs.json'),read('i2em','full-expected.json'),strict=True):
        o=rsp.microwave.i2em(**p)['result']
        np.testing.assert_allclose([o['sigma0_vv_db'],o['sigma0_hh_db']],want,atol=2e-7,rtol=0)
        batch=rsp.microwave.i2em_batch(np.array([[p[k] for k in cols]]),correlation_function=p['correlation_function'])['result']
        np.testing.assert_allclose(batch['sigma0_hh_db'][0],want[1],atol=2e-7,rtol=0)

def test_water_cloud_independent_descriptors_and_vwc_compatibility():
    p=dict(scattering_descriptor=3.,attenuation_descriptor=.5,descriptor_mode='independent')
    o=rsp.microwave.water_cloud(**p)['result'];c=np.cos(np.deg2rad(40));t=np.exp(-2*.2*.5/c)
    assert o['sigma0']==pytest.approx(.1*3*c*(1-t)+.1*t)
    table=rsp.parameter_grid('microwave.water-cloud',{},base={'scattering_descriptor':3.,'attenuation_descriptor':.5})
    b=rsp.microwave.water_cloud_batch(table,descriptor_mode='independent')['result']
    assert b['sigma0'][0]==pytest.approx(o['sigma0'])

def test_prior_leaf_kernel_and_passive_model_invariants():
    # Energy conservation, reciprocity and isothermal equilibrium are independent
    # physical constraints, not copies of implementation equations.
    for n in [1.,1.5,5.]:
        o=rsp.vegetation.prospect_d(n=n,chlorophyll=0,carotenoids=0,water=0,dry_matter=0)['result']
        np.testing.assert_allclose(o['reflectance']+o['transmittance'],1.,atol=2e-12)
    for s,v,az in [(0,0,0),(10,60,70),(80,40,180),(35,35,0)]:
        a=rsp.surface.ross_li(solar_zenith_deg=s,view_zenith_deg=v,relative_azimuth_deg=az)['result']
        b=rsp.surface.ross_li(solar_zenith_deg=v,view_zenith_deg=s,relative_azimuth_deg=az)['result']
        assert a['brf']==pytest.approx(b['brf'],abs=2e-13)
    for tau in [0,1e-12,.3,5.]:
        o=rsp.microwave.tau_omega(soil_temperature_k=300,vegetation_temperature_k=300,soil_reflectivity=0,single_scattering_albedo=0,vegetation_optical_depth=tau)['result']
        assert o['brightness_temperature_k']==pytest.approx(300,abs=1e-12)
