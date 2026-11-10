use rsphysics_bsm::{Input, Model};
use rsphysics_model_api::{ModelAdapter, TypedAdapter};
use serde_json::{Value, json};
fn check(text: &str, tol: f64) {
    let f: Value = serde_json::from_str(text).unwrap();
    let m = Model::new().unwrap();
    for c in f["cases"].as_array().unwrap() {
        let input: Input = serde_json::from_value(c["parameters"].clone()).unwrap();
        let o = serde_json::to_value(m.simulate(&input).unwrap()).unwrap();
        for (k, e) in c["expected"].as_object().unwrap() {
            let tolerance = if k.contains("hemispherical") {
                2e-5
            } else {
                tol
            };
            if let Some(v) = e.as_f64() {
                assert!(
                    (o[k].as_f64().unwrap() - v).abs() <= tolerance * v.abs().max(1.),
                    "{} {k}",
                    c["name"]
                );
            } else {
                let values = e.as_array().unwrap();
                let got = o[k].as_array().unwrap();
                assert_eq!(values.len(), got.len());
                for (a, b) in got.iter().zip(values) {
                    assert!(
                        (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= tolerance,
                        "{} {k}",
                        c["name"]
                    );
                }
            }
        }
        for k in ["principal_brf", "hemisphere_brf"] {
            if let Some(a) = c.get(k) {
                let a = a.as_array().unwrap();
                let b = o[k].as_array().unwrap();
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 2e-11);
                }
            }
        }
    }
}
#[test]
fn reference_spectra_and_angles() {
    check(
        include_str!("../../../../../benchmarks/reference/bsm/reference.json"),
        2e-10,
    );
    check(
        include_str!("../../../../../benchmarks/reference/bsm/complete.json"),
        2e-10,
    );
}
#[test]
fn schema_and_unknown_fields() {
    let m = TypedAdapter::new(Model::new().unwrap()).unwrap();
    assert!(m.run_json(&json!({})).is_ok());
    assert!(m.run_json(&json!({"unused_parameter":1})).is_err());
}
