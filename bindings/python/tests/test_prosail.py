"""Author-R positive-LAI cases plus a separately labelled analytical bare-soil limit."""
import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

SUITE=json.loads((Path(__file__).resolve().parents[3]/'benchmarks/reference/prosail-d/expected.json').read_text())
ID='vegetation.prosail-d'
DEFAULTS={k:v['default'] for k,v in rsp.describe(ID)['input_schema']['properties'].items()}
for c in SUITE['cases']:c['parameters']=DEFAULTS|c['parameters']
def compare(result,case):
    np.testing.assert_array_equal(result['wavelengths_nm'],SUITE['wavelengths_nm'])
    for key in ('brf','hemispherical_reflectance'):
        assert result[key].dtype==np.float64
        assert result[key].shape==(2101,)
        np.testing.assert_allclose(result[key],case[key],atol=SUITE['tolerance']['absolute'],rtol=SUITE['tolerance']['relative'])

@pytest.mark.parametrize('case',SUITE['cases'],ids=lambda c:c['name'])
def test_author_or_analytical_reference(case):
    envelope=rsp.vegetation.prosail_d(**case['parameters'])
    compare(envelope['result'],case)
    assert envelope['manifest']['backend']=='python'
    assert envelope['manifest']['parameters']==case['parameters']
    assert envelope['manifest']['data_hashes']['soil_coefficients']=='sha256:'+SUITE['soil_sha256']

def test_generic_numpy_batch_and_semantics():
    compare(rsp.run(ID)['result'],SUITE['cases'][0])
    matrix=np.array([[c['parameters'][key] for key in rsp.vegetation.PROSAIL_D_PARAMETERS] for c in SUITE['cases']])
    batch=rsp.vegetation.prosail_d_batch(matrix)
    assert batch['result']['brf'].shape==(25,2101)
    for i,case in enumerate(SUITE['cases']):
        compare({key:value if key=='wavelengths_nm' else value[i] for key,value in batch['result'].items()},case)
    assert rsp.vegetation.prosail_d_batch(np.empty((0,18)))['result']['brf'].shape==(0,2101)
    descriptor=rsp.describe(ID)['metadata']
    assert descriptor['capabilities']['wasm'] is False
    assert descriptor['output_semantics']['/brf']['quantity']=='brf'
    assert 'transmittance' not in rsp.run(ID)['result']
    # A directional BRF can exceed unity; it is not clipped like a hemispherical flux.
    hot=rsp.vegetation.prosail_d(n=3,chlorophyll=0,carotenoids=0,water=.001,dry_matter=.001,leaf_area_index=8,solar_zenith_deg=80,view_zenith_deg=80,relative_azimuth_deg=0)
    assert hot['result']['brf'].max()>1

def test_errors_and_batch_rows():
    for parameters in ({'leaf_area_index':-1},{'view_zenith_deg':90},{'leaf_angle_distribution':'verhoef','lidf_a':.8,'lidf_b':.8}):
        with pytest.raises(rsp.ModelError):rsp.run(ID,parameters)
    with pytest.raises(rsp.ModelError,match='row 1'):rsp.run_batch(ID,[{}, {'hotspot':-1}])
    for value in (np.zeros((2,15)),np.zeros(16),np.zeros((1,16),dtype=complex)):
        with pytest.raises(rsp.ModelError):rsp.vegetation.prosail_d_batch(value)
