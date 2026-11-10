use rsphysics_fvcb::{BigLeafInput, BigLeafModel, Input, Model, evaluate_big_leaf, evaluate_leaf};
use rsphysics_model_api::{ModelAdapter, TypedAdapter};
use serde_json::{Value, json};

#[test]
fn independent_equations() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/fvcb/reference.json"
    ))
    .unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let big = case["model_id"] == "ecosystem.fvcb-big-leaf";
        let result = if big {
            let p: BigLeafInput = serde_json::from_value(case["parameters"].clone()).unwrap();
            serde_json::to_value(BigLeafModel::new().unwrap().simulate(&p).unwrap()).unwrap()
        } else {
            let p: Input = serde_json::from_value(case["parameters"].clone()).unwrap();
            serde_json::to_value(Model::new().unwrap().simulate(&p).unwrap()).unwrap()
        };
        for (key, expected) in case["expected"].as_object().unwrap() {
            if let Some(value) = expected.as_f64() {
                let actual = result[key].as_f64().unwrap();
                assert!(
                    (actual - value).abs() <= 2e-8 * value.abs().max(1.),
                    "{} {key}: {actual} != {value}",
                    case["name"]
                );
            } else {
                assert_eq!(&result[key], expected, "{} {key}", case["name"]);
            }
        }
        if let Some(curves) = case.get("curves") {
            for (key, expected) in curves.as_object().unwrap() {
                for (actual, value) in result[key]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(expected.as_array().unwrap())
                {
                    let (actual, value) = (actual.as_f64().unwrap(), value.as_f64().unwrap());
                    assert!(
                        (actual - value).abs() <= 2e-8 * value.abs().max(1.),
                        "{} {key}",
                        case["name"]
                    );
                }
            }
        }
    }
}

#[test]
fn schemas_and_unknown_fields() {
    let leaf = TypedAdapter::new(Model::new().unwrap()).unwrap();
    assert!(leaf.run_json(&json!({})).is_ok());
    assert!(leaf.run_json(&json!({"wrong_input":1.})).is_err());
    assert!(
        leaf.run_json(&json!({"gas_exchange_mode":"prescribed_ci"}))
            .is_ok()
    );
    let canopy = TypedAdapter::new(BigLeafModel::new().unwrap()).unwrap();
    assert!(canopy.run_json(&json!({})).is_ok());
    assert!(
        canopy
            .run_json(&json!({"absorbed_par_umol_m2_s":1.}))
            .is_err()
    );
}

#[test]
fn leaf_and_uniform_unit_area_canopy_agree() {
    let leaf = evaluate_leaf(&Input::default()).unwrap();
    let canopy = evaluate_big_leaf(&BigLeafInput {
        leaf_area_index: 1.,
        nitrogen_extinction_coefficient: 0.,
        ..Default::default()
    })
    .unwrap();
    assert!((leaf.net_assimilation_umol_m2_s - canopy.net_assimilation_umol_m2_s).abs() < 1e-12);
}
