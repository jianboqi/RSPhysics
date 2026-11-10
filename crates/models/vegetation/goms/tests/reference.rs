use rsphysics_goms::{GeometryConvention, Input, Model, evaluate};
use serde_json::Value;
fn compare(a: &Value, b: &Value, path: &str) {
    if let Some(v) = b.as_f64() {
        let got = a.as_f64().unwrap();
        assert!(
            (got - v).abs() <= 2e-10 + 2e-9 * v.abs(),
            "{path}: {got} != {v}"
        );
    } else if let Some(row) = b.as_array() {
        assert_eq!(a.as_array().unwrap().len(), row.len(), "{path}");
        for (i, v) in row.iter().enumerate() {
            compare(&a[i], v, &format!("{path}/{i}"));
        }
    } else {
        for (k, v) in b.as_object().unwrap() {
            compare(&a[k], v, &format!("{path}/{k}"));
        }
    }
}
#[test]
fn complete_c_reference() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/goms");
    let inputs: Vec<Input> =
        serde_json::from_slice(&std::fs::read(root.join("inputs.json")).unwrap()).unwrap();
    let compressed = std::fs::read(root.join("expected.json.zlib")).unwrap();
    let expected: Value =
        serde_json::from_slice(&miniz_oxide::inflate::decompress_to_vec_zlib(&compressed).unwrap())
            .unwrap();
    let model = Model::new().unwrap();
    for (i, p) in inputs.iter().enumerate() {
        compare(
            &serde_json::to_value(model.simulate(p).unwrap()).unwrap(),
            &expected[i],
            &format!("case {i}"),
        );
    }
}
#[test]
fn nadir_ground_probability_and_azimuth_rotation() {
    let p = Input {
        solar_zenith_deg: 0.,
        view_zenith_deg: 0.,
        ..Input::default()
    };
    assert!(
        (evaluate(&p).unwrap().sunlit_ground_weight - (-0.1 * std::f64::consts::PI).exp()).abs()
            < 1e-15
    );
    let legacy = Input {
        geometry_convention: GeometryConvention::LegacyC,
        ..p
    };
    assert!((evaluate(&legacy).unwrap().sunlit_ground_weight - 0.5895955307385816).abs() < 1e-14);
    let a = Input {
        solar_azimuth_deg: 350.,
        view_azimuth_deg: 10.,
        ..Input::default()
    };
    let b = Input {
        solar_azimuth_deg: 10.,
        view_azimuth_deg: 30.,
        ..a.clone()
    };
    assert!((evaluate(&a).unwrap().brf[0] - evaluate(&b).unwrap().brf[0]).abs() < 1e-14);
}

#[test]
fn hotspot_is_sun_observer_alignment_not_the_global_curve_maximum() {
    for slope in [0., 20.] {
        let p = Input {
            slope_deg: slope,
            solar_zenith_deg: 45.,
            solar_azimuth_deg: 0.,
            view_zenith_deg: 45.,
            view_azimuth_deg: 0.,
            ..Input::default()
        };
        let aligned = evaluate(&p).unwrap();
        let opposite = evaluate(&Input {
            view_azimuth_deg: 180.,
            ..p.clone()
        })
        .unwrap();
        assert!(aligned.shaded_weight.abs() < 1e-12);
        assert!(aligned.brf[0] > opposite.brf[0]);
        let o = Model::new().unwrap().simulate(&p).unwrap();
        assert_eq!(o.hotspot_view_zenith_deg, 45.);
        assert_eq!(o.hotspot_view_azimuth_deg, 0.);
        assert!((o.principal_brf[0][130] - aligned.brf[0]).abs() < 1e-14);
        assert!(o.principal_shaded_weight[130].abs() < 1e-12);
        assert!(o.principal_shaded_weight[40] > 0.);
    }
}
#[test]
fn multiband_validation_and_terrain_mask() {
    let p = Input {
        sunlit_ground_reflectance: vec![0.2, 0.3],
        ..Input::default()
    };
    assert!(evaluate(&p).is_err());
    let p = Input {
        solar_zenith_deg: 85.,
        solar_azimuth_deg: 180.,
        slope_deg: 60.,
        ..Input::default()
    };
    assert!(evaluate(&p).is_err());
    let p = Input {
        slope_deg: 20.,
        ..Input::default()
    };
    let o = Model::new().unwrap().simulate(&p).unwrap();
    assert!(o.hemisphere_valid.contains(&0.));
    for (i, &valid) in o.hemisphere_valid.iter().enumerate() {
        if valid == 0. {
            assert_eq!(o.hemisphere_brf[0][i], 0.);
        }
    }
}
