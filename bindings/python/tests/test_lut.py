import numpy as np
import pytest

import rsphysics as rsp
from rsphysics import lut


def test_grid_levels_and_seeded_latin_hypercube_strata():
    grid = lut.sample_grid({"lai": [1, 2], "height": range(3, 6)})
    np.testing.assert_array_equal(grid.values, [[1, 3], [1, 4], [1, 5], [2, 3], [2, 4], [2, 5]])
    assert grid.names == ("lai", "height")
    with pytest.raises(rsp.ModelError, match="exceeding max_samples"):
        lut.sample_grid({"lai": [1, 2], "height": [3, 4]}, max_samples=3)
    ranges = {"lai": [1, 4], "roughness": [0.001, 0.01]}
    lhs = lut.sample_lhs(ranges, 32, seed=91, log_parameters=["roughness"])
    np.testing.assert_array_equal(lhs.values, lut.sample_lhs(
        ranges, 32, seed=lhs.design["seed"], log_parameters=["roughness"],
    ).values)
    unit_coordinates = np.column_stack(((lhs.values[:, 0] - 1) / 3, np.log10(lhs.values[:, 1]) + 3))
    np.testing.assert_array_equal(np.sort(np.floor(32 * unit_coordinates), axis=0),
                                  np.tile(np.arange(32)[:, None], (1, 2)))
    assert not lhs.values.flags.writeable


def test_external_table_matches_numpy_and_round_trips(tmp_path):
    channels = (lut.Channel("red", "reflectance"), lut.Channel("nir", "reflectance"))
    context = {"sensor": "instrument-a", "view_zenith_deg": 15}
    table = lut.LookupTable.from_arrays(
        {"lai": [1, 2, 3]}, [[0.1, 0.4], [0.12, 0.3], [0.1, 0.5]], channels, context=context,
    )
    file = tmp_path / "saved-table.npz"
    table.save(file)
    table.save(file)
    restored = lut.LookupTable.load(file)
    assert restored.context == context
    assert restored.channels == channels
    assert restored.samples.design == table.samples.design
    np.testing.assert_array_equal(restored.simulated, table.simulated)
    observations = np.array([[0.1, np.nan], [0.11, 0.34]])
    mask = np.isfinite(observations)
    sigma = [0.02, 0.03]
    result = restored.match(observations, channels=channels, context=context, k=2,
                            sigma=sigma, mask=mask, max_score=0.5)
    np.testing.assert_array_equal(result["indices"][0], [0, 2])
    np.testing.assert_array_equal(result["near_boundary"], [[True], [False]])
    np.testing.assert_array_equal(result["poor_fit"], [False, True])
    distances = np.sqrt(np.mean(((table.simulated - observations[1]) / sigma) ** 2, axis=1))
    np.testing.assert_allclose(result["scores"][1], np.sort(distances)[:2])
    assert np.isnan(result["residuals"][0, :, 1]).all()
    np.testing.assert_allclose(result["residuals"][1, 0], [0.01, -0.04])
    with pytest.raises(rsp.ModelError, match="names, quantities, units and order"):
        restored.match(observations, channels=channels[::-1], context=context)
    with pytest.raises(rsp.ModelError, match="context"):
        restored.match(observations, channels=channels, context={"sensor": "instrument-b"})
    with pytest.raises(rsp.ModelError, match="at least one active channel"):
        restored.match(observations[:1], channels=channels, context=context, mask=False)
    with pytest.raises(rsp.ModelError, match="sigma > 0"):
        restored.match(observations[1], channels=channels, context=context, sigma=[0.01, 0])
    empty = restored.match(np.empty((0, 2)), channels=channels, context=context)
    assert empty["parameters"].shape == (0, 1, 1)
    assert empty["residuals"].shape == (0, 1, 2)
    assert empty["poor_fit"] is None


def test_mixed_units_require_an_error_scale():
    channels = (lut.Channel("red", "reflectance"), lut.Channel("V", "brightness_temperature", "K"))
    table = lut.LookupTable.from_arrays({"p": [1, 2]}, [[0.1, 200], [0.2, 210]], channels)
    with pytest.raises(rsp.ModelError, match="different units"):
        table.match([0.1, 200], channels=channels)
    np.testing.assert_array_equal(table.match([0.1, 200], channels=channels, sigma=[0.01, 2])["indices"], [[0]])


def test_threaded_match_preserves_rank_scores_and_optional_residuals():
    rng = np.random.default_rng(47)
    channels = tuple(lut.Channel(str(i), "reflectance") for i in range(4))
    simulated = rng.random((173, 4))
    simulated[1] = simulated[0]
    observations = np.asfortranarray(simulated[:47])
    sigma = np.asfortranarray(rng.uniform(0.01, 0.03, observations.shape))
    mask = np.asfortranarray(rng.random(observations.shape) > 0.4)
    mask[:, 0] = True
    table = lut.LookupTable.from_arrays({"p": np.arange(len(simulated))}, simulated, channels)
    with pytest.raises(ValueError):
        table.simulated.setflags(write=True)
    expected = table.match(observations, channels=channels, sigma=sigma, mask=mask, k=4)
    actual = table.match(observations, channels=channels, sigma=sigma, mask=mask, k=4,
                         workers=8, return_residuals=False)
    np.testing.assert_array_equal(actual["indices"][0, :2], [0, 1])
    for name in ("indices", "scores", "parameters", "near_boundary"):
        np.testing.assert_array_equal(actual[name], expected[name])
    assert actual["residuals"] is None
    with pytest.raises(rsp.ModelError, match="workers"):
        table.match(observations, channels=channels, workers=0)


def test_image_blocks_handle_views_masks_missing_bands_and_empty_blocks():
    channels = (lut.Channel("red", "reflectance"), lut.Channel("nir", "reflectance"))
    table = lut.LookupTable.from_arrays({"lai": [1, 2, 3]}, [[0.1, 0.4], [0.12, 0.3], [0.2, 0.5]], channels)
    image = np.array([[[.1, .4], [.12, np.nan], [.12, .3]],
                      [[np.nan, np.nan], [.2, .5], [.2, .5]]], dtype=np.float32)[:, ::-1, :]
    pixel_mask = [[True, True, False], [False, False, True]]
    band_mask = np.ones_like(image, dtype=bool)
    band_mask[0, 0, 0] = False
    sigma = np.array([[[0.02]], [[0.03]]])
    blocks = list(table.iter_match(image, channels=channels, mask=band_mask, pixel_mask=pixel_mask,
                                   sigma=sigma, workers=4, block_size=2, k=2, return_residuals=True))
    assert [(b["start"], b["stop"]) for b in blocks] == [(0, 2), (2, 4), (4, 6)]
    np.testing.assert_array_equal(blocks[0]["pixel_indices"], [0, 1])
    assert blocks[1]["pixel_indices"].size == 0
    assert blocks[2]["pixel_indices"].size == 0
    expected = table.match(image[0, :2], channels=channels, mask=[[False, True], [True, False]],
                           sigma=0.02, k=2)
    for name in ("indices", "scores", "parameters", "residuals"):
        np.testing.assert_array_equal(blocks[0][name], expected[name])
    block = next(table.iter_match(image.reshape(-1, 2), channels=channels, block_size=6))
    np.testing.assert_array_equal(block["pixel_indices"], [0, 1, 2, 3, 4])
    assert block["residuals"] is None


@pytest.mark.parametrize("method", ["grid", "lhs"])
def test_inform_retrieves_held_out_lai_from_sensor_bands(method):
    sensor = rsp.sensors.load("landsat-8.oli").select(["B4", "B5", "B6"])
    channels = tuple(lut.Channel(band, "directional_reflectance") for band in sensor.band_ids)
    geometry = {"solar_zenith_deg": 30, "view_zenith_deg": 12, "relative_azimuth_deg": 40}
    context = {"sensor": sensor.id, **geometry}
    fixed = {**geometry, "stem_density_per_ha": 650, "diffuse_fraction": 0.1}
    samples = (lut.sample_grid({"crown_lai": np.linspace(0.5, 3.5, 16)}) if method == "grid"
               else lut.sample_lhs({"crown_lai": (0.5, 3.5)}, 16, seed=17))
    table = lut.build("vegetation.inform", samples, output="reflectance", channels=channels,
                      fixed=fixed, sensor=sensor, context=context, batch_size=5)
    truth = 2.35
    assert not np.any(samples.values == truth)
    spectrum = rsp.vegetation.inform(crown_lai=truth, **fixed)["result"]
    observed = sensor.convolve(spectrum["wavelengths_nm"], spectrum["reflectance"])["values"]
    observed += [0.00005, -0.00007, 0.00005]
    fitted = table.match(observed, channels=channels, context=context, k=3)
    assert abs(fitted["parameters"][0, 0, 0] - truth) < 0.25
    assert table.metadata["accepted_samples"] == 16
    assert table.metadata["sensor"]["bands"] == ["B4", "B5", "B6"]
    assert table.metadata["model"]["model_id"] == "vegetation.inform"


def test_i2em_uses_the_same_lut_interface_for_polarimetric_backscatter():
    samples = lut.sample_lhs({"rms_height_m": (0.0015, 0.008)}, 24, seed=4)
    channels = tuple(lut.Channel(p, "backscatter_coefficient", "dB") for p in ("VV", "HH"))
    fields = ("sigma0_vv_db", "sigma0_hh_db")
    fixed = {"frequency_ghz": 5.3, "observation_zenith_deg": 35, "correlation_length_m": 0.06}
    table = lut.build("microwave.i2em", samples, output=fields, channels=channels, fixed=fixed,
                      constraint=lambda row: row["rms_height_m"] < 0.007, batch_size=7)
    truth = 0.0047
    result = rsp.microwave.i2em(rms_height_m=truth, **fixed)["result"]
    fitted = table.match([result[key] for key in fields], channels=channels, sigma=[0.5, 0.5], k=2)
    assert abs(fitted["parameters"][0, 0, 0] - truth) < 0.0004
    assert table.metadata["attempted_samples"] == 24
    assert table.metadata["accepted_samples"] + table.metadata["rejected_samples"] == 24
    assert table.metadata["rejected_samples"] > 0
    assert table.samples.values[:, 0].max() < 0.007


def test_build_accepts_nested_model_fields_and_custom_projection():
    channels = (lut.Channel("850_nm", "directional_reflectance"),)
    samples = lut.sample_grid({"crown_leaf.chlorophyll": [25, 45]})
    spectra = {"wavelengths_nm": np.array([550., 850.])}
    fixed = {"crown_leaf": {"water": 0.02}, "spectra": spectra}
    table = lut.build("vegetation.inform", samples, output=lambda result: result["reflectance"][1:],
                      channels=channels, fixed=fixed)
    expected = rsp.vegetation.inform(crown_leaf={"water": 0.02, "chlorophyll": 45},
                                    spectra=spectra)["result"]["reflectance"][1]
    np.testing.assert_allclose(table.simulated[1], [expected], atol=1e-14)
    with pytest.raises(rsp.ModelError, match="already specified"):
        lut.build("vegetation.inform", samples, output="reflectance", channels=channels,
                  fixed={"crown_leaf": {"chlorophyll": 25}})
