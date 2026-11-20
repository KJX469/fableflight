//! The surface-element aerodynamics: the wing, the tailplane and the fin cut
//! into spanwise strips, each flying in its own local airflow, the forces and
//! moments summed about the centre of gravity.
//!
//! Each strip sees the aircraft's airspeed plus the velocity its own position
//! adds as the aircraft rotates (ω × r: what damps a roll, a pitch and a yaw),
//! the wing's downwash (at the tail, lagged by the time the air takes to get
//! there: the damping of a quick pitch), the propeller's slipstream and its
//! swirl (the tail's authority at low speed and full power, the left-turning
//! tendency), and ground effect. It works out its angle of attack, adds what
//! its control surface or flap does, and takes a lift and drag coefficient
//! from its airfoil: linear up to its own maximum, then a separated flat-plate
//! flow it leaves and rejoins with a lag. Stalls start where the local angle
//! of attack is highest (at the root of a washed-out wing, at the tip of a
//! swept one) and a wing drops because one side really is flying at a higher
//! angle than the other. Damage takes strips away: a shortened wing loses its
//! outboard strips, a missing elevator its control.
//!
//! What strips cannot give is added as coefficients (`Fuse`): the fuselage's
//! own unstable pitching and yawing moments and side force, and the roll due
//! to sideslip of the wing-fuselage junction (a high wing's extra stability).
//!
//! Everything comes from data: an aircraft opts in with a `Rig` on its `Spec`
//! and the airframe is built from that and the geometry the `Spec` already
//! has (the wing's sections, the aileron span). Nothing here knows which
//! aircraft it is flying.
//!
//! Frame: the body frame, +X right, +Y up, -Z forward, metres.

use crate::aircraft::Spec;
use glam::DVec3;

/// An airfoil section's characteristics (2-D, per radian).
#[derive(Clone, Copy, Debug)]
pub struct Foil2d {
    /// lift-curve slope (per rad; thin-airfoil theory gives 2π, real sections ~5.7-6.3)
    pub cl_a: f64,
    /// zero-lift angle (rad; negative for a cambered section)
    pub a0: f64,
    /// maximum lift coefficient, clean
    pub cl_max: f64,
    /// minimum drag coefficient
    pub cd0: f64,
    /// pitching moment about the quarter chord
    pub cm0: f64,
}

impl Foil2d {
    /// A symmetric tail section (NACA 0009-0012).
    pub const SYMMETRIC: Foil2d = Foil2d { cl_a: 6.0, a0: 0.0, cl_max: 1.2, cd0: 0.0075, cm0: 0.0 };
}

/// A straight-tapered surface, given by its quarter-chord line (the right or
/// upper half; a horizontal surface is mirrored to the left).
#[derive(Clone, Copy, Debug)]
pub struct Planform {
    /// quarter-chord point at the root and at the tip (body frame, m)
    pub root: DVec3,
    pub tip: DVec3,
    /// chord at the root and at the tip (m)
    pub chord: [f64; 2],
    /// incidence (rad, leading edge up) at the root and the tip
    pub incidence: [f64; 2],
}

/// A control surface or flap: where it runs along the span (m from the root
/// of its surface), its chord as a fraction of the surface's, and its travel
/// (rad) trailing edge up (or left) and down (or right), both positive.
#[derive(Clone, Copy, Debug)]
pub struct Ctrl {
    pub span: [f64; 2],
    pub chord: f64,
    pub travel: [f64; 2],
}

/// What strips cannot show: the fuselage and the wing-fuselage junction, as
/// stability-axis coefficients in the usual sign convention (per radian; on
/// the wing area, chord and span).
#[derive(Clone, Copy, Debug)]
pub struct Fuse {
    /// pitching moment from angle of attack (positive: nose-up, destabilising)
    pub cm_a: f64,
    /// yawing moment from sideslip (negative: destabilising)
    pub cn_b: f64,
    /// rolling moment from sideslip (negative: stabilising, more for a high wing)
    pub cl_b: f64,
    /// side force from sideslip (negative)
    pub cy_b: f64,
    /// lift from angle of attack (per rad: the fuselage, chines, strakes
    /// and leading-edge extensions; 0 for most)
    pub cl_a: f64,
}

/// An aircraft's aerodynamic rig: what the surface-element model needs that
/// the `Spec` does not already say.
#[derive(Clone, Copy, Debug)]
pub struct Rig {
    /// the wing's section; spanwise strips per wing half
    pub wing_foil: Foil2d,
    pub wing_strips: usize,
    /// span efficiency (Oswald), for the induced drag
    pub oswald: f64,
    /// flaps: span and chord, their full deflection taken from `Spec::flaps`;
    /// the extra maximum lift fully down
    pub flap: Ctrl,
    pub flap_dcl_max: f64,
    /// ailerons: chord (a fraction of the wing's, or if `aileron_chord_m` is
    /// set, that many metres along the whole span, as a delta's elevons are)
    /// and travel (span from `Spec::aileron`)
    pub aileron_chord: f64,
    pub aileron_chord_m: f64,
    pub aileron_travel: [f64; 2],
    /// a tailless aircraft (a delta): no horizontal tail; the wing's trailing
    /// edge surfaces (the flap and aileron spans) are elevons, worked by both
    /// the stick's pitch and its roll, with the elevator's travel
    pub elevons: bool,
    /// the horizontal tail, its section, the elevator (span over the whole
    /// half; chord 1 for an all-moving stabilator), and the trim's range as
    /// elevator deflection (nose down, nose up)
    pub htail: Planform,
    pub htail_foil: Foil2d,
    pub elevator: Ctrl,
    pub trim: [f64; 2],
    /// the fin, its section and the rudder (travel left, right)
    pub fin: Planform,
    pub fin_foil: Foil2d,
    pub rudder: Ctrl,
    /// the fuselage's share
    pub fuse: Fuse,
    /// how much of the wing's downwash reaches the tail (1: all of the
    /// lifting-line value), and the tail's dynamic pressure ratio outside the slipstream
    pub downwash: f64,
    pub eta_tail: f64,
    /// the propeller hub (body frame), for its slipstream; the slipstream's
    /// swirl as a fraction of its axial speed (sense from the propeller's turn:
    /// positive clockwise from the cockpit)
    pub prop_hub: DVec3,
    pub swirl: f64,
    /// where the centre of gravity is: false at the body frame's origin (the
    /// convention: the origin is the reference CG, `Spec::cg[0]`); true at
    /// `Spec::cg` on the wing's own mean aerodynamic chord, for a model whose
    /// origin is not its CG
    pub cg_from_mac: bool,
    /// where on `Spec::mac` (as a fraction of it, from its leading edge) the
    /// model's origin sits, for a type whose CG limits are given on a
    /// reference chord the origin is not at the reference CG of (0: it is)
    pub datum: f64,
    /// the trim moves the whole tailplane (a trimmable stabiliser or a
    /// stabilator) rather than a tab on the elevator
    pub trim_stab: bool,
    /// tailerons: the tailplane halves move apart for roll, this far (rad) at full stick
    pub taileron: f64,
    /// inboard (all-speed) ailerons: span, chord and travel (span [0, 0]: none);
    /// and whether the outboard ones work only with the flaps out
    pub aileron_in: Ctrl,
    pub outboard_lock: bool,
    /// how far both ailerons droop (trailing edge down, rad) with the flaps
    /// full out, to flap the whole span (they still roll about it)
    pub aileron_droop: f64,
    /// spoiler panels on the wing: span, chord and full travel; how much of
    /// their travel the roll command uses (on the side whose aileron goes up)
    pub spoiler: Ctrl,
    pub roll_spoiler: f64,
    /// the extra maximum lift of the leading-edge slats, out
    pub slat_dcl_max: f64,
    /// leading-edge vortex lift (Polhamus's K_v, about π for a slender delta)
    /// on the wing's strips swept more than 50° at the leading edge, and the
    /// angle of attack (rad) where the vortex bursts
    pub vortex: f64,
    pub vortex_burst: f64,
    /// the flight control system between the stick and the surfaces
    pub fcs: Fcs,
    /// a CG moved with the Mach number by pumping fuel fore and aft (Concorde,
    /// the SR-71): (Mach, CG as a fraction of `Spec::mac`) pairs, empty for none
    pub cg_mach: &'static [(f64, f64)],
}

/// The flight control system: what the stick asks for and the stability
/// augmentation the real aircraft has. All zero: the stick moves the surfaces.
#[derive(Clone, Copy, Debug)]
pub struct Fcs {
    /// fly-by-wire in pitch: the stick commands load factor (up to
    /// `Spec::fbw_g`, down to -0.4 of it), with an angle-of-attack limit (rad)
    pub g_law: bool,
    pub alpha_max: f64,
    /// fly-by-wire in roll: the roll rate (rad/s) full stick commands (0: the stick moves the ailerons)
    pub roll_rate: f64,
    /// fly-by-wire in yaw: sideslip held at nil (the pedals command it, rad
    /// at full pedal) and the yaw damped
    pub beta_cmd: f64,
    /// dampers on a conventional system (fraction of the surface's travel
    /// per rad/s of pitch, roll and yaw rate; the yaw damper is washed out so it
    /// leaves a steady turn alone), and the most of the travel they may use
    pub damp: [f64; 3],
    pub damp_auth: f64,
    /// the dampers' gains scheduled down in proportion above this dynamic
    /// pressure (Pa; 0: fixed), as a real system's are
    pub damp_q: f64,
    /// the stick's roll and the pedals' authority geared down in proportion
    /// above this dynamic pressure (Pa; 0: never), as a ratio changer does
    pub gear_q: f64,
    /// how much of that gearing the stick's pitch gets too (0 none, 1 all)
    pub gear_pitch: f64,
    /// sideslip fed to the rudder (fraction of its travel per rad: a lateral
    /// accelerometer's signal), stiffening a weak fin
    pub yaw_beta: f64,
    /// speed stability for the g law (a C*U law, as an airliner's or a
    /// business jet's): load factor taken off per fraction of the trimmed
    /// speed it is slow by, so that let go it returns to the speed it was
    /// trimmed at, as a conventional aircraft does (0: none; a fighter's
    /// law holds 1 g whatever the speed)
    pub speed_hold: f64,
    /// a flare mode for the g law: below 50 ft with the gear down the stick
    /// asks for angle of attack above what it flew at 50 ft (rad at full
    /// stick), as a conventional aircraft's elevator does, so a steady pull
    /// flares it rather than climbing it away (0: none)
    pub flare_alpha: f64,
}

impl Fcs {
    pub const NONE: Fcs = Fcs { g_law: false, alpha_max: 0.0, roll_rate: 0.0, beta_cmd: 0.0, damp: [0.0; 3], damp_auth: 0.0, damp_q: 0.0, gear_q: 0.0, gear_pitch: 0.0, yaw_beta: 0.0, speed_hold: 0.0, flare_alpha: 0.0 };
}

impl Rig {
    /// Where the fuel transfer puts the CG at this Mach number (None: it has none).
    pub fn cg_for(&self, mach: f64) -> Option<f64> {
        let t = self.cg_mach;
        let (first, last) = (t.first()?, t.last()?);
        if mach <= first.0 {
            return Some(first.1);
        }
        for w in t.windows(2) {
            if mach <= w[1].0 {
                return Some(w[0].1 + (w[1].1 - w[0].1) * (mach - w[0].0) / (w[1].0 - w[0].0));
            }
        }
        Some(last.1)
    }

    /// No extras: one aileron each side, no spoilers, slats or vortex lift,
    /// an elevator trim tab, a CG at the origin, no augmentation. An
    /// aircraft's rig starts from this (`..Rig::BASE`) and sets what it has.
    pub const BASE: Rig = Rig {
        wing_foil: Foil2d { cl_a: 6.0, a0: 0.0, cl_max: 1.4, cd0: 0.006, cm0: 0.0 },
        wing_strips: 12,
        oswald: 0.8,
        flap: Ctrl { span: [0.0, 0.0], chord: 0.25, travel: [0.0, 0.0] },
        flap_dcl_max: 0.0,
        aileron_chord: 0.25,
        aileron_chord_m: 0.0,
        aileron_travel: [0.35, 0.26],
        elevons: false,
        htail: Planform { root: DVec3::ZERO, tip: DVec3::ZERO, chord: [0.0; 2], incidence: [0.0; 2] },
        htail_foil: Foil2d::SYMMETRIC,
        elevator: Ctrl { span: [0.0, 0.0], chord: 0.35, travel: [0.44, 0.35] },
        trim: [0.17, 0.17],
        fin: Planform { root: DVec3::ZERO, tip: DVec3::ZERO, chord: [0.0; 2], incidence: [0.0; 2] },
        fin_foil: Foil2d::SYMMETRIC,
        rudder: Ctrl { span: [0.0, 0.0], chord: 0.3, travel: [0.44, 0.44] },
        fuse: Fuse { cm_a: 0.2, cn_b: -0.04, cl_b: -0.03, cy_b: -0.2, cl_a: 0.0 },
        downwash: 1.0,
        eta_tail: 0.9,
        prop_hub: DVec3::ZERO,
        swirl: 0.0,
        cg_from_mac: false,
        datum: 0.0,
        trim_stab: false,
        taileron: 0.0,
        aileron_in: Ctrl { span: [0.0, 0.0], chord: 0.0, travel: [0.0, 0.0] },
        outboard_lock: false,
        aileron_droop: 0.0,
        spoiler: Ctrl { span: [0.0, 0.0], chord: 0.15, travel: [0.0, 0.0] },
        roll_spoiler: 0.0,
        slat_dcl_max: 0.0,
        vortex: 0.0,
        vortex_burst: 0.6,
        fcs: Fcs::NONE,
        cg_mach: &[],
    };

    /// A first rig for any aircraft, from what its `Spec` already says: the
    /// wing from its sections, the section's lift and stall from the whole
    /// aircraft's polar, the tail from its tips (`Spec::tail`) sized by typical
    /// tail volume coefficients (Raymer, Aircraft Design, table 6.4), the
    /// fuselage's pitching moment from its length (Raymer eq. 16.25 with a
    /// typical fineness), typical control travel, and the augmentation its
    /// class usually has. Every aircraft without a rig of its own flies on
    /// this (a downloaded one too); tuning it to the real aircraft (its tail
    /// areas, sections and travel) is what an aircraft's own `Rig` is for.
    pub fn derive(spec: &Spec) -> Rig {
        let jet = spec.is_jet();
        let span = spec.span as f64;
        let s = spec.wing_area;
        let ar = (2.0 * span).powi(2) / s;
        let a2d = 6.0;
        let deg = |d: f64| d.to_radians();
        // The aircraft's lift slope and maximum lift, back to a section's.
        let cl_max_3d = spec.cl0 + spec.cl_a * spec.stall_alpha;
        let a0 = -spec.cl0 / spec.cl_a;
        let oswald = (1.0 / (std::f64::consts::PI * ar * spec.k_ind)).clamp(0.5, 0.95);
        let root = (spec.wing_sect)(0.0);
        let mac = s / (2.0 * span);
        let wing_ac = root.le.z as f64 + 0.25 * root.c as f64;
        let t = &spec.tail;
        let st = t.stab_tip;
        // No tailplane if the "stabiliser tip" lies on the wing itself (a delta's elevons).
        let ws = (spec.wing_sect)(st.x as f32);
        let elevons = (st.x as f32) < spec.span && st.z <= (ws.le.z + ws.c) as f64 + 0.5;
        // Horizontal tail: about 0.7 (light aircraft) to 0.9 (airliners, fighters) tail volume.
        let lt = (st.z - wing_ac).max(0.3 * span);
        let sh = if jet { 0.85 } else { 0.7 } * s * mac / lt;
        let bh = (st.x * 2.0).max(0.5);
        let ch = sh / bh;
        let htail = Planform {
            root: DVec3::new(0.0, st.y, st.z - 0.3 * ch),
            tip: DVec3::new(st.x, st.y, st.z),
            chord: [1.25 * ch, 0.75 * ch],
            incidence: [-0.03, -0.03],
        };
        // Fin: 0.04 (light aircraft) to 0.08 (jets); twin fins if its root is off the centreline.
        let (fb, ft) = (t.fin_base, t.fin_tip);
        let twin = fb.x.abs() > 0.3;
        let lv = (fb.z - wing_ac).max(0.3 * span);
        let sv = if jet { 0.08 } else { 0.04 } * s * 2.0 * span / lv / if twin { 2.0 } else { 1.0 };
        let hv = (ft - fb).length().max(0.5);
        let cv = sv / hv;
        let fin = Planform { root: fb, tip: ft, chord: [1.3 * cv, 0.7 * cv], incidence: [0.0, 0.0] };
        // An agile jet's tailplane moves as a whole (a stabilator), and an
        // airliner's is trimmed by moving it.
        let stabilator = jet && spec.g_limit >= 6.0;
        let nose = spec.contacts.iter().map(|c| c.0[2]).fold(0.0f64, f64::min);
        let tail_z = spec.contacts.iter().map(|c| c.0[2]).fold(0.0f64, f64::max);
        // Fuselage: Raymer's K_f W² L / (c S) per degree, K_f ~0.01 for a
        // wing a third of the way back, W ~ L/10.
        let len = (tail_z - nose).max(2.0);
        let cm_a = (0.01 * (0.1 * len).powi(2) * len / (mac * s) * 57.3).clamp(0.05, 0.6);
        // Flaps, unless the lever works something else (Concorde's nose).
        let flap_deg = if spec.lever.is_some() { 0.0 } else { spec.flaps.last().copied().unwrap_or(0.0) };
        let flap = Ctrl { span: [0.12 * span, spec.aileron[0] as f64], chord: 0.28, travel: [0.0, flap_deg.to_radians()] };
        // A big jet's spoilers dump lift (and help roll); a fighter's speedbrake is a panel.
        // A model built about its CG has its origin on the wing's chord where a
        // CG goes; one whose origin lies well behind that was not, and its CG is
        // put on the chord instead.
        let first = Rig { wing_foil: Foil2d { cl_a: a2d, a0, cl_max: 1.0, cd0: 0.006, cm0: 0.0 }, ..Rig::BASE };
        let wing = Airframe::build(spec, &first);
        let cg_from_mac = -wing.mac_le.z / wing.mac > 0.45;
        let spoiler = if spec.spoiler_cl > 0.0 { Ctrl { span: flap.span, chord: 0.15, travel: [0.0, deg(45.0)] } } else { Rig::BASE.spoiler };
        Rig {
            wing_foil: Foil2d { cl_a: a2d, a0, cl_max: (cl_max_3d * 1.08).max(0.8), cd0: 0.006, cm0: if jet { -0.02 } else { -0.045 } },
            wing_strips: 12,
            oswald,
            flap,
            flap_dcl_max: if flap_deg > 0.0 { 2.0 * spec.flap_cl } else { 0.0 },
            aileron_chord: 0.25,
            aileron_travel: [deg(20.0), deg(15.0)],
            elevons,
            htail,
            htail_foil: Foil2d::SYMMETRIC,
            elevator: if stabilator { Ctrl { span: [0.0, st.x], chord: 1.0, travel: [deg(20.0), deg(10.0)] } } else { Ctrl { span: [0.0, st.x], chord: 0.35, travel: [deg(25.0), deg(20.0)] } },
            trim: if stabilator || jet { [deg(4.0), deg(10.0)] } else { [deg(10.0), deg(10.0)] },
            fin,
            fin_foil: Foil2d::SYMMETRIC,
            rudder: Ctrl { span: [0.0, hv], chord: 0.3, travel: [deg(25.0), deg(25.0)] },
            fuse: Fuse { cm_a, cn_b: -0.04, cl_b: -0.03, cy_b: -0.2, cl_a: 0.0 },
            prop_hub: DVec3::new(0.0, 0.0, nose),
            swirl: if jet { 0.0 } else { 0.25 },
            trim_stab: jet,
            taileron: if stabilator && spec.fbw_g > 0.0 { deg(5.0) } else { 0.0 },
            spoiler,
            roll_spoiler: if spec.spoiler_cl > 0.0 { 0.5 } else { 0.0 },
            slat_dcl_max: a2d * spec.stall_slat,
            vortex: if elevons { 2.5 } else { 0.0 },
            vortex_burst: deg(35.0),
            cg_from_mac,
            fcs: if spec.fbw_g > 0.0 {
                Fcs { g_law: true, alpha_max: spec.alpha_trim, roll_rate: deg(200.0), beta_cmd: deg(8.0), ..Fcs::NONE }
            } else if jet {
                // Jets have dampers: a yaw damper at least.
                Fcs { damp: [0.0, 0.0, 0.6], damp_auth: 0.3, ..Fcs::NONE }
            } else {
                Fcs::NONE
            },
            ..Rig::BASE
        }
    }
}

/// Which surface a strip belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// side 0 left, 1 right
    Wing(usize),
    Tail(usize),
    Fin,
}

/// Which control a wing strip carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WingCtrl {
    None,
    Aileron,
    InboardAileron,
}

/// One spanwise strip.
#[derive(Clone, Debug)]
pub struct Strip {
    pub part: Part,
    /// span station (m from the centreline, or from the fin's root)
    pub station: f64,
    /// quarter-chord centre (body frame)
    pub at: DVec3,
    /// unit vectors: along the chord (leading to trailing edge), the
    /// surface's normal (the side lift is positive toward), along the span
    pub chord_dir: DVec3,
    pub normal: DVec3,
    pub span_dir: DVec3,
    pub chord: f64,
    pub area: f64,
    pub foil: Foil2d,
    /// its surface's aspect ratio, and the tangents of its leading edge's
    /// and its mid-chord line's sweep, for the lift slope at each Mach number
    pub ar: f64,
    pub tan_le: f64,
    pub tan_c2: f64,
    /// 3-D lift slope at low speed, and the induced-drag factor of its surface
    pub cl_a: f64,
    pub k_ind: f64,
    /// effectiveness (rad of angle of attack per rad) of its flap and of its control surface
    pub flap_tau: f64,
    pub ctrl_tau: f64,
    /// how far behind the quarter chord (fraction of the chord) the lift
    /// each adds acts (`flap_cm`)
    pub flap_cm: f64,
    pub ctrl_cm: f64,
    pub ctrl: WingCtrl,
    /// under a spoiler panel
    pub spoiler: bool,
    /// the share of its surface's lift it carries against a strip of the
    /// same chord on its own (Schrenk: the load is the mean of the chord's
    /// and an ellipse's, so it falls away toward the tip as a real wing's does)
    pub load: f64,
    /// the lift slope of half its surface's aspect ratio over its own: what
    /// the two halves' difference in angle of attack is worth
    pub anti: f64,
}

/// The airframe as the model flies it, built once per aircraft from its rig.
#[derive(Clone, Debug)]
pub struct Airframe {
    pub rig: Rig,
    pub strips: Vec<Strip>,
    /// wing: reference area, span, mean aerodynamic chord and its leading edge's z
    pub s_ref: f64,
    pub b_ref: f64,
    pub mac: f64,
    pub mac_le: DVec3,
    pub ar: f64,
    /// the drag area (m²) of everything but the surfaces' sections: what
    /// makes the total parasite drag the `Spec`'s cd0
    pub cda_body: f64,
    /// flaps' full deflection (rad)
    pub flap_max: f64,
    /// wing to tail distance (m), for the downwash's lag
    pub tail_arm: f64,
    /// the reference CG (fraction of `Spec::mac`) and that chord, for the CG's shift with loading
    pub cg_ref: f64,
    pub mac_ref: f64,
    /// where the lift slope stops rising with the Mach number (`Spec::mcrit`)
    pub m_peak: f64,
}

/// Lift-curve slope (per rad) of a finite surface from its section's, at a
/// Mach number: DATCOM's formula below the speed of sound (Helmbold's at low
/// speed, with Prandtl-Glauert and the sweep of the mid-chord line), linear
/// supersonic theory above it (Ackeret's 4/β less the tips' relief, and no
/// more than a slender wing's πA/2 while the leading edge is inside the Mach
/// cone), blended through the transonic. Past `m_peak` (the wing's critical
/// Mach number) the shocks on it let the flow behind them go: the slope stops
/// rising there and sags through the transonic, rather than climbing to the
/// Prandtl-Glauert singularity.
pub fn slope(a2d: f64, ar: f64, tan_c2: f64, tan_le: f64, mach: f64, m_peak: f64) -> f64 {
    use std::f64::consts::PI;
    let k = a2d / (2.0 * PI);
    let sub = |m: f64| {
        let b2 = 1.0 - m * m;
        2.0 * PI * ar / (2.0 + (ar * ar * b2 / (k * k) * (1.0 + tan_c2 * tan_c2 / b2) + 4.0).sqrt())
    };
    let sup = |m: f64| {
        let b = (m * m - 1.0).sqrt();
        let a = 4.0 / b * (1.0 - 1.0 / (2.0 * ar * b)).max(0.5);
        if b < tan_le { a.min(0.5 * PI * ar) } else { a }
    };
    let (m0, m1) = (m_peak.clamp(0.5, 0.92), 1.2);
    if mach <= m0 {
        sub(mach.max(0.0))
    } else if mach >= m1 {
        sup(mach)
    } else {
        let w = smooth(m0, m1, mach);
        (sub(m0) * (1.0 - w) + sup(m1) * w) * (1.0 - 0.2 * (w * std::f64::consts::PI).sin())
    }
}

/// A flap or control surface's effectiveness: the angle of attack its
/// deflection is worth, per radian (thin-airfoil theory, chord fraction `cf`).
pub fn flap_tau(cf: f64) -> f64 {
    if cf <= 0.0 {
        return 0.0;
    }
    let th = (2.0 * cf.min(1.0) - 1.0).clamp(-1.0, 1.0).acos();
    1.0 - (th - th.sin()) / std::f64::consts::PI
}

/// A strip's airflow (see `Airframe::forces`): what is left of it, the flow
/// in its plane, its speed, angle of attack, dynamic pressure ratio,
/// control and flap left, arm from the CG.
type StripFlow = (f64, DVec3, f64, f64, f64, f64, f64, DVec3);

/// Where the lift a flap or control surface adds acts, behind the quarter
/// chord, as a fraction of the chord (thin-airfoil theory: its moment over
/// its lift, ½ sin θ (1 − cos θ) over 2 (π − θ + sin θ), the hinge at
/// cos θ = 2 cf − 1): 0.17 for a quarter-chord flap, toward 0.25 (mid-chord)
/// for a narrow one, none for a surface that turns whole.
pub fn flap_cm(cf: f64) -> f64 {
    if cf <= 0.0 || cf >= 1.0 {
        return 0.0;
    }
    let th = (2.0 * cf - 1.0).clamp(-1.0, 1.0).acos();
    th.sin() * (1.0 - th.cos()) / (4.0 * (std::f64::consts::PI - th + th.sin()))
}

/// The elevator's (or the elevons' pitch share of) deflection, rad,
/// trailing edge down positive: back on the stick raises it (less lift
/// behind the centre of gravity, nose up). A trim tab's trim is in it; a
/// moving stabiliser's is not (it turns the whole surface: `stab_trim`).
fn pitch_defl(inp: &Input, rig: &Rig) -> f64 {
    let e = inp.elevator;
    let stick = if e > 0.0 { -e * rig.elevator.travel[0] } else { -e * rig.elevator.travel[1] };
    let trim = if rig.trim_stab && !rig.elevons { 0.0 } else { inp.trim };
    (stick - trim).clamp(-rig.elevator.travel[0], rig.elevator.travel[1])
}

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Airframe {
    pub fn build(spec: &Spec, rig: &Rig) -> Self {
        let mut strips = Vec::new();
        // The wing, from the `Spec`'s own sections: strips spaced closer
        // toward the tip (cosine spacing), where the load changes fastest.
        let span = spec.span as f64;
        let n = rig.wing_strips.max(4);
        let edge = |i: usize| span * (0.5 * std::f64::consts::PI * i as f64 / n as f64).sin();
        let (mut area, mut mac_num, mut le_num) = (0.0, 0.0, DVec3::ZERO);
        let mut wing = Vec::new();
        for i in 0..n {
            let (y0, y1) = (edge(i), edge(i + 1));
            let ym = 0.5 * (y0 + y1);
            let s = (spec.wing_sect)(ym as f32);
            let (c, le) = (s.c as f64, s.le.as_dvec3());
            let cdir = s.cdir.as_dvec3().normalize();
            let qc = le + cdir * (0.25 * c);
            let a = c * (y1 - y0);
            // The sweep of its leading edge and its mid-chord line.
            let (s0, s1) = ((spec.wing_sect)(y0 as f32), (spec.wing_sect)(y1 as f32));
            let dy = (y1 - y0).max(1e-3);
            let tan_le = ((s1.le.z - s0.le.z) as f64 / dy).max(0.0);
            let tan_c2 = ((s1.le.z + 0.5 * s1.c - s0.le.z - 0.5 * s0.c) as f64 / dy).abs();
            area += 2.0 * a;
            mac_num += 2.0 * a * c;
            le_num += le * (2.0 * a);
            wing.push((ym, qc, cdir, s.up.as_dvec3().normalize(), c, a, tan_le, tan_c2));
        }
        let mac = mac_num / area;
        let mac_le = DVec3::new(0.0, le_num.y / area, le_num.z / area);
        let ar = (2.0 * span).powi(2) / area;
        let k_ind = 1.0 / (std::f64::consts::PI * rig.oswald * ar);
        let ail = [spec.aileron[0] as f64, spec.aileron[1] as f64];
        let within = |c: &Ctrl, y: f64| c.span[1] > c.span[0] && y >= c.span[0] && y <= c.span[1];
        for (ym, qc, cdir, up, c, a, tan_le, tan_c2) in wing {
            for side in 0..2 {
                let m = if side == 0 { DVec3::new(-1.0, 1.0, 1.0) } else { DVec3::ONE };
                let span_dir = up.cross(cdir).normalize() * m;
                let in_flap = within(&rig.flap, ym);
                let ctrl = if (ym >= ail[0] && ym <= ail[1]) || (rig.elevons && in_flap) {
                    WingCtrl::Aileron
                } else if within(&rig.aileron_in, ym) {
                    WingCtrl::InboardAileron
                } else {
                    WingCtrl::None
                };
                strips.push(Strip {
                    part: Part::Wing(side),
                    station: ym,
                    at: qc * m,
                    chord_dir: cdir,
                    normal: up * m,
                    span_dir,
                    chord: c,
                    area: a,
                    foil: rig.wing_foil,
                    ar,
                    tan_le,
                    tan_c2,
                    cl_a: slope(rig.wing_foil.cl_a, ar, tan_c2, tan_le, 0.0, 0.92),
                    k_ind,
                    flap_tau: if in_flap && !rig.elevons && ctrl != WingCtrl::InboardAileron { flap_tau(rig.flap.chord) } else { 0.0 },
                    ctrl_tau: match ctrl {
                        WingCtrl::Aileron => flap_tau(if rig.aileron_chord_m > 0.0 { (rig.aileron_chord_m / c).min(0.5) } else { rig.aileron_chord }),
                        WingCtrl::InboardAileron => flap_tau(rig.aileron_in.chord),
                        WingCtrl::None => 0.0,
                    },
                    flap_cm: if in_flap && !rig.elevons && ctrl != WingCtrl::InboardAileron { flap_cm(rig.flap.chord) } else { 0.0 },
                    ctrl_cm: match ctrl {
                        WingCtrl::Aileron => flap_cm(if rig.aileron_chord_m > 0.0 { (rig.aileron_chord_m / c).min(0.5) } else { rig.aileron_chord }),
                        WingCtrl::InboardAileron => flap_cm(rig.aileron_in.chord),
                        WingCtrl::None => 0.0,
                    },
                    ctrl,
                    spoiler: within(&rig.spoiler, ym),
                    load: 1.0,
                    anti: slope(rig.wing_foil.cl_a, 0.5 * ar, tan_c2, tan_le, 0.0, 0.92) / slope(rig.wing_foil.cl_a, ar, tan_c2, tan_le, 0.0, 0.92),
                });
            }
        }
        // The tail surfaces: straight-tapered, uniform strips. A fin whose
        // root is off the centreline is one of a pair.
        let mut surface = |pf: &Planform, foil: Foil2d, ctrl: &Ctrl, n: usize, fin: bool| {
            let len = (pf.tip - pf.root).length();
            if len < 0.05 || pf.chord[0] <= 0.0 {
                return;
            }
            let twin = fin && pf.root.x.abs() > 0.05;
            let s_half = 0.5 * (pf.chord[0] + pf.chord[1]) * len;
            // (A fin on a fuselage is end-plated there: twice its geometric aspect ratio, about.)
            let ar = if fin { 2.0 * len * len / s_half } else { (2.0 * len).powi(2) / (2.0 * s_half) };
            // Sweep of its leading edge and mid-chord line (the planform is given at the quarter chord).
            let dz = pf.tip.z - pf.root.z;
            let tan_le = ((dz - 0.25 * (pf.chord[1] - pf.chord[0])) / len).max(0.0);
            let tan_c2 = ((dz + 0.25 * (pf.chord[1] - pf.chord[0])) / len).abs();
            let k = 1.0 / (std::f64::consts::PI * 0.8 * ar);
            let along = (pf.tip - pf.root).normalize();
            for i in 0..n {
                let u = (i as f64 + 0.5) / n as f64;
                let c = pf.chord[0] + (pf.chord[1] - pf.chord[0]) * u;
                let inc = pf.incidence[0] + (pf.incidence[1] - pf.incidence[0]) * u;
                let at = pf.root + (pf.tip - pf.root) * u;
                let station = len * u;
                let cdir = if fin { DVec3::Z } else { DVec3::new(0.0, -inc.sin(), inc.cos()) };
                let sides: &[usize] = if fin && !twin { &[1] } else { &[0, 1] };
                for &side in sides {
                    let m = if side == 1 { DVec3::ONE } else { DVec3::new(-1.0, 1.0, 1.0) };
                    // Across the chord, in the surface's plane (as the wing's): the
                    // lift slope already has the sweep in it, and turning the flow
                    // to the swept line as well would count it twice (and stall it early).
                    let span_dir = (along - cdir * along.dot(cdir)).normalize() * m;
                    let in_ctrl = station >= ctrl.span[0] && station <= ctrl.span[1];
                    strips.push(Strip {
                        part: if fin { Part::Fin } else { Part::Tail(side) },
                        station,
                        at: at * m,
                        chord_dir: cdir,
                        // A fin's lift is positive to the left (rudder right, tail left, nose right).
                        normal: if fin { cdir.cross(span_dir).normalize() } else { tail_normal(span_dir, cdir) },
                        span_dir,
                        chord: c,
                        area: c * len / n as f64,
                        foil,
                        ar,
                        tan_le,
                        tan_c2,
                        cl_a: slope(foil.cl_a, ar, tan_c2, tan_le, 0.0, 0.92),
                        k_ind: k,
                        flap_tau: 0.0,
                        ctrl_tau: if in_ctrl { flap_tau(ctrl.chord) } else { 0.0 },
                        flap_cm: 0.0,
                        ctrl_cm: if in_ctrl { flap_cm(ctrl.chord) } else { 0.0 },
                        ctrl: WingCtrl::None,
                        spoiler: false,
                        load: 1.0,
                        anti: slope(foil.cl_a, 0.5 * ar, tan_c2, tan_le, 0.0, 0.92) / slope(foil.cl_a, ar, tan_c2, tan_le, 0.0, 0.92),
                    });
                }
            }
        };
        if !rig.elevons {
            surface(&rig.htail, rig.htail_foil, &rig.elevator, 6, false);
        }
        surface(&rig.fin, rig.fin_foil, &rig.rudder, 5, true);
        schrenk(&mut strips, |p| matches!(p, Part::Wing(_)), span);
        if !rig.elevons {
            let half = (rig.htail.tip - rig.htail.root).length() + rig.htail.root.x.abs();
            schrenk(&mut strips, |p| matches!(p, Part::Tail(_)), half);
        }
        let s_ref = spec.wing_area;
        // The parasite drag left for the fuselage, gear fairings, struts and
        // the rest, once the sections' own is counted.
        let profile: f64 = strips.iter().map(|s| s.foil.cd0 * s.area).sum();
        let cda_body = (spec.cd0 * s_ref - profile).max(0.0);
        let flap_max = rig.flap.travel[1].max(0.0);
        let tail_arm = if rig.elevons { 0.5 * mac } else { (rig.htail.root.z - mac_le.z - 0.25 * mac).max(0.5) };
        let (cg_ref, mac_ref) = if rig.cg_from_mac { (0.0, mac) } else if rig.datum > 0.0 { (rig.datum, spec.mac) } else { (spec.cg[0], spec.mac) };
        let m_peak = if spec.mcrit > 0.0 { spec.mcrit } else { 0.92 };
        let mut af = Airframe { rig: *rig, strips, s_ref, b_ref: 2.0 * span, mac, mac_le, ar, cda_body, flap_max, tail_arm, cg_ref, mac_ref, m_peak };
        if rig.cg_from_mac {
            // (cg_at measures from the chord's leading edge then)
            af.cg_ref = -mac_le.z / mac;
        }
        af
    }

    /// The centre of gravity (body frame) for a CG at `frac` of `Spec::mac`:
    /// the origin at the reference loading, moved along the body as the
    /// fuel and payload move it.
    pub fn cg_at(&self, frac: f64) -> DVec3 {
        DVec3::new(0.0, 0.0, (frac - self.cg_ref) * self.mac_ref)
    }
}

/// Schrenk's approximation of a surface's spanwise load: each strip's lift
/// is the mean of what its chord alone and an ellipse of the same area and
/// span would carry there, as a share of its own (so the total is kept).
fn schrenk(strips: &mut [Strip], of: impl Fn(Part) -> bool, half_span: f64) {
    let area: f64 = strips.iter().filter(|s| of(s.part)).map(|s| s.area).sum();
    if area <= 0.0 || half_span <= 0.0 {
        return;
    }
    let c0 = 4.0 * area / (std::f64::consts::PI * 2.0 * half_span);
    for s in strips.iter_mut().filter(|s| of(s.part)) {
        let eta = (s.at.x.abs() / half_span).min(1.0);
        let c_ell = c0 * (1.0 - eta * eta).sqrt();
        s.load = 0.5 * (1.0 + c_ell / s.chord);
    }
    // (the ellipse's area over strips at their mid-points falls a little short)
    let sum: f64 = strips.iter().filter(|s| of(s.part)).map(|s| s.area * s.load).sum();
    for s in strips.iter_mut().filter(|s| of(s.part)) {
        s.load *= area / sum;
    }
}

/// The normal of a horizontal surface's strip: perpendicular to its chord
/// and its span, on the upper side.
fn tail_normal(span_dir: DVec3, cdir: DVec3) -> DVec3 {
    let n = cdir.cross(span_dir).normalize();
    if n.y < 0.0 { -n } else { n }
}

/// What the pilot and the aircraft's state set this instant.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// airspeed (body frame, the aircraft's velocity through the air) and rotation (rad/s)
    pub vb: DVec3,
    pub omega: DVec3,
    pub rho: f64,
    pub mach: f64,
    /// the centre of gravity (body frame)
    pub cg: DVec3,
    /// the surfaces as the flight control system sets them: elevator (+
    /// nose up), aileron (+ roll right), rudder (+ nose right), -1..1
    pub elevator: f64,
    /// the trim, as elevator deflection (rad, + nose up)
    pub trim: f64,
    pub aileron: f64,
    pub rudder: f64,
    /// fixed rigging (aileron, rudder, as fractions of their travel): what
    /// the factory sets so it flies straight hands off in the cruise
    pub rigging: [f64; 2],
    /// flaps 0..1 of full, slats 0..1, speedbrake spoilers 0..1
    pub flaps: f64,
    pub slats: f64,
    pub spoilers: f64,
    /// what is left: per side (left, right) of the ailerons, flaps,
    /// elevators and stabilisers; the fin and rudder; the wing's remaining span station
    pub ail_ok: [f64; 2],
    pub flap_ok: [f64; 2],
    pub elev_ok: [f64; 2],
    pub stab_ok: [f64; 2],
    pub fin_ok: f64,
    pub rud_ok: f64,
    pub wing_span: [f64; 2],
    /// ground effect (1 clear of the ground, less near it)
    pub ge: f64,
    /// the propeller's thrust (N) and disc radius (m), for its slipstream
    pub thrust: f64,
    pub prop_r: f64,
}

/// The model's memory between steps: each strip's separation (0 attached, 1
/// separated) and the downwash on its way from the wing to the tail.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub sep: Vec<f64>,
    pub eps: f64,
}

/// What the strips add up to.
#[derive(Clone, Copy, Debug, Default)]
pub struct Output {
    /// force (body frame, N) and moment about the centre of gravity (N m)
    pub force: DVec3,
    pub moment: DVec3,
    /// the wing's lift coefficient, its centreline angle of attack, its most
    /// separated strip (0..1), and the separation's left-right difference
    pub cl_wing: f64,
    pub alpha: f64,
    pub stall: f64,
    pub stall_lr: f64,
}

impl Airframe {
    /// Forces and moments for this instant; `dt` advances the separation and
    /// the downwash (0 leaves them as they are, for trimming).
    pub fn forces(&self, inp: &Input, st: &mut State, dt: f64) -> Output {
        let rig = &self.rig;
        let v = inp.vb.length();
        let mut out = Output::default();
        if v < 0.5 {
            return out;
        }
        if st.sep.len() != self.strips.len() {
            st.sep = vec![0.0; self.strips.len()];
        }
        let vh = inp.vb / v;
        out.alpha = (-inp.vb.y).atan2(-inp.vb.z);
        let beta = (inp.vb.x / v).clamp(-1.0, 1.0).asin();
        let q = 0.5 * inp.rho * v * v;
        // The slipstream (momentum theory): the induced speed at the disc,
        // twice that far behind it, contracted to a narrower tube.
        let v_ax = (-inp.vb.z).max(0.0);
        let disc = std::f64::consts::PI * inp.prop_r * inp.prop_r;
        let (dv, r_s) = if inp.thrust > 0.0 && disc > 0.0 {
            let vi = 0.5 * (-v_ax + (v_ax * v_ax + 2.0 * inp.thrust / (inp.rho * disc)).sqrt());
            (2.0 * vi, inp.prop_r * ((v_ax + vi) / (v_ax + 2.0 * vi).max(0.1)).sqrt())
        } else {
            (0.0, inp.prop_r)
        };
        // The roll command (stick and rigging): which way each side's surfaces go.
        let roll = (inp.aileron + inp.rigging[0]).clamp(-1.0, 1.0);
        let side_roll = |side: usize| if side == 1 { -roll } else { roll };
        // Perpendicular to the flow, "down" for the downwash.
        let down = (DVec3::NEG_Y - vh * vh.dot(DVec3::NEG_Y)).normalize_or_zero();
        let (mut f, mut m) = (DVec3::ZERO, DVec3::ZERO);
        let (mut l_wing, mut s_wing) = (0.0, 0.0);
        let mut sep_side = [0.0f64; 2];
        let mut n_side = [0.0f64; 2];
        // Each strip's airflow first: (what is left of it, the flow in its
        // plane, its speed, angle of attack, dynamic pressure ratio, control
        // and flap left, arm from the CG).
        let flows: Vec<Option<StripFlow>> = self
            .strips
            .iter()
            .map(|s| {
                let (scale, ctrl_ok, flap_ok) = match s.part {
                    Part::Wing(side) => (if s.station + 0.5 * s.area / s.chord <= inp.wing_span[side] { 1.0 } else { 0.0 }, inp.ail_ok[side], inp.flap_ok[side]),
                    Part::Tail(side) => (inp.stab_ok[side], inp.elev_ok[side], 0.0),
                    Part::Fin => (inp.fin_ok, inp.rud_ok, 0.0),
                };
                if scale <= 0.0 {
                    return None;
                }
                let r = s.at - inp.cg;
                // The strip's velocity through the air around it.
                let mut fl = inp.vb + inp.omega.cross(r);
                let mut eta = 1.0;
                if !matches!(s.part, Part::Wing(_)) {
                    eta = rig.eta_tail;
                    fl -= down * (st.eps * v);
                }
                // In the slipstream: blown back, and swirled round the axis.
                if dv > 0.0 && s.at.z > rig.prop_hub.z {
                    let rel = s.at - rig.prop_hub;
                    let radial = DVec3::new(rel.x, rel.y, 0.0);
                    let d = radial.length();
                    let inside = 1.0 - smooth(0.8 * r_s, 1.2 * r_s, d);
                    if inside > 0.0 {
                        let tang = DVec3::NEG_Z.cross(radial).normalize_or_zero() * (d / r_s).min(1.0);
                        fl -= (DVec3::Z * dv + tang * (rig.swirl * dv)) * inside;
                        eta = eta + (1.0 - eta) * inside;
                    }
                }
                let fp = fl - s.span_dir * fl.dot(s.span_dir);
                let vl = fp.length();
                if vl < 0.1 {
                    return None;
                }
                let alpha = (-fp.dot(s.normal)).atan2(-fp.dot(s.chord_dir));
                Some((scale, fp, vl, alpha, eta, ctrl_ok, flap_ok, r))
            })
            .collect();
        // Where the two halves of a wing or tailplane meet the air at
        // different angles (rolling, yawing, sideslipping on dihedral or
        // sweep), that difference lifts each half as a wing of half the
        // aspect ratio would: less than what they share (DATCOM's roll damping).
        let mut alphas: Vec<f64> = flows.iter().map(|f| f.map_or(0.0, |f| f.3)).collect();
        for i in 0..self.strips.len().saturating_sub(1) {
            let pair = matches!((self.strips[i].part, self.strips[i + 1].part), (Part::Wing(0), Part::Wing(1)) | (Part::Tail(0), Part::Tail(1)));
            if pair && flows[i].is_some() && flows[i + 1].is_some() {
                let k = self.strips[i].anti;
                let (sym, dif) = (0.5 * (alphas[i] + alphas[i + 1]), 0.5 * (alphas[i + 1] - alphas[i]));
                alphas[i] = sym - k * dif;
                alphas[i + 1] = sym + k * dif;
            }
        }
        for (i, s) in self.strips.iter().enumerate() {
            let Some((scale, fp, vl, _, eta, ctrl_ok, flap_ok, r)) = flows[i] else { continue };
            let alpha = alphas[i];
            // The surface's deflection, as angle of attack.
            let (mut d_ctrl, mut flap, mut spoil, mut whole) = (0.0, 0.0, 0.0, 0.0);
            match s.part {
                Part::Wing(side) => {
                    let x = side_roll(side);
                    match s.ctrl {
                        WingCtrl::Aileron => {
                            // Right stick: the right aileron up, the left down.
                            let lock = if rig.outboard_lock { smooth(0.0, 0.1, inp.flaps) } else { 1.0 };
                            d_ctrl = lock * x * (if x > 0.0 { rig.aileron_travel[1] } else { rig.aileron_travel[0] }) + rig.aileron_droop * inp.flaps;
                            if rig.elevons {
                                d_ctrl = (d_ctrl + pitch_defl(inp, rig)).clamp(-rig.elevator.travel[0], rig.elevator.travel[1]);
                            }
                        }
                        WingCtrl::InboardAileron => d_ctrl = x * if x > 0.0 { rig.aileron_in.travel[1] } else { rig.aileron_in.travel[0] },
                        WingCtrl::None => {}
                    }
                    if s.flap_tau > 0.0 {
                        flap = inp.flaps * self.flap_max * flap_ok;
                    }
                    if s.spoiler {
                        // The speedbrake, and on the side whose aileron goes up, the roll spoilers.
                        spoil = (inp.spoilers + rig.roll_spoiler * (-x).max(0.0)).min(1.0);
                    }
                }
                Part::Tail(side) => {
                    d_ctrl = (pitch_defl(inp, rig) + rig.taileron * side_roll(side)).clamp(-rig.elevator.travel[0] - rig.taileron, rig.elevator.travel[1] + rig.taileron);
                    // A moving stabiliser's trim turns all of it, leading edge down for nose up.
                    if rig.trim_stab {
                        if s.ctrl_tau > 0.99 {
                            // A stabilator: the trim and the stick move the one surface, within its one travel.
                            d_ctrl = (d_ctrl - inp.trim).clamp(-rig.elevator.travel[0] - rig.taileron, rig.elevator.travel[1] + rig.taileron);
                        } else {
                            whole = -inp.trim;
                        }
                    }
                }
                Part::Fin => {
                    let r = (inp.rudder + inp.rigging[1]).clamp(-1.0, 1.0);
                    d_ctrl = r * if r > 0.0 { rig.rudder.travel[1] } else { rig.rudder.travel[0] };
                }
            }
            // Big deflections lose effectiveness as the flow over them
            // separates: thin-airfoil theory's angle times DATCOM's correction
            // for a plain flap (fig. 6.1.1.1-40: about 0.9 at 5°, 0.65 at 20°, 0.5 past 30°).
            // (A surface that moves whole has no hinge line to lose its flow at.)
            let all_moving = s.ctrl_tau > 0.99;
            let eff = |d: f64| if all_moving { d } else { d * (1.0 - 0.5 * smooth(0.0, 0.55, d.abs())) };
            let (da_ctrl, da_flap) = (s.ctrl_tau * eff(d_ctrl) * ctrl_ok, s.flap_tau * eff(flap));
            let da = da_ctrl + da_flap + whole;
            // (and how far back the lift those add acts, as a moment arm in chords)
            let dm = s.ctrl_cm * da_ctrl + s.flap_cm * da_flap;
            let ae = alpha + da - s.foil.a0;
            // Compressibility: the lift slope at this Mach number, and the
            // section's maximum lift falling away (the buffet boundary).
            let cl_a = if inp.mach > 0.2 { slope(s.foil.cl_a, s.ar, s.tan_c2, s.tan_le, inp.mach, self.m_peak) } else { s.cl_a };
            let m_n = inp.mach / (1.0 + s.tan_c2 * s.tan_c2).sqrt();
            let cl_a = cl_a * s.load;
            let cl_lin = cl_a * ae;
            let mut cl_max = (s.foil.cl_max + if s.flap_tau > 0.0 { rig.flap_dcl_max * flap / self.flap_max.max(0.01) } else { 0.0 }) * (1.0 - 0.3 * smooth(0.3, 0.9, m_n));
            if let Part::Wing(_) = s.part {
                cl_max += rig.slat_dcl_max * inp.slats;
            }
            // Separation: past the angle where the section reaches its maximum,
            // the flow breaks away over about 7°; it lets go quickly and
            // reattaches slowly (a few chord lengths of travel later).
            // A hinged surface deflected raises the section's maximum lift by
            // about two thirds of the lift it adds (DATCOM's plain flap; down
            // for a maximum the other way when it goes up); a surface that turns whole moves no camber.
            let d_cam = if all_moving { 0.0 } else { 0.65 * s.ctrl_tau * eff(d_ctrl) * ctrl_ok };
            let mut a_stall = cl_max / cl_a + d_cam.max(0.0);
            // (A cambered section stalls sooner upside down; a symmetric one, a tail's, alike both ways.)
            let mut a_neg = (1.0 - 0.2 * smooth(0.0, 0.03, -s.foil.a0)) * cl_max / cl_a + (-d_cam).max(0.0);
            // Under a leading-edge vortex the flow has left the edge from the
            // start and stays on: what lets go is the vortex, when it bursts.
            let vortex = rig.vortex > 0.0 && s.tan_le > 1.19;
            if vortex {
                (a_stall, a_neg) = (a_stall.max(rig.vortex_burst), a_neg.max(rig.vortex_burst));
            }
            let target = if ae >= 0.0 { smooth(a_stall - 0.02, a_stall + 0.11, ae) } else { smooth(a_neg - 0.02, a_neg + 0.11, -ae) };
            let sep = &mut st.sep[i];
            if dt > 0.0 {
                let rate = vl / s.chord * if target > *sep { 0.5 } else { 0.08 };
                *sep += (target - *sep) * (dt * rate).min(1.0);
            }
            let sp = *sep;
            let alpha_g = alpha + 0.5 * da;
            let cl_fp = 2.0 * alpha_g.sin() * alpha_g.cos() * 0.95;
            let mut cl = cl_lin * (1.0 - sp) + cl_fp * sp;
            // A spoiler up spoils most of the lift behind it (its own wake: not the
            // wing's stall, which comes at the angle it would anyway).
            if spoil > 0.0 {
                cl *= 1.0 - 0.75 * spoil;
            }
            // A sharp, highly swept leading edge rolls its flow up into a
            // vortex whose suction lifts too (Polhamus), until it bursts; the
            // leading edge's suction is lost with it, so it drags as it lifts.
            let mut cd_v = 0.0;
            if rig.vortex > 0.0 && s.tan_le > 1.19 {
                let a = alpha_g;
                let cl_v = rig.vortex * a.sin().powi(2) * a.cos() * a.signum() * (1.0 - smooth(rig.vortex_burst - 0.05, rig.vortex_burst + 0.15, a.abs())) * (1.0 - smooth(1.0, 1.6, inp.mach));
                cl += cl_v;
                cd_v = cl_v * a.sin();
            }
            let mut kind = s.k_ind;
            if let Part::Wing(side) = s.part {
                // Ground effect: more lift close to it, the more so the smaller the
                // aspect ratio (a slender delta's cushion is three times a glider's),
                // and felt higher up than the induced drag's relief (`ge` is that:
                // McCormick's (16h/b)² / (1 + (16h/b)²); the lift's goes as 6h/b,
                // about 9 % for a light single at h/b 0.1, 20 % for a slender delta at 0.15).
                let r = 0.375 * (inp.ge / (1.0 - inp.ge).max(1e-6)).sqrt();
                cl *= 1.0 + 0.12 * (7.5 / self.ar).clamp(1.0, 3.0) / (1.0 + r * r);
                kind *= inp.ge;
                l_wing += cl * s.area;
                s_wing += s.area;
                sep_side[side] += sp * s.area;
                n_side[side] += s.area;
                out.stall = out.stall.max(sp);
            }
            // (A deflected surface's own profile drag; the flaps' is in the `Spec`'s cd_flap, the speedbrake's in its spoiler_cd.)
            let roll_spoil = if s.spoiler { (spoil - inp.spoilers).max(0.0) } else { 0.0 };
            let cd = s.foil.cd0 + kind * cl * cl + cd_v + sp * 1.8 * alpha_g.sin().powi(2) + 0.25 * (s.ctrl_tau * eff(d_ctrl)).powi(2) + 0.5 * rig.spoiler.chord * (roll_spoil * rig.spoiler.travel[1]).sin().powi(2);
            // The camber a deflection adds pitches the section nose down; once
            // separated, the centre of pressure moves back toward mid-chord;
            // past the speed of sound, where the leading edge is outside the
            // Mach cone, it moves there too.
            let b_sup = (inp.mach * inp.mach - 1.0).max(0.0).sqrt();
            // (Behind a subsonic leading edge, less than half as far: linear theory puts a
            // slender delta's centre at two thirds of its root chord, DATCOM's
            // fig. 4.1.4.2, against about 0.55 below the speed of sound.)
            let ac_aft = 0.25 * smooth(0.95, 1.3, inp.mach) * (0.4 + 0.6 * smooth(0.6, 1.2, b_sup / s.tan_le.max(0.05)));
            let cm = (s.foil.cm0 - cl_a * dm - ac_aft * cl_lin) * (1.0 - sp) - sp * 0.25 * cl_fp;
            let ql = 0.5 * inp.rho * vl * vl * eta * scale;
            let fh = fp / vl;
            let lift_dir = (s.normal - fh * s.normal.dot(fh)).normalize_or_zero();
            let force = lift_dir * (ql * s.area * cl) - fh * (ql * s.area * cd);
            f += force;
            m += r.cross(force) + s.normal.cross(s.chord_dir) * (ql * s.area * s.chord * cm);
        }
        out.cl_wing = if s_wing > 0.0 { l_wing / s_wing } else { 0.0 };
        out.stall_lr = sep_side[1] / n_side[1].max(1e-6) - sep_side[0] / n_side[0].max(1e-6);
        // The downwash follows the wing's lift after the air has crossed the tail arm.
        let eps_target = rig.downwash * 2.0 * out.cl_wing / (std::f64::consts::PI * self.ar);
        if dt > 0.0 {
            st.eps += (eps_target - st.eps) * (dt * v / self.tail_arm).min(1.0);
        }
        // The fuselage's share, and the drag of everything not a surface.
        let fz = &rig.fuse;
        let (sq, c, b) = (q * self.s_ref, self.mac, self.b_ref);
        let a_body = out.alpha.clamp(-0.5, 0.5);
        let b_body = beta.clamp(-0.5, 0.5);
        let lift_dir = DVec3::X.cross(vh).normalize_or_zero();
        // The body's lift (a chine's, a lifting fuselage's) sheds its vortices
        // over the same span as the wing's: the induced drag goes as the square
        // of the two together, so it adds its own share and the cross term.
        let cl_b = fz.cl_a * out.alpha.sin() * out.alpha.cos();
        let cdi_b = inp.ge * cl_b * (2.0 * out.cl_wing + cl_b) / (std::f64::consts::PI * rig.oswald * self.ar);
        f += DVec3::X * (sq * fz.cy_b * b_body) - vh * (q * self.cda_body + sq * cdi_b) + lift_dir * (sq * cl_b);
        m += DVec3::new(sq * c * fz.cm_a * a_body, -sq * b * fz.cn_b * b_body, -sq * b * fz.cl_b * b_body);
        out.force = f;
        out.moment = m;
        out
    }
}
