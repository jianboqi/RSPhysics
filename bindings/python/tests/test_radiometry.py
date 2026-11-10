import numpy as np
import pytest
import rsphysics as rsp

def test_calibration_flags_and_no_implicit_zero_mask_or_clipping():
    out=rsp.radiometry.linear_calibrate([[0,1,255],[np.nan,3,4]],gain=.2,offset=-.1,unit='W m^-2 sr^-1 um^-1',nodata=1,saturation=255,valid_mask=[[True,True,True],[True,False,True]])
    assert out['values'].shape==(2,3);assert out['values'][0,0]==-.1
    np.testing.assert_array_equal(out['flags'],[[0,2,4],[1,8,0]])
    np.testing.assert_array_equal(out['valid'],[[True,False,False],[False,False,True]])
    assert np.isnan(out['values'][0,1]);assert out['flags'].dtype==np.uint8
    assert rsp.radiometry.linear_calibrate([],gain=1,offset=0,unit='1')['values'].shape==(0,)

def test_product_reflectance_and_irradiance_paths_are_distinct():
    a=rsp.radiometry.dn_to_toa_reflectance([1000],reflectance_mult=2e-5,reflectance_add=-.1,solar_zenith_deg=60)
    np.testing.assert_allclose(a['values'],[-.16],rtol=1e-14)
    b=rsp.radiometry.radiance_to_toa_reflectance([1,2],solar_irradiance=np.pi,solar_zenith_deg=60,earth_sun_distance_au=1.02)
    np.testing.assert_allclose(b['values'],np.array([2,4])*1.02**2,rtol=1e-14)
    assert b['unit']=='1' and b['values'][1]>1

def test_brightness_temperature_units_masks_and_rational_formula():
    k1,k2=774.8853,1321.0789
    a=rsp.radiometry.radiance_to_brightness_temperature([10,0,-1,np.nan],k1=k1,k2=k2)
    np.testing.assert_allclose(a['values'][0],k2/np.log1p(k1/10),rtol=1e-14)
    np.testing.assert_array_equal(a['flags'],[0,16,16,1])
    b=rsp.radiometry.radiance_to_brightness_temperature(.01,k1=k1,k2=k2,radiance_unit='W m^-2 sr^-1 nm^-1')
    np.testing.assert_allclose(a['values'][0],b['values']);assert b['unit']=='K'
    # A huge positive ratio must not overflow in the log computation.
    assert np.isfinite(rsp.radiometry.radiance_to_brightness_temperature(1e-300,k1=1e100,k2=k2)['values'])

def test_invalid_metadata_is_not_hidden_by_empty_or_masked_data():
    for kw in [dict(gain=np.nan,offset=0,unit='1'),dict(gain=1,offset=0,unit=''),dict(gain=1,offset=0,unit='1',valid_mask=[True])]:
        with pytest.raises(rsp.ModelError):rsp.radiometry.linear_calibrate([],**kw)
    with pytest.raises(rsp.ModelError):rsp.radiometry.dn_to_toa_reflectance([],reflectance_mult=1,reflectance_add=0,solar_zenith_deg=90)
    with pytest.raises(rsp.ModelError):rsp.radiometry.radiance_to_toa_reflectance([],solar_irradiance=0,solar_zenith_deg=20)
    with pytest.raises(rsp.ModelError):rsp.radiometry.radiance_to_brightness_temperature([],k1=-1,k2=2)
    with pytest.raises(rsp.ModelError):rsp.radiometry.radiance_to_brightness_temperature([1],k1=1,k2=2,radiance_unit='unknown')
