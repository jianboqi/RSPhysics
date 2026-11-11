"""Installed direct-library API against original-Fortran reference reports."""

from pathlib import Path
from datetime import timedelta
from io import BytesIO
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


@pytest.mark.parametrize("program", ["standard", "lutaero"])
def test_extended_inputs_against_fortran(program):
    # These values were generated with the corresponding original executables.
    window = sixs.Spectrum.band(42)
    target = sixs.Surface.sampled_reflectance(
        [0.4, 0.5, 0.6, 0.7, 0.8], [0.13, 0.18, 0.28, 0.35, 0.4], window
    )
    cases = [
        (dict(geometry=sixs.Geometry.user(max_scattering_order=1)), 0.1872633),
        (dict(geometry=sixs.Geometry.user(max_scattering_order=7)), 0.2045928),
        (
            dict(spectrum=window, surface=sixs.Surface.heterogeneous(target, 0.1)),
            0.2729267,
        ),
        (
            dict(
                aerosol=sixs.Aerosol.layered(
                    [[0.8, 0.2, 4], [1.2, 0.1, 4]], components=[0.5, 0.3, 0.1, 0.1]
                )
            ),
            0.1808047,
        ),
    ]
    assert sixs.prepare(cases[0][0])["prepared"]["input_cards"].splitlines()[0] == "-11"
    assert sixs.prepare(cases[1][0])["prepared"]["input_cards"].splitlines()[0] == "-70"
    for configuration, reference in cases:
        assert sixs.run(configuration, program=program)["result"][
            "apparent_reflectance"
        ] == pytest.approx(reference, abs=2e-6)
    with pytest.raises(rsp.ModelError, match="max_scattering_order"):
        sixs.prepare({"geometry": {"mode": 2, "max_scattering_order": 1}})


def test_structured_diagnostics_and_products_from_original_reports():
    cases = {case["name"]: case for case in SUITE["cases"]}
    original = sixs.parse_report(cases["default"]["report"])
    diagnostics = original["diagnostics"]
    assert diagnostics["direct_solar_irradiance_w_m2_um"] == 1077.845
    assert diagnostics["diffuse_irradiance_fraction"] == 0.205
    assert diagnostics["atmospheric_intrinsic_radiance_w_m2_sr_um"] == 23.724
    assert diagnostics["pixel_reflectance"] == 0.126
    assert diagnostics["apparent_polarized_reflectance"] == 0.0058
    assert diagnostics["plane_of_polarization_deg"] == 31.14
    for name in ("step", "default-lut", "mie-8"):
        case = cases[name]
        original = sixs.parse_report(case["report"], files=case["files"])
        actual = sixs.run(case["parameters"])["result"]
        if name == "step":
            assert (
                len(actual["spectral_columns"])
                == actual["spectral_rows"].shape[1]
                == 11
            )
            np.testing.assert_allclose(
                actual["spectral_rows"], original["spectral_rows"], atol=0.11
            )
        elif name == "default-lut":
            lut = actual["aerosol_lut"]
            assert lut["rows"][0]["view_zenith_deg"] == 20
            assert lut["rows"][0]["scattering_angles_deg"][-1] == pytest.approx(
                130, abs=1e-4
            )
            assert lut["rows"][0]["intrinsic_reflectance"] == pytest.approx(
                original["aerosol_lut"]["rows"][0]["intrinsic_reflectance"], abs=1e-5
            )
        else:
            table = actual["mie_tables"]["aerosol.mie"]
            assert (
                table["phase_i"].shape
                == table["phase_q"].shape
                == table["phase_u"].shape
                == (83, 20)
            )
            np.testing.assert_allclose(
                table["phase_i"],
                original["mie_tables"]["aerosol.mie"]["phase_i"],
                atol=1e-3,
                rtol=1e-4,
            )


def test_scan_and_spectral_interpolation_helpers():
    response = sixs.Spectrum.from_samples([0.5, 0.503, 0.505], [0.0, 1.0, 0.0])
    assert response["response"] == pytest.approx([0.0, 5 / 6, 0.0])
    result = sixs.sensor_bands("modis", band_ids=[43, 42])
    assert result["band_ids"] == [43, 42]
    assert result["apparent_reflectance"] == pytest.approx(
        [0.1942419, 0.1928082], abs=2e-6
    )
    result = sixs.principal_plane([-20, 0, 20], {"geometry": {"solar_azimuth_deg": 30}})
    assert result["view_azimuth_deg"].tolist() == [210, 30, 30]
    assert sixs.azimuth_scan(20, step_deg=120)["view_azimuth_deg"].tolist() == [
        0,
        120,
        240,
    ]
    with pytest.raises(rsp.ModelError, match="requires structured"):
        sixs.angles([0, 20], 90, {"geometry": {"mode": 1}})


def test_observation_helpers(tmp_path):
    geo = sixs.Geometry.from_time_and_location(40, -105, "2026-07-01T12:00:00-07:00")
    assert 16 < geo["solar_zenith_deg"] < 18
    assert 175 < geo["solar_azimuth_deg"] < 180
    assert geo == sixs.Geometry.from_time_and_location(40, -105, "2026-07-01T19:00:00Z")
    assert sixs.Atmosphere.from_latitude_and_date(45, "2026-07-01")["profile"] == 2
    assert sixs.Atmosphere.from_latitude_and_date(-45, "2026-07-01")["profile"] == 3
    csv = tmp_path / "site.lev20"
    csv.write_text(
        "AERONET Version 3\nDate(dd:mm:yyyy),Time(hh:mm:ss),AOD_500nm,AOD_675nm\n01:07:2026,19:10:00,0.22,0.162962962962963\n"
    )
    assert sixs.Aerosol.from_aeronet(csv, "2026-07-01T19:00:00Z")[
        "aot550"
    ] == pytest.approx(0.2)
    with pytest.raises(rsp.ModelError, match="No valid AERONET"):
        sixs.Aerosol.from_aeronet(
            csv, "2026-07-01T19:00:00Z", max_age=timedelta(minutes=1)
        )
    sounding = tmp_path / "sounding.txt"
    sounding.write_text(
        "".join(f"{name:>7}" for name in ("PRES", "HGHT", "TEMP", "MIXR"))
        + "\n"
        + "".join(
            "".join(f"{v:7.2f}" for v in row) + "\n"
            for row in [[900, 1000, 14, 5], [800, 2000, 7, 4], [700, 3000, 0, 3]]
        )
    )
    levels = np.array(sixs.Atmosphere.from_wyoming(sounding)["levels"])
    assert levels.shape == (34, 5)
    assert np.all(np.diff(levels[:, 1]) <= 0)
    assert levels[2, 1:3] == pytest.approx([800, 280.15])


def test_named_diagnostics_from_reference_reports():
    cases = {case["name"]: case["report"] for case in SUITE["cases"]}
    ground = sixs.parse_report(cases["ground"])["diagnostics"]
    assert ground["ground_pressure_hpa"] == 1013
    assert ground["ground_altitude_km"] == 0
    assert ground["visibility_km"] == 26.74
    assert ground["aot550"] == 0.2
    assert ground["solar_spectral_irradiance_w_m2_um"] == 1810.209
    band = sixs.parse_report(cases["uniform"])["diagnostics"]
    assert band["integrated_filter_response_um"] == 0.1
    assert band["integrated_solar_irradiance_w_m2"] == 179.415
    assert band["solar_spectral_irradiance_w_m2_um"] is None
    ocean = sixs.parse_report(cases["brdf-6"])["diagnostics"]
    assert ocean["ocean_foam_reflectance"] == 0.00019
    assert ocean["ocean_water_reflectance"] == 0.00605
    assert ocean["ocean_glint_reflectance"] == 0.00751
    assert ocean["reflectance_water_vapour_mixed_with_aerosol"] == 0.063


def test_named_band_and_sampled_surface_in_multi_band_run():
    assert len(sixs.Band) == 198
    assert sixs.Spectrum.band("LANDSAT_TM_B1") == sixs.Spectrum.band(
        sixs.Band.LANDSAT_TM_B1
    )
    x, y = [0.4, 0.55, 0.7], [0.11, 0.25, 0.34]
    curve = sixs.Surface.sampled_curve(x, y)
    sample = sixs.sensor_bands(
        "tm",
        {"surface": sixs.Surface.heterogeneous(curve, 0.1)},
        band_ids=["LANDSAT_TM_B2", "LANDSAT_TM_B1"],
        output="diagnostics.pixel_radiance_w_m2_sr_um",
    )
    assert sample["band_ids"] == [26, 25]
    assert sample["centre_wavelengths_um"] == pytest.approx([0.575, 0.495])
    for i, band in enumerate(sample["band_ids"]):
        window = sixs.Spectrum.band(band)
        target = sixs.Surface.sampled_reflectance(x, y, window)
        direct = sixs.run(
            spectrum=window, surface=sixs.Surface.heterogeneous(target, 0.1)
        )["result"]
        assert sample["apparent_reflectance"][i] == pytest.approx(
            direct["apparent_reflectance"], abs=1e-6
        )
        assert sample["values"][i] == pytest.approx(
            direct["diagnostics"]["pixel_radiance_w_m2_sr_um"], abs=0.002
        )
        assert sixs.run(spectrum=window, surface=sixs.Surface.from_samples(x, y))[
            "result"
        ]["apparent_reflectance"] == pytest.approx(
            sixs.run(spectrum=window, surface={"mode": "lambertian", "target": target})[
                "result"
            ]["apparent_reflectance"],
            abs=1e-6,
        )
    with pytest.raises(rsp.ModelError, match="Surface reflectance sampling"):
        sixs.prepare(
            {
                "spectrum": sixs.Spectrum.band(42),
                "surface": sixs.Surface.from_samples([0.63, 0.7], [0.2, 0.25]),
            }
        )


def test_arbitrary_output_and_angular_grid():
    field = "quantities.global_gas_trans.2"
    scan = sixs.angular_grid([10, 25], [0, 90, 180], direction="solar", output=field)
    assert scan["values"].shape == scan["apparent_reflectance"].shape == (2, 3)
    assert (
        scan["runs"][-1]["manifest"]["parameters"]["geometry"]["solar_zenith_deg"] == 25
    )
    assert (
        scan["runs"][-1]["manifest"]["parameters"]["geometry"]["solar_azimuth_deg"]
        == 180
    )
    np.testing.assert_equal(
        scan["values"].ravel(), sixs.extract_output(scan["runs"], field)
    )
    vn = sixs.vnir(spacing_um=0.5, output="diagnostics.ocean_foam_reflectance")
    assert vn["wavelengths_um"] == pytest.approx([0.4, 0.9, 1.4])
    assert np.isnan(vn["values"]).all()
    assert sixs.full_spectrum(spacing_um=1.25)["wavelengths_um"] == pytest.approx(
        [0.25, 1.5, 2.75, 4]
    )
    with pytest.raises(rsp.ModelError, match="non-numeric"):
        sixs.extract_output(scan["runs"], "report")


@pytest.mark.parametrize("version", [2, 3])
def test_aeronet_inversion_prefers_nearest_complete_observation(tmp_path, version):
    file = tmp_path / "sample.all"
    fields = (
        "AOD_Coincident_Input[440nm],AOD_Coincident_Input[675nm],"
        "Refractive_Index-Real_Part[440nm],Refractive_Index-Real_Part[675nm],"
        "Refractive_Index-Imaginary_Part[440nm],Refractive_Index-Imaginary_Part[675nm],"
        if version == 3
        else "AOT_440,AOT_675,REFR(440),REFR(675),REFI(440),REFI(675),"
    )
    file.write_text(
        f"AERONET Version {version}\n"
        f"Date(dd:mm:yyyy),Time(hh:mm:ss),{fields}"
        "0.050000,0.100000,0.200000,0.400000\n"
        "01:07:2026,19:00:00,0.25,0.162962963,1.46,1.53,-999,0.009,0.001,0.006,0.004,0.001\n"
        "01:07:2026,19:10:00,0.25,0.162962963,1.46,1.53,0.012,0.009,0.001,0.006,0.004,0.001\n"
    )
    aerosol = sixs.Aerosol.from_aeronet_inversion(file, "2026-07-01T19:00:00Z")
    assert aerosol["model"] == 11
    assert aerosol["aot550"] == pytest.approx(0.2)
    assert (
        aerosol["refractive_real"][0] == 1.46 and aerosol["refractive_real"][-1] == 1.53
    )
    assert aerosol["refractive_imag"][0] == 0.012
    assert np.isfinite(sixs.run(aerosol=aerosol)["result"]["apparent_reflectance"])


def test_aster_and_sounding_adapters(tmp_path, monkeypatch):
    aster = tmp_path / "leaf.txt"
    aster.write_text(
        "X Units: Wavelength (micrometers)\nY Units: Reflectance (percent)\n\n0.70 40\n0.50 20\n"
    )
    surface = sixs.Surface.from_aster(aster)
    assert surface["target"]["wavelengths_um"] == [0.5, 0.7]
    assert surface["target"]["values"] == [0.2, 0.4]
    assert sixs.run(surface=surface)["result"]["apparent_reflectance"] > 0
    raw = "".join(f"{name:>7}" for name in ("PRES", "HGHT", "TEMP", "MIXR")) + "\n"
    raw += "".join(
        "".join(f"{v:7.2f}" for v in row) + "\n"
        for row in [[900, 1000, 14, 5], [800, 2000, 7, 4], [700, 3000, 0, 3]]
    )
    monkeypatch.setattr(
        "rsphysics._sixs_observations.urlopen",
        lambda url, timeout: BytesIO(("<pre>\n" + raw + "</pre>").encode()),
    )
    wyoming = sixs.Atmosphere.from_wyoming_url(
        "https://weather.uwyo.edu/wsgi/sounding?type=TEXT%3ALIST"
    )
    assert wyoming["levels"][2][1:3] == pytest.approx([800, 280.15])
    bas = tmp_path / "bas.txt"
    bas.write_text(
        "header\n0 0 900 1000 14 0 4\n0 0 800 2000 7 0 0\n0 0 700 3000 0 0 -5\n"
    )
    profile = sixs.Atmosphere.from_bas(bas)
    assert profile["levels"][2][1:3] == pytest.approx([800, 280.15])
    assert profile["levels"][2][3] > 0
    csv = (
        "time,pressure_hPa,geopotential height_m,temperature_C,dew point temperature_C\n"
        "2026-07-01 12:00:00,900,1000,14,4\n"
        "2026-07-01 12:00:00,800,2000,7,0\n"
        "2026-07-01 12:00:00,700,3000,0,-5\n"
    )
    monkeypatch.setattr(
        "rsphysics._sixs_observations.urlopen",
        lambda url, timeout: BytesIO(csv.encode()),
    )
    assert (
        sixs.Atmosphere.from_wyoming_url(
            "https://weather.uwyo.edu/wsgi/sounding?type=TEXT%3ACSV"
        )
        == profile
    )


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
