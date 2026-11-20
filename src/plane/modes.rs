//! Handling regression harness: the dynamic modes and control response of
//! each aircraft, measured by flying it in an empty world the way a flight
//! test card does (trim, a doublet or pulse, hands off, watch it settle).
//!
//! `performance_is_*` in plane.rs checks the steady numbers (speeds, climb,
//! glide); this checks how the aircraft moves between them, which is what
//! the flight model's rotational side decides:
//!
//! - short period: the quick pitch oscillation after an elevator pulse;
//! - phugoid: the slow exchange of speed and height, hands off;
//! - Dutch roll: the yaw-roll wobble after a rudder pulse;
//! - roll rate: the steady rate and time to 45° of bank at full aileron;
//! - spiral: what the bank does in 20 s hands off from 20°.
//!
//! Every run prints a table row (run with `--nocapture` to see it) with
//! reference figures beside it. The assertions are deliberately loose:
//! they catch a mode going unstable or vanishing, not small tuning moves,
//! so a change to the flight model can be judged by the numbers it prints.
//!
//! Reference figures (light aircraft from Roskam, Airplane Flight Dynamics,
//! part I, Appendix B, the Cessna 182 at cruise as the nearest tabulated
//! type; the 747 from Heffley & Jewell, NASA CR-2144, 1972, case at Mach 0.8
//! / 40,000 ft and the power-approach case) are typical values, not
//! measurements of these exact variants: they show where the modes ought to
//! fall, within perhaps ±30%.

use super::*;
use crate::aero;
use crate::aircraft::{b747, c172};

const DT: f64 = 1.0 / 180.0;

fn empty_world() -> World {
    let dir = std::env::temp_dir().join("fablegame-test-cache");
    let src = std::sync::Arc::new(crate::sources::Sources::new(crate::fetch::Fetcher::new(dir, true), String::new()));
    World::new(crate::sources::Loader::new(src, 0), 14)
}

/// An oscillation fitted from its peaks.
#[derive(Debug, Clone, Copy)]
pub struct Osc {
    /// s (0 when there was no oscillation to see)
    pub period: f64,
    /// damping ratio (1 when it never came back past the mean)
    pub zeta: f64,
}

/// Period and damping of `x` (sampled every `dt`) about its final value,
/// from the successive extremes (logarithmic decrement).
pub fn fit_osc(x: &[f64], dt: f64) -> Osc {
    let n = x.len();
    let tail = n / 5;
    let mean = x[n - tail..].iter().sum::<f64>() / tail.max(1) as f64;
    let y: Vec<f64> = x.iter().map(|v| v - mean).collect();
    let amp0 = y.iter().fold(0.0f64, |m, v| m.max(v.abs()));
    // Extremes between zero crossings, ignoring noise below 3% of the swing.
    let mut peaks: Vec<(usize, f64)> = Vec::new();
    let mut i = 0;
    while i < n {
        let s = y[i].signum();
        let mut best = (i, y[i]);
        let start = i;
        while i < n && y[i].signum() == s {
            if y[i].abs() > best.1.abs() {
                best = (i, y[i]);
            }
            i += 1;
        }
        if best.1.abs() > 0.03 * amp0 && i < n && i - start > 2 {
            peaks.push(best);
        }
    }
    if peaks.len() < 3 {
        return Osc { period: 0.0, zeta: 1.0 };
    }
    let k = peaks.len() - 1;
    let period = 2.0 * (peaks[k].0 - peaks[0].0) as f64 * dt / k as f64;
    // Half-cycle decrements, averaged.
    let mut d = 0.0;
    for w in peaks.windows(2) {
        d += (w[0].1.abs() / w[1].1.abs()).ln();
    }
    let delta = 2.0 * d / k as f64;
    Osc { period, zeta: delta / (4.0 * std::f64::consts::PI.powi(2) + delta * delta).sqrt() }
}

fn bank(p: &Plane) -> f64 {
    (-p.right().dot(p.pos.normalize())).asin()
}

fn level_wings(p: &Plane) -> f64 {
    (-(-p.right().dot(p.pos.normalize())) * 3.0 + p.omega.z * 0.5).clamp(-1.0, 1.0)
}

fn trimmed(spec: &'static Spec, alt: f64, speed: f64) -> (Plane, World) {
    trimmed_in(spec, alt, speed, None)
}

/// Trimmed at this height and speed, with the flaps at a notch and the gear down if `cfg` says so.
fn trimmed_in(spec: &'static Spec, alt: f64, speed: f64, cfg: Option<u8>) -> (Plane, World) {
    let w = empty_world();
    let mut p = Plane::spawn(spec, 0.6, -2.1, alt, 0.0, speed);
    if let Some(n) = cfg {
        p.configure(n, true, 0.0);
    }
    // Settle hands off (wings held level) so the run starts from steady flight.
    for _ in 0..(5.0 / DT) as usize {
        let c = Controls { roll: level_wings(&p), throttle_set: Some(p.throttle), ..Default::default() };
        assert!(p.step(DT, &c, &w).is_none());
    }
    (p, w)
}

/// Run `secs`, controls from `ctl(t, plane)`, recording `rec(plane)`.
fn record(p: &mut Plane, w: &World, secs: f64, mut ctl: impl FnMut(f64, &Plane) -> Controls, rec: impl Fn(&Plane) -> f64) -> Vec<f64> {
    let thr = p.throttle;
    let mut out = Vec::with_capacity((secs / DT) as usize);
    for i in 0..(secs / DT) as usize {
        let mut c = ctl(i as f64 * DT, p);
        c.throttle_set.get_or_insert(thr);
        assert!(p.step(DT, &c, w).is_none(), "crashed during the test");
        out.push(rec(p));
    }
    out
}

/// What the harness measured for one aircraft at one condition.
#[derive(Debug, Clone, Copy)]
pub struct Modes {
    pub short: Osc,
    pub phugoid: Osc,
    pub dutch: Osc,
    /// steady roll rate at full aileron, °/s, and seconds to 45° of bank
    pub roll_rate: f64,
    pub to_45: f64,
    /// bank after 20 s hands off from 20°, degrees (more than 20: spiral divergent)
    pub spiral: f64,
}

pub fn measure_in(spec: &'static Spec, alt: f64, speed: f64, cfg: Option<u8>) -> Modes {
    let trimmed = |spec, alt, speed| trimmed_in(spec, alt, speed, cfg);
    // Short period: a half-second pull, then hands off; the angle of attack rings.
    let (mut p, w) = trimmed(spec, alt, speed);
    let a = record(&mut p, &w, 8.0, |t, p| Controls { pitch: if t < 0.5 { 0.4 } else { 0.0 }, roll: level_wings(p), ..Default::default() }, |p| p.alpha);
    let short = fit_osc(&a[(0.5 / DT) as usize..], DT);

    // Phugoid: two seconds' push, hands off; the airspeed swings slowly.
    let (mut p, w) = trimmed(spec, alt, speed);
    let v = record(&mut p, &w, 300.0, |t, p| Controls { pitch: if t < 2.0 { -0.25 } else { 0.0 }, roll: level_wings(p), ..Default::default() }, |p| p.airspeed().length());
    let phugoid = fit_osc(&v[(2.0 / DT) as usize..], DT);

    // Dutch roll: a one-second rudder pulse, hands and feet off.
    let (mut p, w) = trimmed(spec, alt, speed);
    let r = record(&mut p, &w, 30.0, |t, _| Controls { yaw: if t < 1.0 { 0.6 } else { 0.0 }, ..Default::default() }, |p| p.omega.y);
    let dutch = fit_osc(&r[(1.0 / DT) as usize..], DT);

    // Roll: full aileron from wings level.
    let (mut p, w) = trimmed(spec, alt, speed);
    let mut to_45 = f64::NAN;
    let mut peak: f64 = 0.0;
    let b = record(&mut p, &w, 4.0, |_, _| Controls { roll: 1.0, ..Default::default() }, bank);
    for (i, &x) in b.iter().enumerate() {
        if to_45.is_nan() && x > 45f64.to_radians() {
            to_45 = i as f64 * DT;
        }
        if i > 0 && x < 60f64.to_radians() {
            peak = peak.max((x - b[i - 1]) / DT);
        }
    }
    let roll_rate = peak.to_degrees();

    // Spiral: roll to 20°, let go, see where the bank goes in 20 s.
    let (mut p, w) = trimmed(spec, alt, speed);
    let target = 20f64.to_radians();
    record(&mut p, &w, 8.0, |_, p| Controls { roll: ((target - bank(p)) * 3.0 + p.omega.z * 0.5).clamp(-1.0, 1.0), ..Default::default() }, bank);
    let b = record(&mut p, &w, 20.0, |_, _| Controls::default(), bank);
    let spiral = b.last().unwrap().to_degrees();

    Modes { short, phugoid, dutch, roll_rate, to_45, spiral }
}

fn osc_str(o: &Osc, digits: usize) -> String {
    if o.period == 0.0 { "no overshoot".into() } else { format!("{:.*} s ζ {:.3}", digits, o.period, o.zeta) }
}

fn report(name: &str, m: &Modes, refs: &str) {
    println!(
        "{name}: short period {} | phugoid {} | Dutch roll {} | roll {:.0}°/s, 45° in {:.1} s | spiral 20° → {:.0}° in 20 s",
        osc_str(&m.short, 1), osc_str(&m.phugoid, 0), osc_str(&m.dutch, 1), m.roll_rate, m.to_45, m.spiral
    );
    println!("    reference: {refs}");
}

#[test]
fn the_fit_reads_a_damped_oscillation() {
    let (wn, zeta) = (2.0f64, 0.15f64);
    let wd = wn * (1.0 - zeta * zeta).sqrt();
    let x: Vec<f64> = (0..20_000).map(|i| i as f64 * 0.001).map(|t| 3.0 + (-zeta * wn * t).exp() * (wd * t).cos()).collect();
    let o = fit_osc(&x, 0.001);
    assert!((o.period / (std::f64::consts::TAU / wd) - 1.0).abs() < 0.01, "{o:?}");
    assert!((o.zeta - zeta).abs() < 0.01, "{o:?}");
    let flat: Vec<f64> = (0..1000).map(|i| (-(i as f64) * 0.01).exp()).collect();
    assert_eq!(fit_osc(&flat, 0.01).period, 0.0);
}

/// The flying-qualities criteria every aircraft is held to: MIL-F-8785C
/// (Military Specification, Flying Qualities of Piloted Airplanes, 1980),
/// Level 1, flight phase category B (cruise, climb, descent), the standard
/// civil certification and simulator work also lean on:
///
/// - short period: damping ratio 0.30 to 2.0 (§3.2.2.1.2, table V);
/// - phugoid: damping ratio at least 0.04 (§3.2.1.2);
/// - Dutch roll: damping ratio at least 0.08, ζω at least 0.15 rad/s and
///   frequency at least 0.4 rad/s (§3.3.1.1, table VI);
/// - spiral: if it diverges, no faster than doubling the bank in 20 s
///   (§3.3.1.3, table VIII): from 20°, no more than 40° after 20 s.
///
/// A civil transport is certified to FAR/CS 25.181, which asks for no
/// phugoid damping at all, only that it be controllable: for one, the phugoid
/// is held to MIL-F-8785C Level 3 instead (time to double at least 55 s), as
/// Concorde's, flown on autothrottle, needs (a slender delta below its
/// minimum-drag speed subsonic and the air's thinning with height at Mach 2
/// leave its long-period mode about neutral).
///
/// Returns what fails.
pub fn level_1(m: &Modes, transport: bool) -> Vec<String> {
    let mut bad = Vec::new();
    if m.short.period > 0.0 && m.short.zeta < 0.30 {
        bad.push(format!("short period ζ {:.2} < 0.30", m.short.zeta));
    }
    if m.phugoid.period > 0.0 {
        let w = std::f64::consts::TAU / m.phugoid.period;
        let t2 = if m.phugoid.zeta < 0.0 { std::f64::consts::LN_2 / (-m.phugoid.zeta * w) } else { f64::INFINITY };
        if transport && t2 < 55.0 {
            bad.push(format!("phugoid doubles in {t2:.0} s < 55 s"));
        } else if !transport && m.phugoid.zeta < 0.04 {
            bad.push(format!("phugoid ζ {:.3} < 0.04", m.phugoid.zeta));
        }
    }
    if m.dutch.period > 0.0 {
        // (FAR 25.181(b) asks of a transport's only that it be positively damped; Level 1 is kept for all.)
        let wd = std::f64::consts::TAU / m.dutch.period;
        let wn = wd / (1.0 - m.dutch.zeta.powi(2)).max(1e-6).sqrt();
        if m.dutch.zeta < 0.08 || m.dutch.zeta * wn < 0.15 || wn < 0.4 {
            bad.push(format!("Dutch roll ζ {:.3}, ζω {:.2}, ω {:.2}", m.dutch.zeta, m.dutch.zeta * wn, wn));
        }
    }
    if m.spiral.abs() > 40.0 {
        bad.push(format!("spiral 20° → {:.0}° in 20 s", m.spiral));
    }
    if m.roll_rate.is_nan() || m.roll_rate <= 5.0 {
        bad.push(format!("roll rate {:.0}°/s", m.roll_rate));
    }
    bad
}

fn check(name: &str, spec: &'static Spec, alt: f64, speed: f64, refs: &str) {
    check_in(name, spec, alt, speed, None, refs)
}

/// Civil transports (see `level_1`).
fn transport(spec: &Spec) -> bool {
    spec.keys.iter().any(|k| ["747", "concorde"].contains(k))
}

fn check_in(name: &str, spec: &'static Spec, alt: f64, speed: f64, cfg: Option<u8>, refs: &str) {
    assert!(spec.rig.is_some(), "{name} flies on a rig of its own");
    let m = measure_in(spec, alt, speed, cfg);
    report(name, &m, refs);
    let bad = level_1(&m, transport(spec));
    assert!(bad.is_empty(), "{name} misses MIL-F-8785C Level 1: {}", bad.join("; "));
}

#[test]
fn handling_of_a_cessna_172() {
    check("C172 at 107 kt, 1500 m", &c172::SPEC, 1500.0, 55.0, "Roskam (C182, cruise): short period ~1.3 s ζ ~0.8 | phugoid ~30 s ζ ~0.1 | Dutch roll ~2.5 s ζ ~0.2 | roll ~45-70°/s | spiral near neutral");
}

#[test]
fn handling_of_a_robin_dr400() {
    check("DR400 at 117 kt, 1500 m", &crate::aircraft::dr400::SPEC, 1500.0, 60.0, "light-aircraft values as the C172's; crisper in roll");
}

#[test]
fn handling_of_a_747() {
    let h = 10_668.0;
    check("747 at M0.84, FL350", &b747::SPEC, h, 0.84 * sound_speed(h), "Heffley & Jewell: short period ~7 s ζ ~0.35 | phugoid ~90-100 s ζ ~0.05 | Dutch roll ~5-6 s, ζ ~0.05-0.1 without the yaw dampers (on here) | roll ~10-20°/s");
}

#[test]
fn handling_of_a_747_on_approach() {
    check_in("747 on approach, 150 kt, flaps 30, gear down, 600 m", &b747::SPEC, 600.0, 77.0, Some(6), "Heffley & Jewell power approach: short period ~6 s | Dutch roll ~6-7 s");
}

#[test]
fn handling_of_a_g650() {
    check("G650 at M0.85, FL410", &crate::aircraft::g650::SPEC, 12_500.0, 0.85 * sound_speed(12_500.0), "MIL-F-8785C Level 1 (its yaw damper on)");
}

#[test]
fn handling_of_a_concorde_subsonic() {
    check("Concorde at M0.93, FL290", &crate::aircraft::concorde::SPEC, 8_840.0, 0.93 * sound_speed(8_840.0), "MIL-F-8785C Level 1 (its autostabilisation on)");
}

#[test]
fn handling_of_a_concorde_at_mach_2() {
    check("Concorde at M2.0, FL550", &crate::aircraft::concorde::SPEC, 16_760.0, 2.0 * sound_speed(16_760.0), "MIL-F-8785C Level 1 (its autostabilisation on)");
}

#[test]
fn handling_of_a_phantom() {
    check("F-4E at M0.9, FL350", &crate::aircraft::f4::SPEC, 10_668.0, 0.9 * sound_speed(10_668.0), "Heffley & Jewell F-4C (dampers on): short period ~2.5-3.5 s | Dutch roll ~3 s | roll ~150°/s");
}

#[test]
fn handling_of_a_hornet() {
    check("F/A-18C at M0.7, 6000 m", &crate::aircraft::fa18::SPEC, 6_000.0, 0.7 * sound_speed(6_000.0), "fly-by-wire: MIL-F-8785C Level 1 | roll ~180-220°/s");
}

#[test]
fn handling_of_a_raptor() {
    check("F-22A at M0.9, 9000 m", &crate::aircraft::f22::SPEC, 9_000.0, 0.9 * sound_speed(9_000.0), "fly-by-wire: MIL-F-8785C Level 1 | roll ~200°/s");
}

#[test]
fn handling_of_a_blackbird() {
    check("SR-71A at M0.9, 9000 m", &crate::aircraft::sr71::SPEC, 9_000.0, 0.9 * sound_speed(9_000.0), "MIL-F-8785C Level 1 (its stability augmentation on)");
}

#[test]
fn handling_of_a_blackbird_at_mach_3() {
    check("SR-71A at M3.0, 24000 m", &crate::aircraft::sr71::SPEC, 24_000.0, 3.0 * sound_speed(24_000.0), "MIL-F-8785C Level 1 (its stability augmentation on)");
}

/// Slow down at idle a knot a second, wings level, until the wing lets go;
/// the lowest indicated airspeed reached (kt), and whether the nose dropped.
pub fn stall_speed(spec: &'static Spec, notch: u8) -> (f64, bool) {
    let (mut p, w) = trimmed(spec, 1500.0, spec.approach.0 * 1.3);
    p.flap_set = notch;
    let mut target = p.ias();
    let (mut min_ias, mut dropped) = (f64::MAX, false);
    for i in 0..(90.0 / DT) as usize {
        target -= 0.514 * DT;
        let err = p.ias() - target;
        let c = Controls { pitch: (err * 0.08 + p.omega.x * 0.0).clamp(-1.0, 1.0), roll: level_wings(&p), throttle_set: Some(0.0), ..Default::default() };
        assert!(p.step(DT, &c, &w).is_none());
        if i as f64 * DT > 10.0 {
            min_ias = min_ias.min(p.ias());
            if p.stalled && p.omega.x < -0.05 {
                dropped = true;
                break;
            }
        }
    }
    (min_ias * 1.943_84, dropped)
}

#[test]
fn a_cessna_172_stalls_where_the_book_says() {
    // Cessna 172RG POH, at 2650 lb, power off, most forward CG: 50 KIAS
    // clean, 42 KIAS flaps 30° (ours is 2425 lb: a few knots less).
    let (clean, d0) = stall_speed(&c172::SPEC, 0);
    let (full, d3) = stall_speed(&c172::SPEC, 3);
    println!("C172 stall: {clean:.0} kt clean (nose drop {d0}), {full:.0} kt flaps 30° (nose drop {d3}); POH 50 / 42 KIAS at 2650 lb");
    assert!((44.0..54.0).contains(&clean), "{clean}");
    assert!((36.0..47.0).contains(&full), "{full}");
    assert!(full < clean - 3.0);
}

#[test]
fn every_aircraft_can_be_rigged_from_its_spec() {
    // The generic rig: built from what each `Spec` already says, every
    // aircraft's strips carry its weight at its cruise speed and fly a
    // minute hands off without coming apart. (Tuning each one to its real
    // handling is its own `Rig`'s job; this is the starting point.)
    let (mut flying, mut total) = (0, 0);
    for spec in crate::aircraft::list() {
        if spec.rocket().is_some() {
            continue;
        }
        let rig: &'static aero::Rig = Box::leak(Box::new(aero::Rig::derive(spec)));
        let s: &'static Spec = Box::leak(Box::new(Spec { rig: Some(rig), ..spec.clone() }));
        let w = empty_world();
        let h = if spec.is_jet() { 6_000.0 } else { 1_500.0 };
        let mut p = Plane::spawn(s, 0.6, -2.1, h, 0.0, spec.cruise);
        let h0 = geo::pos_to_lla(p.pos).2;
        for _ in 0..(60.0 / DT) as usize {
            let c = Controls { roll: level_wings(&p), throttle_set: Some(p.throttle), ..Default::default() };
            assert!(p.step(DT, &c, &w).is_none(), "{} crashed", spec.name);
        }
        let dh = geo::pos_to_lla(p.pos).2 - h0;
        let v = p.airspeed().length();
        println!("{}: trim {:+.2}, alpha {:.1}°, after 60 s {:+.0} m, {:.0} → {:.0} m/s", spec.name, p.pitch_trim, p.alpha.to_degrees(), dh, spec.cruise, v);
        assert!(p.omega.is_finite() && v.is_finite());
        let settled = p.airframe.is_some() && dh.abs() < 100.0 && (v - spec.cruise).abs() < 0.1 * spec.cruise;
        if !settled {
            println!("    {} does not hold its height and speed on its derived rig", spec.name);
        }
        flying += settled as usize;
        total += 1;
    }
    println!("{flying} of {total} fly trimmed on their derived rig");
    assert_eq!(flying, total, "every aircraft should fly on the rig derived from its spec");
}

/// The stability derivatives the strips give at a trimmed condition (body
/// axes, per radian, the usual signs: Cl positive right wing down, Cn
/// positive nose right), by perturbing the airflow and the rotation.
#[derive(Debug, Clone, Copy)]
pub struct Derivs {
    pub cl_alpha: f64,
    pub cm_alpha: f64,
    pub cm_q: f64,
    pub cl_beta: f64,
    pub cn_beta: f64,
    pub cy_beta: f64,
    pub cl_p: f64,
    pub cn_p: f64,
    pub cl_r: f64,
    pub cn_r: f64,
    pub l_over_d: f64,
}

pub fn derivs(spec: &'static Spec, alt: f64, speed: f64) -> Derivs {
    let p = Plane::spawn(spec, 0.6, -2.1, alt, 0.0, speed);
    let af = p.airframe.as_deref().expect("a rig");
    let (rho, a) = (density(alt), sound_speed(alt));
    let alpha = p.trim;
    let base = |vb: DVec3, om: DVec3| {
        let mut inp = p.strip_input(af, vb, (rho, a), 1.0);
        inp.omega = om;
        (inp.elevator, inp.aileron, inp.rudder) = (0.0, 0.0, 0.0);
        let mut st = aero::State { sep: vec![0.0; af.strips.len()], eps: 0.0, gust: Vec::new() };
        let mut out = af.forces(&inp, &mut st, 0.0);
        for _ in 0..3 {
            st.eps = af.rig.downwash * 2.0 * out.cl_wing / (std::f64::consts::PI * af.ar);
            out = af.forces(&inp, &mut st, 0.0);
        }
        out
    };
    let vb = |al: f64, be: f64| DVec3::new(speed * be.sin(), -speed * al.sin() * be.cos(), -speed * al.cos() * be.cos());
    let q = 0.5 * rho * speed * speed;
    let (s, b, c) = (af.s_ref, af.b_ref, af.mac);
    let d = 0.01;
    let o0 = base(vb(alpha, 0.0), DVec3::ZERO);
    let lift = |o: &aero::Output, al: f64| o.force.dot(DVec3::X.cross(vb(al, 0.0) / speed).normalize());
    let drag = -o0.force.dot(vb(alpha, 0.0) / speed);
    let oa = base(vb(alpha + d, 0.0), DVec3::ZERO);
    let ob = base(vb(alpha, d), DVec3::ZERO);
    let rate = |w: f64, l: f64| w * 2.0 * speed / l;
    let oq = base(vb(alpha, 0.0), DVec3::new(rate(d, c), 0.0, 0.0));
    let op = base(vb(alpha, 0.0), DVec3::new(0.0, 0.0, -rate(d, b)));
    let or = base(vb(alpha, 0.0), DVec3::new(0.0, -rate(d, b), 0.0));
    let cm = |o: &aero::Output| o.moment.x / (q * s * c);
    let cl = |o: &aero::Output| -o.moment.z / (q * s * b);
    let cn = |o: &aero::Output| -o.moment.y / (q * s * b);
    Derivs {
        cl_alpha: (lift(&oa, alpha + d) - lift(&o0, alpha)) / (q * s * d),
        cm_alpha: (cm(&oa) - cm(&o0)) / d,
        cm_q: (cm(&oq) - cm(&o0)) / d,
        cl_beta: (cl(&ob) - cl(&o0)) / d,
        cn_beta: (cn(&ob) - cn(&o0)) / d,
        cy_beta: (ob.force.x - o0.force.x) / (q * s * d),
        cl_p: (cl(&op) - cl(&o0)) / d,
        cn_p: (cn(&op) - cn(&o0)) / d,
        cl_r: (cl(&or) - cl(&o0)) / d,
        cn_r: (cn(&or) - cn(&o0)) / d,
        l_over_d: lift(&o0, alpha) / (drag + q * s * 0.0),
    }
}
#[test]
fn a_robin_dr400_stalls_where_the_book_says() {
    // Robin DR 400/180 flight manual §5.1: 105 km/h (57 KIAS) clean, 95 km/h
    // (51 KIAS) landing flaps, at 1100 kg; scaled to the weight it flies at here.
    let p = Plane::spawn(&crate::aircraft::dr400::SPEC, 0.6, -2.1, 1500.0, 0.0, 50.0);
    let k = (p.mass() / 1100.0).sqrt();
    let (clean, d0) = stall_speed(&crate::aircraft::dr400::SPEC, 0);
    let (full, d2) = stall_speed(&crate::aircraft::dr400::SPEC, 2);
    println!("DR400 stall at {:.0} kg: {clean:.0} kt clean (nose drop {d0}), {full:.0} kt flaps 60° (nose drop {d2}); manual {:.0} / {:.0} KIAS at this weight", p.mass(), 57.0 * k, 51.0 * k);
    assert!((clean - 57.0 * k).abs() < 4.0, "{clean}");
    assert!((full - 51.0 * k).abs() < 4.0, "{full}");
}

#[test]
fn every_aircraft_is_statically_stable_and_damped() {
    // The signs a flyable airframe needs at a typical flight point, from
    // the strip model's own force and moment differences: pitch and weather-
    // cock stability, dihedral effect, and damping in pitch, roll and yaw.
    use crate::aircraft::{b747, c172, concorde, dr400, f22, f4, fa18, g650, sr71};
    let h = 10668.0;
    let points: [(&'static Spec, f64, f64); 9] = [
        (&c172::SPEC, 1500.0, 55.0),
        (&dr400::SPEC, 1500.0, 60.0),
        (&b747::SPEC, h, 0.84 * sound_speed(h)),
        (&g650::SPEC, 12_500.0, 0.85 * sound_speed(12_500.0)),
        (&f4::SPEC, h, 0.9 * sound_speed(h)),
        (&fa18::SPEC, h, 0.85 * sound_speed(h)),
        (&f22::SPEC, h, 0.9 * sound_speed(h)),
        (&concorde::SPEC, 8840.0, 285.0),
        (&sr71::SPEC, 9000.0, 273.0),
    ];
    for (spec, alt, v) in points {
        let d = derivs(spec, alt, v);
        println!("{:24} CLa {:.2} Cma {:.3} Cmq {:.1} | Clb {:.3} Cnb {:.3} CYb {:.2} | Clp {:.3} Cnp {:.3} Clr {:.3} Cnr {:.3} | L/D {:.1}", spec.name, d.cl_alpha, d.cm_alpha, d.cm_q, d.cl_beta, d.cn_beta, d.cy_beta, d.cl_p, d.cn_p, d.cl_r, d.cn_r, d.l_over_d);
        assert!(d.cl_alpha > 1.5, "{}: lift slope {}", spec.name, d.cl_alpha);
        // A fly-by-wire airframe may be relaxed (the F-22 is, subsonic):
        // its pitch law holds it, so it only must not run away fast.
        let fbw = spec.rig.is_some_and(|r| r.fcs.g_law);
        assert!(d.cm_alpha < if fbw { 0.1 } else { 0.0 }, "{}: pitch unstable, Cma {}", spec.name, d.cm_alpha);
        assert!(d.cn_beta > 0.0, "{}: no weathercock stability, Cnb {}", spec.name, d.cn_beta);
        assert!(d.cl_beta < 0.0, "{}: no dihedral effect, Clb {}", spec.name, d.cl_beta);
        assert!(d.cy_beta < 0.0, "{}: sideslip force the wrong way, CYb {}", spec.name, d.cy_beta);
        assert!(d.cm_q < 0.0 && d.cl_p < 0.0 && d.cn_r < 0.0, "{}: an undamped axis", spec.name);
    }
}


