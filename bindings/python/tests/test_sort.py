import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[3]

@pytest.mark.parametrize('slug',['sort','sort-l'])
def test_complete_spectra_and_numpy_selectors_match_original_python(slug):
    cases=json.loads((ROOT/f'benchmarks/reference/{slug}/inputs.json').read_text())
    expected=json.loads((ROOT/f'benchmarks/reference/{slug}/expected.json').read_text())['cases']
    model='soil.'+slug;columns=rsp.parameters_for(model)
    for case,want in zip(cases,expected):
        parameters=case['parameters'];out=rsp.run(model,parameters)
        fixed={'roughness_correction':parameters['roughness_correction']} if slug=='sort' else None
        array=rsp.run_array(model,np.array([[parameters[k] for k in columns]]),fixed=fixed)
        assert array['manifests'][0]==out['manifest']
        for key,values in out['result'].items():
            np.testing.assert_allclose(values,want[key],atol=2e-9,rtol=2e-9)
            np.testing.assert_allclose(array['result'][key] if key=='wavelengths_nm' else array['result'][key][0],values,atol=2e-12,rtol=2e-12)
        assert out['manifest']['data_hashes']['soil_basis']=='sha256:ad52839c4b9ad852595c0830ec9332476b399be7fb9f582e881c2acd8bfd4e88'

def test_soil_wrappers_dry_limits_batch_empty_and_errors():
    for call in [rsp.soil.sort,rsp.soil.sort_l]:
        result=call(water_coverage=0)['result'];keys=('dry_brf','wet_brf') if 'dry_brf'in result else ('dry_reflectance','wet_reflectance')
        np.testing.assert_array_equal(result[keys[0]],result[keys[1]])
    for model in ['soil.sort','soil.sort-l']:
        columns=rsp.parameters_for(model);base=rsp.parameter_grid(model,{})
        batch=np.repeat(base,3,axis=0);chunks=list(rsp.iter_array(model,batch,chunk_size=2,shared_axes=True))
        assert [c['row_offset'] for c in chunks]==[0,2]
        empty=rsp.run_array(model,np.empty((0,len(columns))),shared_axes=True)
        assert empty['coordinates']['wavelengths_nm'].shape==(2001,)
        assert all(x.shape==(0,2001) for x in empty['result'].values())
        endian=rsp.run_array(model,batch.astype('>f8'));assert len(endian['manifests'])==3
    with pytest.raises(rsp.ModelError):rsp.soil.sort(roughness_correction='shiltz_full')
    with pytest.raises(rsp.ModelError):rsp.soil.sort(water_film_thickness_mm=-1)
    with pytest.raises(rsp.ModelError):rsp.soil.sort(phase_asymmetry=0,phase_backscatter=.5)
    with pytest.raises(rsp.ModelError):rsp.soil.sort_l(soil_brightness=5)
    with pytest.raises(rsp.ModelError):rsp.run_array('soil.sort',rsp.parameter_grid('soil.sort',{}),fixed={'water_film_mode':'adding'})
