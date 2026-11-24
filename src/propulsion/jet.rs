//! Gas turbines: turbofans and turbojets, with or without afterburning.
//!
//! - **Spools.** The core (N2) is what the fuel control accelerates, along a
//!   schedule that keeps the compressor clear of surge: slowly from idle,
//!   where there is little air to burn fuel in, and briskly near the top.
//!   The fan (N1) is driven by the core's turbine and follows it a moment
//!   later. So a big fan takes several seconds from idle to take-off thrust
//!   (certification asks for no more than five to 95%), most of it in the
//!   first half of the lever's travel; from a high idle it answers at once.
//! - **Thrust lapse.** The thrust available is the static sea-level rating
//!   times a lapse in the total pressure and temperature of the air at the
//!   inlet (δ0, θ0) and the Mach number: Mattingly's installed-engine lapses
//!   (*Aircraft Engine Design*, 2nd ed., §2.3.2) for high-bypass turbofans,
//!   low-bypass mixed-flow turbofans and turbojets, each dry and with the
//!   afterburner, blended by the bypass ratio. Below the throttle ratio
//!   (`theta_break`, the θ0 where the engine reaches its turbine temperature
//!   limit) it is limited by the compressor's pressure, above it by its
//!   temperature: so an airliner's take-off thrust is flat-rated up to a hot
//!   day and then falls, and an engine designed for Mach 2 keeps its thrust
//!   to Mach 2.
//! - **Fuel.** Thrust specific fuel consumption rises with flight Mach and
//!   falls with the air's temperature (the same source's √θ fits), so cruise
//!   burns about what the real engines burn per pound of thrust, not their
//!   static figure; part power costs a little more per pound.
//! - **Afterburner.** Lit only near full dry power: a fraction of a second
//!   for the igniters, then the zones light in turn over a second or two.
//!
//! The data a type needs is its ratings and a few numbers about its build.
use super::{lag, smooth, Ambient, P0, RHO0};

/// Pounds per pound-force per hour to kilograms per newton-second.
pub const LB_LBF_H: f64 = 0.453_592 / (4.448_222 * 3600.0);

#[derive(Clone, Debug)]
pub struct JetData {
    /// static sea-level thrust of each engine (N): maximum dry, and with the afterburner (0: none)
    pub thrust: f64,
    pub wet: f64,
    /// bypass ratio (0 a turbojet)
    pub bpr: f64,
    /// the θ0 where it reaches its temperature limit (1.0 to 1.1 for an
    /// airliner's flat rating, more for an engine designed to fly fast)
    pub theta_break: f64,
    /// static sea-level specific fuel consumption, dry at full power and
    /// with the afterburner (kg/N s)
    pub tsfc: f64,
    pub tsfc_wet: f64,
    /// fan and core speeds at ground idle (fractions of 100%), and the thrust there (fraction of rated)
    pub idle: [f64; 2],
    pub idle_thrust: f64,
    /// seconds from idle to 95% thrust, and from full back to idle thrust
    pub spool_up: f64,
    pub spool_down: f64,
    /// fan revs at 100% N1
    pub n1_rpm: f64,
    /// reverse thrust as a fraction of forward
    pub reverse: f64,
    /// the inlet's capture area (m²): ram drag when windmilling
    pub inlet: f64,
    /// a fixed-geometry or turbo-ramjet inlet's design Mach number (0: no
    /// limit): past it the inlet chokes and under half the thrust is left a
    /// tenth of a Mach beyond
    pub inlet_mach: f64,
    /// the Mach number by which the dry thrust has all but gone (0: it lapses
    /// as the family does): a turbojet built to cruise in afterburner, its
    /// core doing less and less as the bypass bleed takes over
    pub dry_mach: f64,
}

impl JetData {
    /// The fan's capture area from its rating: about the mass flow a
    /// turbofan of this bypass ratio swallows at full thrust, at 0.5 Mach at the face.
    pub fn inlet_for(thrust: f64, bpr: f64) -> f64 {
        let v_jet = 650.0 / (1.0 + 0.45 * bpr);
        thrust / v_jet / (RHO0 * 170.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JetIn {
    /// thrust lever (0 idle .. 1 full dry), and the afterburner asked for
    pub throttle: f64,
    pub burner: bool,
    pub amb: Ambient,
    /// fuel control at RUN with fuel to it, and the starter turning it
    pub fuel: bool,
    pub starter: bool,
    /// reversers deployed (0..1)
    pub reverser: f64,
    /// what is left of it (damage, 0..1)
    pub health: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JetOut {
    /// net thrust along its axis (N; negative reversed or windmilling)
    pub thrust: f64,
    /// fuel flow (kg/s)
    pub fuel_flow: f64,
    /// thrust as a fraction of its dry rating at sea level
    pub power: f64,
}

#[derive(Clone, Debug)]
pub struct Jet {
    pub data: JetData,
    /// fan and core speeds (fractions of 100%)
    pub n1: f64,
    pub n2: f64,
    /// burning, and how far the afterburner is lit (0..1), how long it has been asked for
    pub running: bool,
    pub burner: f64,
    ab_wait: f64,
    /// exhaust gas temperature (K)
    pub egt: f64,
    /// the core's acceleration constant (found from `spool_up`)
    k_up: f64,
    pub last: JetOut,
}

/// The installed thrust lapse α (thrust over static sea-level maximum dry
/// thrust) for bypass ratio `bpr`, dry or with the afterburner, and the
/// ratio of afterburning to dry thrust the engine's family has statically.
fn lapse_family(bpr: f64, wet: bool, th0: f64, d0: f64, m: f64, tr: f64) -> f64 {
    let over = (th0 - tr).max(0.0);
    let sm = m.max(0.0).sqrt();
    // high bypass (no afterburner: wet is dry). Mattingly's 0.49 is for a
    // bypass ratio near 8; the ram drag behind the √M term is a smaller part
    // of a hotter, faster jet's thrust, so it shrinks with the bypass ratio,
    // down to none in his mixed-flow family. As (BPR/8)^¾ (an estimate, set
    // between a JT9D's take-off lapse and a light G650 still climbing at
    // FL510): 0.34 at 5, 0.31 at 4.35.
    let k_ram = 0.49 * (bpr / 8.0).clamp(0.0, 1.0).powf(0.75);
    let hi = d0 * (1.0 - k_ram * sm - 3.0 * over / (1.5 + m));
    // low bypass, mixed flow: dry normalised by its static 0.6 of the wet
    let lo = if wet { d0 * (1.0 - 3.5 * over / th0) } else { d0 * (1.0 - 3.8 * over / th0) };
    // turbojet: dry normalised by its static 0.8 (and its static 1 - 0.1·0)
    let tj = if wet {
        d0 * (1.0 - 0.3 * (th0 - 1.0) - 0.1 * sm - 1.5 * over / th0)
    } else {
        d0 * (1.0 - 0.16 * sm - 24.0 * over / ((9.0 + m) * th0))
    };
    let a = if bpr >= 3.0 {
        hi
    } else if bpr >= 1.0 {
        let f = (bpr - 1.0) / 2.0;
        lo * (1.0 - f) + hi * f
    } else if bpr >= 0.25 {
        lo
    } else {
        let f = bpr / 0.25;
        tj * (1.0 - f) + lo * f
    };
    a.max(0.0)
}

/// TSFC's rise with Mach and fall with temperature, relative to static sea level.
fn tsfc_family(bpr: f64, wet: bool, m: f64, theta: f64) -> f64 {
    let (c0, c1) = if bpr >= 3.0 {
        (0.4, 0.45)
    } else if bpr >= 0.25 {
        if wet { (1.6, 0.27) } else { (0.9, 0.30) }
    } else if wet {
        (1.5, 0.23)
    } else {
        (1.1, 0.30)
    };
    (c0 + c1 * m.max(0.0)) / c0 * theta.sqrt()
}

impl Jet {
    pub fn new(data: JetData) -> Jet {
        let mut j = Jet { n1: 0.0, n2: 0.0, running: false, burner: 0.0, ab_wait: 0.0, egt: 288.15, k_up: 1.0, last: JetOut::default(), data };
        // Find the acceleration constant that takes it from idle to 95% thrust in `spool_up`.
        let (mut lo, mut hi): (f64, f64) = (0.05, 20.0);
        for _ in 0..40 {
            let k: f64 = (lo * hi).sqrt();
            j.k_up = k;
            if j.accel_time() > j.data.spool_up { lo = k } else { hi = k }
        }
        j
    }

    fn accel_time(&self) -> f64 {
        let mut s = self.clone();
        (s.n2, s.n1, s.running) = (s.data.idle[1], s.data.idle[0], true);
        let dt = 0.02;
        let mut t = 0.0;
        while t < 60.0 {
            s.spool(1.0, true, false, 1.0, dt);
            if s.thrust_frac() >= 0.95 {
                return t;
            }
            t += dt;
        }
        t
    }

    /// N1 the core at `n2` drives the fan to, steadily.
    fn n1_of(&self, n2: f64) -> f64 {
        let [i1, i2] = self.data.idle;
        if n2 >= i2 {
            i1 + (1.0 - i1) * ((n2 - i2) / (1.0 - i2)).powf(1.4)
        } else {
            i1 * (n2 / i2).powi(2)
        }
    }
    /// The core speed whose fan gives `n1`.
    fn n2_for(&self, n1: f64) -> f64 {
        let [i1, i2] = self.data.idle;
        i2 + (1.0 - i2) * ((n1 - i1) / (1.0 - i1)).max(0.0).powf(1.0 / 1.4)
    }
    /// N1 the lever asks for.
    pub fn n1_target(&self, throttle: f64) -> f64 {
        let i1 = self.data.idle[0];
        // Thrust goes roughly as the lever's travel, so the fan speed as its root.
        let tf = self.data.idle_thrust + (1.0 - self.data.idle_thrust) * throttle.clamp(0.0, 1.0);
        i1 + (1.0 - i1) * ((tf - self.data.idle_thrust) / (1.0 - self.data.idle_thrust)).powf(1.0 / 1.7)
    }
    /// Dry thrust as a fraction of what is available at full, at this fan speed.
    pub fn thrust_frac(&self) -> f64 {
        let i1 = self.data.idle[0];
        if self.n1 >= i1 {
            let x = (self.n1 - i1) / (1.0 - i1);
            self.data.idle_thrust + (1.0 - self.data.idle_thrust) * x.powf(1.7)
        } else {
            self.data.idle_thrust * (self.n1 / i1).powi(2)
        }
    }

    fn spool(&mut self, throttle: f64, fuel: bool, starter: bool, health: f64, dt: f64) {
        let d = &self.data;
        let i2 = d.idle[1];
        if !fuel || health <= 0.0 {
            self.running = false;
        } else if !self.running && self.n2 > 0.55 * i2 {
            // Light-off once the starter has the core turning fast enough.
            self.running = true;
        }
        let n2_want = if self.running {
            self.n2_for(self.n1_target(throttle)).min(i2 + (1.0 - i2) * health)
        } else if starter {
            0.62 * i2
        } else {
            0.0
        };
        if n2_want > self.n2 {
            // The fuel control's schedule: slow from idle, quicker with air to burn the fuel in.
            let x = ((self.n2 - i2) / (1.0 - i2)).clamp(0.0, 1.0);
            let k = if self.running && self.n2 >= 0.95 * i2 { self.k_up * (0.25 + 0.75 * x) } else { 0.35 };
            let step = ((n2_want - self.n2) * k * dt).max(dt * 0.004);
            self.n2 = (self.n2 + step).min(n2_want);
        } else {
            let k = 3.0 / d.spool_down.max(0.1);
            self.n2 += (n2_want - self.n2) * (k * dt).min(1.0);
        }
        // The fan follows the core a moment later.
        self.n1 = lag(self.n1, self.n1_of(self.n2), 0.35, dt);
    }

    /// Thrust (N) available in this air, dry and with the burner fully lit.
    pub fn available(&self, amb: &Ambient) -> (f64, f64) {
        let d = &self.data;
        // Past Mach 1 the intake's shocks cost total pressure: the standard
        // recovery of MIL-E-5007D, 1 - 0.075 (M - 1)^1.35 (0.93 at Mach 2).
        let (th0, m) = (amb.theta0(), amb.mach);
        let recovery = if m > 1.0 { 1.0 - 0.075 * (m - 1.0).powf(1.35) } else { 1.0 };
        let d0 = amb.delta0() * recovery;
        let inlet = if d.inlet_mach > 0.0 { 1.0 - 0.65 * smooth(d.inlet_mach - 0.05, d.inlet_mach + 0.15, m) } else { 1.0 };
        let fade = if d.dry_mach > 0.0 { 1.0 - 0.97 * smooth(0.4 * d.dry_mach, d.dry_mach, m) } else { 1.0 };
        let dry = d.thrust * lapse_family(d.bpr, false, th0, d0, m, d.theta_break) * inlet;
        let wet = if d.wet > 0.0 { d.wet * lapse_family(d.bpr, true, th0, d0, m, d.theta_break) * inlet } else { dry };
        (dry * fade, wet.max(dry * fade))
    }

    pub fn step(&mut self, inp: &JetIn, dt: f64) -> JetOut {
        let amb = inp.amb;
        let ab_ok = self.data.wet > 0.0 && inp.burner && self.running && self.n2 > 0.9 && inp.throttle > 0.95;
        // With the burner wanted, the lever is at full dry.
        let throttle = if inp.burner && self.data.wet > 0.0 { 1.0 } else { inp.throttle };
        self.spool(throttle, inp.fuel, inp.starter, inp.health, dt);
        if ab_ok {
            self.ab_wait += dt;
        } else {
            self.ab_wait = 0.0;
        }
        let lit = ab_ok && self.ab_wait > 0.35;
        self.burner = if lit { (self.burner + dt / 1.5).min(1.0) } else { (self.burner - dt / 0.5).max(0.0) };
        let (dry_av, wet_av) = self.available(&amb);
        let theta = amb.t / super::T0;
        let frac = if self.running { self.thrust_frac() } else { 0.0 };
        let dry = dry_av * frac * inp.health.max(0.0).sqrt();
        let thrust = dry + self.burner * (wet_av - dry_av);
        // Windmilling (or unlit), it drags the air it swallows.
        let q = 0.5 * amb.rho() * amb.v * amb.v;
        let ram = q * self.data.inlet * 0.3 * (1.0 - smooth(0.0, 0.3, frac));
        let mut net = thrust - ram;
        net *= 1.0 - inp.reverser * (1.0 + self.data.reverse);
        let ts_dry = self.data.tsfc * tsfc_family(self.data.bpr, false, amb.mach, theta);
        let ts_wet = self.data.tsfc_wet.max(self.data.tsfc) * tsfc_family(self.data.bpr, true, amb.mach, theta);
        // Part power costs a little more per newton: a floor of what idling burns.
        let ff_dry = if self.running { ts_dry * (0.94 * dry + 0.06 * dry_av) } else { 0.0 };
        let fuel_flow = if self.running { (1.0 - self.burner) * ff_dry + self.burner * ts_wet * (dry_av + (wet_av - dry_av)) } else { 0.0 };
        // Exhaust temperature: up with the fan, and hotter in thin hot air.
        let egt_target = if self.running { amb.t + (380.0 + 420.0 * frac) * (0.85 + 0.15 * theta) * (P0 / amb.p).powf(0.03) } else { amb.t + 40.0 * self.n2 };
        self.egt = lag(self.egt, egt_target, 2.0, dt);
        self.last = JetOut { thrust: net, fuel_flow, power: thrust / self.data.thrust };
        self.last
    }

    /// Settle it running as the lever has it (a spawn with engines running).
    pub fn settle(&mut self, throttle: f64) {
        self.running = true;
        self.n1 = self.n1_target(throttle);
        self.n2 = self.n2_for(self.n1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A JT9D-7-class high-bypass turbofan (747-200): 46,500 lbf, BPR 5, SFC 0.36.
    fn fan() -> Jet {
        Jet::new(JetData {
            thrust: 206_800.0,
            wet: 0.0,
            bpr: 5.0,
            theta_break: 1.05,
            tsfc: 0.36 * LB_LBF_H,
            tsfc_wet: 0.0,
            idle: [0.22, 0.6],
            idle_thrust: 0.04,
            spool_up: 6.0,
            spool_down: 4.0,
            n1_rpm: 3650.0,
            reverse: 0.4,
            inlet: JetData::inlet_for(206_800.0, 5.0),
            inlet_mach: 0.0,
            dry_mach: 0.0,
        })
    }
    /// An F404-class low-bypass afterburning turbofan: 11,000 / 17,700 lbf, BPR 0.34.
    fn fighter() -> Jet {
        Jet::new(JetData {
            thrust: 48_900.0,
            wet: 78_700.0,
            bpr: 0.34,
            theta_break: 1.08,
            tsfc: 0.81 * LB_LBF_H,
            tsfc_wet: 1.74 * LB_LBF_H,
            idle: [0.6, 0.7],
            idle_thrust: 0.06,
            spool_up: 4.0,
            spool_down: 2.5,
            n1_rpm: 13_200.0,
            reverse: 0.0,
            inlet: JetData::inlet_for(48_900.0, 0.34),
            inlet_mach: 0.0,
            dry_mach: 0.0,
        })
    }
    fn run(j: &mut Jet, throttle: f64, burner: bool, amb: Ambient, secs: f64) -> JetOut {
        let inp = JetIn { throttle, burner, amb, fuel: true, starter: false, reverser: 0.0, health: 1.0 };
        let mut o = JetOut::default();
        for _ in 0..(secs / 0.02) as usize {
            o = j.step(&inp, 0.02);
        }
        o
    }

    #[test]
    fn spools_up_in_its_time() {
        let mut j = fan();
        j.settle(0.0);
        let amb = Ambient::isa(0.0, 0.0, 0.0);
        let idle = run(&mut j, 0.0, false, amb, 5.0);
        assert!((idle.thrust / 206_800.0 - 0.04).abs() < 0.02, "{}", idle.thrust / 206_800.0);
        // Half a second after slamming it open, still little thrust...
        let o = run(&mut j, 1.0, false, amb, 0.5);
        assert!(o.thrust < 0.25 * 206_800.0, "{}", o.thrust);
        // ...95% by about its spool-up time.
        let o = run(&mut j, 1.0, false, amb, 6.0);
        assert!(o.thrust > 0.94 * 206_800.0, "{}", o.thrust / 206_800.0);
        assert!((j.n1 - 1.0).abs() < 0.02);
    }

    #[test]
    fn high_bypass_lapses_like_an_airliners() {
        let mut j = fan();
        j.settle(1.0);
        // Take-off roll at 150 kt: a bypass-5 fan loses some 12–25% by then
        // (more the higher its bypass ratio).
        let roll = run(&mut j, 1.0, false, Ambient::isa(0.0, 0.0, 77.0), 2.0).thrust / 206_800.0;
        assert!((0.75..0.88).contains(&roll), "{roll}");
        // Max climb/cruise at 35,000 ft, M 0.85: about 22–27% of static (JT9D: ~11,000 lbf available).
        let a = Ambient::isa(10_668.0, 0.0, 0.0);
        let a = Ambient { v: 0.85 * a.sound(), mach: 0.85, ..a };
        let cr = run(&mut j, 1.0, false, a, 2.0);
        let r = cr.thrust / 206_800.0;
        assert!((0.19..0.30).contains(&r), "{r}");
        // Cruise TSFC about 0.62–0.66 lb/lbf/h (JT9D-7 at M 0.85).
        let tsfc = cr.fuel_flow / cr.thrust / LB_LBF_H;
        assert!((0.58..0.7).contains(&tsfc), "{tsfc}");
    }

    #[test]
    fn flat_rated_then_falls_on_a_hot_day() {
        let mut j = fan();
        j.settle(1.0);
        let isa = run(&mut j, 1.0, false, Ambient::isa(0.0, 0.0, 0.0), 1.0).thrust;
        let warm = run(&mut j, 1.0, false, Ambient::isa(0.0, 12.0, 0.0), 1.0).thrust;
        let hot = run(&mut j, 1.0, false, Ambient::isa(0.0, 30.0, 0.0), 1.0).thrust;
        assert!((warm / isa - 1.0).abs() < 0.01);
        assert!(hot < 0.9 * isa, "{}", hot / isa);
    }

    #[test]
    fn afterburner_lights_and_gains_with_speed() {
        let mut j = fighter();
        j.settle(1.0);
        let sl = Ambient::isa(0.0, 0.0, 0.0);
        let mil = run(&mut j, 1.0, false, sl, 1.0);
        // Asked for, it lights after a moment.
        let o = run(&mut j, 1.0, true, sl, 0.2);
        assert!(o.thrust < mil.thrust * 1.05);
        let max = run(&mut j, 1.0, true, sl, 3.0);
        assert!((max.thrust / 78_700.0 - 1.0).abs() < 0.03, "{}", max.thrust);
        assert!(max.fuel_flow > 2.5 * mil.fuel_flow);
        // At 30,000 ft and Mach 1.4 the ram recovery holds afterburning thrust up.
        let a = Ambient::isa(9144.0, 0.0, 0.0);
        let a = Ambient { v: 1.4 * a.sound(), mach: 1.4, ..a };
        let hi = run(&mut j, 1.0, true, a, 1.0).thrust / 78_700.0;
        assert!((0.6..0.95).contains(&hi), "{hi}");
        // Below full dry power it will not light.
        let mut k = fighter();
        k.settle(0.5);
        run(&mut k, 0.5, true, sl, 0.5);
        assert!(k.burner == 0.0 || k.n2 > 0.9);
    }

    #[test]
    fn starts_and_windmills() {
        let mut j = fan();
        let sl = Ambient::isa(0.0, 0.0, 0.0);
        let inp = JetIn { throttle: 0.0, burner: false, amb: sl, fuel: true, starter: true, reverser: 0.0, health: 1.0 };
        for _ in 0..(40.0 / 0.02) as usize {
            j.step(&inp, 0.02);
        }
        assert!(j.running && (j.n1 - 0.22).abs() < 0.03, "{} {}", j.running, j.n1);
        // Fuel off in flight: it runs down and drags.
        let a = Ambient::isa(3000.0, 0.0, 120.0);
        let inp = JetIn { fuel: false, starter: false, amb: a, ..inp };
        let mut o = JetOut::default();
        for _ in 0..(20.0 / 0.02) as usize {
            o = j.step(&inp, 0.02);
        }
        assert!(!j.running && o.thrust < 0.0 && o.fuel_flow == 0.0);
    }

    #[test]
    fn reversers_reverse() {
        let mut j = fan();
        j.settle(0.8);
        let inp = JetIn { throttle: 0.8, burner: false, amb: Ambient::isa(0.0, 0.0, 50.0), fuel: true, starter: false, reverser: 1.0, health: 1.0 };
        let o = j.step(&inp, 0.02);
        assert!(o.thrust < 0.0);
    }
}
