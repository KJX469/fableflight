//! Ground handling (gear.rs) on every aircraft: standing, rolling, braking,
//! steering, touching down; on dry, wet and snowy ground, pavement and grass.
use super::*;
use crate::aircraft::{Leg, Tyre, BUILTIN};
use crate::gear::{self, Surface};
use std::sync::Arc;

const DT: f64 = 1.0 / 180.0;
const KT: f64 = 1.943_84;

fn world(surface: Surface) -> World {
    let dir = std::env::temp_dir().join("fablegame-test-cache");
    let src = Arc::new(crate::sources::Sources::new(crate::fetch::Fetcher::new(dir, true), String::new()));
    let mut w = World::new(crate::sources::Loader::new(src, 0), 14);
    w.flat = Some(0.0);
    w.flat_surface = surface;
    w
}

/// Every aircraft that stands on wheels.
fn wheeled() -> impl Iterator<Item = &'static Spec> {
    BUILTIN.iter().copied().filter(|s| !s.gear.is_empty() && s.rocket().is_none())
}

/// On its wheels on the runway, released, engines at idle, rolling at `v` m/s.
fn on_runway(spec: &'static Spec, v: f64) -> Plane {
    let mut p = Plane::spawn_ground(spec, 0.66, -2.13, 0.0, 0.5, 0.0);
    p.hold = false;
    p.t = 2.0;
    p.vel = p.forward() * v;
    for (gi, l) in spec.gear.iter().enumerate() {
        p.wheel_spin[gi] = v / l.wheel_r;
    }
    p
}

fn up(p: &Plane) -> DVec3 {
    p.pos.normalize()
}
fn ground_speed(p: &Plane) -> f64 {
    let u = up(p);
    (p.vel - u * p.vel.dot(u)).length()
}
/// Turning to the left (rad/s).
fn yaw_rate(p: &Plane) -> f64 {
    p.omega.dot(p.rot.inverse() * up(p))
}
fn idle() -> Controls {
    Controls { throttle_set: Some(0.0), ..Default::default() }
}

/// Steps `secs`; false if it crashed.
fn run(p: &mut Plane, w: &World, secs: f64, mut c: impl FnMut(&Plane) -> Controls) -> bool {
    for _ in 0..(secs / DT) as usize {
        let ctl = c(p);
        if p.step(DT, &ctl, w).is_some() {
            return false;
        }
        p.notes.clear();
        p.shed.clear();
    }
    true
}

/// Brakes on from `v`: how far it goes before it stops (m), and whether any wheel skidded.
fn stop_from(spec: &'static Spec, w: &World, v: f64, wet: f64, snow: f64) -> (f64, bool) {
    let mut p = on_runway(spec, v);
    p.ground_wx = gear::Cond { rain: wet.min(1.0), snow };
    p.water = wet * 3.0;
    let start = p.pos;
    let mut skid = false;
    let mut t = 0.0;
    while ground_speed(&p) > 0.5 && t < 120.0 {
        assert!(p.step(DT, &Controls { brake: true, ..idle() }, w).is_none(), "{} crashed braking", spec.name);
        // (keep the water standing: the test is about the runway as it is)
        p.water = wet * 3.0;
        // (a wheel brake-locked: stopped while the aircraft still rolls on)
        skid |= (0..spec.gear.len()).any(|i| p.wheels[i].skid && p.wheels[i].on && p.wheel_spin[i] * spec.gear[i].wheel_r < 0.3 * ground_speed(&p) && ground_speed(&p) > 3.0);
        t += DT;
    }
    ((p.pos - start).length(), skid)
}

#[test]
fn every_aircraft_stands_still_on_its_gear() {
    let w = world(Surface::Concrete);
    for spec in wheeled() {
        let mut p = on_runway(spec, 0.0);
        p.parking_brake = true;
        assert!(run(&mut p, &w, 20.0, |_| idle()), "{} crashed standing", spec.name);
        let start = p.pos;
        assert!(run(&mut p, &w, 10.0, |_| idle()));
        let drift = (p.pos - start).length();
        let loads = gear::static_loads(spec, p.mass());
        println!("{}: drift {drift:.4} m in 10 s, speed {:.4} m/s, rate {:.4} rad/s, strokes {:?}, loads {:?}", spec.name, p.vel.length(), p.omega.length(), &p.compress[..spec.gear.len()], &loads[..spec.gear.len()]);
        assert!(drift < 0.02, "{} creeps {drift} m", spec.name);
        assert!(p.omega.length() < 0.002, "{} rocks", spec.name);
        for (gi, l) in spec.gear.iter().enumerate() {
            assert!(p.compress[gi] > 0.0 && p.compress[gi] < l.travel, "{} {} stroke {}", spec.name, l.name, p.compress[gi]);
            assert!(p.dmg.gear[gi]);
        }
        // The legs share the weight.
        assert!((loads.iter().sum::<f64>() - p.mass() * 9.81).abs() < 1.0);
    }
}

#[test]
fn brakes_hold_against_run_up_power() {
    let w = world(Surface::Concrete);
    for spec in [&crate::aircraft::c172::SPEC, &crate::aircraft::dr400::SPEC] {
        let mut p = on_runway(spec, 0.0);
        p.parking_brake = true;
        assert!(run(&mut p, &w, 5.0, |_| idle()));
        let start = p.pos;
        assert!(run(&mut p, &w, 10.0, |_| Controls { throttle_set: Some(0.55), ..Default::default() }));
        let d = (p.pos - start).length();
        println!("{}: {d:.3} m at run-up power, {:.0} rpm", spec.name, p.prop_rate * 60.0 / std::f64::consts::TAU);
        assert!(d < 0.05, "{} slides {d} m on its brakes", spec.name);
        // Brakes off, it taxis away.
        p.parking_brake = false;
        assert!(run(&mut p, &w, 5.0, |_| Controls { throttle_set: Some(0.35), ..Default::default() }));
        assert!(ground_speed(&p) > 1.5, "{} does not taxi", spec.name);
    }
}

#[test]
fn every_aircraft_brakes_to_a_stop_and_wet_or_snow_takes_longer() {
    let w = world(Surface::Concrete);
    for spec in wheeled() {
        let v = spec.approach.0 * 0.6;
        let (dry, dry_skid) = stop_from(spec, &w, v, 0.0, 0.0);
        let (wet, _) = stop_from(spec, &w, v, 1.0, 0.0);
        let (snow, snow_skid) = stop_from(spec, &w, v, 0.0, 1.0);
        let decel = v * v / (2.0 * dry);
        let anti = spec.gear.iter().any(|l| l.tyre.antiskid);
        println!("{}: from {:.0} kt dry {dry:.0} m ({decel:.1} m/s², skid {dry_skid}), wet {wet:.0} m, snow {snow:.0} m (skid {snow_skid}), anti-skid {anti}", spec.name, v * KT);
        // Wheel brakes alone (no reversers, no spoilers needed): 0.2-0.6 g.
        assert!(decel > 1.8 && decel < 6.5, "{}: {decel} m/s²", spec.name);
        // (wet grips less; it only stops longer where the brakes could out-grip it)
        assert!(wet > dry * 0.99, "{}: wet {wet} vs dry {dry}", spec.name);
        if spec.gear.iter().any(|l| l.brakes && l.brake_mu > 0.5) {
            assert!(wet > dry * 1.1, "{}: wet {wet} vs dry {dry}", spec.name);
        }
        assert!(snow > dry * 1.1, "{}: snow {snow} vs dry {dry}", spec.name);
        if anti {
            assert!(!dry_skid && !snow_skid, "{}: anti-skid let a wheel lock", spec.name);
        }
    }
}

#[test]
fn without_anti_skid_a_light_aircraft_locks_its_wheels_on_snow() {
    let w = world(Surface::Asphalt);
    let (_, skid) = stop_from(&crate::aircraft::c172::SPEC, &w, 15.0, 0.0, 1.0);
    assert!(skid);
}

#[test]
fn tyres_aquaplane_past_nine_root_psi() {
    let leg = &crate::aircraft::b747::SPEC.gear[1];
    let cond = gear::Cond { rain: 1.0, snow: 0.0 };
    let vp = gear::aquaplane_speed(leg.tyre.pressure, false);
    println!("747 aquaplanes at {:.0} kt", vp * KT);
    assert!((vp * KT - 9.0 * (200.0f64).sqrt()).abs() < 2.0);
    let slow = gear::grip(leg, Surface::Concrete, &cond, 4.0, vp * 0.7, false);
    let fast = gear::grip(leg, Surface::Concrete, &cond, 4.0, vp * 1.1, false);
    let damp = gear::grip(leg, Surface::Concrete, &cond, 0.3, vp * 1.1, false);
    let dry = gear::grip(leg, Surface::Concrete, &gear::Cond::default(), 0.0, vp * 1.1, false);
    println!("mu: dry {:.2}, damp {:.2}, flooded below {:.2}, above {:.2}", dry.mu, damp.mu, slow.mu, fast.mu);
    assert!(fast.aqua > 0.9 && fast.mu < 0.1);
    assert!(slow.aqua < 0.01 && slow.mu > 0.3);
    assert!(damp.aqua < 0.01 && damp.mu < dry.mu);
    // A light aircraft's soft tyres aquaplane much slower.
    let c172 = &crate::aircraft::c172::SPEC.gear[1];
    assert!(gear::aquaplane_speed(c172.tyre.pressure, false) * KT < 65.0);
}

#[test]
fn grass_drags_and_bumps() {
    let paved = world(Surface::Asphalt);
    let grass = world(Surface::Turf);
    let roll = |w: &World| {
        let mut p = on_runway(&crate::aircraft::c172::SPEC, 0.0);
        let start = p.pos;
        let mut shake: f64 = 0.0;
        assert!(run(&mut p, w, 12.0, |p| {
            if p.t > 6.0 {
                shake = shake.max((p.g_load - 1.0).abs());
            }
            Controls { throttle_set: Some(1.0), pitch: -0.2, ..Default::default() }
        }));
        ((p.pos - start).length(), shake)
    };
    let (d_paved, s_paved) = roll(&paved);
    let (d_grass, s_grass) = roll(&grass);
    println!("12 s at full power: {d_paved:.0} m on asphalt, {d_grass:.0} m on grass; peak load change {s_paved:.3} vs {s_grass:.3} g");
    assert!(d_grass < d_paved * 0.95);
    assert!(s_grass > s_paved * 2.0);
    // A jet's hard narrow tyres bog down in the grass far worse.
    let jet = &crate::aircraft::fa18::SPEC;
    let lg = gear::grip(&jet.gear[1], Surface::Turf, &gear::Cond::default(), 0.0, 10.0, false);
    let lc = gear::grip(&crate::aircraft::c172::SPEC.gear[1], Surface::Turf, &gear::Cond::default(), 0.0, 10.0, false);
    assert!(lg.roll > lc.roll * 1.5);
}

#[test]
fn touchdown_spins_the_wheels_up_with_a_puff_of_rubber() {
    let w = world(Surface::Concrete);
    for spec in wheeled() {
        let v = spec.approach.0;
        let mut p = on_runway(spec, v);
        // A metre up, sinking at 1.5 m/s, wheels stopped.
        p.pos += up(&p) * 0.08;
        p.vel -= up(&p) * 1.5;
        p.wheel_spin = [0.0; MAX_GEAR];
        p.smoke.clear();
        let main = spec.gear.iter().position(|l| l.steer == 0.0).unwrap();
        let mut rolled = None;
        let mut t = 0.0;
        let mut max_up: f64 = 0.0;
        let mut touched = false;
        let mut all = [false; MAX_GEAR];
        while t < 4.0 {
            let level = Controls { roll: (p.right().dot(up(&p)) * 3.0 + p.omega.z * 0.5).clamp(-1.0, 1.0), pitch: if touched { -0.3 } else { 0.0 }, ..idle() };
            assert!(p.step(DT, &level, &w).is_none(), "{} crashed landing", spec.name);
            for (gi, wh) in p.wheels.iter().enumerate().take(spec.gear.len()) {
                all[gi] |= wh.on;
            }
            t += DT;
            touched |= p.on_ground;
            if touched {
                max_up = max_up.max(p.vel.dot(up(&p)));
            }
            let r = spec.gear[main].wheel_r;
            if touched && rolled.is_none() && p.wheels[main].on && !p.wheels[main].skid && (p.wheel_spin[main] * r - p.vel.length()).abs() < 1.0 {
                rolled = Some(t);
            }
        }
        println!("{}: touched {touched}, wheels rolling after {rolled:?} s, rebound {max_up:.2} m/s, smoke {}", spec.name, p.smoke.len());
        assert!(touched && rolled.is_some(), "{}: wheels never spun up", spec.name);
        assert!(all.iter().take(spec.gear.len()).all(|&a| a), "{}: not every wheel came down", spec.name);
        assert!(!p.smoke.is_empty(), "{}: no rubber smoke", spec.name);
        assert!(max_up < 0.8, "{}: bounced off at {max_up} m/s", spec.name);
        assert!(p.dmg.gear.iter().take(spec.gear.len()).all(|&g| g), "{}: lost gear", spec.name);
    }
}

#[test]
fn every_aircraft_runs_straight_and_turns_on_the_ground() {
    let w = world(Surface::Concrete);
    for spec in wheeled() {
        // Full power along the runway, hands off: it tracks straight.
        let mut p = on_runway(spec, 0.0);
        let (start, dir) = (p.pos, p.forward());
        let vr = spec.approach.0 / 1.3 * 0.8;
        let mut t = 0.0;
        let mut pedal_max: f64 = 0.0;
        while ground_speed(&p) < vr && t < 60.0 {
            // the pilot keeps it on the heading with the pedals (a single
            // pulls left at full power, as the real one does)
            let yaw_err = dir.cross(p.forward()).dot(up(&p));
            let pedal = (yaw_err * 8.0 + p.omega.dot(p.rot.inverse() * up(&p)) * 2.0).clamp(-1.0, 1.0);
            pedal_max = pedal_max.max(pedal.abs());
            assert!(p.step(DT, &Controls { throttle_set: Some(1.0), pitch: -0.3, yaw: pedal, ..Default::default() }, &w).is_none());
            p.notes.clear();
            t += DT;
        }
        let d = p.pos - start;
        let side = (d - dir * d.dot(dir) - up(&p) * d.dot(up(&p))).length();
        // heading only, across the ground (not the nose rising as it nears flying speed)
        let flat = |v: DVec3| (v - up(&p) * v.dot(up(&p))).normalize();
        let yaw = flat(p.forward()).dot(flat(dir)).clamp(-1.0, 1.0).acos().to_degrees();
        let pitch = (p.forward().dot(up(&p)).asin() - dir.dot(up(&p)).asin()).to_degrees();
        println!("{}: {:.0} kt after {t:.0} s, {:.0} m: {side:.2} m off the line, {yaw:.2}° off heading, nose {pitch:+.1}°, pedal up to {pedal_max:.2}", spec.name, ground_speed(&p) * KT, d.length());
        assert!(side < 2.0 && yaw < 2.0, "{} wanders on the runway", spec.name);
        assert!(pedal_max < 0.9, "{} needs full pedal to hold the line", spec.name);

        // Taxiing at 8 kt with full rudder: it turns, on a sensible radius.
        let mut p = on_runway(spec, 8.0 / KT);
        let c = |p: &Plane| {
            let v = ground_speed(p);
            Controls { yaw: 1.0, throttle_set: Some(((8.0 / KT - v) * 0.15 + 0.3).clamp(0.0, 1.0)), brake_right: if v > 9.0 / KT { 0.3 } else { 0.0 }, ..Default::default() }
        };
        let mut turned = 0.0;
        assert!(run(&mut p, &w, 12.0, |p| {
            turned += p.omega.dot(p.rot.inverse() * up(p)).abs() * DT;
            c(p)
        }), "{} crashed turning", spec.name);
        let turned: f64 = turned.to_degrees();
        let rate = p.omega.dot(p.rot.inverse() * up(&p)).abs();
        let radius = ground_speed(&p) / rate.max(1e-6);
        println!("   taxi turn: {turned:.0}° in 12 s, radius {radius:.1} m at {:.1} kt", ground_speed(&p) * KT);
        assert!(turned > 40.0, "{} barely turns", spec.name);
        assert!(radius > 1.0 && radius < 4.0 * spec.span as f64, "{}: radius {radius}", spec.name);
        assert!(p.right().dot(up(&p)).abs() < 0.1, "{} leans over in the turn", spec.name);
    }
}

/// A taildragger made from the Cessna's data: its nose leg moved to the tail
/// as a steerable tailwheel that breaks out to full swivel (as hangar.rs
/// rigs a downloaded one), and its mains well ahead of the centre of gravity
/// (wherever the load puts it).
fn taildragger(caster_nose: bool) -> &'static Spec {
    let base = &crate::aircraft::c172::SPEC;
    let mut s = base.clone();
    let mut legs: Vec<Leg> = base.gear.to_vec();
    if caster_nose {
        // a free-castering nose wheel: steered with the brakes
        legs[0].steer = 0.0;
        legs[0].tyre = Tyre { caster: std::f64::consts::PI, ..legs[0].tyre };
    } else {
        let y = legs[0].at.y;
        legs[0].at = DVec3::new(0.0, y + 0.55, 4.6);
        legs[0].wheel_r = 0.1;
        legs[0].tyre = Tyre { caster: std::f64::consts::PI, ..legs[0].tyre };
        for l in legs.iter_mut().skip(1) {
            l.at.z = -0.7;
        }
    }
    s.gear = Box::leak(legs.into_boxed_slice());
    Box::leak(Box::new(s))
}

#[test]
fn a_tailwheel_steers_breaks_out_and_locks_again() {
    let w = world(Surface::Asphalt);
    let spec = taildragger(false);
    let mut p = Plane::spawn_ground(spec, 0.66, -2.13, 0.0, 0.5, 0.0);
    p.hold = false;
    p.t = 2.0;
    // It sits back on its tail.
    assert!(run(&mut p, &w, 5.0, |_| idle()));
    let pitch = p.forward().dot(up(&p)).asin().to_degrees();
    println!("taildragger sits {pitch:.1}° nose up");
    assert!(pitch > 3.0);
    // Taxi straight with a little rudder: the tailwheel stays locked.
    assert!(run(&mut p, &w, 8.0, |p| Controls { throttle_set: Some(((4.0 - ground_speed(p)) * 0.2 + 0.35).clamp(0.0, 1.0)), yaw: 0.1, ..Default::default() }));
    assert!(!p.wheels[0].free, "the tailwheel broke out on a gentle turn");
    assert!(ground_speed(&p) > 2.0);
    // A hard brake-and-rudder pivot: it breaks out to swivel.
    let mut broke = false;
    assert!(run(&mut p, &w, 5.0, |p| Controls { throttle_set: Some(0.3), yaw: 1.0, brake_right: 1.0, ..Default::default() }.also(&mut broke, p.wheels[0].free)));
    println!("pivoted, broke out {broke}, swivel {:.2}", p.wheels[0].swivel);
    assert!(broke, "the tailwheel never broke out");
    // Straightened out with the brakes against the swing (a taildragger
    // wants to keep turning): it locks back on.
    assert!(run(&mut p, &w, 10.0, |p| {
        let r = yaw_rate(p);
        Controls { throttle_set: Some(if ground_speed(p) < 3.0 { 0.3 } else { 0.0 }), brake_left: (-r * 4.0).clamp(0.0, 1.0), brake_right: (r * 4.0).clamp(0.0, 1.0), ..Default::default() }
    }));
    println!("straightened: {:.1} m/s, turning {:.3} rad/s, swivel {:.2}", ground_speed(&p), yaw_rate(&p), p.wheels[0].swivel);
    assert!(!p.wheels[0].free, "the tailwheel never locked again (swivel {:.2})", p.wheels[0].swivel);
}

#[test]
fn a_castering_nose_wheel_follows_and_the_brakes_steer() {
    let w = world(Surface::Asphalt);
    let spec = taildragger(true);
    let mut p = on_runway(spec, 4.0);
    // The right brake turns it right (yaw to the right is negative about up).
    let mut turned = 0.0;
    assert!(run(&mut p, &w, 3.0, |p| {
        turned += yaw_rate(p) * DT;
        Controls { throttle_set: Some(if ground_speed(p) < 4.0 { 0.35 } else { 0.1 }), brake_right: 0.6, ..Default::default() }
    }));
    println!("castering nose: turned {:.0}° (+ left), swivel {:.2}", turned.to_degrees(), p.wheels[0].swivel);
    assert!(turned < -0.3, "did not turn right on the right brake");
    assert!(p.wheels[0].swivel > 0.05, "the nose wheel did not swivel into the turn");
}

trait Also: Sized {
    fn also(self, flag: &mut bool, v: bool) -> Self {
        *flag |= v;
        self
    }
}
impl Also for Controls {}


#[test]
fn a_float_slides_on_land_and_stops() {
    // the Cessna on floats: its legs are pontoons, nothing rolls
    let mut s = crate::aircraft::c172::SPEC.clone();
    s.floats = true;
    let spec: &'static Spec = Box::leak(Box::new(s));
    let w = world(Surface::Turf);
    let mut p = on_runway(spec, 0.0);
    // At rest it stays put, even with some power on.
    assert!(run(&mut p, &w, 4.0, |_| Controls { throttle_set: Some(0.15), ..Default::default() }));
    assert!(ground_speed(&p) < 0.3, "float creeps at {} m/s", ground_speed(&p));
    // Coasting onto the grass at 10 m/s, it slides to a stop far sooner than wheels.
    let stop = |spec: &'static Spec| {
        let mut p = on_runway(spec, 10.0);
        let start = p.pos;
        assert!(run(&mut p, &w, 20.0, |_| idle()), "{} crashed", spec.name);
        ((p.pos - start).length(), ground_speed(&p))
    };
    let (d_float, v_float) = stop(spec);
    let (d_wheels, _) = stop(&crate::aircraft::c172::SPEC);
    println!("from 10 m/s on grass: floats {d_float:.1} m, wheels {d_wheels:.1} m");
    assert!(v_float < 0.3 && d_float < 25.0 && d_float < d_wheels * 0.5);
}
