"""Representative original 6SV2.1 options, including every built-in band."""

import json
from pathlib import Path


def cases():
    root = Path(__file__).resolve().parents[3]
    resources = json.loads(
        (root / "crates/models/atmosphere/sixs/data/resources.json").read_text()
    )
    rows = [{"name": "default", "parameters": {}}]

    def add(name, **parameters):
        rows.append(dict(name=name, parameters=parameters))

    for profile in range(7):
        add(f"atmosphere-{profile}", atmosphere={"profile": profile})
    add(
        "radiosonde",
        atmosphere={"profile": 7, "levels": resources["profiles"]["6"]["levels"]},
    )
    add(
        "water-ozone",
        atmosphere={"profile": 8, "water_vapour_g_cm2": 2.0, "ozone_atm_cm": 0.3},
    )
    for model in [0, 1, 2, 3, 5, 6, 7]:
        add(f"aerosol-{model}", aerosol={"model": model})
    add("mixture", aerosol={"model": 4, "components": [0.5, 0.3, 0.1, 0.1]})
    add("visibility", aerosol={"visibility_km": 40.0})
    add(
        "layered",
        aerosol={"model": -1, "layers": [[2, 0.17, 1], [8, 0.03, 1], [90, 0, 1]]},
    )
    for model, extra in [
        (8, {}),
        (9, {"gamma": [2.0, 10.0, 1.0]}),
        (10, {"junge_exponent": 3.5}),
        (11, {"sunphotometer": [[0.05, 0.1], [0.1, 0.2], [0.2, 0.1], [0.5, 0.01]]}),
    ]:
        add(
            f"mie-{model}",
            aerosol={
                "model": model,
                "radius_min_um": 0.05,
                "radius_max_um": 0.5,
                **extra,
            },
        )
    for name, altitudes in [
        ("ground", {"sensor_height_km": 0}),
        ("aircraft-auto", {"sensor_height_km": 3}),
        (
            "aircraft-columns",
            {
                "sensor_height_km": 3,
                "path_water_vapour_g_cm2": 1.0,
                "path_ozone_atm_cm": 0.1,
                "path_aot550": 0.1,
            },
        ),
        ("target-pressure", {"target_pressure_hpa": 850}),
        ("target-altitude", {"target_altitude_km": 2}),
    ]:
        add(name, altitudes=altitudes)
    for mode, extra in [
        (1, {"column": 2501, "line": 1251, "utc_hour": 12.0}),
        (2, {"column": 6499, "line": 8666, "utc_hour": 17.0}),
        (3, {"column": 6499, "line": 8666, "utc_hour": 21.0}),
        (
            4,
            {
                "column": 1024,
                "utc_hour": 12.0,
                "ascending_node_longitude_deg": 0.0,
                "ascending_node_hour": 12.0,
            },
        ),
        (
            5,
            {
                "column": 1024,
                "utc_hour": 12.0,
                "ascending_node_longitude_deg": 0.0,
                "ascending_node_hour": 12.0,
            },
        ),
        (6, {"longitude_deg": 0.0, "latitude_deg": 30.0, "utc_hour": 12.0}),
        (7, {"longitude_deg": 0.0, "latitude_deg": 30.0, "utc_hour": 12.0}),
    ]:
        add(f"geometry-{mode}", geometry={"mode": mode, **extra})
    add("uniform", spectrum={"mode": "uniform", "lower_um": 0.5, "upper_um": 0.6})
    add("step", spectrum={"mode": "step", "lower_um": 0.5, "upper_um": 0.51})
    add(
        "response",
        spectrum={
            "mode": "response",
            "lower_um": 0.5,
            "upper_um": 0.6,
            "response": [1.0] * 41,
        },
    )
    for band in resources["bands"]:
        add(
            f'band-{band["id"]}',
            spectrum={"mode": "band", "band": band["id"]},
            atmosphere={"profile": 0},
            aerosol={"model": 0},
        )
    for selector in range(1, 5):
        add(f"surface-{selector}", surface={"target": {"spectrum": selector}})
    add(
        "surface-spectrum",
        surface={
            "target": {
                "spectrum": -1,
                "lower_um": 0.5,
                "upper_um": 0.6,
                "values": [0.2] * 41,
            }
        },
    )
    add(
        "heterogeneous",
        surface={
            "mode": "heterogeneous",
            "target": {"value": 0.2},
            "background": {"value": 0.4},
            "radius_km": 1.0,
        },
    )
    brdfs = {
        1: [0.5, -0.2, 0.1, 0.1],
        2: [1, 1, 1, 2, 0.1, 0, 0, 0.6, -0.2, 0],
        3: [0.2, 0.01, 0.01],
        4: [0.001, 0.001, 0.01, 0.2],
        5: [0.2, 0.8],
        6: [5, 0, 34.3, 0.1],
        7: [5, 1, 3, 0.1, 0.2, 0.1, 0.2],
        8: [0.2, -0.2, 0.7],
        9: [3, 0.5, 45, 0.1, 30, 0.015, 1.2, 1, 0.3],
        10: [0.3091, 0.129, 0.0762],
        11: [0.3091, 0.129, 0.0762],
    }
    for model, parameters in brdfs.items():
        add(
            f"brdf-{model}",
            surface={"mode": "brdf", "brdf_model": model, "parameters": parameters},
        )
    add(
        "brdf-measured-constant",
        surface={
            "mode": "brdf",
            "brdf_model": 0,
            "sun_table": [[0.2] * 10 for _ in range(13)],
            "view_table": [[0.2] * 10 for _ in range(13)],
            "albedo": 0.2,
            "directional_reflectance": 0.2,
        },
    )
    for mode, value in [("reflectance", 0.2), ("radiance", 100.0)]:
        add("correction-" + mode, correction={"mode": mode, "value": value})
    for mode, extra in [
        ("stokes", {"q": 0.001, "u": 0.002}),
        ("vegetation", {"vegetation_fraction": 0.5}),
        ("ocean", {"wind_speed_m_s": 5.0, "wind_azimuth_deg": 0.0}),
    ]:
        add("polarization-" + mode, polarization={"mode": mode, **extra})
    # Every configuration family also passes through the LUT main, while the
    # exhaustive built-in-band sweep is evaluated once by the standard main.
    for case in list(rows):
        if not case["name"].startswith("band-"):
            rows.append(
                dict(
                    name=case["name"] + "-lut",
                    parameters={**case["parameters"], "program": "lutaero"},
                )
            )
    return rows
