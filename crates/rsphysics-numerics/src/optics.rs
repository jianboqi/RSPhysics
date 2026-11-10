//! Single planar nonmagnetic interface. Incident medium is real and lossless.
//! e^(-i omega t); transmitted n+i*k has n>=0,k>=0, not both zero.
use num_complex::Complex64 as C;
use rsphysics_core::DomainError;
#[derive(Debug, Clone, Copy)]
pub struct Fresnel {
    pub amplitude_r_s: C,
    pub amplitude_r_p: C,
    pub reflectance_s: f64,
    pub reflectance_p: f64,
    pub reflectance_unpolarized: f64,
    /// Poynting flux entering medium 2 at the interface, not slab throughput.
    pub interface_transmittance_s: f64,
    pub interface_transmittance_p: f64,
    pub interface_transmittance_unpolarized: f64,
    pub total_internal_reflection: bool,
}
fn finite(z: C) -> bool {
    z.re.is_finite() && z.im.is_finite()
}
fn interface(a: C, b: C) -> Result<(C, f64, f64), DomainError> {
    let scale = a.norm().max(b.norm());
    if !scale.is_finite() || scale == 0. {
        return Err(DomainError(
            "Fresnel admittance exceeds supported f64 range",
        ));
    }
    let a = a / scale;
    let b = b / scale;
    let sum = a + b;
    let r = (a - b) / sum;
    let power = r.norm_sqr();
    let flux = 4. * (a * b.conj()).re / sum.norm_sqr();
    if !finite(r) || !power.is_finite() || !flux.is_finite() || flux < -1e-12 || power > 1. + 1e-12
    {
        return Err(DomainError("nonfinite or non-passive Fresnel result"));
    }
    Ok((r, power, flux))
}
pub fn fresnel(
    incidence_angle_deg: f64,
    n_incident: f64,
    n_transmitted: C,
) -> Result<Fresnel, DomainError> {
    if !incidence_angle_deg.is_finite()
        || !(0.0..90.0).contains(&incidence_angle_deg)
        || !n_incident.is_finite()
        || n_incident <= 0.
        || !finite(n_transmitted)
        || n_transmitted.re < 0.
        || n_transmitted.im < 0.
        || n_transmitted == C::new(0., 0.)
    {
        return Err(DomainError(
            "Fresnel requires angle in [0,90), positive real incident index and passive nonzero transmitted n+i*k (n,k>=0)",
        ));
    }
    let m = n_transmitted / n_incident;
    if !finite(m) || m == C::new(0., 0.) {
        return Err(DomainError(
            "relative refractive index outside supported f64 range",
        ));
    }
    let theta = incidence_angle_deg.to_radians();
    let c = theta.cos();
    let s = theta.sin();
    // Avoid loss of cos(theta)^2 at grazing incidence for equal/near-equal media.
    let q = if m.norm() > 2. {
        m * (C::new(1., 0.) - (C::new(s, 0.) / m).powi(2)).sqrt()
    } else if m.norm() < 0.5 {
        (m * m - C::new(s * s, 0.)).sqrt()
    } else {
        C::new(
            (m.re - 1.).mul_add(m.re + 1., c * c) - m.im * m.im,
            2. * m.re * m.im,
        )
        .sqrt()
    };
    // Principal square root is the decaying/forward branch for this passive scope.
    let (rs, r_s, t_s) = interface(C::new(c, 0.), q)?;
    let (rp, r_p, t_p) = interface(m * m * c, q)?;
    Ok(Fresnel {
        amplitude_r_s: rs,
        amplitude_r_p: rp,
        reflectance_s: r_s,
        reflectance_p: r_p,
        reflectance_unpolarized: (r_s + r_p) / 2.,
        interface_transmittance_s: t_s,
        interface_transmittance_p: t_p,
        interface_transmittance_unpolarized: (t_s + t_p) / 2.,
        total_internal_reflection: n_transmitted.im == 0. && q.re == 0.,
    })
}
/// Relative permittivities; nonmagnetic media and the same e^(-i omega t) sign.
pub fn fresnel_permittivity(
    angle_deg: f64,
    epsilon_incident: f64,
    epsilon_transmitted: C,
) -> Result<Fresnel, DomainError> {
    if !epsilon_incident.is_finite()
        || epsilon_incident <= 0.
        || !finite(epsilon_transmitted)
        || epsilon_transmitted.im < 0.
        || epsilon_transmitted == C::new(0., 0.)
    {
        return Err(DomainError(
            "permittivity requires positive real incident epsilon and passive nonzero transmitted epsilon with Im>=0",
        ));
    }
    fresnel(
        angle_deg,
        epsilon_incident.sqrt(),
        epsilon_transmitted.sqrt(),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_brewster_and_matched_grazing() {
        let a = fresnel(0., 1., C::new(1.5, 0.)).unwrap();
        assert!((a.reflectance_s - 0.04).abs() < 1e-15);
        assert!((a.amplitude_r_s.re + a.amplitude_r_p.re).abs() < 1e-15);
        let b = fresnel(1.5_f64.atan().to_degrees(), 1., C::new(1.5, 0.)).unwrap();
        assert!(b.reflectance_p < 1e-29);
        let m = fresnel(89.999999, 1., C::new(1., 0.)).unwrap();
        assert!(m.reflectance_unpolarized < 1e-28);
        assert!((m.interface_transmittance_s - 1.).abs() < 1e-14);
    }
    #[test]
    fn total_reflection_absorption_and_permittivity() {
        let tir = fresnel(60., 1.5, C::new(1., 0.)).unwrap();
        assert!(tir.total_internal_reflection);
        assert!((tir.reflectance_s - 1.).abs() < 1e-14);
        assert_eq!(tir.interface_transmittance_s, 0.);
        for n in [C::new(1.5, 0.), C::new(0.2, 3.), C::new(0., 2.)] {
            for angle in [0., 40., 89.9] {
                let a = fresnel(angle, 1., n).unwrap();
                assert!((a.reflectance_s + a.interface_transmittance_s - 1.).abs() < 1e-14);
                assert!((a.reflectance_p + a.interface_transmittance_p - 1.).abs() < 1e-14);
                let b = fresnel_permittivity(angle, 1., n * n).unwrap();
                assert!((a.reflectance_s - b.reflectance_s).abs() < 1e-14);
            }
        }
        assert!(fresnel(90., 1., C::new(1.5, 0.)).is_err());
        assert!(fresnel(0., 1., C::new(1.5, -0.1)).is_err());
    }
}
