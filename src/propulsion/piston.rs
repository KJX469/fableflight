//! A four-stroke piston engine, breathing through its throttle.
//!
//! A mean-value model: what matters is the air the cylinders draw in and the
//! fuel the carburettor or injector meters to it, not each firing.
//!
//! - **Manifold pressure.** The cylinders swallow `VE · ρ_manifold · V_d · n/2`
//!   of air; the throttle plate passes what a compressible orifice of its open
//!   area lets through from the air upstream. Where the two flows match is the
//!   manifold pressure: about the outside air's at full throttle, a third of
//!   it at idle, and lower the faster the engine turns at a part throttle, as
//!   on the gauge. A turbocharger or supercharger raises the air upstream.
//! - **Mixture.** The carburettor (or the injector's servo) meters fuel by the
//!   pressure drop across its venturi, so with the lever where it was the
//!   mixture grows richer as the air thins (the fuel/air ratio goes as
//!   1/√ρ). The lever leans it from there. Power follows the equivalence ratio
//!   φ (fuel/air over stoichiometric): best power about 15% rich of
//!   stoichiometric, peak exhaust temperature at it, a few percent less power
//!   full rich at sea level and more at height, lean of peak less again, and
//!   it misfires and stops past the lean and rich limits.
//! - **Losses.** Friction rises with the revs; pumping is the manifold vacuum
//!   working against the pistons, so a throttled engine wastes more.
//! - **Heat.** Exhaust gas temperature follows φ (hottest at stoichiometric)
//!   and the load; the cylinder heads and oil warm with the heat the fuel
//!   gives up and cool with the air through the cowling (more with the cowl
//!   flaps open, the propeller's slipstream helping on the ground).
//!
//! Only the engine's rated power, revs and displacement are needed; the
//! efficiency and the throttle's sizes are found from them when it is built
//! (`Piston::new`), so a downloaded aircraft with nothing but a rating flies
//! on the same physics.
use super::{interp, lag, smooth, Ambient, GAMMA, P0, RHO0, R_AIR, T0};
use std::f64::consts::{PI, TAU};

/// Avgas: lower heating value (J/kg) and stoichiometric fuel/air ratio.
pub const LHV: f64 = 43.5e6;
pub const FAR_ST: f64 = 0.0676;

/// Brake power relative to best power against the equivalence ratio φ
/// (Lycoming and Continental operator's manuals' leaning curves, and Taylor,
/// *The Internal-Combustion Engine in Theory and Practice* vol. 1 fig. 13-6):
/// best power near φ 1.15, 96–97% at peak EGT (φ 1), lean of peak falling
/// away to the lean misfire limit, rich falling more slowly to flooding.
const POWER_PHI: [(f64, f64); 14] = [
    (0.55, 0.0),
    (0.62, 0.35),
    (0.70, 0.64),
    (0.80, 0.81),
    (0.90, 0.915),
    (1.00, 0.968),
    (1.10, 0.996),
    (1.16, 1.0),
    (1.25, 0.992),
    (1.40, 0.958),
    (1.60, 0.89),
    (1.80, 0.79),
    (2.05, 0.6),
    (2.40, 0.0),
];
/// The equivalence ratio where power is best.
pub const PHI_BEST: f64 = 1.16;

#[derive(Clone, Copy, Debug)]
pub enum Boost {
    /// normally aspirated
    None,
    /// a turbocharger whose wastegate holds `max_map` (Pa) at full throttle up
    /// to its critical height `critical` (m pressure height)
    Turbo { max_map: f64, critical: f64 },
    /// a gear-driven supercharger: pressure ratio `ratio` at rated revs
    Super { ratio: f64 },
}

#[derive(Clone, Debug)]
pub struct PistonData {
    /// rated brake power (W) at rated revs (rad/s), full throttle, sea level,
    /// standard day, best-power mixture
    pub power: f64,
    pub w_rated: f64,
    /// swept volume of all the cylinders (m³)
    pub displacement: f64,
    /// friction: constant and at rated revs, growing with the square of the revs (N m)
    pub fric0: f64,
    pub fric1: f64,
    /// fuel/air ratio with the mixture full rich at sea level (about 0.085, φ ≈ 1.26)
    pub rich_far: f64,
    /// a carburettor (it can ice; carb heat warms its air) rather than injection
    pub carb: bool,
    pub boost: Boost,
    /// it has cowl flaps
    pub cowl_flaps: bool,
}

impl PistonData {
    /// A normally aspirated engine of `power` W at `rpm` from `litres` of cylinders.
    pub fn new(power: f64, rpm: f64, litres: f64) -> PistonData {
        PistonData {
            power,
            w_rated: rpm * TAU / 60.0,
            displacement: litres * 1e-3,
            fric0: 0.05 * power / (rpm * TAU / 60.0),
            fric1: 0.07 * power / (rpm * TAU / 60.0),
            rich_far: 0.085,
            carb: true,
            boost: Boost::None,
            cowl_flaps: false,
        }
    }
    /// Displacement from power for a downloaded engine that does not give it:
    /// about 31 kW per litre for aero engines at 2,700 rpm (O-320 to IO-540).
    pub fn displacement_for(power: f64, rpm: f64) -> f64 {
        power / 31_000.0 * (2700.0 / rpm.max(1500.0)) * 1e-3
    }
}

/// What the pilot and the airframe give the engine.
#[derive(Clone, Copy, Debug)]
pub struct PistonIn {
    /// throttle (0 idle stop .. 1 full) and mixture lever (0 idle cut-off .. 1 full rich)
    pub throttle: f64,
    pub mixture: f64,
    /// crankshaft speed (rad/s)
    pub w: f64,
    pub amb: Ambient,
    /// magnetos: how many are on (0, 1, 2)
    pub mags: u8,
    /// fuel is reaching it
    pub fuel: bool,
    /// carburettor heat (0..1) and cowl flaps (0 shut .. 1 open)
    pub carb_heat: f64,
    pub cowl: f64,
    /// the dynamic pressure of the air the cowling sees (Pa): the free stream's plus the slipstream's
    pub q_cool: f64,
    /// what is left of it (damage, 0..1)
    pub health: f64,
}

/// What it gives back, and what its gauges read.
#[derive(Clone, Copy, Debug, Default)]
pub struct PistonOut {
    /// torque at the crankshaft (N m): what the cylinders make less friction
    /// and pumping (negative when not firing: what it takes to turn it)
    pub torque: f64,
    /// shaft power (W, 0 when it is being turned rather than turning)
    pub power: f64,
    /// manifold pressure (Pa) and fuel flow (kg/s)
    pub map: f64,
    pub fuel_flow: f64,
    /// equivalence ratio of what it is burning
    pub phi: f64,
    /// it is firing, and how roughly (0 smooth .. 1 about to quit)
    pub firing: bool,
    pub rough: f64,
}

/// One engine: its data, what was worked out from it, and its temperatures.
#[derive(Clone, Debug)]
pub struct Piston {
    pub data: PistonData,
    /// indicated efficiency at best power, and the throttle's open and closed
    /// effective areas (m²)
    eta: f64,
    a_max: f64,
    a_idle: f64,
    /// what the turbocharger is giving (pressure ratio), lagging the demand
    pub boost_pr: f64,
    /// exhaust gas, cylinder head and oil temperatures (K)
    pub egt: f64,
    pub cht: f64,
    pub oil_t: f64,
    pub last: PistonOut,
}

/// Mass flow (kg/s per m² of throttle) of air at `p`, `t` through an orifice
/// to the pressure ratio `pr` downstream (choked below 0.528).
fn orifice(p: f64, t: f64, pr: f64) -> f64 {
    let pr = pr.max(0.528);
    let psi = (2.0 * GAMMA / (GAMMA - 1.0) * (pr.powf(2.0 / GAMMA) - pr.powf((GAMMA + 1.0) / GAMMA))).max(0.0).sqrt();
    0.8 * p / (R_AIR * t).sqrt() * psi
}

/// Volumetric efficiency at shaft speed `w` and manifold pressure `map`,
/// with the charge at `t` and exhausting into `p_exh`: falls a little off the
/// revs it is tuned for, at a deep vacuum more exhaust stays in the cylinder,
/// and a cold charge is warmed more by the hot ports and cylinder walls, so
/// that the air drawn in goes as p/√T rather than p/T (the SAE and FAA
/// standard-day power correction, δ·√(T_std/T)).
fn ve(d: &PistonData, w: f64, map: f64, t: f64, p_exh: f64) -> f64 {
    let x = w / d.w_rated - 0.85;
    0.84 * (1.0 - 0.25 * x * x) * (1.0 - 0.07 * (p_exh / map.max(1.0) - 1.0).max(0.0)) * (t / T0).sqrt()
}

impl Piston {
    pub fn new(data: PistonData) -> Piston {
        let mut p = Piston { data, eta: 0.3, a_max: 1.0, a_idle: 0.01, boost_pr: 1.0, egt: T0, cht: T0, oil_t: T0, last: PistonOut::default() };
        let d = &p.data;
        // Upstream of the throttle at sea level, rated revs, wide open.
        let p_up = match d.boost {
            Boost::Turbo { max_map, .. } => max_map / 0.97,
            Boost::Super { ratio } => P0 * ratio,
            Boost::None => P0,
        };
        let t_up = T0 * Self::compress_t(p_up / P0);
        p.boost_pr = p_up / P0;
        // Size the throttle: wide open at rated revs it drops 3% of the air's
        // pressure; closed at 650 rpm it holds the manifold to a third.
        let demand = |w: f64, map: f64| ve(d, w, map, t_up, P0 * 1.05) * map / (R_AIR * t_up) * d.displacement * w / (4.0 * PI);
        p.a_max = demand(d.w_rated, 0.97 * p_up) / orifice(p_up, t_up, 0.97);
        let w_idle = 650.0 * TAU / 60.0;
        p.a_idle = demand(w_idle, 0.33 * P0) / orifice(P0, T0, 0.33);
        // The efficiency that makes rated power at best power there.
        let map = 0.97 * p_up;
        let m_air = demand(d.w_rated, map);
        let pump = (P0 * 1.05 - map).max(0.0) * d.displacement / (4.0 * PI);
        let fric = d.fric0 + d.fric1;
        p.eta = (d.power + (fric + pump) * d.w_rated) / (m_air * FAR_ST * PHI_BEST * LHV);
        p
    }

    /// Indicated efficiency worked out from the rating (0.3 to 0.38 for a real engine).
    pub fn efficiency(&self) -> f64 {
        self.eta
    }

    /// Temperature ratio across a compressor of 70% efficiency at pressure ratio `pr`.
    fn compress_t(pr: f64) -> f64 {
        1.0 + (pr.max(1.0).powf((GAMMA - 1.0) / GAMMA) - 1.0) / 0.7
    }

    /// The pressure ratio the turbo or supercharger would give now.
    fn boost_target(&self, amb: &Ambient, w: f64, throttle: f64) -> f64 {
        match self.data.boost {
            Boost::None => 1.0,
            Boost::Super { ratio } => 1.0 + (ratio - 1.0) * (w / self.data.w_rated).powi(2),
            Boost::Turbo { max_map, critical } => {
                // The wastegate holds the deck pressure for the throttle's
                // demand; the most the turbine can give is set by the height
                // where it can just make `max_map` (and the exhaust driving it).
                let p_crit = Ambient::isa(critical, 0.0, 0.0).p;
                let most = (max_map / 0.97 / p_crit) * smooth(0.25, 0.8, w / self.data.w_rated).max(0.05);
                let want = (max_map / 0.97 / amb.p) * (0.35 + 0.65 * throttle);
                want.min(most).max(1.0)
            }
        }
    }

    /// The air upstream of the throttle plate: pressure and temperature.
    fn upstream(&self, amb: &Ambient, carb_heat: f64) -> (f64, f64) {
        // A little ram pressure from the intake facing forward.
        let ram = 0.5 * amb.rho() * amb.v * amb.v * 0.5;
        let p = (amb.p + ram) * self.boost_pr;
        let mut t = amb.t * Self::compress_t(self.boost_pr);
        if self.data.carb {
            t += 50.0 * carb_heat.clamp(0.0, 1.0);
        }
        (p, t)
    }

    /// The fuel/air ratio the metering gives with the lever at `lever`
    /// (0 cut-off) for air of density `rho` upstream.
    pub fn far(&self, lever: f64, rho: f64) -> f64 {
        if lever < 0.03 {
            return 0.0;
        }
        self.data.rich_far * (RHO0 / rho.max(0.01)).sqrt() * (0.3 + 0.7 * lever.min(1.0))
    }

    /// The lever setting that gives the equivalence ratio `phi` in this air
    /// (the auto-mixture assist's), held to the lever's travel.
    pub fn lever_for(&self, phi: f64, amb: &Ambient, carb_heat: f64) -> f64 {
        let (p, t) = self.upstream(amb, carb_heat);
        let rich = self.far(1.0, p / (R_AIR * t));
        ((phi * FAR_ST / rich - 0.3) / 0.7).clamp(0.04, 1.0)
    }

    /// Manifold pressure (Pa) and air mass flow (kg/s) with the throttle at
    /// `throttle`, turning at `w`, the air upstream at `p_up`, `t_up`.
    fn breathe(&self, throttle: f64, w: f64, p_up: f64, t_up: f64, p_exh: f64) -> (f64, f64) {
        let d = &self.data;
        // The butterfly's open area grows as 1 - cos of its angle.
        let th = throttle.clamp(0.0, 1.0);
        let area = self.a_idle + (self.a_max - self.a_idle) * (1.0 - (th * PI / 2.0).cos());
        let demand = |map: f64| ve(d, w, map, t_up, p_exh) * map / (R_AIR * t_up) * d.displacement * w.max(0.0) / (4.0 * PI);
        if w < 1.0 {
            return (p_up, 0.0);
        }
        let (mut lo, mut hi) = (0.02, 1.0);
        for _ in 0..32 {
            let pr = 0.5 * (lo + hi);
            if area * orifice(p_up, t_up, pr) > demand(pr * p_up) { lo = pr } else { hi = pr }
        }
        let map = 0.5 * (lo + hi) * p_up;
        (map, demand(map))
    }

    /// Friction torque at `w` (N m): what it takes to turn it over, firing or not.
    pub fn friction(&self, w: f64) -> f64 {
        self.data.fric0 + self.data.fric1 * (w / self.data.w_rated).powi(2)
    }

    /// One step: the torque at the crankshaft and the gauges.
    pub fn step(&mut self, inp: &PistonIn, dt: f64) -> PistonOut {
        let d = self.data.clone();
        let amb = inp.amb;
        let target = self.boost_target(&amb, inp.w, inp.throttle);
        // A turbocharger takes a couple of seconds to spool; a supercharger is geared.
        self.boost_pr = match d.boost {
            Boost::Turbo { .. } => lag(self.boost_pr, target, 2.0, dt),
            _ => target,
        };
        let (p_up, t_up) = self.upstream(&amb, inp.carb_heat);
        let p_exh = amb.p * 1.05 + if matches!(d.boost, Boost::Turbo { .. }) { (self.boost_pr - 1.0) * amb.p * 0.8 } else { 0.0 };
        let (map, m_air) = self.breathe(inp.throttle, inp.w, p_up, t_up, p_exh);
        let far = if inp.fuel { self.far(inp.mixture, p_up / (R_AIR * t_up)) } else { 0.0 };
        let phi = far / FAR_ST;
        let rel = interp(&POWER_PHI, phi);
        let firing = inp.mags > 0 && rel > 0.0 && inp.w > 8.0 && inp.health > 0.0;
        // One magneto fires each cylinder from one plug: the flame takes longer
        // across it, a few percent down (the run-up's mag drop).
        let mags = if inp.mags >= 2 { 1.0 } else { 0.96 };
        let p_ind = if firing { m_air * FAR_ST * PHI_BEST * LHV * self.eta * rel * mags * inp.health } else { 0.0 };
        let w = inp.w.max(1.0);
        let pump = (p_exh - map).max(0.0) * d.displacement / (4.0 * PI);
        let torque = p_ind / w - self.friction(inp.w) - if inp.w > 0.5 { pump } else { 0.0 };
        let fuel_flow = if firing || (inp.fuel && inp.w > 8.0 && inp.mixture > 0.03) { m_air * far } else { 0.0 };
        // Near either limit it runs rough (and power comes and goes).
        let rough = if firing { 1.0 - smooth(0.6, 0.78, phi) * (1.0 - smooth(1.75, 2.2, phi)) } else { 0.0 };

        // Temperatures. Exhaust: hottest burning at stoichiometric, cooler rich
        // (the extra fuel takes heat to evaporate) and lean (less to burn).
        let load = (p_ind / (d.power * 1.15)).clamp(0.0, 1.5);
        let egt_rise = if firing {
            let k = if phi > 1.0 { 1.6 } else { 1.25 };
            (620.0 + 180.0 * load) * (1.0 - k * (phi - 1.0).powi(2)).max(0.2)
        } else {
            0.0
        };
        self.egt = lag(self.egt, t_up + egt_rise, 4.0, dt);
        // Heads and oil: the heat in against the cooling air across the fins
        // (more with the cowl flaps open, and some by convection alone).
        let heat = if firing { load.powf(0.6) * (1.0 - 0.9 * (phi - 1.05).powi(2)).max(0.3) } else { 0.0 };
        let cowl = if d.cowl_flaps { 0.55 + 0.45 * inp.cowl } else { 0.8 };
        let cool = (cowl * inp.q_cool.max(0.0) / 1500.0).powf(0.3) + 0.1;
        self.cht = lag(self.cht, amb.t + 234.0 * heat / cool, 50.0, dt);
        self.oil_t = lag(self.oil_t, amb.t + if firing { (32.0 + 55.0 * load) / cool.sqrt() } else { 0.0 }, 240.0, dt);

        self.last = PistonOut { torque, power: (torque * inp.w).max(0.0), map, fuel_flow, phi, firing, rough };
        self.last
    }

    /// Warm it up as if it had been running at this power (a spawn in flight
    /// or on the runway ready to go).
    pub fn warm(&mut self, inp: &PistonIn) {
        for _ in 0..600 {
            self.step(inp, 1.0);
        }
    }

    /// Cold, as it is before the morning's first flight.
    pub fn cold(&mut self, t: f64) {
        (self.egt, self.cht, self.oil_t) = (t, t, t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Lycoming O-360-A/F (180 hp at 2,700 rpm, 361 cu in, carburetted).
    fn o360() -> Piston {
        Piston::new(PistonData::new(134_226.0, 2700.0, 5.916))
    }
    fn run(e: &mut Piston, throttle: f64, mixture: f64, rpm: f64, amb: Ambient) -> PistonOut {
        let inp = PistonIn { throttle, mixture, w: rpm * TAU / 60.0, amb, mags: 2, fuel: true, carb_heat: 0.0, cowl: 0.0, q_cool: 2000.0, health: 1.0 };
        e.step(&inp, 0.01)
    }
    fn best_power(e: &mut Piston, throttle: f64, rpm: f64, amb: Ambient) -> (PistonOut, f64) {
        let lever = e.lever_for(PHI_BEST, &amb, 0.0);
        (run(e, throttle, lever, rpm, amb), lever)
    }

    #[test]
    fn efficiency_is_an_engines() {
        let e = o360();
        assert!((0.28..0.40).contains(&e.efficiency()), "{}", e.efficiency());
    }

    #[test]
    fn rated_power_at_sea_level_best_power() {
        let mut e = o360();
        let (o, _) = best_power(&mut e, 1.0, 2700.0, Ambient::isa(0.0, 0.0, 0.0));
        assert!((o.power / 134_226.0 - 1.0).abs() < 0.01, "{}", o.power / 745.7);
        // Full throttle: the manifold about the air's own (29.9 inHg less an inch or so).
        let inhg = o.map / 3386.4;
        assert!((28.0..29.9).contains(&inhg), "{inhg}");
        // Best power burns ~0.47 lb/hp/h (Lycoming O-360 curves: ~14 gph at 180 hp).
        let bsfc = o.fuel_flow * 3600.0 * 2.2046 / (o.power / 745.7);
        assert!((0.42..0.52).contains(&bsfc), "{bsfc}");
    }

    #[test]
    fn idle_manifold_and_part_throttle() {
        let mut e = o360();
        let amb = Ambient::isa(0.0, 0.0, 0.0);
        let idle = run(&mut e, 0.0, 1.0, 650.0, amb);
        let inhg = idle.map / 3386.4;
        assert!((9.0..12.5).contains(&inhg), "idle {inhg}");
        // More revs at the same throttle draw the manifold down.
        let a = run(&mut e, 0.35, 1.0, 2000.0, amb).map;
        let b = run(&mut e, 0.35, 1.0, 2500.0, amb).map;
        assert!(b < a - 500.0, "{a} {b}");
    }

    #[test]
    fn power_falls_with_height_and_leaning_wins_some_back() {
        let mut e = o360();
        let sl = run(&mut e, 1.0, 1.0, 2700.0, Ambient::isa(0.0, 0.0, 0.0)).power;
        let amb = Ambient::isa(3048.0, 0.0, 0.0);
        let rich = run(&mut e, 1.0, 1.0, 2700.0, amb);
        let (lean, _) = best_power(&mut e, 1.0, 2700.0, amb);
        // Full rich at 10,000 ft it is much richer than at sea level...
        assert!(rich.phi > 1.38, "{}", rich.phi);
        // ...and leaning to best power gains a few percent.
        assert!(lean.power > rich.power * 1.03, "{} {}", lean.power, rich.power);
        // At 10,000 ft (σ 0.74) a normally aspirated engine makes about 70%
        // of sea-level power (Gagg-Ferrar: 1.132σ - 0.132 = 0.705).
        let r = lean.power / sl;
        assert!((0.64..0.76).contains(&r), "{r}");
    }

    #[test]
    fn hot_day_costs_power() {
        let mut e = o360();
        let std = best_power(&mut e, 1.0, 2700.0, Ambient::isa(0.0, 0.0, 0.0)).0.power;
        let hot = best_power(&mut e, 1.0, 2700.0, Ambient::isa(0.0, 20.0, 0.0)).0.power;
        // 20 K hotter: about 3.5% less dense, about 4% less power.
        assert!(hot < std * 0.975 && hot > std * 0.93, "{}", hot / std);
    }

    #[test]
    fn egt_peaks_at_stoichiometric_and_mixture_limits_kill_it() {
        let mut e = o360();
        let amb = Ambient::isa(1500.0, 0.0, 0.0);
        let mut best = (0.0, 0.0);
        let mut best_pow = (0.0, 0.0);
        let mut lever = 1.0;
        while lever > 0.05 {
            let inp = PistonIn { throttle: 0.7, mixture: lever, w: 2400.0 * TAU / 60.0, amb, mags: 2, fuel: true, carb_heat: 0.0, cowl: 0.0, q_cool: 2000.0, health: 1.0 };
            e.warm(&inp);
            let o = e.last;
            if e.egt > best.0 {
                best = (e.egt, o.phi);
            }
            if o.power > best_pow.0 {
                best_pow = (o.power, o.phi);
            }
            lever -= 0.02;
        }
        assert!((best.1 - 1.0).abs() < 0.05, "peak EGT at phi {}", best.1);
        assert!((best_pow.1 - PHI_BEST).abs() < 0.08, "best power at phi {}", best_pow.1);
        // Peak EGT cruising: about 1,400–1,550 °F.
        let f = (best.0 - 273.15) * 1.8 + 32.0;
        assert!((1350.0..1600.0).contains(&f), "{f}");
        // Idle cut-off stops it.
        assert!(!run(&mut e, 0.7, 0.0, 2400.0, amb).firing);
        // Fully lean it misfires.
        assert!(run(&mut e, 0.7, 0.04, 2400.0, amb).power < 1000.0);
    }

    #[test]
    fn one_magneto_drops_a_little() {
        let mut e = o360();
        let amb = Ambient::isa(0.0, 0.0, 0.0);
        let mut inp = PistonIn { throttle: 0.5, mixture: 1.0, w: 1800.0 * TAU / 60.0, amb, mags: 2, fuel: true, carb_heat: 0.0, cowl: 0.0, q_cool: 2000.0, health: 1.0 };
        let both = e.step(&inp, 0.01).torque;
        inp.mags = 1;
        let one = e.step(&inp, 0.01).torque;
        inp.mags = 0;
        let none = e.step(&inp, 0.01);
        assert!(one < both && one > 0.9 * both);
        assert!(!none.firing && none.torque < 0.0);
    }

    #[test]
    fn heads_run_hot_climbing_cooler_cruising() {
        let mut e = o360();
        let amb = Ambient::isa(0.0, 0.0, 38.0);
        // Climbing at full power, 75 kt, slipstream adding to the cooling air.
        let climb = PistonIn { throttle: 1.0, mixture: 1.0, w: 2600.0 * TAU / 60.0, amb, mags: 2, fuel: true, carb_heat: 0.0, cowl: 1.0, q_cool: 1500.0, health: 1.0 };
        e.warm(&climb);
        let cht_climb = (e.cht - 273.15) * 1.8 + 32.0;
        let oil_climb = (e.oil_t - 273.15) * 1.8 + 32.0;
        // Cruising lean at 65%, 120 kt.
        let amb = Ambient::isa(1800.0, 0.0, 62.0);
        let lever = e.lever_for(1.08, &amb, 0.0);
        let cruise = PistonIn { throttle: 0.62, mixture: lever, w: 2450.0 * TAU / 60.0, amb, cowl: 0.0, q_cool: 2400.0, ..climb };
        e.warm(&cruise);
        let cht_cruise = (e.cht - 273.15) * 1.8 + 32.0;
        assert!((380.0..450.0).contains(&cht_climb), "climb CHT {cht_climb}");
        assert!((300.0..410.0).contains(&cht_cruise), "cruise CHT {cht_cruise}");
        assert!(cht_cruise < cht_climb);
        assert!((150.0..235.0).contains(&oil_climb), "oil {oil_climb}");
    }

    #[test]
    fn turbo_holds_power_to_its_critical_height() {
        // A TSIO-540-like engine: 310 hp at 2,575 rpm, 38 inHg to 17,000 ft.
        let mut d = PistonData::new(231_000.0, 2575.0, 8.86);
        d.boost = Boost::Turbo { max_map: 38.0 * 3386.4, critical: 5200.0 };
        d.carb = false;
        let mut e = Piston::new(d);
        let mut at = |h: f64| {
            let amb = Ambient::isa(h, 0.0, 0.0);
            let lever = e.lever_for(PHI_BEST, &amb, 0.0);
            let inp = PistonIn { throttle: 1.0, mixture: lever, w: 2575.0 * TAU / 60.0, amb, mags: 2, fuel: true, carb_heat: 0.0, cowl: 0.0, q_cool: 2000.0, health: 1.0 };
            for _ in 0..400 {
                e.step(&inp, 0.05);
            }
            e.last
        };
        let sl = at(0.0);
        let mid = at(4000.0);
        let high = at(7500.0);
        assert!((sl.map / 3386.4 - 38.0).abs() < 1.0, "{}", sl.map / 3386.4);
        assert!((mid.map / 3386.4 - 38.0).abs() < 1.0, "{}", mid.map / 3386.4);
        assert!(mid.power > 0.9 * sl.power);
        assert!(high.map < sl.map * 0.9 && high.power < 0.85 * sl.power);
    }
}
