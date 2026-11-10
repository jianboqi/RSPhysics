use rsphysics_fluspect_cx::{Input, Model, fluorescence_from_irradiance};
use serde_json::Value;

fn compare(a: &Value, b: &Value) {
    if let Some(v) = b.as_f64() {
        let got = a.as_f64().unwrap();
        assert!((got - v).abs() <= 2e-10 + 2e-9 * v.abs(), "{got} != {v}");
    } else if let Some(row) = b.as_array() {
        assert_eq!(a.as_array().unwrap().len(), row.len());
        for (i, v) in row.iter().enumerate() {
            compare(&a[i], v);
        }
    } else {
        for (k, v) in b.as_object().unwrap() {
            compare(&a[k], v);
        }
    }
}
#[test]
fn full_and_pro_reference_matrices() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../benchmarks/reference/fluspect-cx");
    let inputs: Vec<Input> =
        serde_json::from_slice(&std::fs::read(root.join("full-inputs.json")).unwrap()).unwrap();
    let compressed = std::fs::read(root.join("full-expected.json.zlib")).unwrap();
    let expected: Value =
        serde_json::from_slice(&miniz_oxide::inflate::decompress_to_vec_zlib(&compressed).unwrap())
            .unwrap();
    let model = Model::new().unwrap();
    for (i, p) in inputs.iter().enumerate() {
        compare(
            &serde_json::to_value(model.simulate(p).unwrap()).unwrap(),
            &expected[i],
        );
    }
}
#[test]
fn irradiance_conversion_uses_photon_wavelength_ratio() {
    let o = Model::new().unwrap().simulate(&Input::default()).unwrap();
    assert_eq!(o.fluorescence_backward.len(), 211);
    assert_eq!(o.fluorescence_backward[0].len(), 351);
    let mut incident = vec![0.; 351];
    incident[50] = 2.;
    let flux = fluorescence_from_irradiance(&o, &incident).unwrap();
    for (f, w) in o.emission_wavelengths_nm.iter().enumerate() {
        assert!(
            (flux.backward_w_m2_nm[f] - o.fluorescence_backward[f][50] * 2. * 450. / w).abs()
                < 1e-15
        );
    }
    assert!(o.fluorescence_backward[1].iter().all(|v| *v == 0.));
    assert!(fluorescence_from_irradiance(&o, &[1.]).is_err());
}
