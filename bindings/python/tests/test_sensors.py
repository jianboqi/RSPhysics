import hashlib
import json
import os
from pathlib import Path
from importlib.resources import files
import numpy as np
import pytest
import rsphysics as rsp

ENTRIES=rsp.sensors.list_sensors()
BUNDLED=[e for e in ENTRIES if e['distribution']=='bundled']

@pytest.mark.parametrize('entry',BUNDLED,ids=[e['id'] for e in BUNDLED])
def test_all_bundled_profiles_are_pinned_and_constant_preserving(entry):
    sensor=rsp.sensors.load(entry['id']);assert sensor.band_ids==tuple(entry['band_ids'])
    profile=sensor.metadata
    assert profile['source']['sha256']==entry['source']['sha256']
    for band in sensor.bands:
        out=band.convolve(band.wavelengths_nm,np.full((2,len(band.response)),.314))
        np.testing.assert_allclose(out['values'],[.314,.314],rtol=2e-14,atol=2e-14)
        assert out['coverage']==1
        assert not band.response.flags.writeable
        assert band.wavelengths_nm[0]<=out['effective_wavelength_nm']<=band.wavelengths_nm[-1]

def test_offline_s2_registry_and_checksum_failures(tmp_path,monkeypatch):
    assert len(ENTRIES)==12
    assert sum(len(e['band_ids']) for e in ENTRIES)==112
    def forbidden(*a,**kw):raise AssertionError('load must never use network')
    monkeypatch.setattr(rsp.sensors,'urlopen',forbidden)
    with pytest.raises(rsp.ModelError,match='local official table'):rsp.sensors.load('sentinel-2a.msi',cache_dir=tmp_path)
    (tmp_path/'sentinel-2a-msi.json').write_text('{}')
    with pytest.raises(rsp.ModelError,match='checksum'):rsp.sensors.load('sentinel-2a.msi',cache_dir=tmp_path)
    original=tmp_path/'wrong.xlsx';original.write_bytes(b'not official')
    with pytest.raises(rsp.ModelError,match='checksum'):rsp.sensors.import_file('sentinel-2a.msi',original,cache_dir=tmp_path)
    # ESA response arrays and the original workbook must not be inside package resources.
    resources=files('rsphysics').joinpath('data/sensors')
    assert not any(p.name.startswith('sentinel-2') for p in resources.iterdir())

def test_multi_band_batch_and_explicit_domain_limits():
    canopy=rsp.vegetation.prosail_d()['result'];sensor=rsp.sensors.load('landsat-8.oli')
    batch=np.stack([canopy['brf'],canopy['brf']*2])
    out=sensor.select(['B4','B5']).convolve(canopy['wavelengths_nm'],batch)
    assert out['values'].shape==(2,2);assert out['bands']==('B4','B5')
    np.testing.assert_allclose(out['values'][1],out['values'][0]*2)
    for i,band in enumerate(sensor.select(['B4','B5']).bands):np.testing.assert_allclose(out['values'][0,i],band.convolve(canopy['wavelengths_nm'],canopy['brf'])['values'])
    thermal=rsp.sensors.load('landsat-9.tirs-2')
    with pytest.raises(rsp.ModelError,match='overlap'):thermal.convolve(canopy['wavelengths_nm'],canopy['brf'])
    modis=rsp.sensors.load('aqua.modis')
    with pytest.raises(rsp.ModelError):modis.convolve(canopy['wavelengths_nm'],canopy['brf'])
    partial=modis.convolve(canopy['wavelengths_nm'],canopy['brf'],allow_partial=True)
    assert np.all(partial['coverage']<=1) and np.any(partial['coverage']<1)
    assert 'not all 36' in modis.metadata['scope']

def test_preprocessing_and_custom_responses_are_explicit():
    l8=rsp.sensors.load('landsat-8.oli').metadata
    assert sum(b['negative_samples_clipped'] for b in l8['bands'])==43
    l4=rsp.sensors.load('landsat-4.tm').metadata
    assert [b['id'] for b in l4['bands'] if b['source_rows_reordered']]==['B5']
    with pytest.raises(rsp.ModelError):rsp.sensors.Bandpass('bad',[400,500],[-1,1])
    with pytest.raises(rsp.ModelError):rsp.sensors.Bandpass('bad',[500,400],[1,1])
    custom=rsp.sensors.Bandpass('test',[400,500,600],[0,1,0]);assert custom.convolve([400,500,600],[1,2,3])['values']==2
    with pytest.raises(rsp.ModelError):rsp.sensors.load('unknown')
    with pytest.raises(rsp.ModelError):rsp.sensors.load('landsat-8.oli').select(['unknown'])

@pytest.mark.skipif(not os.environ.get('RSPHYSICS_TEST_S2_CACHE'),reason='optional local ESA data; never fetched during normal tests')
def test_actual_official_s2_v5_import_and_all_three_platforms(tmp_path):
    cache=Path(os.environ['RSPHYSICS_TEST_S2_CACHE'])
    original=cache/'raw/sentinel-2-srf-v5.xlsx'
    rsp.sensors.import_file('sentinel-2a.msi',original,cache_dir=tmp_path)
    canopy=rsp.vegetation.prosail_d()['result']
    for platform in ['a','b','c']:
        sensor=rsp.sensors.load(f'sentinel-2{platform}.msi',cache_dir=tmp_path)
        assert sensor.version=='5.0';assert len(sensor.bands)==13
        out=sensor.convolve(canopy['wavelengths_nm'],canopy['brf'])
        assert out['values'].shape==(13,);np.testing.assert_array_equal(out['coverage'],np.ones(13))
