//! Engines and what they drive, one engine at a time.
//!
//! Each engine is simulated on its own: a piston engine breathing through
//! its throttle (piston.rs) turning a propeller (prop.rs), or a gas turbine
//! (jet.rs). An aircraft is only data: what its engines are, where they are
//! mounted and which way they turn. The models here know nothing of any one
//! type; they take the air around them and the pilot's levers and give back
//! a force, a moment about the engine's own axis and what the gauges read.
//!
//! Everything is in SI units. The engine frame is the body frame (+X right,
//! +Y up, -Z forward); a mount's `axis` is the way its thrust pushes.

pub mod jet;
pub mod piston;
pub mod prop;
pub mod unit;

use glam::DVec3;

/// Gas constant of dry air (J/kg K) and its ratio of specific heats.
pub const R_AIR: f64 = 287.052_87;
pub const GAMMA: f64 = 1.4;
/// The standard sea-level pressure (Pa), temperature (K) and density (kg/m³).
pub const P0: f64 = 101_325.0;
pub const T0: f64 = 288.15;
pub const RHO0: f64 = 1.225;

/// The air an engine breathes.
#[derive(Clone, Copy, Debug)]
pub struct Ambient {
    /// static pressure (Pa) and temperature (K)
    pub p: f64,
    pub t: f64,
    /// true airspeed along the engine's axis (m/s), and the Mach number
    pub v: f64,
    pub mach: f64,
}

impl Ambient {
    pub fn rho(&self) -> f64 {
        self.p / (R_AIR * self.t)
    }
    pub fn sound(&self) -> f64 {
        (GAMMA * R_AIR * self.t).sqrt()
    }
    /// The International Standard Atmosphere at pressure height `h` (m), with
    /// the temperature `dev` kelvin off standard, flying at `v` m/s.
    pub fn isa(h: f64, dev: f64, v: f64) -> Ambient {
        let (t_std, p) = if h < 11_000.0 {
            let t = T0 - 0.0065 * h;
            (t, P0 * (t / T0).powf(5.255_88))
        } else {
            let p11 = P0 * (216.65_f64 / T0).powf(5.255_88);
            (216.65, p11 * (-(h - 11_000.0) / 6341.62).exp())
        };
        let t = t_std + dev;
        Ambient { p, t, v, mach: v / (GAMMA * R_AIR * t).sqrt() }
    }
    /// Total (stagnation) temperature and pressure ratios to the standard
    /// sea-level values: the θ0 and δ0 a gas turbine's ratings are given in.
    pub fn theta0(&self) -> f64 {
        self.t / T0 * (1.0 + 0.2 * self.mach * self.mach)
    }
    pub fn delta0(&self) -> f64 {
        self.p / P0 * (1.0 + 0.2 * self.mach * self.mach).powf(3.5)
    }
}

/// Where an engine is and which way it works.
#[derive(Clone, Copy, Debug)]
pub struct Mount {
    /// the propeller hub or the nozzle exit, body frame (m)
    pub at: DVec3,
    /// the direction its thrust pushes, body frame (unit); straight ahead is -Z
    pub axis: DVec3,
    /// which way it turns seen from behind (the cockpit): +1 clockwise, -1 counter
    pub spin: f64,
}

impl Mount {
    pub const fn ahead(x: f64, y: f64, z: f64, spin: f64) -> Mount {
        Mount { at: DVec3::new(x, y, z), axis: DVec3::NEG_Z, spin }
    }
}

/// The engines a type flies with, one per mount, from its data.
pub fn units(spec: &crate::aircraft::Spec) -> Vec<unit::Unit> {
    use crate::aircraft::Engine;
    use unit::{Kind, Unit};
    match &spec.engine {
        Engine::Piston(pe) => {
            let eng = piston::Piston::new(piston::PistonData {
                power: pe.power,
                w_rated: pe.w_rated,
                displacement: pe.litres * 1e-3,
                fric0: pe.fric0,
                fric1: pe.fric1,
                rich_far: 0.085,
                carb: pe.carb,
                boost: pe.boost,
                cowl_flaps: pe.cowl_flaps,
            });
            let prop = prop::Prop::new(prop::PropData { d: pe.prop_d, blades: pe.blades, chord: pe.chord, pitch: pe.pitch, inertia: pe.prop_i, gear: 1.0 });
            let mounts: &[Mount] = if pe.mounts.is_empty() { &[Mount::ahead(0.0, 0.0, -2.0, 1.0)] } else { pe.mounts };
            mounts.iter().map(|m| Unit::new(*m, Kind::Piston { eng: eng.clone(), prop: prop.clone() })).collect()
        }
        Engine::Fan(fe) => {
            let jet = jet::Jet::new(jet::JetData {
                thrust: fe.thrust,
                wet: if fe.boost >= 1.2 { fe.thrust * fe.boost } else { 0.0 },
                bpr: fe.bpr,
                theta_break: fe.theta_break,
                tsfc: fe.tsfc[0] * jet::LB_LBF_H,
                tsfc_wet: fe.tsfc[1] * jet::LB_LBF_H,
                idle: [fe.idle, fe.idle_n2],
                idle_thrust: fe.idle_thrust,
                spool_up: fe.spool[0],
                spool_down: fe.spool[1],
                n1_rpm: fe.n1_rpm,
                reverse: fe.reverse,
                inlet: jet::JetData::inlet_for(fe.thrust, fe.bpr),
                inlet_mach: spec.inlet_mach,
                dry_mach: spec.dry_mach,
            });
            (0..fe.n)
                .map(|i| {
                    // A type without mounts of its own (or fewer than its
                    // engines) has them all on the centreline at the nozzles.
                    let m = fe.mounts.get(i).copied().unwrap_or(Mount::ahead(0.0, 0.0, fe.nozzle_z, 1.0));
                    Unit::new(m, Kind::Jet(jet.clone()))
                })
                .collect()
        }
        Engine::Rocket(_) => Vec::new(),
    }
}

/// Linear interpolation in a table of (x, y) pairs sorted by x, held at the ends.
pub fn interp(table: &[(f64, f64)], x: f64) -> f64 {
    let n = table.len();
    if x <= table[0].0 {
        return table[0].1;
    }
    if x >= table[n - 1].0 {
        return table[n - 1].1;
    }
    let i = table.partition_point(|p| p.0 <= x) - 1;
    let (a, b) = (table[i], table[i + 1]);
    a.1 + (b.1 - a.1) * (x - a.0) / (b.0 - a.0)
}

pub fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// First-order lag of `v` toward `to` with time constant `tau` (s).
pub fn lag(v: f64, to: f64, tau: f64, dt: f64) -> f64 {
    v + (to - v) * (1.0 - (-dt / tau.max(1e-6)).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_atmosphere() {
        let a = Ambient::isa(0.0, 0.0, 0.0);
        assert!((a.rho() - RHO0).abs() < 1e-3);
        // 10,000 ft: 69.68 kPa, 268.3 K (ICAO tables)
        let a = Ambient::isa(3048.0, 0.0, 0.0);
        assert!((a.p - 69_682.0).abs() < 150.0, "{}", a.p);
        assert!((a.t - 268.34).abs() < 0.1);
        // 36,089 ft tropopause and 50,000 ft: 11.6 kPa
        let a = Ambient::isa(15_240.0, 0.0, 0.0);
        assert!((a.p - 11_597.0).abs() < 100.0, "{}", a.p);
    }

    #[test]
    fn interpolates_and_holds() {
        let t = [(0.0, 0.0), (1.0, 10.0), (3.0, 30.0)];
        assert_eq!(interp(&t, -1.0), 0.0);
        assert_eq!(interp(&t, 0.5), 5.0);
        assert_eq!(interp(&t, 2.0), 20.0);
        assert_eq!(interp(&t, 9.0), 30.0);
    }
}
