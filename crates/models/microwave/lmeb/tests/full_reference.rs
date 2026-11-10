use rsphysics_lmeb::{Input, Model};
use serde_json::Value;
#[test]
fn all_dielectric_and_surface_branches() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/lmeb");
    let cases: Vec<Input> =
        serde_json::from_slice(&std::fs::read(root.join("full-inputs.json")).unwrap()).unwrap();
    let expected: Value =
        serde_json::from_slice(&std::fs::read(root.join("full-expected.json")).unwrap()).unwrap();
    let model = Model::new().unwrap();
    for (i, p) in cases.iter().enumerate() {
        let got = serde_json::to_value(model.simulate(p).unwrap()).unwrap();
        for (k, w) in expected[i].as_object().unwrap() {
            let close = |a: f64, b: f64| {
                assert!(
                    (a - b).abs() <= 2e-9 + 2e-9 * b.abs(),
                    "case {i}/{k}: {a} vs {b}"
                )
            };
            if let Some(v) = w.as_f64() {
                close(got[k].as_f64().unwrap(), v);
            } else {
                for (j, x) in w.as_array().unwrap().iter().enumerate() {
                    close(got[k][j].as_f64().unwrap(), x.as_f64().unwrap());
                }
            }
        }
    }
}
