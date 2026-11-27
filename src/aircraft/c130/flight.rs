//! The C-130H's `Spec`: mass, engines, aerodynamics, limits and handling,
//! tuned against published performance (the tests `performance_is_a_c130h`,
//! `a_c130h_takes_off_and_climbs` and `a_c130h_lands_short_with_its_propellers_reversed`
//! in plane.rs check it). Geometry comes from `dims` (contacts, gear, eye, tail
//! break points); see there for the airframe's sources.
//!
//! Tags: [2+] two or more sources agree, [1] one source, [est] an estimate
//! (mine, from the physics or from a planning rule), [anec] anecdotal.
//!
//! # Sources
//!
//! - S1 USAF C-130 fact sheet via Wikipedia's C-130H specifications,
//!   https://en.wikipedia.org/wiki/Lockheed_C-130_Hercules (and
//!   https://www.af.mil/About-Us/Fact-Sheets/Display/Article/1555054/c-130-hercules/):
//!   max speed 320 kt at 20,000 ft, cruise 292 kt, ceiling 33,000 ft empty,
//!   23,000 ft with a 42,000 lb payload, climb 1,830 ft/min, take-off
//!   distance 3,586 ft at 155,000 lb, 1,400 ft at 80,000 lb, range 2,050 nm,
//!   ferry 3,995 nm, empty 75,800 lb, T56-A-15 4,590 shp.
//! - S2 https://www.aerospaceweb.org/aircraft/transport-m/c130 : climb 1,900
//!   ft/min, empty 75,745 lb, normal take-off 155,000 lb, overload 175,000 lb,
//!   payload 42,580 lb, fuel 44,240 lb internal + 18,160 lb external.
//! - S3 an unofficial crew study sheet of TO 1C-130H-1 limits,
//!   https://www.baseops.net/wp-content/uploads/2015/08/c130h1_Ops_Limits-1.pdf :
//!   Mach 0.64 max; flaps 50% 180 kt, 100% 145 kt (230 - flap% = speed);
//!   gear and lights 165 kt; ramp and door 150 kt; painted control surfaces
//!   250 KIAS; thunderstorm penetration at most 180 kt, 65 kt above the
//!   power-off stall; max bank 60° (45° with flaps); torque 19,600 in-lb;
//!   nose wheel steering 60° (20° above 155,000 lb); weights 155,000 /
//!   175,000 / landing 130,000 lb; CG 15-30% MAC.
//! - S4 unofficial TOLD study notes from TO 1C-130H-1-1,
//!   https://www.baseops.net/wp-content/uploads/2015/08/TOLD_Gouge_Jan06.doc
//!   and the "Brain Book" (C-130H2 gouge,
//!   https://www.baseops.net/wp-content/uploads/2015/08/BrainBookACQ.pdf):
//!   normal threshold speed 1.35 x the power-off stall, max-effort 1.28,
//!   touchdown 1.2; minimum threshold 106.5 KIAS, minimum touchdown 97;
//!   approach = threshold + 10; take-off speed 1.2 x the power-ON stall
//!   (50% flaps); 50% flaps to take off, 100% to land; climb below 10,000 ft
//!   at 180 KIAS (also S6); cruise 280 KTAS on long legs; load factor -1 to
//!   +3 g (0 to 2.5 with more than 25,000 lb of cargo); max-effort landing
//!   at most 130,000 lb and 540 fpm; ground time 50 lb of fuel a minute;
//!   gear retraction about 18 s.
//! - S5 "C-130 Discussion Items" (USAF test-data paper,
//!   https://www.baseops.net/wp-content/uploads/2015/08/C-130-Discussion-Items-2009Mar.pdf):
//!   power-off stalls benign, buffet 10 kt before the stall clean, 5-6 kt at
//!   50% flaps, 2-3 kt at 100%; power ON adds about 12 kt of margin (blown
//!   lift) but the left wing then drops hard.
//! - S6 AOPA Pilot, January 2017,
//!   https://www.aopa.org/news-and-media/all-news/2017/january/pilot/c130-hercules :
//!   4,910 eshp, 1,021 rpm props, max torque 19,600 in-lb, 180 kt climb,
//!   "airspeed redline of 320 knots", boosted controls "lighter than many
//!   lightplanes", 60° banks routine, reverse gives "incredible stopping power".
//! - S7 NAVFAC aircraft characteristics (archived uscost.net page,
//!   http://www.uscost.net/aircraftcharacteristics/acc130.htm): take-off
//!   1,400 ft at 80,000 lb, landing 1,150 ft at 80,000 lb and 2,020 ft at the
//!   maximum landing weight.
//!
//! - S8 CGTO 1C-130-1, USCG C-130 flight manual (15 December 2000, change 1
//!   July 2001; built on T.O. 1C-130(H)-1, performance in T.O. 1C-130H-1-1),
//!   https://uniforce-sog.org/wp-content/uploads/2024/07/USCG-Lockheed-C-130-Flight-Manual.pdf :
//!   section V "Airspeed limitations" and figure 5-5 sheet 1 "Limit Flight
//!   Speed vs Altitude Chart": maximum speed VD 320 KIAS from sea level to
//!   15,000 ft, Mach 0.64 above; recommended speed VH about 270 KIAS for the
//!   lightest loadings (area A), less for heavier ones; flap speeds 10% 220,
//!   50% 180, 100% 145 KIAS; gear 165; ramp (or ramp and door) open 150,
//!   cargo door only 185; landing lights 250 KIAS. The airspeed indicator's
//!   banded pointer shows the maximum allowable speed.
//! - S9 the "Brain Book" (S4) limits page: "Max Speed 0.64 Mach, 15,000
//!   316 KIAS, 20,000 290 KIAS".
//!
//! Not found in any public source: the Dash-1 stall speed charts, VMCA, the
//! T56's power at altitude,
//! the propellers' static thrust, inertias. Those below are estimates.
//!
//! # Mass, fuel and balance
//!
//! `Spec::mass` (tanks half full) makes the maximum normal take-off weight,
//! 155,000 lb (70,307 kg), with all 62,400 lb (28,300 kg) of fuel on board
//! (S2: internal plus the two 1,360 US gal external tanks): operating empty
//! 75,860 lb (34,410 kg; 75,745-75,800 lb quoted [2+]), 7,600 kg of crew and
//! cargo, half the fuel. Payload up to 42,580 lb [2+]. CG 25% MAC, limits
//! 15-30% [1]. Inertia [est]: typical nondimensional radii of gyration of a
//! four-engine transport (Roskam, Airplane Design part V: Rx 0.27, Ry 0.34,
//! Rz 0.45) at the reference mass. Fuel flows: ground idle 50 lb a minute
//! for the four (S4), 0.40 kg/s; take-off about 2,450 lb/h each [est, an
//! ESFC near 0.50 lb/shp/h], 1.24 kg/s. The burn follows the thrust (shared
//! code), which at 20,000 ft and 280 KTAS gives about 4,400 lb/h: what the
//! ferry range implies (62,400 lb for 3,995 nm at about 280 kt).
//!
//! # Engines (four Allison T56-A-15 and Hamilton Standard 54H60s)
//!
//! Flown as `Engine::Fan` (see mod.rs: n1 is the propellers' rpm, 1,020 at
//! 100%). At the take-off torque limit, 19,600 in-lb, the shaft gives about
//! 4,300 shp [est from the torque at 13,820 rpm; 4,591 shp is the thermodynamic
//! rating]. Static thrust 55 kN each (12,400 lbf, 2.9 lbf/shp) [est: momentum
//! theory through the 4.115 m disc, figure of merit near 0.7, plus about
//! 3 kN of jet thrust]. In flight a propeller's thrust is its power over its
//! speed: `mach_lapse` [2.0, 1.256] is fitted (least squares, 30-170 m/s at
//! sea level) to 4 x 4,300 shp at an efficiency rising from 0.7 at 60 m/s to
//! 0.85 above 120 m/s plus the jet thrust; the test checks the efficiency the
//! thrust implies at 60, 90 and 150 m/s. The density lapse (sigma^0.8) is the
//! shared code's. Reverse: the propellers in full reverse give about half
//! the static forward thrust (`reverse` 1.8 times what the fans make at the
//! reversing n1) [est]. Flight idle to 90% thrust in about 2 s.
//!
//! # Aerodynamics [est, fitted to the figures]
//!
//! CL 0.30 at zero (3° root incidence, cambered 64A3xx sections), 5.0 a
//! radian (aspect ratio 10.1). Power-off CLmax 1.7 clean (stall at 16°),
//! 2.55 with 100% flaps (`flap_cl` 1.0, stall 1.7° earlier: it flies its
//! finals nose low, a little below the glide path's angle): 124 kt clean at
//! 155,000 lb, 114 kt clean and 93 kt with 100% flaps at 130,000 lb, so a
//! normal threshold (1.35 Vs) of 125 kt and a final (+10) of 135, matching
//! S4's speed rules, S3's 180 kt penetration speed (65 kt above the stall)
//! and the 90-ish kt flapped stalls quoted by a former crewman [anec]. CD0
//! 0.026 (an equivalent flat plate near 45 ft²), Oswald factor 0.74, gear
//! 0.012 (it folds into the sponsons), flaps 0.065 at 100%. Fitted together
//! with the thrust to the climb, the ceilings and the top speed.
//!
//! # Reference targets and measured results
//!
//! (sea level ISA unless stated; KTAS true, KIAS indicated)
//!
//! | Point | Configuration | Target (source) | Measured |
//! |---|---|---|---|
//! | Stall, power off | 155,000 lb clean / 50% | ~124 / ~111 kt [est] | 124 / 111 KIAS (formula) |
//! | Stall, power off | 130,000 lb clean / 100% | ~114 (<=115, S3) / ~93 kt [est, anec 91] | 114 / 93 KIAS |
//! | Stall, `--fly-check` | 56.2 t (123,800 lb) clean, 1,500 m | 105-115 [est] | 110 KIAS at 15.7° |
//! | Take-off run | 155,000 lb, 50% flaps | 3,586 ft, 1,093 m (S1) | 1,179 m (3,867 ft), off at 131 KIAS |
//! | Climb | 155,000 lb clean, 180 KIAS, 1,000-2,700 ft | 1,830-1,900 fpm at sea level (S1, S2) | 1,601 fpm |
//! | Time to climb | 155,000 lb, 1,000 to 10,000 ft at 180 KIAS | 6-8 min [est] | 7.9 min (839 lb from brake release) |
//! | Ceiling (100 fpm left) | 155,000 lb at 23,000 ft | 23,000 ft (S1) | 167 fpm left there (ceiling ~24,500 ft) |
//! | Ceiling (100 fpm left) | 47 t at 33,000 ft | 33,000 ft "empty" (S1) | 332 fpm left there |
//! | Max level speed | 56.2 t, 20,000 ft | 320 KTAS (S1) | 299 KTAS (222 KIAS) |
//! | Max level speed | 56.2 t, 1,000 ft / 1,500 m | ~300-310 KTAS [est from power] | 315 KTAS / 290 KIAS (`--fly-check`) |
//! | Cruise | 56.2 t, 20,000 ft, 280 KTAS | 280-292 KTAS (S1, S4); ~4,400 lb/h (ferry range) | held at 92% throttle, 4,395 lb/h, alpha 2.3° |
//! | Range at that burn | all 62,400 lb | 3,995 nm ferry (S1) | 3,976 nm |
//! | Static thrust | 4 engines | 2.6-3.2 lbf/shp [est] | 220 kN, 0.32 of 155,000 lb |
//! | Propeller efficiency implied | sea level, 60/90/150 m/s | ~0.7/0.8/0.85 + jet [est] | 0.71/0.86/0.93 |
//! | Power response | flight idle to 90% thrust | 1-4 s [est] | 2.5 s |
//! | Roll rate | full aileron, 200 KIAS, 56.2 t | 15-30°/s [est; light boosted controls, S6] | 25°/s |
//! | Final approach | 56.2 t, 100% flaps, gear, 3° | 1.35 Vs + 10 = 132 kt (S4) | 130 kt hands off, 654 fpm, throttle 0.50, alpha -0.1° |
//! | Landing from 50 ft | 130,000 lb, 100% flaps, 125 KIAS, brakes + reverse | 2,750 ft total [1: a search summary of a NATA C-130 sheet, not read]; roll 1,150-2,020 ft (S7) | 314 m air + 541 m roll (1,776 ft) = 855 m (2,806 ft); 119 KIAS at touch-down |
//! | Landing roll, brakes only | same | longer | 815 m (2,675 ft): reverse saves a third |
//!
//! # Simulator limitations and remaining discrepancies
//!
//! - One throttle for all four engines and one engine health: no engine-out,
//!   no asymmetric thrust or VMCA, no feathering; damage fails all four.
//! - No constant-speed propeller or turboprop behaviour: the propellers'
//!   rpm follows the throttle (72% at idle, 100% at full) like a fan's n1,
//!   where a real T56 runs at 100% in flight and changes blade angle. No
//!   torque, TIT or prop-wash effects; no power-on (blown-lift) stall
//!   margin: the stall is the power-off one at any power.
//! - Flight idle still gives a little thrust; the real props at flight idle
//!   (low pitch stop) drag hard, so idle descents here are shallower.
//! - The shared density lapse is steeper than a flat-rated (torque-limited)
//!   T56's: low down it reaches about 315 KTAS flat out, at 20,000 ft only
//!   about 300 (320 published); at 20,000 ft its 280 KTAS cruise needs about
//!   92% throttle, where the real one cruises on normal cruise power.
//!   Ceilings come out 1,500-3,000 ft high.
//! - The cargo ramp and aft door ride the shared canopy action
//!   (`Plane::canopy`, `Input::Canopy`): they open or close in a canopy's two
//!   seconds (the real ones take tens of seconds [est]) at any airspeed,
//!   with the canopy's fixed drag (CD + 0.03 when open), and speed never
//!   damages them: the 150 KIAS ramp-and-door limit (185 KIAS with the door
//!   alone open; S3, S8) is not enforced. The shared code has no per-type
//!   canopy time, speed limit or damage, and is not changed for one type.
//! - No gear-extension speed limit (165 kt, S3) in the model; overspeed
//!   warnings use `VNE_KT`, 320 KIAS (S8's VD, S6's redline) and Mach 0.64
//!   (S3, S8).
use super::dims::*;
use crate::aircraft::{Engine, Fan, Handling, Hole, Hull, Leg, Part, Sound, Spec, Tail};
use glam::DVec3;

const KT: f64 = 1.0 / 1.94384;

/// The C-130H's maximum airspeed, knots indicated: VD, the "maximum speed"
/// line of the Limit Flight Speed vs. Altitude chart (S8, figure 5-5 sheet 1),
/// constant at 320 KIAS from sea level to 15,000 ft; above that Mach 0.64
/// (`SPEC.mmo`) limits, 290 KIAS at 20,000 ft (S9). [2+, high confidence: S8
/// is the flight manual's chart, read at 320 ± 2 kt; S6 quotes the same
/// "redline of 320 knots"; S9's 316 KIAS at 15,000 ft is the Mach line
/// meeting it.] The lower "recommended" speed VH (about 270 KIAS for the
/// lightest loadings, less when heavy) is advisory, for turbulence, and is
/// not modelled. `SPEC.vne`, the overspeed warning and the cockpit airspeed
/// indicator's limit all read this one value.
pub const VNE_KT: f64 = 320.0;

/// The two figures the oleos are sized from: the reference mass's weight on
/// the nose gear and on each of the four main struts at rest.
const W_REF: f64 = 56_160.0 * 9.81;
const NOSE_SHARE: f64 = (MAIN_D - ZN) as f64 / WHEELBASE as f64;

const fn main_leg(x: f32, z: f32, name: &'static str, kick: f64) -> Leg {
    Leg {
        at: DVec3::new(x as f64, (by(MAIN_SQUAT)) as f64, z as f64),
        wheel_r: MAIN_TYRE_R as f64,
        travel: 0.40,
        // compressed MAIN_SQUAT at rest under a quarter of what the nose does not carry
        k: W_REF * (1.0 - NOSE_SHARE) / 4.0 / MAIN_SQUAT as f64,
        c: 1.4e5,
        nf_max: 1.6e6,
        steer: 0.0,
        brakes: true,
        side_k: 2.6e5,
        side_mu: 0.8,
        roll_k: 5.0e3,
        roll_mu: 0.015,
        brake_k: 1.3e5,
        brake_mu: 0.5,
        fold_vn: 4.5,
        fold_kick: DVec3::new(0.3, 0.0, kick),
        name,
    }
}

pub static SPEC: Spec = Spec {
    id: 11,
    name: "Lockheed C-130H Hercules",
    keys: &["c130", "c130h", "hercules"],
    blurb: "Four-engine turboprop military transport, 70 t. Heavy, but its boosted controls are light and quick; about 280 kt cruise, 50% flaps to take off, 100% and about 130 kt on final. Z reverses the propellers, canopy key: cargo ramp.",
    origin: None,
    ext: None,
    // Rendered by the game (see paint.rs: stills.sh's shot, AMC grey, from higher and further ahead).
    still: include_bytes!("../../../assets/aircraft/c130.png"),
    paint: None,
    build: super::build,
    // Estimates of how its empty mass (about 34.7 t) divides (the wings with their two engines each).
    groups: [("fuselage", 15_000.0), ("tail", 1_800.0), ("left wing", 6_500.0), ("right wing", 6_500.0), ("flight deck", 2_500.0), ("radome", 100.0), ("gear", 2_300.0)],
    span: SPAN,
    wing_sect,
    fuse: |z| {
        let s = sec(z);
        (s.top, s.mid, s.bot)
    },
    aileron: AILERON,
    tank: TANK,
    tail: Tail {
        fin_y0: by(FIN_ROOT_H),
        fin_h: FIN_H,
        rudder_y0: by(FIN_ROOT_H) + RUDDER_SPAN[0],
        rudder_h: RUDDER_SPAN[1] - RUDDER_SPAN[0],
        stab_root: 1.0,
        elev_root: ELEV_ROOT,
        break_z: [bz(RAMP_TIP_D), bz(AFT_DOOR_END_D)],
        fin_base: DVec3::new(0.0, by(FIN_ROOT_H) as f64, bz(25.5) as f64),
        fin_tip: DVec3::new(0.0, by(HEIGHT) as f64, bz(27.4) as f64),
        stab_tip: DVec3::new(STAB_SPAN as f64 - 0.1, by(STAB_H) as f64, bz(28.1) as f64),
    },
    eye: EYE,
    eye2: None,
    sees_inside: super::deck::sees_inside,
    hotspots: &[],
    contacts: &[
        // the radome, under it, over the flight deck, the nose's sides
        ([0.0, by(2.21) as f64, bz(0.0) as f64], Part::Nose),
        ([0.0, by(0.98) as f64, bz(1.95) as f64], Part::Nose),
        ([0.0, by(3.98) as f64, bz(3.60) as f64], Part::Nose),
        ([-1.74, by(2.30) as f64, bz(2.60) as f64], Part::Nose),
        ([1.74, by(2.30) as f64, bz(2.60) as f64], Part::Nose),
        // the flat belly and the sponsons' undersides
        ([0.0, by(FUSE_BOT_H) as f64, bz(6.5) as f64], Part::Belly),
        ([0.0, by(FUSE_BOT_H) as f64, bz(12.0) as f64], Part::Belly),
        ([0.0, by(FUSE_BOT_H) as f64, bz(18.5) as f64], Part::Belly),
        ([-2.30, by(SPON_BOT_H + 0.03) as f64, bz(MAIN_D) as f64], Part::Belly),
        ([2.30, by(SPON_BOT_H + 0.03) as f64, bz(MAIN_D) as f64], Part::Belly),
        ([-FUSE_HALF_W as f64, by(FUSE_MID_H) as f64, bz(6.5) as f64], Part::Side),
        ([FUSE_HALF_W as f64, by(FUSE_MID_H) as f64, bz(6.5) as f64], Part::Side),
        ([-FUSE_HALF_W as f64, by(FUSE_MID_H) as f64, bz(19.0) as f64], Part::Side),
        ([FUSE_HALF_W as f64, by(FUSE_MID_H) as f64, bz(19.0) as f64], Part::Side),
        ([-2.62, by(1.74) as f64, bz(MAIN_D) as f64], Part::Side),
        ([2.62, by(1.74) as f64, bz(MAIN_D) as f64], Part::Side),
        ([0.0, by(FUSE_TOP_H) as f64, bz(6.5) as f64], Part::Roof),
        ([0.0, by(WING_ROOT_H + 0.46) as f64, bz(13.0) as f64], Part::Roof),
        ([0.0, by(FUSE_TOP_H) as f64, bz(19.0) as f64], Part::Roof),
        // under the nacelles, the propellers' lowest tips, the external tanks
        ([-ENG[0][0] as f64, by(ENG[0][1] - NAC_R) as f64, bz(10.5) as f64], Part::Pod(0)),
        ([ENG[0][0] as f64, by(ENG[0][1] - NAC_R) as f64, bz(10.5) as f64], Part::Pod(1)),
        ([-ENG[1][0] as f64, by(ENG[1][1] - NAC_R) as f64, bz(10.7) as f64], Part::Pod(0)),
        ([ENG[1][0] as f64, by(ENG[1][1] - NAC_R) as f64, bz(10.7) as f64], Part::Pod(1)),
        ([-ENG[0][0] as f64, by(ENG[0][1] - PROP_R) as f64, bz(ENG[0][2]) as f64], Part::Pod(0)),
        ([ENG[0][0] as f64, by(ENG[0][1] - PROP_R) as f64, bz(ENG[0][2]) as f64], Part::Pod(1)),
        ([-ENG[1][0] as f64, by(ENG[1][1] - PROP_R) as f64, bz(ENG[1][2]) as f64], Part::Pod(0)),
        ([ENG[1][0] as f64, by(ENG[1][1] - PROP_R) as f64, bz(ENG[1][2]) as f64], Part::Pod(1)),
        ([-EXT_TANK[0] as f64, by(EXT_TANK[1] - EXT_TANK[4]) as f64, bz(EXT_TANK[2] + 3.3) as f64], Part::Pod(0)),
        ([EXT_TANK[0] as f64, by(EXT_TANK[1] - EXT_TANK[4]) as f64, bz(EXT_TANK[2] + 3.3) as f64], Part::Pod(1)),
    ],
    // under the upswept tail: the ramp, the aft door, the tail cone
    tail_pts: &[[0.0, by(0.80) as f64, bz(20.4) as f64], [0.0, by(1.62) as f64, bz(23.0) as f64], [0.0, by(2.65) as f64, bz(26.0) as f64], [0.0, by(3.36) as f64, bz(28.5) as f64], [0.0, by(3.74) as f64, bz(29.79) as f64]],
    wing_st: [3.0 / SPAN, 7.0 / SPAN, 11.0 / SPAN, 15.0 / SPAN, 19.0 / SPAN],
    hard: [
        DVec3::new(0.0, by(2.21) as f64, bz(0.0) as f64),
        DVec3::new(-SPAN as f64, by(5.25) as f64, bz(12.75) as f64),
        DVec3::new(SPAN as f64, by(5.25) as f64, bz(12.75) as f64),
        DVec3::new(0.0, by(3.80) as f64, bz(D_TAIL) as f64),
        DVec3::new(0.0, by(HEIGHT) as f64, bz(27.4) as f64),
        DVec3::new(0.0, by(FUSE_BOT_H) as f64, bz(12.0) as f64),
        DVec3::new(ENG[0][0] as f64, by(PROP_TIP_LOW_H) as f64, bz(ENG[0][2]) as f64),
    ],
    gear: &[
        Leg {
            at: DVec3::new(0.0, by(NOSE_SQUAT) as f64, bz(NOSE_D) as f64),
            wheel_r: NOSE_TYRE_R as f64,
            travel: 0.35,
            k: W_REF * NOSE_SHARE / NOSE_SQUAT as f64,
            c: 1.0e5,
            nf_max: 9.0e5,
            steer: 1.0,
            brakes: false,
            side_k: 1.5e5,
            side_mu: 0.8,
            roll_k: 3.0e3,
            roll_mu: 0.015,
            brake_k: 0.0,
            brake_mu: 0.0,
            fold_vn: 3.8,
            fold_kick: DVec3::new(-0.4, 0.0, 0.0),
            name: "NOSE GEAR",
        },
        main_leg(-TRACK * 0.5, MAIN_AXLE_FWD.z, "LEFT FWD MAIN", 0.3),
        main_leg(TRACK * 0.5, MAIN_AXLE_FWD.z, "RIGHT FWD MAIN", -0.3),
        main_leg(-TRACK * 0.5, MAIN_AXLE_AFT.z, "LEFT AFT MAIN", 0.2),
        main_leg(TRACK * 0.5, MAIN_AXLE_AFT.z, "RIGHT AFT MAIN", -0.2),
    ],
    hull: Hull { k: 3.2e6, c: 5.0e5, nf_max: 6.5e6, drag: 1.6e5 },
    hole: Hole {
        fus: [FUSE_HALF_W, 0.5 * (FUSE_TOP_H - FUSE_BOT_H), by(0.5 * (FUSE_TOP_H + FUSE_BOT_H)), 1.2],
        wing: [by(WING_ROOT_H), DIHEDRAL, 0.46, 0.16],
        stab: [by(STAB_H), 0.20, STAB_SPAN],
        fin: [by(FIN_ROOT_H), FIN_H, 0.40, 0.15],
        pods: [[ENG[0][0], by(ENG[0][1]), NAC_R], [ENG[1][0], by(ENG[1][1]), NAC_R]],
    },
    // Tanks half full (see the module docs): 70,307 kg with them full.
    mass: 56_160.0,
    fuel_kg: 28_300.0,
    fuel_flow: [0.40, 1.24, 1.30],
    payload: [7_600.0, 19_300.0, 0.0],
    mac: MAC as f64,
    cg: [0.25, 0.15, 0.30],
    fuel_arm: 0.3,
    // Pitch, yaw, roll.
    inertia: DVec3::new(1.44e6, 3.50e6, 1.67e6),
    wing_area: WING_AREA as f64,
    b: B as f64,
    cl0: 0.30,
    cl_a: 5.0,
    cd0: 0.026,
    // 1 / (pi A e), e about 0.74
    k_ind: 0.0425,
    cd_gear: 0.012,
    cd_flap: [0.02, 0.045],
    flap_cl: 1.0,
    stall_alpha: 0.28,
    stall_flap: 0.03,
    stall_slat: 0.0,
    alpha_trim: 0.22,
    mcrit: 0.62,
    cd_wave: 0.0,
    // No spoilers.
    spoiler_cd: 0.0,
    spoiler_cl: 0.0,
    chute: None,
    // 0, 50% (take-off) and 100% (landing); 100% is 36° [est].
    flaps: &[0.0, 18.0, 36.0],
    vfe: &[1e9, 180.0 * KT, 145.0 * KT],
    flap_rate: 1.0 / 10.0,
    slats_at: 0,
    lever: None,
    gear_rate: 1.0 / 18.0,
    // (hydraulic gear and flaps, not electric)
    electric_actuators: false,
    // VD 320 KIAS to 15,000 ft, Mach 0.64 above (S8, see `VNE_KT`).
    vne: VNE_KT * KT,
    mmo: 0.64,
    // [est] flutter sets in 50 kt (about 15%) past VD, as on the 747.
    flutter_v: (VNE_KT + 50.0) * KT,
    inlet_mach: 0.0,
    dry_mach: 0.0,
    // Limit load 2.5 g at the maximum normal weight [est], ultimate 1.5 times it.
    g_limit: 3.0,
    g_break: 4.5,
    fbw_g: 0.0,
    // Nose wheel steering by tiller, about 60° either way [est].
    steer_max: 1.05,
    steer_fade: 15.0,
    engine: Engine::Fan(Fan {
        n: 4,
        thrust: 55_000.0,
        // the propellers turn at about 72% on the ground at low-speed idle [est]: they blur at idle
        idle: 0.72,
        idle_thrust: 0.03,
        n1_rpm: 1020.0,
        spool_up: 1.2,
        spool_down: 1.2,
        reverse: 1.8,
        reverser_time: 1.0,
        boost: 1.05,
        mach_lapse: [2.0, 1.256],
        nozzle_z: bz(EXHAUST_D[0]) as f64,
    }),
    handling: Handling {
        q_ref: 6_500.0,
        a_max: 1.8,
        damp_v: 68.0,
        ad_min: 0.3,
        ad_max: 1.8,
        pitch_k: 3.5,
        pitch_k_lost: 1.0,
        pitch_ctl: 0.35,
        alpha_ctl: 0.22,
        pitch_d: 5.0,
        roll_ctl: 0.75,
        roll_d: 2.75,
        roll_beta: 0.3,
        roll_clean: 1.0,
        yaw_ctl: 0.25,
        yaw_beta: 1.4,
        yaw_d: 2.5,
        dihedral: 0.10,
        adverse: 0.15,
        stall_pitch: 0.4,
        stall_drop: 0.6,
        stall_yaw: 0.9,
    },
    // About 230 kt indicated down low; a 125 kt final with 100% flaps.
    cruise: 118.0,
    approach: (130.0 * KT, 2),
    cam_dist: 52.0,
    shadow_r: 24.0,
    rotor: None,
    floats: false,
    sound: Sound::Turboprop,
};
