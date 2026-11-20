//! Landing gear on the ground: the struts, the tyres, the brakes and the
//! steering, the ground they roll on and what the weather has done to it.
//!
//! Everything here is generic: a leg is only data (`aircraft::Leg` and its
//! `Tyre`), so every aircraft, built in or downloaded, rolls on the same model.
//!
//! - **Struts.** A sprung steel leg is a plain spring and damper. An oleo is a
//!   gas spring (polytropic: soft near full extension, stiffening hard as it
//!   closes, so a firm arrival bottoms it) with oil forced through an orifice
//!   (damping that grows with the speed of the stroke and is several times
//!   stiffer on the rebound, so the aircraft does not bounce back off it). The
//!   oleo is matched to the leg's spring rate `k` where it stands at rest, so
//!   the aircraft sits as high as it always has.
//! - **Tyres.** Side force comes from the slip angle through the cornering
//!   stiffness, with the tyre's relaxation length (it takes about a tyre's
//!   radius of rolling for the force to build) and a peak at a few degrees of
//!   slip, past which it slides and grips less. Braking and side forces share
//!   one friction circle, so a locked wheel cannot also corner. Standing still,
//!   the footprint holds like a stiff spring (no creeping on a slope, no jitter).
//! - **Wheels and brakes.** Each leg's wheels spin: they spin up with a jolt of
//!   drag at touchdown, brakes can lock them (a skid grips less, and a locked
//!   wheel only slides), and anti-skid lets the brakes off just short of a lock.
//! - **Steering.** Steered wheels follow the steering; castering wheels swivel
//!   to trail their own path, and a steerable tailwheel breaks out to swivel
//!   freely when it is pushed past what its steering springs hold.
//! - **The ground.** Pavement grips better than grass; grass and rough ground
//!   drag more (more still on a narrow high-pressure tyre that sinks in) and
//!   bump the struts; rain leaves water standing on pavement, which grips less
//!   the faster it is crossed, and past about 9√psi kt the tyre aquaplanes;
//!   lying snow grips like compacted snow and drags in soft ground.
use crate::aircraft::{Leg, Spec, MAX_GEAR};
use glam::{DQuat, DVec3};

/// What a tyre rolls on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Surface {
    Concrete,
    Asphalt,
    /// a mown grass strip (or an unpaved runway)
    Turf,
    /// open ground off the airfield
    Rough,
}

struct Props {
    /// peak friction against dry concrete's
    grip: f64,
    /// what is left of it wet, at low speed
    wet: f64,
    /// extra rolling resistance (of the load), for a tyre at about 3 bar
    roll: f64,
    /// how uneven it is (m)
    bump: f64,
    /// sheds water into grooves and drains (pavement), or soaks it up
    paved: bool,
}

impl Surface {
    // Friction: ESDU 71026 and NASA TN D-2770 ranges for aircraft tyres (dry
    // concrete 0.7-0.9, wet 0.4-0.6 falling with speed, dry grass about 0.5,
    // wet grass 0.2-0.35); rolling resistance 0.01-0.02 on pavement, 0.04-0.10
    // on grass (light-aircraft handbooks' grass corrections); estimates.
    fn props(self) -> Props {
        match self {
            Surface::Concrete => Props { grip: 1.0, wet: 0.62, roll: 0.0, bump: 0.003, paved: true },
            Surface::Asphalt => Props { grip: 0.96, wet: 0.58, roll: 0.002, bump: 0.004, paved: true },
            Surface::Turf => Props { grip: 0.62, wet: 0.55, roll: 0.045, bump: 0.018, paved: false },
            Surface::Rough => Props { grip: 0.58, wet: 0.55, roll: 0.075, bump: 0.045, paved: false },
        }
    }
    pub fn paved(self) -> bool {
        self.props().paved
    }
    /// How far the ground at `p` (planet frame) is above or below its mean
    /// (m): the joints and patches of pavement, the tussocks of a field.
    pub fn bump(self, p: DVec3) -> f64 {
        let a = self.props().bump;
        a * (0.7 * noise(p / 2.7) + 0.3 * noise(p / 0.9 + DVec3::splat(17.3)))
    }
}

/// Smooth value noise in -1..1 over space.
fn noise(p: DVec3) -> f64 {
    let f = p.floor();
    let t = p - f;
    let s = t * t * (DVec3::splat(3.0) - 2.0 * t);
    let h = |x: f64, y: f64, z: f64| {
        let mut k = (x as i64).wrapping_mul(0x9E37_79B9_7F4A_7C15u64 as i64) ^ (y as i64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F_u64 as i64) ^ (z as i64).wrapping_mul(0x1656_67B1_9E37_79F9);
        k ^= k >> 29;
        k = k.wrapping_mul(0xBF58_476D_1CE4_E5B9u64 as i64);
        k ^= k >> 32;
        (k & 0xFFFF) as f64 / 32767.5 - 1.0
    };
    let l = |a: f64, b: f64, t: f64| a + (b - a) * t;
    let (x, y, z) = (f.x, f.y, f.z);
    l(
        l(l(h(x, y, z), h(x + 1.0, y, z), s.x), l(h(x, y + 1.0, z), h(x + 1.0, y + 1.0, z), s.x), s.y),
        l(l(h(x, y, z + 1.0), h(x + 1.0, y, z + 1.0), s.x), l(h(x, y + 1.0, z + 1.0), h(x + 1.0, y + 1.0, z + 1.0), s.x), s.y),
        s.z,
    )
}

/// The weather on the ground under the aircraft (set from the weather each frame).
#[derive(Clone, Copy, Debug, Default)]
pub struct Cond {
    /// rain (not snow) falling there now, 0..1
    pub rain: f64,
    /// snow lying on the ground, 0..1
    pub snow: f64,
}

/// The water that rain of `rain` (0..1) leaves standing on a runway once it
/// has settled (mm): a grooved runway drains a shower; heavy rain floods it
/// past the 3 mm at which it counts as contaminated.
pub fn water_eq(rain: f64) -> f64 {
    4.0 * rain.clamp(0.0, 1.0).powf(1.3)
}

/// Standing water (mm) after `dt`: rain puts it down over a couple of
/// minutes; it drains and dries over about a quarter of an hour.
pub fn water_step(depth: f64, rain: f64, dt: f64) -> f64 {
    let goal = water_eq(rain);
    let tau = if goal > depth { 120.0 } else { 900.0 };
    goal + (depth - goal) * (-dt / tau).exp()
}

/// How a tyre grips just now.
#[derive(Clone, Copy, Debug)]
pub struct Grip {
    /// peak friction, and what a sliding tyre keeps
    pub mu: f64,
    pub slide: f64,
    /// rolling resistance (of the load)
    pub roll: f64,
    /// aquaplaning, 0..1
    #[cfg_attr(not(test), allow(dead_code))]
    pub aqua: f64,
}

/// The speed a tyre at `kpa` starts to aquaplane at (m/s): Horne's 9 √psi kt
/// while it is turning, 7.7 √psi kt once it has spun down (NASA TN D-2056).
pub fn aquaplane_speed(kpa: f64, spun_down: bool) -> f64 {
    let psi = kpa / 6.894_76;
    (if spun_down { 7.7 } else { 9.0 }) * psi.max(1.0).sqrt() * 0.514_444
}

/// How `leg`'s tyres grip on `s` with the weather `cond`, `water` mm of
/// it standing on the ground, at `v` m/s; `locked` while a wheel is skidding.
pub fn grip(leg: &Leg, s: Surface, cond: &Cond, water: f64, v: f64, locked: bool) -> Grip {
    let p = s.props();
    let t = &leg.tyre;
    let mut mu = leg.mu * p.grip;
    // Soft ground: a narrow high-pressure tyre sinks in and ploughs.
    let sink = if p.paved { 1.0 } else { (t.pressure / 300.0).sqrt().clamp(0.7, 3.0) };
    let mut roll = leg.roll_mu + p.roll * sink;
    // Wet: a film of water under the footprint, worse the faster it is
    // crossed (less time to squeeze it out). Grass soaks rain up but is slick.
    let wet = if p.paved { 1.0 - (-water / 0.25).exp() } else { cond.rain.max(1.0 - (-water / 0.5).exp()).min(1.0) };
    let vp = aquaplane_speed(t.pressure, locked);
    let fast = (v / vp).clamp(0.0, 1.0);
    mu *= 1.0 - wet * (1.0 - p.wet * (1.0 - 0.35 * fast));
    // Standing water deeper than the tread: past the aquaplaning speed the
    // tyre rides up on a wedge of water and keeps almost no grip.
    let aqua = if p.paved { smooth(0.92, 1.05, v / vp) * smooth(1.0, 3.0, water) } else { 0.0 };
    mu += (0.05 - mu) * aqua;
    // Water ahead of the tyre is pushed aside: drag (contaminant drag, small).
    roll += 0.002 * (water - 3.0).max(0.0) * fast * fast;
    // Snow: compacted on pavement (about 0.3), soft and deep in a field.
    if cond.snow > 0.0 {
        let k = cond.snow.clamp(0.0, 1.0);
        mu += (0.32 * leg.mu / 0.8 - mu).min(0.0) * k;
        roll += k * if p.paved { 0.02 } else { 0.10 * sink };
    }
    let slide = mu * (0.78 - 0.2 * wet);
    Grip { mu, slide, roll, aqua }
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// What each leg carries standing still at the aircraft's `mass` (N): the
/// legs as springs under a rigid airframe balanced about its origin.
pub fn static_loads(spec: &Spec, mass: f64) -> [f64; MAX_GEAR] {
    let mut out = [0.0; MAX_GEAR];
    let legs = &spec.gear[..spec.gear.len().min(MAX_GEAR)];
    if legs.is_empty() {
        return out;
    }
    let w = mass * 9.81;
    // Displacement d = h + a x + b z under each leg; force k d. Equilibrium:
    // the forces carry the weight with no moment about the origin.
    let mut m = [[0.0f64; 3]; 3];
    for l in legs {
        let r = [1.0, l.at.x, l.at.z];
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] += l.k * r[i] * r[j];
            }
        }
    }
    let rhs = [w, 0.0, 0.0];
    let det = |m: &[[f64; 3]; 3]| m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let d = det(&m);
    let scale = m[0][0] * m[1][1].max(1.0) * m[2][2].max(1.0);
    if d.abs() > 1e-9 * scale {
        let mut sol = [0.0; 3];
        for (c, s) in sol.iter_mut().enumerate() {
            let mut mc = m;
            for r in 0..3 {
                mc[r][c] = rhs[r];
            }
            *s = det(&mc) / d;
        }
        for (i, l) in legs.iter().enumerate() {
            out[i] = (l.k * (sol[0] + sol[1] * l.at.x + sol[2] * l.at.z)).max(0.02 * w);
        }
    } else {
        for o in out.iter_mut().take(legs.len()) {
            *o = w / legs.len() as f64;
        }
    }
    // (Clamping the odd leg that would pull: renormalise to the weight.)
    let sum: f64 = out.iter().sum();
    out.iter_mut().for_each(|o| *o *= w / sum);
    out
}

/// One leg's wheels as they roll.
#[derive(Clone, Copy, Debug, Default)]
pub struct Wheel {
    /// how far the footprint is wound up against the ground, along and across the wheel (m)
    pub patch: [f64; 2],
    /// a castering wheel's swivel (rad, + right of where the steering points)
    pub swivel: f64,
    /// a steerable tailwheel broken out to swivel freely
    pub free: bool,
    /// the wheels are slipping round against the ground: locked by the
    /// brakes, or spinning up at touchdown
    pub skid: bool,
    /// still rolling with the ground, since it last touched
    pub on: bool,
}

/// One leg touching the ground this step.
pub struct Contact<'a> {
    pub leg: &'a Leg,
    /// what it carries standing still (N)
    pub w0: f64,
    /// how far the ground is into the fully extended leg (m)
    pub pen: f64,
    /// the contact point's velocity (planet frame) and the ground's normal there
    pub v: DVec3,
    pub n: DVec3,
    /// where the wheel points (planet frame), steering included
    pub heading: DVec3,
    /// brake pressure 0..1
    pub brake: f64,
    pub grip: Grip,
    pub dt: f64,
}

/// What a leg does to the aircraft.
pub struct Push {
    /// the force at the contact (planet frame), and its share along the ground's normal
    pub f: DVec3,
    pub nf: f64,
    /// how fast the tyre slides over the ground (m/s): rubber smoke and scuffed tracks
    pub slide: f64,
    /// where the wheel points (planet frame), and how fast it is rolling
    pub heading: DVec3,
    pub along: f64,
    pub across: f64,
}

/// Wheel inertia of a leg (kg m²): wheel, tyre and brake rotor together,
/// sized from the load each wheel carries (an estimate: about 6 kg for a
/// light aircraft's wheel, 100 kg for an airliner's).
fn inertia(leg: &Leg, w0: f64) -> f64 {
    let n = leg.tyre.wheels.max(1) as f64;
    let m = 1.5 * (w0 / n / 1000.0).max(0.5).powf(0.8);
    n * 0.6 * m * leg.wheel_r * leg.wheel_r
}

/// The oleo or spring force of a leg closed by `x` m at `rate` m/s (+ closing).
pub fn strut(leg: &Leg, w0: f64, x: f64, rate: f64) -> f64 {
    let k = leg.k;
    let spring = if leg.tyre.oleo {
        // Gas spring matched to `k` at the static stroke: F = P0 (L / (L - x))^n.
        let n = 1.3;
        let w = w0.max(k * leg.travel * 0.05);
        let xs = (w / k).min(0.8 * leg.travel);
        let l = xs + n * w / k;
        let p0 = w * ((l - xs) / l).powf(n);
        let gas = |x: f64| p0 * (l / (l - x)).powf(n);
        let lim = 0.92 * l;
        let f = if x < lim { gas(x) } else { gas(lim) + n * gas(lim) / (l - lim) * (x - lim) };
        // The tyre takes the first of it, up to the strut's preload.
        f.min(4.0 * k * x)
    } else {
        k * x
    };
    // Bottomed out: metal on metal.
    let stop = 30.0 * k * (x - leg.travel).max(0.0);
    let damp = if !leg.tyre.oleo {
        leg.c * rate
    } else if rate > 0.0 {
        // through the orifice: grows with the square of the stroke's speed
        // (the leg's `c` is its rate at half a metre a second)
        leg.c * rate * (0.75 + 0.5 * rate)
    } else {
        // the rebound valve
        2.5 * leg.c * rate
    };
    (spring + stop + damp).clamp(0.0, leg.nf_max)
}

/// The forces of one leg on the ground, stepping its wheels' state.
pub fn contact(c: &Contact, w: &mut Wheel, spin: &mut f64) -> Push {
    let leg = c.leg;
    let t = &leg.tyre;
    let r = leg.wheel_r.max(0.05);
    let dt = c.dt;
    let n = c.n;
    let vn = c.v.dot(n);
    let nf = strut(leg, c.w0, c.pen, -vn);
    let g = &c.grip;
    let along_ground = |d: DVec3| (d - n * d.dot(n)).normalize_or_zero();
    let mut h = along_ground(c.heading);
    let vt = c.v - n * vn;

    // A castering wheel turns to trail its own path (its axle behind the
    // swivel: it lines up within a trail or so of rolling).
    if t.caster > 0.0 {
        let steerable = leg.steer != 0.0;
        if steerable && !w.free {
            w.swivel = 0.0;
        } else {
            let hs = DQuat::from_axis_angle(n, -w.swivel) * h;
            let (vx, vy) = (vt.dot(hs), vt.dot(hs.cross(n)));
            let speed = vx.hypot(vy);
            if speed > 0.05 {
                // (a full-castering wheel rolls backwards by turning right round;
                // a limited one just stays where it is)
                let a = if t.caster >= 3.0 { vy.atan2(vx) } else if vx > 0.0 { vy.atan2(vx) } else { 0.0 };
                w.swivel += a * (1.0 - (-speed * dt / (0.6 * r)).exp());
                w.swivel = if t.caster >= 3.0 { (w.swivel + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI } else { w.swivel.clamp(-t.caster, t.caster) };
            }
            // A steerable tailwheel locks back on as it comes round straight.
            if steerable && w.swivel.abs() < 0.05 {
                (w.free, w.swivel) = (false, 0.0);
            }
        }
        h = DQuat::from_axis_angle(n, -w.swivel) * h;
    }
    let s = h.cross(n);
    let (vx, vy) = (vt.dot(h), vt.dot(s));
    let swivels = t.caster > 0.0 && (leg.steer == 0.0 || w.free);

    // Tyre stiffness: cornering stiffness about 7 per radian of the load
    // (aircraft tyres: NASA TR R-64, 0.1-0.15 per degree), wound up over a
    // relaxation length of about the tyre's radius.
    let load = nf.max(0.05 * c.w0);
    let sigma = r;
    let ky = 7.0 * load / sigma;
    let kx = 1.5 * ky;
    let m_leg = c.w0 / 9.81;
    let cy = 2.0 * 0.7 * (ky * m_leg).sqrt();
    let cx = 2.0 * 0.7 * (kx * m_leg).sqrt();

    // Brakes: `brake_mu` of the leg's static load at full pressure; anti-skid
    // holds them just short of what the tyres can take.
    let mut brake = c.brake * leg.brake_mu * c.w0;
    let roll = g.roll * nf;
    if t.antiskid {
        brake = brake.min((0.9 * g.mu * nf - roll).max(0.0));
    }
    let inertia = inertia(leg, c.w0);

    // Touching down with the wheels stopped (or slower than the ground
    // goes by): they are dragged round, sliding, until they roll.
    if !w.on {
        w.on = true;
        w.skid = (*spin * r - vx).abs() > 2.0;
    }
    if !w.skid && vx.abs() > 1.0 && brake + roll > g.mu * nf {
        // More brake than grip: the wheels lock.
        w.skid = true;
    }
    let (fx, slip_x);
    if w.skid {
        let slip = vx - *spin * r;
        // Friction turns the wheel towards rolling, the brakes hold it.
        let drive = g.slide * nf * r * slip.signum();
        // (anti-skid lets the brakes right off until the wheels turn again)
        let hold = (if t.antiskid { 0.0 } else { brake } + roll) * r / inertia * dt;
        *spin += drive / inertia * dt;
        *spin = if spin.abs() > hold { *spin - hold * spin.signum() } else { 0.0 };
        let slip1 = vx - *spin * r;
        if slip1.signum() != slip.signum() || slip1.abs() < 0.3 || (vx.abs() < 1.0 && brake + roll <= g.mu * nf) {
            // rolling again
            w.skid = false;
            *spin = vx / r;
        }
        slip_x = slip;
        fx = -g.slide * nf * slip.signum() * (slip.abs() / 0.5).min(1.0);
        w.patch[0] = 0.0;
    } else {
        *spin = vx / r;
        slip_x = 0.0;
        // The brakes (and the rolling resistance) hold the footprint like a
        // clutch: stuck while they can, slipping at what they take.
        let cap = (brake + roll).min(g.mu * nf);
        w.patch[0] += vx * dt;
        let mut f = -(kx * w.patch[0] + cx * vx);
        if f.abs() > cap {
            f = cap * f.signum();
            w.patch[0] = w.patch[0].clamp(-cap / kx, cap / kx);
        }
        fx = f;
    }

    // Side force: the footprint winds up with the slip, relaxing over σ.
    let ax = vx.abs();
    let d0 = w.patch[1];
    w.patch[1] = if ax > 0.05 {
        let ss = sigma * vy / ax;
        ss + (d0 - ss) * (-ax * dt / sigma).exp()
    } else {
        d0 + vy * dt
    };
    let rate = vy - ax * w.patch[1] / sigma;
    let fy_lin = -(ky * w.patch[1] + cy * rate);
    let mut slide = slip_x.abs();
    let fy = if w.skid {
        // A sliding tyre: friction against where it slides.
        let sv = slip_x.hypot(vy).max(0.3);
        w.patch[1] = 0.0;
        slide = sv;
        -g.slide * nf * vy / sv
    } else {
        let mut cap = ((g.mu * nf).powi(2) - fx * fx).max((0.15 * g.mu * nf).powi(2)).sqrt();
        if swivels {
            // A free caster turns rather than push sideways: only the swivel's
            // friction (its shimmy damper) is felt.
            cap = cap.min(0.08 * nf);
        }
        let u = fy_lin.abs() / cap.max(1e-6);
        if u > 1.0 {
            // past the peak: sliding, gripping less the further it slides
            let k = g.slide / g.mu.max(1e-6);
            let f = cap * (k + (1.0 - k) * (-(u - 1.0) * 2.0).exp()) * fy_lin.signum();
            w.patch[1] = -f / ky;
            slide = slide.max((vy.abs() - 0.15 * ax).max(0.0));
            f
        } else {
            fy_lin
        }
    };
    // A steerable tailwheel pushed past its steering springs breaks out.
    if t.caster > 0.0 && leg.steer != 0.0 && !w.free && fy_lin.abs() > 0.35 * nf {
        w.free = true;
    }
    Push { f: n * nf + h * fx + s * fy, nf, slide, heading: h, along: vx, across: vy }
}

/// The wheels left the ground: the footprint lets go.
pub fn lift_off(w: &mut Wheel) {
    w.patch = [0.0; 2];
    w.skid = false;
    w.on = false;
    if !w.free {
        w.swivel *= 0.9;
    }
}
