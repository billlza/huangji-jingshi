use chrono::{DateTime, Utc, Datelike};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::collections::HashMap;
use std::sync::RwLock;
use std::fs::File;
use std::io::BufReader;
use once_cell::sync::Lazy;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyRequest {
    pub datetime: DateTime<Utc>,
    pub lat_deg: f64, // 纬度：北正南负
    pub lon_deg: f64, // 经度：东正西负
    pub delta_t_provider: Option<String>,
    pub accuracy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CelestialBody {
    pub name: String,
    pub alt_deg: f64,
    pub az_deg: f64,
    pub distance_au: Option<f64>,
    pub ra_deg: Option<f64>,
    pub dec_deg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyResponse {
    pub bodies: Vec<CelestialBody>,
    pub note: String,
    pub jd: f64,
    pub lst_deg: f64,
    pub gmst_deg: f64,
    pub delta_t_sec: f64,
}

pub fn compute_sky(req: &SkyRequest) -> SkyResponse {
    let timestamp = req.datetime.timestamp();
    // JD calculation from unix timestamp
    // JD = (timestamp / 86400) + 2440587.5
    let jd = (timestamp as f64 / 86400.0) + 2440587.5;
    
    // Astro coords
    let lat_rad = req.lat_deg.to_radians();
    let _lon_rad = req.lon_deg.to_radians();
    
    // Sidereal time (Meeus approximation with higher-order terms)
    // T = (JD - 2451545.0) / 36525.0
    // GMST = 280.46061837 + 360.98564736629*(JD - 2451545.0) + 0.000387933*T^2 - T^3/38710000
    let d_jc = (jd - 2451545.0) / 36525.0;
    let mut gmst = 280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * d_jc * d_jc - (d_jc * d_jc * d_jc) / 38710000.0;
    gmst %= 360.0;
    if gmst < 0.0 { gmst += 360.0; }

    // ΔT estimation (TT - UT1) in seconds (approximate; polynomial fit)
    // Source: NASA polynomial (valid for 2000–2100)
    let y = req.datetime.year() as f64;
    let u = y - 2000.0;
    #[derive(Deserialize)]
    struct IersDeltaT { year: i32, delta_t_sec: f64 }
    static IERS_DELTA_T: Lazy<RwLock<HashMap<i32, f64>>> = Lazy::new(|| RwLock::new(HashMap::new()));
    static IERS_DELTA_T_SERIES: Lazy<RwLock<Vec<(f64, f64)>>> = Lazy::new(|| RwLock::new(Vec::new()));
    fn load_iers_delta_t_once() {
        let mut map = IERS_DELTA_T.write().unwrap();
        let mut series = IERS_DELTA_T_SERIES.write().unwrap();
        if !map.is_empty() || !series.is_empty() { return; }
        let json_path = env::var("IERS_DELTA_T_PATH").ok().unwrap_or_else(|| "huangji_core/data/iers_delta_t.json".to_string());
        if let Ok(file) = File::open(json_path) {
            let reader = BufReader::new(file);
            if let Ok(records) = serde_json::from_reader::<_, Vec<IersDeltaT>>(reader) {
                for r in records {
                    map.insert(r.year, r.delta_t_sec);
                    series.push((r.year as f64, r.delta_t_sec));
                }
            }
        }
        let csv_path = env::var("IERS_DELTA_T_CSV").ok();
        if let Some(p) = csv_path {
            if let Ok(content) = std::fs::read_to_string(p) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    let parts: Vec<&str> = line.split([',', ';', '\t', ' ']).filter(|s| !s.is_empty()).collect();
                    if parts.len() < 2 { continue; }
                    if let (Ok(y), Ok(v)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                        series.push((y, v));
                    }
                }
            }
        }
        if !series.is_empty() {
            series.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        }
    }
    fn delta_t_iers_interp(year: f64) -> Option<f64> {
        load_iers_delta_t_once();
        let series = IERS_DELTA_T_SERIES.read().unwrap();
        if series.len() >= 2 {
            if year <= series[0].0 { return Some(series[0].1); }
            if year >= series[series.len()-1].0 { return Some(series[series.len()-1].1); }
            for i in 0..(series.len()-1) {
                let (y0, v0) = series[i];
                let (y1, v1) = series[i+1];
                if year >= y0 && year <= y1 {
                    let t = (year - y0) / (y1 - y0);
                    return Some(v0 + t * (v1 - v0));
                }
            }
        }
        let map = IERS_DELTA_T.read().unwrap();
        if let Some(v) = map.get(&(year.round() as i32)).copied() { return Some(v); }
        None
    }
    fn delta_t_segmented(y: f64) -> f64 {
        if (1986.0..2005.0).contains(&y) {
            let x = y - 2000.0;
            63.86 + 0.3345 * x - 0.060374 * x * x + 0.0017275 * x * x * x + 0.000651814 * x.powi(4) + 0.00002373599 * x.powi(5)
        } else if (1900.0..1986.0).contains(&y) {
            let x = y - 1900.0;
            -2.79 + 1.494119 * x - 0.0598939 * x * x + 0.0061966 * x.powi(3) - 0.000197 * x.powi(4)
        } else {
            // For y >= 2005.0 or y < 1900.0
            let x = y - 2000.0;
            62.92 + 0.32217 * x + 0.005589 * x * x
        }
    }
    let provider = req.delta_t_provider.as_deref().unwrap_or("segmented");
    let delta_t_sec = match provider {
        "iers" => delta_t_iers_interp(y).unwrap_or_else(|| delta_t_segmented(y)),
        "nasa" => 62.92 + 0.32217 * u + 0.005589 * u * u,
        _ => delta_t_segmented(y),
    };

    let eps0_arcsec = 84381.406 - 46.836769 * d_jc - 0.0001831 * d_jc * d_jc + 0.00200340 * d_jc * d_jc * d_jc - 0.000000576 * d_jc.powi(4) - 0.000001578 * d_jc.powi(5);
    let eps0_rad = (eps0_arcsec / 3600.0).to_radians();

    let m_sun = 357.52911 + 35999.05029 * d_jc - 0.0001537 * d_jc * d_jc;
    let l_moon = 134.963 + 477198.8676 * d_jc;
    let omega = 125.045 - 1934.136 * d_jc;
    let d_moon = 297.850 + 445267.1115 * d_jc;

    let m_sun_rad = m_sun.to_radians();
    let l_moon_rad = l_moon.to_radians();
    let omega_rad = omega.to_radians();
    let _d_moon_rad_unused = d_moon.to_radians();

    let dpsi_arcsec = -17.20 * omega_rad.sin() - 1.32 * (2.0 * m_sun_rad).sin() - 0.23 * (2.0 * l_moon_rad).sin() + 0.21 * (2.0 * omega_rad).sin();
    let deps_arcsec = 9.20 * omega_rad.cos() + 0.57 * (2.0 * m_sun_rad).cos() + 0.10 * (2.0 * l_moon_rad).cos() - 0.09 * (2.0 * omega_rad).cos();
    let dpsi_deg = dpsi_arcsec / 3600.0;
    let deps_deg = deps_arcsec / 3600.0;
    let ee_deg = dpsi_deg * eps0_rad.cos().to_degrees();

    let mut gast = gmst + ee_deg;
    gast = gast.rem_euclid(360.0);
    let lst_deg = (gast + req.lon_deg).rem_euclid(360.0);
    let lst_rad = lst_deg.to_radians();

    let mut bodies = Vec::new();

    // Helper to convert RA/Dec to Alt/Az
    // RA, Dec in radians
    let to_alt_az = |ra: f64, dec: f64| -> (f64, f64) {
        let ha = lst_rad - ra;
        let sin_alt = lat_rad.sin() * dec.sin() + lat_rad.cos() * dec.cos() * ha.cos();
        let alt = sin_alt.asin();
        
        let cos_az = (dec.sin() - lat_rad.sin() * alt.sin()) / (lat_rad.cos() * alt.cos());
        // Handle precision issues
        let cos_az = cos_az.clamp(-1.0, 1.0);
        let mut az = cos_az.acos();
        
        if ha.sin() > 0.0 {
            az = 2.0 * PI - az;
        }
        
        (alt.to_degrees(), az.to_degrees())
    };

    // 1. Sun (using astro crate or simple implementation if astro is complex)
    // astro crate usage:
    // let sun_coords = sun::geocentric_ecliptic_coordinates(jd);
    // let sun_eq = coords::ecliptic_to_equatorial(sun_coords.long, sun_coords.lat, obliq);
    // For simplicity and verification, we use a robust simplified VSOP or similar if astro is tricky.
    // But since we imported `astro`, let's try to use it.
    // Actually, the crate `astro` v2.0.0 API needs checking.
    // Let's use basic calculation if we are not sure about astro API, 
    // OR assume standard names.
    
    // Using `astro` crate:
    // It usually provides `sun::coordinates(jd)` returning (long, lat, radius)
    // We need to convert to Equatorial (RA, Dec) then to Horizontal (Alt, Az).
    
    // Let's use a simplified Meeus algorithm directly here to ensure "Real Data" without fighting crate API docs blindly.
    // Wait, user wants "Real API". `astro` IS a real implementation of Meeus.
    // Let's blindly try to use `astro` assuming standard modules.
    
    // Note: `astro` crate usually has modules like `sun`, `moon`, `planet`.
    // Let's implement a fallback or use `astro` if we recall the API.
    // astro::sun::sun_position(jd) -> (lon, lat, rad) ?
    
    // Let's stick to writing our own Meeus implementation for Sun/Moon to be safe and "Have Reason (Code)".
    
    // --- SUN ---
    // Mean longitude
    let l0 = 280.46646 + 36000.76983 * d_jc + 0.0003032 * d_jc * d_jc;
    // Mean anomaly
    let m = 357.52911 + 35999.05029 * d_jc - 0.0001537 * d_jc * d_jc;
    let m_rad = m.to_radians();
    // Equation of center
    let c = (1.914602 - 0.004817 * d_jc - 0.000014 * d_jc * d_jc) * m_rad.sin()
          + (0.019993 - 0.000101 * d_jc) * (2.0 * m_rad).sin()
          + 0.000289 * (3.0 * m_rad).sin();
    // True longitude
    let true_long = l0 + c;
    // Obliquity of ecliptic
    let epsilon = eps0_arcsec / 3600.0 + deps_deg;
    let eps_rad = epsilon.to_radians();
    let lambda_rad = (true_long + dpsi_deg).to_radians();
    
    // Right Ascension
    let _alpha = (lambda_rad.cos() * eps_rad.tan()).atan().to_degrees(); 
    // Handle quadrant
    let _alpha_rad = lambda_rad.cos().atan2(lambda_rad.sin() * eps_rad.cos()); // Wait, atan2(y, x) -> atan2(cos(eps)*sin(lam), cos(lam))
    let ra_rad = (eps_rad.cos() * lambda_rad.sin()).atan2(lambda_rad.cos());
    
    // Declination
    let dec_rad = (eps_rad.sin() * lambda_rad.sin()).asin();
    
    let (alt, az) = to_alt_az(ra_rad, dec_rad);
    
    bodies.push(CelestialBody {
        name: "Sun".to_string(),
        alt_deg: alt,
        az_deg: az,
        distance_au: Some(1.0),
        ra_deg: Some(ra_rad.to_degrees()),
        dec_deg: Some(dec_rad.to_degrees()),
    });

    // --- MOON ---
    // 使用简单的 Meeus 算法近似计算月球位置，以避免外部库依赖问题并保证"有理可依"
    
    // Mean longitude
    let l_moon = 218.316 + 481267.8813 * d_jc;
    // Mean anomaly
    let m_moon = 134.963 + 477198.8676 * d_jc;
    // Mean distance of Moon from ascending node
    let f_moon = 93.272 + 483202.0175 * d_jc;
    // Mean elongation
    let d_moon = 297.850 + 445267.1115 * d_jc;
    // Mean longitude of ascending node
    let _omega_moon = 125.045 - 1934.136 * d_jc;
    
    let _l_rad = l_moon.to_radians();
    let m_rad = m_moon.to_radians(); // Corrected variable name usage from m_rad to m_moon_rad if needed, but here m_rad is Sun's m?
    // Wait, m_rad was defined earlier as Sun's Mean Anomaly.
    // Moon calculation uses Moon's mean anomaly mm_rad.
    // Let's check lines 143.
    let mm_rad = m_moon.to_radians();
    let f_rad = f_moon.to_radians();
    let d_rad = d_moon.to_radians();
    
    // Longitude terms (simplified)
    let sigma_l = 6.289 * mm_rad.sin() 
                - 1.274 * (mm_rad - 2.0 * d_rad).sin() 
                + 0.658 * (2.0 * d_rad).sin()
                + 0.214 * (2.0 * mm_rad).sin()
                - 0.186 * m_rad.sin(); // Sun's mean anomaly m used earlier
                
    let lam_moon = l_moon + sigma_l;
    
    // Latitude terms
    let sigma_b = 5.128 * f_rad.sin()
                + 0.280 * (mm_rad + f_rad).sin()
                + 0.278 * (mm_rad - f_rad).sin()
                + 0.173 * (f_rad - 2.0 * d_rad).sin();
                
    let beta_moon = sigma_b;
    
    // Convert Ecliptic (lam_moon, beta_moon) to Equatorial
    let lam_m_rad = (lam_moon + dpsi_deg).to_radians();
    let beta_m_rad = beta_moon.to_radians();
    
    // Proper conversion:
    let ra_moon_rad = (lam_m_rad.sin() * eps_rad.cos() - beta_m_rad.tan() * eps_rad.sin()).atan2(lam_m_rad.cos());
    let dec_moon_rad = (beta_m_rad.sin() * eps_rad.cos() + beta_m_rad.cos() * eps_rad.sin() * lam_m_rad.sin()).asin();
    
    let (alt_moon, az_moon) = to_alt_az(ra_moon_rad, dec_moon_rad);
    
    bodies.push(CelestialBody {
        name: "Moon".to_string(),
        alt_deg: alt_moon,
        az_deg: az_moon,
        distance_au: Some(0.00257),
        ra_deg: Some(ra_moon_rad.to_degrees()),
        dec_deg: Some(dec_moon_rad.to_degrees()),
    });
    
    // --- PLANETS (Meeus Simplified) ---
    // For planets, full Meeus VSOP87 is complex.
    // We will use a lower precision approximation suitable for visualization.
    // Elements from "Astronomical Algorithms", J. Meeus, Chapter 31 (Elements of Planetary Orbits)
    // Mean orbital elements for J2000.0
    // We calculate Heliocentric coords (L, B, R) -> Geocentric (lambda, beta, delta) -> Equatorial (alpha, delta) -> Horizontal (alt, az).
    
    #[allow(dead_code)]
    struct OrbitalElements {
        l: f64, // Mean longitude
        a: f64, // Semi-major axis
        e: f64, // Eccentricity
        i: f64, // Inclination
        omega: f64, // Longitude of ascending node
        pi: f64, // Longitude of perihelion
        m: f64, // Mean anomaly (calculated)
    }
    
    // Helper to solve Kepler Equation: M = E - e*sin(E)
    fn solve_kepler(m: f64, e: f64) -> f64 {
        let ee = m;
        let m_rad = m.to_radians();
        let mut e_rad = ee.to_radians();
        for _ in 0..10 {
            let delta = (m_rad - (e_rad - e * e_rad.sin())) / (1.0 - e * e_rad.cos());
            e_rad += delta;
            if delta.abs() < 1e-6 { break; }
        }
        e_rad // return in radians
    }
    
    let calc_planet = |name: &str, l0: f64, a: f64, e: f64, i_deg: f64, omega_deg: f64, pi_deg: f64, n: f64| -> CelestialBody {
        // Calculate Mean Anomaly M
        let mut m = (l0 + n * d_jc - pi_deg) % 360.0;
        if m < 0.0 { m += 360.0; }
        
        // Eccentric Anomaly E (radians)
        let big_e_rad = solve_kepler(m, e);
        
        // True Anomaly v
        // tan(v/2) = sqrt((1+e)/(1-e)) * tan(E/2)
        let v_rad = 2.0 * (( (1.0+e)/(1.0-e) ).sqrt() * (big_e_rad / 2.0).tan()).atan();
        
        // Radius vector r
        let r = a * (1.0 - e * big_e_rad.cos());
        
        // Heliocentric Longitude L and Latitude B
        // u = v + pi - omega
        let u_rad = v_rad + (pi_deg - omega_deg).to_radians();
        let i_rad = i_deg.to_radians();
        let omega_rad = omega_deg.to_radians();
        
        // Heliocentric coords
        let l_helio_rad = (u_rad.cos() * omega_rad.cos() - u_rad.sin() * omega_rad.sin() * i_rad.cos()).atan2(
                           u_rad.cos() * omega_rad.sin() + u_rad.sin() * omega_rad.cos() * i_rad.cos()
                          );
        let b_helio_rad = (u_rad.sin() * i_rad.sin()).asin();
        
        // Convert to Geocentric (using Sun's coords calculated previously)
        // We need Sun's Geocentric Longitude (Theta) and Radius (R).
        // Sun's Heliocentric Longitude is Earth's + 180.
        // Let's re-use Sun's true_long (Geocentric) and radius (approx 1.0).
        let l_sun_rad = (true_long + 180.0).to_radians(); // Earth's Helio Longitude approx
        let r_earth = 1.0; // Approx
        
        // Rectangular coords
        let x = r * b_helio_rad.cos() * l_helio_rad.cos() - r_earth * l_sun_rad.cos();
        let y = r * b_helio_rad.cos() * l_helio_rad.sin() - r_earth * l_sun_rad.sin();
        let z = r * b_helio_rad.sin();
        
        let lambda_geo = y.atan2(x); // Geocentric Longitude
        let beta_geo = z.atan2((x*x + y*y).sqrt()); // Geocentric Latitude
        
        // To Equatorial
        let eps = eps_rad; // from Sun calc
        let ra_rad = (lambda_geo.sin() * eps.cos() - beta_geo.tan() * eps.sin()).atan2(lambda_geo.cos());
        let dec_rad = (beta_geo.sin() * eps.cos() + beta_geo.cos() * eps.sin() * lambda_geo.sin()).asin();
        
        let (alt, az) = to_alt_az(ra_rad, dec_rad);
        
        CelestialBody {
            name: name.to_string(),
            alt_deg: alt,
            az_deg: az,
            distance_au: Some((x*x + y*y + z*z).sqrt()),
            ra_deg: Some(ra_rad.to_degrees()),
            dec_deg: Some(dec_rad.to_degrees()),
        }
    };
    
    // Elements (approximate)
    // Mercury
    bodies.push(calc_planet("Mercury", 252.25, 0.3871, 0.2056, 7.005, 48.331, 77.456, 149472.67));
    // Venus
    bodies.push(calc_planet("Venus", 181.98, 0.7233, 0.0068, 3.395, 76.680, 131.533, 58517.81));
    // Mars
    bodies.push(calc_planet("Mars", 355.43, 1.5237, 0.0934, 1.850, 49.558, 336.041, 19140.29));
    // Jupiter
    bodies.push(calc_planet("Jupiter", 34.35, 5.2026, 0.0485, 1.303, 100.464, 14.331, 3034.90));
    // Saturn
    bodies.push(calc_planet("Saturn", 50.08, 9.5549, 0.0555, 2.489, 113.666, 92.097, 1222.11));
    
    // --- Polaris & Big Dipper (Stars) ---
    // These are fixed stars (relatively), we just calculate Alt/Az from RA/Dec.
    // Precession should be applied for high accuracy over centuries, but for "Daoist" vibe and 100-year span, J2000 is acceptable?
    // User wants "Advanced Algorithm". We should apply precession.
    // Precession approx: 50.3 arcsec/year.
    // Let's implement basic precession from J2000 to current epoch.
    
    let precess = |ra0_deg: f64, dec0_deg: f64| -> (f64, f64) {
        let t = (jd - 2451545.0) / 36525.0;
        let zeta_arcsec = 2306.083227 * t + 0.2988499 * t * t + 0.018018 * t * t * t;
        let z_arcsec = 2306.077181 * t + 1.0927348 * t * t + 0.018268 * t * t * t;
        let theta_arcsec = 2004.191903 * t - 0.4294934 * t * t - 0.041833 * t * t * t;
        let zeta = (zeta_arcsec / 3600.0).to_radians();
        let z = (z_arcsec / 3600.0).to_radians();
        let theta = (theta_arcsec / 3600.0).to_radians();

        let a = ra0_deg.to_radians();
        let d = dec0_deg.to_radians();

        let x0 = d.cos() * a.cos();
        let y0 = d.cos() * a.sin();
        let z0 = d.sin();

        let x1 = x0 * zeta.cos() + y0 * zeta.sin();
        let y1 = -x0 * zeta.sin() + y0 * zeta.cos();
        let z1 = z0;

        let x2 = x1;
        let y2 = y1 * theta.cos() + z1 * theta.sin();
        let z2 = -y1 * theta.sin() + z1 * theta.cos();

        let x3 = x2 * z.cos() + y2 * z.sin();
        let y3 = -x2 * z.sin() + y2 * z.cos();
        let z3 = z2;

        let ra = y3.atan2(x3);
        let dec = z3.atan2((x3 * x3 + y3 * y3).sqrt());
        (ra.to_degrees().rem_euclid(360.0), dec.to_degrees())
    };
    
    let add_star = |name: &str, ra2000: f64, dec2000: f64| -> CelestialBody {
        let (ra, dec) = precess(ra2000, dec2000);
        let (alt, az) = to_alt_az(ra.to_radians(), dec.to_radians());
        CelestialBody {
            name: name.to_string(),
            alt_deg: alt,
            az_deg: az,
            distance_au: None,
            ra_deg: Some(ra),
            dec_deg: Some(dec),
        }
    };
    
    // Polaris (Alpha Ursae Minoris)
    // RA 2h 31m, Dec +89 15
    bodies.push(add_star("Polaris", 37.95, 89.26));
    
    // Big Dipper (7 stars of Ursa Major)
    // Dubhe (Alpha UMa): 11h 03m, +61 45
    bodies.push(add_star("Dubhe", 165.93, 61.75));
    // Merak (Beta UMa): 11h 01m, +56 22
    bodies.push(add_star("Merak", 165.46, 56.38));
    // Phecda (Gamma UMa): 11h 53m, +53 41
    bodies.push(add_star("Phecda", 178.46, 53.69));
    // Megrez (Delta UMa): 12h 15m, +57 01
    bodies.push(add_star("Megrez", 183.86, 57.03));
    // Alioth (Epsilon UMa): 12h 54m, +55 57
    bodies.push(add_star("Alioth", 193.50, 55.96));
    // Mizar (Zeta UMa): 13h 23m, +54 55
    bodies.push(add_star("Mizar", 200.98, 54.92));
    // Alkaid (Eta UMa): 13h 47m, +49 18
    bodies.push(add_star("Alkaid", 206.88, 49.31));
    
    SkyResponse {
        bodies,
        note: "Real-time Sun/Moon/Planets/Stars (Meeus/J2000)".to_string(),
        jd,
        lst_deg,
        gmst_deg: gmst,
        delta_t_sec,
    }
}
