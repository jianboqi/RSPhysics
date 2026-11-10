use rsphysics_i2em::{Input, evaluate};
use serde_json::Value;
#[test]
fn original_matlab_copol() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/i2em");
    let inputs: Vec<Input> =
        serde_json::from_str(&std::fs::read_to_string(root.join("inputs.json")).unwrap()).unwrap();
    let expected: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    assert_eq!(inputs.len(), 20);
    for (i, p) in inputs.iter().enumerate() {
        let o = evaluate(p).unwrap();
        for (j, v) in [o.sigma0_vv_db, o.sigma0_hh_db].iter().enumerate() {
            let want = expected[i][j].as_f64().unwrap();
            assert!(
                (v - want).abs() < 2e-9,
                "case {i}, channel {j}: {v} != {want}"
            );
        }
        assert!((10. * o.sigma0_vv.log10() - o.sigma0_vv_db).abs() < 1e-12);
        assert!((10. * o.sigma0_hh.log10() - o.sigma0_hh_db).abs() < 1e-12);
    }
}
#[test]
fn domain_and_smooth_limit() {
    let p = Input::default();
    for bad in [
        Input {
            frequency_ghz: f64::NAN,
            ..p
        },
        Input {
            rms_height_m: 0.,
            ..p
        },
        Input {
            permittivity_loss: -1.,
            ..p
        },
        Input {
            observation_zenith_deg: 0.,
            ..p
        },
        Input {
            rms_height_m: 0.3,
            ..p
        },
        Input {
            correlation_length_m: 0.01,
            ..p
        },
        Input {
            rms_height_m: 0.30 / (2. * std::f64::consts::PI * p.frequency_ghz),
            ..p
        },
    ] {
        assert!(evaluate(&bad).is_err());
    }
    let a = evaluate(&Input {
        rms_height_m: 0.00001,
        ..p
    })
    .unwrap();
    let b = evaluate(&Input {
        rms_height_m: 0.00002,
        ..p
    })
    .unwrap();
    assert!((b.sigma0_vv / a.sigma0_vv - 4.).abs() < 0.002);
    assert!((b.sigma0_hh / a.sigma0_hh - 4.).abs() < 0.002);
}
