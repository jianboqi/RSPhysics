import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp
from rsphysics import microwave

ROOT=Path(__file__).resolve().parents[3]/'benchmarks/reference/i2em'
CASES=json.loads((ROOT/'inputs.json').read_text(encoding='utf-8'))
EXPECTED=json.loads((ROOT/'expected.json').read_text(encoding='utf-8'))

@pytest.mark.parametrize('index',range(20))
def test_original_matlab(index):
    out=microwave.i2em(**CASES[index])['result']
    np.testing.assert_allclose([out['sigma0_vv_db'],out['sigma0_hh_db']],EXPECTED[index],atol=2e-9,rtol=0)
    np.testing.assert_allclose([out['sigma0_vv'],out['sigma0_hh']],10**(np.array(EXPECTED[index])/10),rtol=1e-10)

@pytest.mark.parametrize('correlation',['exponential','gaussian'])
def test_numpy_and_grid(correlation):
    columns=rsp.parameters_for('microwave.i2em')
    assert columns==microwave.I2EM_PARAMETERS
    cases=[p for p in CASES if p['correlation_function']==correlation]
    matrix=np.array([[p[k] for k in columns] for p in cases])
    batch=microwave.i2em_batch(matrix,correlation_function=correlation)['result']
    for i,p in enumerate(cases):
        scalar=microwave.i2em(**p)['result']
        for key in ['sigma0_vv','sigma0_hh','sigma0_vv_db','sigma0_hh_db','ks','height_length_ratio']:
            assert batch[key].shape==(len(cases),)
            np.testing.assert_allclose(batch[key][i],scalar[key],rtol=1e-13)
    empty=microwave.i2em_batch(matrix[:0],correlation_function=correlation)
    assert empty['result']['sigma0_vv'].shape==(0,)
    grid=rsp.parameter_grid('microwave.i2em',{'observation_zenith_deg':[20,40,60]})
    direct=rsp.run_array('microwave.i2em',grid,fixed={'correlation_function':correlation})
    chunks=list(rsp.iter_array('microwave.i2em',grid,chunk_size=2,fixed={'correlation_function':correlation}))
    np.testing.assert_array_equal(np.concatenate([c['result']['sigma0_hh'] for c in chunks]),direct['result']['sigma0_hh'])

@pytest.mark.parametrize('kwargs',[{'rms_height_m':0.},{'rms_height_m':.1},{'correlation_length_m':.01},{'permittivity_loss':-1.},{'observation_zenith_deg':0.},{'correlation_function':'x_power'}])
def test_domain(kwargs):
    with pytest.raises(rsp.ModelError):microwave.i2em(**kwargs)

def test_batch_error_and_selector():
    grid=rsp.parameter_grid('microwave.i2em',{'observation_zenith_deg':[20,40]})
    grid[1,3]=.1
    with pytest.raises(rsp.ModelError,match='row 1'):microwave.i2em_batch(grid)
    with pytest.raises(rsp.ModelError):rsp.run_array('microwave.i2em',grid[:1],fixed={'frequency_ghz':5.3})
