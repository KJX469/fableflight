//! The fuselage: the skin from the radome to the tail cone with the wing-root
//! fairing on the crown, the flight deck's shell and glass, the sponsons and
//! the main gear wells in them, the crew and paratroop doors, the cargo ramp
//! and aft cargo door with the cargo compartment's lining and floor behind
//! them, and what stands on the skin (antennas, probes, lights).
use super::*;
use std::f32::consts::FRAC_PI_2;

// ---------------------------------------------------------------- the skin with the wing-root fairing

/// Where the wing-root fairing joins the fuselage's sides (height) [est].
const H_JOIN: f32 = 3.30;

/// The wing's lower surface on the centreline at chord fraction xc (body y).
fn wing_lower0(xc: f32) -> f32 {
    wing_foil().pt(0.0, xc.clamp(0.0, 1.0), -1.0, -1.0).y
}

/// The wing-root fairing at drawing distance d: how far it has grown (0..1),
/// its top (body y) and its half width. [est, from photographs and the
/// three-views: on the C-130 the wing sits on the crown; a fairing rises
/// from the crown about 2 m ahead of the leading edge to meet it under its
/// nose, swells out a little past the fuselage's sides under the root, and
/// runs down again onto the crown in about 2.5 m behind the trailing edge.]
fn fairing(d: f32) -> (f32, f32, f32) {
    let (dl, dt) = (LE0_D, LE0_D + CR);
    let f = smoothstep(dl - 1.9, dl - 0.15, d) * (1.0 - smoothstep(dt + 0.15, dt + 2.6, d));
    let nose = by(4.46);
    let top = if d <= dl {
        nose
    } else if d >= dt {
        wing_lower0(1.0) + 0.04
    } else {
        let xc = (d - dl) / CR;
        nose + (wing_lower0(xc) + 0.04 - nose) * smoothstep(0.03, 0.15, xc)
    };
    (f, top, 2.24 - 0.06 * smoothstep(dt - 0.5, dt + 0.3, d))
}

/// The fairing's section, t from 0 (on top) to 1 (where it meets the
/// fuselage's side at (xj, yj)): a flat-topped superellipse out to its half
/// width, then down and in to the join.
fn fair_pt(t: f32, top: f32, w: f32, xj: f32, yj: f32) -> Vec2 {
    let yw = top - 0.07;
    if t < 0.5 {
        let (s, c) = (t * PI).sin_cos();
        Vec2::new(w * s.max(0.0).sqrt(), yw + (top - yw) * c.max(0.0).sqrt())
    } else {
        let u = (t - 0.5) * 2.0;
        let (p0, p1, p2) = (Vec2::new(w, yw), Vec2::new(w, yj), Vec2::new(xj, yj));
        p0 * (1.0 - u) * (1.0 - u) + p1 * (2.0 * u * (1.0 - u)) + p2 * (u * u)
    }
}

/// A point on the outside skin at station z, angle th round the section
/// (right side): the fuselage, with the wing-root fairing where it is.
pub(super) fn body_pt(z: f32, th: f32) -> Vec3 {
    let p = fuse_pt(z, th);
    let (f, top, w) = fairing(z + ZN);
    if f <= 0.0 {
        return p;
    }
    let yj = by(H_JOIN);
    let thj = fuse_th(z, yj);
    if th >= thj {
        return p;
    }
    let xj = sec_pt(&sec(z), thj).x;
    let mut q = fair_pt(th / thj, top, w, xj, yj);
    // The wing box broadens the shoulders. Keep the rounded pressure hull
    // wherever it stands above the fairing, including aft of the wing.
    let s = sec(z);
    let arc = (1.0 - (q.x / s.w).powf(s.nt)).max(0.0).powf(1.0 / s.nt);
    q.y = q.y.max(s.mid + (s.top - s.mid) * arc);
    p.lerp(Vec3::new(q.x, q.y, z), f)
}
fn body_fn(z: f32, th: f32) -> Vec3 {
    body_pt(z, th)
}
fn fuse_fn(z: f32, th: f32) -> Vec3 {
    fuse_pt(z, th)
}
/// The crown's height at station z (with the fairing).
pub(super) fn crown(z: f32) -> f32 {
    body_pt(z, 0.0).y
}

/// The angle round the section at z (from the crown) where the radome's
/// joint crosses it: 0 ahead of the joint's top, the joint's foot
/// (`RADOME_JOINT[0]` high) from `D_RADOME` aft.
pub(super) fn joint_th(z: f32) -> f32 {
    let y = by(radome_joint_h(z + ZN));
    if y >= sec(z).top - 1e-4 {
        0.0
    } else {
        fuse_th(z, y)
    }
}

/// Points along the radome's joint on the flight deck section's side of
/// it (3 cm off), about every 7.5 cm, right side from the crown round to
/// the keel, then the left side back: where its fasteners go.
fn radome_ring() -> Vec<(Vec3, Vec3)> {
    let off = 0.03;
    let mut line: Vec<Vec3> = Vec::new();
    // the curved part, from the top of the nose down to the joint's foot
    let n = 60;
    for k in 0..=n {
        let d = RADOME_JOINT[1] + (D_RADOME - RADOME_JOINT[1]) * k as f32 / n as f32;
        let z = bz(d);
        let y = by(radome_joint_h(d)) + off;
        if y < sec(z).top {
            line.push(fuse_pt(z, fuse_th(z, y)));
        }
    }
    // the upright part, round to the keel
    let z = bz(D_RADOME) + off;
    let th0 = fuse_th(z, by(RADOME_JOINT[0]) + off);
    for k in 0..=80 {
        line.push(fuse_pt(z, th0 + (PI - th0) * k as f32 / 80.0));
    }
    // resampled evenly
    let mut out = Vec::new();
    let (mut acc, step) = (0.0, 0.075);
    let mut next = 0.0;
    for w in line.windows(2) {
        let l = (w[1] - w[0]).length();
        while next <= acc + l {
            let q = w[0].lerp(w[1], (next - acc) / l.max(1e-6));
            let th = fuse_th(q.z, q.y);
            out.push((fuse_pt(q.z, th), fuse_n(q.z, th)));
            next += step;
        }
        acc += l;
    }
    let left: Vec<(Vec3, Vec3)> = out.iter().map(|(p, n)| (*p * Vec3::new(-1.0, 1.0, 1.0), *n * Vec3::new(-1.0, 1.0, 1.0))).collect();
    out.extend(left);
    out
}

/// Skin row spacing: fine round the nose and the fairing, coarser along the box.
fn skin_step(d: f32) -> f32 {
    if d < 2.6 {
        0.05
    } else if d < 4.8 {
        0.10
    } else if (8.6..19.0).contains(&d) {
        0.15
    } else if d > 19.3 {
        0.14
    } else {
        0.3
    }
}

/// The angle on the lower half of the section at z where the skin is `x` out.
fn th_lower_at(z: f32, x: f32) -> f32 {
    let s = sec(z);
    let (mut a, mut b) = (FRAC_PI_2, PI);
    for _ in 0..30 {
        let m = (a + b) * 0.5;
        if sec_pt(&s, m).x > x { a = m } else { b = m }
    }
    (a + b) * 0.5
}
/// The cargo ramp's and aft door's side edges: how far out they run (they
/// take the underside of the upswept tail), and the angle there. [est]
fn x_edge(z: f32) -> f32 {
    RAMP_HALF_W.min(0.82 * sec(z).w)
}
fn th_edge(z: f32) -> f32 {
    th_lower_at(z, x_edge(z))
}
/// The ramp's floor height (body y) at drawing distance d, closed: level
/// with the cargo floor at the hinge, rising with the tail's underside.
fn ramp_floor(d: f32) -> f32 {
    let tip = sec(bz(RAMP_TIP_D)).bot + 0.25;
    let y0 = by(CARGO_FLOOR_H);
    y0 + (tip - y0) * ((d - D_RAMP) / (RAMP_TIP_D - D_RAMP)).clamp(0.0, 1.0)
}
/// The cargo floor's half width (the 123 in compartment, less the side rails).
const FLOOR_HALF_W: f32 = 1.52;

/// Walls and a roof round a gear well's outline in plan (z0, z1, |x|0, |x|1),
/// both sides when it is off the centreline.
pub(super) fn well_box(m: &mut Mesh, w: [f32; 4], roof: f32) {
    let mut one = Mesh::default();
    let pts = [(w[0], w[2]), (w[1], w[2]), (w[1], w[3]), (w[0], w[3])];
    for k in 0..4 {
        let (a, c) = (pts[k], pts[(k + 1) % 4]);
        let rows: Vec<Vec<SP>> = (0..=10)
            .map(|i| {
                let t = i as f32 / 10.0;
                let (z, x) = (a.0 + (c.0 - a.0) * t, a.1 + (c.1 - a.1) * t);
                let ys = skin_bottom(z, x) - 0.02;
                vec![(Vec3::new(x, ys, z), [z, ys], 0.0), (Vec3::new(x, roof.max(ys + 0.05), z), [z, roof], 0.0)]
            })
            .collect();
        one.grid(&WELL_MAT, &rows, false, true);
    }
    one.quad(&WELL_MAT, [Vec3::new(w[2], roof, w[0]), Vec3::new(w[2], roof, w[1]), Vec3::new(w[3], roof, w[1]), Vec3::new(w[3], roof, w[0])]);
    // Stringers across the roof.
    let mut z = w[0] + 0.25;
    while z < w[1] - 0.1 {
        one.rbox(&WELL_MAT, Vec3::new((w[2] + w[3]) * 0.5, roof - 0.03, z), Vec3::new((w[3] - w[2]) * 0.5, 0.03, 0.012), 0.004, Quat::IDENTITY);
        z += 0.5;
    }
    if w[2] > 0.0 {
        m.append(&mirrored(&one));
    }
    m.append(&one);
}

// ---------------------------------------------------------------- fuselage

pub(super) fn fuselage(b: &mut Builder) {
    let skin = SKIN.pat(pat::FUSE);
    let zero = |_: f32| 0.0;
    let pi = |_: f32| PI;
    // The radome: composite, no pattern (no rivets or panels); its joint's
    // fasteners on the deck section. Each section from its joint down.
    let radome = section_grid(&stations(D_NOSE, D_RADOME, &skin_step), 96, &fuse_fn, &joint_th, &pi, 0.0, false, &mt(mat::LIVERY, 0.20, 0.22, 0.22));
    b.fixed("radome", G_PROP, radome);

    // The body, FS 245 to the ramp hinge, with the wing-root fairing on its crown.
    let mut body = section_grid(&stations(D_DECK, D_RAMP, &skin_step), 96, &body_fn, &zero, &pi, 0.0, false, &skin);

    // The tail cone: open underneath where the ramp and the aft door close it.
    let mut cone = section_grid(&stations(D_RAMP, AFT_DOOR_END_D, &skin_step), 96, &fuse_fn, &zero, &th_edge, 0.0, false, &skin);
    cone.append(&section_grid(&stations(AFT_DOOR_END_D, D_TAIL, &skin_step), 96, &fuse_fn, &zero, &pi, 0.0, false, &skin));
    // The opening's jambs: the skin's edge turned in.
    let mut jamb = Mesh::default();
    let rows: Vec<Vec<SP>> = stations(D_RAMP, AFT_DOOR_END_D, &|_| 0.15)
        .iter()
        .map(|&d| {
            let z = bz(d);
            let th = th_edge(z);
            let (p, n) = (fuse_pt(z, th), fuse_n(z, th));
            vec![(p, [z, 0.0], 0.0), (p - n * 0.10, [z, 0.1], 0.0)]
        })
        .collect();
    jamb.grid(&WELL_MAT, &rows, false, true);
    cone.append(&mirrored(&jamb));
    cone.append(&jamb);
    // The tail cone's tip: the white tail light.
    cone.lathe(&mt(mat::LAMP, 0.95, 0.95, 0.95).x(lamp::TAIL), TAIL_LIGHT - Vec3::Z * 0.03, Vec3::Z, &[[0.06, 0.0], [0.06, 0.03], [0.035, 0.06], [0.0, 0.07]], 12, true);

    // The cargo compartment inside, seen with the ramp down: lining, floor,
    // the wall under the flight deck. (The flight deck itself is deck.rs's.)
    let lining = mt(mat::SATIN, 0.36, 0.40, 0.35).pat(pat::LINING);
    let d_in = D_DECK + 0.08;
    let mut inside = section_grid(&stations(d_in, D_RAMP, &|_| 0.4), 64, &fuse_fn, &zero, &pi, 0.10, true, &lining);
    inside.append(&section_grid(&stations(D_RAMP, AFT_DOOR_END_D, &|_| 0.3), 64, &fuse_fn, &zero, &th_edge, 0.10, true, &lining));
    inside.append(&section_grid(&stations(AFT_DOOR_END_D, 28.3, &|_| 0.3), 48, &fuse_fn, &zero, &pi, 0.10, true, &lining));
    // its frames, every 20 in or so.
    let mut d = d_in + 0.3;
    while d < 27.8 {
        let z = bz(d);
        let s = sec(z);
        let lim = if d > D_RAMP && d < AFT_DOOR_END_D { th_edge(z) } else { PI };
        let pts: Vec<Vec3> = (0..=24).map(|k| { let th = lim * k as f32 / 24.0; let q = sec_pt(&s, th); Vec3::new(q.x, q.y, z) - fuse_n(z, th) * 0.13 }).collect();
        inside.tube(&mt(mat::SATIN, 0.30, 0.33, 0.29), &pts, &|_| 0.025, 5, true);
        inside.tube(&mt(mat::SATIN, 0.30, 0.33, 0.29), &pts.iter().map(|p| *p * Vec3::new(-1.0, 1.0, 1.0)).collect::<Vec<_>>(), &|_| 0.025, 5, true);
        d += 0.52;
    }
    let floor_m = mt(mat::SATIN, 0.16, 0.17, 0.16).pat(pat::NONSLIP);
    let (z0, z1) = (bz(d_in), bz(D_RAMP - 0.01));
    inside.rbox(&floor_m, Vec3::new(0.0, by(CARGO_FLOOR_H) - 0.03, (z0 + z1) * 0.5), Vec3::new(FLOOR_HALF_W, 0.03, (z1 - z0) * 0.5), 0.005, Quat::IDENTITY);
    // the roller conveyor tracks along the floor
    for x in [-1.05f32, -0.45, 0.45, 1.05] {
        inside.rbox(&mt(mat::ALU, 0.5, 0.5, 0.52), Vec3::new(x, by(CARGO_FLOOR_H) + 0.008, (z0 + z1) * 0.5), Vec3::new(0.05, 0.01, (z1 - z0) * 0.5), 0.004, Quat::IDENTITY);
    }
    // the wall under the flight deck, across the front of the compartment
    let hw = half_width_at(z0, by(1.6)).unwrap_or(2.0) - 0.1;
    inside.quad(&lining, [Vec3::new(-hw, by(CARGO_FLOOR_H), z0), Vec3::new(hw, by(CARGO_FLOOR_H), z0), Vec3::new(hw, DECK_FLOOR_Y, z0), Vec3::new(-hw, DECK_FLOOR_Y, z0)]);
    // the closed end of the lining in the tail
    let zt = bz(28.3);
    let st = sec(zt);
    let ring: Vec<Vec3> = (0..48).map(|k| { let a = k as f32 / 48.0 * std::f32::consts::TAU; let th = if a > PI { std::f32::consts::TAU - a } else { a }; let q = sec_pt(&st, th); Vec3::new(if a > PI { -q.x } else { q.x } * 0.85, st.mid + (q.y - st.mid) * 0.85, zt) }).collect();
    inside.fan(&lining, Vec3::new(0.0, st.mid, zt), &ring, Vec3::NEG_Z);
    let k = b.fixed("cargo compartment", G_FUSE, inside);
    b.pieces[k].occluder = false;

    // Portholes: round windows in the cargo compartment's sides [est: places
    // from photographs], a dark glossy pane in a screwed frame.
    for (d, h) in [(7.75f32, 2.72f32), (8.95, 2.72), (15.35, 2.98), (16.55, 2.98)] {
        for sx in [-1.0f32, 1.0] {
            let (p, n) = on_side(bz(d), by(h), sx, 0.0);
            body.disc(&mt(mat::PAINT, 0.012, 0.016, 0.02), p + n * 0.004, n, Vec3::Y, 0.15, 24);
            body.lathe(&SKIN, p, n, &[[0.19, 0.0], [0.19, 0.004], [0.15, 0.007], [0.15, 0.0]], 24, false);
            for k in 0..12 {
                let a = k as f32 / 12.0 * std::f32::consts::TAU;
                let up = (Vec3::Y - n * n.y).normalize();
                let side = n.cross(up);
                body.screw(&PAINTED_SCREW, p + n * 0.004 + (up * a.cos() + side * a.sin()) * 0.172, n, 0.006, d * 13.0 + k as f32 + sx);
            }
        }
    }

    // Lights: the red anticollision beacons on the crown aft of the wing and under the belly.
    let beacon = mt(mat::LAMP, 0.9, 0.06, 0.03).x(lamp::BEACON);
    let top = Vec3::new(0.0, crown(BEACON_TOP.z), BEACON_TOP.z);
    body.lathe(&SKIN, top, Vec3::Y, &[[0.13, 0.0], [0.13, 0.025], [0.10, 0.035]], 20, false);
    body.lathe(&beacon, top + Vec3::Y * 0.03, Vec3::Y, &[[0.09, 0.0], [0.09, 0.05], [0.06, 0.11], [0.0, 0.13]], 16, true);
    let bot = Vec3::new(0.0, sec(BEACON_BOTTOM.z).bot, BEACON_BOTTOM.z);
    body.lathe(&SKIN, bot, Vec3::NEG_Y, &[[0.13, 0.0], [0.13, 0.025], [0.10, 0.035]], 20, false);
    body.lathe(&beacon, bot - Vec3::Y * 0.03, Vec3::NEG_Y, &[[0.09, 0.0], [0.09, 0.05], [0.06, 0.11], [0.0, 0.13]], 16, true);

    // Antennas [est: types and places from photographs]: UHF/VHF blades and
    // the TACAN on the crown, the IFF, VHF and radar altimeter's under the belly.
    let ant = mt(mat::LIVERY, 0.20, 0.22, 0.22);
    for (d, h, c) in [(7.6f32, 0.30f32, 0.32f32), (19.3, 0.26, 0.30), (20.4, 0.18, 0.22)] {
        let z = bz(d);
        let p = Vec3::new(0.0, crown(z), z);
        blade_antenna(if d > D_RAMP { &mut cone } else { &mut body }, p, Vec3::Y, Vec3::NEG_Z, h, c, &ant);
    }
    for (d, h, c) in [(5.6f32, 0.20f32, 0.24f32), (9.2, 0.16, 0.22), (17.9, 0.24, 0.30)] {
        let z = bz(d);
        let p = Vec3::new(0.0, sec(z).bot, z);
        blade_antenna(&mut body, p, Vec3::NEG_Y, Vec3::NEG_Z, h, c, &ant);
    }
    // radar altimeter aerials: two flush plates under the belly
    for d in [17.0f32, 17.6] {
        let z = bz(d);
        body.rbox(&mt(mat::SATIN, 0.25, 0.26, 0.25), Vec3::new(0.0, sec(z).bot - 0.004, z), Vec3::new(0.12, 0.006, 0.12), 0.004, Quat::IDENTITY);
    }
    // The HF wire aerials from the masts over the flight deck to the fin.
    let tip = fin_sect(5.2);
    for sx in [-1.0f32, 1.0] {
        let z = bz(6.3);
        let base = Vec3::new(sx * 0.35, crown(z) - 0.02, z);
        let top = base + Vec3::new(0.0, 0.32, 0.12);
        cone.tube(&ant, &[base, top], &|t| 0.02 - 0.01 * t, 8, true);
        let end = tip.le + Vec3::new(sx * 0.06, 0.0, tip.c * 0.08);
        cone.tube(&DARK, &[top, top.lerp(end, 0.5) - Vec3::Y * 0.05, end], &|_| 0.0045, 4, false);
        cone.lathe(&mt(mat::PLASTIC, 0.8, 0.8, 0.78), end + (top - end).normalize() * 0.12, (top - end).normalize(), &[[0.015, 0.0], [0.015, 0.08], [0.0, 0.09]], 8, true);
    }

    // The main gear wells' walls, in the body under the sponsons.
    // (The wells themselves are in `sponsons`.)

    // The paratroop doors, aft of the sponsons each side [est: they open in
    // and slide up on tracks inside; nothing drives them yet].
    for sx in [-1.0f32, 1.0] {
        let mut door = Mesh::default();
        let [d0, d1, h0, h1] = PARA_DOOR;
        door_panel(&mut door, d0, d1, h0, h1, sx, &skin);
        // its window and handle
        let (p, n) = on_side(bz(d0 + 0.5), by(h1 - 0.55), sx, 0.012);
        let up = (Vec3::Y - n * n.y).normalize();
        door.slab(&mt(mat::PAINT, 0.012, 0.016, 0.02), &[Vec2::new(-0.17, -0.13), Vec2::new(0.17, -0.13), Vec2::new(0.17, 0.13), Vec2::new(-0.17, 0.13)], &[], p, Vec3::Z * sx.signum(), up, 0.002);
        let (p, n) = on_side(bz(d1 - 0.18), by(h0 + 0.95), sx, 0.012);
        door.rbox(&mt(mat::STEEL, 0.5, 0.5, 0.52), p + n * 0.015, Vec3::new(0.02, 0.10, 0.02), 0.01, Quat::IDENTITY);
        let (pivot, _) = on_side(bz((d0 + d1) * 0.5), by(h1), sx, 0.0);
        let name = if sx < 0.0 { "left paratroop door" } else { "right paratroop door" };
        b.add(name, G_FUSE, door, Anim::slide(Drive::new(Input::Canopy, 0.0), Vec3::new(-sx * 0.35, 1.0, 0.0).normalize()), pivot, Vec3::Z, None);
    }

    b.fixed("fuselage", G_FUSE, body);
    let t = b.fixed("tail cone", G_TAIL, cone);
    b.role(t, Role::TailCone);
    ramp_and_door(b);
}

/// A door's outer panel on the fuselage side between d0..d1, h0..h1 (side sx),
/// standing just proud of the skin with its edges.
fn door_panel(m: &mut Mesh, d0: f32, d1: f32, h0: f32, h1: f32, sx: f32, mat: &Mat) {
    let at = |a: f32, c: f32, off: f32| -> SP {
        let z = bz(d0 + (d1 - d0) * a);
        let y = by(h0 + (h1 - h0) * c);
        let (p, _) = on_side(z, y, sx, off);
        let th = fuse_th(z, y);
        (p, [z, sx * th * 2.0], 0.0)
    };
    let mut door = Mesh::default();
    door.patch(mat, 10, 16, sx > 0.0, |a, c| at(a, c, 0.006));
    door.patch(&WELL_MAT, 10, 16, sx < 0.0, |a, c| at(a, c, -0.03));
    for e in 0..4 {
        let rows: Vec<Vec<SP>> = (0..=12)
            .map(|i| {
                let t = i as f32 / 12.0;
                let (a, c) = match e { 0 => (t, 0.0), 1 => (1.0, t), 2 => (1.0 - t, 1.0), _ => (0.0, 1.0 - t) };
                vec![at(a, c, 0.006), at(a, c, -0.03)]
            })
            .collect();
        door.grid(&WELL_MAT, &rows, false, sx < 0.0);
    }
    m.append(&door);
}

/// The cargo ramp and the aft cargo door: closed, they are the underside
/// of the upswept tail between the opening's edges. The ramp hinges down
/// about the floor's aft edge (FS 737) to the ground; the door swings up
/// into the tail about its hinge at the opening's aft end.
fn ramp_and_door(b: &mut Builder) {
    let skin = SKIN.pat(pat::FUSE);
    let pi = |_: f32| PI;
    let edge = |z: f32| th_edge(z) + 0.002;
    // The ramp.
    let (a, c) = (D_RAMP + 0.012, RAMP_TIP_D - 0.006);
    let mut ramp = section_grid(&stations(a, c, &|_| 0.12), 48, &fuse_fn, &edge, &pi, 0.0, false, &skin);
    let floor_m = mt(mat::SATIN, 0.16, 0.17, 0.16).pat(pat::NONSLIP);
    let xf = |z: f32| (x_edge(z) - 0.03).min(FLOOR_HALF_W);
    ramp.patch(&floor_m, 16, 8, false, |u, v| {
        let d = a + (c - a) * u;
        let z = bz(d);
        let x = xf(z) * (2.0 * v - 1.0);
        (Vec3::new(x, ramp_floor(d), z), [z, x], 0.0)
    });
    // its sides, its ends
    let mut side = Mesh::default();
    let rows: Vec<Vec<SP>> = stations(a, c, &|_| 0.12)
        .iter()
        .map(|&d| {
            let z = bz(d);
            let p = fuse_pt(z, edge(z));
            vec![(p, [z, 0.0], 0.0), (Vec3::new(xf(z), ramp_floor(d), z), [z, 0.3], 0.0)]
        })
        .collect();
    side.grid(&WELL_MAT, &rows, false, false);
    ramp.append(&mirrored(&side));
    ramp.append(&side);
    for (d, n) in [(a, Vec3::NEG_Z), (c, Vec3::Z)] {
        let z = bz(d);
        let th0 = edge(z);
        let mut ring: Vec<Vec3> = (0..=16).map(|k| { let th = th0 + (PI - th0) * k as f32 / 16.0; fuse_pt(z, th) }).collect();
        let back: Vec<Vec3> = ring.iter().rev().skip(1).map(|p| *p * Vec3::new(-1.0, 1.0, 1.0)).collect();
        ring.extend(back);
        ring.push(Vec3::new(-xf(z), ramp_floor(d), z));
        ring.push(Vec3::new(xf(z), ramp_floor(d), z));
        let ctr = ring.iter().copied().sum::<Vec3>() / ring.len() as f32;
        if n.z > 0.0 {
            ring.reverse();
        }
        ramp.fan(&WELL_MAT, ctr, &ring, n);
    }
    // the hinge lugs, and the ramp's toe plates folded on its tip
    for sx in [-1.0f32, 1.0] {
        ramp.rbox(&mt(mat::STEEL, 0.45, 0.45, 0.47), RAMP_HINGE + Vec3::new(sx * 1.25, 0.0, 0.0), Vec3::new(0.05, 0.08, 0.10), 0.02, Quat::IDENTITY);
        for k in 0..2 {
            let x = sx * (0.35 + 0.65 * k as f32);
            let z = bz(c) - 0.35;
            ramp.rbox(&mt(mat::ALU, 0.45, 0.46, 0.47), Vec3::new(x, ramp_floor(c - 0.35) + 0.012, z), Vec3::new(0.28, 0.012, 0.33), 0.006, Quat::from_rotation_x(-((ramp_floor(c) - ramp_floor(a)) / (c - a)).atan()));
        }
    }
    let k = b.add("cargo ramp", G_TAIL, ramp, Anim::hinge(Drive::new(Input::Canopy, RAMP_OPEN)), RAMP_HINGE, Vec3::X, None);
    b.role(k, Role::Other);

    // The aft door: skin outside, primer inside, its edges.
    let (a, c) = (RAMP_TIP_D + 0.006, AFT_DOOR_END_D - 0.012);
    let mut door = section_grid(&stations(a, c, &|_| 0.12), 48, &fuse_fn, &edge, &pi, 0.0, false, &skin);
    door.append(&section_grid(&stations(a, c, &|_| 0.2), 48, &fuse_fn, &edge, &pi, 0.07, true, &WELL_MAT));
    let mut side = Mesh::default();
    let rows: Vec<Vec<SP>> = stations(a, c, &|_| 0.12)
        .iter()
        .map(|&d| {
            let z = bz(d);
            let th = edge(z);
            let p = fuse_pt(z, th);
            vec![(p, [z, 0.0], 0.0), (p - fuse_n(z, th) * 0.07, [z, 0.07], 0.0)]
        })
        .collect();
    side.grid(&WELL_MAT, &rows, false, true);
    door.append(&mirrored(&side));
    door.append(&side);
    for (d, flip) in [(a, true), (c, false)] {
        let z = bz(d);
        let th0 = edge(z);
        let rows: Vec<Vec<SP>> = (0..=24)
            .map(|k| {
                let th = th0 + (PI - th0) * k as f32 / 24.0;
                let (p, n) = (fuse_pt(z, th), fuse_n(z, th));
                vec![(p, [z, th], 0.0), (p - n * 0.07, [z, th], 0.0)]
            })
            .collect();
        let mut e = Mesh::default();
        e.grid(&WELL_MAT, &rows, false, flip);
        door.append(&both_sides(e));
    }
    b.add("aft cargo door", G_TAIL, door, Anim::hinge(Drive::new(Input::Canopy, AFT_DOOR_OPEN)), AFT_DOOR_HINGE, Vec3::X, None);
}

// ---------------------------------------------------------------- sponsons

/// The main gear's door: the sponson's lower outboard skin over the well,
/// between these fractions of its section (`spon_pt`). It rides down on
/// links from the struts and hangs over the tyres' outboard shoulders with
/// the gear down. [est,
/// from photographs]
pub(super) const DOOR_T: [f32; 2] = [0.50, 0.68];
/// How far the door drops with the gear down: it hangs over the upper
/// outboard part of the tyres [est, from photographs].
pub(super) const DOOR_DROP: f32 = 0.45;

/// The main gear fairings on the lower fuselage sides, from `dims::spon_pt`,
/// with the main gear wells, their tracks and screw jacks; the door strip
/// over the well is the gear door's (gear.rs).
pub(super) fn sponsons(b: &mut Builder) {
    let skin = SKIN.pat(pat::FUSE);
    let mut r = Mesh::default();
    let ds = stations(SPON_D[0], SPON_D[1], &|d| if d < SPON_D[0] + 1.2 || d > SPON_D[1] - 1.2 { 0.06 } else { 0.2 });
    let (w0, w1) = (MAIN_WELL[0] + 0.02, MAIN_WELL[1] - 0.02);
    let grid = |m: &mut Mesh, ds: &[f32], t0: f32, t1: f32, nt: usize| {
        let rows: Vec<Vec<SP>> = ds
            .iter()
            .map(|&d| {
                let z = bz(d);
                (0..=nt).map(|k| { let t = t0 + (t1 - t0) * k as f32 / nt as f32; (spon_pt(z, t), [z, t], 0.0) }).collect()
            })
            .collect();
        m.grid(&skin, &rows, false, false);
    };
    grid(&mut r, &ds, 0.0, DOOR_T[0], 18);
    grid(&mut r, &ds, DOOR_T[1], 1.0, 14);
    let fore: Vec<f32> = ds.iter().copied().filter(|&d| bz(d) <= w0).chain(std::iter::once(w0 + ZN)).collect();
    let aft: Vec<f32> = std::iter::once(w1 + ZN).chain(ds.iter().copied().filter(|&d| bz(d) >= w1)).collect();
    grid(&mut r, &fore, DOOR_T[0], DOOR_T[1], 10);
    grid(&mut r, &aft, DOOR_T[0], DOOR_T[1], 10);
    // The door's opening: the sponson's edge turned in round it.
    let mut lip = Mesh::default();
    for (z0, z1, fixed_t) in [(w0, w1, Some(DOOR_T[0])), (w0, w1, Some(DOOR_T[1])), (w0, w0, None), (w1, w1, None)] {
        let rows: Vec<Vec<SP>> = (0..=10)
            .map(|i| {
                let u = i as f32 / 10.0;
                let (z, t) = match fixed_t { Some(t) => (z0 + (z1 - z0) * u, t), None => (z0, DOOR_T[0] + (DOOR_T[1] - DOOR_T[0]) * u) };
                let p = spon_pt(z, t);
                let inn = Vec3::new((p.x - 0.06).max(SPON_XI), p.y, z);
                vec![(p, [z, t], 0.0), (inn, [z, t], 0.0)]
            })
            .collect();
        lip.grid(&WELL_MAT, &rows, false, false);
        lip.grid(&WELL_MAT, &rows, false, true);
    }
    r.append(&lip);
    // The well's outboard skirt: a fixed panel just inside the door's
    // opening, down to below the door's top edge with the gear down, so the
    // dropped door hangs under unbroken skin (photographs show no open slot
    // over it, e.g. https://commons.wikimedia.org/wiki/File:JASDF_KC-130H(85-1080)_right_main_landing_gear_right_front_view_at_Komaki_Air_Base_March_2,_2019.jpg ).
    // With the door shut it lies hidden inside the sponson. [est]
    let mut skirt = Mesh::default();
    let rows: Vec<Vec<SP>> = stations(w0 + ZN, w1 + ZN, &|_| 0.2)
        .iter()
        .map(|&d| {
            let z = bz(d);
            let top = spon_pt(z, DOOR_T[0]);
            let x = top.x - 0.10;
            let y1 = top.y + 0.03;
            let y0 = top.y - DOOR_DROP - 0.06;
            vec![(Vec3::new(x, y1, z), [z, DOOR_T[0]], 0.0), (Vec3::new(x, y0, z), [z, DOOR_T[0] + 0.08], 0.0)]
        })
        .collect();
    skirt.grid(&skin, &rows, false, false);
    skirt.grid(&skin, &rows, false, true);
    r.append(&skirt);
    // The wells, and in each the vertical tracks the struts ride and the
    // screw jack that drives them [est: C-130 main gear: each strut is
    // raised straight up by a jack screw along its tracks].
    super::fuselage::well_box(&mut r, MAIN_WELL, MAIN_WELL_ROOF);
    let track = mt(mat::STEEL, 0.50, 0.51, 0.53);
    for axle in [MAIN_AXLE_FWD, MAIN_AXLE_AFT] {
        let x = super::gear::MAIN_STRUT_X;
        for dz in [-0.17f32, 0.17] {
            r.rbox(&track, Vec3::new(x, (MAIN_WELL_ROOF + by(1.0)) * 0.5, axle.z + dz), Vec3::new(0.035, (MAIN_WELL_ROOF - by(1.0)) * 0.5, 0.025), 0.006, Quat::IDENTITY);
        }
        r.tube(&mt(mat::STEEL, 0.62, 0.62, 0.64).x(1.0), &[Vec3::new(x, MAIN_WELL_ROOF, axle.z), Vec3::new(x, MAIN_STRUT_TOP_Y + 0.05, axle.z)], &|_| 0.032, 10, false);
        r.rbox(&WELL_MAT, Vec3::new(x, MAIN_WELL_ROOF - 0.06, axle.z), Vec3::new(0.10, 0.06, 0.10), 0.01, Quat::IDENTITY);
        // the gearbox's torque shaft along the well's roof
        r.tube(&track, &[Vec3::new(x + 0.12, MAIN_WELL_ROOF - 0.05, MAIN_WELL[0] + 0.1), Vec3::new(x + 0.12, MAIN_WELL_ROOF - 0.05, MAIN_WELL[1] - 0.1)], &|_| 0.022, 8, true);
        r.tube(&HOSE, &[Vec3::new(x - 0.08, MAIN_WELL_ROOF - 0.04, axle.z - 0.3), Vec3::new(x - 0.10, by(1.6), axle.z - 0.12), Vec3::new(x - 0.10, by(1.15), axle.z - 0.1)], &|_| 0.012, 6, true);
    }
    // The exhaust on each sponson's nose (left: the gas turbine compressor's,
    // right: the cargo air conditioning's) [est from photographs]: an oval
    // lip pointing forward and out.
    let z = bz(SPON_D[0] + 0.55);
    let p = spon_pt(z, 0.40);
    let n = Vec3::new(0.55, 0.0, -0.84).normalize();
    let mut ex = Mesh::default();
    ex.lathe(&EXHAUST_MAT, Vec3::ZERO, Vec3::Z, &[[0.10, -0.06], [0.10, 0.02], [0.085, 0.03], [0.07, 0.0], [0.07, -0.12]], 20, false);
    ex.disc(&DARK, Vec3::Z * -0.08, Vec3::Z, Vec3::Y, 0.07, 16);
    for v in &mut ex.v {
        let q = Vec3::from(v.pos);
        v.pos = (Vec3::new(q.x * 1.6, q.y, q.z)).into();
    }
    ex.transform(0, glam::Mat4::from_rotation_translation(Quat::from_rotation_arc(Vec3::Z, n), p - n * 0.01));
    r.append(&ex);
    // the gas turbine compressor's air intake grille on top of the left
    // sponson [est: place from photographs]
    let mut g = Mesh::default();
    let z = bz(11.4);
    let p = spon_pt(z, 0.12);
    g.rbox(&DARK, p + Vec3::Y * 0.002, Vec3::new(0.16, 0.012, 0.30), 0.01, Quat::from_rotation_z(-0.45));
    for k in 0..7 {
        g.rbox(&mt(mat::SATIN, 0.25, 0.26, 0.25), p + Vec3::new(0.0, 0.012, -0.24 + k as f32 * 0.08), Vec3::new(0.15, 0.006, 0.012), 0.003, Quat::from_rotation_z(-0.45));
    }
    let mut m = both_sides(r);
    m.append(&mirrored(&g));
    b.fixed("sponsons", G_FUSE, m);
}

// ---------------------------------------------------------------- flight deck shell

/// The nose section round the flight deck (its skin has the windows cut out
/// by the shader, `dims::window`), the glass behind the openings, the nose
/// gear well, the probes, the wipers and the crew entrance door.
pub(super) fn flight_deck(b: &mut Builder) {
    let zero = |_: f32| 0.0;
    let pi = |_: f32| PI;
    let mut deck = section_grid(&stations(D_RADOME, D_DECK, &skin_step), 96, &fuse_fn, &zero, &pi, 0.0, false, &SKIN.pat(pat::FUSE));
    // Over the radome's curved joint: each section from the crown down to it.
    deck.append(&section_grid(&stations(RADOME_JOINT[1], D_RADOME, &skin_step), 96, &fuse_fn, &zero, &joint_th, 0.0, false, &SKIN.pat(pat::FUSE)));
    well_box(&mut deck, NOSE_WELL, NOSE_WELL_ROOF);
    // Fasteners along the radome's joint.
    let ring = radome_ring();
    screw_row(&mut deck, &ring, 0.008, 3.0);
    // Pitot-static probes, two each side under the chin windows [est: places from the side photograph of 05-1085].
    for sx in [-1.0f32, 1.0] {
        for (d, h) in [(2.78f32, 2.26f32), (2.80, 1.92)] {
            let (p, n) = on_side(bz(d), by(h), sx, 0.0);
            deck.rbox(&ALU, p + n * 0.004, Vec3::new(0.035, 0.05, 0.06).max(Vec3::splat(0.004)), 0.01, Quat::from_rotation_arc(Vec3::X, n));
            deck.tube(&ALU, &[p - n * 0.01, p + n * 0.11, p + n * 0.13 - Vec3::Z * 0.20], &|t| 0.013 - 0.004 * t, 8, true);
        }
        // the angle of attack vane and the total temperature probe
        let (p, n) = on_side(bz(3.2), by(1.75), sx, 0.0);
        deck.slab(&DARK, &[Vec2::new(0.0, 0.0), Vec2::new(0.09, 0.0), Vec2::new(0.06, 0.07), Vec2::new(0.0, 0.07)], &[], p, Vec3::Z, n, 0.008);
    }
    // Windshield wipers on the two centre panes, parked along their lower frame.
    for sx in [-1.0f32, 1.0] {
        let x = sx * 0.40;
        // the skin point at that x where the pane's lower frame is
        let at_h = |h: f32| -> Vec3 {
            let mut best = Vec3::ZERO;
            let mut d = D_GLASS[0];
            while d < 3.0 {
                let z = bz(d);
                let s = sec(z);
                if let Some(t) = (0..400).map(|k| k as f32 / 400.0 * FRAC_PI_2).find(|&t| sec_pt(&s, t).x >= x.abs()) {
                    let q = sec_pt(&s, t);
                    if q.y >= by(h) {
                        best = Vec3::new(x, q.y, z);
                        break;
                    }
                }
                d += 0.005;
            }
            best
        };
        let foot = WINDOWS[0].pts[0][1];
        let p0 = at_h(foot - 0.06);
        let p1 = at_h(foot - 0.01);
        let along = Vec3::new(-sx * 0.55, 0.0, 0.0);
        deck.tube(&DARK, &[p0 + Vec3::Y * 0.02, p1 + along + Vec3::Y * 0.02], &|_| 0.008, 6, true);
        deck.rbox(&DARK, p0 + Vec3::Y * 0.02, Vec3::new(0.025, 0.02, 0.025), 0.008, Quat::IDENTITY);
    }
    b.fixed("flight deck section", G_ENGINE, deck);

    // One sheet of glass just inside the skin over the whole glazed nose:
    // the shader draws it only where `window` is negative.
    let g = section_grid(&stations(D_GLASS[0].max(D_RADOME), D_GLASS[1], &|_| 0.04), 96, &fuse_fn, &zero, &pi, 0.012, false, &GLASS_MAT);
    let k = b.fixed("windows", G_ENGINE, g);
    b.pieces[k].glass = true;
    b.pieces[k].occluder = false;

    // The crew entrance door, left side under the flight deck: an airstair
    // door hinged along its sill, opening out and down [est; nothing drives
    // it yet].
    let [d0, d1, h0, h1] = CREW_DOOR;
    let mut door = Mesh::default();
    door_panel(&mut door, d0, d1, h0, h1, -1.0, &SKIN.pat(pat::FUSE));
    let (p, n) = on_side(bz(d0 + 0.45), by(h1 - 0.45), -1.0, 0.012);
    let up = (Vec3::Y - n * n.y).normalize();
    door.disc(&mt(mat::PAINT, 0.012, 0.016, 0.02), p, n, up, 0.11, 20);
    let (p, n) = on_side(bz(d1 - 0.16), by(h0 + 0.9), -1.0, 0.012);
    door.rbox(&mt(mat::STEEL, 0.5, 0.5, 0.52), p + n * 0.012, Vec3::new(0.02, 0.025, 0.09), 0.01, Quat::IDENTITY);
    let (pivot, _) = on_side(bz((d0 + d1) * 0.5), by(h0), -1.0, 0.0);
    b.add("crew door", G_ENGINE, door, Anim::hinge(Drive::new(Input::Canopy, 0.0)), pivot, Vec3::Z, None);
}
