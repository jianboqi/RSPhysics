import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp
ROOT=Path(__file__).resolve().parents[3]
SUITES={slug:json.loads((ROOT/f'benchmarks/reference/{slug}/expected.json').read_text()) for slug in ['tau-omega','water-cloud']}
CASES=[(slug,c) for slug,s in SUITES.items() for c in s['cases']]
@pytest.mark.parametrize('slug,case',CASES,ids=[slug+'-'+c['name'] for slug,c in CASES])
def test_all_reference_fields_and_sweeps(slug,case):
    envelope=getattr(rsp.microwave,slug.replace('-','_'))(**case['parameters'])
    actual=envelope['result']
    for key,expected in case['expected'].items():np.testing.assert_allclose(actual[key],expected,rtol=2e-12,atol=2e-12)
    assert envelope['manifest']['backend']=='python'
    assert 'wavelengths_nm' not in actual
    if 'sense_canopy' in case:
        for key,expected in case['sense_canopy'].items():np.testing.assert_allclose(actual[key],expected,rtol=2e-12,atol=2e-12)

@pytest.mark.parametrize('slug',['tau-omega','water-cloud'])
def test_batch_shapes_defaults_and_errors(slug):
    fn=slug.replace('-','_');columns=getattr(rsp.microwave,fn.upper()+'_PARAMETERS')
    params=SUITES[slug]['cases'][0]['parameters'];table=np.array([[params[k] for k in columns]]*2)
    out=getattr(rsp.microwave,fn+'_batch')(table)
    assert out['result']['sweep_total'].shape==(2,81)
    scalar='sigma0' if slug=='water-cloud' else 'brightness_temperature_k'
    assert out['result'][scalar].shape==(2,)
    empty=getattr(rsp.microwave,fn+'_batch')(np.empty((0,len(columns))))
    assert empty['result']['sweep_total'].shape==(0,81)
    assert empty['result'][scalar].shape==(0,)
    assert rsp.run('microwave.'+slug,{})['manifest']['parameters']==params
    for parameters in ({'observation_zenith_deg':90},{'observation_zenith_deg':float('nan')},{'unknown':1}):
        with pytest.raises(rsp.ModelError):rsp.run('microwave.'+slug,parameters)
    for array in ([1,2],np.empty((1,1)),[['bad']*len(columns)]):
        with pytest.raises(rsp.ModelError):getattr(rsp.microwave,fn+'_batch')(array)
    with pytest.raises(rsp.ModelError):rsp.run_batch('microwave.'+slug,[{}, {'observation_zenith_deg':90}])
