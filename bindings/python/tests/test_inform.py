import numpy as np
import pytest

import rsphysics as rsp


ID = "vegetation.inform"


def test_inform_numpy_spectra_and_independent_leaf_defaults():
    spectra = {
        "wavelengths_nm": np.array([550., 700., 850.]),
        "soil_reflectance": np.array([.1, .2, .3]),
    }
    response = rsp.vegetation.inform(
        crown_diameter_m=0, understory_lai=0,
        crown_leaf={"chlorophyll": 45}, understory_leaf={"water": .018},
        spectra=spectra,
    )
    output = response["result"]
    np.testing.assert_array_equal(output["reflectance"], spectra["soil_reflectance"])
    assert output["sunlit_gap_fraction"] == 1
    assert response["manifest"]["parameters"]["crown_leaf"]["n"] == 2
    assert response["manifest"]["parameters"]["understory_leaf"]["chlorophyll"] == 50
    with pytest.raises(rsp.ModelError, match="soil_reflectance"):
        rsp.vegetation.inform(spectra={**spectra, "soil_reflectance": [.1]})


def test_inform_numpy_batch_and_grid_match_scalar():
    assert rsp.parameters_for(ID) == rsp.vegetation.INFORM_PARAMETERS
    grid = rsp.parameter_grid(ID, {"view_zenith_deg": [0, 30]})
    spectra = {"wavelengths_nm": np.array([550., 680., 850.])}
    result = rsp.vegetation.inform_batch(grid, spectra=spectra)["result"]
    assert result["reflectance"].shape == (2, 3)
    assert result["crown_cover"].shape == (2,)
    assert rsp.vegetation.inform_batch(grid[:0], spectra=spectra)["result"]["reflectance"].shape == (0, 3)
    for i, zenith in enumerate((0, 30)):
        scalar = rsp.vegetation.inform(view_zenith_deg=zenith, spectra=spectra)["result"]
        np.testing.assert_allclose(result["reflectance"][i], scalar["reflectance"], atol=1e-14)
