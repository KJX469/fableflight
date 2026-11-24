//! One engine on the airframe: what it is, where it is, its own lever and
//! state, and the force and moment it puts on the aircraft.
use super::jet::{Jet, JetIn};
use super::piston::{Piston, PistonIn, PHI_BEST};
use super::prop::Prop;
use super::{Ambient, Mount};
use glam::DVec3;

#[derive(Clone)]
pub enum Kind {
    /// a piston engine turning a propeller
    Piston { eng: Piston, prop: Prop },
    Jet(Jet),
}

/// What the cockpit gives every engine (each may have its own lever, see `Unit::lever`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Levers {
    /// thrust lever (0..1), mixture (0 cut-off .. 1 full rich), propeller (0 low rpm/feather .. 1 high)
    pub throttle: f64,
    pub mixture: f64,
    pub prop: f64,
    /// the afterburner, and the reversers' position (0..1)
    pub burner: bool,
    pub reverser: f64,
    /// magnetos on (piston) or fuel control at RUN (jet), fuel reaching the engines
    pub ignition: bool,
    pub fuel: bool,
    /// the starter turning (with power to it), carb heat and cowl flaps (0..1)
    pub starter: bool,
    pub carb_heat: f64,
    pub cowl: f64,
    /// the auto-mixture assist holds the best mixture for the power
    pub auto_mixture: bool,
}

/// The air around the aircraft and its motion through it.
#[derive(Clone, Copy, Debug)]
pub struct Flight {
    /// static pressure (Pa) and temperature (K)
    pub p: f64,
    pub t: f64,
    /// the aircraft's velocity through the air, body frame (m/s)
    pub vb: DVec3,
    /// rotation rate, body frame (rad/s): a propeller's blades see it too
    pub omega: DVec3,
}

impl Flight {
    pub fn ambient(&self, mount: &Mount) -> Ambient {
        let v = (self.vb + self.omega.cross(mount.at)).dot(mount.axis);
        let a = Ambient { p: self.p, t: self.t, v, mach: 0.0 };
        Ambient { mach: self.vb.length() / a.sound(), ..a }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UnitOut {
    /// force (body frame, N) and moment about the reference CG (N m)
    pub force: DVec3,
    pub moment: DVec3,
    /// thrust along the axis (N)
    pub thrust: f64,
    /// fuel flow (kg/s)
    pub fuel_flow: f64,
    /// shaft power as a fraction of rated (a jet: thrust as a fraction of its dry rating)
    pub power: f64,
    /// running on its own
    pub running: bool,
}

/// The mixture the auto-mixture assist holds: rich for cooling at high power,
/// rich at idle as a carburettor's idle circuit runs it, best power between.
fn auto_phi(throttle: f64) -> f64 {
    if throttle > 0.85 {
        1.26
    } else {
        PHI_BEST + (1.3 - PHI_BEST) * (1.0 - super::smooth(0.02, 0.12, throttle))
    }
}

#[derive(Clone)]
pub struct Unit {
    pub mount: Mount,
    pub kind: Kind,
    /// its own thrust lever, when it is not ganged to the others
    pub lever: Option<f64>,
    /// its own fuel shut off (fuel control at CUTOFF, mixture pulled), or failed
    pub cut: bool,
    pub failed: bool,
    pub out: UnitOut,
}

impl Unit {
    pub fn new(mount: Mount, kind: Kind) -> Unit {
        Unit { mount, kind, lever: None, cut: false, failed: false, out: UnitOut::default() }
    }

    /// Propeller (or fan) speed, rad/s.
    pub fn rate(&self) -> f64 {
        match &self.kind {
            Kind::Piston { prop, .. } => prop.w,
            Kind::Jet(j) => j.n1 * j.data.n1_rpm * std::f64::consts::TAU / 60.0,
        }
    }
    /// Spin angular momentum (body frame, kg m²/s): the propeller's (a jet's
    /// spools are small beside an airframe's and are left out).
    pub fn spin(&self) -> DVec3 {
        match &self.kind {
            Kind::Piston { prop, .. } => self.mount.axis * (self.mount.spin * prop.data.inertia * prop.w),
            Kind::Jet(_) => DVec3::ZERO,
        }
    }

    /// One step. `health` is what damage has left of it (0..1), `blades` of its propeller.
    pub fn step(&mut self, lv: &Levers, fl: &Flight, health: f64, blades: f64, dt: f64) -> UnitOut {
        let throttle = self.lever.unwrap_or(lv.throttle);
        let health = if self.failed { 0.0 } else { health };
        let amb = fl.ambient(&self.mount);
        let out = match &mut self.kind {
            Kind::Piston { eng, prop } => {
                let g = prop.data.gear;
                let w = prop.w / g;
                let disc = std::f64::consts::FRAC_PI_4 * prop.data.d * prop.data.d;
                let q = 0.5 * amb.rho() * amb.v * amb.v;
                let q_cool = q + self.out.thrust.max(0.0) / disc;
                let mixture = if lv.auto_mixture && lv.mixture > 0.05 {
                    eng.lever_for(auto_phi(throttle), &amb, lv.carb_heat).min(1.0)
                } else {
                    lv.mixture
                };
                let inp = PistonIn {
                    throttle,
                    mixture: if self.cut { 0.0 } else { mixture },
                    w,
                    amb,
                    mags: if lv.ignition { 2 } else { 0 },
                    fuel: lv.fuel,
                    carb_heat: lv.carb_heat,
                    cowl: lv.cowl,
                    q_cool,
                    health,
                };
                let o = eng.step(&inp, dt);
                // The starter cranks it at ~200 rpm; its torque falls away as it spins up.
                let crank = if lv.starter { 180.0 * (1.0 - w / 25.0).max(0.0) } else { 0.0 };
                // Not firing, each cylinder compresses its charge for nothing: a
                // drag that stops a shut-down engine in a couple of seconds.
                let comp = if o.firing { 0.0 } else { 40.0 * (1.0 - w / 90.0).clamp(0.0, 1.0) };
                prop.govern(lv.prop, w > 50.0, dt);
                let po = prop.loads(&self.mount, fl.vb + fl.omega.cross(self.mount.at), &amb, blades);
                let drive = (o.torque + crank - comp) / g - po.torque;
                if blades <= 0.0 {
                    prop.w += (0.0 - prop.w) * (dt * 2.0).min(1.0);
                } else if prop.w > 1.0 || drive > 40.0 {
                    prop.w = (prop.w + drive / prop.data.inertia * dt).max(0.0);
                } else {
                    // Stopped: the compression holds it until the air pushes harder.
                    prop.w = 0.0;
                }
                let force = po.force;
                UnitOut {
                    force,
                    moment: self.mount.at.cross(force) + po.moment,
                    thrust: po.thrust,
                    fuel_flow: o.fuel_flow,
                    power: o.power / eng.data.power,
                    running: o.firing,
                }
            }
            Kind::Jet(j) => {
                let inp = JetIn {
                    throttle,
                    burner: lv.burner,
                    amb,
                    fuel: lv.ignition && lv.fuel && !self.cut,
                    starter: lv.starter,
                    reverser: lv.reverser,
                    health,
                };
                let o = j.step(&inp, dt);
                let force = self.mount.axis * o.thrust;
                UnitOut { force, moment: self.mount.at.cross(force), thrust: o.thrust, fuel_flow: o.fuel_flow, power: o.power, running: j.running }
            }
        };
        self.out = out;
        out
    }

    /// Put it straight into the steady state this lever and this air give
    /// (a spawn, or a reconfigure in flight), running; returns its thrust (N).
    pub fn settle(&mut self, lv: &Levers, fl: &Flight) -> f64 {
        let throttle = self.lever.unwrap_or(lv.throttle);
        let amb = fl.ambient(&self.mount);
        match &mut self.kind {
            Kind::Piston { eng, prop } => {
                let lever = if lv.auto_mixture || lv.mixture <= 0.05 { eng.lever_for(auto_phi(throttle), &amb, 0.0).min(1.0) } else { lv.mixture };
                let disc = std::f64::consts::FRAC_PI_4 * prop.data.d * prop.data.d;
                let q_cool = 0.5 * amb.rho() * amb.v * amb.v + 0.5 * eng.data.power / disc / 30.0;
                let inp = |w: f64| PistonIn { throttle, mixture: lever, w, amb, mags: 2, fuel: true, carb_heat: 0.0, cowl: 0.0, q_cool, health: 1.0 };
                let vb = fl.vb + fl.omega.cross(self.mount.at);
                let g = prop.data.gear;
                // Engine torque less what the blades take, at shaft speed `w` and blade angle `beta`.
                let excess = |eng: &mut Piston, prop: &mut Prop, w: f64, beta: f64| {
                    prop.w = w * g;
                    prop.beta = beta;
                    let q_e = eng.clone().step(&inp(w), 0.0).torque / g;
                    q_e - prop.loads(&self.mount, vb, &amb, 1.0).torque
                };
                let solve_w = |eng: &mut Piston, prop: &mut Prop, beta: f64| {
                    let (mut lo, mut hi) = (10.0, 1.6 * eng.data.w_rated);
                    for _ in 0..40 {
                        let w = 0.5 * (lo + hi);
                        if excess(eng, prop, w, beta) > 0.0 { lo = w } else { hi = w }
                    }
                    0.5 * (lo + hi)
                };
                let beta0 = prop.beta;
                match (prop.data.pitch, prop.governed(lv.prop)) {
                    (super::prop::Pitch::Constant { fine, coarse, .. }, Some(target)) => {
                        let (fine, coarse) = (fine.to_radians(), coarse.to_radians());
                        // The blade angle that holds the governed revs, if the engine can turn them.
                        let w_t = target / g;
                        if excess(eng, prop, w_t, fine) <= 0.0 {
                            let w = solve_w(eng, prop, fine);
                            (prop.w, prop.beta) = (w * g, fine);
                        } else {
                            let (mut lo, mut hi) = (fine, coarse);
                            for _ in 0..40 {
                                let b = 0.5 * (lo + hi);
                                if excess(eng, prop, w_t, b) > 0.0 { lo = b } else { hi = b }
                            }
                            (prop.w, prop.beta) = (target, 0.5 * (lo + hi));
                        }
                    }
                    _ => {
                        let w = solve_w(eng, prop, beta0);
                        (prop.w, prop.beta) = (w * g, beta0);
                    }
                }
                let o = eng.step(&inp(prop.w / g), 0.0);
                let po = prop.loads(&self.mount, vb, &amb, 1.0);
                self.out = UnitOut { force: po.force, moment: self.mount.at.cross(po.force) + po.moment, thrust: po.thrust, fuel_flow: o.fuel_flow, power: o.power / eng.data.power, running: true };
                // Warm, as it would be after running there.
                eng.warm(&inp(prop.w / g));
            }
            Kind::Jet(j) => {
                j.settle(throttle);
                // With the burner asked for (and the lever up), fully lit.
                j.burner = if lv.burner && j.data.wet > 0.0 && throttle > 0.95 { 1.0 } else { 0.0 };
                let inp = JetIn { throttle, burner: false, amb, fuel: true, starter: false, reverser: 0.0, health: 1.0 };
                // Hold the spools while the temperatures and the lit burner settle.
                let (n1, n2) = (j.n1, j.n2);
                let o = j.step(&inp, 0.0);
                (j.n1, j.n2) = (n1, n2);
                let force = self.mount.axis * o.thrust;
                self.out = UnitOut { force, moment: self.mount.at.cross(force), thrust: o.thrust, fuel_flow: o.fuel_flow, power: o.power, running: true };
            }
        }
        self.out.thrust
    }

    /// Stopped dead, cold (cold and dark).
    pub fn stop(&mut self, t: f64) {
        match &mut self.kind {
            Kind::Piston { eng, prop } => {
                prop.w = 0.0;
                eng.cold(t);
            }
            Kind::Jet(j) => {
                (j.n1, j.n2, j.running, j.burner, j.egt) = (0.0, 0.0, false, 0.0, t);
            }
        }
        self.out = UnitOut::default();
    }

    /// Idling, as after a start on the ground.
    pub fn idle(&mut self) {
        match &mut self.kind {
            Kind::Piston { prop, .. } => prop.w = 65.0,
            Kind::Jet(j) => j.settle(0.0),
        }
    }

}
