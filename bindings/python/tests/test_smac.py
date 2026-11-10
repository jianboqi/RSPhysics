import hashlib
import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[3]
SUITE=json.loads((ROOT/'benchmarks/reference/smac/expected.json').read_text())
ID='atmosphere.smac'

@pytest.mark.parametrize('case',SUITE['cases'],ids=lambda c:c['name'])
def test_original_reference(case):
    out=rsp.atmosphere.smac(**case['input'])
    for key,value in case['expected'].items():
        np.testing.assert_allclose(out['result'][key],value,rtol=2e-10,atol=2e-11)
    assert out['manifest']['parameters']==case['input']
    assert out['manifest']['backend']=='python'
    assert out['manifest']['data_hashes']['coefficients.json.zlib']=='sha256:'+SUITE['coefficient_pack_sha256']

@pytest.mark.parametrize('case',SUITE['curves'])
def test_original_sensitivity_and_numpy_batch(case):
    p=case['input'];columns=rsp.parameters_for(ID)
    assert columns==rsp.atmosphere.SMAC_PARAMETERS
    table=np.array([[p[k] for k in columns]]*3)
    strided=np.zeros((3,16));strided[:,::2]=table;strided=strided[:,::2]
    selectors={k:p[k] for k in ['coefficient_id','direction']}
    batch=rsp.run_array(ID,strided,shared_axes=True,fixed=selectors)
    dedicated=rsp.atmosphere.smac_batch(table,**selectors)
    assert batch['result']['reflectance'].shape==(3,)
    assert batch['result']['outside_unit_interval'].dtype==np.int64
    np.testing.assert_array_equal(batch['coordinates']['sweep_aot550'],case['sweep_aot550'])
    for row in batch['result']['sweep_reflectance']:
        np.testing.assert_allclose(row,case['sweep_reflectance'],rtol=2e-10,atol=2e-11)
    for key,values in batch['result'].items():np.testing.assert_array_equal(values,dedicated['result'][key])
    for dtype in ['>f8',np.float32]:rsp.atmosphere.smac_batch(table.astype(dtype),**selectors)
    empty=rsp.atmosphere.smac_batch(np.empty((0,8)),**selectors,shared_axes=True)
    assert empty['result']['sweep_reflectance'].shape==(0,21)
    assert empty['manifests']==[]
    chunks=list(rsp.iter_array(ID,table,chunk_size=2,fixed=selectors))
    assert [c['row_offset'] for c in chunks]==[0,2]

def test_coefficients_provenance_lossless_export_and_catalog():
    records=rsp.atmosphere.smac_coefficients();assert len(records)==367
    assert len({r['id'] for r in records})==367
    assert len(rsp.atmosphere.smac_coefficients(satellite='Sentinel-2C / MSI'))==13
    assert len(rsp.atmosphere.smac_coefficients(satellite='Landsat-8 / OLI'))==9
    for r in records:
        source=rsp.atmosphere.smac_coefficient_source(r['id'])
        assert hashlib.sha256(source['text'].encode('ascii')).hexdigest()==source['sha256']
        assert r['sha256']==source['sha256']
    records[0]['band']='mutated';assert rsp.atmosphere.smac_coefficients()[0]['band']!='mutated'
    schema=rsp.describe(ID)['input_schema']['properties']
    assert len(schema['coefficient_id']['enum'])==367
    assert schema['direction']['enum']==['toa_to_surface','surface_to_toa']
    assert not rsp.describe(ID)['metadata']['capabilities']['wasm']

def test_roundtrip_unclipped_and_invalid_inputs():
    forward=rsp.atmosphere.smac(direction='surface_to_toa',reflectance=.35)
    inverse=rsp.atmosphere.smac(reflectance=forward['result']['reflectance'])
    np.testing.assert_allclose(inverse['result']['reflectance'],.35,atol=2e-12)
    negative=rsp.atmosphere.smac(reflectance=0)['result']
    assert negative['reflectance']<0 and negative['outside_unit_interval']==1
    for p in [{'coefficient_id':'landsat-9.oli-2.B4.continental'},{'aot550':1.01},{'solar_zenith_deg':71},{'pressure_hpa':0},{'direction':'bogus'},{'direction':'surface_to_toa','reflectance':1.01},{'water_vapour_g_cm2':float('nan')}]:
        with pytest.raises(rsp.ModelError):rsp.atmosphere.smac(**p)
    with pytest.raises(rsp.ModelError):rsp.atmosphere.smac_coefficient_source('unknown')
    table=rsp.parameter_grid(ID,{'aot550':[0,.2,.5,1.]});table[3,5]=1.1
    with pytest.raises(rsp.ModelError,match='row 3'):list(rsp.iter_array(ID,table,chunk_size=2))
    for selectors in [{'reflectance':.5},{'coefficient_id':'unknown'},{'direction':'bogus'}]:
        with pytest.raises(rsp.ModelError):rsp.run_array(ID,np.empty((0,8)),fixed=selectors)
    with pytest.raises(rsp.ModelError):rsp.run_array('microwave.tau-omega',np.empty((0,6)),fixed={'direction':'toa_to_surface'})
