"""Independent equations, physical balances and explicit formulation choices."""
import json
from pathlib import Path

import numpy as np
import pytest
import rsphysics as rsp

FIXTURE=json.loads((Path(__file__).resolve().parents[3]/'benchmarks/reference/fvcb/reference.json').read_text(encoding='utf8'))


@pytest.mark.parametrize('case',FIXTURE['cases'],ids=lambda x:x['model_id']+'/'+x['name'])
def test_published_equation_reference(case):
    result=rsp.run(case['model_id'],case['parameters'])['result']
    for name,expected in case['expected'].items():
        if expected is None or isinstance(expected,bool):assert result[name] is expected
        else:np.testing.assert_allclose(result[name],expected,rtol=2e-8,atol=2e-8,err_msg=name)
    for name,expected in case.get('curves',{}).items():
        np.testing.assert_allclose(result[name],expected,rtol=2e-8,atol=2e-8,err_msg=name)


@pytest.mark.parametrize('model',['ecosystem.fvcb','ecosystem.fvcb-big-leaf'])
@pytest.mark.parametrize('mode',['prescribed_ci','medlyn','ball_berry','leuning'])
def test_native_batch_and_shared_coordinates(model,mode):
    table=rsp.parameter_grid(model,{'vcmax25_umol_m2_s':[40.,80.],'jmax25_umol_m2_s':[100.,180.]})
    fixed={'gas_exchange_mode':'prescribed_ci'} if mode=='prescribed_ci' else {'stomatal_model':mode}
    result=rsp.run_array(model,table,shared_axes=True,fixed=fixed)
    columns=rsp.parameters_for(model)
    assert len(columns)==(28 if model.endswith('big-leaf') else 26)
    assert result['coordinates']['ci_axis_umol_mol'].shape==(61,)
    assert result['result']['ci_net_assimilation'].shape==(4,61)
    assert result['result']['light_net_assimilation'].shape==(4,51)
    for i,row in enumerate(table):
        scalar=rsp.run(model,dict(zip(columns,row))|fixed)['result']
        for key,values in result['result'].items():
            expected=scalar[key]
            if expected is None:assert np.isnan(values[i])
            else:np.testing.assert_allclose(values[i],expected,rtol=2e-12,atol=2e-12,err_msg=key)
    chunks=list(rsp.iter_array(model,table,shared_axes=True,chunk_size=3,fixed=fixed))
    np.testing.assert_array_equal(np.concatenate([x['result']['net_assimilation_umol_m2_s'] for x in chunks]),result['result']['net_assimilation_umol_m2_s'])


def test_medlyn_zero_intercept_and_conductance_units():
    r=rsp.ecosystem.fvcb(minimum_stomatal_conductance_water_mol_m2_s=0.)['result']
    assert r['intercellular_co2_umol_mol']/400.==pytest.approx(4/5,abs=1e-13)
    assert r['stomatal_conductance_water_mol_m2_s']==pytest.approx(1.6*r['stomatal_conductance_co2_mol_m2_s'])
    assert r['net_assimilation_umol_m2_s']==pytest.approx(r['stomatal_conductance_co2_mol_m2_s']*(400-r['intercellular_co2_umol_mol']),abs=1e-10)


def test_homogeneous_area_scaling():
    leaf=rsp.ecosystem.fvcb(absorbed_par_umol_m2_s=1000.)['result']
    canopy=rsp.ecosystem.fvcb_big_leaf(leaf_area_index=3.,nitrogen_extinction_coefficient=0.,absorbed_par_ground_umol_m2_s=3000.)['result']
    for key in ['net_assimilation_umol_m2_s','day_respiration_umol_m2_s','stomatal_conductance_water_mol_m2_s','transpiration_mmol_m2_s']:
        assert canopy[key]==pytest.approx(3*leaf[key],rel=1e-9,abs=1e-9)
    assert canopy['intercellular_co2_umol_mol']==pytest.approx(leaf['intercellular_co2_umol_mol'],abs=1e-8)


def test_respiring_darkness_and_tpu_switch():
    night=rsp.ecosystem.fvcb(absorbed_par_umol_m2_s=0.)['result']
    assert night['net_assimilation_umol_m2_s']==-night['day_respiration_umol_m2_s']
    assert night['intercellular_co2_umol_mol']>400.
    assert night['stomatal_conductance_water_mol_m2_s']==.01
    tpu=rsp.ecosystem.fvcb(tpu_limitation='simple',tpu_umol_m2_s=5.)['result']
    assert tpu['limitation_code']==2
    assert tpu['net_assimilation_umol_m2_s']==pytest.approx(3*5-1.2)
    disabled=rsp.ecosystem.fvcb(tpu_limitation='off',tpu_umol_m2_s=0.)['result']
    assert disabled['net_assimilation_umol_m2_s']>0


def test_fixed_ci_and_zero_canopy_missing_values_are_explicit():
    leaf=rsp.ecosystem.fvcb(gas_exchange_mode='prescribed_ci')['result']
    assert leaf['intercellular_co2_defined'] and not leaf['gas_exchange_computed']
    assert leaf['stomatal_conductance_water_mol_m2_s'] is None
    p=rsp.parameter_grid('ecosystem.fvcb-big-leaf',{},base={'leaf_area_index':0.,'absorbed_par_ground_umol_m2_s':0.})
    batch=rsp.ecosystem.fvcb_big_leaf_batch(p)
    assert np.isnan(batch['result']['intercellular_co2_umol_mol'][0])
    assert batch['result']['intercellular_co2_defined'][0]==0
    assert batch['result']['net_assimilation_umol_m2_s'][0]==0


def test_temperature_normalization_and_pressure_conversion():
    for mode in ['arrhenius','peaked_arrhenius']:
        r=rsp.ecosystem.fvcb(temperature_response=mode)['result']
        assert r['vcmax_umol_m2_s']==80.
        assert r['jmax_umol_m2_s']==160.
        assert r['kc_umol_mol']==404.9
        assert r['gamma_star_umol_mol']==42.75
    r=rsp.ecosystem.fvcb(pressure_pa=50000.)['result']
    assert r['kc_umol_mol']==pytest.approx(404.9*101325/50000)


@pytest.mark.parametrize('zero',['absorbed_par_umol_m2_s','vcmax25_umol_m2_s','jmax25_umol_m2_s'])
def test_subcompensation_limits_do_not_create_photorespiration_without_capacity(zero):
    r=rsp.ecosystem.fvcb(gas_exchange_mode='prescribed_ci',intercellular_co2_umol_mol=0.,**{zero:0.})['result']
    assert r['net_assimilation_umol_m2_s']==-r['day_respiration_umol_m2_s']


@pytest.mark.parametrize('parameters',[
    {'stomatal_model':'unknown'},{'wrong_input':1.},
    {'vapour_pressure_deficit_kpa':8.,'leaf_temperature_c':10.},
    {'temperature_response':'peaked_arrhenius','vcmax_activation_j_mol':100000.,'vcmax_deactivation_j_mol':50000.},
    {'minimum_stomatal_conductance_water_mol_m2_s':0.,'absorbed_par_umol_m2_s':0.},
    {'absorbed_par_umol_m2_s':float('nan')},
])
def test_invalid_combinations_are_reported(parameters):
    with pytest.raises(rsp.ModelError):rsp.run('ecosystem.fvcb',parameters)


def test_batch_strides_byte_order_empty_and_error_row():
    model='ecosystem.fvcb'
    table=rsp.parameter_grid(model,{'vcmax25_umol_m2_s':[40.,60.,80.,100.]})
    reference=rsp.run_array(model,table[::2])['result']['net_assimilation_umol_m2_s']
    converted=table[::2].astype('>f8')
    np.testing.assert_array_equal(rsp.run_array(model,converted)['result']['net_assimilation_umol_m2_s'],reference)
    empty=rsp.run_array(model,table[:0],shared_axes=True)
    assert empty['result']['ci_net_assimilation'].shape==(0,61)
    table[3,0]=-1.
    with pytest.raises(rsp.ModelError,match='batch row 3'):
        list(rsp.iter_array(model,table,chunk_size=2))
