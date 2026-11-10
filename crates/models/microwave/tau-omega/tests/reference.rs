use rsphysics_model_api::PhysicsModel;
use rsphysics_tau_omega::{Input, Model, evaluate};
use serde_json::Value;
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 2e-12 + 2e-12 * b.abs(), "{a} != {b}");
}
#[test]
fn matches_published_equations_and_reference_sweeps() {
    let suite: Value = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/tau-omega/expected.json"
    ))
    .unwrap();
    let model = Model::new().unwrap();
    assert_eq!(suite["cases"].as_array().unwrap().len(), 32);
    for case in suite["cases"].as_array().unwrap() {
        let p: Input = serde_json::from_value(case["parameters"].clone()).unwrap();
        let out = model.simulate(&p).unwrap();
        model.validate_output(&out).unwrap();
        let actual = serde_json::to_value(out).unwrap();
        for key in [
            "brightness_temperature_k",
            "soil_emission_k",
            "vegetation_emission_k",
            "reflected_vegetation_emission_k",
            "transmissivity",
        ] {
            close(
                actual[key].as_f64().unwrap(),
                case["expected"][key].as_f64().unwrap(),
            );
        }
        for key in ["sweep_zenith_deg", "sweep_total"] {
            for (a, b) in actual[key]
                .as_array()
                .unwrap()
                .iter()
                .zip(case["expected"][key].as_array().unwrap())
            {
                close(a.as_f64().unwrap(), b.as_f64().unwrap());
            }
        }
    }
}
#[test]
fn physical_limits_and_invalid_inputs() {
    let bare = evaluate(&Input {
        vegetation_optical_depth: 0.,
        ..Input::default()
    })
    .unwrap();
    close(bare.brightness_temperature_k, 210.);
    close(bare.vegetation_emission_k, 0.);
    let black = evaluate(&Input {
        soil_reflectivity: 0.,
        single_scattering_albedo: 0.,
        soil_temperature_k: 300.,
        vegetation_temperature_k: 300.,
        ..Input::default()
    })
    .unwrap();
    close(black.brightness_temperature_k, 300.);
    let weak = evaluate(&Input {
        vegetation_optical_depth: 1e-15,
        ..Input::default()
    })
    .unwrap();
    assert!(weak.vegetation_emission_k > 0.);
    for p in [
        Input {
            soil_temperature_k: f64::NAN,
            ..Input::default()
        },
        Input {
            soil_reflectivity: -0.01,
            ..Input::default()
        },
        Input {
            single_scattering_albedo: 1.1,
            ..Input::default()
        },
        Input {
            vegetation_optical_depth: f64::INFINITY,
            ..Input::default()
        },
        Input {
            observation_zenith_deg: 90.,
            ..Input::default()
        },
    ] {
        assert!(evaluate(&p).is_err());
    }
}
