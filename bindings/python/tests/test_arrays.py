import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[3]
MODELS=['vegetation.prospect-d','vegetation.prosail-d','surface.ross-li','microwave.tau-omega','microwave.water-cloud','microwave.lmeb','ecosystem.gort-gpp','soil.sort-l']

@pytest.mark.parametrize('model',MODELS)
def test_typed_buffers_match_all_existing_references_and_json_runtime(model):
    slug=model.split('.')[1];suite=json.loads((ROOT/f'benchmarks/reference/{slug}/inputs.json').read_text())
    columns=rsp.parameters_for(model);cases=suite if isinstance(suite,list) else suite['cases']
    matrix=np.array([[c['parameters'][k] for k in columns] for c in cases])
    actual=rsp.run_array(model,matrix)
    for i,c in enumerate(cases):
        expected=rsp.run(model,c['parameters'])
        assert actual['manifests'][i]==expected['manifest']
        for key,value in expected['result'].items():
            np.testing.assert_allclose(actual['result'][key] if key=='wavelengths_nm' else actual['result'][key][i],value,rtol=2e-12,atol=2e-12)
    assert len(actual['manifests'])==len(matrix)
    if model=='surface.ross-li':assert actual['result']['negative_sample_count'].dtype==np.int64

@pytest.mark.parametrize('model',MODELS)
def test_shared_axes_empty_chunks_and_strided_matrices(model):
    matrix=rsp.parameter_grid(model,{})
    table=np.repeat(matrix,5,axis=0);view=np.zeros((5,matrix.shape[1]*2));view[:,::2]=table;view=view[:,::2]
    assert not view.flags.c_contiguous
    legacy=rsp.run_array(model,view);shared=rsp.run_array(model,view,shared_axes=True)
    assert shared['coordinates']
    for key,coordinate in shared['coordinates'].items():
        assert coordinate.ndim==1
        np.testing.assert_array_equal(coordinate,legacy['result'][key] if key=='wavelengths_nm' else legacy['result'][key][0])
        assert key not in shared['result']
    for key in shared['result']:np.testing.assert_array_equal(shared['result'][key],legacy['result'][key])
    chunks=list(rsp.iter_array(model,view,chunk_size=2,shared_axes=True));assert [c['row_offset'] for c in chunks]==[0,2,4]
    for key in shared['result']:np.testing.assert_allclose(np.concatenate([c['result'][key] for c in chunks]),shared['result'][key])
    empty=rsp.run_array(model,np.empty((0,len(rsp.parameters_for(model)))),shared_axes=True)
    assert all(v.shape[0]==0 for v in empty['result'].values());assert empty['manifests']==[]
    assert len(list(rsp.iter_array(model,np.empty((0,len(rsp.parameters_for(model)))))))==1
    # Float32 inputs are promoted, not viewed as wrong-width f64 buffers.
    rsp.run_array(model,table.astype('>f8'));rsp.run_array(model,table.astype(np.float32))

def test_grid_column_contract_and_global_error_rows():
    assert rsp.parameters_for('vegetation.prospect-d')==rsp.vegetation.PROSPECT_D_PARAMETERS
    assert rsp.parameters_for('vegetation.prosail-d')==rsp.vegetation.PROSAIL_D_PARAMETERS
    assert rsp.parameters_for('surface.ross-li')==rsp.surface.ROSS_LI_PARAMETERS
    assert rsp.parameters_for('microwave.tau-omega')==rsp.microwave.TAU_OMEGA_PARAMETERS
    assert rsp.parameters_for('microwave.water-cloud')==rsp.microwave.WATER_CLOUD_PARAMETERS
    model='vegetation.prospect-d';table=rsp.parameter_grid(model,{'chlorophyll':[10,20],'water':[.01,.02]},base={'n':2})
    assert table.shape==(4,8);assert np.all(table[:,0]==2)
    np.testing.assert_array_equal(table[:,1],[10,10,20,20])
    table[3,0]=0
    with pytest.raises(rsp.ModelError,match='row 3'):list(rsp.iter_array(model,table,chunk_size=2))
    for value in [np.zeros(8),np.zeros((1,7)),np.zeros((1,8),dtype=complex),[['x']*8]]:
        with pytest.raises(rsp.ModelError):rsp.run_array(model,value)
    for size in [0,-1,1.2,False]:
        with pytest.raises(rsp.ModelError):list(rsp.iter_array(model,table,chunk_size=size))
    with pytest.raises(rsp.ModelError):rsp.parameter_grid(model,{'typo':[1]})
    with pytest.raises(rsp.ModelError):rsp.parameter_grid(model,{'n':[1,2]},max_samples=1)
    with pytest.raises(rsp.ModelError):rsp.run_array('unknown.model',np.zeros((1,8)))

def test_numeric_batch_does_not_call_the_json_batch_transport(monkeypatch):
    from rsphysics import _native
    def forbidden(*args):raise AssertionError('scientific output traversed JSON batch transport')
    monkeypatch.setattr(_native,'run_batch_json',forbidden)
    for model in MODELS:
        out=rsp.run_array(model,rsp.parameter_grid(model,{}));assert len(out['manifests'])==1
