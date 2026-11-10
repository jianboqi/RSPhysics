use rsphysics_smac::{Direction, Input, Model, evaluate};
use serde::Deserialize;
#[derive(Deserialize)]
struct Case {
    name: String,
    input: Input,
    expected: serde_json::Value,
}
#[derive(Deserialize)]
struct Suite {
    cases: Vec<Case>,
    curves: Vec<serde_json::Value>,
}
fn suite() -> Suite {
    serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/smac/expected.json"
    ))
    .unwrap()
}
fn close(a: f64, b: f64, name: &str) {
    assert!(
        (a - b).abs() <= 2e-11 + 2e-10 * b.abs(),
        "{name}: {a} != {b}"
    );
}
#[test]
fn original_reference_all_coefficients_and_domains() {
    let data = suite();
    assert_eq!(data.cases.len(), 839);
    for case in data.cases {
        let actual = serde_json::to_value(
            evaluate(&case.input).unwrap_or_else(|e| panic!("{}: {e}", case.name)),
        )
        .unwrap();
        for (key, value) in case.expected.as_object().unwrap() {
            close(
                actual[key].as_f64().unwrap(),
                value.as_f64().unwrap(),
                &format!("{}/{key}", case.name),
            );
        }
    }
}
#[test]
fn original_reference_complete_sensitivity_curves() {
    let model = Model::new().unwrap();
    for case in suite().curves {
        let input: Input = serde_json::from_value(case["input"].clone()).unwrap();
        let out = model.simulate(&input).unwrap();
        for (a, b) in out
            .sweep_reflectance
            .iter()
            .zip(case["sweep_reflectance"].as_array().unwrap())
        {
            close(*a, b.as_f64().unwrap(), "AOT sweep");
        }
        assert_eq!(out.sweep_aot550.len(), 21);
    }
}
#[test]
fn forward_inverse_round_trip_and_unclipped_results() {
    let mut p = Input {
        direction: Direction::SurfaceToToa,
        ..Input::default()
    };
    for r in [0., 0.01, 0.2, 0.7, 1.] {
        p.reflectance = r;
        let toa = evaluate(&p).unwrap().reflectance;
        let back = evaluate(&Input {
            direction: Direction::ToaToSurface,
            reflectance: toa,
            ..p.clone()
        })
        .unwrap()
        .reflectance;
        close(back, r, "round trip");
    }
    let result = evaluate(&Input {
        reflectance: 0.,
        ..Input::default()
    })
    .unwrap();
    assert!(result.reflectance < 0.);
    assert_eq!(result.outside_unit_interval, 1);
}
#[test]
fn resource_round_trip_identity_and_input_errors() {
    let lib = rsphysics_smac::coefficients::library().unwrap();
    assert_eq!(lib.records.len(), 367);
    assert_eq!(lib.sources.len(), 331);
    for c in &lib.records {
        assert_eq!(c.values.len(), 49);
        assert!(lib.sources.contains_key(&c.source_key));
    }
    for p in [
        Input {
            coefficient_id: "landsat-9.oli-2.B4.continental".into(),
            ..Input::default()
        },
        Input {
            aot550: 1.1,
            ..Input::default()
        },
        Input {
            pressure_hpa: f64::NAN,
            ..Input::default()
        },
        Input {
            solar_zenith_deg: 71.,
            ..Input::default()
        },
        Input {
            direction: Direction::SurfaceToToa,
            reflectance: 1.1,
            ..Input::default()
        },
    ] {
        assert!(evaluate(&p).is_err());
    }
}
