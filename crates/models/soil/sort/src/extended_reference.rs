use super::*;
use serde_json::Value;
fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../benchmarks/reference/sort")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(name)).unwrap()).unwrap()
}
#[test]
fn shiltz_single_and_multifacet_source_quadrature() {
    for c in read("roughness-expected.json").as_array().unwrap() {
        let p = Input {
            roughness_correction: serde_json::from_value(c["mode"].clone()).unwrap(),
            rms_slope: c["rms_slope"].as_f64().unwrap(),
            multifacet_reflectance: 0.3,
            ..Input::default()
        };
        let a: Vec<f64> = c["geometry_deg"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap().to_radians())
            .collect();
        for (j, w) in c["ssa"].as_array().unwrap().iter().enumerate() {
            let got = shiltz::at_order(&p, w.as_f64().unwrap(), a[0], a[1], a[2], 200);
            let want = c["expected"][j].as_f64().unwrap();
            assert!(
                (got - want).abs() < 2e-10 + 2e-9 * want.abs(),
                "{} {a:?}: {got} != {want}",
                c["mode"]
            );
        }
    }
}
#[test]
fn angular_film_adding_matches_source_fourier_transport() {
    for c in read("layer-expected.json").as_array().unwrap() {
        let p = Input {
            roughness_correction: serde_json::from_value(c["mode"].clone()).unwrap(),
            rms_slope: 0.2,
            multifacet_reflectance: 0.2,
            solid_filling_factor: 0.05,
            phase_asymmetry: 0.2,
            phase_backscatter: 0.,
            hotspot_amplitude: 0.1,
            hotspot_width: 0.2,
            ..Input::default()
        };
        let got = layer::adding(
            &p,
            c["ssa"].as_f64().unwrap(),
            c["index"].as_f64().unwrap(),
            c["tau"].as_f64().unwrap(),
            25_f64.to_radians(),
            15_f64.to_radians(),
            60_f64.to_radians(),
        )
        .unwrap();
        let want = c["expected"].as_f64().unwrap();
        assert!(
            (got - want).abs() < 2e-10 + 2e-9 * want.abs(),
            "{got} != {want}"
        );
    }
}
#[test]
fn external_substrate_and_directional_single_pass_spectra() {
    let bytes = std::fs::read(root().join("extended-expected.json.zlib")).unwrap();
    let cases: Value =
        serde_json::from_slice(&miniz_oxide::inflate::decompress_to_vec_zlib(&bytes).unwrap())
            .unwrap();
    let model = Model::new().unwrap();
    for c in cases.as_array().unwrap() {
        let p = Input {
            solar_zenith_deg: 45.,
            view_zenith_deg: 25.,
            relative_azimuth_deg: 60.,
            phase_asymmetry: 0.2,
            phase_backscatter: 0.,
            solid_filling_factor: 0.05,
            hotspot_amplitude: 0.1,
            hotspot_width: 0.2,
            rms_slope: 0.2,
            multifacet_reflectance: 0.2,
            soil_brightness: 0.3,
            water_film_mode: serde_json::from_value(c["parameters"]["water_film_mode"].clone())
                .unwrap(),
            substrate_angle_mode: serde_json::from_value(
                c["parameters"]["substrate_angle_mode"].clone(),
            )
            .unwrap(),
            ..Input::default()
        };
        let o = serde_json::to_value(model.simulate(&p).unwrap()).unwrap();
        for key in ["dry_brf", "wet_brf"] {
            for (j, w) in c[key].as_array().unwrap().iter().enumerate() {
                let want = w.as_f64().unwrap();
                let got = o[key][j].as_f64().unwrap();
                assert!(
                    (got - want).abs() < 2e-9 + 2e-9 * want.abs(),
                    "{key}/{j}: {got} != {want}"
                );
            }
        }
    }
}
