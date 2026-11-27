//! The wing: the centre section over the fuselage, the outer wings with
//! their leading-edge sections, the Lockheed-Fowler flaps on their tracks,
//! the ailerons and their tabs, lights, static dischargers, and the spars
//! and ribs a torn wing shows.
use super::*;

/// The wing as drawn: `dims::wing_sect`, its tip rounded off in plan and
/// thickness over the last 0.45 m (the C-130's tips are rounded caps).
pub(super) fn render_wing() -> Foil {
    Foil {
        at: Box::new(|s: f32| {
            let mut k = wing_sect(s);
            let r = 0.45;
            if s > SPAN - r {
                let u = ((s - (SPAN - r)) / r).min(1.0);
                let q = 1.0 - (1.0 - u * u).max(0.0).sqrt();
                k.le += k.cdir * (k.c * 0.28 * q);
                k.c -= k.c * 0.50 * q;
                k.tf *= (1.0 - u * u).max(0.03).sqrt();
            }
            k
        }),
        t: 0.18,
        camber: 0.02,
    }
}

/// Where the outer wings' skins start (span station): inboard of it the
/// centre section, which stays with the fuselage when a wing breaks.
const X0: f32 = WING_ROOT_X - 0.3;
/// The leading-edge sections: the hot-air anti-iced leading edge is a row
/// of removable sections (chord fraction they reach back to; their span
/// joints) [est].
const LE_XC: f32 = 0.12;
const LE_JOINTS: [f32; 9] = [X0, 3.9, 6.1, 8.9, 11.3, 13.6, 15.7, 17.8, SPAN - 0.3];
/// The flap tracks (span stations) [est: three under each flap, one of them
/// in line with the nacelle]. They are inside the wing: the C-130 has no
/// track fairings under it (its underside behind the nacelles is clean in
/// photographs, e.g. https://commons.wikimedia.org/wiki/File:Lockheed_C-130H_Hercules_%E2%80%9885-1079_079%E2%80%99_(33890468248).jpg ),
/// and they show only in the slot that opens between the wing and the flap.
const TRACKS: [[f32; 3]; 2] = [[3.0, 4.98, 6.95], [8.15, 10.30, 12.75]];
/// The aileron's tab (span stations, chord fraction of its hinge) [est].
const AIL_TAB: [f32; 2] = [14.35, 16.55];
const TAB_XC: f32 = 0.915;

/// A flap's hinge (right side, between span stations s0 and s1): a point on
/// it and its axis, the direction its nose slides along the tracks (aft and
/// a little down) and how far.
fn flap_hinge(f: &Foil, s0: f32, s1: f32) -> (Vec3, Vec3, Vec3, f32) {
    let sm = (s0 + s1) * 0.5;
    let k = wing_sect(sm);
    let axis = (f.pt(s1, XF_FLAP + 0.03, 0.0, -1.0) - f.pt(s0, XF_FLAP + 0.03, 0.0, -1.0)).normalize();
    let pivot = f.pt(sm, XF_FLAP + 0.03, 0.0, -1.0);
    let dir = (k.cdir - k.up * 0.22).normalize();
    (pivot, axis, dir, 0.65 * (1.0 - XF_FLAP) * k.c)
}
/// A flap track's beam: half width and half depth (m) [est].
const TRACK_HALF: [f32; 2] = [0.022, 0.040];
/// The ends of the track beam at span station t (flaps up): from inside the
/// wing, the flap's full travel plus a little ahead of the hinge, back to
/// the hinge, on the line the hinge slides along.
fn track_ends(f: &Foil, s0: f32, s1: f32, t: f32) -> (Vec3, Vec3) {
    let (pivot, axis, dir, travel) = flap_hinge(f, s0, s1);
    let hinge = pivot + axis * ((t - pivot.x) / axis.x);
    (hinge - dir * (travel + 0.03 * wing_sect(t).c), hinge)
}

pub(super) fn wings(b: &mut Builder) {
    let f = render_wing();
    let skin = SKIN.pat(pat::WING);
    let ctrl = SKIN.pat(pat::CTRL);
    let one = |_: f32| 1.0;
    let zero = |_: f32| 0.0;
    let le = |_: f32| LE_XC;

    // The centre section, over the fuselage between the two outer wings.
    let mut cs = Mesh::default();
    f.skin(&mut cs, &skin, 0.0, X0, 6, &zero, &le, 16, false);
    f.skin(&mut cs, &skin, 0.0, X0, 6, &le, &one, 30, false);
    let cs = { let l = mirrored(&cs); let mut m = cs; m.append(&l); m };
    b.fixed("wing centre section", G_FUSE, cs);

    let mut w = Mesh::default();
    // Skin, leaving the trailing edge open where the flaps and the aileron are.
    let spans = [(X0, FLAP_IB[0], 1.0), (FLAP_IB[0], FLAP_IB[1], XF_FLAP), (FLAP_IB[1], FLAP_OB[0], 1.0), (FLAP_OB[0], FLAP_OB[1], XF_FLAP), (FLAP_OB[1], AILERON[0], 1.0), (AILERON[0], AILERON[1], XF_AIL), (AILERON[1], SPAN, 1.0)];
    for &(s0, s1, x1) in &spans {
        let ns = ((s1 - s0) * 2.5).ceil().max(1.0) as usize;
        f.skin(&mut w, &skin, s0, s1, ns, &zero, &le, 18, false);
        f.skin(&mut w, &skin, s0, s1, ns, &le, &|_| x1, 26, false);
        if x1 < 1.0 {
            f.web(&mut w, &WELL_MAT, s0, s1, ns, &|_| x1);
            f.cap(&mut w, &WELL_MAT, s0, x1, 1.0, false);
            f.cap(&mut w, &WELL_MAT, s1, x1, 1.0, false);
        }
    }
    f.cap(&mut w, &skin, SPAN, 0.0, 1.0, false);
    // The leading-edge sections' fasteners: rows along their back edges, up
    // and under, and across at the joints.
    for side in [1.0f32, -1.0] {
        let mut s = X0 + 0.1;
        let mut row = Vec::new();
        while s < SPAN - 0.35 {
            row.push(foil_frame(&f, s, LE_XC, side));
            s += 0.16;
        }
        screw_row(&mut w, &row, 0.0042, side * 7.0);
    }
    for &s in &LE_JOINTS[1..LE_JOINTS.len() - 1] {
        let row: Vec<(Vec3, Vec3)> = (0..=14).map(|k| { let t = k as f32 / 14.0 * 2.0 - 1.0; foil_frame(&f, s, LE_XC * t.abs().max(0.004), t.signum()) }).collect();
        screw_row(&mut w, &row, 0.0042, s);
    }
    // Front and rear spar fastener rows on the upper skin, the fuel filler
    // caps and the inspection plates underneath.
    for (xc, side) in [(0.16, 1.0f32), (0.62, 1.0)] {
        let mut s = X0 + 0.2;
        let mut row = Vec::new();
        while s < SPAN - 0.6 {
            row.push(foil_frame(&f, s, xc, side));
            s += 0.30;
        }
        screw_row(&mut w, &row, 0.0042, xc * 31.0 + side);
    }
    for s in [3.6f32, 7.9, 12.0, 16.4] {
        let (p, n) = foil_frame(&f, s, 0.34, 1.0);
        w.lathe(&mt(mat::LIVERY, 0.20, 0.22, 0.22), p - n * 0.003, n, &[[0.075, 0.0], [0.075, 0.008], [0.06, 0.012], [0.0, 0.012]], 18, false);
    }
    for &(s, xc) in &[(3.4f32, 0.42f32), (6.2, 0.40), (9.0, 0.42), (11.8, 0.40), (14.8, 0.42), (17.6, 0.40)] {
        let (p, n) = foil_frame(&f, s, xc, -1.0);
        inspection_plate(&mut w, p + n * 0.003, n, Vec3::Z, 0.12, 8);
    }
    // The fuel vent under the tip, and the dump mast.
    let (p, n) = foil_frame(&f, SPAN - 1.6, 0.55, -1.0);
    w.tube(&ALU, &[p - n * 0.01, p + n * 0.08 + wing_sect(SPAN - 1.6).cdir * 0.06], &|t| 0.02 - 0.006 * t, 8, true);

    // Lights: the navigation light in the tip's leading edge (green on the
    // right), a white strobe behind it; the retractable landing light
    // under the outer wing (lens flush, as retracted); the wing tip's
    // static dischargers.
    let tip = f.pt(SPAN - 0.10, 0.08, 0.0, -1.0);
    w.lathe(&mt(mat::LAMP, 0.1, 0.9, 0.2).x(lamp::NAV_GREEN), tip, Vec3::new(0.6, 0.0, -0.8), &[[0.06, 0.0], [0.06, 0.04], [0.04, 0.08], [0.0, 0.09]], 14, true);
    let st = f.pt(SPAN - 0.08, 0.55, 0.0, -1.0);
    w.lathe(&mt(mat::LAMP, 0.9, 0.9, 0.9).x(lamp::STROBE), st, Vec3::X, &[[0.035, 0.0], [0.035, 0.03], [0.0, 0.05]], 10, true);
    let (p, n) = foil_frame(&f, LANDING_LIGHT_X, 0.10, -1.0);
    w.lathe(&DARK, p - n * 0.004, n, &[[0.17, 0.0], [0.17, 0.006], [0.0, 0.006]], 20, false);
    w.disc(&mt(mat::LAMP, 0.95, 0.92, 0.85).x(lamp::LANDING), p + n * 0.004, (n + wing_sect(LANDING_LIGHT_X).cdir * -0.6).normalize(), Vec3::Z, 0.12, 18);
    for s in [SPAN - 0.55, SPAN - 0.25] {
        wick(&mut w, f.pt(s, 1.0, 0.0, -1.0), wing_sect(s).cdir);
    }

    // Flaps: two Lockheed-Fowler flaps each side. Each runs aft along its
    // tracks first, then turns down to full travel (`FLAP_MAX_DEG`): a turn
    // about its nose plus a slide aft and a little down along the tracks
    // [est: 0.65 of the flap's chord of travel].
    //
    // The tracks: a steel beam at each `TRACKS` station on the line the
    // flap's nose (its hinge, `pivot`) slides along, reaching from the hinge
    // forward into the wing behind its rear web. They slide with the flap
    // but do not turn with it, so at any setting a beam runs from inside the
    // wing, across the open slot, into the flap's nose: what shows of the
    // real tracks. Flaps up, the beam lies wholly inside the wing and the
    // flap's nose (the test `flap_tracks_hide_inside_the_wing` checks it).
    let track_mat = mt(mat::STEEL, 0.30, 0.31, 0.32);
    for (i, (span, names, tnames)) in [
        (FLAP_IB, ["left inboard flap", "right inboard flap"], ["left inboard flap tracks", "right inboard flap tracks"]),
        (FLAP_OB, ["left outboard flap", "right outboard flap"], ["left outboard flap tracks", "right outboard flap tracks"]),
    ]
    .into_iter()
    .enumerate()
    {
        let mut fl = Mesh::default();
        let (s0, s1) = (span[0] + 0.03, span[1] - 0.03);
        let ns = ((s1 - s0) * 2.5).ceil() as usize;
        f.skin(&mut fl, &ctrl, s0, s1, ns, &|_| XF_FLAP + 0.006, &one, 16, true);
        f.cap(&mut fl, &ctrl, s0, XF_FLAP + 0.006, 1.0, true);
        f.cap(&mut fl, &ctrl, s1, XF_FLAP + 0.006, 1.0, true);
        // the flap's trailing-edge static dischargers are on the ailerons only
        let (pivot, axis, dir, travel) = flap_hinge(&f, s0, s1);
        let mut tr = Mesh::default();
        for t in TRACKS[i] {
            let (a, bb) = track_ends(&f, s0, s1, t);
            tr.rbox(&track_mat, (a + bb) * 0.5, Vec3::new(TRACK_HALF[0], TRACK_HALF[1], (bb - a).length() * 0.5), 0.008, Quat::from_rotation_arc(Vec3::Z, (bb - a).normalize()));
        }
        let (l, r) = b.pair(names, [G_LWING, G_RWING], fl, |s| Anim::turn_slide(Drive::new(Input::Flaps, FLAP_MAX_DEG.to_radians()).smooth(0.15, 1.0), Drive::new(Input::Flaps, travel).smooth(0.0, 0.7), dir * Vec3::new(s, 1.0, 1.0)), pivot, axis, [None, None]);
        b.role(l, Role::Flap(0)).role(r, Role::Flap(1));
        let (tl, trr) = b.pair(tnames, [G_LWING, G_RWING], tr, |s| Anim::slide(Drive::new(Input::Flaps, travel).smooth(0.0, 0.7), dir * Vec3::new(s, 1.0, 1.0)), pivot, axis, [None, None]);
        b.role(tl, Role::Flap(0)).role(trr, Role::Flap(1));
    }

    // Ailerons, with the tab let into each one's trailing edge.
    let mut al = Mesh::default();
    let (s0, s1) = (AILERON[0] + 0.02, AILERON[1] - 0.02);
    for (a, c, x1) in [(s0, AIL_TAB[0], 1.0), (AIL_TAB[0], AIL_TAB[1], TAB_XC - 0.004), (AIL_TAB[1], s1, 1.0)] {
        let ns = ((c - a) * 2.5).ceil().max(1.0) as usize;
        f.skin(&mut al, &ctrl, a, c, ns, &|_| XF_AIL + 0.006, &|_| x1, 12, true);
        if x1 < 1.0 {
            f.web(&mut al, &WELL_MAT, a, c, ns, &|_| x1);
            f.cap(&mut al, &WELL_MAT, a, x1, 1.0, false);
            f.cap(&mut al, &WELL_MAT, c, x1, 1.0, false);
        }
    }
    f.cap(&mut al, &ctrl, s0, XF_AIL + 0.006, 1.0, true);
    f.cap(&mut al, &ctrl, s1, XF_AIL + 0.006, 1.0, true);
    for s in [17.0f32, 18.2, 19.4] {
        wick(&mut al, f.pt(s, 1.0, 0.0, -1.0), wing_sect(s).cdir);
    }
    // the hinge fairings under the aileron's nose
    for s in [14.4f32, 16.6, 18.8] {
        let (p, n) = foil_frame(&f, s, XF_AIL - 0.02, -1.0);
        w.rbox(&SKIN, p + n * 0.04, Vec3::new(0.03, 0.04, 0.16), 0.02, Quat::from_rotation_arc(Vec3::Y, -n));
    }
    let axis = (f.pt(s1, XF_AIL + 0.02, 0.0, -1.0) - f.pt(s0, XF_AIL + 0.02, 0.0, -1.0)).normalize();
    let pivot = f.pt((s0 + s1) * 0.5, XF_AIL + 0.02, 0.0, -1.0);
    let (l, r) = b.pair(["left aileron", "right aileron"], [G_LWING, G_RWING], al, |s| Anim::hinge(Drive::new(Input::Aileron(s), 0.35)), pivot, axis, [None, None]);
    b.role(l, Role::Aileron(0)).role(r, Role::Aileron(1));
    // The tab: geared to move against the aileron [est: about 0.8 of its angle].
    let mut tab = Mesh::default();
    let (a, c) = (AIL_TAB[0] + 0.015, AIL_TAB[1] - 0.015);
    f.skin(&mut tab, &ctrl, a, c, 6, &|_| TAB_XC + 0.003, &one, 6, true);
    f.cap(&mut tab, &ctrl, a, TAB_XC + 0.003, 1.0, true);
    f.cap(&mut tab, &ctrl, c, TAB_XC + 0.003, 1.0, true);
    let tax = (f.pt(c, TAB_XC + 0.01, 0.0, -1.0) - f.pt(a, TAB_XC + 0.01, 0.0, -1.0)).normalize();
    let tp = f.pt((a + c) * 0.5, TAB_XC + 0.01, 0.0, -1.0);
    let (tl, tr) = b.pair(["left aileron tab", "right aileron tab"], [G_LWING, G_RWING], tab, |s| Anim::hinge(Drive::new(Input::Aileron(s), -0.28)), tp, tax, [Some(l), Some(r)]);
    b.role(tl, Role::Aileron(0)).role(tr, Role::Aileron(1));

    let mut lw = mirrored(&w);
    for v in &mut lw.v {
        if v.mat[0] == mat::LAMP && v.mat[3] == lamp::NAV_GREEN {
            v.mat[3] = lamp::NAV_RED;
            v.col = [0.95, 0.1, 0.05, v.col[3]];
        }
    }
    b.fixed("left wing", G_LWING, lw);
    b.fixed("right wing", G_RWING, w);

    // Spars and ribs for the stump when a wing breaks.
    let wf = wing_foil();
    let mut ws = Mesh::default();
    let mut ribs = Vec::new();
    let mut s = X0 + 0.4;
    while s < SPAN - 0.4 {
        let to = spans.iter().find(|sp| s >= sp.0 && s < sp.1).map(|sp| sp.2).unwrap_or(1.0) - 0.03;
        ribs.push((s, 0.03, to.min(0.97)));
        s += 0.75;
    }
    structure(&wf, &mut ws, X0, SPAN - 0.4, &[0.15, 0.62], &ribs);
    let l = b.fixed("left wing structure", G_LWING, mirrored(&ws));
    let r = b.fixed("right wing structure", G_RWING, ws);
    b.role(l, Role::WingStructure(0)).role(r, Role::WingStructure(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flaps up, no part of a track beam shows: every corner along it is
    /// between the wing's (or the retracted flap's) upper and lower skins.
    #[test]
    fn flap_tracks_hide_inside_the_wing() {
        let f = render_wing();
        for (i, span) in [FLAP_IB, FLAP_OB].into_iter().enumerate() {
            let (s0, s1) = (span[0] + 0.03, span[1] - 0.03);
            for t in TRACKS[i] {
                let (a, b) = track_ends(&f, s0, s1, t);
                let rot = Quat::from_rotation_arc(Vec3::Z, (b - a).normalize());
                let k = wing_sect(t);
                for j in 0..=20 {
                    let c = a + (b - a) * (j as f32 / 20.0);
                    for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                        let p = c + rot * Vec3::new(sx * TRACK_HALF[0], sy * TRACK_HALF[1], 0.0);
                        let xc = (p - k.le).dot(k.cdir) / k.c;
                        let h = (p - k.le).dot(k.up);
                        let up = (f.pt(t, xc, 1.0, -1.0) - k.le).dot(k.up);
                        let lo = (f.pt(t, xc, -1.0, -1.0) - k.le).dot(k.up);
                        assert!(xc > 0.2 && xc < 0.98 && h < up - 0.005 && h > lo + 0.005, "track at {t} m: corner at {xc:.3} chord, {h:.3} m, skins {lo:.3}..{up:.3}");
                    }
                }
            }
        }
    }
}
