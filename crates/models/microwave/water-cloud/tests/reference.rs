use rsphysics_model_api::PhysicsModel;
use rsphysics_water_cloud::{Input, Model, evaluate};
use serde_json::Value;
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 2e-12 + 2e-12 * b.abs(), "{a} != {b}");
}
#[test]
fn matches_published_equations_and_reference_sweeps() {
    let suite: Value = serde_json::from_str(include_str!(
        "../../../../../benchmarks/reference/water-cloud/expected.json"
    ))
    .unwrap();
    let model = Model::new().unwrap();
    assert_eq!(suite["cases"].as_array().unwrap().len(), 32);
    for case in suite["cases"].as_array().unwrap() {
        let p: Input = serde_json::from_value(case["parameters"].clone()).unwrap();
        let out = model.simulate(&p).unwrap();
        model.validate_output(&out).unwrap();
        let actual = serde_json::to_value(out).unwrap();
        for key in [
            "sigma0",
            "vegetation_backscatter",
            "attenuated_soil_backscatter",
            "two_way_transmissivity",
        ] {
            close(
                actual[key].as_f64().unwrap(),
                case["expected"][key].as_f64().unwrap(),
            );
        }
        for key in ["sweep_zenith_deg", "sweep_total"] {
            for (a, b) in actual[key]
                .as_array()
                .unwrap()
                .iter()
                .zip(case["expected"][key].as_array().unwrap())
            {
                close(a.as_f64().unwrap(), b.as_f64().unwrap());
            }
        }
        for angle in 0..=80 {
            let p = Input {
                observation_zenith_deg: f64::from(angle),
                ..p
            };
            let v = evaluate(&p).unwrap();
            close(
                v.vegetation_backscatter,
                case["sense_canopy_sweep"]["vegetation_backscatter"][angle as usize]
                    .as_f64()
                    .unwrap(),
            );
            close(
                v.two_way_transmissivity,
                case["sense_canopy_sweep"]["two_way_transmissivity"][angle as usize]
                    .as_f64()
                    .unwrap(),
            );
        }
    }
}
#[test]
fn physical_limits_and_invalid_inputs() {
    let bare = evaluate(&Input {
        vegetation_water_content: 0.,
        ..Input::default()
    })
    .unwrap();
    close(bare.sigma0, 0.1);
    close(bare.vegetation_backscatter, 0.);
    close(bare.two_way_transmissivity, 1.);
    let zero = evaluate(&Input {
        scattering_coefficient: 0.,
        soil_backscatter: 0.,
        ..Input::default()
    })
    .unwrap();
    close(zero.sigma0, 0.);
    let large = evaluate(&Input {
        scattering_coefficient: 2.,
        vegetation_water_content: 10.,
        ..Input::default()
    })
    .unwrap();
    assert!(large.sigma0 > 1.);
    let weak = evaluate(&Input {
        attenuation_coefficient: 1e-15,
        ..Input::default()
    })
    .unwrap();
    assert!(weak.vegetation_backscatter > 0.);
    for p in [
        Input {
            soil_backscatter: -10.,
            ..Input::default()
        },
        Input {
            vegetation_water_content: f64::NAN,
            ..Input::default()
        },
        Input {
            attenuation_coefficient: f64::INFINITY,
            ..Input::default()
        },
        Input {
            observation_zenith_deg: 90.,
            ..Input::default()
        },
    ] {
        assert!(evaluate(&p).is_err());
    }
}

#[test]
fn independent_scattering_and_attenuation_descriptors() {
    use rsphysics_water_cloud::DescriptorMode;
    let input = Input {
        descriptor_mode: DescriptorMode::Independent,
        scattering_descriptor: 3.,
        attenuation_descriptor: 0.5,
        ..Input::default()
    };
    let o = evaluate(&input).unwrap();
    let c = 40_f64.to_radians().cos();
    let t = (-2. * 0.2 * 0.5 / c).exp();
    close(o.two_way_transmissivity, t);
    close(o.vegetation_backscatter, 0.1 * 3. * c * (1. - t));
    let no_attenuation = evaluate(&Input {
        attenuation_descriptor: 0.,
        ..input
    })
    .unwrap();
    close(no_attenuation.sigma0, 0.1);
}
