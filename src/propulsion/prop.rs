//! Propellers: what the blades do, worked out from their shape.
//!
//! When a type is loaded each of its propellers is put through a blade-element
//! momentum analysis (Glauert's, with Prandtl's tip loss): the blade, from its
//! diameter, blade count, chord and pitch, is cut into stations; each sees the
//! air coming at it from ahead (the flight speed plus the flow the disc
//! itself induces) and from its own turning, and lifts and drags like the
//! cambered section it is, stall included. From that come the thrust and
//! power coefficients over the advance ratio J = V/(nD) and the blade angle:
//! fixed-pitch propellers keep one angle, constant-speed ones a range, from
//! reverse to feathered. In flight the tables are read, not recomputed.
//!
//! The disc is then flown in sectors. Coming at it at an angle (climbing at
//! a high angle of attack, or sideslipping) the air adds to the speed of the
//! blades going one way round and takes from the other: the descending blade
//! takes the bigger bite. Each sector's thrust and torque from the tables at
//! its own effective advance ratio gives the off-centre thrust (P-factor)
//! and the sideways force on the disc (its "fin" effect) without any extra
//! rule. The torque that turns the propeller turns the airframe the other
//! way, and its spin is a gyroscope (the rigid body takes care of that).
//!
//! A constant-speed propeller's governor moves the blades to hold the revs
//! the lever sets, as fast as its oil can move them; at the fine stop it is
//! a fixed-pitch propeller again.
use super::{Ambient, Mount};
use glam::DVec3;
use std::collections::HashMap;
use std::f64::consts::{PI, TAU};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pitch {
    /// blades fixed at this geometric pitch (m of advance per turn at 75% radius)
    Fixed(f64),
    /// a governor holding `rpm` (lever low .. high) between the fine and
    /// coarse stops (blade angle at 75% radius, degrees); it feathers to
    /// `feather` degrees with the lever pulled past its low end (0: it cannot)
    Constant { fine: f64, coarse: f64, rpm: [f64; 2], feather: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PropData {
    /// diameter (m) and number of blades
    pub d: f64,
    pub blades: u32,
    /// blade chord at 75% radius as a fraction of the diameter (0.06 to 0.09 for a light aircraft's)
    pub chord: f64,
    pub pitch: Pitch,
    /// propeller and everything turning with it, at the propeller (kg m²)
    pub inertia: f64,
    /// propeller turns per crankshaft turn (1: direct drive)
    pub gear: f64,
}

/// The propeller's coefficients over blade angle and advance ratio.
pub struct Tables {
    /// blade angles at 75% radius (rad), advance ratios
    pub betas: Vec<f64>,
    pub js: Vec<f64>,
    /// thrust and power coefficients, [beta][j]
    pub ct: Vec<Vec<f64>>,
    pub cp: Vec<Vec<f64>>,
}

/// The blade section: a cambered propeller section (Clark Y), lift and drag
/// at angle of attack `a` (rad) through and past the stall.
fn section(a: f64) -> (f64, f64) {
    let a0 = -3.5_f64.to_radians();
    let stall = 14.0_f64.to_radians();
    let lin = 0.9 * TAU * (a - a0);
    // Attached, rounding off into the stall, then a flat plate.
    let plate_cl = 1.1 * (2.0 * a).sin();
    let plate_cd = 0.03 + 1.8 * a.sin().powi(2);
    let k = super::smooth(stall, stall + 8.0_f64.to_radians(), a.abs());
    let cl_att = if a > stall { lin.min(1.25) } else if a < -stall + a0 { lin.max(-0.9) } else { lin };
    let cd_att = 0.009 + 0.008 * (cl_att - 0.4).powi(2);
    (cl_att * (1.0 - k) + plate_cl * k, cd_att * (1.0 - k) + plate_cd * k)
}

/// Thrust and power coefficients at advance ratio `j` and blade angle `beta75` (rad).
fn bem(p: &PropData, beta75: f64, j: f64) -> (f64, f64) {
    const N: usize = 24;
    let r_tip = 0.5 * p.d;
    // Constant geometric pitch along the blade, as for the angle at 75%.
    let pitch = TAU * 0.75 * r_tip * beta75.tan();
    let b = p.blades as f64;
    // Work with n = 1 rev/s, D as given, ρ = 1: the coefficients are what is left.
    let omega = TAU;
    let v = j * p.d;
    let (mut ct, mut cp) = (0.0, 0.0);
    for i in 0..N {
        let x = 0.2 + 0.8 * (i as f64 + 0.5) / N as f64;
        let dr = 0.8 * r_tip / N as f64;
        let r = x * r_tip;
        // Chord: widest inboard, narrowing to the tip.
        let c = p.chord * p.d * ((1.0 - x * x).max(0.02) / (1.0 - 0.5625)).powf(0.3);
        let beta = if beta75.abs() > 1.5 { beta75 } else { (pitch / (TAU * r)).atan() };
        let sigma = b * c / (TAU * r);
        let ut = omega * r;
        // The induced axial velocity where blade lift and momentum agree.
        let blade = |vi: f64| {
            let ua = v + vi;
            let phi = ua.atan2(ut);
            let (cl, cd) = section(beta - phi);
            let w2 = ua * ua + ut * ut;
            // Prandtl's tip and hub (spinner) losses.
            let f = {
                let s = phi.sin().abs().max(1e-3);
                let tip = (-0.5 * b * (1.0 - x) / (x * s)).exp();
                let hub = (-0.5 * b * (x - 0.15) / (0.15 * s)).exp();
                (4.0 / (PI * PI) * tip.clamp(0.0, 1.0).acos() * hub.clamp(0.0, 1.0).acos()).max(0.02)
            };
            let dt = 0.5 * w2 * sigma * TAU * r * (cl * phi.cos() - cd * phi.sin());
            let dq = 0.5 * w2 * sigma * TAU * r * (cl * phi.sin() + cd * phi.cos()) * r;
            // (κ 1.15: the inflow through a real disc is not uniform, which
            // costs induced power, as blade-element analyses of rotors allow.)
            let mom = 4.0 * PI * r * f * (v + vi).abs() * vi / 1.15;
            (dt - mom, dt, dq)
        };
        let (mut lo, mut hi) = (-0.5 * v - 0.5 * ut, 2.0 * v + ut);
        let flo = blade(lo).0;
        let (mut dt, mut dq) = (0.0, 0.0);
        for _ in 0..50 {
            let m = 0.5 * (lo + hi);
            let (res, t_, q_) = blade(m);
            (dt, dq) = (t_, q_);
            if (res > 0.0) == (flo > 0.0) { lo = m } else { hi = m }
        }
        ct += dt * dr;
        cp += dq * dr * omega;
    }
    // T = Ct ρ n² D⁴, P = Cp ρ n³ D⁵ with n = 1, ρ = 1. The analysis leaves
    // out the slipstream's swirl and the spinner and cowling behind the disc,
    // which together cost a real installation about 3% of its thrust. Standing
    // still the slipstream is fastest beside the free stream, so it scrubs the
    // cowling hardest and the thick blade roots work worst: strip analysis
    // over-predicts static thrust by some 10-15% (figures of merit near 0.7
    // against the 0.55-0.6 measured), a loss that is gone by the climb.
    let static_loss = 0.12 * (1.0 - j / 0.6).max(0.0).powi(2);
    ((0.97 - static_loss) * ct / p.d.powi(4), cp / p.d.powi(5))
}

impl PropData {
    /// The blade angle at 75% radius (rad) of a fixed pitch.
    pub fn beta_of_pitch(&self, pitch: f64) -> f64 {
        (pitch / (TAU * 0.375 * self.d)).atan()
    }
    fn betas(&self) -> Vec<f64> {
        match self.pitch {
            Pitch::Fixed(pt) => vec![self.beta_of_pitch(pt)],
            Pitch::Constant { fine, coarse, feather, .. } => {
                let top = if feather > 0.0 { feather } else { coarse };
                let mut b = fine - 4.0;
                let mut v = Vec::new();
                while b < top + 2.0 {
                    v.push(b.to_radians());
                    b += 2.0;
                }
                v
            }
        }
    }
    fn key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        format!("{self:?}").hash(&mut h);
        h.finish()
    }
    /// The coefficient tables (worked out once per propeller and kept).
    pub fn tables(&self) -> Arc<Tables> {
        static CACHE: OnceLock<Mutex<HashMap<u64, Arc<Tables>>>> = OnceLock::new();
        let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        if let Some(t) = cache.lock().unwrap().get(&self.key()) {
            return t.clone();
        }
        let betas = self.betas();
        let js: Vec<f64> = (0..=72).map(|i| i as f64 * 0.05).collect();
        let mut ct = Vec::new();
        let mut cp = Vec::new();
        for &b in &betas {
            let (t, p): (Vec<f64>, Vec<f64>) = js.iter().map(|&j| bem(self, b, j)).unzip();
            ct.push(t);
            cp.push(p);
        }
        let t = Arc::new(Tables { betas, js, ct, cp });
        cache.lock().unwrap().insert(self.key(), t.clone());
        t
    }
}

impl Tables {
    /// Ct and Cp at advance ratio `j`, blade angle `beta` (rad).
    pub fn at(&self, beta: f64, j: f64) -> (f64, f64) {
        let row = |k: usize| {
            let jj = j.clamp(0.0, *self.js.last().unwrap());
            let dj = self.js[1] - self.js[0];
            let i = ((jj / dj) as usize).min(self.js.len() - 2);
            let f = (jj - self.js[i]) / dj;
            (self.ct[k][i] * (1.0 - f) + self.ct[k][i + 1] * f, self.cp[k][i] * (1.0 - f) + self.cp[k][i + 1] * f)
        };
        if self.betas.len() == 1 {
            return row(0);
        }
        let b = beta.clamp(self.betas[0], *self.betas.last().unwrap());
        let i = (self.betas.partition_point(|&x| x <= b).max(1) - 1).min(self.betas.len() - 2);
        let f = (b - self.betas[i]) / (self.betas[i + 1] - self.betas[i]);
        let (a, c) = (row(i), row(i + 1));
        (a.0 * (1.0 - f) + c.0 * f, a.1 * (1.0 - f) + c.1 * f)
    }
}

/// One propeller as it turns.
#[derive(Clone)]
pub struct Prop {
    pub data: PropData,
    pub tables: Arc<Tables>,
    /// blade angle at 75% radius (rad), and its speed (rad/s)
    pub beta: f64,
    pub w: f64,
    /// the revs at the governor's last look
    w_prev: f64,
}

/// What the propeller does to the airframe this step (body frame).
#[derive(Clone, Copy, Debug, Default)]
pub struct PropOut {
    /// force (N) at the hub, and the moment about the hub (N m): the
    /// off-centre thrust and the reaction to the torque turning it
    pub force: DVec3,
    pub moment: DVec3,
    /// thrust along the axis (N) and the torque the blades take (N m, at the propeller)
    pub thrust: f64,
    pub torque: f64,
}

impl Prop {
    pub fn new(data: PropData) -> Prop {
        let tables = data.tables();
        let beta = match data.pitch {
            Pitch::Fixed(p) => data.beta_of_pitch(p),
            Pitch::Constant { fine, .. } => fine.to_radians(),
        };
        Prop { data, tables, beta, w: 0.0, w_prev: 0.0 }
    }

    /// The governor's revs for a lever at `lever` (0..1), or None to feather.
    pub fn governed(&self, lever: f64) -> Option<f64> {
        match self.data.pitch {
            Pitch::Fixed(_) => None,
            Pitch::Constant { rpm, feather, .. } => {
                if feather > 0.0 && lever < 0.02 {
                    None
                } else {
                    Some((rpm[0] + (rpm[1] - rpm[0]) * lever.clamp(0.0, 1.0)) * TAU / 60.0)
                }
            }
        }
    }

    /// Move the blades: the governor's flyweights against the speeder spring
    /// open its pilot valve as far as the revs are off, and oil flows to the
    /// hub as fast as that lets it (so the blades move at a rate set by the
    /// error, an integral action); the flyweights' own swing as the revs
    /// change damps it. The oil pressure is there only with the engine
    /// turning.
    pub fn govern(&mut self, lever: f64, oil: bool, dt: f64) {
        let Pitch::Constant { fine, coarse, feather, .. } = self.data.pitch else { return };
        let (fine, coarse) = (fine.to_radians(), coarse.to_radians());
        let max_rate = 20.0_f64.to_radians();
        let rate = match self.governed(lever) {
            Some(target) if oil => {
                let e = (self.w - target) / target;
                let de = if dt > 0.0 { (self.w - self.w_prev) / target / dt } else { 0.0 };
                (2.5 * e + 0.25 * de).clamp(-max_rate, max_rate)
            }
            // Without oil pressure a single's blades go to fine (a twin's springs feather them).
            Some(_) => if feather > 0.0 { max_rate } else { -max_rate },
            None => max_rate,
        };
        self.w_prev = self.w;
        let hi = if feather > 0.0 && (self.governed(lever).is_none() || !oil || self.beta > coarse + 0.01) { feather.to_radians() } else { coarse };
        self.beta = (self.beta + rate * dt).clamp(fine, hi);
    }

    /// The loads with the air at `amb` coming at it with velocity `vb` (body
    /// frame, the aircraft's velocity through the air), turning at `self.w`.
    pub fn loads(&self, mount: &Mount, vb: DVec3, amb: &Ambient, k: f64) -> PropOut {
        const SECTORS: usize = 8;
        let d = self.data.d;
        let rho = amb.rho();
        let n = self.w / TAU;
        let axis = mount.axis;
        let v_ax = vb.dot(axis);
        // Two directions in the disc: "up" and "right" as seen from behind.
        let up0 = (DVec3::Y - axis * axis.y).normalize_or(DVec3::Y);
        let right0 = axis.cross(up0);
        let r_c = 0.75 * 0.5 * d;
        let tip_mach = ((PI * n * d).powi(2) + v_ax * v_ax).sqrt() / amb.sound();
        // Past the critical Mach at the tips, drag rises and thrust falls.
        let mx = (tip_mach - 0.85).max(0.0);
        let (kt, kp) = (1.0 - 1.5 * mx * mx, 1.0 + 6.0 * mx * mx);
        let (mut force, mut moment, mut thrust, mut torque) = (DVec3::ZERO, DVec3::ZERO, 0.0, 0.0);
        for s in 0..SECTORS {
            let psi = TAU * (s as f64 + 0.5) / SECTORS as f64;
            // Radial direction of this sector, and the way a blade there moves.
            let radial = up0 * psi.cos() + right0 * psi.sin();
            let tangent = (axis.cross(radial) * mount.spin).normalize();
            // The airflow across the disc adds to (or takes from) the blade's own speed.
            let n_eff = n + vb.dot(tangent) / (TAU * r_c);
            let (t, q) = if n_eff.abs() < 0.05 {
                // Stopped (or as good as): the blades a flat plate edgewise.
                (-0.5 * rho * v_ax * v_ax.abs() * 0.05 * d * d / SECTORS as f64, 0.0)
            } else {
                let je = v_ax.max(0.0) / (n_eff * d);
                let (ct, cp) = self.tables.at(self.beta, je);
                let t = ct * kt * rho * n_eff * n_eff * d.powi(4) / SECTORS as f64;
                let q = cp * kp * rho * n_eff * n_eff * d.powi(5) / (TAU * SECTORS as f64);
                (t, q)
            };
            // Windmilling fast, a stopped or feathered propeller still drags no worse
            // than a plate of its blade area.
            let t = t.max(-0.5 * rho * v_ax * v_ax * 0.1 * d * d / SECTORS as f64);
            thrust += t;
            torque += q;
            force += axis * t;
            // The blade's drag (from the torque) pushes the disc against the blade's motion.
            force -= tangent * (q / r_c);
            moment += (radial * r_c).cross(axis * t);
        }
        // The engine turns the propeller one way; the airframe is turned the other.
        moment -= axis * (mount.spin * torque);
        PropOut { force: force * k, moment: moment * k, thrust: thrust * k, torque: torque * k }
    }
}

/// Efficiency η = J Ct / Cp at advance ratio `j` and blade angle `beta`.
pub fn efficiency(t: &Tables, beta: f64, j: f64) -> f64 {
    let (ct, cp) = t.at(beta, j);
    if cp > 1e-6 { j * ct / cp } else { 0.0 }
}

/// A fixed-pitch propeller's static revs, against a constant engine power (W):
/// for checking a pitch against a type's quoted static rpm.
pub fn static_rpm(p: &PropData, power: f64, rho: f64) -> f64 {
    let t = p.tables();
    let beta = match p.pitch {
        Pitch::Fixed(pt) => p.beta_of_pitch(pt),
        Pitch::Constant { fine, .. } => fine.to_radians(),
    };
    let cp = t.at(beta, 0.0).1;
    // P = Cp ρ n³ D⁵
    (power / (cp * rho * p.d.powi(5))).cbrt() * 60.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::propulsion::Ambient;

    fn fixed(d: f64, pitch: f64) -> PropData {
        PropData { d, blades: 2, chord: 0.075, pitch: Pitch::Fixed(pitch), inertia: 3.0, gear: 1.0 }
    }

    #[test]
    fn static_coefficients_are_a_real_propellers() {
        // A 76 x 60 two-blade metal propeller (a 172's): static Ct about 0.1,
        // Cp about 0.045 (NACA TR 640 for Clark Y blades at 15–20° at 75% radius).
        let p = fixed(1.93, 1.52);
        let t = p.tables();
        let beta = p.beta_of_pitch(1.52);
        assert!((15.0..22.0).contains(&beta.to_degrees()), "{}", beta.to_degrees());
        let (ct, cp) = t.at(beta, 0.0);
        assert!((0.07..0.14).contains(&ct), "ct {ct}");
        assert!((0.03..0.07).contains(&cp), "cp {cp}");
    }

    #[test]
    fn efficiency_peaks_where_it_should() {
        let p = fixed(1.93, 1.52);
        let t = p.tables();
        let beta = p.beta_of_pitch(1.52);
        let (mut best, mut at) = (0.0, 0.0);
        for i in 1..30 {
            let j = i as f64 * 0.05;
            let e = efficiency(&t, beta, j);
            if e > best {
                (best, at) = (e, j);
            }
        }
        // Two-blade metal propellers: about 80–87% at a J a little under P/D.
        assert!((0.78..0.89).contains(&best), "{best} at {at}");
        assert!((0.55..0.85).contains(&at), "{at}");
        // Past the J where thrust runs out it windmills: thrust and power both negative.
        let (ct, cp) = t.at(beta, 1.2);
        assert!(ct < 0.0 && cp < 0.0, "{ct} {cp}");
    }

    #[test]
    fn static_rpm_of_a_172() {
        // A 172S (160 hp, 1C235 75 x 60): about 2,300–2,400 rpm static on a full-throttle run-up.
        let p = fixed(1.905, 1.524);
        let rpm = static_rpm(&p, 0.86 * 119_000.0, 1.225);
        assert!((2100.0..2550.0).contains(&rpm), "{rpm}");
    }

    #[test]
    fn coarser_blades_take_more_power() {
        let p = PropData { d: 1.95, blades: 2, chord: 0.075, pitch: Pitch::Constant { fine: 12.0, coarse: 32.0, rpm: [2100.0, 2700.0], feather: 0.0 }, inertia: 3.0, gear: 1.0 };
        let t = p.tables();
        let a = t.at(14.0_f64.to_radians(), 0.5).1;
        let b = t.at(24.0_f64.to_radians(), 0.5).1;
        assert!(b > a * 1.5, "{a} {b}");
    }

    #[test]
    fn p_factor_and_torque_go_the_right_way() {
        // Clockwise from the cockpit, climbing at 12° angle of attack: the
        // right (descending) blade bites harder, yawing the nose left.
        let mut p = Prop::new(fixed(1.93, 1.52));
        p.w = 2500.0 * TAU / 60.0;
        let mount = Mount::ahead(0.0, 0.0, -2.0, 1.0);
        let a = 12.0_f64.to_radians();
        let v = 35.0;
        let vb = DVec3::new(0.0, -v * a.sin(), -v * a.cos());
        let amb = Ambient::isa(0.0, 0.0, v);
        let o = p.loads(&mount, vb, &amb, 1.0);
        assert!(o.thrust > 1000.0, "{}", o.thrust);
        // Yaw left about the hub: positive about +Y (body +Y up; nose to -X is left).
        assert!(o.moment.y > 0.0, "yaw {}", o.moment.y);
        // The torque reaction rolls it left (about +Z the right wing rises).
        assert!(o.moment.z > 0.0, "roll {}", o.moment.z);
        // The disc's normal force is upward with the air coming from below.
        assert!(o.force.y > 0.0, "normal force {}", o.force.y);
        // Mirror image for a propeller turning the other way.
        let o2 = p.loads(&Mount::ahead(0.0, 0.0, -2.0, -1.0), vb, &amb, 1.0);
        assert!(o2.moment.y < 0.0 && o2.moment.z < 0.0);
        // Straight ahead, no asymmetry.
        let o3 = p.loads(&mount, DVec3::new(0.0, 0.0, -v), &amb, 1.0);
        assert!(o3.moment.y.abs() < 1.0 && o3.force.y.abs() < 1.0, "{:?}", o3.moment);
    }

    #[test]
    fn governor_holds_the_revs() {
        let data = PropData { d: 1.95, blades: 2, chord: 0.075, pitch: Pitch::Constant { fine: 12.0, coarse: 32.0, rpm: [2100.0, 2700.0], feather: 0.0 }, inertia: 3.0, gear: 1.0 };
        let mut p = Prop::new(data);
        // A constant 120 kW turning it at 55 m/s, governor at 2,400 rpm.
        let amb = Ambient::isa(0.0, 0.0, 55.0);
        let mount = Mount::ahead(0.0, 0.0, -2.0, 1.0);
        p.w = 2000.0 * TAU / 60.0;
        let lever = 0.5;
        let dt = 1.0 / 120.0;
        for _ in 0..(120 * 20) {
            let o = p.loads(&mount, DVec3::new(0.0, 0.0, -55.0), &amb, 1.0);
            let q_eng = 120_000.0 / p.w.max(10.0);
            p.w += (q_eng - o.torque) / p.data.inertia * dt;
            p.govern(lever, true, dt);
        }
        let rpm = p.w * 60.0 / TAU;
        assert!((rpm - 2400.0).abs() < 25.0, "{rpm}");
        assert!(p.beta.to_degrees() > 12.5 && p.beta.to_degrees() < 32.0);
    }
}
