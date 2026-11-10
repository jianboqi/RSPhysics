"""Original MATLAB references and array contracts for the two supplied models."""
from pathlib import Path
import json
import numpy as np
import pytest
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[3]
IDS=[('microwave.lmeb','lmeb'),('ecosystem.gort-gpp','gort-gpp')]

@pytest.mark.parametrize('model_id,directory',IDS)
def test_original_matlab_cases_and_numpy(model_id,directory):
    ref=ROOT/'benchmarks'/'reference'/directory
    inputs=json.loads((ref/'inputs.json').read_text())
    expected=json.loads((ref/'expected.json').read_text())['cases']
    defaults={k:v['default'] for k,v in rsp.describe(model_id)['input_schema']['properties'].items()}
    for c in inputs:c['parameters']=defaults|c['parameters']
    fixed={k:v for k,v in inputs[0]['parameters'].items() if isinstance(v,str)}
    columns=rsp.parameters_for(model_id)
    matrix=np.array([[c['parameters'][n] for n in columns] for c in inputs])
    batch=rsp.run_array(model_id,matrix,shared_axes=True,fixed=fixed)
    for i,(case,want) in enumerate(zip(inputs,expected,strict=True)):
        single=rsp.run(model_id,case['parameters'])['result']
        for name,value in want.items():
            if name=='name':continue
            np.testing.assert_allclose(single[name],value,rtol=2e-9,atol=2e-9)
            np.testing.assert_allclose(batch['result'][name][i],value,rtol=2e-9,atol=2e-9)
    assert len(batch['manifests'])==32
    empty=rsp.run_array(model_id,matrix[:0],shared_axes=True,fixed=fixed)
    assert empty['result'][next(k for k in expected[0] if k!='name')].shape[0]==0
    chunks=list(rsp.iter_array(model_id,matrix,chunk_size=7,shared_axes=True,fixed=fixed))
    assert [c['row_offset'] for c in chunks]==[0,7,14,21,28]

def test_lmeb_physics_and_domains():
    result=rsp.microwave.lmeb(observation_zenith_deg=0.)['result']
    assert result['brightness_temperature_h_k']==pytest.approx(result['brightness_temperature_v_k'])
    bare=rsp.microwave.lmeb(nadir_optical_depth=0.)['result']
    assert bare['transmissivity_h']==1.
    assert bare['transmissivity_v']==1.
    assert len(bare['sweep_zenith_deg'])==81
    for p in [dict(soil_moisture=0.),dict(sand_fraction=.8,clay_fraction=.5),dict(scattering_albedo_h=.9,scattering_albedo_v_difference=.2)]:
        with pytest.raises(rsp.ModelError):rsp.microwave.lmeb(**p)

def test_gort_physical_meaning_and_domains():
    p=rsp.ecosystem.gort_gpp()['result']
    assert p['sunlit_leaf_area_index']+p['shaded_leaf_area_index']==pytest.approx(4.31)
    assert p['net_assimilation_umol_m2_s']==pytest.approx(p['sunlit_assimilation_umol_m2_s']*p['sunlit_leaf_area_index']+p['shaded_assimilation_umol_m2_s']*p['shaded_leaf_area_index'])
    dark=rsp.ecosystem.gort_gpp(direct_par_umol_m2_s=0.,diffuse_par_umol_m2_s=0.)['result']
    assert dark['net_assimilation_umol_m2_s']<0.  # Net assimilation includes respiration.
    assert np.all((p['between_crown_gaps']>=0)&(p['between_crown_gaps']<=1))
    for params in [dict(solar_zenith_deg=0.,numerical_convention="legacy_matlab"),dict(crown_base_height_m=20.,crown_top_height_m=10.),dict(air_temperature_c=10.,vapour_pressure_deficit_pa=5000.)]:
        with pytest.raises(rsp.ModelError):rsp.ecosystem.gort_gpp(**params)
