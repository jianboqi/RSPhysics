use rsphysics_ross_li::{Input, Model, Point, evaluate};
use serde::Deserialize;
#[derive(Deserialize)]
struct Suite {
    tolerance: Tol,
    original_tolerance: Tol,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Tol {
    absolute: f64,
    relative: f64,
}
#[derive(Deserialize)]
struct Case {
    name: String,
    parameters: Input,
    point: Expected,
    original_point: Legacy,
    principal_brf: Vec<f64>,
    hemisphere_brf: Vec<f64>,
    original_principal_brf: Vec<f64>,
    original_hemisphere_brf: Vec<f64>,
}
#[derive(Deserialize)]
struct Expected {
    brf: f64,
    brdf: f64,
    k_iso: f64,
    k_vol: f64,
    k_geo: f64,
}
#[derive(Deserialize)]
struct Legacy {
    brf: f64,
    k_vol: f64,
    k_geo: f64,
}
fn check(a: f64, e: f64, t: &Tol, name: &str) {
    assert!(
        (a - e).abs() <= t.absolute + t.relative * e.abs(),
        "{name}: actual={a:.17e}, reference={e:.17e}"
    );
}
#[test]
fn matches_32_original_c_and_independent_80_digit_formula_cases() {
    let suite: Suite = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/ross-li/expected.json"
    ))
    .unwrap();
    assert_eq!(suite.cases.len(), 32);
    let model = Model::new().unwrap();
    let mut max: f64 = 0.;
    for c in suite.cases {
        let actual = model.simulate(&c.parameters).unwrap();
        let p = actual.point;
        for (a, e) in [
            (p.brf, c.point.brf),
            (p.brdf, c.point.brdf),
            (p.k_iso, c.point.k_iso),
            (p.k_vol, c.point.k_vol),
            (p.k_geo, c.point.k_geo),
        ] {
            check(a, e, &suite.tolerance, &c.name);
            max = max.max((a - e).abs());
        }
        for (a, e) in [
            (p.brf, c.original_point.brf),
            (p.k_vol, c.original_point.k_vol),
            (p.k_geo, c.original_point.k_geo),
        ] {
            check(a, e, &suite.original_tolerance, &c.name);
        }
        for (a, precise, original) in [
            (
                &actual.principal_brf,
                c.principal_brf,
                c.original_principal_brf,
            ),
            (
                &actual.hemisphere_brf,
                c.hemisphere_brf,
                c.original_hemisphere_brf,
            ),
        ] {
            assert_eq!(a.len(), precise.len());
            for ((&a, e), o) in a.iter().zip(precise).zip(original) {
                check(a, e, &suite.tolerance, &c.name);
                check(a, o, &suite.original_tolerance, &c.name);
                max = max.max((a - e).abs());
            }
        }
        assert_eq!(
            actual.principal_view_zenith_deg,
            (-85..=85).map(f64::from).collect::<Vec<_>>()
        );
        for (i, (&z, &a)) in actual
            .hemisphere_view_zenith_deg
            .iter()
            .zip(&actual.hemisphere_azimuth_deg)
            .enumerate()
        {
            assert_eq!(z, (i / 36 * 5) as f64);
            assert_eq!(a, (i % 36 * 10) as f64);
        }
    }
    println!("Ross-Li max absolute error against 80-digit formula: {max:.9e}");
}
#[test]
fn reciprocity_isotropic_limit_and_nonphysical_diagnostics() {
    for s in [0., 10., 45., 85.] {
        for v in [0., 20., 60., 85.] {
            for a in [0., 60., 180.] {
                let p = Input {
                    solar_zenith_deg: s,
                    view_zenith_deg: v,
                    relative_azimuth_deg: a,
                    ..Input::default()
                };
                let q = Input {
                    solar_zenith_deg: v,
                    view_zenith_deg: s,
                    ..p
                };
                assert!((evaluate(&p).unwrap().brf - evaluate(&q).unwrap().brf).abs() < 2e-12);
            }
        }
    }
    let model = Model::new().unwrap();
    let out = model
        .simulate(&Input {
            f_vol: 0.,
            f_geo: 0.,
            ..Input::default()
        })
        .unwrap();
    assert!(out.hemisphere_brf.iter().all(|v| *v == 0.2));
    assert_eq!(out.negative_sample_count, 0);
    let zero: Point = evaluate(&Input {
        solar_zenith_deg: 0.,
        view_zenith_deg: 0.,
        ..Input::default()
    })
    .unwrap();
    assert_eq!(zero.k_vol, 0.);
    assert_eq!(zero.k_geo, 0.);
    let negative = model
        .simulate(&Input {
            f_iso: 0.,
            f_vol: 0.,
            f_geo: 1.,
            ..Input::default()
        })
        .unwrap();
    assert!(negative.point.brf < 0.);
    assert!(negative.negative_sample_count > 0);
    assert!(
        evaluate(&Input {
            f_iso: 1.,
            f_vol: 1.,
            f_geo: 0.,
            solar_zenith_deg: 85.,
            view_zenith_deg: 85.,
            ..Input::default()
        })
        .unwrap()
        .brf > 1.
    );
}
#[test]
fn invalid_inputs_are_rejected() {
    for p in [
        Input {
            view_zenith_deg: 90.,
            ..Input::default()
        },
        Input {
            f_geo: -1.,
            ..Input::default()
        },
        Input {
            f_iso: f64::NAN,
            ..Input::default()
        },
        Input {
            relative_azimuth_deg: 360.,
            ..Input::default()
        },
    ] {
        assert!(evaluate(&p).is_err());
    }
}
