//! The C-130H's dimensional contract: every position, outline and envelope
//! the exterior, the flight deck, the flight model and the paint share. The
//! other files build to these numbers; change one here only knowing that the
//! others were built to it (the tests at the bottom check the facts that
//! keep them consistent).
//!
//! # Frame
//!
//! Body frame: +X right, +Y up, -Z forward. The origin is the reference
//! centre of gravity: at 25% of the mean aerodynamic chord, `ZN` behind the
//! nose tip, and `YG` above the ground with the aircraft at rest on its gear
//! at the reference mass (`Spec::mass`). Drawings are read as `d`, the
//! distance back from the nose tip, and `h`, the height above the ground:
//! `bz(d)` and `by(h)` turn them into body z and y. Lockheed fuselage
//! stations (FS, inches) convert with `fs`.
//!
//! # Sources (fetched 2026-11; [2+] two or more agree, [1] one source, [est] estimate)
//!
//! - Overall: length 97 ft 9 in (29.79 m), span 132 ft 7 in (40.41 m), height
//!   38 ft 3 in (11.66 m), wing area 1,745 ft² (162.1 m²), aspect ratio 10.1
//!   [2+]: https://en.wikipedia.org/wiki/Lockheed_C-130_Hercules,
//!   https://www.globalsecurity.org/military/systems/aircraft/c-130-specs.htm,
//!   https://aerospaceweb.org/aircraft/transport-m/c130/ . (38 ft 10 in is
//!   the C-130J's; the H's is 38 ft 3 in.)
//! - Wing: NACA 64A318 root, 64A412 tip [2+] (Lednicer,
//!   https://m-selig.ae.illinois.edu/ads/aircraft.html); MAC 164.5 in
//!   (4.178 m), Jane's via
//!   https://rotorcraft.arc.nasa.gov/Publications/files/KottapalliA_AHS_2010.pdf
//!   [1]; dihedral 2°30', incidence 3° root / 0° tip, quarter-chord sweep 0°
//!   [1] (Jane's-derived, https://baike.baidu.com/en/item/C-130%20Hercules/5058110);
//!   centre wing to outer wing joint at WS 220, inboard nacelle on the WS 178
//!   and 213 ribs [1] (Lockheed, https://asipcon.com/proceedings/proceedings_2006/2006_PDFs/Tuesday/0430_Bateman.pdf).
//!   C-130H wing and empennage chords: Structural Loads Handbook, Appendix C,
//!   tables C.4–C.6: https://fenix.tecnico.ulisboa.pt/downloadFile/395143117996/58159%20-%20STRUCTURAL%20LOADS%20HANDBOOK.pdf
//!   The 192 in centre wing starts tapering at WS 220 to a 100 in tip.
//!   These nominal chords give 162.91 m² before rounding the tips.
//!   Tailplane root/tip chords are 182.8/67.9 in, fin chords 250/74 in.
//!   Tail trailing-edge stations remain estimates from the three-view.
//! - Fuselage: cargo floor FS 245 to FS 737, ramp FS 737 (hinge) to FS 869,
//!   compartment 123 in wide, 108 in high [1] (AMC load planning workbook,
//!   https://milairx.com/wp-content/uploads/2020/12/AMC-ACLP-Workbook-36-101-Vol-II-1Jan15.pdf);
//!   cargo floor 41 in above the ground, compartment 10 ft 3 in wide, 9 ft 2 in
//!   high forward and 9 ft under the wing box, fuselage 14 ft 2 in (4.32 m)
//!   wide outside, track 14 ft 3 in, the nose gear about 11 ft 6 in aft of
//!   the nose [1] (C-130J pocket guide's general arrangement; same fuselage
//!   section, nose, wing and gear as the H:
//!   https://www.lockheedmartin.com/content/dam/lockheed-martin/aero/documents/C-130J/C130JPocketGuide.pdf);
//!   wheelbase 32 ft 0.75 in (9.77 m) [2+] (Jane's; the J-30's 40 ft 4 in less
//!   its 100 in plug). The outside height of the fuselage, the nose and tail
//!   profiles, the sponsons and the tail surfaces are NOT published: they are
//!   estimates from photographs and three-view drawings, kept consistent with
//!   the figures above (cargo floor, interior height, overall height).
//! - Propellers: Hamilton Standard 54H60, four blades, 13 ft 6 in (4.115 m)
//!   [2+], 1,020 rpm governed [1] (Lockheed Service News,
//!   https://www.lockheedmartin.com/content/dam/lockheed-martin/aero/documents/sustainment/csc/service-news/sn-mag-v21-v30/v26n3.pdf).
//!   Nacelles: inboard about WS 196 (4.98 m, between its attach ribs), outboard
//!   about 10.3 m (measured off the pocket guide's plan view, ±0.5 m) [est].
//! - Tail: tailplane span 52 ft 8 in (16.05 m) [2+]. Fin and tailplane chord sources above.
//! - Tyres: mains 56x20-20 (one each on four struts, two in tandem a side),
//!   nose twin 39x13-16: the commonly quoted C-130 sizes, NOT verified
//!   against a manual [est].
//! - Flight deck: no eye reference point, floor height or window layout was
//!   found in a public source (TO 1C-130H-1 figures are not freely
//!   available). The envelope below is an ESTIMATE: 23 windows in all [2+]
//!   (windshield, the windows above it, side windows and two windows below
//!   the pilots' eyes: a former RAF Hercules pilot,
//!   https://www.twz.com/air/confessions-of-a-c-130-hercules-pilot); their
//!   places and the nose's heights read off the three-view and photographs
//!   (`WINDOWS`, `FUSE`). `WINDOWS` cuts 18 of them (nine a side: the
//!   windshield's centre and corner panes and the small pane under the
//!   corner, two of the upper row's three, two of the side row's three, the
//!   two chin windows), so five of the real panes are not modelled. The
//!   floor (`DECK_FLOOR_H`) is put level with the chin windows' sills and
//!   the eye 1.20 m over it.
//! - Known departures of the flight-deck envelope (`DECK_*`, `OVHD_*`,
//!   `WINDOWS`) from the real aircraft, left as they are because the flight
//!   deck is built to them: the overhead panel (`OVHD_FWD_Z`..`OVHD_AFT_Z`,
//!   3.40-4.40 m from the nose) sits further aft than the real one, which
//!   starts over the windshield's top frame; the windshield's lower edge is
//!   about 10 cm lower than the photographs show, so that the line over the
//!   glareshield meets glass; the lining levels off 2.05 m over the floor
//!   (`DECK_HEADROOM`) where the real headliner follows the roof higher.
//!
//! The fuselage station of the nose tip is not published either: FS 245 is
//! put 6.95 m behind it (FS_NOSE, an estimate), which puts the ramp hinge
//! (FS 737) at 19.45 m and the main gear (13.26 m, from the nose gear at
//! 3.50 m and the wheelbase) under the wing's quarter chord.
#![allow(clippy::excessive_precision)]
use crate::airframe::{sd_poly, Foil, Sect};
use glam::{Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI};

// ---------------------------------------------------------------- frame

/// Nose tip to the reference centre of gravity, and the ground to it (m).
/// [est] The CG is put on the wing's quarter chord at 25% MAC, 1.16 m ahead
/// of the main gear (12% of the weight on the nose gear); its height is an
/// estimate for a loaded transport (floor 1.04 m, wing and engines at 4-5 m).
pub const ZN: f32 = 12.10;
pub const YG: f32 = 2.70;
/// Body z and y from drawing distance-from-nose and height-above-ground.
pub const fn bz(d: f32) -> f32 {
    d - ZN
}
pub const fn by(h: f32) -> f32 {
    h - YG
}
/// Fuselage station (inches) of the nose tip [est], and body z of a station.
#[cfg_attr(not(test), allow(dead_code))] // (a sourced figure the tests check the drawing against)
pub const FS_NOSE: f32 = -28.6;
#[cfg_attr(not(test), allow(dead_code))] // (a sourced figure the tests check the drawing against)
pub const fn fs(station: f32) -> f32 {
    bz((station - FS_NOSE) * 0.0254)
}

/// Headline dimensions (m): length nose to tail cone, span, height to the fin tip.
#[cfg_attr(not(test), allow(dead_code))] // (a sourced figure the tests check the drawing against)
pub const LENGTH: f32 = 29.79;
pub const B: f32 = 40.41;
pub const HEIGHT: f32 = 11.66;
/// Gross wing area (m²) and mean aerodynamic chord (m).
pub const WING_AREA: f32 = 162.12;
pub const MAC: f32 = 4.178;

// ---------------------------------------------------------------- fuselage

/// Fuselage sections: distance from the nose, half width, top, height of the
/// widest point, bottom (heights above the ground), squareness of the upper
/// and the lower half (2 an ellipse, more squarer). The constant section
/// (6.5 m to the ramp hinge) follows the near-circular outer shell in the
/// Lockheed pocket guide cross sections (pp. 10–11, reference above).
/// The 4.32 m beam and widest point 1.38 m above the cargo floor set the
/// scale. Crown at 4.68 m and keel at 0.55 m are estimates, allowing a
/// slightly fuller lower section around the floor rails (the Wikimedia
/// three-view's side view, scaled to the 29.79 m length, puts the crown at
/// 4.69 m and the belly at 0.58 m, and its fin tip at 11.7 m).
///
/// The nose (2026-11-24, read off that three-view's side and plan views
/// at 37.7 px/m, a head-on photograph of a KC-130H and a side photograph of
/// a C-130H's radome scaled by its 39 in nose tyre; listed in mod.rs): the
/// flight deck's roof is flush with the crown, as on every C-130. From the
/// radome's top (3.0 m high, about 1.4 m back) the windshield and the roof
/// over it climb steeply: 3.7 m high 2 m back, 4.4 m at 2.7 m, the crown's
/// height by 3.6 m. In plan the nose tapers smoothly, an ogive with no step
/// at the radome: 2.0 m across at the radome's base (1.75 m back), 2.7 m at
/// 2.3 m, 3.5 m at 3.1 m, 4.0 m at 4.3 m. The radome is about as wide as it
/// is high from ahead (2.06 m by 2.1 m on the photograph): round on top,
/// widest about 2.3 m up, its sides drawing in to a flattish bottom just
/// above the belly, its tip at 2.2 m, its chin sloping up to it. The lower
/// nose is slab-sided with a flat belly. (Earlier versions, 2026-11, kept
/// the roof low, about 4 m, under a crown that rose behind it, and the
/// radome first too round and then too narrow, 1.44 m across.) The tail
/// sweeps up from the ramp hinge (19.45 m) to the tail cone's tip at
/// 3.80 m. [est, except the width and the floor]
#[rustfmt::skip]
const FUSE: [[f32; 7]; 27] = [
    [0.00, 0.04, 2.22, 2.20, 2.18, 2.0, 2.0],
    [0.10, 0.34, 2.49, 2.21, 1.86, 2.0, 2.2],
    [0.30, 0.53, 2.64, 2.23, 1.66, 2.0, 2.4],
    [0.60, 0.66, 2.78, 2.25, 1.45, 2.0, 2.6],
    [1.00, 0.79, 2.89, 2.28, 1.28, 2.0, 2.8],
    [1.40, 0.90, 3.00, 2.31, 1.15, 2.0, 3.0],
    [1.60, 0.95, 3.20, 2.33, 1.08, 2.0, 3.1],
    [1.75, 1.00, 3.41, 2.34, 1.02, 2.0, 3.1],
    [2.00, 1.14, 3.73, 2.36, 0.95, 2.0, 3.1],
    [2.30, 1.35, 4.06, 2.38, 0.89, 2.0, 3.0],
    [2.70, 1.57, 4.37, 2.39, 0.81, 2.0, 3.0],
    [3.10, 1.75, 4.56, 2.40, 0.74, 2.0, 2.9],
    [3.60, 1.91, 4.66, 2.41, 0.69, 2.0, 2.8],
    [4.30, 2.03, 4.68, 2.42, 0.64, 2.0, 2.7],
    [5.30, 2.13, 4.68, 2.42, 0.59, 2.0, 2.6],
    [6.50, 2.16, FUSE_TOP_H, 2.42, FUSE_BOT_H, 2.0, 2.6],
    [19.45, 2.16, FUSE_TOP_H, 2.42, FUSE_BOT_H, 2.0, 2.6],
    [20.40, 2.15, 4.64, 2.46, 0.80, 2.0, 2.7],
    [21.60, 2.10, 4.48, 2.58, 1.15, 2.0, 2.5],
    [23.00, 1.98, 4.24, 2.78, 1.62, 2.0, 2.5],
    [24.50, 1.74, 4.11, 3.02, 2.15, 2.0, 2.3],
    [26.00, 1.40, 4.09, 3.27, 2.65, 2.0, 2.2],
    [27.40, 1.02, 4.06, 3.48, 3.06, 2.0, 2.1],
    [28.50, 0.62, 4.01, 3.64, 3.36, 2.0, 2.0],
    [29.30, 0.30, 3.94, 3.74, 3.58, 2.0, 2.0],
    [29.60, 0.14, 3.89, 3.78, 3.68, 2.0, 2.0],
    [29.79, 0.03, 3.86, 3.80, 3.74, 2.0, 2.0],
];
/// The fuselage from the nose tip to the tail cone's tip (drawing distances),
/// and where its parts join: the radome, the flight deck section (to the
/// flight deck's aft bulkhead, FS 245), the cargo ramp's hinge (FS 737; the
/// tail cone, ramp and aft door behind it).
pub const D_NOSE: f32 = 0.0;
pub const D_RADOME: f32 = 1.75;
pub const D_DECK: f32 = 6.95;
pub const D_RAMP: f32 = 19.45;
pub const D_TAIL: f32 = 29.79;
/// The radome's joint, seen from the side: upright at `D_RADOME` up to
/// `RADOME_JOINT[0]` high, then curving forward (d falling as the square of
/// the height) to `RADOME_JOINT[1]` back at `RADOME_JOINT[2]` high, where
/// it meets the top of the nose under the windshield. [est: the photograph
/// of 05-1085's radome listed in mod.rs, scaled by its 39 in nose tyre,
/// puts the upright part 1.66 m behind the tip, from 1.04 m up to 2.42 m,
/// and its top end about 0.37 m further forward at 3.0 m; the three-view
/// draws it 1.88 m back at the bottom and 1.43 m at the top.]
pub const RADOME_JOINT: [f32; 3] = [2.45, 1.40, 3.05];
/// The joint's distance from the nose at height h (d_j: the radome is the
/// skin ahead of it; plane.wgsl's `c130_joint`).
#[cfg_attr(not(test), allow(dead_code))] // (the shader's copy; the tests check the inverse against it)
pub fn radome_joint_d(h: f32) -> f32 {
    let u = ((h - RADOME_JOINT[0]) / (RADOME_JOINT[2] - RADOME_JOINT[0])).clamp(0.0, 1.0);
    D_RADOME - (D_RADOME - RADOME_JOINT[1]) * u * u
}
/// The joint's height at distance d (its inverse, on the curved part;
/// `RADOME_JOINT[2]` and above ahead of it).
pub fn radome_joint_h(d: f32) -> f32 {
    let u = ((D_RADOME - d) / (D_RADOME - RADOME_JOINT[1])).clamp(0.0, 1.0).sqrt();
    RADOME_JOINT[0] + u * (RADOME_JOINT[2] - RADOME_JOINT[0])
}
/// The glazed part of the nose the window glass sheet covers.
pub const D_GLASS: [f32; 2] = [1.5, 4.0];
/// The outside skin's constant section: half width, top, widest, bottom (heights).
pub const FUSE_HALF_W: f32 = 2.16;
pub const FUSE_TOP_H: f32 = 4.68;
pub const FUSE_MID_H: f32 = 2.42;
pub const FUSE_BOT_H: f32 = 0.55;
/// The cargo floor's height [1] and its stations (FS 245 to the ramp hinge, FS 737).
pub const CARGO_FLOOR_H: f32 = 1.04;
#[cfg_attr(not(test), allow(dead_code))] // (a sourced figure the tests check the drawing against)
pub const CARGO_D: [f32; 2] = [6.95, 19.45];

#[derive(Clone, Copy, Debug)]
pub struct Sec {
    pub w: f32,
    pub top: f32,
    pub mid: f32,
    pub bot: f32,
    pub nt: f32,
    pub nb: f32,
}

/// Monotone cubic through the table (no overshoot between stations).
fn pchip(col: usize, d: f32) -> f32 {
    let k = &FUSE;
    let n = k.len();
    if d <= k[0][0] {
        return k[0][col];
    }
    if d >= k[n - 1][0] {
        return k[n - 1][col];
    }
    let i = (0..n - 1).find(|&i| d < k[i + 1][0]).unwrap_or(n - 2);
    let h = |i: usize| k[i + 1][0] - k[i][0];
    let dd = |i: usize| (k[i + 1][col] - k[i][col]) / h(i);
    let slope = |i: usize| -> f32 {
        if i == 0 || i == n - 1 {
            return if i == 0 { dd(0) } else { dd(n - 2) };
        }
        let (a, b) = (dd(i - 1), dd(i));
        if a * b <= 0.0 {
            0.0
        } else {
            let (w1, w2) = (2.0 * h(i) + h(i - 1), h(i) + 2.0 * h(i - 1));
            (w1 + w2) / (w1 / a + w2 / b)
        }
    };
    let t = (d - k[i][0]) / h(i);
    let (y0, y1, m0, m1) = (k[i][col], k[i + 1][col], slope(i) * h(i), slope(i + 1) * h(i));
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * y0 + (t3 - 2.0 * t2 + t) * m0 + (-2.0 * t3 + 3.0 * t2) * y1 + (t3 - t2) * m1
}
/// The fuselage section at body station z (body frame heights). (Not the
/// sponsons: they are a fairing of their own, `spon`.)
pub fn sec(z: f32) -> Sec {
    let d = z + ZN;
    Sec { w: pchip(1, d), top: by(pchip(2, d)), mid: by(pchip(3, d)), bot: by(pchip(4, d)), nt: pchip(5, d), nb: pchip(6, d) }
}
/// Point on a section at angle th (0 = top, pi/2 = right side, pi = bottom).
pub fn sec_pt(s: &Sec, th: f32) -> Vec2 {
    let (sn, cs) = th.sin_cos();
    let n = if cs > 0.0 { s.nt } else { s.nb };
    let e = |v: f32| v.signum() * v.abs().powf(2.0 / n);
    let h = if cs > 0.0 { s.top - s.mid } else { s.mid - s.bot };
    Vec2::new(s.w * e(sn), s.mid + h * e(cs))
}
pub fn fuse_pt(z: f32, th: f32) -> Vec3 {
    let p = sec_pt(&sec(z), th);
    Vec3::new(p.x, p.y, z)
}
/// Outward normal of the fuselage skin at (z, th).
pub fn fuse_n(z: f32, th: f32) -> Vec3 {
    let e = 1e-3;
    let du = fuse_pt(z + e, th) - fuse_pt(z - e, th);
    let dv = fuse_pt(z, th + e) - fuse_pt(z, th - e);
    du.cross(dv).normalize()
}
/// Angle on the section at z whose point is at height y on the right side.
pub fn fuse_th(z: f32, y: f32) -> f32 {
    let s = sec(z);
    let (mut a, mut b) = (0.0f32, PI);
    for _ in 0..30 {
        let m = (a + b) * 0.5;
        if sec_pt(&s, m).y > y { a = m } else { b = m }
    }
    (a + b) * 0.5
}
/// The fuselage's half width at station z and height y (None above or below it).
pub fn half_width_at(z: f32, y: f32) -> Option<f32> {
    let s = sec(z);
    if y >= s.top || y <= s.bot {
        return None;
    }
    Some(sec_pt(&s, fuse_th(z, y)).x)
}
/// Point and outward normal on the skin at station z, height y, on side sx (+1 right), lifted by `off`.
pub fn on_side(z: f32, y: f32, sx: f32, off: f32) -> (Vec3, Vec3) {
    let th = fuse_th(z, y);
    let (mut p, mut n) = (fuse_pt(z, th), fuse_n(z, th));
    p.x *= sx;
    n.x *= sx;
    (p + n * off, n)
}
/// Height of the fuselage's lower skin at station z, lateral position x (None outside it).
pub fn fuse_bottom_y(z: f32, x: f32) -> Option<f32> {
    let s = sec(z);
    if x.abs() >= s.w * 0.999 {
        return None;
    }
    let x = x.abs();
    let (mut a, mut b) = (FRAC_PI_2, PI);
    for _ in 0..30 {
        let m = (a + b) * 0.5;
        if sec_pt(&s, m).x > x { a = m } else { b = m }
    }
    Some(sec_pt(&s, (a + b) * 0.5).y)
}

// ---------------------------------------------------------------- sponsons

/// The main gear fairings ("sponsons") on the lower fuselage sides, modelled
/// as a fairing of their own rather than in `sec`: `sec` stays the clean
/// pressure hull (what `Spec::fuse`, the damage model and the hole use) and
/// the sponson is a body joined to its side. Each section is a superellipse
/// quadrant (squareness `SPON_N`) from an inner edge buried in the fuselage
/// (`SPON_XI`) out to `SPON_XO`, between `SPON_BOT` and `SPON_TOP`, its
/// ends rounded off over `SPON_D`. [est, from photographs: they reach from
/// about the cargo floor up to above mid-height, ahead of and behind the
/// tandem wheels]
pub const SPON_D: [f32; 2] = [9.90, 17.20];
pub const SPON_XI: f32 = 1.20;
pub const SPON_XO: f32 = 2.66;
pub const SPON_TOP_H: f32 = 2.70;
pub const SPON_BOT_H: f32 = 0.78;
pub const SPON_N: f32 = 4.0;

/// How full the sponson's section is at station z (1 in the middle, 0 at its ends).
fn spon_fill(z: f32) -> f32 {
    let d = z + ZN;
    if d <= SPON_D[0] || d >= SPON_D[1] {
        return 0.0;
    }
    let u = (2.0 * d - SPON_D[0] - SPON_D[1]) / (SPON_D[1] - SPON_D[0]);
    (1.0 - u.abs().powi(4)).max(0.0).sqrt()
}
/// Point on the right sponson at station z, t from 0 (top, inner edge) round
/// the outside to 1 (bottom, inner edge).
pub fn spon_pt(z: f32, t: f32) -> Vec3 {
    let f = spon_fill(z);
    let a = (SPON_XO - SPON_XI) * f;
    let ym = by(0.5 * (SPON_TOP_H + SPON_BOT_H));
    let bb = 0.5 * (SPON_TOP_H - SPON_BOT_H) * f.sqrt();
    let ph = t.clamp(0.0, 1.0) * PI;
    let (sn, cs) = ph.sin_cos();
    let e = |v: f32| v.signum() * v.abs().powf(2.0 / SPON_N);
    Vec3::new(SPON_XI + a * e(sn), ym + bb * e(cs), z)
}
/// Height of the sponson's underside at (z, |x|), if it is there.
pub fn spon_bottom_y(z: f32, x: f32) -> Option<f32> {
    let f = spon_fill(z);
    let a = (SPON_XO - SPON_XI) * f;
    let x = x.abs() - SPON_XI;
    if f <= 0.0 || x <= 0.0 || x >= a {
        return None;
    }
    let ym = by(0.5 * (SPON_TOP_H + SPON_BOT_H));
    let bb = 0.5 * (SPON_TOP_H - SPON_BOT_H) * f.sqrt();
    Some(ym - bb * (1.0 - (x / a).powf(SPON_N)).max(0.0).powf(1.0 / SPON_N))
}
/// The lowest skin at (z, x): the fuselage's or a sponson's.
pub fn skin_bottom(z: f32, x: f32) -> f32 {
    match (fuse_bottom_y(z, x), spon_bottom_y(z, x)) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        _ => sec(z).bot,
    }
}

// ---------------------------------------------------------------- wing

/// Span station of the wing tip (m out from the centreline).
pub const SPAN: f32 = 20.205;
/// Centre and tapered outer panels: chord on the centreline and at the tip, the
/// leading edge on the centreline (quarter chord unswept: the centre of
/// gravity's station, `ZN`, is the quarter chord's).
pub const CR: f32 = 4.8768;
pub const CT: f32 = 2.54;
pub const TAPER_X: f32 = 5.588;
pub const LE0_D: f32 = ZN - 0.25 * CR;
/// Where the MAC lies: its span station and its leading edge's distance from the nose.
#[cfg_attr(not(test), allow(dead_code))] // (a sourced figure the tests check the drawing against)
pub const MAC_X: f32 = TAPER_X + (SPAN - TAPER_X) * (CR - MAC) / (CR - CT);
#[cfg_attr(not(test), allow(dead_code))] // (a sourced figure the tests check the drawing against)
pub const LEMAC_D: f32 = ZN - 0.25 * MAC;
/// Lateral reference for the wing root. The centre wing crosses the
/// upper cargo shell and blends into the shoulders through its fairing.
pub const WING_ROOT_X: f32 = FUSE_HALF_W;
/// The wing's mean plane at the root (height), and its dihedral (2°30').
pub const WING_ROOT_H: f32 = 4.46;
pub const DIHEDRAL: f32 = 0.043661; // tan 2.5°

pub fn chord(x: f32) -> f32 {
    CR - (CR - CT) * outer_fraction(x)
}
fn outer_fraction(x: f32) -> f32 {
    ((x.abs() - TAPER_X) / (SPAN - TAPER_X)).clamp(0.0, 1.0)
}
/// Leading edge (distance from the nose) at span station x.
pub fn le_d(x: f32) -> f32 {
    LE0_D + 0.25 * (CR - chord(x))
}
/// Height of the wing's mean plane at span station x (flat across the
/// centre wing, 2°30' of dihedral outboard of it).
pub fn wing_mid_h(x: f32) -> f32 {
    WING_ROOT_H + DIHEDRAL * (x.abs() - TAPER_X).max(0.0)
}
/// Thickness to chord: 18% at the root (64A318) to 12% at the tip (64A412).
pub fn wing_tc(x: f32) -> f32 {
    0.18 - 0.06 * outer_fraction(x)
}
/// Incidence (rad): 3° at the root washing out to 0° at the tip.
pub fn wing_inc(x: f32) -> f32 {
    (3.0 * (1.0 - outer_fraction(x))).to_radians()
}
/// Right wing section at span station x.
pub fn wing_sect(x: f32) -> Sect {
    let x = x.max(0.0);
    let le = bz(le_d(x));
    let c = chord(x);
    let inc = wing_inc(x);
    let dih = if x > TAPER_X { DIHEDRAL } else { 0.0 };
    let ym = by(wing_mid_h(x));
    Sect { le: Vec3::new(x, ym + inc.sin() * c * 0.5, le), c, cdir: Vec3::new(0.0, -inc.sin(), inc.cos()), up: Vec3::new(-dih, inc.cos(), inc.sin()).normalize(), tf: wing_tc(x) / 0.18 }
}
pub fn wing_foil() -> Foil {
    Foil { at: Box::new(wing_sect), t: 0.18, camber: 0.02 }
}
/// Control surfaces along the span (span stations): the inboard and outboard
/// Lockheed-Fowler flaps, the aileron [est: the flaps run from the fuselage
/// to the aileron, which takes the outer 30% of the semispan]. The chord
/// fraction each starts at [est].
pub const FLAP_IB: [f32; 2] = [2.25, 7.45];
pub const FLAP_OB: [f32; 2] = [7.55, 13.55];
pub const AILERON: [f32; 2] = [13.65, 19.70];
pub const XF_FLAP: f32 = 0.70;
pub const XF_AIL: f32 = 0.75;
/// Full flap travel (100%), degrees [est]; 50% is the take-off setting.
pub const FLAP_MAX_DEG: f32 = 36.0;
/// The fuel tanks' span (the four integral wing tanks and the two
/// auxiliary tanks in the centre wing) [est].
pub const TANK: [f32; 2] = [2.2, 18.0];

// ---------------------------------------------------------------- engines

/// The engines, right side (index 0 inboard, 1 outboard): span station of
/// the thrust line, its height above the ground, and the propeller plane's
/// distance from the nose. [est: the span stations as sourced above; the
/// thrust line hung 0.4 m under the wing's lower skin; the propeller 2.45 m
/// ahead of the leading edge]
pub const ENG: [[f32; 3]; 2] = [[4.98, 3.80, 8.47], [10.30, 4.03, 8.65]];
/// Propeller radius (13 ft 6 in), its spinner's radius and length ahead of
/// the propeller plane [est].
pub const PROP_R: f32 = 2.0574;
pub const SPINNER_R: f32 = 0.40;
pub const SPINNER_L: f32 = 0.95;
/// The nacelle's radius [est]. (Its length is `engines.rs`'s: it closes on
/// the wing's lower skin just ahead of the flaps.)
pub const NAC_R: f32 = 0.66;
/// The T56's exhaust: out of the nacelle's outboard side just under the
/// wing's lower skin, pointing aft (the engine and its tailpipe hang under
/// the wing box; the soot runs back under the wing and the flaps, never over
/// them), offset from the thrust line (out, up) and its distance from the
/// nose, and the pipe's radius. [The place: photographs, e.g. a USAF C-130's
/// tailpipe exit seen from below, opening under the wing's lower skin with
/// the soot round it, https://commons.wikimedia.org/wiki/File:C-130_Maintenance_DVIDS120973.jpg ;
/// the clean upper wing in https://commons.wikimedia.org/wiki/File:C-130_wing_2006-06-05_F-0994L-526.jpg .
/// The offsets and the radius: est, no drawing found.]
pub const EXHAUST_OFF: [f32; 2] = [0.50, 0.42];
pub const EXHAUST_D: [f32; 2] = [12.15, 12.30];
pub const EXHAUST_R: f32 = 0.20;
/// The propeller hub, right side, body frame.
pub const fn prop_hub(e: usize) -> Vec3 {
    Vec3::new(ENG[e][0], by(ENG[e][1]), bz(ENG[e][2]))
}
/// The exhaust's exit, right side, body frame.
pub const fn exhaust(e: usize) -> Vec3 {
    Vec3::new(ENG[e][0] + EXHAUST_OFF[0], by(ENG[e][1] + EXHAUST_OFF[1]), bz(EXHAUST_D[e]))
}
/// The lowest a propeller tip comes (height above the ground at rest).
pub const PROP_TIP_LOW_H: f32 = ENG[0][1] - PROP_R;
/// The external fuel tanks (1,360 US gal, 5.15 m³ each [2+]) under each
/// wing between the engines: span station, height of their axis, nose
/// distance, length, radius. [est: no dimensioned drawing found. The
/// three-view's front view gives about 1.17 m across; its plan view puts the
/// nose 1.5 m ahead of the leading edge; side photographs (e.g. JASDF
/// 85-1079, listed in mod.rs) show a long slender tank, its pointed tail well
/// behind the trailing edge, about 7.3-8.3 m long. 7.6 m by 1.2 m holds
/// about 5.6 m³ with this profile (`engines::tank_r`), the fuel and its
/// structure.]
pub const EXT_TANK: [f32; 5] = [7.64, 3.15, 9.40, 7.60, 0.60];
pub const EXT_TANK_L: f32 = EXT_TANK[3];
pub const EXT_TANK_R: f32 = EXT_TANK[4];
pub const fn ext_tank_nose() -> Vec3 {
    Vec3::new(EXT_TANK[0], by(EXT_TANK[1]), bz(EXT_TANK[2]))
}

// ---------------------------------------------------------------- tail

/// The tailplane: half span (52 ft 8 in overall), leading and trailing edge
/// at the root (centreline) and the tip, its height (no dihedral) [est
/// except the span and chords].
pub const STAB_SPAN: f32 = 8.025;
pub const STAB_LE: [f32; 2] = [29.30 - 4.64312, 29.05 - 1.7247];
pub const STAB_TE: [f32; 2] = [29.30, 29.05];
pub const STAB_H: f32 = 4.00;
/// The elevators: hinge chord fraction, and how far out from the centreline they start.
pub const ELEV_HINGE: f32 = 0.70;
pub const ELEV_ROOT: f32 = 0.75;
pub fn stab_sect(s: f32) -> Sect {
    let u = (s.abs() / STAB_SPAN).min(1.0);
    let le = STAB_LE[0] + (STAB_LE[1] - STAB_LE[0]) * u;
    let te = STAB_TE[0] + (STAB_TE[1] - STAB_TE[0]) * u;
    Sect { le: Vec3::new(s, by(STAB_H), bz(le)), c: te - le, cdir: Vec3::Z, up: Vec3::Y, tf: 1.0 - 0.2 * u }
}
pub fn stab_foil() -> Foil {
    Foil { at: Box::new(stab_sect), t: 0.12, camber: 0.0 }
}
/// The fin: its root on the crown, its height to 11.66 m overall; leading and
/// trailing edges at the root and the tip; the rudder's hinge chord fraction
/// and its span along the fin (from the root) (stations estimated, chords from Table C.6).
pub const FIN_ROOT_H: f32 = 4.10;
pub const FIN_H: f32 = HEIGHT - FIN_ROOT_H;
pub const FIN_LE: [f32; 2] = [28.55 - 6.35, 28.65 - 1.8796];
pub const FIN_TE: [f32; 2] = [28.55, 28.65];
pub const RUDDER_HINGE: f32 = 0.66;
pub const RUDDER_SPAN: [f32; 2] = [0.35, 7.25];
pub fn fin_sect(s: f32) -> Sect {
    let u = (s / FIN_H).clamp(0.0, 1.0);
    let le = FIN_LE[0] + (FIN_LE[1] - FIN_LE[0]) * u;
    let te = FIN_TE[0] + (FIN_TE[1] - FIN_TE[0]) * u;
    Sect { le: Vec3::new(0.0, by(FIN_ROOT_H) + s, bz(le)), c: te - le, cdir: Vec3::Z, up: Vec3::X, tf: 1.0 - 0.2 * u }
}
pub fn fin_foil() -> Foil {
    Foil { at: Box::new(fin_sect), t: 0.13, camber: 0.0 }
}

// ---------------------------------------------------------------- landing gear

/// Tyres: mains 56x20-20, nose 39x13-16 (radius, width, m) [est, see above].
pub const MAIN_TYRE_R: f32 = 0.711;
pub const MAIN_TYRE_W: f32 = 0.508;
pub const NOSE_TYRE_R: f32 = 0.495;
pub const NOSE_TYRE_W: f32 = 0.330;
/// Main track (14 ft 3 in) and wheelbase (nose axle to the middle of the tandem pair).
pub const TRACK: f32 = 4.34;
pub const WHEELBASE: f32 = 9.76;
/// Static compression of the oleos at the reference mass (m): the gear's
/// positions here are FULLY EXTENDED; at rest they ride this much higher.
pub const MAIN_SQUAT: f32 = 0.139;
pub const NOSE_SQUAT: f32 = 0.10;
/// Nose gear: pivot, axle fully extended (3.50 m behind the nose), axle
/// stowed (swung forward into the well under the flight deck floor); the
/// two wheels' half spacing [est].
pub const NOSE_D: f32 = 3.50;
pub const NOSE_PIVOT: Vec3 = Vec3::new(0.0, by(1.68), bz(3.62));
pub const NOSE_AXLE: Vec3 = Vec3::new(0.0, by(NOSE_TYRE_R + NOSE_SQUAT), bz(NOSE_D));
pub const NOSE_STOWED: Vec3 = Vec3::new(0.0, by(1.438), bz(2.556));
pub const NOSE_WHEEL_X: f32 = 0.24;
/// Main gear: on each side two wheels in tandem, 1.55 m apart [est], each on
/// its own strut; the struts rise straight up `MAIN_RISE` into the sponson.
/// Axles fully extended (right side), the struts' tops.
pub const MAIN_D: f32 = NOSE_D + WHEELBASE;
pub const MAIN_TANDEM: f32 = 1.55;
pub const MAIN_AXLE_FWD: Vec3 = Vec3::new(TRACK * 0.5, by(MAIN_TYRE_R + MAIN_SQUAT), bz(MAIN_D - MAIN_TANDEM * 0.5));
pub const MAIN_AXLE_AFT: Vec3 = Vec3::new(TRACK * 0.5, by(MAIN_TYRE_R + MAIN_SQUAT), bz(MAIN_D + MAIN_TANDEM * 0.5));
pub const MAIN_RISE: f32 = 0.80;
pub const MAIN_STRUT_TOP_Y: f32 = by(1.75);
/// Gear wells in plan (body z0, z1, then x0, x1; the nose well straddles the
/// centreline, the main wells are |x|), the height of their roofs (body y),
/// and the height below which the skin is cut for them.
pub const NOSE_WELL: [f32; 4] = [bz(1.98), bz(4.08), -0.45, 0.45];
pub const NOSE_WELL_ROOF: f32 = by(1.96);
pub const NOSE_WELL_CUT: f32 = by(1.30);
/// (The main well reaches in to 1.66 m so the struts, inboard of their
/// wheels on stub axles, pass through it.)
pub const MAIN_WELL: [f32; 4] = [bz(MAIN_D - MAIN_TANDEM * 0.5 - 0.80), bz(MAIN_D + MAIN_TANDEM * 0.5 + 0.80), 1.66, 2.52];
pub const MAIN_WELL_ROOF: f32 = by(2.50);
pub const MAIN_WELL_CUT: f32 = by(1.10);

fn sd_box(p: Vec2, b: [f32; 4]) -> f32 {
    let c = Vec2::new((b[0] + b[1]) * 0.5, (b[2] + b[3]) * 0.5);
    let h = Vec2::new((b[1] - b[0]) * 0.5, (b[3] - b[2]) * 0.5);
    let q = (p - c).abs() - h;
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0)
}
/// Signed distance to the gear wells' openings (negative inside, 5 mm inside
/// their walls), body frame. Keep in step with `well_c130` in plane.wgsl.
pub fn well(p: Vec3) -> f32 {
    let mut d = 1.0f32;
    if p.y < NOSE_WELL_CUT {
        d = d.min(sd_box(Vec2::new(p.z, p.x), NOSE_WELL));
    }
    if p.y < MAIN_WELL_CUT {
        d = d.min(sd_box(Vec2::new(p.z, p.x.abs()), MAIN_WELL));
    }
    d + 0.005
}

// ---------------------------------------------------------------- ramp and doors

/// The cargo ramp: hinged across the floor at FS 737, 127 in long, 123 in
/// wide; closed it forms the underside of the tail from the hinge to
/// `RAMP_TIP_D`. It opens down by `RAMP_OPEN` (rad about +X; positive turns
/// its aft end down) until its toe meets the ground.
pub const RAMP_HINGE: Vec3 = Vec3::new(0.0, by(0.80), bz(D_RAMP));
pub const RAMP_TIP_D: f32 = 22.50;
pub const RAMP_HALF_W: f32 = 1.60;
pub const RAMP_OPEN: f32 = 0.46;
/// The aft cargo door above the ramp: from the ramp's tip (`RAMP_TIP_D`) to
/// its hinge, high in the tail cone at its aft end; it swings up into the
/// tail, as far as its edges clear the tail cone's sides (positive turns its
/// front up) [est].
pub const AFT_DOOR_END_D: f32 = 26.0;
pub const AFT_DOOR_HINGE: Vec3 = Vec3::new(0.0, by(3.00), bz(AFT_DOOR_END_D));
pub const AFT_DOOR_OPEN: f32 = 0.45;
/// The crew entrance door, forward left under the flight deck (d0, d1, h0,
/// h1 on the side view), and the two paratroop doors ahead of the ramp
/// (each side) [est].
pub const CREW_DOOR: [f32; 4] = [5.30, 6.20, 0.95, 2.70];
pub const PARA_DOOR: [f32; 4] = [17.75, 18.75, 1.10, 3.05];

// ---------------------------------------------------------------- lights

/// Anticollision beacons on the crown aft of the wing and under the belly,
/// the white tail light at the tail cone's tip; the navigation lights are at
/// the wing tips (`SPAN`), the landing lights under the outer wing at
/// `LANDING_LIGHT_X` [est].
pub const BEACON_TOP: Vec3 = Vec3::new(0.0, by(FUSE_TOP_H), bz(18.5));
pub const BEACON_BOTTOM: Vec3 = Vec3::new(0.0, by(FUSE_BOT_H), bz(8.0));
pub const TAIL_LIGHT: Vec3 = Vec3::new(0.0, by(3.80), bz(D_TAIL));
pub const LANDING_LIGHT_X: f32 = 12.6;

// ---------------------------------------------------------------- windows

/// A window opening on the flight deck: its outline (corner radius added
/// round it) in one of three projections, and the gate that picks out the
/// part of the skin it is on.
///
/// - `FRONT`: seen from ahead, (|x|, h); only on the skin between the two
///   stations of `gate` (the windshield's d range).
/// - `SIDE`: seen from the side, (d, h); only where |x| > `gate[0]`.
/// - `TOP`: seen from above, (d, |x|); only where h > `gate[0]`.
#[derive(Clone, Copy, Debug)]
pub struct Win {
    pub view: u8,
    pub pts: [[f32; 2]; 4],
    pub gate: [f32; 2],
}
pub const FRONT: u8 = 0;
pub const SIDE: u8 = 1;
#[allow(dead_code)] // (no window is drawn in it at present)
pub const TOP: u8 = 2;
/// The windows' corner radius (m).
pub const WIN_R: f32 = 0.035;
/// The flight deck's windows, each side: the windshield's centre and
/// corner panes, the small pane under the corner pane, two panes of the
/// upper row over the side windows, two side windows beside the pilot, the
/// two lower ("chin") windows by the rudder pedals. [est: read off the
/// three-view's side view (side windows 2.62-3.76 m back, 3.47-3.92 m high;
/// the upper row 2.84-3.76 m, 3.98-4.27 m; the chin windows 2.31-2.89 m,
/// 2.41-3.02 m; the corner pane 2.20-2.73 m, 3.47-4.00 m), checked against
/// the photographs listed in mod.rs (the chin windows 2.47-3.11 m high, the
/// windshield's lower edge about 0.45 m above the radome's top: here 0.35,
/// so that the view over the glareshield meets glass). The real
/// side and upper rows have three panes each: here two.]
pub const WINDOWS: [Win; 9] = [
    Win { view: FRONT, pts: [[0.07, 3.40], [0.68, 3.40], [0.66, 4.00], [0.07, 4.04]], gate: [1.55, 2.70] },
    Win { view: FRONT, pts: [[0.76, 3.40], [1.16, 3.40], [1.13, 3.80], [0.76, 3.96]], gate: [1.55, 2.70] },
    Win { view: FRONT, pts: [[0.66, 3.07], [0.98, 3.07], [0.98, 3.31], [0.70, 3.31]], gate: [1.55, 2.70] },
    Win { view: SIDE, pts: [[2.88, 4.02], [3.24, 4.02], [3.24, 4.25], [2.92, 4.25]], gate: [0.7, 0.0] },
    Win { view: SIDE, pts: [[3.32, 4.02], [3.68, 4.02], [3.68, 4.25], [3.32, 4.25]], gate: [0.7, 0.0] },
    Win { view: SIDE, pts: [[2.72, 3.47], [3.12, 3.47], [3.12, 3.88], [2.76, 3.88]], gate: [1.0, 0.0] },
    Win { view: SIDE, pts: [[3.20, 3.47], [3.68, 3.47], [3.68, 3.88], [3.20, 3.88]], gate: [1.0, 0.0] },
    Win { view: SIDE, pts: [[2.14, 2.50], [2.42, 2.48], [2.42, 3.06], [2.14, 3.08]], gate: [0.5, 0.0] },
    Win { view: SIDE, pts: [[2.50, 2.48], [2.80, 2.46], [2.80, 3.04], [2.50, 3.06]], gate: [0.5, 0.0] },
];
/// Signed distance to the nearest window opening (negative inside), body
/// frame. Keep in step with `window_c130` in plane.wgsl.
pub fn window(p: Vec3) -> f32 {
    let (d, h, ax) = (p.z + ZN, p.y + YG, p.x.abs());
    if !(D_GLASS[0]..=D_GLASS[1]).contains(&d) || h < 1.3 {
        return 1.0;
    }
    let mut best = 1.0f32;
    for w in &WINDOWS {
        let (q, on) = match w.view {
            FRONT => (Vec2::new(ax, h), d >= w.gate[0] && d <= w.gate[1]),
            SIDE => (Vec2::new(d, h), ax > w.gate[0]),
            _ => (Vec2::new(d, ax), h > w.gate[0]),
        };
        if on {
            let poly = w.pts.map(|v| Vec2::new(v[0], v[1]));
            best = best.min(sd_poly(q, &poly) - WIN_R);
        }
    }
    best
}

// ---------------------------------------------------------------- flight deck envelope

/// The flight deck: its floor's height, its forward limit (the
/// windshield's foot, under the glareshield; the rudder pedals hang under
/// the panel aft of it) and its aft bulkhead (FS 245, the cargo
/// compartment's front), as drawing distances. Everything the cockpit
/// builds stays inside: above the floor, between the stations, under
/// `deck_ceiling` and within `deck_half_w`. The exterior keeps its shell
/// (skin, frames, insulation) outside the same surfaces. [est: the floor is
/// put level with the chin windows' sills, which look out beside the
/// rudder pedals, 1.46 m over the cargo floor. Until 2026-11-24 the whole
/// flight deck stood 0.45 m lower, under a roof that was too low.]
pub const DECK_FLOOR_H: f32 = 2.50;
pub const DECK_FLOOR_Y: f32 = by(DECK_FLOOR_H);
pub const DECK_FWD_Z: f32 = bz(1.74);
pub const DECK_AFT_Z: f32 = bz(D_DECK);
/// How far inside the outer skin the deck's lining runs (skin, frames and
/// lining together), m.
pub const LINING: f32 = 0.10;
/// The lining's ceiling (body y) at station z: under the roof, levelling
/// off 2.05 m over the floor (where the roof rounds over to the crown the
/// lining, 10 cm in, would pinch).
pub const DECK_HEADROOM: f32 = 2.05;
pub fn deck_ceiling(z: f32) -> f32 {
    sec(z).top.min(by(DECK_FLOOR_H + DECK_HEADROOM)) - LINING - 0.02
}
/// The lining's half width at station z and height y (0 where the section
/// does not reach that height).
pub fn deck_half_w(z: f32, y: f32) -> f32 {
    half_width_at(z, y).map_or(0.0, |w| (w - LINING).max(0.0))
}
/// Whether a point is inside the deck's envelope.
#[cfg_attr(not(test), allow(dead_code))] // (the flight deck's builders check against it in the tests)
pub fn in_deck(p: Vec3) -> bool {
    p.z >= DECK_FWD_Z && p.z <= DECK_AFT_Z && p.y >= DECK_FLOOR_Y && p.y <= deck_ceiling(p.z) && p.x.abs() <= deck_half_w(p.z, p.y)
}
/// The pilot's eye (left seat; the co-pilot's is its mirror image, 0.62 m
/// right of the centreline): 1.20 m above the floor, 3.55 m behind the nose
/// tip. From it the view over the glareshield lip down past the nose is
/// about 11° [est].
pub const EYE_H: f32 = DECK_FLOOR_H + 1.20;
pub const EYE_D: f32 = 3.55;
pub const EYE: Vec3 = Vec3::new(-0.62, by(EYE_H), bz(EYE_D));
/// The seats' reference points: the middle of the cushion's top, 0.78 m
/// below the eye and 0.10 m behind it.
pub const SEAT_L: Vec3 = Vec3::new(-0.62, by(EYE_H - 0.78), bz(EYE_D + 0.10));
pub const SEAT_R: Vec3 = Vec3::new(0.62, by(EYE_H - 0.78), bz(EYE_D + 0.10));
/// The main instrument panel: its face's station (0.75 m ahead of the
/// eye), its top (just under the glareshield lip) and its bottom (body y).
pub const PANEL_Z: f32 = bz(2.80);
pub const PANEL_TOP_Y: f32 = by(DECK_FLOOR_H + 1.01);
pub const PANEL_BOT_Y: f32 = by(DECK_FLOOR_H + 0.35);
/// The glareshield: its lip over the panel (height, body y) and the
/// station it reaches forward to under the windshield. The lip is on the
/// line from the eye over the nose to the windshield's lower edge.
pub const GLARE_Y: f32 = by(DECK_FLOOR_H + 1.05);
pub const GLARE_Z: f32 = bz(2.15);
/// The centre pedestal between the seats: half width, top (body y), aft
/// end, and the station of the power levers' pivots on it.
pub const PED_HALF_W: f32 = 0.20;
pub const PED_TOP_Y: f32 = by(DECK_FLOOR_H + 0.80);
pub const PED_AFT_Z: f32 = bz(3.75);
pub const PED_LEVER_Z: f32 = bz(3.05);
/// The overhead panel: its forward and aft stations, its lowest face (body y), half width.
pub const OVHD_FWD_Z: f32 = bz(3.40);
pub const OVHD_AFT_Z: f32 = bz(4.40);
pub const OVHD_LOW_Y: f32 = by(DECK_FLOOR_H + 1.65);
pub const OVHD_HALF_W: f32 = 0.40;
/// Where the landing gear lever sits on the main panel: its rest point, on
/// the centre panel's face just right of the centreline, as the flight deck
/// builds it (deck.rs `GEAR_U` for x, this height for its slot) [est, from
/// photographs].
pub const GEAR_HANDLE: Vec3 = Vec3::new(0.072, by(DECK_FLOOR_H + 0.81), PANEL_Z + 0.03);

#[cfg(test)]
mod tests {
    use super::*;

    fn inside_skin(p: Vec3, clear: f32) -> bool {
        let s = sec(p.z);
        p.y < s.top - clear && p.y > s.bot + clear && half_width_at(p.z, p.y).is_some_and(|w| p.x.abs() < w - clear)
    }
    /// Where a ray from inside first leaves the fuselage skin.
    fn exit(o: Vec3, d: Vec3) -> Vec3 {
        let mut t = 0.0;
        while inside_skin(o + d * t, 0.0) && t < 20.0 {
            t += 0.005;
        }
        o + d * t
    }
    fn dir(yaw_deg: f32, pitch_deg: f32) -> Vec3 {
        let (y, p) = (yaw_deg.to_radians(), pitch_deg.to_radians());
        Vec3::new(y.sin() * p.cos(), p.sin(), -y.cos() * p.cos())
    }

    #[test]
    fn headline_dimensions() {
        assert!((bz(D_TAIL) - bz(D_NOSE) - LENGTH).abs() < 1e-4);
        assert!((2.0 * SPAN - B).abs() < 0.01);
        assert!((by(FIN_ROOT_H) + FIN_H - by(HEIGHT)).abs() < 1e-4);
        // Integrate the rectangular centre and trapezoidal outer panels before tip rounding.
        assert_eq!(chord(TAPER_X), chord(0.0));
        assert!((chord(SPAN) - CT).abs() < 1e-5);
        let outer = SPAN - TAPER_X;
        let area = 2.0 * TAPER_X * CR + outer * (CR + CT);
        let mac = 2.0 * (TAPER_X * CR * CR + outer * (CR * CR + CR * CT + CT * CT) / 3.0) / area;
        assert!((area - WING_AREA).abs() / WING_AREA < 0.005, "{area}");
        assert!((mac - MAC).abs() < 0.01, "{mac}");
        assert!((le_d(MAC_X) - LEMAC_D).abs() < 0.01 && (chord(MAC_X) - MAC).abs() < 0.01);
        // Tailplane span 52 ft 8 in.
        assert!((2.0 * STAB_SPAN - 16.05).abs() < 0.01);
        // The fuselage is 4.32 m wide; FS 245 and FS 737 where the cargo floor runs.
        assert!((2.0 * sec(bz(12.0)).w - 4.32).abs() < 0.01);
        // The clean cargo barrel is nearly circular and encloses the full
        // 123-inch load floor, including at the start of the ramp.
        for d in [D_DECK, 12.0, D_RAMP] {
            let z = bz(d);
            let s = sec(z);
            assert!(((s.top - s.bot) / (2.0 * s.w) - 1.0).abs() < 0.05);
            assert!(half_width_at(z, by(CARGO_FLOOR_H)).unwrap() > 1.562);
        }
        assert!((fs(245.0) - bz(CARGO_D[0])).abs() < 0.02 && (fs(737.0) - bz(CARGO_D[1])).abs() < 0.02);
    }

    #[test]
    fn gear_geometry() {
        // Wheelbase, track, and the gear under the airframe.
        assert!(((MAIN_AXLE_FWD.z + MAIN_AXLE_AFT.z) * 0.5 - NOSE_AXLE.z - WHEELBASE).abs() < 1e-4);
        assert!((2.0 * MAIN_AXLE_FWD.x - TRACK).abs() < 1e-4);
        // At rest the tyres sit on the ground.
        assert!((NOSE_AXLE.y - NOSE_SQUAT - NOSE_TYRE_R - by(0.0)).abs() < 1e-4);
        assert!((MAIN_AXLE_FWD.y - MAIN_SQUAT - MAIN_TYRE_R - by(0.0)).abs() < 1e-4);
        // The centre of gravity is ahead of the main gear, with about 12% of the weight on the nose.
        let share = (MAIN_D - ZN) / WHEELBASE;
        assert!(share > 0.08 && share < 0.16, "{share}");
        // The nose leg is rigid: the stowed axle is as far from the pivot.
        assert!(((NOSE_STOWED - NOSE_PIVOT).length() - (NOSE_AXLE - NOSE_PIVOT).length()).abs() < 0.01);
        // Stowed, the nose wheels are under the flight deck floor and above the skin, in their well.
        assert!(NOSE_STOWED.y + NOSE_TYRE_R < DECK_FLOOR_Y - 0.05);
        assert!(NOSE_STOWED.y - NOSE_TYRE_R > skin_bottom(NOSE_STOWED.z, 0.0));
        assert!(NOSE_STOWED.z - NOSE_TYRE_R > NOSE_WELL[0] && NOSE_AXLE.z + NOSE_TYRE_R < NOSE_WELL[1]);
        // Stowed, the main wheels are inside the sponson.
        for a in [MAIN_AXLE_FWD, MAIN_AXLE_AFT] {
            let up = a + Vec3::Y * MAIN_RISE;
            for (dy, dx) in [(-MAIN_TYRE_R, 0.0), (MAIN_TYRE_R, 0.0), (0.0, MAIN_TYRE_W * 0.5), (-MAIN_TYRE_R * 0.7, MAIN_TYRE_W * 0.5), (MAIN_TYRE_R * 0.7, MAIN_TYRE_W * 0.5)] {
                let q = up + Vec3::new(dx, dy, 0.0);
                let bot = spon_bottom_y(q.z, q.x).unwrap();
                let top_t = (0..200).map(|k| spon_pt(q.z, k as f32 / 200.0)).filter(|p| (p.x - q.x).abs() < 0.03 && p.y > q.y).count();
                assert!(q.y > bot && top_t > 0, "wheel at {q:?} pokes out of the sponson");
            }
            assert!(a.z - MAIN_TYRE_R > MAIN_WELL[0] && a.z + MAIN_TYRE_R < MAIN_WELL[1]);
            assert!(MAIN_STRUT_TOP_Y + MAIN_RISE < MAIN_WELL_ROOF + 0.1);
            assert!(well(Vec3::new(a.x, skin_bottom(a.z, a.x) + 0.01, a.z)) < 0.0);
        }
        // The sponsons reach out over the wheels.
        assert!(SPON_XO > TRACK * 0.5 + MAIN_TYRE_W * 0.5 + 0.1);
    }

    #[test]
    fn propellers_clear_the_ground_the_fuselage_and_each_other() {
        // Tip clearance at rest: on the ground and from the fuselage.
        println!("prop tip ground clearance {:.2} m (inboard), {:.2} m (outboard)", PROP_TIP_LOW_H, ENG[1][1] - PROP_R);
        assert!(PROP_TIP_LOW_H > 1.4 && PROP_TIP_LOW_H < 2.2);
        let hub = prop_hub(0);
        let s = sec(hub.z);
        let gap = (0..360).map(|k| {
            let th = k as f32 / 360.0 * PI;
            let p = sec_pt(&s, th);
            Vec2::new(hub.x - p.x, hub.y - p.y).length()
        }).fold(f32::MAX, f32::min) - PROP_R;
        println!("inboard prop tip to fuselage {gap:.2} m");
        assert!(gap > 0.5 && gap < 1.2, "{gap}");
        let between = prop_hub(1).truncate().distance(prop_hub(0).truncate()) - 2.0 * PROP_R;
        println!("between the propellers {between:.2} m");
        assert!(between > 0.8);
        // The propeller planes are ahead of the leading edge; the nacelles meet the wing.
        for e in 0..2 {
            let h = prop_hub(e);
            assert!(h.z + SPINNER_L * 0.0 < bz(le_d(h.x)) - 1.5);
            let lower = wing_foil().pt(h.x, 0.3, -1.0, -1.0).y;
            assert!(h.y + NAC_R > lower && h.y < lower, "nacelle {e} hangs under the wing");
        }
    }

    #[test]
    fn the_tail_fits_the_tail_cone() {
        // The tailplane's root and the fin's root are inside / on the tail cone.
        let z = bz(28.0);
        let s = sec(z);
        assert!(by(STAB_H) < s.top && by(STAB_H) > s.bot);
        assert!((by(FIN_ROOT_H) - sec(bz(24.0)).top).abs() < 0.05);
        assert!(bz(STAB_TE[0]) < bz(D_TAIL) && bz(FIN_TE[0]) < bz(D_TAIL));
        // The fin tip makes the overall height; the ramp hinge is at FS 737.
        assert!((fin_sect(FIN_H).le.y - by(HEIGHT)).abs() < 1e-4);
        assert!((RAMP_HINGE.z - fs(737.0)).abs() < 0.02);
    }

    #[test]
    fn the_nose_matches_the_three_view() {
        // The roof is flush with the crown: no step behind the flight deck.
        let mut prev = sec(bz(0.0)).top;
        let mut d = 0.05;
        while d < D_RAMP {
            let t = sec(bz(d)).top;
            assert!(t >= prev - 1e-4 && t <= by(FUSE_TOP_H) + 1e-4, "the top dips or humps at d {d:.2}");
            prev = t;
            d += 0.05;
        }
        // Read off the three-view's side view (height of the top and the
        // bottom, m) and plan view (width), give or take 6 cm.
        for (d, top, bot, w) in [(0.3f32, 2.66f32, 1.64f32, 1.06f32), (1.0, 2.91, 1.26, 1.57), (2.0, 3.74, 0.93, 2.28), (2.7, 4.37, 0.79, 3.13), (3.6, 4.67, 0.67, 3.78), (5.3, 4.70, 0.58, 4.25)] {
            let s = sec(bz(d));
            assert!((s.top + YG - top).abs() < 0.06 && (s.bot + YG - bot).abs() < 0.06 && (2.0 * s.w - w).abs() < 0.12, "d {d}: top {:.2} bottom {:.2} width {:.2}", s.top + YG, s.bot + YG, 2.0 * s.w);
        }
        // The radome is about as wide as it is high from ahead (2.06 m by 2.1 m on the photograph).
        let (mut lo, mut hi, mut wide) = (f32::MAX, f32::MIN, 0.0f32);
        let mut d = 0.0;
        while d <= D_RADOME {
            let z = bz(d);
            let s = sec(z);
            for k in 0..=200 {
                let p = sec_pt(&s, k as f32 / 200.0 * PI);
                if d <= radome_joint_d(p.y + YG) + 1e-3 {
                    lo = lo.min(p.y + YG);
                    hi = hi.max(p.y + YG);
                    wide = wide.max(2.0 * p.x);
                }
            }
            d += 0.01;
        }
        println!("radome from ahead: {wide:.2} m wide, {:.2} m high ({lo:.2} to {hi:.2})", hi - lo);
        assert!((wide - 2.0).abs() < 0.12 && (hi - lo - 2.0).abs() < 0.15, "{wide} x {}", hi - lo);
        // The joint's two forms agree, and it is on the skin at its top.
        for h in [2.5f32, 2.7, 2.9, 3.0] {
            assert!((radome_joint_h(radome_joint_d(h)) - h).abs() < 1e-3);
        }
        assert!(sec(bz(RADOME_JOINT[1])).top + YG < RADOME_JOINT[2]);
        // The windshield is above the radome, the chin windows above the deck floor.
        assert!(WINDOWS[0].pts[0][1] - RADOME_JOINT[2] > 0.3);
        assert!(WINDOWS[7].pts[1][1] > DECK_FLOOR_H - 0.05);
    }

    #[test]
    fn flight_deck_envelope_is_consistent() {
        // The eye is inside the fuselage with room round the head, and in the envelope.
        assert!(inside_skin(EYE, 0.4), "eye too near the skin");
        assert!(in_deck(EYE) && in_deck(EYE * Vec3::new(-1.0, 1.0, 1.0)));
        assert!(deck_ceiling(EYE.z) - EYE.y > 0.5);
        // The envelope fits inside the skin with the lining's clearance.
        let mut z = DECK_FWD_Z;
        while z <= DECK_AFT_Z {
            let mut y = DECK_FLOOR_Y;
            while y < deck_ceiling(z) {
                let w = deck_half_w(z, y);
                if w > 0.0 {
                    let p = Vec3::new(w, y, z);
                    assert!(half_width_at(z, y).unwrap() - p.x >= LINING - 1e-4);
                }
                y += 0.1;
            }
            assert!(deck_ceiling(z) < sec(z).top - LINING);
            z += 0.1;
        }
        // The floor clears the nose well and the skin underneath.
        assert!(DECK_FLOOR_Y > NOSE_WELL_ROOF + 0.05);
        assert!(DECK_FLOOR_Y > sec(DECK_FWD_Z).bot + 0.5);
        // The furniture stations lie inside it.
        for p in [SEAT_L, SEAT_R, Vec3::new(0.0, PANEL_TOP_Y, PANEL_Z), Vec3::new(0.6, PANEL_BOT_Y, PANEL_Z), Vec3::new(0.0, GLARE_Y, GLARE_Z),
                  Vec3::new(PED_HALF_W, PED_TOP_Y, PED_AFT_Z), Vec3::new(OVHD_HALF_W, OVHD_LOW_Y, OVHD_FWD_Z), Vec3::new(0.0, OVHD_LOW_Y + 0.08, OVHD_AFT_Z), GEAR_HANDLE] {
            assert!(in_deck(p) || (p.z < DECK_FWD_Z && inside_skin(p, LINING)), "{p:?} outside the flight deck");
        }
        assert!(SEAT_L.y > DECK_FLOOR_Y + 0.3 && EYE.y - SEAT_L.y > 0.7);
        assert!(PANEL_Z < EYE.z - 0.5 && PED_LEVER_Z > PANEL_Z);
        assert!(OVHD_LOW_Y > EYE.y + 0.35);
    }

    #[test]
    fn the_pilot_sees_out() {
        // Straight ahead, and ahead-and-up: through the windshield.
        for (yaw, pitch) in [(0.0, 0.0), (0.0, 5.0), (0.0, 10.0), (-20.0, 0.0), (20.0, 0.0), (-40.0, 0.0)] {
            let p = exit(EYE, dir(yaw, pitch));
            assert!(window(p) < -0.01, "yaw {yaw} pitch {pitch}: skin at {p:?} (d {:.2} h {:.2}) is not a window", p.z + ZN, p.y + YG);
        }
        // Out to the left, through the side window; up and out, through the upper row.
        let p = exit(EYE, dir(-90.0, 0.0));
        assert!(window(p) < 0.0, "left: {p:?}");
        let upper = Vec3::new(-1.5, by(4.13), bz(3.06));
        let p = exit(EYE, (upper - EYE).normalize());
        assert!(window(p) < 0.0, "upper row: {p:?} (d {:.2} h {:.2})", p.z + ZN, p.y + YG);
        // Over the glareshield lip: the line from the eye just over it reaches the windshield.
        let lip = Vec3::new(EYE.x, GLARE_Y + 0.01, PANEL_Z);
        let d = (lip - EYE).normalize();
        let p = exit(EYE, d);
        let deg = (-d.y).asin().to_degrees();
        println!("over-the-nose view {deg:.1}° down, meeting the skin at d {:.2} h {:.2}", p.z + ZN, p.y + YG);
        assert!(window(p) < 0.0 && deg > 8.0 && deg < 18.0);
        // Down and behind: no window; nor through the floor.
        assert!(window(exit(EYE, dir(180.0, 0.0))) > 0.0);
        // The window outlines are on the deck section, clear of the radome's joint and the aft bulkhead.
        for w in &WINDOWS {
            for v in w.pts {
                let d = if w.view == FRONT { w.gate[0] } else { v[0] };
                assert!((D_RADOME - 0.3..D_DECK).contains(&d));
            }
        }
    }
    /// plane.wgsl's copies (`window_c130`, `well_c130`, the deck box) are these numbers.
    #[test]
    fn shader_matches() {
        let src = include_str!("../../shaders/plane.wgsl");
        let has = |t: String| assert!(src.contains(&t), "plane.wgsl lacks {t:?}");
        has(format!("const C130_ZN = {ZN:?};"));
        has(format!("const C130_YG = {YG:?};"));
        has(format!("const C130_WIN_R = {WIN_R:?};"));
        for w in &WINDOWS {
            let p = w.pts;
            has(format!("vec4f({:?}, {:?}, {:?}, {:?}), vec4f({:?}, {:?}, {:?}, {:?})", p[0][0], p[0][1], p[1][0], p[1][1], p[2][0], p[2][1], p[3][0], p[3][1]));
            has(format!("vec4f({:?}, {:?}, {:?}, 0.0)", w.view as f32, w.gate[0], w.gate[1]));
        }
        let r = |v: f32| (v * 1000.0).round() / 1000.0;
        has(format!("if (p.y < {:?}) {{ d = min(d, sd_box2(vec2f(p.z, p.x), vec4f({:?}, {:?}, {:?}, {:?}))); }}", r(NOSE_WELL_CUT), r(NOSE_WELL[0]), r(NOSE_WELL[1]), NOSE_WELL[2], NOSE_WELL[3]));
        has(format!("if (p.y < {:?}) {{ d = min(d, sd_box2(vec2f(p.z, abs(p.x)), vec4f({:?}, {:?}, {:?}, {:?}))); }}", r(MAIN_WELL_CUT), r(MAIN_WELL[0]), r(MAIN_WELL[1]), MAIN_WELL[2], MAIN_WELL[3]));
        has(format!("const C130_DECK = vec4f(1.95, {:?}, {:?}, {:?});", r(DECK_FLOOR_Y), r(DECK_FWD_Z), r(DECK_AFT_Z)));
        has(format!("array<vec4f, {}>(", 2 * WINDOWS.len()));
        has(format!("for (var i = 0; i < {}; i++) {{", WINDOWS.len()));
        has(format!("if (d < {:?} || d > {:?} || h < 1.3) {{ return 1.0; }}", D_GLASS[0], D_GLASS[1]));
        // (the deck box's half width and top contain the envelope)
        let mut z = DECK_FWD_Z;
        while z < DECK_AFT_Z {
            assert!(deck_ceiling(z) < 1.9 && (0..20).all(|k| deck_half_w(z, DECK_FLOOR_Y + k as f32 * 0.1) < 1.95 + 0.12));
            z += 0.2;
        }
    }
}

