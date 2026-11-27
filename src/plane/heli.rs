//! a helicopter: the collective (the throttle lever) sets the main rotor's
//! thrust, the cyclic (the stick) tilts the disc and with it the fuselage,
//! the pedals work the tail rotor against the main rotor's torque. the rotor
//! turns at its governed speed while the engine runs and is kept up by the
//! air in an autorotation when it stops (collective down). thrust grows with
//! translational lift past about 15 kt and in ground effect within a rotor
//! diameter of the ground; it stands on skids. a handling model tuned to a
//! light piston helicopter's published figures, not blade element theory.

use super::{density, Impact, Plane, RHO0};
use crate::aircraft::Rotor;
use crate::geo;
use crate::world::World;
use glam::{DQuat, DVec3};

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Plane {
    pub(super) fn step_rotor(&mut self, dt: f64, c: &super::Controls, world: &World, rt: &Rotor) -> Option<Impact> {
        let spec = self.spec;
        self.dmg_prev = self.dmg;
        self.dmg.vibe = 0.0;
        // the collective: the lever stays where it is put.
        self.throttle = match c.throttle_set {
            Some(t) => t.clamp(0.0, 1.0),
            None => (self.throttle + c.throttle * dt * 0.45).clamp(0.0, 1.0),
        };
        let target = DVec3::new(c.pitch, c.yaw, c.roll);
        self.stick += (target - self.stick) * (dt * 6.0).min(1.0);
        let (lat, lon, h) = geo::pos_to_lla(self.pos);
        let upw = self.pos.normalize();
        let rho = density(h);
        let ground = world.ground(lat, lon);
        let mass = self.mass();
        let inertia = spec.inertia;
        let w0 = rt.rpm * std::f64::consts::TAU / 60.0;
        // an air start comes with the rotor turning.
        if !self.on_ground && self.running && self.prop_rate < 1.0 {
            self.prop_rate = w0;
        }
        // the engine: it fires with the magnetos, fuel and the starter (or the rotor turning it).
        let cranking = (c.starter || self.auto_start > 0.0) && self.sys.powered();
        self.cranking = cranking;
        let turning = self.prop_rate > 0.15 * w0 || cranking;
        self.running = self.ignition && self.fuel > 0.0 && self.mixture > 0.08 && turning && self.dmg.engine > 0.0 && !self.sys.fail.engine;
        if self.auto_start > 0.0 {
            self.auto_start = if self.running { 0.0 } else { self.auto_start - dt };
        }
        // the rotor's speed: the governor holds it with the engine running; without, the
        // air through the disc keeps it turning in a descent with the collective down.
        let air_v = self.vel - self.wind;
        let vz = air_v.dot(upw);
        let w = self.prop_rate;
        let load = self.throttle;
        let drive = if self.running { (w0 * 1.02 - w) * 0.9 } else { 0.0 };
        let windmill = if !self.running { (-vz).max(0.0) * 0.55 * (1.0 - load) - 2.0 * load } else { 0.0 };
        let drag = 0.012 * w * (0.4 + load) * rho / RHO0;
        self.prop_rate = (w + (drive + windmill - drag + if cranking && !self.running { 3.0 } else { 0.0 }) * dt).clamp(0.0, w0 * 1.15);
        self.prop += self.prop_rate * dt;
        let rpm_k = self.prop_rate / w0;
        if self.running && rpm_k < 0.9 && !self.on_ground {
            self.note("LOW ROTOR RPM");
        }
        // fuel: about 40 kg an hour in a hover in an r44, at full collective a bit more.
        let ff = spec.fuel_flow;
        self.fuel_flow = if self.running { ff[0] + (ff[1] - ff[0]) * load } else { 0.0 };
        self.fuel = (self.fuel - self.fuel_flow * dt).max(0.0);
        self.power = load * rpm_k;
        let charging = self.running && rpm_k > 0.8;
        self.sys.update(dt, charging, cranking, true);

        // the main rotor's thrust, along the disc tilted by the cyclic.
        let agl = h - ground.as_ref().map_or(h - 1000.0, |g| g.h);
        let horiz = (air_v - upw * vz).length();
        let tl = 1.0 + 0.16 * smooth(6.0, 14.0, horiz) - 0.10 * smooth(45.0, 70.0, horiz);
        let ge = 1.0 + 0.14 * (1.0 - (agl / (2.0 * rt.radius)).clamp(0.0, 1.0));
        let inflow = (1.0 - 0.045 * vz).clamp(0.6, 1.35);
        let thrust = rt.thrust * (load * 1.08 - 0.04).max(0.0) * rpm_k * rpm_k * (rho / RHO0) * tl * ge * inflow;
        // (forward stick tips the disc forward, toward -z; right stick to the right, +x)
        let disc = DQuat::from_rotation_x(self.stick.x * 0.10) * DQuat::from_rotation_z(-self.stick.z * 0.09);
        let disc_up = disc * DVec3::Y;
        let mut force_b = disc_up * thrust;
        // the fuselage's drag through the air, and its weathervaning tail.
        let vb = self.rot.inverse() * air_v;
        let v = vb.length();
        force_b -= vb * (0.5 * rho * v * rt.drag_area);
        // the tail rotor against the torque: the pedals set its thrust.
        let torque = rt.thrust * rt.radius * 0.11 * load * rpm_k * if self.running { 1.0 } else { 0.15 };
        let pedal = -self.stick.y * rt.pedal * rpm_k * rpm_k;
        let mut torque_b = DVec3::ZERO;
        // (the main rotor turns anticlockwise seen from above: the fuselage swings nose right without left pedal)
        torque_b.y += -torque + torque * 0.9 + pedal;
        // the cyclic tilts the disc and the fuselage hangs under it; damped by the rotor.
        let auth = rpm_k * rpm_k * (0.5 + 0.5 * (thrust / (mass * 9.81)).clamp(0.0, 1.5));
        torque_b.x += inertia.x * (self.stick.x * rt.cyclic * auth - self.omega.x * 2.2);
        torque_b.z += inertia.z * (-self.stick.z * rt.cyclic * 1.1 * auth - self.omega.z * 2.6);
        // (moving sideways, the tail swings the nose round into the way it is going)
        torque_b.y += inertia.y * (-self.omega.y * 1.6 - if v > 15.0 { vb.x / v * 0.8 * (v / 40.0).min(1.5) } else { 0.0 });
        // a light stability augmentation: tipped over, it eases back toward level.
        let up_b = self.rot.inverse() * upw;
        torque_b.x += inertia.x * (up_b.z * 0.6);
        torque_b.z += inertia.z * (-up_b.x * 0.5);
        let g = 9.81;
        let mut force_w = self.rot * force_b - upw * (mass * g) + self.gust * mass * 0.05;
        // the skids: four points with a spring and damper and the friction of steel
        // on the ground; too hard, too far over, or the blades in the ground: a crash.
        self.on_ground = false;
        if let Some(gr) = ground.as_ref().filter(|g| h - g.h < rt.radius * 2.0 + 10.0) {
            let height = |p: DVec3| {
                let (la, lo, ph) = geo::pos_to_lla(p);
                world.ground(la, lo).map(|g| (g.h - ph, g.water, g.h))
            };
            // the blade tips and the tail rotor.
            for k in 0..8 {
                let a = std::f64::consts::TAU * k as f64 / 8.0;
                let tip = rt.hub + disc * DVec3::new(a.cos() * rt.radius, 0.0, a.sin() * rt.radius);
                if let Some((pen, water, gh)) = height(self.pos + self.rot * tip) {
                    if pen > 0.0 && rpm_k > 0.2 {
                        self.note("ROTOR STRIKE");
                        return self.impact(Impact { pos: self.pos + self.rot * tip, vel: self.vel, water, ground_h: gh, building: None }, "rotor strike");
                    }
                }
            }
            if let Some((pen, water, gh)) = height(self.pos + self.rot * rt.tail) {
                if pen > 0.0 && rpm_k > 0.2 {
                    self.note("TAIL ROTOR STRIKE");
                    return self.impact(Impact { pos: self.pos + self.rot * rt.tail, vel: self.vel, water, ground_h: gh, building: None }, "tail rotor strike");
                }
            }
            let (ks, cs) = (mass * 9.81 / 4.0 / 0.05, mass * 2.0);
            for leg in spec.gear.iter() {
                let rb = leg.at;
                let p = self.pos + self.rot * rb;
                let Some((pen, water, gh)) = height(p) else { continue };
                if pen <= 0.0 {
                    continue;
                }
                let pv = self.vel + self.rot * self.omega.cross(rb);
                let vn = pv.dot(upw);
                if vn < -3.8 || water || up_b.y < 0.72 {
                    return self.impact(Impact { pos: p, vel: self.vel, water, ground_h: gh, building: None }, if water { "water" } else if up_b.y < 0.72 { "rolled over" } else { "hard landing" });
                }
                self.on_ground = true;
                let nf = (pen * ks - vn * cs).max(0.0);
                let tang = pv - upw * vn;
                let mu = if self.parking_brake || c.brake { 0.7 } else { 0.45 };
                let f = upw * nf - tang.normalize_or_zero() * (mu * nf).min(tang.length() * cs);
                force_w += f;
                torque_b += rb.cross(self.rot.inverse() * f);
            }
            let _ = gr;
        }
        if self.on_ground && self.throttle < 0.3 {
            // settled on the skids, it stays put.
            self.omega *= 0.9;
        }
        // the motion.
        let acc = force_w / mass;
        self.sf = self.rot.inverse() * (acc + upw * g);
        self.g_load = self.sf.y / 9.81;
        self.vel += acc * dt;
        self.pos += self.vel * dt;
        self.omega += (torque_b - self.omega.cross(inertia * self.omega)) / inertia * dt;
        self.omega = self.omega.clamp_length_max(3.0);
        self.rot = (self.rot * DQuat::from_scaled_axis(self.omega * dt)).normalize();
        self.alpha = (-vb.y).atan2(-vb.z.min(-0.1));
        self.dmg.vibe += ((1.0 - rpm_k).abs() * 0.6 + smooth(60.0, 75.0, horiz) * 0.5) as f32;
        if horiz > rt.vne * 1.15 && !self.on_ground {
            self.note("RETREATING BLADE STALL");
            self.omega.z += 0.4 * dt;
        }
        None
    }
}
