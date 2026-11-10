use rsphysics_foursail_tir::{Input, Model, evaluate};
use rsphysics_model_api::{ErrorKind, PhysicsModel};
use rsphysics_numerics::thermal::{WavelengthDensity, planck_radiance};
use serde_json::Value;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/foursail-tir/expected.json"
    ))
    .unwrap()
}
#[test]
fn independent_matrix_exponential_oracle_and_complete_default_responses() {
    let suite = fixture();
    for c in suite["cases"].as_array().unwrap() {
        let p: Input = serde_json::from_value(c["parameters"].clone()).unwrap();
        let got = serde_json::to_value(evaluate(&p).unwrap()).unwrap();
        for (key, want) in c["result"].as_object().unwrap() {
            let a = got[key].as_f64().unwrap();
            let b = want.as_f64().unwrap();
            assert!(
                (a - b).abs() <= 2e-9 + 2e-10 * b.abs(),
                "{} {key}: {a} vs {b}",
                c["name"]
            );
        }
    }
    let o =
        serde_json::to_value(Model::new().unwrap().simulate(&Input::default()).unwrap()).unwrap();
    for (key, want) in suite["default_arrays"].as_object().unwrap() {
        let a = o[key].as_array().unwrap();
        let b = want.as_array().unwrap();
        assert_eq!(a.len(), b.len());
        for (a, b) in a.iter().zip(b) {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            assert!((a - b).abs() <= 2e-9 + 2e-10 * b.abs(), "{key}: {a} vs {b}");
        }
    }
}
#[test]
fn isothermal_equilibrium_and_kirchhoff_identity() {
    for lai in [0., 0.01, 3., 10.] {
        for leaf in [0., 0.5, 0.98, 1.] {
            for soil in [0., 0.95, 1.] {
                let p = Input {
                    leaf_area_index: lai,
                    leaf_emissivity: leaf,
                    soil_emissivity: soil,
                    sunlit_leaf_temperature_k: 300.,
                    shaded_leaf_temperature_k: 300.,
                    sunlit_soil_temperature_k: 300.,
                    shaded_soil_temperature_k: 300.,
                    sky_brightness_temperature_k: 300.,
                    ..Default::default()
                };
                let o = evaluate(&p).unwrap();
                let b = planck_radiance(p.wavelength_nm, 300., WavelengthDensity::PerMicrometre)
                    .unwrap();
                assert!((o.brightness_temperature_k - 300.).abs() < 1e-9);
                assert!((o.radiance_w_m2_sr_um - b).abs() < 1e-10);
                assert!(
                    (o.leaf_emission_weight + o.soil_emission_weight + o.sky_reflectance - 1.)
                        .abs()
                        < 1e-12
                );
                for f in [
                    o.upwelling_flux_w_m2_um,
                    o.ground_downwelling_irradiance_w_m2_um,
                    o.ground_upwelling_flux_w_m2_um,
                ] {
                    assert!((f - std::f64::consts::PI * b).abs() < 1e-9);
                }
            }
        }
    }
    let p = Input {
        sunlit_leaf_temperature_k: 300.,
        shaded_leaf_temperature_k: 300.,
        sunlit_soil_temperature_k: 300.,
        shaded_soil_temperature_k: 300.,
        sky_brightness_temperature_k: 300.,
        ..Default::default()
    };
    let o = Model::new().unwrap().simulate(&p).unwrap();
    assert!(
        o.principal_brightness_temperature_k
            .iter()
            .chain(&o.hemisphere_brightness_temperature_k)
            .chain(&o.spectral_brightness_temperature_k)
            .all(|t| (t - 300.).abs() < 1e-9)
    );
}
#[test]
fn bare_soil_and_emissivity_independence_from_solar_direction() {
    let dark = evaluate(&Input {
        leaf_emissivity: 0.,
        soil_emissivity: 0.,
        sky_brightness_temperature_k: 0.,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(dark.radiance_w_m2_sr_um, 0.);
    assert_eq!(dark.brightness_temperature_k, 0.);
    assert_eq!(dark.ground_upwelling_flux_w_m2_um, 0.);
    let p = Input {
        leaf_area_index: 0.,
        direct_irradiance_w_m2_um: 10.,
        ..Default::default()
    };
    let o = evaluate(&p).unwrap();
    let b = |t| planck_radiance(p.wavelength_nm, t, WavelengthDensity::PerMicrometre).unwrap();
    let expected = p.soil_emissivity * b(p.sunlit_soil_temperature_k)
        + (1. - p.soil_emissivity)
            * (b(p.sky_brightness_temperature_k)
                + p.direct_irradiance_w_m2_um / std::f64::consts::PI);
    assert!((o.radiance_w_m2_sr_um - expected).abs() < 1e-12);
    assert_eq!(o.leaf_emission_weight, 0.);
    assert_eq!(o.shaded_soil_weight, 0.);
    let p = Input::default();
    let a = evaluate(&p).unwrap();
    let b = evaluate(&Input {
        solar_zenith_deg: 80.,
        relative_azimuth_deg: 180.,
        hotspot: 0.,
        ..p
    })
    .unwrap();
    assert!((a.effective_emissivity - b.effective_emissivity).abs() < 1e-12);
}
#[test]
fn rejects_invalid_physical_domains() {
    for p in [
        Input {
            leaf_emissivity: 1.1,
            ..Default::default()
        },
        Input {
            wavelength_nm: f64::NAN,
            ..Default::default()
        },
        Input {
            sunlit_soil_temperature_k: 0.,
            ..Default::default()
        },
    ] {
        assert_eq!(evaluate(&p).unwrap_err().kind, ErrorKind::OutOfDomain);
    }
    let m = Model::new().unwrap();
    assert!(m.validate_input(&Input::default()).is_ok());
}
