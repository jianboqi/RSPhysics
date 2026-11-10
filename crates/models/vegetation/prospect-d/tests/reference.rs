use rsphysics_prospect_d::{Input, Model};
use serde::Deserialize;

#[derive(Deserialize)]
struct Suite {
    wavelengths_nm: Vec<f64>,
    tolerance: Tolerance,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Tolerance {
    absolute: f64,
    relative: f64,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    parameters: Input,
    reflectance: Vec<f64>,
    transmittance: Vec<f64>,
}

#[test]
fn full_spectra_match_32_author_r_reference_cases() {
    let suite: Suite = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/prospect-d/expected.json"
    ))
    .unwrap();
    let model = Model::new().unwrap();
    let mut max_error: f64 = 0.0;
    for case in suite.cases {
        let actual = model
            .simulate(&case.parameters)
            .unwrap_or_else(|e| panic!("{}: {e}", case.name));
        assert_eq!(actual.wavelengths_nm.wavelengths_nm(), suite.wavelengths_nm);
        for (quantity, actual, expected) in [
            ("reflectance", &actual.reflectance, case.reflectance),
            ("transmittance", &actual.transmittance, case.transmittance),
        ] {
            assert_eq!(actual.len(), expected.len());
            for (band, (&a, e)) in actual.iter().zip(expected).enumerate() {
                let error = (a - e).abs();
                max_error = max_error.max(error);
                assert!(
                    error <= suite.tolerance.absolute + suite.tolerance.relative * e.abs(),
                    "{} {quantity} band {band}: Rust={a:.17e}, R={e:.17e}, abs={error:.3e}",
                    case.name
                );
            }
        }
    }
    println!("maximum full-suite absolute error = {max_error:.6e}");
}

#[test]
fn direct_rust_inputs_reject_nonfinite_negative_and_outside_supported_envelope() {
    let model = Model::new().unwrap();
    for input in [
        Input {
            n: 0.0,
            ..Input::default()
        },
        Input {
            water: -0.01,
            ..Input::default()
        },
        Input {
            chlorophyll: f64::NAN,
            ..Input::default()
        },
        Input {
            incidence_angle_deg: 0.0,
            ..Input::default()
        },
        Input {
            n: f64::INFINITY,
            ..Input::default()
        },
    ] {
        assert!(model.simulate(&input).is_err());
    }
}
