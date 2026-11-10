from pathlib import Path
import json
import numpy as np
import pytest
import rsphysics as rsp
ROOT=Path(__file__).resolve().parents[3]
SUITE=json.loads((ROOT/'benchmarks/reference/foursail-tir/expected.json').read_text())

@pytest.mark.parametrize('case',SUITE['cases'],ids=lambda c:c['name'])
def test_independent_transfer_equation_reference(case):
    result=rsp.thermal.foursail(**case['parameters'])
    assert result['manifest']['model_id']=='thermal.foursail'
    for name,wanted in case['result'].items():np.testing.assert_allclose(result['result'][name],wanted,rtol=2e-10,atol=2e-9)

def test_spectral_and_angular_arrays_match_independent_oracle():
    result=rsp.thermal.foursail()['result']
    for name,wanted in SUITE['default_arrays'].items():np.testing.assert_allclose(result[name],wanted,rtol=2e-10,atol=2e-9)
    assert result['wavelengths_nm'].shape==(61,)
    assert result['hemisphere_brightness_temperature_k'].shape==(648,)

def test_native_batch_shapes_strides_empty_and_no_json_transport(monkeypatch):
    columns=rsp.thermal.FOURSAIL_PARAMETERS
    assert rsp.parameters_for('thermal.foursail')==columns
    matrix=rsp.parameter_grid('thermal.foursail',{'wavelength_nm':[9000.,10000.,12000.]})
    storage=np.empty((3,len(columns)*2));storage[:,::2]=matrix;view=storage[:,::2]
    from rsphysics import _native
    monkeypatch.setattr(_native,'run_batch_json',lambda *args:pytest.fail('JSON batch transport used'))
    batch=rsp.thermal.foursail_batch(view,shared_axes=True)
    assert batch['result']['radiance_w_m2_sr_um'].shape==(3,)
    assert batch['coordinates']['wavelengths_nm'].shape==(61,)
    for i,p in enumerate(matrix):
        result=rsp.thermal.foursail(**dict(zip(columns,p)))
        for key,value in result['result'].items():np.testing.assert_allclose(batch['coordinates'][key] if key in batch['coordinates'] else batch['result'][key][i],value,rtol=1e-12,atol=1e-12)
        assert batch['manifests'][i]==result['manifest']
    empty=rsp.thermal.foursail_batch(np.empty((0,len(columns))),shared_axes=True)
    assert empty['result']['spectral_radiance_w_m2_sr_um'].shape==(0,61)
    assert empty['manifests']==[]
    with pytest.raises(rsp.ModelError,match='row 1'):
        bad=matrix.copy();bad[1,columns.index('soil_emissivity')]=1.5;rsp.thermal.foursail_batch(bad)

def test_four_temperatures_are_separate_and_energy_components_sum():
    p=SUITE['cases'][0]['parameters'];o=rsp.thermal.foursail(**p)['result']
    fields=['sunlit_leaf_radiance_w_m2_sr_um','shaded_leaf_radiance_w_m2_sr_um','sunlit_soil_radiance_w_m2_sr_um','shaded_soil_radiance_w_m2_sr_um','reflected_sky_radiance_w_m2_sr_um','reflected_direct_radiance_w_m2_sr_um']
    assert sum(o[k] for k in fields)==pytest.approx(o['radiance_w_m2_sr_um'])
    for name in ['sunlit_leaf_temperature_k','shaded_leaf_temperature_k','sunlit_soil_temperature_k','shaded_soil_temperature_k']:
        assert rsp.thermal.foursail(**(p|{name:p[name]+5}))['result']['radiance_w_m2_sr_um']>o['radiance_w_m2_sr_um']
    assert o['sky_reflectance']+o['effective_emissivity']==pytest.approx(1.)
