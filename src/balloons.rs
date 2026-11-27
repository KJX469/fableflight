//! Hot-air balloons: the morning (and, here and there, evening) flights of
//! the places known for them, drifting with the day's wind.
//!
//! The models come from Blender (assets/blender/balloons.py,
//! assets/balloons.bin.gz): an envelope in eight fictional colour schemes,
//! near and far, its skirt and flying wires, two rigs (a sport balloon after
//! the Cameron O-105, a passenger balloon after the Ultramagic N-425, whose
//! envelope is the same one scaled by `LARGE_SCALE`), and the burner flame.
//! They are drawn as instanced solids (inst.wgsl, by material: ripstop
//! nylon with its seams and load tapes, woven wicker, suede, rope, steel):
//! the cloth lets the sun through and glows from inside while the burner
//! is lit, and each burner's flame is drawn as moving fire. About one
//! balloon in three trails a flag on a line under its basket.
//!
//! Each site has a schedule for the day, made from the site and the date (so
//! it is the same every time the day is flown): how many balloons, when each
//! takes off (from about a quarter of an hour before sunrise to three
//! quarters of an hour after, and for some sites in the two hours before
//! sunset), from which launch field, how high it cruises and for how long.
//! A balloon only takes off in calm air (under about 5 m/s at the ground)
//! without rain or storms; once up, it goes where the wind at its height
//! takes it, climbs and descends with the burner, and lands about an hour
//! later. Arriving in the middle of the flights, the balloons already up are
//! flown forward to where they would be now.
//!
//! Sites and their figures (balloons on a good morning in season; the
//! festivals' dates and numbers are typical ones, approximate):
//! - Göreme, Cappadocia: the authorities allow up to about 150 balloons a
//!   morning, nearly all large passenger balloons, all year;
//! - Albuquerque: the International Balloon Fiesta, nine days from the first
//!   Saturday of October, some 500 balloons (more than the game draws); a few
//!   commercial flights the rest of the year;
//! - Château-d'Œx: the Festival International de Ballons, late January;
//! - Chambley-Bussières: the Lorraine Mondial Air Ballons, late July in odd
//!   years, the largest gathering in Europe; flights through the summer;
//! - Bristol: the International Balloon Fiesta, August;
//! - Canberra: the Balloon Spectacular, March;
//! - Luxor (West Bank), Bagan (October to March), Teotihuacan, the Masai
//!   Mara and the Napa Valley: daily passenger flights;
//! - the Arve valley under the Mont Blanc and Flanders (evening flights,
//!   May to September): a handful of balloons.
//!
//! Counts and seasons are estimates from operators' and festivals' published
//! figures and press reports, not data.

use crate::geo;
use crate::graphics::Level;
use crate::render::{Inst, MeshVertex};
use glam::{DVec3, Mat4};
use std::sync::OnceLock;

// ------------------------------------------------------------------- models

/// The colour schemes, as named in balloons.py.
pub const SCHEMES: [&str; 8] = ["rainbow", "harlequin", "bands", "chevron", "blocks", "wave", "checks", "stripes"];
/// The pieces after the envelopes (near, then far), in mesh-id order.
/// How many meshes the balloons add to the instanced solids.
pub const N_MESHES: u32 = (2 * SCHEMES.len() + PARTS.len()) as u32;
const PARTS: [&str; 11] = ["skirt", "wires sport", "wires large", "rig sport", "rig large", "rig far", "flame", "flag fr", "flag be", "flag ch", "flag de"];
/// First mesh id of the balloons in the instanced-solid library (main.rs `solids`).
pub const BASE: u32 = crate::birds::BASE + (crate::birds::Kind::ALL.len() * crate::birds::PIECES.len()) as u32;
const ENV: u32 = BASE;
const ENV_FAR: u32 = BASE + SCHEMES.len() as u32;
const SKIRT: u32 = BASE + 2 * SCHEMES.len() as u32;
const WIRES_SPORT: u32 = SKIRT + 1;
const WIRES_LARGE: u32 = SKIRT + 2;
const RIG_SPORT: u32 = SKIRT + 3;
const RIG_LARGE: u32 = SKIRT + 4;
const RIG_FAR: u32 = SKIRT + 5;
const FLAME: u32 = SKIRT + 6;
/// Flags on a line under the basket: France, Belgium, Switzerland, Germany.
const FLAG: u32 = SKIRT + 7;
const FLAGS: u32 = 4;
/// Where a flag's line is tied, from the middle of the basket floor (x, y
/// up, z): a corner under the rim, sport and passenger basket.
const FLAG_AT: [[f64; 3]; 2] = [[0.60, 0.95, 0.75], [1.08, 1.10, 2.28]];
/// The burners lit within this are counted for the log (main.rs).
pub const FLAME_JET: f64 = 600.0;
/// Each burner's place on its frame (x, z), sport and passenger rig.
const BURNERS: [&[(f64, f64)]; 2] = [&[(-0.17, 0.0), (0.17, 0.0)], &[(-0.2, -0.2), (0.2, -0.2), (-0.2, 0.2), (0.2, 0.2)]];

/// The passenger balloon's envelope is the sport one this much bigger.
pub const LARGE_SCALE: f64 = 1.6;
/// The envelope, mouth to crown, and its radius at its widest (balloons.py: H, R).
pub const ENV_H: f64 = 19.5;
pub const ENV_R: f64 = 9.3;
/// Above the basket floor: the mouth and the burner nozzle (balloons.py).
const MOUTH: [f64; 2] = [4.55, 6.2];
const NOZZLE: [f64; 2] = [2.82, 3.35];

pub struct Models {
    pub meshes: Vec<(Vec<MeshVertex>, Vec<u32>)>,
}

pub fn models() -> &'static Models {
    static M: OnceLock<Models> = OnceLock::new();
    M.get_or_init(|| {
        let objs = crate::aircraft::fa18::load(include_bytes!("../assets/balloons.bin.gz"), "balloons");
        let names: Vec<String> = SCHEMES.iter().map(|s| format!("env {s}")).chain(SCHEMES.iter().map(|s| format!("env {s} far"))).chain(PARTS.iter().map(|s| s.to_string())).collect();
        let meshes = names
            .iter()
            .map(|name| {
                let o = objs.get(name).unwrap_or_else(|| panic!("balloons model: no piece \"{name}\""));
                // (Colour alpha: inst.wgsl's material, from balloons.py's
                // MAT_CODE: nylon, seams, tapes, Nomex, wicker, leather,
                // rope, steel, the flame; 1 plain diffuse.)
                let v = o.mesh.v.iter().map(|v| MeshVertex { pos: v.pos, nrm: v.nrm, col: [v.col[0], v.col[1], v.col[2], if v.mat[0] > 0.5 { v.mat[0] } else { 1.0 }] }).collect();
                (v, o.mesh.i.clone())
            })
            .collect();
        Models { meshes }
    })
}

/// How many balloons at most, by setting.
pub fn cap(level: Level) -> usize {
    match level {
        Level::Off => 0,
        Level::Low => 25,
        Level::Medium => 80,
        Level::High | Level::Ultra => 220,
    }
}

// -------------------------------------------------------------------- sites

pub struct Site {
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
    /// ground height (m), for when the terrain is not loaded
    pub elev: f64,
    /// launch fields lie within this (km)
    pub spread_km: f64,
    /// balloons on a good morning in season
    pub daily: (u32, u32),
    /// share of large passenger balloons
    pub large: f64,
    /// months flown (bit 0 January)
    pub months: u16,
    /// also flights in the evening (two hours before sunset)
    pub evening: bool,
    /// only in the evening
    pub evening_only: bool,
    /// festivals: first and last day of the year, balloons a morning, odd years only
    pub festivals: &'static [(u32, u32, u32, bool)],
}

const ALL_YEAR: u16 = 0xFFF;
const fn months(a: u32, b: u32) -> u16 {
    // a..=b, 1-based, wrapping over the new year.
    let mut m = 0u16;
    let mut i = a;
    loop {
        m |= 1 << (i - 1);
        if i == b {
            break;
        }
        i = if i == 12 { 1 } else { i + 1 };
    }
    m
}

pub static SITES: &[Site] = &[
    Site { name: "Cappadocia", lat: 38.6431, lon: 34.8289, elev: 1100.0, spread_km: 6.0, daily: (70, 150), large: 0.95, months: ALL_YEAR, evening: false, evening_only: false, festivals: &[] },
    Site { name: "Albuquerque", lat: 35.1966, lon: -106.5975, elev: 1550.0, spread_km: 3.0, daily: (2, 7), large: 0.25, months: ALL_YEAR, evening: false, evening_only: false, festivals: &[(277, 285, 500, false)] },
    Site { name: "Château-d'Œx", lat: 46.4747, lon: 7.1358, elev: 960.0, spread_km: 2.0, daily: (1, 3), large: 0.05, months: months(12, 3), evening: false, evening_only: false, festivals: &[(24, 32, 70, false)] },
    Site { name: "Chambley", lat: 49.0306, lon: 5.8786, elev: 260.0, spread_km: 3.0, daily: (2, 6), large: 0.1, months: months(4, 10), evening: true, evening_only: false, festivals: &[(206, 215, 350, true)] },
    Site { name: "Bristol", lat: 51.4436, lon: -2.6435, elev: 70.0, spread_km: 2.0, daily: (1, 3), large: 0.15, months: months(4, 9), evening: true, evening_only: false, festivals: &[(220, 223, 100, false)] },
    Site { name: "Canberra", lat: -35.2984, lon: 149.1311, elev: 580.0, spread_km: 3.0, daily: (1, 4), large: 0.3, months: ALL_YEAR, evening: false, evening_only: false, festivals: &[(68, 76, 40, false)] },
    Site { name: "Luxor", lat: 25.7200, lon: 32.6020, elev: 80.0, spread_km: 3.0, daily: (15, 40), large: 0.9, months: ALL_YEAR, evening: false, evening_only: false, festivals: &[] },
    Site { name: "Bagan", lat: 21.1717, lon: 94.8585, elev: 60.0, spread_km: 4.0, daily: (10, 25), large: 0.9, months: months(10, 3), evening: false, evening_only: false, festivals: &[] },
    Site { name: "Teotihuacan", lat: 19.6900, lon: -98.8600, elev: 2300.0, spread_km: 4.0, daily: (15, 35), large: 0.7, months: ALL_YEAR, evening: false, evening_only: false, festivals: &[] },
    Site { name: "Masai Mara", lat: -1.4900, lon: 35.1400, elev: 1550.0, spread_km: 8.0, daily: (6, 14), large: 0.8, months: ALL_YEAR, evening: false, evening_only: false, festivals: &[] },
    Site { name: "Napa Valley", lat: 38.4000, lon: -122.3600, elev: 30.0, spread_km: 6.0, daily: (3, 9), large: 0.3, months: ALL_YEAR, evening: false, evening_only: false, festivals: &[] },
    Site { name: "Arve valley", lat: 46.0700, lon: 6.4100, elev: 450.0, spread_km: 5.0, daily: (2, 5), large: 0.1, months: months(4, 10), evening: true, evening_only: false, festivals: &[] },
    Site { name: "Flanders", lat: 51.0540, lon: 3.7250, elev: 10.0, spread_km: 12.0, daily: (2, 7), large: 0.05, months: months(5, 9), evening: true, evening_only: true, festivals: &[] },
];

/// The sun's hour angle at rising (h, local solar time from noon), with the
/// game's sun (main.rs `sun_dir`) and the usual 0.83° for refraction and the
/// sun's size; None in polar day or night.
pub fn half_day(lat_deg: f64, doy: f64) -> Option<f64> {
    let decl = (-23.44f64).to_radians() * (std::f64::consts::TAU * (doy + 10.0) / 365.25).cos();
    let lat = lat_deg.to_radians();
    let c = ((-0.83f64).to_radians().sin() - lat.sin() * decl.sin()) / (lat.cos() * decl.cos());
    (c.abs() <= 1.0).then(|| c.acos().to_degrees() / 15.0)
}

// ------------------------------------------------------------------ flights

#[derive(Clone, Copy, Debug)]
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.next()
    }
}

/// One balloon's day, as planned.
#[derive(Clone, Copy, Debug)]
pub struct Plan {
    /// take-off, local solar hour
    pub launch: f64,
    /// launch point
    pub lat: f64,
    pub lon: f64,
    pub large: bool,
    pub scheme: u8,
    /// cruising heights above the ground (m): the first and the second half
    pub cruise: (f64, f64),
    /// flight time (s)
    pub minutes: f64,
    pub seed: u64,
}

/// The day's flights at a site (deterministic: the same date, the same flights).
pub fn schedule(site: &Site, si: usize, doy: u32, year_odd: bool) -> Vec<Plan> {
    let month = month_of(doy);
    let fest = site.festivals.iter().find(|f| doy >= f.0 && doy <= f.1 && (!f.3 || year_odd)).map(|f| f.2);
    if fest.is_none() && site.months & (1 << (month - 1)) == 0 {
        return Vec::new();
    }
    let mut rng = Rng(0x2545_F491_4F6C_DD1D ^ (si as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (doy as u64 + 1).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    for _ in 0..4 {
        rng.next();
    }
    let Some(half) = half_day(site.lat, doy as f64) else { return Vec::new() };
    let (rise, set) = (12.0 - half, 12.0 + half);
    let n = match fest {
        Some(n) => n,
        None => rng.range(site.daily.0 as f64, site.daily.1 as f64 + 0.999) as u32,
    };
    // Launch fields: a few, the balloons gathered on them.
    let fields: Vec<(f64, f64)> = (0..(1 + n / 12).min(8))
        .map(|_| {
            let (a, r) = (rng.range(0.0, std::f64::consts::TAU), site.spread_km * 1000.0 * rng.next().sqrt());
            let dlat = r * a.cos() / geo::R;
            let dlon = r * a.sin() / (geo::R * site.lat.to_radians().cos().max(0.1));
            (site.lat + dlat.to_degrees(), site.lon + dlon.to_degrees())
        })
        .collect();
    let mut out = Vec::with_capacity(n as usize);
    for i in 0..n {
        let evening = site.evening_only || (site.evening && fest.is_none() && rng.next() < 0.35);
        let launch = if evening { set - rng.range(1.6, 2.4) } else { rise + rng.range(-0.25, 0.75) };
        let f = fields[(i as usize) % fields.len()];
        // (Some tens of metres apart on the field.)
        let (a, r) = (rng.range(0.0, std::f64::consts::TAU), rng.range(0.0, 180.0));
        let lat = f.0 + (r * a.cos() / geo::R).to_degrees();
        let lon = f.1 + (r * a.sin() / (geo::R * f.0.to_radians().cos().max(0.1))).to_degrees();
        let large = rng.next() < site.large;
        let cruise = (rng.range(120.0, 650.0), rng.range(80.0, 900.0));
        out.push(Plan { launch, lat, lon, large, scheme: (rng.next() * SCHEMES.len() as f64) as u8 % SCHEMES.len() as u8, cruise, minutes: rng.range(55.0, 80.0), seed: rng.0 | 1 });
    }
    out
}

fn month_of(doy: u32) -> u32 {
    const ENDS: [u32; 12] = [31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];
    ENDS.iter().position(|&e| doy < e).map_or(12, |i| i as u32 + 1)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// standing on its field, inflated, before take-off
    Standing,
    Flying,
    /// down, burner off, for a while before it is packed away
    Landed,
    /// hit by an aircraft: the envelope torn, falling
    Torn,
}

#[derive(Clone, Debug)]
pub struct Balloon {
    pub plan: Plan,
    pub site: usize,
    /// basket floor, planet frame
    pub p: DVec3,
    /// horizontal drift and climb rate
    pub v: DVec3,
    pub vz: f64,
    pub phase: Phase,
    /// time in the air (s), since landing (s)
    pub flown: f64,
    pub down: f64,
    /// heading of the basket (rad) and how fast it turns (rad/s)
    pub yaw: f64,
    pub yaw_rate: f64,
    /// burner: on, time left in this burn or this pause
    pub burner: bool,
    pub burn_t: f64,
    /// how far the envelope has collapsed (Torn: 0..1)
    pub collapse: f64,
    rng: Rng,
}

/// What the balloons need from the world.
pub struct Env<'a> {
    /// the ground height at a point, if known
    pub ground: &'a dyn Fn(DVec3) -> Option<f64>,
    /// the steady wind at a point, `agl` metres up
    pub wind: &'a dyn Fn(DVec3, f64) -> DVec3,
    /// launches go ahead: calm and dry where the site is
    pub calm: &'a dyn Fn(DVec3) -> bool,
}

impl Balloon {
    fn new(plan: Plan, site: usize, elev: f64) -> Self {
        let p = geo::lla_to_pos(plan.lat.to_radians(), plan.lon.to_radians(), elev);
        let mut rng = Rng(plan.seed);
        let yaw = rng.range(0.0, std::f64::consts::TAU);
        Balloon { plan, site, p, v: DVec3::ZERO, vz: 0.0, phase: Phase::Standing, flown: 0.0, down: 0.0, yaw, yaw_rate: rng.range(-0.02, 0.02), burner: false, burn_t: rng.range(2.0, 20.0), collapse: 0.0, rng }
    }

    fn mouth(&self) -> f64 {
        MOUTH[self.plan.large as usize]
    }

    /// Which flag hangs under its basket, if any (about one balloon in three).
    pub fn flag(&self) -> Option<u32> {
        (self.plan.seed % 3 == 0).then_some(((self.plan.seed >> 8) % FLAGS as u64) as u32)
    }

    /// Where the middle of the envelope is (for collisions).
    pub fn envelope(&self) -> (DVec3, f64) {
        let k = if self.plan.large { LARGE_SCALE } else { 1.0 };
        let up = self.p.normalize();
        (self.p + up * (self.mouth() + 0.6 * ENV_H * k), ENV_R * k * 0.95)
    }

    /// One step of `dt` seconds.
    fn step(&mut self, dt: f64, ground_h: f64, env: &Env) {
        let up = self.p.normalize();
        let agl = self.p.length() - geo::R - ground_h;
        // Turning slowly about the vertical, as balloons do.
        if self.rng.next() < dt / 30.0 {
            self.yaw_rate = self.rng.range(-0.025, 0.025);
        }
        self.yaw += self.yaw_rate * dt;
        let (want_vz, can_burn) = match self.phase {
            Phase::Standing => (0.0, true),
            Phase::Landed => {
                self.down += dt;
                (0.0, false)
            }
            Phase::Torn => {
                self.collapse = (self.collapse + dt / 6.0).min(1.0);
                (-12.0, false)
            }
            Phase::Flying => {
                self.flown += dt;
                let t = self.flown / (self.plan.minutes * 60.0);
                let target = if t < 0.5 { self.plan.cruise.0 } else if t < 0.9 { self.plan.cruise.1 } else { -5.0 };
                let max = if self.plan.large { 2.0 } else { 2.8 };
                (((target - agl) / 40.0).clamp(-max, max), true)
            }
        };
        // The envelope answers the burner slowly (about a quarter of a minute).
        let tau = if self.phase == Phase::Torn { 2.0 } else { 14.0 };
        self.vz += (want_vz - self.vz) * (dt / tau).min(1.0);
        // Burner: longer burns while climbing, short ones to hold the height.
        self.burn_t -= dt;
        if self.burn_t <= 0.0 {
            let need = can_burn && (want_vz - self.vz > -0.4 || self.phase == Phase::Standing);
            self.burner = need && !self.burner;
            self.burn_t = if self.burner {
                if want_vz > 0.8 { self.rng.range(4.0, 9.0) } else { self.rng.range(1.5, 4.0) }
            } else if want_vz > 0.8 {
                self.rng.range(3.0, 8.0)
            } else {
                self.rng.range(12.0, 35.0)
            };
        }
        if !can_burn {
            self.burner = false;
        }
        // Drift: the balloon goes with the air at its height (a little behind it).
        if matches!(self.phase, Phase::Flying | Phase::Torn) {
            let w = (env.wind)(self.p, agl.max(2.0));
            let w = w - up * w.dot(up);
            let drag = if agl < 3.0 { 0.0 } else { 1.0 };
            self.v += (w * drag - self.v) * (dt / 20.0).min(1.0);
        } else {
            self.v = DVec3::ZERO;
        }
        self.p += self.v * dt + up * self.vz * dt;
        // The ground.
        let agl = self.p.length() - geo::R - ground_h;
        if agl < 0.0 {
            self.p += up * -agl;
            if self.vz < 0.0 {
                self.vz = 0.0;
            }
            if self.phase == Phase::Flying && self.flown > 60.0 {
                self.phase = Phase::Landed;
                self.v = DVec3::ZERO;
            }
        }
    }
}

pub struct Balloons {
    pub list: Vec<Balloon>,
    /// the day and the local hour at the sites when they were last planned
    day: Option<(u32, f64)>,
    /// sites near enough to fly (index), as last checked
    active: Vec<usize>,
    /// plans already started today, by site and index
    started: Vec<(usize, usize)>,
    pending: Vec<(usize, usize, Plan)>,
    check: f64,
    /// asked for on the command line: balloons up around (km, bearing) now
    forced: Option<(f64, f64, u32, Option<f64>)>,
}

impl Default for Balloons {
    fn default() -> Self {
        Balloons { list: Vec::new(), day: None, active: Vec::new(), started: Vec::new(), pending: Vec::new(), check: 0.0, forced: None }
    }
}

/// Sites within this of the camera fly (m).
const NEAR_SITE: f64 = 120_000.0;
/// Balloons are drawn within this (m).
const DRAW: f64 = 45_000.0;
/// The near models within this (m).
const NEAR_MODEL: f64 = 2_500.0;

impl Balloons {
    /// With a group of `n` balloons asked for, flying now `km` away on `bearing`.
    pub fn with_forced(forced: Option<(f64, f64, u32, Option<f64>)>) -> Self {
        Balloons { forced, ..Default::default() }
    }

    pub fn count(&self) -> usize {
        self.list.len()
    }

    pub fn nearest(&self, p: DVec3) -> f64 {
        self.list.iter().map(|b| b.p.distance(p)).fold(f64::INFINITY, f64::min)
    }

    /// Advances the flights. `local` gives the local solar hour at a
    /// longitude (degrees), `doy` the day of the year, `year_odd` for the
    /// festivals of odd years.
    #[allow(clippy::too_many_arguments)]
    pub fn step(&mut self, dt: f64, cam: DVec3, level: Level, utc: f64, doy: f64, year_odd: bool, env: &Env) {
        let max = cap(level);
        if max == 0 {
            self.list.clear();
            self.day = None;
            return;
        }
        let day = doy.floor() as u32;
        // A jump in time (the clock set by hand, a new day): plan again.
        let jumped = self.day.is_some_and(|(d, u)| d != day || (utc - u).abs() > 0.2 && (utc - u).abs() < 23.8);
        if self.day.is_none() || jumped {
            self.list.clear();
            self.started.clear();
            self.pending.clear();
            self.active.clear();
            self.check = 0.0;
        }
        self.day = Some((day, utc));
        // (Once the ground under the camera is known, so the heights are right.)
        if self.forced.is_some() && (env.ground)(cam).is_some() {
            if let Some((km, brg, n, agl)) = self.forced.take() {
                self.spawn_here(cam, km, brg, n, agl, env);
            }
        }
        // Which sites are near, now and then.
        self.check -= dt;
        if self.check <= 0.0 {
            self.check = 5.0;
            let near: Vec<usize> = SITES.iter().enumerate().filter(|(_, s)| geo::lla_to_pos(s.lat.to_radians(), s.lon.to_radians(), 0.0).distance(cam.normalize() * geo::R) < NEAR_SITE).map(|(i, _)| i).collect();
            for &si in &near {
                if !self.active.contains(&si) {
                    self.activate(si, utc, day, year_odd, env, max);
                }
            }
            // Sites left behind: their balloons go.
            self.active.retain(|si| near.contains(si));
            let active = self.active.clone();
            self.list.retain(|b| b.site == usize::MAX || active.contains(&b.site));
            self.pending.retain(|(si, _, _)| active.contains(si));
        }
        // Take-offs due (and the balloons standing a few minutes before).
        let mut due = Vec::new();
        self.pending.retain(|&(si, i, plan)| {
            let local = local_hour(utc, SITES[si].lon);
            if hours_between(plan.launch - 0.15, local) >= 0.0 {
                due.push((si, i, plan));
                false
            } else {
                true
            }
        });
        for (si, i, plan) in due {
            if self.list.len() >= max {
                break;
            }
            let site = &SITES[si];
            let p0 = geo::lla_to_pos(plan.lat.to_radians(), plan.lon.to_radians(), site.elev);
            if !(env.calm)(p0) {
                continue;
            }
            let g = (env.ground)(p0).unwrap_or(site.elev);
            self.list.push(Balloon::new(plan, si, g));
            self.started.push((si, i));
        }
        // Fly.
        for b in &mut self.list {
            let local = if b.site == usize::MAX { b.plan.launch + 1.0 } else { local_hour(utc, SITES[b.site].lon) };
            if b.phase == Phase::Standing && hours_between(b.plan.launch, local) >= 0.0 {
                b.phase = Phase::Flying;
            }
            let g = (env.ground)(b.p).unwrap_or_else(|| if b.site == usize::MAX { 0.0 } else { SITES[b.site].elev });
            b.step(dt, g, env);
        }
        // Packed away a while after landing.
        self.list.retain(|b| !(b.phase == Phase::Landed && b.down > 600.0) && !(b.phase == Phase::Torn && b.down > 120.0));
        for b in &mut self.list {
            if b.phase == Phase::Torn && b.p.length() - geo::R - (env.ground)(b.p).unwrap_or(0.0) < 1.0 {
                b.down += dt;
            }
        }
    }

    /// A site comes near: its day's flights, those already up flown forward to now.
    fn activate(&mut self, si: usize, utc: f64, day: u32, year_odd: bool, env: &Env, max: usize) {
        self.active.push(si);
        let site = &SITES[si];
        let local = local_hour(utc, site.lon);
        log::info!("balloons: {} is near, {:.2} h local solar time", site.name, local);
        for (i, plan) in schedule(site, si, day, year_odd).into_iter().enumerate() {
            if self.started.contains(&(si, i)) {
                continue;
            }
            let since = hours_between(plan.launch, local) * 3600.0;
            if since < -0.15 * 3600.0 {
                self.pending.push((si, i, plan));
                continue;
            }
            // Over and packed away already?
            if since > plan.minutes * 60.0 + 900.0 || self.list.len() >= max {
                continue;
            }
            let p0 = geo::lla_to_pos(plan.lat.to_radians(), plan.lon.to_radians(), site.elev);
            if !(env.calm)(p0) {
                continue;
            }
            let g0 = (env.ground)(p0).unwrap_or(site.elev);
            let mut b = Balloon::new(plan, si, g0);
            self.started.push((si, i));
            if since > 0.0 {
                b.phase = Phase::Flying;
                let mut t = 0.0;
                while t < since {
                    let h = (since - t).min(2.0);
                    let g = (env.ground)(b.p).unwrap_or(site.elev);
                    b.step(h, g, env);
                    t += h;
                }
            }
            self.list.push(b);
        }
    }

    /// `n` balloons in the air around a point `km` away on `bearing` (the
    /// command line's --balloons-at): for trying them out anywhere.
    fn spawn_here(&mut self, cam: DVec3, km: f64, brg: f64, n: u32, agl: Option<f64>, env: &Env) {
        let (e, nn, _) = geo::enu(cam);
        let a = brg.to_radians();
        let c = cam + (e * a.sin() + nn * a.cos()) * km * 1000.0;
        let mut rng = Rng(0x1234_5678_9ABC_DEF1 ^ n as u64);
        for i in 0..n {
            // (A small group close together, a big one spread over a few km.)
            let spread = (150.0 + 120.0 * n as f64).min(2500.0);
            let off = if n == 1 { DVec3::ZERO } else { (e * rng.range(-1.0, 1.0) + nn * rng.range(-1.0, 1.0)) * spread };
            let p = c + off;
            let (la, lo, _) = geo::pos_to_lla(p);
            let g = (env.ground)(p).unwrap_or(0.0);
            let plan = Plan { launch: 0.0, lat: la.to_degrees(), lon: lo.to_degrees(), large: rng.next() < 0.4, scheme: (i % SCHEMES.len() as u32) as u8, cruise: (rng.range(150.0, 600.0), rng.range(100.0, 700.0)), minutes: rng.range(55.0, 80.0), seed: rng.0 | 1 };
            let mut b = Balloon::new(plan, usize::MAX, g);
            b.phase = Phase::Flying;
            b.flown = 300.0;
            // (At the height asked for, holding it; else somewhere up to 500 m.)
            let h = agl.unwrap_or_else(|| rng.range(80.0, 500.0));
            b.p = b.p.normalize() * (geo::R + g + h);
            if agl.is_some() {
                // (Climbing slowly from there: the burner works often, to be seen.)
                b.plan.cruise = (h + 150.0, h + 150.0);
                b.plan.minutes = 600.0;
            }
            self.list.push(b);
        }
    }

    /// The burner jets lit near the camera, for the particle system: each
    /// burner's nozzle, the balloon's up, and how fast the balloon moves.
    pub fn jets(&self, cam: DVec3) -> Vec<(DVec3, DVec3, DVec3)> {
        let mut out = Vec::new();
        for b in self.list.iter().filter(|b| b.burner && b.p.distance(cam) < FLAME_JET) {
            let up = b.p.normalize();
            let (e, n, _) = geo::enu(b.p);
            let fwd = e * b.yaw.sin() + n * b.yaw.cos();
            let right = fwd.cross(up).normalize();
            let big = b.plan.large as usize;
            let nozzle = b.p + up * NOZZLE[big];
            let vel = b.v + up * b.vz;
            // (The sport rig's double burner, the passenger rig's four.)
            for &(x, z) in BURNERS[big] {
                out.push((nozzle + right * x - fwd * z, up, vel));
            }
        }
        out
    }

    /// An aircraft at `a` (moving to `b`, radius `r`) through an envelope: that
    /// balloon is torn and falls. Returns where it was hit.
    pub fn hit(&mut self, a: DVec3, b: DVec3, r: f64) -> Option<DVec3> {
        for bl in &mut self.list {
            if bl.phase == Phase::Torn {
                continue;
            }
            let (c, rad) = bl.envelope();
            let ab = b - a;
            let t = if ab.length_squared() > 1e-9 { ((c - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
            let q = a + ab * t;
            if q.distance(c) < rad + r {
                bl.phase = Phase::Torn;
                bl.burner = false;
                bl.down = 0.0;
                return Some(q);
            }
        }
        None
    }

    /// The instanced solids for the balloons in view of the camera at `cam`.
    /// `people`: the pilot and passengers (scenery.rs `people`), up close.
    pub fn instances(&self, cam: DVec3, time: f64, visible: &dyn Fn(DVec3, f64) -> bool, out: &mut Vec<(u32, Inst)>, people: &mut Vec<(u32, Inst)>) {
        for (k, b) in self.list.iter().enumerate() {
            let rel = b.p - cam;
            let d = rel.length();
            let big = b.plan.large as usize;
            let ks = if b.plan.large { LARGE_SCALE } else { 1.0 };
            if d > DRAW || !visible(rel + b.p.normalize() * (MOUTH[big] + 0.5 * ENV_H * ks), ENV_H * ks) {
                continue;
            }
            let up = b.p.normalize();
            let (e, n, _) = geo::enu(b.p);
            let fwd = e * b.yaw.sin() + n * b.yaw.cos();
            let right = fwd.cross(up).normalize();
            let base = |scale: DVec3, lift: f64| -> Mat4 {
                Mat4::from_cols(
                    (right * scale.x).as_vec3().extend(0.0),
                    (up * scale.y).as_vec3().extend(0.0),
                    (-fwd * scale.z).as_vec3().extend(0.0),
                    (rel + up * lift).as_vec3().extend(1.0),
                )
            };
            // A torn envelope sags and folds in on itself as it falls.
            let squash = 1.0 - 0.7 * b.collapse;
            let env_s = DVec3::new(ks * (1.0 - 0.45 * b.collapse), ks * squash, ks * (1.0 - 0.45 * b.collapse));
            let scheme = b.plan.scheme as u32 % SCHEMES.len() as u32;
            if d > NEAR_MODEL {
                out.push((ENV_FAR + scheme, scaled(base(env_s, MOUTH[big]), ks)));
                if d < 9_000.0 {
                    out.push((RIG_FAR, inst(base(DVec3::splat(if b.plan.large { 1.6 } else { 1.0 }), 0.0))));
                }
            } else {
                let mut ei = scaled(base(env_s, MOUTH[big]), ks);
                // (The burner lit: the envelope glows from inside, inst.wgsl.)
                ei.ext[1] = if b.burner { 1.0 } else { 0.0 };
                out.push((ENV + scheme, ei));
                // (No scoop: like most envelopes, the cloth runs down to the
                // mouth; the "skirt" piece stays in the pack, unused.)
                let _ = SKIRT;
                if b.collapse < 0.3 {
                    out.push((if b.plan.large { WIRES_LARGE } else { WIRES_SPORT }, inst(base(DVec3::ONE, 0.0))));
                }
                out.push((if b.plan.large { RIG_LARGE } else { RIG_SPORT }, inst(base(DVec3::ONE, 0.0))));
                if d < PEOPLE_DRAW {
                    crew(b, rel, right, up, fwd, d > PEOPLE_NEAR, people);
                }
            }
            if let Some(f) = b.flag() {
                if d < 3_000.0 {
                    let [x, y, z] = FLAG_AT[big];
                    let m = base(DVec3::ONE, 0.0) * Mat4::from_translation(glam::Vec3::new(x as f32, y as f32, z as f32));
                    out.push((FLAG + f, inst(m)));
                }
            }
            if b.burner && d < 15_000.0 {
                // A roaring jet from each burner, three metres or so long,
                // surging (inst.wgsl draws the fire itself, moving).
                for (j, &(x, z)) in BURNERS[big].iter().enumerate() {
                    let ph = k as f64 * 1.7 + j as f64 * 2.3;
                    let f = (time * 11.0 + ph).sin() * 0.5 + (time * 17.0 + ph * 1.3).sin() * 0.3;
                    let len = (if b.plan.large { 3.4 } else { 3.0 }) * (1.0 + 0.07 * f);
                    let w = 1.15 + 0.08 * f;
                    let m = base(DVec3::new(w, len, w), NOZZLE[big]) * Mat4::from_translation(glam::Vec3::new(x as f32, 0.0, z as f32) / glam::Vec3::new(w as f32, len as f32, w as f32));
                    let mut fi = inst(m);
                    fi.ext = [1.0, len as f32, (ph * 0.37).fract() as f32, 0.0];
                    out.push((FLAME, fi));
                }
            }
        }
    }
}

/// How far away the people in a basket are still drawn (m), and up to where
/// in full (beyond: people.py's far versions, a tenth of the triangles).
const PEOPLE_DRAW: f64 = 400.0;
const PEOPLE_NEAR: f64 = 50.0;

/// Where the people stand in the baskets (balloons.py `crew_places`): per
/// rig, sport then large, (x, y, z in the rig's frame, facing from +x
/// towards -z), the pilot first.
fn places() -> &'static [Vec<[f32; 4]>; 2] {
    static P: std::sync::OnceLock<[Vec<[f32; 4]>; 2]> = std::sync::OnceLock::new();
    P.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(include_str!("../assets/balloon_crew.json")).expect("balloon_crew.json");
        let rig = |k: &str| -> Vec<[f32; 4]> {
            v[k].as_array().map(|a| a.iter().map(|p| std::array::from_fn(|i| p[i].as_f64().unwrap_or(0.0) as f32)).collect()).unwrap_or_default()
        };
        [rig("sport"), rig("large")]
    })
}

/// Hair colours (linear), to multiply people.py's grey hair by: the darker
/// ones the more common, grey for the older people.
const HAIR: [[f32; 3]; 7] = [[0.012, 0.009, 0.007], [0.035, 0.020, 0.010], [0.035, 0.020, 0.010], [0.09, 0.05, 0.022], [0.30, 0.20, 0.09], [0.16, 0.05, 0.02], [0.33, 0.32, 0.31]];
/// people.py's people who are older (index): grey or white more often.
const OLD: [bool; 12] = [false, false, false, false, false, false, false, false, false, true, true, false];

/// The pilot and passengers of balloon `b` (camera-relative at `rel`): each
/// balloon its own people, in its own order, with their own hair colours.
fn crew(b: &Balloon, rel: DVec3, right: DVec3, up: DVec3, fwd: DVec3, far: bool, out: &mut Vec<(u32, Inst)>) {
    // (people.py: the people, then their far versions.)
    let n = crate::scenery::people().meshes.len() as u64 / 2;
    if n < 2 || b.collapse > 0.5 {
        return;
    }
    let mut rng = Rng(b.plan.seed ^ 0x9E37_79B9_7F4A_7C15);
    let off = (rng.next() * n as f64) as u64;
    for (k, p) in places()[b.plan.large as usize].iter().enumerate() {
        // (The pilot is people.py's first; the passengers the rest, in turn.)
        let who = if k == 0 { 0 } else { 1 + (off + k as u64 * 5) % (n - 1) };
        let f = p[3] as f64 + (rng.next() - 0.5) * 0.2;
        let ahead = right * f.cos() + fwd * f.sin();
        let left = up.cross(ahead);
        let pos = rel + right * p[0] as f64 + up * p[1] as f64 - fwd * p[2] as f64;
        let (cx, cy, cz, t) = (ahead.as_vec3(), left.as_vec3(), up.as_vec3(), pos.as_vec3());
        let hair = if OLD[who as usize % OLD.len()] && rng.next() < 0.7 { HAIR[6] } else { HAIR[(rng.next() * 6.0) as usize % 6] };
        // (scenery.wgsl: the grey hair, 0.21 linear, times the tint.)
        let tint = [hair[0] / 0.21, hair[1] / 0.21, hair[2] / 0.21, 0.0];
        let m = [[cx.x, cy.x, cz.x, t.x], [cx.y, cy.y, cz.y, t.y], [cx.z, cy.z, cz.z, t.z]];
        out.push(((who + if far { n } else { 0 }) as u32, Inst { m, tint, ext: [0.0, rng.next() as f32, 0.0, 0.0] }));
    }
}

/// Local solar hour at a longitude (degrees), from UTC hours.
fn local_hour(utc: f64, lon: f64) -> f64 {
    (utc + lon / 15.0).rem_euclid(24.0)
}

/// Hours from `a` to `b` on the clock, between -12 and 12.
fn hours_between(a: f64, b: f64) -> f64 {
    (b - a + 12.0).rem_euclid(24.0) - 12.0
}

fn inst(m: Mat4) -> Inst {
    scaled(m, 1.0)
}

/// An instance of a piece drawn `k` times its size (inst.wgsl's ext.x keeps
/// the weave of the cloth at its real size).
fn scaled(m: Mat4, k: f64) -> Inst {
    let c = m.to_cols_array_2d();
    Inst { m: [[c[0][0], c[1][0], c[2][0], c[3][0]], [c[0][1], c[1][1], c[2][1], c[3][1]], [c[0][2], c[1][2], c[2][2], c[3][2]]], tint: [1.0, 1.0, 1.0, 0.0], ext: [k as f32, 0.0, 0.0, 0.0] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_baskets_have_their_people() {
        let pack = crate::scenery::people();
        let n = pack.meshes.len();
        assert!(n >= 4 && n % 2 == 0, "people.py: the people, then their far versions ({n} meshes)");
        assert!(pack.mats.len() <= 128, "the scenery shader holds 128 materials");
        for k in 0..n / 2 {
            let (near, far) = (&pack.meshes[k], &pack.meshes[k + n / 2]);
            assert!(far.idx.len() * 4 < near.idx.len(), "{}: the far version is lighter", near.name);
            // (Standing on the floor, cut off below the rim, no taller than 2 m.)
            let top = near.verts.iter().map(|v| v.pos[2]).fold(0.0f32, f32::max);
            let low = near.verts.iter().map(|v| v.pos[2]).fold(9.0f32, f32::min);
            assert!((1.3..2.0).contains(&top) && low > 0.6, "{}: {low}..{top} m", near.name);
        }
        let [sport, large] = places();
        assert_eq!((sport.len(), large.len()), (3, 21));
        // Everybody inside the wicker, hands and all.
        for (p, (w, l)) in [(sport, (1.25, 1.55)), (large, (2.2, 4.6))] {
            for q in p {
                let (x, z) = (q[0] + 0.42 * q[3].cos(), -q[2] + 0.42 * q[3].sin());
                assert!(x.abs() < w / 2.0 - 0.05 && z.abs() < l / 2.0 - 0.05, "{q:?}");
            }
        }
    }

    #[test]
    fn the_models_load_and_match_the_constants() {
        let m = models();
        assert_eq!(m.meshes.len(), 2 * SCHEMES.len() + PARTS.len());
        let objs = crate::aircraft::fa18::load(include_bytes!("../assets/balloons.bin.gz"), "balloons");
        for (rig, big) in [("rig sport", 0), ("rig large", 1)] {
            let o = &objs[rig];
            // (pivot: the nozzle; axis: the mouth, both above the basket floor, game frame y up.)
            assert!((o.pivot.y as f64 - NOZZLE[big]).abs() < 1e-3, "{rig} nozzle {}", o.pivot.y);
            assert!((o.axis.y as f64 - MOUTH[big]).abs() < 1e-3, "{rig} mouth {}", o.axis.y);
        }
        // The envelope: mouth at its origin, crown ENV_H above, ENV_R across.
        let env = &objs["env rainbow"];
        let top = env.mesh.v.iter().map(|v| v.pos[1]).fold(f32::MIN, f32::max) as f64;
        let wide = env.mesh.v.iter().map(|v| (v.pos[0] * v.pos[0] + v.pos[2] * v.pos[2]).sqrt()).fold(0.0f32, f32::max) as f64;
        assert!((top - ENV_H).abs() < 0.05, "{top}");
        assert!((wide - ENV_R).abs() / ENV_R < 0.04, "{wide}");
    }

    #[test]
    fn sunrise_is_where_the_almanac_puts_it() {
        // Göreme, 21 June: sunrise about 4:55 local solar time (civil 5:32,
        // UTC+3, 35° E is 20 min ahead of the zone's own meridian... the
        // solar figure is what the game's sun uses).
        let h = half_day(38.64, 172.0).unwrap();
        assert!((12.0 - h - 4.75).abs() < 0.25, "{}", 12.0 - h);
        // Equinox: about six.
        let h = half_day(0.0, 80.0).unwrap();
        assert!((12.0 - h - 6.0).abs() < 0.1);
    }

    #[test]
    fn the_schedule_is_the_same_each_time_and_follows_the_seasons() {
        let a = schedule(&SITES[0], 0, 150, false);
        let b = schedule(&SITES[0], 0, 150, false);
        assert!(a.len() >= 70 && a.len() <= 150, "{}", a.len());
        assert_eq!(a.iter().map(|p| p.launch).collect::<Vec<_>>(), b.iter().map(|p| p.launch).collect::<Vec<_>>());
        // Mornings around sunrise; mostly large balloons in Cappadocia.
        let rise = 12.0 - half_day(SITES[0].lat, 150.0).unwrap();
        assert!(a.iter().all(|p| p.launch > rise - 0.3 && p.launch < rise + 0.8));
        assert!(a.iter().filter(|p| p.large).count() as f64 / a.len() as f64 > 0.8);
        // Bagan flies October to March only; Albuquerque's Fiesta fills the sky.
        let bagan = SITES.iter().position(|s| s.name == "Bagan").unwrap();
        assert!(schedule(&SITES[bagan], bagan, 180, false).is_empty());
        assert!(!schedule(&SITES[bagan], bagan, 20, false).is_empty());
        let abq = SITES.iter().position(|s| s.name == "Albuquerque").unwrap();
        assert_eq!(schedule(&SITES[abq], abq, 280, false).len(), 500);
        // Chambley's Mondial only in odd years.
        let ch = SITES.iter().position(|s| s.name == "Chambley").unwrap();
        assert!(schedule(&SITES[ch], ch, 210, true).len() >= 300);
        assert!(schedule(&SITES[ch], ch, 210, false).len() < 10);
    }

    #[test]
    fn a_balloon_takes_off_drifts_and_lands() {
        let site = &SITES[0];
        let plan = Plan { launch: 6.0, lat: site.lat, lon: site.lon, large: true, scheme: 0, cruise: (400.0, 600.0), minutes: 60.0, seed: 99 };
        let mut b = Balloon::new(plan, 0, site.elev);
        b.phase = Phase::Flying;
        let start = b.p;
        let (e, _, _) = geo::enu(start);
        let ground = |_: DVec3| Some(site.elev);
        let wind = |_: DVec3, _: f64| e * 4.0;
        let calm = |_: DVec3| true;
        let env = Env { ground: &ground, wind: &wind, calm: &calm };
        let mut top: f64 = 0.0;
        let mut burned = 0.0;
        for _ in 0..(75 * 60) {
            b.step(1.0, site.elev, &env);
            top = top.max(b.p.length() - geo::R - site.elev);
            burned += b.burner as u32 as f64;
        }
        assert!(top > 350.0 && top < 700.0, "highest {top}");
        assert_eq!(b.phase, Phase::Landed, "{:?} at {:.0} m", b.phase, b.p.length() - geo::R - site.elev);
        // About 4 m/s for an hour: some 14 km east (it lags the wind a little).
        let went = (b.p - start).dot(e);
        assert!(went > 10_000.0 && went < 16_000.0, "{went}");
        // The burner burns now and then, not all the time.
        assert!(burned > 60.0 && burned < 0.5 * 3600.0, "{burned}");
    }

    #[test]
    fn an_aircraft_through_the_envelope_tears_it() {
        let site = &SITES[0];
        let plan = Plan { launch: 6.0, lat: site.lat, lon: site.lon, large: false, scheme: 0, cruise: (400.0, 600.0), minutes: 60.0, seed: 3 };
        let b = Balloon::new(plan, 0, site.elev);
        let mut bs = Balloons::default();
        bs.list.push(b);
        let (c, _) = bs.list[0].envelope();
        let (e, _, _) = geo::enu(c);
        assert!(bs.hit(c - e * 100.0 + c.normalize() * 30.0, c + e * 100.0 + c.normalize() * 30.0, 5.0).is_none(), "passing above");
        assert!(bs.hit(c - e * 100.0, c + e * 100.0, 5.0).is_some());
        assert_eq!(bs.list[0].phase, Phase::Torn);
    }

    #[test]
    fn near_cappadocia_after_sunrise_the_sky_fills() {
        let site = &SITES[0];
        let cam = geo::lla_to_pos(site.lat.to_radians(), (site.lon - 0.05).to_radians(), 1700.0);
        let ground = |_: DVec3| Some(site.elev);
        let wind = |p: DVec3, _: f64| geo::enu(p).0 * 3.0;
        let calm = |_: DVec3| true;
        let env = Env { ground: &ground, wind: &wind, calm: &calm };
        let mut bs = Balloons::default();
        // Day 150, 5.4 h local solar time: half an hour after sunrise.
        let utc = 5.4 - site.lon / 15.0;
        for _ in 0..20 {
            bs.step(0.5, cam, Level::High, utc, 150.0, false, &env);
        }
        let up = bs.list.iter().filter(|b| b.phase == Phase::Flying).count();
        assert!(bs.list.len() > 30, "{} balloons", bs.list.len());
        assert!(up > 10, "{up} flying");
    }
}

