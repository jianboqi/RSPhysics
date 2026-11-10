"""Installed-wheel checks against the independent author-R reference spectra."""
import json
from pathlib import Path
import numpy as np
import pytest
import rsphysics as rsp

ROOT = Path(__file__).resolve().parents[3]
SUITE = json.loads((ROOT / "benchmarks/reference/prospect-d/expected.json").read_text(encoding="utf-8"))
ID = "vegetation.prospect-d"


def assert_spectra(actual, case):
    np.testing.assert_array_equal(actual["wavelengths_nm"], SUITE["wavelengths_nm"])
    for key in ("reflectance", "transmittance"):
        assert actual[key].dtype == np.float64
        assert actual[key].shape == (2101,)
        np.testing.assert_allclose(actual[key], case[key], atol=SUITE["tolerance"]["absolute"], rtol=SUITE["tolerance"]["relative"])
    assert np.all(actual["reflectance"] + actual["transmittance"] <= 1 + 2e-10)


@pytest.mark.parametrize("case", SUITE["cases"], ids=lambda c: c["name"])
def test_dedicated_interface_against_author_r(case):
    envelope = rsp.vegetation.prospect_d(**case["parameters"])
    assert_spectra(envelope["result"], case)
    assert envelope["manifest"]["parameters"] == case["parameters"]
    assert envelope["manifest"]["backend"] == "python"
    assert envelope["manifest"]["data_hashes"]["coefficients"] == "sha256:" + SUITE["coefficients_sha256"]


def test_numpy_batch_computes_all_cases_in_rust_with_order_and_shapes():
    names = rsp.vegetation.PROSPECT_D_PARAMETERS
    matrix = np.array([[case["parameters"][key] for key in names] for case in SUITE["cases"]])
    envelope = rsp.vegetation.prospect_d_batch(matrix)
    assert envelope["result"]["reflectance"].shape == (32, 2101)
    assert envelope["result"]["transmittance"].shape == (32, 2101)
    assert len(envelope["manifests"]) == 32
    for i, case in enumerate(SUITE["cases"]):
        assert_spectra({"wavelengths_nm": envelope["result"]["wavelengths_nm"],
                        "reflectance": envelope["result"]["reflectance"][i],
                        "transmittance": envelope["result"]["transmittance"][i]}, case)
        assert envelope["manifests"][i]["parameters"] == case["parameters"]
    # Non-contiguous inputs must preserve the exact row/column interpretation.
    backing = np.repeat(matrix, 2, axis=1)
    view = backing[:, ::2]
    assert not view.flags.c_contiguous
    np.testing.assert_array_equal(rsp.vegetation.prospect_d_batch(view)["result"]["reflectance"], envelope["result"]["reflectance"])


def test_generic_and_typed_interfaces_share_solver_defaults_and_provenance():
    typed = rsp.vegetation.prospect_d()
    generic = rsp.run(ID)
    assert_spectra(generic["result"], SUITE["cases"][0])
    assert typed["manifest"] == generic["manifest"]
    np.testing.assert_array_equal(typed["result"]["reflectance"], generic["result"]["reflectance"])
    rows = rsp.run_batch(ID, [{"n": 1.0}, {"n": 5.0}])
    assert_spectra(rows[0]["result"], SUITE["cases"][1])
    assert_spectra(rows[1]["result"], SUITE["cases"][2])
    descriptor = rsp.describe(ID)
    assert descriptor["metadata"]["scientific_version"] == "D"
    assert descriptor["metadata"]["status"] == "reference-validated"
    assert {e["backend"] for e in descriptor["metadata"]["validation"]} == {"native", "python"}
    assert descriptor["input_schema"]["properties"]["water"]["x-unit"] == "g cm^-2"
    assert {item["metadata"]["id"] for item in rsp.list_models()} == {ID, "ecosystem.fvcb", "ecosystem.fvcb-big-leaf", "thermal.foursail", "vegetation.prosail-d", "vegetation.goms", "surface.ross-li", "microwave.tau-omega", "microwave.water-cloud", "atmosphere.smac", "microwave.lmeb", "ecosystem.gort-gpp", "soil.sort", "soil.sort-l", "microwave.i2em", "vegetation.fluspect-cx"}


def test_invalid_inputs_and_failed_batch_row_are_explicit():
    for kwargs in ({"water": -0.1}, {"n": 0}, {"chlorophyll": float("nan")}, {"incidence_angle_deg": 0}):
        with pytest.raises(rsp.ModelError):
            rsp.vegetation.prospect_d(**kwargs)
    with pytest.raises(rsp.ModelError) as error:
        rsp.run(ID, {"chlorophyl": 40})
    assert error.value.kind == "invalid_parameter"
    with pytest.raises(rsp.ModelError) as error:
        rsp.run("test.unavailable")
    assert error.value.kind == "unknown_model"
    with pytest.raises(rsp.ModelError, match="row 1"):
        rsp.run_batch(ID, [{}, {"n": 0}])
    matrix = np.array([[case["parameters"][key] for key in rsp.vegetation.PROSPECT_D_PARAMETERS] for case in SUITE["cases"][:2]])
    matrix[1, 0] = 0
    with pytest.raises(rsp.ModelError, match="row 1"):
        rsp.vegetation.prospect_d_batch(matrix)
    for invalid in (np.zeros(8), np.zeros((2, 7)), np.zeros((1, 8), dtype=complex), [["x"] * 8]):
        with pytest.raises(rsp.ModelError):
            rsp.vegetation.prospect_d_batch(invalid)
    for invalid in (None, "not a table", {"n": 1.5}):
        with pytest.raises(rsp.ModelError):
            rsp.run_batch(ID, invalid)
    with pytest.raises(rsp.ModelError):
        rsp.vegetation.prospect_d_batch([[1, 2], [3]])


def test_empty_batches_and_numpy_scalar_inputs():
    assert rsp.run_batch(ID, []) == []
    empty = rsp.vegetation.prospect_d_batch(np.empty((0, 8)))
    assert empty["result"]["reflectance"].shape == (0, 2101)
    assert empty["manifests"] == []
    np.testing.assert_array_equal(rsp.vegetation.prospect_d(n=np.float64(1.5))["result"]["reflectance"], rsp.run(ID)["result"]["reflectance"])
