//! The landing gear: the steerable twin-wheel nose gear retracting forward
//! into its well under the flight deck, with its doors; the four main
//! struts, two in tandem each side, each with its single wheel on a stub
//! axle, raised straight up into the sponson along its tracks by a screw
//! jack; the main gear doors, which ride with the struts.
use super::*;

/// The main struts' line: inboard of the wheels, which hang off stub axles
/// [est, from photographs: the hub caps face outboard, the struts are
/// hidden inboard of the tyres].
pub(super) const MAIN_STRUT_X: f32 = TRACK * 0.5 - (MAIN_TYRE_W * 0.5 + 0.12);
/// The nose gear's doors: forward (closed again once the gear is down) and
/// aft (open while it is down), split at this station.
const NOSE_DOOR_SPLIT: f32 = NOSE_PIVOT.z - 0.62;

/// How far a gear leg turns to stow (rad) and about which axis.
fn stow(pivot: Vec3, axle: Vec3, stowed: Vec3) -> (Vec3, f32) {
    let (a, c) = ((axle - pivot).normalize(), (stowed - pivot).normalize());
    (a.cross(c).normalize(), a.dot(c).clamp(-1.0, 1.0).acos())
}

pub(super) fn gear(b: &mut Builder) {
    nose_gear(b);
    main_gear(b);
}

fn nose_gear(b: &mut Builder) {
    let (axis, ang) = stow(NOSE_PIVOT, NOSE_AXLE, NOSE_STOWED);
    let down = (NOSE_AXLE - NOSE_PIVOT).normalize();
    let len = (NOSE_AXLE - NOSE_PIVOT).length();
    // The leg: the shock strut's cylinder from the trunnion, the drag strut
    // to the well's aft wall, the taxi lights on its front.
    let mut nl = Mesh::default();
    let low = NOSE_PIVOT + down * (len - 0.50);
    nl.tube(&STRUT, &[NOSE_PIVOT - down * 0.06, low], &|t| 0.095 - 0.01 * t, 16, true);
    nl.tube(&STEEL, &[NOSE_PIVOT - Vec3::X * 0.30, NOSE_PIVOT + Vec3::X * 0.30], &|_| 0.065, 12, true);
    for sx in [-1.0f32, 1.0] {
        nl.tube(&STRUT, &[NOSE_PIVOT + Vec3::X * (sx * 0.26), NOSE_PIVOT + down * 0.30 + Vec3::X * (sx * 0.08)], &|_| 0.035, 8, true);
    }
    nl.tube(&STRUT, &[NOSE_PIVOT + down * 0.30, NOSE_PIVOT + Vec3::new(0.0, 0.08, 0.38)], &|_| 0.04, 8, true);
    nl.lathe(&STRUT, low - down * 0.04, down, &[[0.095, -0.03], [0.12, -0.025], [0.12, 0.025], [0.095, 0.03]], 16, false);
    for sx in [-1.0f32, 1.0] {
        let c = NOSE_PIVOT + down * 0.42 + Vec3::new(sx * 0.13, 0.0, -0.10);
        nl.rbox(&STRUT, c, Vec3::new(0.06, 0.06, 0.05), 0.02, Quat::IDENTITY);
        nl.disc(&mt(mat::LAMP, 0.95, 0.92, 0.85).x(lamp::LANDING), c + Vec3::new(0.0, 0.0, -0.052), Vec3::NEG_Z, Vec3::Y, 0.045, 14);
    }
    nl.tube(&HOSE, &[NOSE_PIVOT + down * 0.05 + Vec3::new(0.10, 0.0, 0.08), low + Vec3::new(0.10, 0.0, 0.08)], &|_| 0.01, 6, true);
    let leg = b.add("nose leg", G_GEAR, nl, Anim::hinge(Drive::new(Input::Gear, -ang).smooth(0.15, 0.85).offset(ang)), NOSE_PIVOT, axis, None);
    b.role(leg, Role::Gear(0));
    // The steering collar and its actuator.
    let mut st = Mesh::default();
    st.lathe(&STEEL, low + down * 0.03, down, &[[0.125, 0.0], [0.125, 0.10], [0.10, 0.13]], 18, false);
    st.tube(&STRUT, &[low + down * 0.06 + Vec3::new(0.12, 0.0, 0.0), low + down * 0.06 + Vec3::new(0.18, 0.0, 0.25)], &|_| 0.03, 8, true);
    let steer = b.add("nose steering", G_GEAR, st, Anim::hinge(Drive::new(Input::Steer, 1.0)), NOSE_AXLE, down, Some(leg));
    b.role(steer, Role::Gear(0));
    // The oleo: the chromed piston, the axle, the torque links ahead of it.
    let mut ol = Mesh::default();
    ol.tube(&CHROME, &[low - down * 0.2, NOSE_AXLE - down * 0.08], &|_| 0.065, 14, true);
    ol.rbox(&STRUT, NOSE_AXLE - down * 0.04, Vec3::new(0.09, 0.07, 0.09), 0.03, Quat::IDENTITY);
    ol.tube(&STRUT, &[NOSE_AXLE - Vec3::X * NOSE_WHEEL_X, NOSE_AXLE + Vec3::X * NOSE_WHEEL_X], &|_| 0.045, 10, true);
    let mid = (low + NOSE_AXLE) * 0.5 + Vec3::NEG_Z * 0.16;
    ol.tube(&STRUT, &[low + down * 0.12 + Vec3::NEG_Z * 0.06, mid], &|_| 0.025, 6, true);
    ol.tube(&STRUT, &[mid, NOSE_AXLE - down * 0.10 + Vec3::NEG_Z * 0.07], &|_| 0.025, 6, true);
    ol.tube(&STEEL, &[mid - Vec3::X * 0.04, mid + Vec3::X * 0.04], &|_| 0.018, 8, true);
    // the tow fitting
    ol.tube(&STEEL, &[NOSE_AXLE + Vec3::new(0.0, -0.03, -0.05), NOSE_AXLE + Vec3::new(0.0, -0.06, -0.20)], &|_| 0.022, 8, true);
    let oleo = b.add("nose oleo", G_GEAR, ol, Anim::slide(Drive::new(Input::Compress(0), 1.0), -down), NOSE_AXLE, Vec3::X, Some(steer));
    b.role(oleo, Role::Gear(0));
    let mut nw = Mesh::default();
    for sx in [-1.0f32, 1.0] {
        wheel(&mut nw, NOSE_AXLE + Vec3::X * (sx * NOSE_WHEEL_X), Vec3::X * sx, NOSE_TYRE_R, NOSE_TYRE_W * 0.5, NOSE_TYRE_R * 0.42, false);
    }
    let k = b.add("nose wheels", G_GEAR, nw, Anim::hinge(Drive::new(Input::Wheel(0), -1.0)), NOSE_AXLE, Vec3::X, Some(oleo));
    b.role(k, Role::Gear(0));

    // The doors: two pairs hinged along the well's sides; the forward pair
    // opens for the gear to pass and closes behind it, the aft pair stays
    // open round the strut while it is down [est].
    let [z0, z1, _, hw] = NOSE_WELL;
    for (a, c, anim, names) in [
        (z0, NOSE_DOOR_SPLIT, Drive::new(Input::Gear, 1.45).pulse(0.0, 0.12, 0.88, 1.0), ["left nose door", "right nose door"]),
        (NOSE_DOOR_SPLIT, z1, Drive::new(Input::Gear, 1.40).smooth(0.0, 0.12), ["left aft nose door", "right aft nose door"]),
    ] {
        let mut dr = Mesh::default();
        let at = |u: f32, v: f32, lift: f32| -> SP {
            let z = a + 0.01 + (c - a - 0.02) * u;
            let x = 0.006 + (hw - 0.012) * v;
            (Vec3::new(x, skin_bottom(z, x) + lift, z), [z, x], 0.0)
        };
        dr.patch(&SKIN.pat(pat::FUSE), 12, 6, true, |u, v| at(u, v, -0.004));
        dr.patch(&WELL_MAT, 12, 6, false, |u, v| at(u, v, 0.025));
        for e in 0..4 {
            let rows: Vec<Vec<SP>> = (0..=8).map(|i| { let t = i as f32 / 8.0; let (u, v) = match e { 0 => (t, 0.0), 1 => (t, 1.0), 2 => (0.0, t), _ => (1.0, t) }; vec![at(u, v, -0.004), at(u, v, 0.025)] }).collect();
            dr.grid(&WELL_MAT, &rows, false, false);
        }
        let h0 = Vec3::new(hw, skin_bottom(a, hw), a);
        let h1 = Vec3::new(hw, skin_bottom(c, hw), c);
        b.pair(names, [G_ENGINE, G_ENGINE], dr, move |_| Anim::hinge(anim), h0, h1 - h0, [None, None]);
    }
}

fn main_gear(b: &mut Builder) {
    let names: [[[&str; 2]; 3]; 2] = [
        [["left forward main strut", "right forward main strut"], ["left forward main oleo", "right forward main oleo"], ["left forward main wheel", "right forward main wheel"]],
        [["left aft main strut", "right aft main strut"], ["left aft main oleo", "right aft main oleo"], ["left aft main wheel", "right aft main wheel"]],
    ];
    let rise = Anim::slide(Drive::new(Input::Gear, -MAIN_RISE).offset(MAIN_RISE), Vec3::Y);
    let x = MAIN_STRUT_X;
    let mut strut_ix = [0usize; 2];
    for (i, axle) in [MAIN_AXLE_FWD, MAIN_AXLE_AFT].into_iter().enumerate() {
        // Legs: 1 left forward, 2 right forward, 3 left aft, 4 right aft (flight.rs).
        let legs = [1 + 2 * i as u8, 2 + 2 * i as u8];
        let top = Vec3::new(x, MAIN_STRUT_TOP_Y, axle.z);
        let bottom = Vec3::new(x, axle.y + 0.50, axle.z);
        // The strut's cylinder, the jack screw's nut on top, the shoes that
        // ride the tracks, the brake line.
        let mut s = Mesh::default();
        s.tube(&STRUT, &[top, bottom], &|t| 0.10 - 0.008 * t, 16, true);
        for t in [0.1f32, 0.5, 0.95] {
            s.lathe(&STRUT, top.lerp(bottom, t), Vec3::NEG_Y, &[[0.10, -0.025], [0.125, -0.02], [0.125, 0.02], [0.10, 0.025]], 16, false);
        }
        s.rbox(&STEEL, top - Vec3::Y * 0.04, Vec3::new(0.09, 0.07, 0.09), 0.02, Quat::IDENTITY);
        for dz in [-0.17f32, 0.17] {
            s.rbox(&STRUT, top + Vec3::new(0.0, -0.14, dz * 0.62), Vec3::new(0.05, 0.08, 0.06), 0.015, Quat::IDENTITY);
            s.rbox(&STEEL, top + Vec3::new(0.0, -0.14, dz), Vec3::new(0.05, 0.10, 0.02), 0.008, Quat::IDENTITY);
        }
        s.tube(&HOSE, &[top + Vec3::new(0.08, -0.15, -0.09), bottom + Vec3::new(0.09, 0.05, -0.09)], &|_| 0.011, 6, true);
        let (sl, sr) = b.pair(names[i][0], [G_GEAR, G_GEAR], s, |_| rise, top, Vec3::X, [None, None]);
        b.role(sl, Role::Gear(legs[0])).role(sr, Role::Gear(legs[1]));
        strut_ix[i] = sr;
        // The oleo: the chromed piston, the axle housing and stub axle out
        // to the wheel, the torque links ahead.
        let mut o = Mesh::default();
        o.tube(&CHROME, &[Vec3::new(x, axle.y + 0.62, axle.z), Vec3::new(x, axle.y + 0.04, axle.z)], &|_| 0.072, 14, true);
        o.rbox(&STRUT, Vec3::new(x, axle.y, axle.z), Vec3::new(0.10, 0.11, 0.12), 0.035, Quat::IDENTITY);
        o.tube(&STRUT, &[Vec3::new(x, axle.y, axle.z), Vec3::new(axle.x, axle.y, axle.z)], &|_| 0.065, 12, true);
        let mid = Vec3::new(x, axle.y + 0.30, axle.z - 0.22);
        o.tube(&STRUT, &[Vec3::new(x, axle.y + 0.47, axle.z - 0.08), mid], &|_| 0.03, 6, true);
        o.tube(&STRUT, &[mid, Vec3::new(x, axle.y + 0.10, axle.z - 0.10)], &|_| 0.03, 6, true);
        o.tube(&STEEL, &[mid - Vec3::X * 0.05, mid + Vec3::X * 0.05], &|_| 0.02, 8, true);
        let (ol, or) = b.pair(names[i][1], [G_GEAR, G_GEAR], o, |sd| Anim::slide(Drive::new(Input::Compress(if sd < 0.0 { legs[0] } else { legs[1] }), 1.0), Vec3::Y), axle, Vec3::X, [Some(sl), Some(sr)]);
        b.role(ol, Role::Gear(legs[0])).role(or, Role::Gear(legs[1]));
        let mut wm = Mesh::default();
        wheel(&mut wm, axle, Vec3::X, MAIN_TYRE_R, MAIN_TYRE_W * 0.5, 0.27, true);
        let (wl, wr) = b.pair(names[i][2], [G_GEAR, G_GEAR], wm, |sd| Anim::hinge(Drive::new(Input::Wheel(if sd < 0.0 { legs[0] } else { legs[1] }), -1.0)), axle, Vec3::X, [Some(ol), Some(or)]);
        b.role(wl, Role::Gear(legs[0])).role(wr, Role::Gear(legs[1]));
    }
    let _ = strut_ix;

    // The door: the sponson's lower outboard skin over the well, on links
    // from the struts; up, it closes the sponson's side, down it hangs over
    // the tyres' outboard shoulders [est].
    let [t0, t1] = super::fuselage::DOOR_T;
    let (w0, w1) = (MAIN_WELL[0] + 0.025, MAIN_WELL[1] - 0.025);
    let at = |u: f32, v: f32, inset: f32| -> SP {
        let z = w0 + (w1 - w0) * u;
        let t = t0 + (t1 - t0) * v;
        let p = spon_pt(z, t);
        let e = 1e-3;
        let n = (spon_pt(z, t + e) - spon_pt(z, t - e)).cross(Vec3::Z).normalize_or_zero();
        let n = if n.x < 0.0 { -n } else { n };
        (p - n * inset, [z, t], 0.0)
    };
    let mut d = Mesh::default();
    d.patch(&SKIN.pat(pat::FUSE), 24, 10, false, |u, v| at(u, v, 0.004));
    d.patch(&WELL_MAT, 24, 10, true, |u, v| at(u, v, 0.045));
    for e in 0..4 {
        let rows: Vec<Vec<SP>> = (0..=12).map(|i| { let t = i as f32 / 12.0; let (u, v) = match e { 0 => (t, 0.0), 1 => (t, 1.0), 2 => (0.0, t), _ => (1.0, t) }; vec![at(u, v, 0.004), at(u, v, 0.045)] }).collect();
        d.grid(&WELL_MAT, &rows, false, false);
    }
    // its stiffeners inside and the arms to the struts
    for u in [0.2f32, 0.5, 0.8] {
        let pts: Vec<Vec3> = (0..=8).map(|k| at(u, k as f32 / 8.0, 0.06).0).collect();
        d.tube(&WELL_MAT, &pts, &|_| 0.015, 5, true);
    }
    // its brackets to the struts' carriages, up inside the sponson
    for axle in [MAIN_AXLE_FWD, MAIN_AXLE_AFT] {
        let u = (axle.z - w0) / (w1 - w0);
        let p = at(u, 0.05, 0.05).0;
        d.rbox(&STRUT, p - Vec3::X * 0.04, Vec3::new(0.05, 0.05, 0.08), 0.015, Quat::IDENTITY);
    }
    let _ = x;
    let (l, r) = b.pair(["left main gear door", "right main gear door"], [G_GEAR, G_GEAR], d, |_| Anim::slide(Drive::new(Input::Gear, -super::fuselage::DOOR_DROP).smooth(0.1, 0.9), Vec3::Y), Vec3::new(TRACK * 0.5, by(SPON_BOT_H), MAIN_AXLE_FWD.z), Vec3::X, [None, None]);
    b.role(l, Role::Gear(1)).role(r, Role::Gear(2));
}
