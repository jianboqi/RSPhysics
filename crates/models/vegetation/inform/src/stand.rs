use crate::Input;
use rsphysics_numerics::sail::{Coefficients, Geometry, LeafAngleDistribution, Parameters};
use std::f64::consts::PI;

pub(crate) struct Stand {
    pub cover: f64,
    pub shadow: f64,
    pub shadowed_crown: f64,
    pub sunlit_crown: f64,
    pub shadowed_gap: f64,
    pub sunlit_gap: f64,
    understory: Layer,
    dense_crown: Layer,
    crown: Layer,
    reverse_crown: Layer,
    diffuse: f64,
}

pub(crate) struct Band {
    pub forest: f64,
    pub understory: f64,
    pub dense_crown: f64,
    pub transmittance_sun: f64,
    pub transmittance_view: f64,
    pub crown_factor: f64,
    pub ground_factor: f64,
}

impl Stand {
    pub fn new(input: &Input) -> Self {
        let view = input.view_zenith_deg.to_radians();
        let sun = input.solar_zenith_deg.to_radians();
        let azimuth = input.relative_azimuth_deg.to_radians();
        let diameter = input.crown_diameter_m;
        let projected = PI * (diameter / 2.).powi(2) * input.stem_density_per_ha / 10_000.;
        let cover = -(-projected / view.cos()).exp_m1();
        let shadow = -(-projected / sun.cos()).exp_m1();
        let separation = (view.tan() - sun.tan() * azimuth.cos()).hypot(sun.tan() * azimuth.sin());
        let correlation = if diameter == 0. {
            0.
        } else {
            (-separation * input.tree_height_m / diameter).exp()
        };
        let overlap = correlation * (cover * (1. - cover) * shadow * (1. - shadow)).sqrt();
        // An intersection cannot cover more ground than either marginal.
        let joint = (cover * shadow + overlap).min(cover.min(shadow));
        let parameters = Parameters {
            leaf_area_index: input.crown_lai,
            mean_leaf_angle_deg: input.crown_mean_leaf_angle_deg,
            leaf_angle_distribution: LeafAngleDistribution::Campbell,
            lidf_a: 0.,
            lidf_b: 0.,
            hotspot: 0.,
            solar_zenith_deg: input.solar_zenith_deg,
            view_zenith_deg: input.view_zenith_deg,
            relative_azimuth_deg: input.relative_azimuth_deg,
        };
        Self {
            cover,
            shadow,
            shadowed_crown: joint,
            sunlit_crown: cover - joint,
            shadowed_gap: shadow - joint,
            sunlit_gap: (1. - cover) * (1. - shadow) + (joint - cover * shadow),
            understory: Layer::new(Parameters {
                leaf_area_index: input.understory_lai,
                mean_leaf_angle_deg: input.understory_mean_leaf_angle_deg,
                hotspot: input.understory_hotspot,
                ..parameters
            }),
            dense_crown: Layer::new(Parameters {
                leaf_area_index: 15.,
                hotspot: input.crown_hotspot,
                ..parameters
            }),
            crown: Layer::new(parameters),
            reverse_crown: Layer::new(Parameters {
                solar_zenith_deg: input.view_zenith_deg,
                ..parameters
            }),
            diffuse: input.diffuse_fraction,
        }
    }

    pub fn solve(&self, soil: f64, crown: (f64, f64), understory: (f64, f64)) -> Band {
        let background = if self.understory.parameters.leaf_area_index == 0. {
            soil
        } else {
            reflectance(self.understory.solve(understory), soil, self.diffuse)
        };
        // The directional background can exceed one. Its Lambertian equivalent
        // is bounded when it is used for flux feedback within the crown.
        let boundary = background.clamp(0., 1.);
        let dense_crown = reflectance(self.dense_crown.solve(crown), boundary, self.diffuse);
        let sun = transmittance(self.crown.solve(crown), boundary, self.diffuse).clamp(0., 1.);
        let view =
            transmittance(self.reverse_crown.solve(crown), boundary, self.diffuse).clamp(0., 1.);
        let crown_factor = self.shadowed_crown * (1. - sun * view);
        let ground_factor = self.shadowed_crown * sun * view
            + self.sunlit_crown * view
            + self.shadowed_gap * sun
            + self.sunlit_gap;
        Band {
            forest: dense_crown * crown_factor + background * ground_factor,
            understory: background,
            dense_crown,
            transmittance_sun: sun,
            transmittance_view: view,
            crown_factor,
            ground_factor,
        }
    }
}

struct Layer {
    parameters: Parameters,
    geometry: Geometry,
}

impl Layer {
    fn new(parameters: Parameters) -> Self {
        Self {
            geometry: Geometry::inform(&parameters),
            parameters,
        }
    }

    fn solve(&self, optics: (f64, f64)) -> Coefficients {
        self.geometry
            .inform_coefficients(optics.0, optics.1, &self.parameters)
    }
}

fn reflectance(c: Coefficients, background: f64, diffuse: f64) -> f64 {
    let feedback = 1. - background * c.rdd;
    let directional = c.rso
        + background * c.tsstoo
        + background * ((c.tss + c.tsd) * c.tdo + (c.tsd + background * c.tss * c.rdd) * c.too)
            / feedback;
    let hemispherical_directional = c.rdo + background * c.tdd * (c.tdo + c.too) / feedback;
    directional * (1. - diffuse) + hemispherical_directional * diffuse
}

fn transmittance(c: Coefficients, background: f64, diffuse: f64) -> f64 {
    ((c.tss + c.tsd) * (1. - diffuse) + c.tdd * diffuse) / (1. - background * c.rdd)
}
