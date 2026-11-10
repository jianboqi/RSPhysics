use rsphysics_model_api::PhysicsModel;
use rsphysics_sort::{Input, LambertInput, LambertModel, Model};
use serde_json::Value;

#[test]
fn directional_matches_supplied_python_full_spectra() {
    let model = Model::new().unwrap();
    let inputs: Value = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/sort/inputs.json"
    ))
    .unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/sort/expected.json"
    ))
    .unwrap();
    for (input, want) in inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(expected["cases"].as_array().unwrap())
    {
        let p: Input = serde_json::from_value(input["parameters"].clone()).unwrap();
        let output = model
            .simulate(&p)
            .unwrap_or_else(|e| panic!("{}: {e}", input["name"]));
        for (key, values) in [("dry_brf", output.dry_brf), ("wet_brf", output.wet_brf)] {
            // The acos phase at the exact hotspot amplifies cross-platform trig roundoff.
            for (i, (&actual, want)) in values.iter().zip(want[key].as_array().unwrap()).enumerate()
            {
                let want = want.as_f64().unwrap();
                assert!(
                    (actual - want).abs() <= 2e-9 + 2e-9 * want.abs(),
                    "{} {key} {i}: {actual} vs {want}",
                    input["name"]
                );
            }
        }
        assert_eq!(output.wavelengths_nm.wavelengths_nm().len(), 2001);
    }
}
#[test]
fn lambert_matches_supplied_python_full_spectra() {
    let model = LambertModel::new().unwrap();
    let inputs: Value = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/sort-l/inputs.json"
    ))
    .unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/sort-l/expected.json"
    ))
    .unwrap();
    for (input, want) in inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(expected["cases"].as_array().unwrap())
    {
        let p: LambertInput = serde_json::from_value(input["parameters"].clone()).unwrap();
        let output = model.simulate(&p).unwrap();
        for (key, values) in [
            ("dry_reflectance", output.dry_reflectance),
            ("wet_reflectance", output.wet_reflectance),
        ] {
            for (&actual, want) in values.iter().zip(want[key].as_array().unwrap()) {
                let want = want.as_f64().unwrap();
                assert!(
                    (actual - want).abs() <= 2e-11 + 2e-10 * want.abs(),
                    "{} {key}",
                    input["name"]
                );
            }
        }
    }
}
#[test]
fn dry_limits_and_physical_domains_are_enforced() {
    let model = Model::new().unwrap();
    for p in [
        Input {
            water_coverage: 0.0,
            ..Input::default()
        },
        Input {
            water_film_thickness_mm: 0.0,
            ..Input::default()
        },
    ] {
        let output = model.simulate(&p).unwrap();
        assert_eq!(output.dry_brf, output.wet_brf);
    }
    assert!(
        model
            .simulate(&Input {
                phase_asymmetry: 0.0,
                phase_backscatter: 0.5,
                ..Input::default()
            })
            .is_err()
    );
    assert!(
        model
            .simulate(&Input {
                solar_zenith_deg: f64::NAN,
                ..Input::default()
            })
            .is_err()
    );
    assert!(
        model
            .simulate(&Input {
                solid_filling_factor: 0.75,
                soil_brightness: 1.0,
                ..Input::default()
            })
            .is_err()
    );
    assert!(
        model
            .simulate(&Input {
                water_film_thickness_mm: -0.1,
                ..Input::default()
            })
            .is_err()
    );
    let mut output = model.simulate(&Input::default()).unwrap();
    output.wet_brf.pop();
    assert!(model.validate_output(&output).is_err());
}
