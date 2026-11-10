import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp
SUITE=json.loads((Path(__file__).resolve().parents[3]/'benchmarks/reference/ross-li/expected.json').read_text())
ID='surface.ross-li'
def compare(actual,case):
    for key,value in case['point'].items():np.testing.assert_allclose(actual[key],value,atol=SUITE['tolerance']['absolute'],rtol=SUITE['tolerance']['relative'])
    for key in ('principal_brf','hemisphere_brf'):
        assert actual[key].dtype==np.float64
        np.testing.assert_allclose(actual[key],case[key],atol=SUITE['tolerance']['absolute'],rtol=SUITE['tolerance']['relative'])
        np.testing.assert_allclose(actual[key],case['original_'+key],atol=SUITE['original_tolerance']['absolute'],rtol=SUITE['original_tolerance']['relative'])
    for key,value in case['original_point'].items():np.testing.assert_allclose(actual[key],value,atol=SUITE['original_tolerance']['absolute'],rtol=SUITE['original_tolerance']['relative'])
    np.testing.assert_array_equal(actual['principal_view_zenith_deg'],np.arange(-85,86))
    np.testing.assert_array_equal(actual['hemisphere_view_zenith_deg'],np.repeat(np.arange(0,86,5),36))
    np.testing.assert_array_equal(actual['hemisphere_azimuth_deg'],np.tile(np.arange(0,360,10),18))
@pytest.mark.parametrize('case',SUITE['cases'],ids=lambda c:c['name'])
def test_original_c_and_80_digit_reference(case):
    envelope=rsp.surface.ross_li(**case['parameters']);compare(envelope['result'],case)
    assert envelope['manifest']['backend']=='python'
    assert envelope['manifest']['parameters']==case['parameters']
    assert envelope['manifest']['data_hashes']=={}
def test_batch_scalar_and_array_shapes_and_metadata():
    matrix=np.array([[c['parameters'][k] for k in rsp.surface.ROSS_LI_PARAMETERS] for c in SUITE['cases']])
    batch=rsp.surface.ross_li_batch(matrix)
    assert batch['result']['brf'].shape==(32,)
    assert batch['result']['principal_brf'].shape==(32,171)
    assert batch['result']['hemisphere_brf'].shape==(32,648)
    for i,c in enumerate(SUITE['cases']):compare({k:v[i] for k,v in batch['result'].items()},c)
    compare(rsp.run(ID)['result'],SUITE['cases'][0])
    assert rsp.surface.ross_li_batch(np.empty((0,6)))['result']['hemisphere_brf'].shape==(0,648)
    metadata=rsp.describe(ID)['metadata']
    assert metadata['output_semantics']['/brdf']['unit']=='sr^-1'
    assert metadata['output_semantics']['/brf']['unit']=='1'
    assert metadata['capabilities']['wasm'] is False
    assert 'wavelengths_nm' not in rsp.run(ID)['result']
def test_invalid_parameters_and_raw_nonphysical_values():
    for kwargs in ({'f_iso':-1},{'f_vol':float('nan')},{'view_zenith_deg':90},{'relative_azimuth_deg':360}):
        with pytest.raises(rsp.ModelError):rsp.surface.ross_li(**kwargs)
    with pytest.raises(rsp.ModelError,match='row 1'):rsp.run_batch(ID,[{}, {'f_geo':-1}])
    with pytest.raises(rsp.ModelError):rsp.surface.ross_li_batch(np.zeros((2,5)))
    result=rsp.surface.ross_li(f_iso=0,f_vol=0,f_geo=1)['result']
    assert result['brf']<0 and result['negative_sample_count']>0
    assert rsp.surface.ross_li(f_iso=1,f_vol=1,f_geo=0,solar_zenith_deg=85,view_zenith_deg=85)['result']['brf']>1
