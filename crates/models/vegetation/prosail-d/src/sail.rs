//! Optical 4SAIL outputs backed by the shared public four-stream kernel.
use crate::{Input, LeafAngleDistribution};
use rsphysics_numerics::sail::{Geometry as Kernel, LeafAngleDistribution as Lidf, Parameters};
pub struct Geometry {
    kernel: Kernel,
    parameters: Parameters,
}
impl Geometry {
    pub fn new(p: &Input) -> Self {
        let parameters = Parameters {
            leaf_area_index: p.leaf_area_index,
            mean_leaf_angle_deg: p.mean_leaf_angle_deg,
            leaf_angle_distribution: match p.leaf_angle_distribution {
                LeafAngleDistribution::Campbell => Lidf::Campbell,
                LeafAngleDistribution::Verhoef => Lidf::Verhoef,
            },
            lidf_a: p.lidf_a,
            lidf_b: p.lidf_b,
            hotspot: p.hotspot,
            solar_zenith_deg: p.solar_zenith_deg,
            view_zenith_deg: p.view_zenith_deg,
            relative_azimuth_deg: p.relative_azimuth_deg,
        };
        Self {
            kernel: Kernel::new(&parameters),
            parameters,
        }
    }
    pub fn solve(&self, rho: f64, tau: f64, soil: f64, _p: &Input) -> [f64; 9] {
        let c = self.kernel.coefficients(rho, tau, &self.parameters);
        let dn = 1. - soil * c.rdd;
        let hemi = c.rsd + (c.tsd + c.tss) * soil * c.tdd / dn;
        let brf = c.rso
            + c.tsstoo * soil
            + ((c.tss + c.tsd) * c.tdo + (c.tsd + c.tss * soil * c.rdd) * c.too) * soil / dn;
        let rddt = c.rdd + c.tdd * soil * c.tdd / dn;
        let rdot = c.rdo + c.tdd * soil * (c.tdo + c.too) / dn;
        let abs_dir =
            1. - hemi - (1. - soil) * c.tss - (1. - soil) * (c.tss * soil * c.rdd + c.tsd) / dn;
        let abs_hem = 1. - rddt - (1. - soil) * c.tdd - (1. - soil) * (c.tdd * c.rdd * soil) / dn;
        [
            rdot,
            brf,
            rddt,
            hemi,
            1. - c.too,
            abs_dir,
            abs_hem,
            hemi,
            rddt,
        ]
    }
}
