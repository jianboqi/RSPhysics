import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

CASES=json.loads((Path(__file__).resolve().parents[3]/'benchmarks/reference/spectral/expected.json').read_text())['cases']
@pytest.mark.parametrize('case',CASES,ids=[c['name'] for c in CASES])
def test_exact_rational_reference(case):
    c=case
    for weighted in [False,True]:
        kw=dict(irradiance=c['irradiance'],irradiance_wavelengths_nm=c['irradiance_wavelengths_nm']) if weighted else {}
        actual=rsp.spectral.convolve(c['wavelengths_nm'],c['values'],c['response_wavelengths_nm'],c['response'],**kw)
        np.testing.assert_allclose(actual['values'],c['expected_irradiance_mean' if weighted else 'expected_response_mean'],rtol=2e-12,atol=2e-12)
        assert actual['coverage']==1

def test_shapes_noncontiguous_inputs_and_exact_integrals():
    x=np.array([400.,500.,700.]);values=np.array([[1.,2.,4.],[2.,4.,8.]])
    out=rsp.spectral.resample(x,values[:,::-1][:,::-1],[400,450,600,700])
    np.testing.assert_allclose(out,[[1,1.5,3,4],[2,3,6,8]])
    np.testing.assert_allclose(rsp.spectral.integrate(x,values),[750,1500])
    assert rsp.spectral.integrate(x,values.reshape(1,2,3)).shape==(1,2)
    assert rsp.spectral.convolve(x,np.empty((0,3)),[400,700],[1,1])['values'].shape==(0,)
    assert rsp.spectral.resample(x,np.empty((2,0,3)),[450,600]).shape==(2,0,2)
    scalar=rsp.spectral.integrate(x,values[0]);assert np.asarray(scalar).shape==()

def test_coverage_scale_invariance_and_missing_domain():
    x=[500,600,700];y=[1,2,3];rx=[400,500,600,700,800];r=[0,1,1,1,0]
    with pytest.raises(rsp.ModelError):rsp.spectral.convolve(x,y,rx,r)
    part=rsp.spectral.convolve(x,y,rx,r,allow_partial=True)
    np.testing.assert_allclose(part['coverage'],2/3);np.testing.assert_allclose(part['values'],2)
    scaled=rsp.spectral.convolve(x,y,rx,np.array(r)*1e-200,allow_partial=True)
    np.testing.assert_allclose(scaled['values'],part['values'])
    with pytest.raises(rsp.ModelError):rsp.spectral.convolve([900,1000],[1,2],rx,r,allow_partial=True)

@pytest.mark.parametrize('x,y', [([500,400],[1,2]),([400,400],[1,2]),([400,float('nan')],[1,2]),([400,500],[1,float('nan')]),([400,500],[1]),([0,500],[1,2])])
def test_invalid_axes_and_values(x,y):
    with pytest.raises(rsp.ModelError):rsp.spectral.resample(x,y,[450])

def test_invalid_response_and_illumination():
    for r in ([0,0],[-1,1],[1,float('nan')]):
        with pytest.raises(rsp.ModelError):rsp.spectral.convolve([400,500],[1,2],[400,500],r)
    for kw in [dict(irradiance=[0,0]),dict(irradiance=[-1,1]),dict(irradiance_wavelengths_nm=[400,500]),dict(irradiance=[1,1],irradiance_wavelengths_nm=[450,500]),dict(allow_partial='yes')]:
        with pytest.raises(rsp.ModelError):rsp.spectral.convolve([400,500],[1,2],[400,500],[1,1],**kw)
    with pytest.raises(rsp.ModelError):rsp.spectral.resample([400,500],[1,2],[399])
    with pytest.raises(rsp.ModelError):rsp.spectral.integrate([400,500],[1,2],lower_nm=600)
