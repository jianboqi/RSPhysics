use rsphysics_model_api::{ErrorKind, ModelError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
fn invalid(s: impl Into<String>) -> ModelError {
    ModelError::new(ErrorKind::InvalidParameter, s)
}
fn range(name: &str, x: f64, lo: f64, hi: f64) -> Result<(), ModelError> {
    if !x.is_finite() || x < lo || x > hi {
        Err(invalid(format!(
            "{name} must be finite and in [{lo}, {hi}]"
        )))
    } else {
        Ok(())
    }
}
fn finite(name: &str, x: &[f64]) -> Result<(), ModelError> {
    if x.iter().any(|v| !v.is_finite()) {
        Err(invalid(format!("{name} must be finite")))
    } else {
        Ok(())
    }
}
fn row(values: impl IntoIterator<Item = f64>) -> String {
    values
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn filename(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 80
        && name.is_ascii()
        && !matches!(
            name.to_ascii_lowercase().as_str(),
            "input.cards" | "report.txt" | "error.txt"
        )
        && !name.contains("..")
        && !name
            .chars()
            .any(|c| c == '/' || c == '\\' || c == ':' || c.is_whitespace() || c.is_control())
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Geometry {
    pub mode: u8,
    pub max_scattering_order: Option<u32>,
    pub solar_zenith_deg: f64,
    pub solar_azimuth_deg: f64,
    pub view_zenith_deg: f64,
    pub view_azimuth_deg: f64,
    pub month: u8,
    pub day: u8,
    pub utc_hour: f64,
    pub column: u16,
    pub line: u16,
    pub longitude_deg: f64,
    pub latitude_deg: f64,
    pub ascending_node_longitude_deg: f64,
    pub ascending_node_hour: f64,
}
impl Default for Geometry {
    fn default() -> Self {
        Self {
            mode: 0,
            max_scattering_order: None,
            solar_zenith_deg: 30.,
            solar_azimuth_deg: 0.,
            view_zenith_deg: 20.,
            view_azimuth_deg: 90.,
            month: 7,
            day: 1,
            utc_hour: 12.,
            column: 1000,
            line: 1000,
            longitude_deg: 0.,
            latitude_deg: 30.,
            ascending_node_longitude_deg: 0.,
            ascending_node_hour: 14.,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Atmosphere {
    pub profile: u8,
    pub water_vapour_g_cm2: f64,
    pub ozone_atm_cm: f64,
    pub levels: Vec<[f64; 5]>,
}
impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            profile: 2,
            water_vapour_g_cm2: 2.,
            ozone_atm_cm: 0.3,
            levels: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct LogNormal {
    pub mean_radius_um: f64,
    pub sigma: f64,
    pub fraction: f64,
    pub refractive_real: Vec<f64>,
    pub refractive_imag: Vec<f64>,
}
impl Default for LogNormal {
    fn default() -> Self {
        Self {
            mean_radius_um: 0.1,
            sigma: 1.5,
            fraction: 1.,
            refractive_real: vec![1.5; 20],
            refractive_imag: vec![0.01; 20],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Aerosol {
    pub model: i8,
    pub aot550: f64,
    pub visibility_km: Option<f64>,
    pub components: [f64; 4],
    pub layers: Vec<[f64; 3]>,
    pub radius_min_um: f64,
    pub radius_max_um: f64,
    pub modes: Vec<LogNormal>,
    pub gamma: [f64; 3],
    pub junge_exponent: f64,
    pub sunphotometer: Vec<[f64; 2]>,
    pub refractive_real: Vec<f64>,
    pub refractive_imag: Vec<f64>,
    pub save_mie: bool,
    pub mie_file: String,
}
impl Default for Aerosol {
    fn default() -> Self {
        Self {
            model: 1,
            aot550: 0.2,
            visibility_km: None,
            components: [0.7, 0.29, 0., 0.01],
            layers: vec![],
            radius_min_um: 0.01,
            radius_max_um: 1.,
            modes: vec![LogNormal::default()],
            gamma: [1., 1., 1.],
            junge_exponent: 3.5,
            sunphotometer: vec![],
            refractive_real: vec![1.5; 20],
            refractive_imag: vec![0.01; 20],
            save_mie: true,
            mie_file: "aerosol".into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Altitudes {
    pub target_altitude_km: f64,
    pub target_pressure_hpa: Option<f64>,
    pub sensor_height_km: Option<f64>,
    pub path_water_vapour_g_cm2: f64,
    pub path_ozone_atm_cm: f64,
    pub path_aot550: f64,
}
impl Default for Altitudes {
    fn default() -> Self {
        Self {
            target_altitude_km: 0.,
            target_pressure_hpa: None,
            sensor_height_km: None,
            path_water_vapour_g_cm2: -1.,
            path_ozone_atm_cm: -1.,
            path_aot550: -1.,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Spectrum {
    pub mode: String,
    pub wavelength_um: f64,
    pub lower_um: f64,
    pub upper_um: f64,
    pub response: Vec<f64>,
    pub band: u16,
}
impl Default for Spectrum {
    fn default() -> Self {
        Self {
            mode: "monochromatic".into(),
            wavelength_um: 0.55,
            lower_um: 0.5,
            upper_um: 0.6,
            response: vec![],
            band: 43,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Reflectance {
    pub spectrum: i8,
    pub value: f64,
    pub lower_um: f64,
    pub upper_um: f64,
    pub values: Vec<f64>,
}
impl Default for Reflectance {
    fn default() -> Self {
        Self {
            spectrum: 0,
            value: 0.2,
            lower_um: 0.5,
            upper_um: 0.6,
            values: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Surface {
    pub mode: String,
    pub target: Reflectance,
    pub background: Reflectance,
    pub radius_km: f64,
    pub brdf_model: u8,
    pub parameters: Vec<f64>,
    pub sun_table: Vec<Vec<f64>>,
    pub view_table: Vec<Vec<f64>>,
    pub albedo: f64,
    pub directional_reflectance: f64,
}
impl Default for Surface {
    fn default() -> Self {
        Self {
            mode: "lambertian".into(),
            target: Reflectance::default(),
            background: Reflectance::default(),
            radius_km: 1.,
            brdf_model: 10,
            parameters: vec![0.3091, 0.129, 0.0762],
            sun_table: vec![],
            view_table: vec![],
            albedo: 0.2,
            directional_reflectance: 0.2,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Correction {
    pub mode: String,
    pub value: f64,
}
impl Default for Correction {
    fn default() -> Self {
        Self {
            mode: "none".into(),
            value: 0.2,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Polarization {
    pub mode: String,
    pub q: f64,
    pub u: f64,
    pub vegetation_fraction: f64,
    pub wind_speed_m_s: f64,
    pub wind_azimuth_deg: f64,
}
impl Default for Polarization {
    fn default() -> Self {
        Self {
            mode: "none".into(),
            q: 0.,
            u: 0.,
            vegetation_fraction: 0.5,
            wind_speed_m_s: 5.,
            wind_azimuth_deg: 0.,
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    pub geometry: Geometry,
    pub atmosphere: Atmosphere,
    pub aerosol: Aerosol,
    pub altitudes: Altitudes,
    pub spectrum: Spectrum,
    pub surface: Surface,
    pub correction: Correction,
    pub polarization: Polarization,
    pub program: Program,
    pub input_cards: Option<String>,
    pub files: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(inline)]
pub enum Program {
    #[default]
    Standard,
    Lutaero,
}
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Prepared {
    pub program: String,
    pub input_cards: String,
    pub files: BTreeMap<String, String>,
}
fn grid_count(lo: f64, hi: f64) -> Result<usize, ModelError> {
    range("lower_um", lo, 0.25, 4.)?;
    range("upper_um", hi, lo, 4.)?;
    Ok(((hi - 0.25) / 0.0025).round() as usize - ((lo - 0.25) / 0.0025).round() as usize + 1)
}
fn refractive(nr: &[f64], ni: &[f64], lines: &mut Vec<String>) -> Result<(), ModelError> {
    if nr.len() != 20 || ni.len() != 20 {
        return Err(invalid(
            "Refractive indices require exactly 20 original 6SV2.1 wavelengths",
        ));
    }
    for &x in nr {
        range("real refractive index", x, 1., 5.)?;
    }
    for &x in ni {
        range("imaginary refractive index", x, 0., 5.)?;
    }
    lines.push(row(nr.iter().copied()));
    lines.push(row(ni.iter().copied()));
    Ok(())
}
impl Input {
    pub fn prepare(&self) -> Result<Prepared, ModelError> {
        for (name, text) in &self.files {
            if !filename(name) || !text.is_ascii() || text.len() > 16_000_000 {
                return Err(invalid(
                    "Auxiliary files require simple ASCII filenames and ASCII contents up to 16 MB",
                ));
            }
        }
        let program = match self.program {
            Program::Standard => "sixsV2.1",
            Program::Lutaero => "sixsV2.1-lutaero",
        }
        .into();
        let cards = if let Some(cards) = &self.input_cards {
            if cards.trim().is_empty()
                || !cards.is_ascii()
                || cards.len() > 2_000_000
                || cards.contains('\0')
            {
                return Err(invalid("input_cards must contain nonempty ASCII 6S input"));
            }
            format!("{}\n", cards.trim_end())
        } else {
            self.render()?
        };
        Ok(Prepared {
            program,
            input_cards: cards,
            files: self.files.clone(),
        })
    }
    fn render(&self) -> Result<String, ModelError> {
        let mut s = vec![];
        let g = &self.geometry;
        if g.mode > 7 || !(1..=12).contains(&g.month) {
            return Err(invalid("geometry mode must be 0..7; month 1..12"));
        }
        let days = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if g.day == 0 || g.day > days[g.month as usize - 1] {
            return Err(invalid("Invalid month/day"));
        }
        let geometry_code = match g.max_scattering_order {
            None => i32::from(g.mode),
            // The original special branch requires igeom < -10.
            Some(1) if g.mode == 0 => -11,
            Some(order) if g.mode == 0 && (2..=i32::MAX as u32 / 10).contains(&order) => {
                -(order as i32 * 10)
            }
            Some(_) => {
                return Err(invalid(
                    "max_scattering_order requires user geometry and an integer from 1 to 214748364",
                ));
            }
        };
        s.push(geometry_code.to_string());
        let date = [f64::from(g.month), f64::from(g.day)];
        match g.mode {
            0 => {
                range("solar zenith", g.solar_zenith_deg, 0., 89.)?;
                range("view zenith", g.view_zenith_deg, 0., 89.)?;
                range("solar azimuth", g.solar_azimuth_deg, 0., 360.)?;
                range("view azimuth", g.view_azimuth_deg, 0., 360.)?;
                s.push(row([
                    g.solar_zenith_deg,
                    g.solar_azimuth_deg,
                    g.view_zenith_deg,
                    g.view_azimuth_deg,
                    date[0],
                    date[1],
                ]));
            }
            1..=3 => {
                range("utc_hour", g.utc_hour, 0., 24.)?;
                s.push(row([
                    date[0],
                    date[1],
                    g.utc_hour,
                    f64::from(g.column),
                    f64::from(g.line),
                ]));
            }
            4..=5 => {
                range("AVHRR column", f64::from(g.column), 1., 2048.)?;
                s.push(row([
                    date[0],
                    date[1],
                    g.utc_hour,
                    f64::from(g.column),
                    g.ascending_node_longitude_deg,
                    g.ascending_node_hour,
                ]));
            }
            _ => {
                range("latitude", g.latitude_deg, -90., 90.)?;
                range("longitude", g.longitude_deg, -180., 180.)?;
                s.push(row([
                    date[0],
                    date[1],
                    g.utc_hour,
                    g.longitude_deg,
                    g.latitude_deg,
                ]));
            }
        }
        let a = &self.atmosphere;
        if a.profile > 8 {
            return Err(invalid("atmosphere.profile must be 0..8"));
        }
        s.push(a.profile.to_string());
        if a.profile == 7 {
            if a.levels.len() != 34 {
                return Err(invalid(
                    "A radiosonde profile requires exactly 34 [km,hPa,K,g/m3,g/m3] rows",
                ));
            }
            for (i, v) in a.levels.iter().enumerate() {
                finite("radiosonde", v)?;
                if i > 0 && v[0] <= a.levels[i - 1][0] {
                    return Err(invalid("Radiosonde altitudes must increase"));
                }
                range("profile pressure", v[1], 0., 1200.)?;
                range("profile temperature", v[2], 100., 400.)?;
                range("profile water", v[3], 0., 100.)?;
                range("profile ozone", v[4], 0., 100.)?;
                s.push(row(*v));
            }
        }
        if a.profile == 8 {
            range("water", a.water_vapour_g_cm2, 0., 20.)?;
            range("ozone", a.ozone_atm_cm, 0., 2.)?;
            s.push(row([a.water_vapour_g_cm2, a.ozone_atm_cm]));
        }
        let a = &self.aerosol;
        if !(-1..=12).contains(&a.model) {
            return Err(invalid("aerosol.model must be -1..12"));
        }
        s.push(a.model.to_string());
        let optical_model = if a.model == -1 {
            if a.layers.is_empty() || a.layers.len() > 50 {
                return Err(invalid("Aerosol profile requires 1..50 layers"));
            }
            s.push(a.layers.len().to_string());
            let mut height = 0.;
            let mut kind = None;
            for layer in &a.layers {
                finite("aerosol layer", layer)?;
                range("layer height", layer[0], 0.000001, 300.)?;
                range("layer AOT550", layer[1], 0., 10.)?;
                if !(1. ..=12.).contains(&layer[2])
                    || layer[2].fract() != 0.
                    || kind.is_some_and(|k| k != layer[2])
                {
                    return Err(invalid(
                        "Layers must share one aerosol optical model (1..12)",
                    ));
                }
                kind = Some(layer[2]);
                height += layer[0];
                s.push(format!("{} {} {}", layer[0], layer[1], layer[2] as i32));
            }
            if height > 300. {
                return Err(invalid("Aerosol layer heights exceed 300 km"));
            }
            kind.expect("nonempty validated layers") as i8
        } else {
            a.model
        };
        if optical_model == 4 {
            for &x in &a.components {
                range("aerosol component fraction", x, 0., 1.)?;
            }
            if (a.components.iter().sum::<f64>() - 1.).abs() > 1e-6 {
                return Err(invalid("Aerosol component fractions must sum to 1"));
            }
            s.push(row(a.components));
        }
        if (8..=10).contains(&optical_model) {
            range("radius_min_um", a.radius_min_um, 0.0001, 100.)?;
            range(
                "radius_max_um",
                a.radius_max_um,
                a.radius_min_um + 1e-8,
                100.,
            )?;
        }
        match optical_model {
            8 => {
                if a.modes.is_empty() || a.modes.len() > 4 {
                    return Err(invalid("Log-normal distribution requires 1..4 modes"));
                }
                s.push(format!(
                    "{} {} {}",
                    a.radius_min_um,
                    a.radius_max_um,
                    a.modes.len()
                ));
                let mut total = 0.;
                for m in &a.modes {
                    range("mean_radius_um", m.mean_radius_um, 0.0001, 100.)?;
                    range("sigma", m.sigma, 1.000001, 10.)?;
                    range("mode fraction", m.fraction, 0., 1.)?;
                    total += m.fraction;
                    s.push(row([m.mean_radius_um, m.sigma, m.fraction]));
                    refractive(&m.refractive_real, &m.refractive_imag, &mut s)?;
                }
                if (total - 1.).abs() > 1e-6 {
                    return Err(invalid("Log-normal fractions must sum to 1"));
                }
            }
            9 | 10 => {
                s.push(row([a.radius_min_um, a.radius_max_um]));
                if optical_model == 9 {
                    finite("gamma parameters", &a.gamma)?;
                    s.push(row(a.gamma));
                } else {
                    range("Junge exponent", a.junge_exponent, 0.1, 10.)?;
                    s.push(a.junge_exponent.to_string());
                }
                refractive(&a.refractive_real, &a.refractive_imag, &mut s)?;
            }
            11 => {
                if a.sunphotometer.len() < 2 || a.sunphotometer.len() > 50 {
                    return Err(invalid("Sun-photometer distribution requires 2..50 rows"));
                }
                s.push(a.sunphotometer.len().to_string());
                for (i, v) in a.sunphotometer.iter().enumerate() {
                    range("sun-photometer radius", v[0], 0.0001, 100.)?;
                    range("dV/dlogr", v[1], 0., 1e6)?;
                    if i > 0 && v[0] <= a.sunphotometer[i - 1][0] {
                        return Err(invalid("Sun-photometer radii must increase"));
                    }
                    s.push(row(*v));
                }
                refractive(&a.refractive_real, &a.refractive_imag, &mut s)?;
            }
            12 => {
                if !filename(&a.mie_file) || !self.files.contains_key(&a.mie_file) {
                    return Err(invalid(
                        "Mie reuse requires mie_file and matching files entry",
                    ));
                }
                let count = self.files[&a.mie_file]
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<usize>().ok())
                    .ok_or_else(|| invalid("Mie file must start with its quadrature count"))?;
                if !(3..=1001).contains(&count) || count % 2 == 0 {
                    return Err(invalid("Mie quadrature count must be odd and in 3..1001"));
                }
                s.push(a.mie_file.clone());
            }
            _ => {}
        }
        if (8..=11).contains(&optical_model) {
            if !filename(&a.mie_file) {
                return Err(invalid("Invalid Mie output basename"));
            }
            s.push(if a.save_mie { "1" } else { "0" }.into());
            if a.save_mie {
                s.push(a.mie_file.clone());
            }
        }
        if a.model >= 0 {
            if a.model == 0 {
                s.push("-1".into());
            } else if let Some(v) = a.visibility_km {
                range("visibility_km", v, 0.01, 1000.)?;
                s.push(v.to_string());
            } else {
                range("aot550", a.aot550, 0.000001, 10.)?;
                s.push("0".into());
                s.push(a.aot550.to_string());
            }
        }
        let h = &self.altitudes;
        range("target_altitude_km", h.target_altitude_km, 0., 99.)?;
        if let Some(p) = h.target_pressure_hpa {
            range("pressure", p, 0.01, 1200.)?;
            if h.target_altitude_km != 0. {
                return Err(invalid("Choose target pressure or altitude"));
            }
            s.push(p.to_string());
        } else {
            s.push((-h.target_altitude_km).to_string());
        }
        if let Some(z) = h.sensor_height_km {
            if !z.is_finite() || !(0.0..100.0).contains(&z) || (z as f32) >= 100.0 {
                return Err(invalid("sensor_height_km must be in [0, 100)"));
            }
            s.push((-z).to_string());
            if z > 0. {
                range("path water", h.path_water_vapour_g_cm2, -1., 20.)?;
                range("path ozone", h.path_ozone_atm_cm, -1., 2.)?;
                range("path AOT", h.path_aot550, -1., 10.)?;
                s.push(row([h.path_water_vapour_g_cm2, h.path_ozone_atm_cm]));
                s.push(h.path_aot550.to_string());
            }
        } else {
            s.push("-1000".into());
        }
        let w = &self.spectrum;
        match w.mode.as_str() {
            "monochromatic" => {
                range("wavelength_um", w.wavelength_um, 0.25, 4.)?;
                s.push("-1".into());
                s.push(w.wavelength_um.to_string());
            }
            "band" => {
                if !(2..=199).contains(&w.band) {
                    return Err(invalid("Built-in band must be 2..199"));
                }
                s.push(w.band.to_string());
            }
            "uniform" | "step" | "response" => {
                let count = grid_count(w.lower_um, w.upper_um)?;
                s.push(
                    match w.mode.as_str() {
                        "uniform" => "0",
                        "step" => "-2",
                        _ => "1",
                    }
                    .into(),
                );
                s.push(row([w.lower_um, w.upper_um]));
                if w.mode == "response" {
                    if w.response.len() != count
                        || w.response.iter().any(|x| !x.is_finite() || *x < 0.)
                        || !w.response.iter().any(|x| *x > 0.)
                    {
                        return Err(invalid(format!(
                            "Custom response requires {count} nonnegative 2.5 nm samples and positive support"
                        )));
                    }
                    s.push(row(w.response.iter().copied()));
                }
            }
            _ => {
                return Err(invalid(
                    "Spectrum mode must be monochromatic, uniform, step, response, or band",
                ));
            }
        }
        self.surface.render(&mut s, w)?;
        let c = &self.correction;
        match c.mode.as_str() {
            "none" => s.push("-1".into()),
            "radiance" | "reflectance" => {
                range("correction value", c.value, 0., 1e6)?;
                s.push("0".into());
                s.push(
                    (if c.mode == "reflectance" {
                        -c.value
                    } else {
                        c.value
                    })
                    .to_string(),
                );
            }
            _ => {
                return Err(invalid(
                    "Correction mode must be none, radiance, or reflectance",
                ));
            }
        }
        let p = &self.polarization;
        match p.mode.as_str() {
            "none" => s.push("0".into()),
            "stokes" => {
                range("surface Stokes Q", p.q, -1., 1.)?;
                range("surface Stokes U", p.u, -1., 1.)?;
                s.push("1".into());
                s.push(row([p.q, p.u]));
            }
            "vegetation" => {
                range("vegetation_fraction", p.vegetation_fraction, 0., 1.)?;
                s.push("2".into());
                s.push(p.vegetation_fraction.to_string());
            }
            "ocean" => {
                range("wind_speed_m_s", p.wind_speed_m_s, 0.001, 100.)?;
                range("wind_azimuth_deg", p.wind_azimuth_deg, 0., 360.)?;
                s.push("3".into());
                s.push(row([p.wind_speed_m_s, p.wind_azimuth_deg]));
            }
            _ => {
                return Err(invalid(
                    "Polarization mode must be none, stokes, vegetation, or ocean",
                ));
            }
        }
        Ok(s.join("\n") + "\n")
    }
}
impl Surface {
    fn reflectance(
        &self,
        r: &Reflectance,
        s: &mut Vec<String>,
        uniform: bool,
        w: &Spectrum,
    ) -> Result<(), ModelError> {
        match r.spectrum {
            0 => {
                range("surface reflectance", r.value, 0., 1.)?;
                s.push(r.value.to_string());
            }
            -1 => {
                for &v in &r.values {
                    range("surface spectrum", v, 0., 1.)?;
                }
                let count = if uniform {
                    let n = grid_count(r.lower_um, r.upper_um)?;
                    s.push(row([r.lower_um, r.upper_um]));
                    n
                } else {
                    match w.mode.as_str() {
                        "band" => crate::band_sample_count(w.band),
                        "monochromatic" => 1,
                        _ => grid_count(w.lower_um, w.upper_um)?,
                    }
                };
                if r.values.len() != count {
                    return Err(invalid(format!(
                        "Surface spectrum requires {count} samples"
                    )));
                }
                s.push(row(r.values.iter().copied()));
            }
            1..=4 => {}
            _ => return Err(invalid("Surface spectrum selector must be -1..4")),
        }
        Ok(())
    }
    fn render(&self, s: &mut Vec<String>, w: &Spectrum) -> Result<(), ModelError> {
        match self.mode.as_str() {
            "lambertian" => {
                s.extend(["0".into(), "0".into(), self.target.spectrum.to_string()]);
                self.reflectance(&self.target, s, true, w)?;
            }
            "heterogeneous" => {
                range("target radius km", self.radius_km, 0., 1000.)?;
                s.push("1".into());
                s.push(format!(
                    "{} {} {}",
                    self.target.spectrum, self.background.spectrum, self.radius_km
                ));
                self.reflectance(&self.target, s, false, w)?;
                self.reflectance(&self.background, s, false, w)?;
            }
            "brdf" => {
                if self.brdf_model > 11 {
                    return Err(invalid("BRDF model must be 0..11"));
                }
                s.extend(["0".into(), "1".into(), self.brdf_model.to_string()]);
                if self.brdf_model == 0 {
                    for table in [&self.sun_table, &self.view_table] {
                        if table.len() != 13 || table.iter().any(|r| r.len() != 10) {
                            return Err(invalid(
                                "BRDF tables must contain 13 azimuth rows, each with 10 descending zenith samples (85,80,..,0 deg)",
                            ));
                        }
                        for r in table {
                            finite("BRDF grid", r)?;
                            s.push(row(r.iter().copied()));
                        }
                    }
                    range("BRDF albedo", self.albedo, 0., 1.)?;
                    range("BRF", self.directional_reflectance, 0., 100.)?;
                    s.push(self.albedo.to_string());
                    s.push(self.directional_reflectance.to_string());
                } else {
                    let shapes: &[usize] = match self.brdf_model {
                        1 | 4 => &[4],
                        2 => &[3, 4, 3],
                        5 => &[2],
                        6 => &[4],
                        7 => &[2, 2, 3],
                        9 => &[4, 5],
                        _ => &[3],
                    };
                    if self.parameters.len() != shapes.iter().sum::<usize>() {
                        return Err(invalid(format!(
                            "BRDF {} requires parameter rows {shapes:?}",
                            self.brdf_model
                        )));
                    }
                    finite("BRDF parameters", &self.parameters)?;
                    let mut offset = 0;
                    for &n in shapes {
                        if (self.brdf_model == 2 && offset == 0)
                            || (self.brdf_model == 7 && offset == 0)
                        {
                            if self.parameters[offset..offset + n]
                                .iter()
                                .any(|x| x.fract() != 0.)
                            {
                                return Err(invalid("BRDF option selectors must be integers"));
                            }
                            s.push(
                                self.parameters[offset..offset + n]
                                    .iter()
                                    .map(|x| (*x as i32).to_string())
                                    .collect::<Vec<_>>()
                                    .join(" "),
                            );
                        } else {
                            s.push(row(self.parameters[offset..offset + n].iter().copied()));
                        }
                        offset += n;
                    }
                }
            }
            _ => {
                return Err(invalid(
                    "Surface mode must be lambertian, heterogeneous, or brdf",
                ));
            }
        }
        Ok(())
    }
}
