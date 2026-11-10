use rsphysics_fluspect_cx::{Input, Model};
use serde_json::Value;
#[test]
fn original_matlab_optics_and_full_matrices() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/fluspect-cx");
    let inputs: Vec<Input> =
        serde_json::from_slice(&std::fs::read(root.join("inputs.json")).unwrap()).unwrap();
    let compressed = std::fs::read(root.join("expected.json.zlib")).unwrap();
    let expected: Value =
        serde_json::from_slice(&miniz_oxide::inflate::decompress_to_vec_zlib(&compressed).unwrap())
            .unwrap();
    let model = Model::new().unwrap();
    fn compare(got: &Value, want: &Value, path: &str) {
        if let Some(w) = want.as_f64() {
            let g = got.as_f64().unwrap();
            let absolute = if path.starts_with("case 10/") {
                2e-8
            } else {
                2e-10
            };
            assert!(
                (g - w).abs() <= absolute + 2e-9 * w.abs(),
                "{path}: {g} != {w}"
            );
        } else if let Some(rows) = want.as_array() {
            assert_eq!(got.as_array().unwrap().len(), rows.len(), "{path}");
            for (i, w) in rows.iter().enumerate() {
                compare(&got[i], w, &format!("{path}/{i}"));
            }
        } else {
            for (k, w) in want.as_object().unwrap() {
                compare(&got[k], w, &format!("{path}/{k}"));
            }
        }
    }
    for (i, p) in inputs.iter().enumerate() {
        let out = model.simulate(p).unwrap();
        compare(
            &serde_json::to_value(out).unwrap(),
            &expected[i],
            &format!("case {i}"),
        );
    }
}
#[test]
fn zero_and_linear_efficiency_and_domain() {
    let m = Model::new().unwrap();
    let p = Input::default();
    let a = m.simulate(&p).unwrap();
    let b = m
        .simulate(&Input {
            fluorescence_efficiency: 2. * p.fluorescence_efficiency,
            ..p
        })
        .unwrap();
    assert_eq!(a.reflectance, b.reflectance);
    assert_eq!(a.transmittance, b.transmittance);
    for (x, y) in a
        .fluorescence_backward
        .iter()
        .flatten()
        .zip(b.fluorescence_backward.iter().flatten())
    {
        assert!((2. * x - y).abs() < 1e-14);
    }
    let z = m
        .simulate(&Input {
            fluorescence_efficiency: 0.,
            ..p
        })
        .unwrap();
    assert!(z.fluorescence_backward.iter().flatten().all(|v| *v == 0.));
    assert_eq!(a.reflectance, z.reflectance);
    for bad in [
        Input { n: 0.9, ..p },
        Input {
            xanthophyll_state: 1.1,
            ..p
        },
        Input {
            chlorophyll: f64::NAN,
            ..p
        },
        Input {
            fluorescence_efficiency: -1.,
            ..p
        },
    ] {
        assert!(m.simulate(&bad).is_err());
    }
}
