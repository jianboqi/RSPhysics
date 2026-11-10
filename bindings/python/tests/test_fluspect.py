import json,zlib
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

ROOT=Path(__file__).resolve().parents[3]/'benchmarks/reference/fluspect-cx'
CASES=json.loads((ROOT/'inputs.json').read_text(encoding='utf-8'))
EXPECTED=json.loads(zlib.decompress((ROOT/'expected.json.zlib').read_bytes()))
ID='vegetation.fluspect-cx'

@pytest.mark.parametrize('index',range(12))
def test_matlab_full_spectra_and_matrices(index):
    out=rsp.vegetation.fluspect_cx(**CASES[index])['result']
    for key,want in EXPECTED[index].items():
        np.testing.assert_allclose(out[key],want,atol=2e-8 if index==10 else 2e-10,rtol=2e-9)
    assert out['fluorescence_backward'].shape==(53,71)
    assert out['fluorescence_forward'].shape==(53,71)
    np.testing.assert_array_equal(out['wavelengths_nm'],np.arange(400,2401))
    np.testing.assert_array_equal(out['excitation_wavelengths_nm'],np.arange(400,751,5))
    np.testing.assert_array_equal(out['emission_wavelengths_nm'],np.arange(640,849,4))
    assert np.all(out['reflectance']+out['transmittance']<=1+1e-9)

def test_native_numpy_matrix_shapes_chunks_and_strides():
    cols=rsp.parameters_for(ID);assert cols==rsp.vegetation.FLUSPECT_CX_PARAMETERS
    table=np.array([[p[k] for k in cols] for p in CASES[:3]])
    view=np.zeros((3,18));view[:,::2]=table;view=view[:,::2]
    out=rsp.vegetation.fluspect_cx_batch(view,shared_axes=True)
    assert out['result']['fluorescence_backward'].shape==(3,53,71)
    assert out['result']['reflectance'].shape==(3,2001)
    assert set(out['coordinates'])=={'wavelengths_nm','excitation_wavelengths_nm','emission_wavelengths_nm'}
    for i,p in enumerate(CASES[:3]):
        scalar=rsp.vegetation.fluspect_cx(**p)
        assert scalar['manifest']==out['manifests'][i]
        for key,value in out['result'].items():np.testing.assert_allclose(value[i],scalar['result'][key],rtol=1e-13,atol=1e-14)
    chunks=list(rsp.iter_array(ID,view,chunk_size=2,shared_axes=True))
    assert [c['row_offset'] for c in chunks]==[0,2]
    np.testing.assert_array_equal(np.concatenate([c['result']['fluorescence_forward'] for c in chunks]),out['result']['fluorescence_forward'])
    empty=rsp.vegetation.fluspect_cx_batch(view[:0],shared_axes=True)
    assert empty['result']['fluorescence_forward'].shape==(0,53,71)
    assert empty['result']['reflectance'].shape==(0,2001)

def test_linear_efficiency_zero_fluorescence_and_xanthophyll():
    a=rsp.vegetation.fluspect_cx()['result'];b=rsp.vegetation.fluspect_cx(fluorescence_efficiency=.02)['result']
    np.testing.assert_array_equal(a['reflectance'],b['reflectance'])
    np.testing.assert_allclose(2*a['fluorescence_backward'],b['fluorescence_backward'],rtol=1e-13)
    z=rsp.vegetation.fluspect_cx(fluorescence_efficiency=0)['result']
    assert np.count_nonzero(z['fluorescence_backward'])==0
    cx=rsp.vegetation.fluspect_cx(xanthophyll_state=1)['result']
    assert np.max(np.abs(cx['reflectance']-a['reflectance']))>1e-4

@pytest.mark.parametrize('kw',[{'n':.9},{'fluorescence_efficiency':-.01},{'water':-.01},{'xanthophyll_state':-999},{'chlorophyll':float('nan')}])
def test_invalid_parameters(kw):
    with pytest.raises(rsp.ModelError):rsp.vegetation.fluspect_cx(**kw)
