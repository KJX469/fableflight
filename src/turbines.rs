//! Wind turbines: every one OpenStreetMap knows (power=generator,
//! generator:source=wind; data © OpenStreetMap contributors, ODbL), packed by
//! assets/mkturbines.py into assets/wind_turbines.bin.gz with its hub height
//! and rotor diameter (from its tags, or estimated from its rated power).
//!
//! One model (assets/blender/wind_turbine.py: a generic 3 MW machine, hub at
//! 100 m, rotor 120 m across) is put together from its pieces and scaled to
//! each turbine: the tower to the hub's height, the nacelle and rotor to the
//! rotor's size. The nacelle turns into the day's wind, and the rotor turns
//! at the speed that wind gives (from cut-in at 3 m/s up to the rated tip
//! speed, feathered and still in a gale), each out of step with the others.
//! At sea a yellow transition piece stands under the tower. At night the
//! nacelles' red obstruction lights flash, a whole region's together, as
//! they are synchronised. Flying into a tower or through a rotor is a crash.
//! Drawn as instanced solids (inst.wgsl's painted vehicle surface, tinted).

use crate::geo;
use crate::graphics::Level;
use crate::render::{Inst, MeshVertex};
use glam::{DVec3, Mat4, Vec3};
use std::collections::HashMap;
use std::io::Read;
use std::sync::OnceLock;

// ------------------------------------------------------------------ models

/// The pieces, in wind_turbine.py's export order.
pub const PIECES: [&str; 8] = ["tower", "nacelle", "rotor", "rotor bands", "transition", "tower far", "nacelle far", "rotor far"];
const TOWER: u32 = 0;
const NACELLE: u32 = 1;
const ROTOR: u32 = 2;
const ROTOR_BANDS: u32 = 3;
const TRANSITION: u32 = 4;
const TOWER_FAR: u32 = 5;
const NACELLE_FAR: u32 = 6;
const ROTOR_FAR: u32 = 7;
/// The first mesh id of the turbines among the instanced solids.
pub const BASE: u32 = crate::balloons::BASE + crate::balloons::N_MESHES;

/// The model's own dimensions (wind_turbine.py).
const REF_D: f64 = 120.0;
const TOWER_TOP: f64 = 97.6;
/// The hub's centre from the yaw bearing (game frame: x right, y up, z back).
const ROTOR_AT: [f64; 3] = [0.0, 2.4, -5.2];
const TILT: f32 = 5.0 * std::f32::consts::PI / 180.0;
/// The transition piece's top, where an offshore tower stands (m above the sea).
const SEA_DECK: f64 = 17.0;

pub struct Models {
    pub meshes: Vec<(Vec<MeshVertex>, Vec<u32>)>,
}

pub fn models() -> &'static Models {
    static M: OnceLock<Models> = OnceLock::new();
    M.get_or_init(|| {
        let objs = crate::aircraft::fa18::load(include_bytes!("../assets/wind_turbine.bin.gz"), "wind turbine");
        let meshes = PIECES
            .iter()
            .map(|name| {
                let o = objs.get(*name).unwrap_or_else(|| panic!("wind turbine model: no piece \"{name}\""));
                // (Colour alpha: inst.wgsl's vehicle material, 0 paint, 1 plain.)
                let v = o.mesh.v.iter().map(|v| MeshVertex { pos: v.pos, nrm: v.nrm, col: [v.col[0], v.col[1], v.col[2], v.mat[0]] }).collect();
                (v, o.mesh.i.clone())
            })
            .collect();
        Models { meshes }
    })
}

/// How far away turbines are drawn (m), by setting; their lights further.
pub fn reach(level: Level) -> f64 {
    match level {
        Level::Off => 0.0,
        Level::Low => 8_000.0,
        Level::Medium => 16_000.0,
        Level::High | Level::Ultra => 30_000.0,
    }
}
/// Up to here the full models, beyond the light ones.
const NEAR: f64 = 2_500.0;

// -------------------------------------------------------------------- data

/// One turbine as the pack has it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Site {
    pub lat: f64,
    pub lon: f64,
    pub hub: f64,
    pub rotor: f64,
}

pub struct Data {
    cells: HashMap<(i16, i16), (u32, u32)>,
    recs: Vec<[u8; 6]>,
    cell_of: Vec<(i16, i16)>,
}

impl Data {
    pub fn parse(gz: &[u8]) -> Data {
        let mut raw = Vec::new();
        flate2::read::GzDecoder::new(gz).read_to_end(&mut raw).expect("wind turbines pack");
        assert_eq!(&raw[0..4], b"FFWT", "wind turbines: not a pack");
        let u32_at = |p: usize| u32::from_le_bytes(raw[p..p + 4].try_into().unwrap());
        assert_eq!(u32_at(4), 1, "wind turbines: unknown version (rebuild it with assets/mkturbines.py)");
        let n = u32_at(8) as usize;
        let mut cells = HashMap::with_capacity(n);
        let mut cell_of = Vec::new();
        let mut start = 0u32;
        for k in 0..n {
            let p = 12 + k * 8;
            let la = i16::from_le_bytes([raw[p], raw[p + 1]]);
            let lo = i16::from_le_bytes([raw[p + 2], raw[p + 3]]);
            let c = u32_at(p + 4);
            cells.insert((la, lo), (start, c));
            cell_of.extend(std::iter::repeat_n((la, lo), c as usize));
            start += c;
        }
        let body = 12 + n * 8;
        let recs: Vec<[u8; 6]> = raw[body..].chunks_exact(6).map(|c| c.try_into().unwrap()).collect();
        assert_eq!(recs.len(), start as usize, "wind turbines: pack cut short");
        Data { cells, recs, cell_of }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn len(&self) -> usize {
        self.recs.len()
    }

    fn site(&self, i: usize) -> Site {
        let r = self.recs[i];
        let (la, lo) = self.cell_of[i];
        let dlat = u16::from_le_bytes([r[0], r[1]]) as f64 / 65536.0;
        let dlon = u16::from_le_bytes([r[2], r[3]]) as f64 / 65536.0;
        Site { lat: la as f64 + dlat, lon: lo as f64 + dlon, hub: r[4] as f64, rotor: r[5] as f64 }
    }

    /// The turbines within `km` of (lat, lon) (degrees), with their indices.
    pub fn near(&self, lat: f64, lon: f64, km: f64) -> Vec<(usize, Site)> {
        let dlat = km / 111.0;
        let dlon = km / (111.0 * lat.to_radians().cos().max(0.05));
        let mut out = Vec::new();
        for la in (lat - dlat).floor() as i32..=(lat + dlat).floor() as i32 {
            for lo in (lon - dlon).floor() as i32..=(lon + dlon).floor() as i32 {
                // (Across the antimeridian.)
                let lo_w = (lo + 180).rem_euclid(360) - 180;
                if let Some(&(s, c)) = self.cells.get(&(la as i16, lo_w as i16)) {
                    for i in s as usize..(s + c) as usize {
                        let t = self.site(i);
                        let dy = (t.lat - lat) * 111.0;
                        let mut dx = t.lon - lon;
                        dx -= 360.0 * (dx / 360.0).round();
                        let dx = dx * 111.0 * lat.to_radians().cos();
                        if dx * dx + dy * dy < km * km {
                            out.push((i, t));
                        }
                    }
                }
            }
        }
        out
    }
}

pub fn data() -> &'static Data {
    static D: OnceLock<Data> = OnceLock::new();
    D.get_or_init(|| Data::parse(include_bytes!("../assets/wind_turbines.bin.gz")))
}

// ------------------------------------------------------------ in the world

/// A turbine near the camera, where it stands once its ground is known.
struct Near {
    site: Site,
    /// a steady random number per turbine
    seed: u64,
    /// the foot (planet frame) and whether it stands in the sea
    foot: Option<(DVec3, bool)>,
}

#[derive(Default)]
pub struct Turbines {
    near: Vec<Near>,
    /// where the camera was when `near` was gathered, and how far it reached
    at: Option<(DVec3, f64)>,
    /// seconds since the ground was last looked up for those without
    since: f64,
}

fn hash(i: u64) -> u64 {
    let mut h = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 31;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^ (h >> 29)
}
fn unit(h: u64, k: u32) -> f64 {
    ((h >> (k * 8)) & 0xFFFF) as f64 / 65535.0
}

/// The wind the turbines feel: a direction (where it blows to, planet frame)
/// and a speed (m/s) at 100 m.
pub struct Wind {
    pub dir: DVec3,
    pub speed: f64,
}

impl Turbines {
    /// Gathers the turbines around the camera (again when it has moved a
    /// good way), and stands them on the ground (`ground(lat, lon)` in
    /// radians: height and whether it is water).
    pub fn step(&mut self, dt: f64, cam: DVec3, level: Level, ground: &dyn Fn(f64, f64) -> Option<(f64, bool)>) {
        let reach = reach(level);
        if reach <= 0.0 {
            self.near.clear();
            self.at = None;
            return;
        }
        let stale = self.at.is_none_or(|(p, r)| p.distance(cam) > 0.2 * reach || r != reach);
        if stale {
            let (lat, lon, _) = geo::pos_to_lla(cam);
            let mut old: HashMap<u64, Option<(DVec3, bool)>> = self.near.drain(..).map(|n| (n.seed, n.foot)).collect();
            self.near = data()
                .near(lat.to_degrees(), lon.to_degrees(), reach * 1.25 / 1000.0)
                .into_iter()
                .map(|(i, site)| {
                    let seed = hash(i as u64 + 1);
                    Near { site, seed, foot: old.remove(&seed).flatten() }
                })
                .collect();
            self.at = Some((cam, reach));
            self.since = 1e9;
        }
        self.since += dt;
        if self.since > 1.5 {
            // (Again every so often: the terrain sharpens as its tiles come in.)
            self.since = 0.0;
            for n in self.near.iter_mut() {
                if let Some((h, water)) = ground(n.site.lat.to_radians(), n.site.lon.to_radians()) {
                    let p = geo::lla_to_pos(n.site.lat.to_radians(), n.site.lon.to_radians(), if water { h.max(0.0) } else { h });
                    n.foot = Some((p, water));
                }
            }
        }
    }

    pub fn count(&self) -> usize {
        self.near.len()
    }

    /// How far the nearest standing one is (m), and its foot's height above the sphere.
    pub fn nearest(&self, cam: DVec3) -> (f64, f64) {
        self.near.iter().filter_map(|n| n.foot).map(|(p, _)| (p.distance(cam), p.length() - geo::R)).fold((f64::MAX, 0.0), |a, b| if b.0 < a.0 { b } else { a })
    }

    /// Those whose ground is known (drawn).
    pub fn standing(&self) -> usize {
        self.near.iter().filter(|n| n.foot.is_some()).count()
    }

    /// How a turbine stands: (foot of the tower, up, the way its rotor
    /// faces, right, scales of the tower (across, up) and of the rotor).
    fn frame(n: &Near, wind: &Wind) -> Option<Frame> {
        let (foot, sea) = n.foot?;
        let up = foot.normalize();
        // (Into the wind, a few degrees off: each yaws on its own.)
        let w = wind.dir - up * wind.dir.dot(up);
        let (e, nn, _) = geo::enu(foot);
        let into = if w.length() > 1e-6 { -w.normalize() } else { nn };
        let err = (unit(n.seed, 0) - 0.5) * 0.14 + if wind.speed < 2.0 { (unit(n.seed, 4) - 0.5) * 3.0 } else { 0.0 };
        let right0 = into.cross(up).normalize();
        let fwd = (into * err.cos() + right0 * err.sin()).normalize();
        let right = fwd.cross(up).normalize();
        let _ = e;
        let sr = n.site.rotor / REF_D;
        let base = if sea { foot + up * SEA_DECK } else { foot };
        let hub_from_base = n.site.hub - if sea { SEA_DECK } else { 0.0 };
        let sv = ((hub_from_base - ROTOR_AT[1] * sr) / TOWER_TOP).max(0.1);
        let sx = sr.max(0.5);
        Some(Frame { foot, base, up, fwd, right, sv, sx, sr, sea })
    }

    /// Rotor speed (rad/s) in a wind of `v` m/s at hub height.
    fn omega(rotor: f64, v: f64) -> f64 {
        let r = rotor * 0.5;
        if !(3.0..25.0).contains(&v) {
            // (Idling below cut-in, feathered and parked in a gale.)
            return if v < 3.0 { 0.02 * v } else { 0.0 };
        }
        // Tip-speed ratio about 8 up to the rated tip speed (~80 m/s).
        (8.0 * v / r).min(80.0 / r)
    }

    /// The instanced solids for the turbines in view, and their lights at night.
    #[allow(clippy::too_many_arguments)]
    pub fn instances(&self, cam: DVec3, time: f64, level: Level, wind: &Wind, night: bool, visible: &dyn Fn(DVec3, f64) -> bool, out: &mut Vec<(u32, Inst)>, lights: &mut Vec<crate::airfield::Light>) {
        let reach = reach(level);
        for n in &self.near {
            let Some(f) = Self::frame(n, wind) else { continue };
            let rel = f.foot - cam;
            let d = rel.length();
            let tall = n.site.hub + n.site.rotor * 0.5;
            let top = f.base + f.up * (TOWER_TOP * f.sv);
            if night && d < reach * 1.6 {
                // Red obstruction lights on the nacelle (ICAO medium
                // intensity, ~2000 cd), flashing a second on, a second off,
                // the whole region together.
                let at = top - cam + f.up * (5.2 * f.sr) - f.fwd * (4.8 * f.sr);
                lights.push(crate::airfield::Light { pos: at.as_vec3().into(), inten: 60.0, dir: [0.0; 3], spread: 0.0, col: [255, 28, 20, 0], fx: [20, 0, 128, 0] });
            }
            if d > reach || !visible(rel + f.up * (tall * 0.5), tall * 0.6) {
                continue;
            }
            let far = d > NEAR;
            let k = BASE;
            let tint = {
                let g = 0.6 + 0.05 * unit(n.seed, 2) as f32;
                [g, g * 1.01, g * 0.995, 0.0]
            };
            let mat = |cols: [DVec3; 3], at: DVec3| -> Mat4 { Mat4::from_cols(cols[0].as_vec3().extend(0.0), cols[1].as_vec3().extend(0.0), cols[2].as_vec3().extend(0.0), (at - cam).as_vec3().extend(1.0)) };
            let (right, up, back) = (f.right, f.up, -f.fwd);
            // The tower, on the ground (or the transition piece at sea).
            out.push((k + if far { TOWER_FAR } else { TOWER }, inst(mat([right * f.sx, up * f.sv, back * f.sx], f.base), tint)));
            if f.sea {
                out.push((k + TRANSITION, inst(mat([right * f.sx.max(0.8), up, back * f.sx.max(0.8)], f.foot), [1.0; 4])));
            }
            let nac = mat([right * f.sr, up * f.sr, back * f.sr], top);
            out.push((k + if far { NACELLE_FAR } else { NACELLE }, inst(nac, tint)));
            // The rotor: at the hub, tilted, turning about its axis.
            let v = wind.speed * (n.site.hub / 100.0).powf(0.14);
            let stopped = unit(n.seed, 3) < 0.04;
            let w = if stopped { 0.0 } else { Self::omega(n.site.rotor, v) };
            let spin = (unit(n.seed, 1) * std::f64::consts::TAU + w * time) as f32;
            let at = Vec3::new(ROTOR_AT[0] as f32, ROTOR_AT[1] as f32, ROTOR_AT[2] as f32);
            let rot = nac * Mat4::from_translation(at) * Mat4::from_rotation_x(TILT) * Mat4::from_rotation_z(spin);
            let bands = banded(&n.site, n.seed);
            out.push((k + if far { ROTOR_FAR } else if bands { ROTOR_BANDS } else { ROTOR }, inst(rot, tint)));
        }
    }

    /// An aircraft moving from `a` to `b` (radius `r`) into a tower or
    /// through a rotor's disc: where it hit.
    pub fn hit(&self, a: DVec3, b: DVec3, r: f64, wind: &Wind) -> Option<DVec3> {
        for n in &self.near {
            let Some(f) = Self::frame(n, wind) else { continue };
            if f.foot.distance(b) > n.site.hub + n.site.rotor {
                continue;
            }
            // The tower: a cylinder up to the nacelle.
            let top = f.base + f.up * (TOWER_TOP * f.sv + 4.0 * f.sr);
            let (p, q) = closest(a, b, f.base, top);
            if p.distance(q) < r + 2.2 * f.sx {
                return Some(p);
            }
            // The rotor: its disc (a little thick), out to the tips.
            let hub = f.foot + f.up * n.site.hub;
            let axis = f.fwd;
            let (da, db) = ((a - hub).dot(axis), (b - hub).dot(axis));
            let slab = r + 2.0;
            if da.abs() < slab || db.abs() < slab || da.signum() != db.signum() {
                let t = if (da - db).abs() > 1e-9 { (da / (da - db)).clamp(0.0, 1.0) } else { 0.0 };
                let p = a + (b - a) * t;
                let off = p - hub - axis * (p - hub).dot(axis);
                if off.length() < n.site.rotor * 0.5 + r * 0.5 {
                    return Some(p);
                }
            }
        }
        None
    }
}

struct Frame {
    foot: DVec3,
    /// where the tower stands: the ground, or the transition piece at sea
    base: DVec3,
    up: DVec3,
    fwd: DVec3,
    right: DVec3,
    /// the tower's scale up and across, the rotor's (and nacelle's)
    sv: f64,
    sx: f64,
    sr: f64,
    sea: bool,
}

/// Red bands on the blade tips: the marking for tall turbines in Germany,
/// Denmark and the Netherlands (most there); a few elsewhere.
fn banded(s: &Site, seed: u64) -> bool {
    let tall = s.hub + s.rotor * 0.5 > 100.0;
    let region = (47.2..55.2).contains(&s.lat) && (3.3..15.2).contains(&s.lon) && !(s.lon < 6.0 && s.lat < 50.6);
    tall && unit(seed, 5) < if region { 0.85 } else { 0.06 }
}

/// The closest points between segments a-b and c-d.
fn closest(a: DVec3, b: DVec3, c: DVec3, d: DVec3) -> (DVec3, DVec3) {
    let (u, v, w) = (b - a, d - c, a - c);
    let (aa, bb, cc, dd, ee) = (u.dot(u), u.dot(v), v.dot(v), u.dot(w), v.dot(w));
    let den = aa * cc - bb * bb;
    let mut s = if den > 1e-9 { ((bb * ee - cc * dd) / den).clamp(0.0, 1.0) } else { 0.0 };
    let t = if cc > 1e-9 { ((bb * s + ee) / cc).clamp(0.0, 1.0) } else { 0.0 };
    if aa > 1e-9 {
        s = ((bb * t - dd) / aa).clamp(0.0, 1.0);
    }
    (a + u * s, c + v * t)
}

fn inst(m: Mat4, tint: [f32; 4]) -> Inst {
    let c = m.to_cols_array_2d();
    Inst { m: [[c[0][0], c[1][0], c[2][0], c[3][0]], [c[0][1], c[1][1], c[2][1], c[3][1]], [c[0][2], c[1][2], c[2][2], c[3][2]]], tint, ext: [0.0; 4] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pack_reads_and_finds_turbines_where_they_are() {
        let d = data();
        assert!(d.len() > 100_000, "{} turbines", d.len());
        // Near Aachen, a much-built-up region: some within 10 km; none in
        // the middle of the Sahara.
        let a = d.near(50.78, 6.08, 10.0);
        assert!(!a.is_empty());
        assert!(d.near(23.0, 12.0, 20.0).is_empty());
        for (_, s) in a {
            assert!((50.6..51.0).contains(&s.lat) && (5.8..6.4).contains(&s.lon), "{s:?}");
            assert!(s.hub >= 15.0 && s.rotor >= 5.0 && s.hub > s.rotor * 0.5, "{s:?}");
        }
    }

    #[test]
    fn the_model_has_its_pieces_and_sizes() {
        let m = models();
        assert_eq!(m.meshes.len(), PIECES.len());
        // The tower reaches the yaw bearing, the rotor's tips 60 m out.
        let top = m.meshes[TOWER as usize].0.iter().map(|v| v.pos[1]).fold(0.0f32, f32::max);
        assert!((top as f64 - TOWER_TOP).abs() < 0.1, "{top}");
        let tip = m.meshes[ROTOR as usize].0.iter().map(|v| (v.pos[0] * v.pos[0] + v.pos[1] * v.pos[1]).sqrt()).fold(0.0f32, f32::max);
        assert!((tip as f64 - REF_D * 0.5).abs() < 1.0, "{tip}");
    }

    #[test]
    fn the_rotor_turns_with_the_wind() {
        let d = 120.0;
        assert_eq!(Turbines::omega(d, 30.0), 0.0);
        assert!(Turbines::omega(d, 2.0) < 0.05);
        let (a, b) = (Turbines::omega(d, 5.0), Turbines::omega(d, 8.0));
        assert!(b > a && a > 0.3, "{a} {b}");
        // Rated: about 12-13 rpm for a rotor of this size.
        let rpm = Turbines::omega(d, 15.0) * 60.0 / std::f64::consts::TAU;
        assert!((11.0..14.0).contains(&rpm), "{rpm}");
    }

    #[test]
    fn an_aircraft_into_a_tower_or_the_rotor_hits_it() {
        let lat: f64 = 50.0;
        let foot = geo::lla_to_pos(lat.to_radians(), 0.0, 100.0);
        let site = Site { lat, lon: 0.0, hub: 100.0, rotor: 120.0 };
        let t = Turbines { near: vec![Near { site, seed: 1, foot: Some((foot, false)) }], ..Default::default() };
        let (e, n, u) = geo::enu(foot);
        // Wind from the north: the rotor faces north.
        let wind = Wind { dir: -n, speed: 8.0 };
        // Through the tower at 50 m, west to east.
        assert!(t.hit(foot + u * 50.0 - e * 30.0, foot + u * 50.0 + e * 30.0, 3.0, &wind).is_some());
        // Through the rotor's disc north to south, 40 m off the hub.
        let hub = foot + u * 100.0;
        assert!(t.hit(hub + e * 40.0 + n * 30.0, hub + e * 40.0 - n * 30.0, 3.0, &wind).is_some());
        // Clear past the tips, and well over the top.
        assert!(t.hit(hub + e * 75.0 + n * 30.0, hub + e * 75.0 - n * 30.0, 3.0, &wind).is_none());
        assert!(t.hit(foot + u * 200.0 - e * 30.0, foot + u * 200.0 + e * 30.0, 3.0, &wind).is_none());
    }
}
