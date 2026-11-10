"""Installed direct-library API against original-Fortran reference reports."""

from pathlib import Path
import copy, json, zlib
import numpy as np
import pytest
import rsphysics as rsp
from rsphysics import sixs

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "benchmarks/reference/6sv2.1/full-reference.json.zlib"
SUITE = json.loads(zlib.decompress(FIXTURE.read_bytes()))


@pytest.mark.parametrize("case", SUITE["cases"], ids=lambda c: c["name"])
def test_direct_library_matches_original_fortran(case):
    assert (
        sixs.prepare(case["parameters"])["prepared"]["input_cards"]
        == case["input_cards"]
    )
    envelope = sixs.run(case["parameters"])
    actual = envelope["result"]
    expected = case["expected"]
    assert envelope["manifest"]["model_id"] == sixs.MODEL_ID
    assert envelope["manifest"]["precision"] == "mixed_f32_f64"
    assert (
        envelope["manifest"]["data_hashes"]["original_distribution"]
        == "sha256:" + SUITE["source_archive_sha256"]
    )
    assert actual["apparent_reflectance"] == pytest.approx(
        expected["apparent_reflectance"], abs=2e-6, rel=3e-6
    )
    assert actual["apparent_radiance_w_m2_sr_um"] == pytest.approx(
        expected["apparent_radiance_w_m2_sr_um"], abs=0.0011, rel=3e-6
    )
    assert set(actual["quantities"]) == set(expected["quantities"])
    for key, values in expected["quantities"].items():
        assert len(actual["quantities"][key]) == len(values)
        for x, y in zip(actual["quantities"][key], values):
            if y is None:
                assert x is None or not np.isfinite(x)
            else:
                assert x == pytest.approx(y, abs=1.01e-5, rel=3e-6), (
                    case["name"],
                    key,
                    x,
                    y,
                )
    assert set(actual["files"]) == set(case["files"])
    # Original auxiliary products are retained, including complete Mie/LUT data.
    for name, text in actual["files"].items():
        assert text and len(text.splitlines()) == len(case["files"][name].splitlines())


def test_repeat_state_reset_and_recovery_without_executable(monkeypatch):
    monkeypatch.setenv("PATH", "")
    first = sixs.run()["result"]
    sixs.run(
        aerosol=sixs.Aerosol.predefined(3),
        atmosphere=sixs.Atmosphere.water_ozone(3, 0.4),
    )
    sixs.run(surface=sixs.Surface.modis(0.3, 0.1, 0.05))
    with pytest.raises(rsp.ModelError):
        sixs.run_cards("0\n")
    assert sixs.run()["result"]["report"] == first["report"]


def test_numpy_and_configuration_helpers():
    assert len(sixs.bands()) == 198
    assert [x["id"] for x in sixs.bands()] == list(range(2, 200))
    levels = sixs.standard_levels()
    assert levels.shape == (34, 5)
    assert levels[-1, 0] == 99999.0
    config = sixs.SixS(
        atmosphere=sixs.Atmosphere.radiosonde(levels), spectrum=sixs.Spectrum.band(43)
    )
    assert config.prepare()["prepared"]["program"] == "sixsV2.1"
    assert config.run()["result"]["apparent_reflectance"] > 0
    spectrum = sixs.spectrum(np.array([0.5, 0.55, 0.6]))
    assert spectrum["apparent_reflectance"].shape == (3,)
    assert np.isfinite(spectrum["apparent_reflectance"]).all()


def test_measured_constant_brdf_reduces_to_lambertian():
    constant = sixs.Surface.measured(
        [[0.2] * 10] * 13, [[0.2] * 10] * 13, albedo=0.2, directional_reflectance=0.2
    )
    a = sixs.run(surface=constant)["result"]
    b = sixs.run(surface=sixs.Surface.lambertian(0.2))["result"]
    assert a["apparent_reflectance"] == pytest.approx(
        b["apparent_reflectance"], abs=2e-6
    )


def test_full_raw_input_and_parser_preserve_results():
    for program in ["standard", "lutaero"]:
        config = {"program": program, "spectrum": sixs.Spectrum.band(43)}
        cards = sixs.prepare(config)["prepared"]["input_cards"]
        a = sixs.run(config)
        b = sixs.run_cards(cards, program=program)
        assert a["result"]["report"] == b["result"]["report"]
        parsed = sixs.parse_report(a["result"]["report"], files=a["result"]["files"])
        assert parsed["apparent_reflectance"] == a["result"]["apparent_reflectance"]


@pytest.mark.parametrize(
    "configuration",
    [
        {"geometry": {"solar_zenith_deg": 90}},
        {"atmosphere": {"profile": 9}},
        {"atmosphere": {"profile": 7, "levels": []}},
        {"aerosol": {"model": 20}},
        {"aerosol": {"model": 4, "components": [1.0, 1.0, 0.0, 0.0]}},
        {"aerosol": {"model": 8, "modes": []}},
        {"aerosol": {"model": 8, "modes": [{"refractive_real": [1.5]}]}},
        {"spectrum": {"mode": "band", "band": 200}},
        {"spectrum": {"wavelength_um": 0.1}},
        {"spectrum": {"mode": "response", "response": [1.0]}},
        {"surface": {"mode": "brdf", "brdf_model": 2, "parameters": [1.0]}},
        {"aerosol": {"model": 12, "mie_file": "missing.mie"}},
        {"files": {"../file.mie": "test"}},
        {"program": "unknown"},
        {"unexpected": 1},
    ],
)
def test_invalid_structured_configuration_is_classified(configuration):
    with pytest.raises(rsp.ModelError):
        sixs.run(configuration)


def test_ordered_batch_and_row_errors():
    rows = [{}, {"aerosol": {"model": 0}}]
    runs = sixs.run_batch(rows)
    assert [r["result"]["apparent_reflectance"] for r in runs] == [
        sixs.run(row)["result"]["apparent_reflectance"] for row in rows
    ]
    with pytest.raises(rsp.ModelError, match="row 1"):
        sixs.run_batch([{}, {"geometry": {"solar_zenith_deg": 95}}])


def test_parallel_direct_library_batch_and_runtime_error():
    rows = [{}, {"aerosol": {"model": 0}}]
    actual = sixs.run_batch(rows, workers=2)
    assert [r["result"]["apparent_reflectance"] for r in actual] == [
        sixs.run(row)["result"]["apparent_reflectance"] for row in rows
    ]
    with pytest.raises(rsp.ModelError, match="row 1"):
        sixs.run_batch([{}, {"input_cards": "0\n"}], workers=2)


def test_original_stop_returns_an_error_and_kernel_recovers():
    with pytest.raises(rsp.ModelError):
        sixs.run(surface=sixs.Surface.brdf(7, [9, 1, 3, 0.1, 0.2, 0.1, 0.2]))
    assert sixs.run()["result"]["apparent_reflectance"] == pytest.approx(0.2045928)


@pytest.mark.parametrize(
    "filename", ["input.cards", "report.txt", "error.txt", "a b.mie"]
)
def test_reserved_auxiliary_filenames_rejected(filename):
    with pytest.raises(rsp.ModelError):
        sixs.prepare({"files": {filename: "contents"}})
