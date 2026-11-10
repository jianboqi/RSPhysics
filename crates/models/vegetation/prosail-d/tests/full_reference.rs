use rsphysics_prosail_d::{Input, Model};
use serde_json::Value;
#[test]
fn all_nine_author_outputs_and_both_leaf_distributions() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/prosail-d/full-expected.json.zlib");
    let suite: Value = serde_json::from_slice(
        &miniz_oxide::inflate::decompress_to_vec_zlib(&std::fs::read(file).unwrap()).unwrap(),
    )
    .unwrap();
    let model = Model::new().unwrap();
    for (i, c) in suite["cases"].as_array().unwrap().iter().enumerate() {
        let input: Input = serde_json::from_value(c["parameters"].clone()).unwrap();
        let output = serde_json::to_value(model.simulate(&input).unwrap()).unwrap();
        for (k, v) in c["result"].as_object().unwrap() {
            for (j, x) in v.as_array().unwrap().iter().enumerate() {
                let want = x.as_f64().unwrap();
                let got = output[k][j].as_f64().unwrap();
                assert!(
                    (got - want).abs() < 2e-10 + 2e-8 * want.abs(),
                    "case {i}/{k}/{j}: {got} vs {want}"
                );
            }
        }
    }
}
#[test]
fn conservative_canopy_and_bare_soil_limits() {
    let model = Model::new().unwrap();
    let p = Input {
        chlorophyll: 0.,
        carotenoids: 0.,
        anthocyanins: 0.,
        brown_pigments: 0.,
        water: 0.,
        dry_matter: 0.,
        ..Input::default()
    };
    let o = model.simulate(&p).unwrap();
    assert!(
        o.direct_absorptance
            .iter()
            .chain(&o.diffuse_absorptance)
            .all(|v| v.abs() < 2e-12)
    );
    assert!(o.brf.iter().all(|v| v.is_finite() && *v >= 0.));
    let bare = model
        .simulate(&Input {
            leaf_area_index: 0.,
            ..p
        })
        .unwrap();
    assert_eq!(bare.brf, bare.hemispherical_directional_reflectance);
    assert!(bare.vegetation_cover.iter().all(|v| *v == 0.));
}
