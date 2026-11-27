//! Lockheed C-130H Hercules: the four-turboprop tactical transport, Allison
//! T56-A-15 engines (4,591 shp) turning four-blade Hamilton Standard 54H60
//! propellers (13 ft 6 in), the analog flight deck. Every dimension the
//! exterior, the flight deck, the flight model and the paint share lives in
//! `dims.rs` (the dimensional contract, with its sources and which figures
//! are estimates); `flight.rs` holds the `Spec`, `deck.rs` the flight deck,
//! `paint.rs` the paint schemes.
//!
//! The exterior is built here, one file per part: `fuselage.rs` (the skin
//! with the radome, the flight deck's shell and glass, the wing-root
//! fairing, the sponsons, the doors, the cargo ramp and aft door, the cargo
//! compartment's lining, antennas, probes and lights), `wing.rs` (wings,
//! Fowler flaps, ailerons and tabs, lights, the inner structure),
//! `engines.rs` (the T56 nacelles, the 54H60 propellers, the external
//! tanks), `tail.rs` and `gear.rs`.
//!
//! # References (exterior; fetched 2026-11)
//!
//! The numbers are `dims.rs`'s; the shapes were drawn against:
//! - The C-130J pocket guide's general arrangement (same nose, wing, tail
//!   and gear as the H): https://www.lockheedmartin.com/content/dam/lockheed-martin/aero/documents/C-130J/C130JPocketGuide.pdf
//! - Wikimedia Commons three-view: https://commons.wikimedia.org/wiki/File:Lockheed_C-130H_Hercules_Line_Drawing.svg
//! - Photographs (Wikimedia Commons): JASDF C-130H 85-1079 side view
//!   https://commons.wikimedia.org/wiki/File:Lockheed_C-130H_Hercules_%E2%80%9885-1079_079%E2%80%99_(33890468248).jpg ;
//!   the nose and radome https://commons.wikimedia.org/wiki/File:JASDF_C-130H(05-1085)_radome_%26_nose_landing_gear_right_side_view_at_Iruma_Air_Base_November_3,_2014.jpg ;
//!   the nose head-on https://commons.wikimedia.org/wiki/File:JASDF_KC-130H(85-1080)_fuselage_section_front_view_at_Komaki_Air_Base_March_2,_2019.jpg ;
//!   a USAF C-130H in profile https://commons.wikimedia.org/wiki/File:Lockheed_C-130H_Hercules_-_ANG_46707_-_Stuttgart_-_20090909-01.jpeg ;
//!   the nacelle https://commons.wikimedia.org/wiki/File:JASDF_KC-130H(85-1080)_T56-A-15_turboprop_engine_nacelle(right_wing,_outside)_right_front_view_at_Komaki_Air_Base_March_2,_2019.jpg ;
//!   the main gear and sponsons https://commons.wikimedia.org/wiki/File:JASDF_KC-130H(85-1080)_right_main_landing_gear_right_front_view_at_Komaki_Air_Base_March_2,_2019.jpg
//!   and https://commons.wikimedia.org/wiki/File:JASDF_C-130H(75-1078)_Left_Main_Landing_Gear_at_Komaki_Air_Base_March_13,2016.JPG ;
//!   the propellers and spinner https://commons.wikimedia.org/wiki/File:Defense.gov_News_Photo_110518-F-HG907-924_-_Staff_Sgt._Christopher_Rhodus_performs_an_inspection_on_a_C-130_Hercules_engine_during_a_mobility_air_forces_exercise_at_the_Nevada_Test_and.jpg
//!   and https://commons.wikimedia.org/wiki/File:C-130_wing_2006-06-05_F-0994L-526.jpg ;
//!   the flaps from behind https://commons.wikimedia.org/wiki/File:A_U.S._Air_Force_crewman_from_the_153rd_Air_Wing_Air_National_Guard,_Cheyenne,_Wyo.,_walks_the_wing_of_a_C-130_Hercules_aircraft_prior_to_take_off_.jpg ;
//!   a USAF C-130H taking off https://commons.wikimedia.org/wiki/File:142d_Airlift_Squadron_-_Lockheed_C-130H-LM_Hercules_84-0210.jpg ;
//!   the ramp lowered, flaps down https://commons.wikimedia.org/wiki/File:A_view_of_a_C-130_Hercules_aircraft_from_the_459th_Military_Air_Wing,_as_seen_from_the_top_of_an_ambulance_parked_in_the_foreground,_during_Exercise_Reforger_%2781_-_DPLA_-_18e70fbfa593760eef1b4990c13ae605.jpeg
//! - Shapes read off those pictures are ESTIMATES (marked [est] where they
//!   are set): no dimensioned drawing of the nose, nacelles, fairings,
//!   doors or gear was found.
//!
//! # Surfaces for the paint (plane.wgsl)
//!
//! Patterns (vertex `mat.y`) and their uv:
//! - `pat::FUSE`: the fuselage skin, sponsons, the flight deck section, the
//!   doors, ramp and aft door's outer skins. uv = (body z, arc length round
//!   the section from the crown, m; negative on the left). The sponsons
//!   and the main gear doors: (body z, 0..1 round their section from the
//!   top, `dims::spon_pt`'s t; the sponsons' negative on the left). Cut by `window_c130`
//!   and `well_c130`.
//! - no pattern (0): the radome (composite: no rivets) and small parts.
//! - `pat::WING`: wing skins incl. the centre section over the fuselage.
//!   uv = (span station m, chord position m: + upper surface, − lower).
//! - `pat::CTRL`: flaps, ailerons, elevators, rudder, tabs (same uv as WING).
//! - `pat::TAIL`: fin, dorsal fillet, tailplane (uv as WING, the fin's
//!   "span" is its height above the crown).
//! - `pat::NACELLE`: nacelles, pylons and external tanks. uv = (distance aft
//!   of the propeller plane (tank: of its nose), arc length round from the
//!   top, m).
//! - `pat::LINING`: the cargo compartment's lining; `pat::NONSLIP`: the
//!   cargo floor and the ramp's floor; `pat::TREAD`: tyres.
//!
//! Materials: `mat::LIVERY` for everything painted in the scheme
//! (including the painted-over screw heads along the leading-edge sections,
//! spars and the radome's joint, which are small domes, no pattern);
//! `mat::PROP` for the propeller blades' painted grey section (uv.x is
//! r × 0.4 so the shader's light-aircraft tip band stays off: the 54H60's
//! black root cuffs and black tips are geometry of their own, `mat::PAINT`);
//! `mat::EXHAUST` the jet pipes; `mat::LAMP` lenses (`lamp::*`);
//! `mat::SATIN` wells, primer, interior; `mat::STEEL`/`ALU` gear and
//! fittings; `mat::RUBBER` tyres; `mat::GLASS` the flight deck glazing.
//!
//! # Body frame
//!
//! +X right, +Y up, -Z forward; the origin is the reference centre of
//! gravity, at 25% of the mean aerodynamic chord, `dims::ZN` behind the nose
//! and `dims::YG` above the ground at rest on the gear. Drawing positions are
//! given as `d`, the distance back from the nose, and `h`, the height above
//! the ground (`dims::bz`, `dims::by`).
//!
//! # Simulator simplifications
//!
//! - The four T56 turboprops fly as the game's `Engine::Fan` (four of them):
//!   static thrust, a thrust lapse with Mach number standing in for a
//!   propeller's thrust falling with speed (`mach_lapse`), `reverse` for the
//!   propellers' reverse pitch, and the "fan" speed being the propeller's
//!   (`n1_rpm` 1020, the 54H60's governed speed), so `Input::Prop` turns
//!   the blades and the blur disc shows at the right time. There is no
//!   propeller governor, beta range, torque or engine-out yaw from a
//!   windmilling propeller: the shared physics has none. The blades are
//!   modelled at one fixed pitch (about 30° at the tip); they do not
//!   feather or go to reverse.
//! - The canopy key works the cargo ramp and the aft cargo door
//!   (`Input::Canopy`), in the two seconds a canopy takes rather than the
//!   real ramp's twenty or so.
//! - The crew entrance door and the two paratroop doors are pieces of their
//!   own with their real hinges and travel, but nothing drives them yet (no
//!   `Input` for them): they stay shut.
mod engines;
mod fuselage;
mod gear;
mod tail;
mod wing;

use crate::airframe::*;
use glam::{Quat, Vec2, Vec3};
use std::f32::consts::PI;

pub mod deck;
pub mod dims;
mod flight;
pub mod paint;
pub use flight::SPEC;

use dims::*;

// ---------------------------------------------------------------- materials

/// The paint: the shader paints `mat::LIVERY` (plane.wgsl `livery_c130`).
const SKIN: Mat = mt(mat::LIVERY, 0.20, 0.22, 0.22);
/// Wells, bays and the insides of doors: primer.
const WELL_MAT: Mat = mt(mat::SATIN, 0.30, 0.31, 0.29);
const DARK: Mat = mt(mat::BLACK, 0.02, 0.02, 0.022);
/// The landing gear's paint (gloss light grey).
const STRUT: Mat = mt(mat::PAINT, 0.55, 0.56, 0.56);
const EXHAUST_MAT: Mat = mt(mat::EXHAUST, 0.30, 0.28, 0.26);
const GLASS_MAT: Mat = mt(mat::GLASS, 0.1, 0.1, 0.1);
const HOSE: Mat = mt(mat::BLACK, 0.03, 0.03, 0.032);
/// Flat black paint (propeller cuffs and tips, the spinners, de-ice fairings).
const FLAT_BLACK: Mat = mt(mat::PAINT, 0.018, 0.018, 0.02);

// ---------------------------------------------------------------- the model

pub fn build() -> Model {
    let mut b = Builder::default();
    fuselage::fuselage(&mut b);
    fuselage::sponsons(&mut b);
    fuselage::flight_deck(&mut b);
    wing::wings(&mut b);
    engines::engines(&mut b);
    tail::tail(&mut b);
    gear::gear(&mut b);
    bake_ao(&mut b.pieces, &AoBake { vox: 0.12, reach: 10.0, rays: 18, open: &|q, pat| window(q) < 0.0 || (pat != pat::LINING && well(q) < 0.0) });
    // The flight deck gets its own finer bake, shut in by the nose's skin.
    let mut d = Builder::default();
    deck::build(&mut d);
    let n_deck = d.pieces.len();
    if let Some(k) = b.pieces.iter().position(|p| p.name == "flight deck section") {
        d.fixed("skin (occluder)", G_ENGINE, b.pieces[k].mesh.clone());
    }
    bake_ao(&mut d.pieces, &AoBake { vox: 0.03, reach: 3.0, rays: 32, open: &|q, _| window(q) < 0.0 });
    let off = b.pieces.len();
    for mut p in d.pieces.into_iter().take(n_deck) {
        p.parent = p.parent.map(|i| i + off);
        b.pieces.push(p);
    }
    // The doors that hang open in the air most of the time (the main gear's
    // and the nose gear's aft doors) were baked shut against the airframe.
    for p in b.pieces.iter_mut().filter(|p| p.name.ends_with("gear door") || p.name.contains("aft nose door")) {
        for v in p.mesh.v.iter_mut().filter(|v| v.mat[0] == mat::LIVERY) {
            v.col[3] = v.col[3].max(0.85);
        }
    }
    Model::finish(b.pieces)
}

// ---------------------------------------------------------------- shared helpers

/// Mirror a right-side mesh to the left, winding its triangles back.
fn mirrored(m: &Mesh) -> Mesh {
    let mut l = m.mirror_x();
    for t in l.i.chunks_mut(3) {
        t.swap(1, 2);
    }
    l
}

/// The right half and its mirror image, the left's uv.y negative (the
/// fuselage skin convention: arc length round from the crown).
fn both_sides(r: Mesh) -> Mesh {
    let mut l = mirrored(&r);
    for v in &mut l.v {
        v.uv[1] = -v.uv[1];
    }
    let mut m = r;
    m.append(&l);
    m
}

/// A grid over (station, angle round the section) whose points come from
/// `pt(z, th)`, th from `th0(z)` to `th1(z)` (0 the crown, PI the keel),
/// right side and its mirror; uv = (z, arc length from the crown).
/// `inset` moves it in along the normal, `flip` turns its faces in.
#[allow(clippy::too_many_arguments)]
fn section_grid(ds: &[f32], nt: usize, pt: &dyn Fn(f32, f32) -> Vec3, th0: &dyn Fn(f32) -> f32, th1: &dyn Fn(f32) -> f32, inset: f32, flip: bool, m: &Mat) -> Mesh {
    let rows: Vec<Vec<SP>> = ds
        .iter()
        .map(|&d| {
            let z = bz(d);
            let (a, c) = (th0(z), th1(z));
            // arc length from the crown to th0 first, so the uv is the same as the full skin's.
            let mut arc = 0.0;
            let steps = 48;
            let mut prev = pt(z, 0.0);
            for k in 1..=steps {
                let q = pt(z, a * k as f32 / steps as f32);
                arc += (q - prev).length();
                prev = q;
            }
            let mut prev = pt(z, a);
            (0..=nt)
                .map(|k| {
                    let th = a + (c - a) * k as f32 / nt as f32;
                    let p = pt(z, th);
                    arc += (p - prev).length();
                    prev = p;
                    let q = if inset != 0.0 {
                        let e = 1e-3;
                        let du = pt(z + e, th) - pt(z - e, th);
                        let dv = pt(z, th + e) - pt(z, (th - e).max(0.0));
                        p - du.cross(dv).normalize_or_zero() * inset
                    } else {
                        p
                    };
                    (q, [z, arc], 0.0)
                })
                .collect()
        })
        .collect();
    let mut r = Mesh::default();
    r.grid(m, &rows, false, flip);
    both_sides(r)
}

/// Stations from d0 to d1, `step(d)` apart.
fn stations(d0: f32, d1: f32, step: &dyn Fn(f32) -> f32) -> Vec<f32> {
    let mut ds = Vec::new();
    let mut d = d0;
    while d < d1 - 1e-4 {
        ds.push(d);
        d += step(d).max(0.01);
    }
    ds.push(d1);
    ds
}

/// Point and outward normal on a foil.
fn foil_frame(f: &Foil, s: f32, xc: f32, side: f32) -> (Vec3, Vec3) {
    let p = f.pt(s, xc, side, -1.0);
    let ds = f.pt(s + 0.01, xc, side, -1.0) - f.pt(s - 0.01, xc, side, -1.0);
    let dx = f.pt(s, xc + 0.002, side, -1.0) - f.pt(s, (xc - 0.002).max(0.0), side, -1.0);
    let n = (ds.cross(dx) * side).normalize_or_zero();
    (p, n)
}

/// A static discharger: a thin composite rod trailing aft from a trailing
/// edge, its brush tip at the end.
fn wick(m: &mut Mesh, at: Vec3, dir: Vec3) {
    let d = dir.normalize();
    m.rbox(&mt(mat::SATIN, 0.5, 0.5, 0.5), at + d * 0.01, Vec3::new(0.012, 0.006, 0.03), 0.004, Quat::from_rotation_arc(Vec3::Z, d));
    m.tube(&DARK, &[at - d * 0.02, at + d * 0.17], &|t| 0.0065 * (1.0 - 0.45 * t), 6, true);
    m.tube(&mt(mat::SATIN, 0.35, 0.35, 0.36), &[at + d * 0.17, at + d * 0.205], &|_| 0.0022, 4, true);
}

/// Painted-over screw heads: the scheme's paint (`mat::LIVERY`), so they
/// take each livery's colour and show only as small domes.
const PAINTED_SCREW: Mat = mt(mat::LIVERY, 0.20, 0.22, 0.22);
/// A row of screws along a polyline on a surface (points and normals).
fn screw_row(m: &mut Mesh, pts: &[(Vec3, Vec3)], r: f32, seed: f32) {
    for (k, (p, n)) in pts.iter().enumerate() {
        m.screw(&PAINTED_SCREW, *p + *n * 0.0008, *n, r, seed + k as f32 * 1.37);
    }
}

/// A blade antenna standing on a surface at `p` (normal `n`), its chord along `fwd`.
fn blade_antenna(m: &mut Mesh, p: Vec3, n: Vec3, fwd: Vec3, h: f32, c: f32, mat: &Mat) {
    let n = n.normalize();
    let ax = (-fwd - n * (-fwd).dot(n)).normalize();
    let side = n.cross(ax);
    let pts = [Vec2::new(-c * 0.5, 0.0), Vec2::new(c * 0.5, 0.0), Vec2::new(c * 0.15, h), Vec2::new(-c * 0.12, h)];
    m.slab(mat, &pts, &[], p - n * 0.01 + side * 0.009, ax, n, 0.018);
    // its base plate and screws
    m.rbox(mat, p, Vec3::new(0.025, 0.006, c * 0.55).max(Vec3::splat(0.006)), 0.004, Quat::from_mat3(&glam::Mat3::from_cols(side, n, ax)));
}

/// Pieces whose name is one of a left/right pair (the side's index).
#[cfg(test)]
fn side_of(name: &str) -> Option<usize> {
    if name.starts_with("left ") {
        Some(0)
    } else if name.starts_with("right ") {
        Some(1)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_builds_whole_and_to_size() {
        let t0 = std::time::Instant::now();
        let m = build();
        let (c, h) = m.bounds();
        let nv: usize = m.pieces.iter().map(|p| p.mesh.v.len()).sum();
        for p in &m.pieces {
            assert!(!p.mesh.i.is_empty(), "{} is empty", p.name);
            assert!(p.mesh.v.iter().all(|v| v.pos.iter().chain(v.nrm.iter()).all(|x| x.is_finite())), "{} has NaNs", p.name);
        }
        let len = h.z * 2.0;
        let span = h.x * 2.0;
        let top = c.y + h.y + YG;
        println!("C-130: {} pieces, {nv} vertices, {:.1} s; length {len:.2} m, span {span:.2} m, height {top:.2} m", m.pieces.len(), t0.elapsed().as_secs_f64());
        let mut big: Vec<(usize, &str)> = m.pieces.iter().map(|p| (p.mesh.i.len() / 3, p.name)).collect();
        big.sort_by(|a, b| b.0.cmp(&a.0));
        println!("largest pieces (triangles): {:?}", &big[..12]);
        assert!((len - LENGTH).abs() < 0.3, "{len}");
        assert!((span - B).abs() < 0.3, "{span}");
        assert!((top - HEIGHT).abs() < 0.3, "{top}");
    }

    /// The pieces the damage model and animation key off are all there.
    #[test]
    fn pieces_have_their_roles() {
        let m = build();
        let role = |n: &str| m.pieces.iter().find(|p| p.name == n).map(|p| p.role).unwrap_or_else(|| panic!("no {n}"));
        assert_eq!(role("tail cone"), Role::TailCone);
        assert_eq!(role("right aileron"), Role::Aileron(1));
        assert_eq!(role("left inboard flap"), Role::Flap(0));
        assert_eq!(role("right outboard flap"), Role::Flap(1));
        assert_eq!(role("rudder"), Role::Rudder);
        assert_eq!(role("left elevator"), Role::Elevator);
        assert_eq!(role("fin"), Role::Stab);
        assert_eq!(role("stabiliser"), Role::Stab);
        assert_eq!(role("tail structure"), Role::TailStructure);
        assert_eq!(role("left wing structure"), Role::WingStructure(0));
        assert_eq!(role("right outboard propeller"), Role::Prop);
        assert_eq!(role("left inboard spinner"), Role::Prop);
        assert_eq!(role("nose wheels"), Role::Gear(0));
        assert_eq!(role("right aft main wheel"), Role::Gear(4));
        assert_eq!(role("left forward main strut"), Role::Gear(1));
        for n in ["radome", "fuselage", "flight deck section", "sponsons", "left wing", "right wing", "cargo ramp", "aft cargo door", "left external tank", "right inboard nacelle"] {
            role(n);
        }
        assert!(m.pieces.iter().any(|p| p.glass && p.name == "windows"));
        // Left and right pieces belong to their side's wing where they are on a wing.
        for p in &m.pieces {
            if let Some(s) = side_of(p.name) {
                if p.group == G_LWING || p.group == G_RWING {
                    assert_eq!(p.group, [G_LWING, G_RWING][s], "{}", p.name);
                }
            }
        }
    }

    fn posed(m: &Model, gear: f64, flaps: f64, canopy: f64) -> Vec<glam::Mat4> {
        let mut p = crate::plane::Plane::spawn(&SPEC, 0.6, -2.1, 1000.0, 0.0, 120.0);
        p.gear = gear;
        p.flaps = flaps;
        p.canopy = canopy;
        p.pose(&m.pieces)
    }
    fn world_box(m: &Model, pose: &[glam::Mat4], name: &str) -> (Vec3, Vec3) {
        let i = m.pieces.iter().position(|p| p.name == name).unwrap_or_else(|| panic!("no {name}"));
        m.pieces[i].mesh.v.iter().map(|v| pose[i].transform_point3(Vec3::from(v.pos))).fold((Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)), |(lo, hi), q| (lo.min(q), hi.max(q)))
    }

    /// The gear stows inside the airframe and stands on the ground; the
    /// main gear door is flush with the sponson stowed and clear of the
    /// tyres down.
    #[test]
    fn gear_moves_where_it_should() {
        let m = build();
        let down = posed(&m, 1.0, 0.0, 0.0);
        let up = posed(&m, 0.0, 0.0, 0.0);
        for n in ["right forward main wheel", "right aft main wheel"] {
            let (lo, hi) = world_box(&m, &down, n);
            println!("{n} down: {lo:.2} .. {hi:.2}");
            assert!((lo.y - (by(0.0) + MAIN_SQUAT)).abs() < 0.03, "{n} is not on the ground (fully extended) {lo}");
            let (lo, hi) = world_box(&m, &up, n);
            println!("{n} up: {lo:.2} .. {hi:.2}");
            assert!(lo.y > by(SPON_BOT_H) + 0.05 && hi.x < SPON_XO - 0.1, "{n} stowed outside the sponson");
        }
        let (lo, _) = world_box(&m, &down, "nose wheels");
        assert!((lo.y - (by(0.0) + NOSE_SQUAT)).abs() < 0.03, "nose wheels {lo}");
        let (lo, hi) = world_box(&m, &up, "nose wheels");
        println!("nose wheels up: {lo:.2} .. {hi:.2}");
        assert!(lo.y > sec((lo.z + hi.z) * 0.5).bot - 0.02 && hi.y < DECK_FLOOR_Y && lo.z > NOSE_WELL[0] - 0.05 && hi.z < NOSE_WELL[1] + 0.05, "nose wheels stowed outside their well");
        let (lo, _) = world_box(&m, &down, "right main gear door");
        let (_, whi) = world_box(&m, &down, "right forward main wheel");
        assert!(lo.x > whi.x + 0.02 && lo.y > by(0.0) + 0.15, "main gear door hits the tyres or the ground: door {lo}, tyre {whi}");
    }

    /// The ramp's toe reaches the ground open, and the aft door stays inside the tail.
    #[test]
    fn ramp_and_door_open_inside_their_room() {
        let m = build();
        let open = posed(&m, 1.0, 0.0, 1.0);
        let (lo, _) = world_box(&m, &open, "cargo ramp");
        println!("ramp open: lowest {:.2} m above the ground", lo.y - by(0.0));
        assert!(lo.y - by(0.0) > -0.05 && lo.y - by(0.0) < 0.25);
        let i = m.pieces.iter().position(|p| p.name == "aft cargo door").unwrap();
        let (mut worst, mut at) = (f32::MIN, Vec3::ZERO);
        for v in &m.pieces[i].mesh.v {
            let q = open[i].transform_point3(Vec3::from(v.pos));
            let s = sec(q.z);
            let w = half_width_at(q.z, q.y).unwrap_or(0.0);
            let e = (q.y - s.top).max(q.x.abs() - w);
            if e > worst {
                (worst, at) = (e, q);
            }
        }
        println!("aft door open: {worst:.3} m past the skin at worst, at d {:.2} h {:.2} x {:.2}", at.z + ZN, at.y + YG, at.x);
        assert!(worst < 0.02);
    }

    /// The Fowler flaps run back and down; full travel puts them behind the wing.
    #[test]
    fn flaps_run_aft_and_down() {
        let m = build();
        let up = posed(&m, 1.0, 0.0, 0.0);
        let full = posed(&m, 1.0, 1.0, 0.0);
        for n in ["right inboard flap", "right outboard flap"] {
            let (a0, a1) = world_box(&m, &up, n);
            let (b0, b1) = world_box(&m, &full, n);
            println!("{n}: up z {:.2}..{:.2} y {:.2}..{:.2}; full z {:.2}..{:.2} y {:.2}..{:.2}", a0.z, a1.z, a0.y, a1.y, b0.z, b1.z, b0.y, b1.y);
            assert!(b1.z > a1.z + 0.4 && b0.y < a0.y - 0.4);
            // ...and clear the external tank under them.
            let i = m.pieces.iter().position(|p| p.name == n).unwrap();
            let c = ext_tank_nose();
            let mut worst = f32::MAX;
            for v in &m.pieces[i].mesh.v {
                let q = full[i].transform_point3(Vec3::from(v.pos));
                let u = (q.z - c.z) / EXT_TANK_L;
                if (0.0..1.0).contains(&u) {
                    worst = worst.min(Vec2::new(q.x - c.x, q.y - c.y).length() - EXT_TANK_R * engines::tank_r(u));
                }
            }
            println!("{n} at full travel: {worst:.2} m clear of the external tank");
            assert!(worst > 0.0);
        }
    }
}
