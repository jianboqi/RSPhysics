use rsphysics_core::SpectralAxis;
use rsphysics_inform::{Input, LeafSpectrum, Model, Prospect301Coefficients, SpectralInputs};
use std::{fs, path::PathBuf};

const CASES: &str = include_str!("../fixtures/parameters.csv");
const SPECTRA: [&str; 6] = [
    include_str!("../fixtures/case1.csv"),
    include_str!("../fixtures/case2.csv"),
    include_str!("../fixtures/case3.csv"),
    include_str!("../fixtures/case4.csv"),
    include_str!("../fixtures/case5.csv"),
    include_str!("../fixtures/case6.csv"),
];

fn read_csv(csv: &str) -> Vec<Vec<f64>> {
    let rows: Vec<Vec<f64>> = csv
        .lines()
        .map(|row| row.split(',').map(|x| x.parse().unwrap()).collect())
        .collect();
    (0..rows[0].len())
        .map(|i| rows.iter().map(|row| row[i]).collect())
        .collect()
}

fn compare(
    model: &Model,
    parameters: &[f64],
    reference: &str,
    coefficients: Option<&Prospect301Coefficients>,
) {
    let column = read_csv(reference);
    let input = Input {
        soil_brightness: parameters[0],
        crown_lai: parameters[1],
        understory_lai: parameters[2],
        stem_density_per_ha: parameters[3],
        tree_height_m: parameters[4],
        crown_diameter_m: parameters[5],
        crown_mean_leaf_angle_deg: parameters[6],
        view_zenith_deg: parameters[7],
        solar_zenith_deg: parameters[8],
        relative_azimuth_deg: parameters[9],
        diffuse_fraction: parameters[10],
        crown_leaf: rsphysics_prospect_d::Input {
            n: parameters[11],
            chlorophyll: parameters[12],
            water: parameters[13],
            dry_matter: parameters[14],
            ..Input::default().crown_leaf
        },
        spectra: Some(SpectralInputs {
            wavelengths_nm: SpectralAxis::new(column[0].clone()).unwrap(),
            soil_reflectance: Some(column[1].clone()),
            crown_leaf: coefficients.is_none().then(|| LeafSpectrum {
                reflectance: column[2].clone(),
                transmittance: column[3].clone(),
            }),
            understory_leaf: coefficients.is_none().then(|| LeafSpectrum {
                reflectance: column[4].clone(),
                transmittance: column[5].clone(),
            }),
            prospect_301_coefficients: coefficients.cloned(),
        }),
        ..Input::default()
    };
    let output = model.simulate(&input).unwrap();
    let tol = if coefficients.is_some() { 2e-8 } else { 5e-11 };
    let spectra = [
        ("crown leaf reflectance", 2, &output.crown_leaf_reflectance),
        (
            "crown leaf transmittance",
            3,
            &output.crown_leaf_transmittance,
        ),
        (
            "understory leaf reflectance",
            4,
            &output.understory_leaf_reflectance,
        ),
        (
            "understory leaf transmittance",
            5,
            &output.understory_leaf_transmittance,
        ),
        ("forest reflectance", 6, &output.reflectance),
        ("soil reflectance", 7, &output.soil_reflectance),
        ("understory reflectance", 8, &output.understory_reflectance),
        (
            "dense crown reflectance",
            9,
            &output.dense_crown_reflectance,
        ),
        ("crown factor", 11, &output.crown_factor),
        ("ground factor", 12, &output.ground_factor),
        (
            "solar crown transmittance",
            13,
            &output.crown_transmittance_sun,
        ),
        (
            "view crown transmittance",
            14,
            &output.crown_transmittance_view,
        ),
    ];
    for (name, i, values) in spectra {
        let (band, max_error) = values
            .iter()
            .zip(&column[i])
            .enumerate()
            .map(|(b, (x, y))| (b, (x - y).abs()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .unwrap();
        assert!(
            max_error <= tol,
            "{name}: error {max_error} at {} nm ({}, {}) in {parameters:?}",
            column[0][band],
            values[band],
            column[i][band]
        );
    }
    assert!((output.crown_cover - column[10][0]).abs() < 1e-13);
}

#[test]
fn inform_matches_corrected_matlab_with_supplied_leaf_optics() {
    let model = Model::new().unwrap();
    for (p, spectrum) in CASES.lines().zip(SPECTRA) {
        let parameters = p.split(',').map(|x| x.parse().unwrap()).collect::<Vec<_>>();
        compare(&model, &parameters, spectrum, None);
    }
}

#[test]
#[ignore = "requires the author MATLAB reference and coefficient table"]
fn inform_matches_corrected_matlab_full_spectra_with_prospect_301() {
    let reference =
        PathBuf::from(std::env::var_os("INFORM_REFERENCE_DIR").expect("INFORM_REFERENCE_DIR"));
    let table = read_csv(&fs::read_to_string(reference.join("coefficients.csv")).unwrap());
    let coefficients = Prospect301Coefficients {
        refractive_index: table[1].clone(),
        albino_absorption: table[2].clone(),
        chlorophyll_absorption: table[3].clone(),
        water_absorption: table[4].clone(),
        dry_matter_absorption: table[5].clone(),
    };
    let model = Model::new().unwrap();
    for (i, p) in fs::read_to_string(reference.join("parameters.csv"))
        .unwrap()
        .lines()
        .enumerate()
    {
        let parameters = p.split(',').map(|x| x.parse().unwrap()).collect::<Vec<_>>();
        let spectrum = fs::read_to_string(reference.join(format!("case{}.csv", i + 1))).unwrap();
        compare(&model, &parameters, &spectrum, Some(&coefficients));
    }
}

#[test]
fn crown_and_understory_limits_and_fraction_closure() {
    let model = Model::new().unwrap();
    let axis = SpectralAxis::new(vec![400., 701.5, 2500.]).unwrap();
    let mut input = Input {
        crown_diameter_m: 0.,
        understory_lai: 0.,
        spectra: Some(SpectralInputs {
            wavelengths_nm: axis.clone(),
            soil_reflectance: Some(vec![0.1, 0.2, 0.3]),
            crown_leaf: None,
            understory_leaf: None,
            prospect_301_coefficients: None,
        }),
        ..Input::default()
    };
    let bare = model.simulate(&input).unwrap();
    assert_eq!(bare.wavelengths_nm, axis);
    assert_eq!(bare.reflectance, bare.soil_reflectance);
    assert_eq!(bare.crown_cover, 0.);
    assert_eq!(bare.sunlit_gap_fraction, 1.);
    input.crown_diameter_m = 6.;
    input.crown_lai = 0.;
    input.understory_lai = 2.;
    input.tree_height_m = 0.;
    input.view_zenith_deg = 69.;
    let transparent = model.simulate(&input).unwrap();
    for (forest, background) in transparent
        .reflectance
        .iter()
        .zip(&transparent.understory_reflectance)
    {
        assert!((forest - background).abs() < 1e-14);
    }
    let fractions = [
        transparent.shadowed_crown_fraction,
        transparent.sunlit_crown_fraction,
        transparent.shadowed_gap_fraction,
        transparent.sunlit_gap_fraction,
    ];
    assert!(fractions.into_iter().all(|f| (0.0..=1.0).contains(&f)));
    assert!((fractions.into_iter().sum::<f64>() - 1.).abs() < 1e-14);
    input.spectra.as_mut().unwrap().soil_reflectance = Some(vec![0.2]);
    assert!(model.simulate(&input).is_err());
}

#[test]
fn reflective_background_and_conservative_leaves_keep_mixing_weights_physical() {
    let optics = LeafSpectrum {
        reflectance: vec![0.5],
        transmittance: vec![0.5],
    };
    let input = Input {
        crown_lai: 30.,
        understory_lai: 30.,
        solar_zenith_deg: 0.,
        soil_reflectance: 1.,
        spectra: Some(SpectralInputs {
            wavelengths_nm: SpectralAxis::new(vec![850.]).unwrap(),
            soil_reflectance: None,
            crown_leaf: Some(optics.clone()),
            understory_leaf: Some(optics),
            prospect_301_coefficients: None,
        }),
        ..Input::default()
    };
    let output = Model::new().unwrap().simulate(&input).unwrap();
    assert!(output.understory_reflectance[0] > 1.);
    assert!(output.reflectance[0] >= 0.);
    for value in [
        output.crown_transmittance_sun[0],
        output.crown_transmittance_view[0],
        output.crown_factor[0],
        output.ground_factor[0],
    ] {
        assert!((0.0..=1.0).contains(&value));
    }
}
