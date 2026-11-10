use rsphysics_prosail_d::{Input, Model};
use serde::Deserialize;
#[derive(Deserialize)]
struct Suite {
    wavelengths_nm: Vec<f64>,
    tolerance: Tolerance,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Tolerance {
    absolute: f64,
    relative: f64,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    expected_source: String,
    parameters: Input,
    brf: Vec<f64>,
    hemispherical_reflectance: Vec<f64>,
}
#[test]
fn full_spectra_match_24_author_r_cases_and_separate_bare_soil_limit() {
    let suite: Suite = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/prosail-d/expected.json"
    ))
    .unwrap();
    assert_eq!(suite.cases.len(), 25);
    let model = Model::new().unwrap();
    let mut max: f64 = 0.;
    let mut original = 0;
    for c in suite.cases {
        if c.expected_source == "author_r_fourSAIL" {
            original += 1;
        } else {
            assert_eq!(c.parameters.leaf_area_index, 0.);
        }
        let a = model
            .simulate(&c.parameters)
            .unwrap_or_else(|e| panic!("{}: {e}", c.name));
        assert_eq!(a.wavelengths_nm.wavelengths_nm(), suite.wavelengths_nm);
        for (key, actual, expected) in [
            ("brf", a.brf, c.brf),
            (
                "hemi",
                a.hemispherical_reflectance,
                c.hemispherical_reflectance,
            ),
        ] {
            assert_eq!(actual.len(), expected.len());
            for (i, (a, e)) in actual.into_iter().zip(expected).enumerate() {
                let error = (a - e).abs();
                max = max.max(error);
                assert!(
                    error <= suite.tolerance.absolute + suite.tolerance.relative * e.abs(),
                    "{} {key} {i}: {a:.17e} vs {e:.17e}, error {error}",
                    c.name
                );
            }
        }
    }
    assert_eq!(original, 24);
    println!("PROSAIL maximum absolute error: {max:.9e}");
}
#[test]
fn invalid_domains_are_classified() {
    let model = Model::new().unwrap();
    for input in [
        Input {
            leaf_area_index: -1.,
            ..Input::default()
        },
        Input {
            view_zenith_deg: 90.,
            ..Input::default()
        },
        Input {
            hotspot: f64::NAN,
            ..Input::default()
        },
        Input {
            water: 0.,
            dry_matter: 0.,
            ..Input::default()
        },
    ] {
        assert!(model.simulate(&input).is_err());
    }
}
