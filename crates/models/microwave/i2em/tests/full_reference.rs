use rsphysics_i2em::{Input, evaluate};
#[test]
fn additional_author_roughness_correlations() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/i2em");
    let inputs: Vec<Input> =
        serde_json::from_slice(&std::fs::read(root.join("full-inputs.json")).unwrap()).unwrap();
    let expected: Vec<[f64; 2]> =
        serde_json::from_slice(&std::fs::read(root.join("full-expected.json")).unwrap()).unwrap();
    for (i, (p, e)) in inputs.iter().zip(expected).enumerate() {
        let o = evaluate(p).unwrap();
        assert!(
            (o.sigma0_vv_db - e[0]).abs() < 2e-7,
            "{i} VV {} != {}",
            o.sigma0_vv_db,
            e[0]
        );
        assert!(
            (o.sigma0_hh_db - e[1]).abs() < 2e-7,
            "{i} HH {} != {}",
            o.sigma0_hh_db,
            e[1]
        );
    }
}
