//! The tail: the fin with its long dorsal fillet, the rudder and its tab,
//! the tailplane, the elevators and their tabs, the spars and ribs a broken
//! tail shows. (The tail cone and its light are the fuselage's.)
use super::*;

/// The tailplane as drawn: `dims::stab_sect`, its tips rounded off [est].
fn render_stab() -> Foil {
    Foil {
        at: Box::new(|s: f32| {
            let mut k = stab_sect(s);
            let x = s.abs();
            let r = 0.30;
            if x > STAB_SPAN - r {
                let u = ((x - (STAB_SPAN - r)) / r).min(1.0);
                let q = 1.0 - (1.0 - u * u).max(0.0).sqrt();
                k.le += k.cdir * (k.c * 0.22 * q);
                k.c -= k.c * 0.42 * q;
                k.tf *= (1.0 - u * u).max(0.03).sqrt();
            }
            k
        }),
        t: 0.12,
        camber: 0.0,
    }
}
/// The fin as drawn: `dims::fin_sect`, a rounded cap on top [est].
fn render_fin() -> Foil {
    Foil {
        at: Box::new(|s: f32| {
            let mut k = fin_sect(s);
            let r = 0.22;
            if s > FIN_H - r {
                let u = ((s - (FIN_H - r)) / r).min(1.0);
                let q = 1.0 - (1.0 - u * u).max(0.0).sqrt();
                k.le += k.cdir * (k.c * 0.05 * q);
                k.c -= k.c * 0.10 * q;
                k.tf *= (1.0 - u * u).max(0.03).sqrt();
            }
            k
        }),
        t: 0.13,
        camber: 0.0,
    }
}

/// The elevator's and rudder's tabs: span (from the root / the fin's root)
/// and the chord fraction of their hinges [est].
const ELEV_TAB: [f32; 2] = [1.05, 3.10];
const RUDDER_TAB: [f32; 2] = [0.55, 2.70];
const TAB_XC: f32 = 0.905;

pub(super) fn tail(b: &mut Builder) {
    let stab = render_stab();
    let fin = render_fin();
    let grey = SKIN.pat(pat::TAIL);
    let ctrl = SKIN.pat(pat::CTRL);
    let one = |_: f32| 1.0;
    let zero = |_: f32| 0.0;

    // The tailplane, both halves from inside the tail cone out.
    let mut m = Mesh::default();
    for sgn in [1.0f32, -1.0] {
        let (a, c) = if sgn > 0.0 { (0.0, STAB_SPAN) } else { (-STAB_SPAN, 0.0) };
        stab.skin(&mut m, &grey, a, c, 34, &zero, &|_| ELEV_HINGE - 0.004, 22, false);
        stab.web(&mut m, &WELL_MAT, a, c, 12, &|_| ELEV_HINGE - 0.004);
        // the tip, outboard of the elevator
        let t = sgn * STAB_SPAN;
        let e = sgn * (STAB_SPAN - 0.05);
        let (a, c) = if sgn > 0.0 { (e, t) } else { (t, e) };
        stab.skin(&mut m, &grey, a, c, 3, &|_| ELEV_HINGE - 0.004, &one, 8, false);
        stab.cap(&mut m, &grey, t, 0.0, 1.0, false);
        // leading-edge sections' fasteners
        let row: Vec<(Vec3, Vec3)> = (0..60).flat_map(|k| { let s = sgn * (0.6 + k as f32 * 0.12); [foil_frame(&stab, s, 0.10, 1.0), foil_frame(&stab, s, 0.10, -1.0)] }).filter(|(p, _)| p.x.abs() < STAB_SPAN - 0.3).collect();
        screw_row(&mut m, &row, 0.005, sgn);
    }
    let k = b.fixed("stabiliser", G_TAIL, m);
    b.role(k, Role::Stab);

    // Elevators, a tab let into each one's trailing edge inboard.
    let mut el = Mesh::default();
    let (x0, x1) = (ELEV_ROOT, STAB_SPAN - 0.06);
    for (a, c, xt) in [(x0, ELEV_TAB[0], 1.0), (ELEV_TAB[0], ELEV_TAB[1], TAB_XC - 0.004), (ELEV_TAB[1], x1, 1.0)] {
        stab.skin(&mut el, &ctrl, a, c, ((c - a) * 3.0).ceil() as usize, &|_| ELEV_HINGE + 0.004, &|_| xt, 10, true);
        if xt < 1.0 {
            stab.web(&mut el, &WELL_MAT, a, c, 6, &|_| xt);
            stab.cap(&mut el, &WELL_MAT, a, xt, 1.0, false);
            stab.cap(&mut el, &WELL_MAT, c, xt, 1.0, false);
        }
    }
    stab.cap(&mut el, &ctrl, x0, ELEV_HINGE + 0.004, 1.0, true);
    stab.cap(&mut el, &ctrl, x1, ELEV_HINGE + 0.004, 1.0, true);
    for s in [5.2f32, 6.4, 7.6] {
        wick(&mut el, stab.pt(s, 1.0, 0.0, -1.0), Vec3::Z);
    }
    let p0 = stab.pt(x0, ELEV_HINGE, 0.0, -1.0);
    let p1 = stab.pt(x1, ELEV_HINGE, 0.0, -1.0);
    let (l, r) = b.pair(["left elevator", "right elevator"], [G_TAIL, G_TAIL], el, |_| Anim::hinge(Drive::new(Input::Elevator, -0.40)), p0, p1 - p0, [None, None]);
    b.role(l, Role::Elevator).role(r, Role::Elevator);
    // the tabs: geared against the elevator [est].
    let mut tab = Mesh::default();
    let (a, c) = (ELEV_TAB[0] + 0.015, ELEV_TAB[1] - 0.015);
    stab.skin(&mut tab, &ctrl, a, c, 6, &|_| TAB_XC + 0.003, &one, 6, true);
    stab.cap(&mut tab, &ctrl, a, TAB_XC + 0.003, 1.0, true);
    stab.cap(&mut tab, &ctrl, c, TAB_XC + 0.003, 1.0, true);
    let t0 = stab.pt(a, TAB_XC + 0.01, 0.0, -1.0);
    let t1 = stab.pt(c, TAB_XC + 0.01, 0.0, -1.0);
    let (tl, tr) = b.pair(["left elevator tab", "right elevator tab"], [G_TAIL, G_TAIL], tab, |_| Anim::hinge(Drive::new(Input::Elevator, 0.30)), t0, t1 - t0, [Some(l), Some(r)]);
    b.role(tl, Role::Elevator).role(tr, Role::Elevator);

    // The fin: below, beside and above the rudder; its cap.
    let mut fm = Mesh::default();
    fin.skin(&mut fm, &grey, -0.4, RUDDER_SPAN[0], 4, &zero, &one, 26, false);
    fin.skin(&mut fm, &grey, RUDDER_SPAN[0], RUDDER_SPAN[1], 34, &zero, &|_| RUDDER_HINGE - 0.004, 26, false);
    fin.web(&mut fm, &WELL_MAT, RUDDER_SPAN[0], RUDDER_SPAN[1], 12, &|_| RUDDER_HINGE - 0.004);
    fin.cap(&mut fm, &WELL_MAT, RUDDER_SPAN[0], RUDDER_HINGE - 0.004, 1.0, false);
    fin.cap(&mut fm, &WELL_MAT, RUDDER_SPAN[1], RUDDER_HINGE - 0.004, 1.0, false);
    fin.skin(&mut fm, &grey, RUDDER_SPAN[1], FIN_H, 6, &zero, &one, 26, false);
    fin.cap(&mut fm, &grey, FIN_H, 0.0, 1.0, false);
    let row: Vec<(Vec3, Vec3)> = (0..56).flat_map(|k| { let s = 0.3 + k as f32 * 0.12; [foil_frame(&fin, s, 0.10, 1.0), foil_frame(&fin, s, 0.10, -1.0)] }).filter(|(p, _)| p.y < by(HEIGHT) - 0.3).collect();
    screw_row(&mut fm, &row, 0.005, 9.0);
    // The long dorsal fillet: a low ridge rising off the crown from behind
    // the wing to run into the fin's leading edge [est, from photographs and
    // the three-views].
    let fin_le = |s: f32| FIN_LE[0] + (FIN_LE[1] - FIN_LE[0]) * s / FIN_H;
    let fillet = Foil {
        at: Box::new(move |s: f32| {
            let u = (s / 1.15).clamp(0.0, 1.0);
            let le = fin_le(s.max(0.0)) - 3.5 * (1.0 - u).powi(2);
            let te = fin_le(s.max(0.0)) + 1.3;
            Sect { le: Vec3::new(0.0, by(FIN_ROOT_H) + s, bz(le)), c: te - le, cdir: Vec3::Z, up: Vec3::X, tf: 1.0 - 0.3 * u }
        }),
        t: 0.09,
        camber: 0.0,
    };
    fillet.skin(&mut fm, &grey, -0.25, 1.15, 10, &zero, &one, 20, false);
    // the fin tip's fairing and its static discharger
    wick(&mut fm, fin.pt(FIN_H - 0.35, 1.0, 0.0, -1.0), Vec3::Z);
    let k = b.fixed("fin", G_TAIL, fm);
    b.role(k, Role::Stab);

    // The rudder, its tab let into its trailing edge low down.
    let mut ru = Mesh::default();
    for (a, c, xt) in [(RUDDER_SPAN[0], RUDDER_TAB[0], 1.0), (RUDDER_TAB[0], RUDDER_TAB[1], TAB_XC - 0.004), (RUDDER_TAB[1], RUDDER_SPAN[1], 1.0)] {
        fin.skin(&mut ru, &ctrl, a, c, ((c - a) * 3.0).ceil().max(1.0) as usize, &|_| RUDDER_HINGE + 0.004, &|_| xt, 12, true);
        if xt < 1.0 {
            fin.web(&mut ru, &WELL_MAT, a, c, 6, &|_| xt);
            fin.cap(&mut ru, &WELL_MAT, a, xt, 1.0, false);
            fin.cap(&mut ru, &WELL_MAT, c, xt, 1.0, false);
        }
    }
    fin.cap(&mut ru, &ctrl, RUDDER_SPAN[0], RUDDER_HINGE + 0.004, 1.0, true);
    fin.cap(&mut ru, &ctrl, RUDDER_SPAN[1], RUDDER_HINGE + 0.004, 1.0, true);
    for s in [4.4f32, 5.6, 6.8] {
        wick(&mut ru, fin.pt(s, 1.0, 0.0, -1.0), Vec3::Z);
    }
    let p0 = fin.pt(RUDDER_SPAN[0], RUDDER_HINGE, 0.0, -1.0);
    let p1 = fin.pt(RUDDER_SPAN[1], RUDDER_HINGE, 0.0, -1.0);
    let rk = b.add("rudder", G_TAIL, ru, Anim::hinge(Drive::new(Input::Rudder, 0.45)), p0, p1 - p0, None);
    b.role(rk, Role::Rudder);
    let mut tab = Mesh::default();
    let (a, c) = (RUDDER_TAB[0] + 0.015, RUDDER_TAB[1] - 0.015);
    fin.skin(&mut tab, &ctrl, a, c, 6, &|_| TAB_XC + 0.003, &one, 6, true);
    fin.cap(&mut tab, &ctrl, a, TAB_XC + 0.003, 1.0, true);
    fin.cap(&mut tab, &ctrl, c, TAB_XC + 0.003, 1.0, true);
    let t0 = fin.pt(a, TAB_XC + 0.01, 0.0, -1.0);
    let t1 = fin.pt(c, TAB_XC + 0.01, 0.0, -1.0);
    let tk = b.add("rudder tab", G_TAIL, tab, Anim::hinge(Drive::new(Input::Rudder, -0.30)), t0, t1 - t0, Some(rk));
    b.role(tk, Role::Rudder);

    // Spars and ribs for when it breaks.
    let mut ts = Mesh::default();
    let sf = stab_foil();
    structure(&sf, &mut ts, -STAB_SPAN + 0.3, STAB_SPAN - 0.3, &[0.15, 0.6], &[(-6.0, 0.03, 0.66), (-4.0, 0.03, 0.66), (-2.0, 0.03, 0.66), (2.0, 0.03, 0.66), (4.0, 0.03, 0.66), (6.0, 0.03, 0.66)]);
    let ff = fin_foil();
    structure(&ff, &mut ts, 0.2, FIN_H - 0.3, &[0.15, 0.6], &[(1.5, 0.03, 0.62), (3.0, 0.03, 0.62), (4.5, 0.03, 0.62), (6.0, 0.03, 0.62)]);
    let k = b.fixed("tail structure", G_TAIL, ts);
    b.role(k, Role::TailStructure);
}
