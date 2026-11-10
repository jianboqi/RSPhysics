use rsphysics_lmeb::{Input, Model};
use serde_json::Value;
#[test]
fn original_matlab_reference() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/lmeb");
    let inputs: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("inputs.json")).unwrap()).unwrap();
    let expected: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    let model = Model::new().unwrap();
    for (case, want) in inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(expected["cases"].as_array().unwrap())
    {
        let input: Input = serde_json::from_value(case["parameters"].clone()).unwrap();
        let got = serde_json::to_value(model.simulate(&input).unwrap()).unwrap();
        for (key, value) in want.as_object().unwrap() {
            if key == "name" {
                continue;
            }
            let close = |a: f64, b: f64| {
                assert!(
                    (a - b).abs() <= 2e-9 + 2e-9 * b.abs(),
                    "{} {key}: {a} != {b}",
                    case["name"]
                )
            };
            if let Some(b) = value.as_f64() {
                close(got[key].as_f64().unwrap(), b)
            } else {
                for (a, b) in got[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(value.as_array().unwrap())
                {
                    close(a.as_f64().unwrap(), b.as_f64().unwrap());
                }
            }
        }
    }
}
#[test]
fn physical_domain_checks() {
    let model = Model::new().unwrap();
    let input = Input {
        soil_moisture: f64::NAN,
        ..Input::default()
    };
    assert!(model.simulate(&input).is_err());
}
