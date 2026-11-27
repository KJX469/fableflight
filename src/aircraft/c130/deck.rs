//! The flight deck of the C-130H Hercules: the analog "round-dial" deck of
//! the H (not the J's glass cockpit nor an avionics modernisation).
//!
//! # Layout (as built here)
//!
//! - Main instrument panel across the deck, dark gull grey: the pilot's and
//!   the copilot's flight instrument groups (clock, airspeed, attitude
//!   director, altimeter; turn and slip, bearing-distance-heading indicator,
//!   horizontal situation indicator, radar altimeter; vertical speed; the
//!   elevator and rudder trim tab indicators under the HSI), and left of the
//!   centreline the engine instrument stack: four columns (engines 1 to 4)
//!   of nine dials each, top to bottom torquemeter, tachometer (% RPM),
//!   turbine inlet temperature, fuel flow, oil temperature, oil pressure,
//!   gearbox oil pressure, oil quantity, oil cooler flap. Right of the stack
//!   (the copilot's side of the centre): the nacelle overheat lights, the
//!   three landing gear position indicators over the landing gear lever, the
//!   flap position indicator, the landing light panel and the hydraulic
//!   pressure gauges (brakes, auxiliary, utility, booster, rudder boost).
//! - A slim padded glareshield with the altitude alerter on top.
//! - The pedestal: the throttle quadrant (a compact hump about 0.6 m above
//!   the floor just under the centre panel, so the engine stack and the gear
//!   handle are seen over it) with the four power levers (left half, close
//!   together) and the four condition levers (right half, more widely
//!   spaced, numbered tags; FEATHER a red band at the aft end of their
//!   travel), the synchrophaser and temperature datum
//!   switches behind them, the trim panel (elevator tab switch, aileron and
//!   rudder trim knobs), the autopilot controller, the flap lever with its
//!   percent-extended scale at the aft end, radio panels down its sides.
//! - The overhead: the fire emergency panel (hazard stripes, the five clear
//!   T-handles: four engines and the GTC) along its forward edge, the engine
//!   start panel, the electrical panel (battery switch), the fuel panel (black
//!   with red flow lines and the quantity gauges), anti-ice and air
//!   conditioning, circuit breakers aft.
//! - Side panels from the main panel's ends out to the walls (the oxygen
//!   regulator and the interphone panel on each); side consoles, the nose
//!   wheel steering wheel on the pilot's and aft of it the ramp and door
//!   switch (see "Simplified"); a riveted trim panel along each wall above
//!   the console with the map case, the oxygen hose and mask.
//! - The two control columns with their U-shaped wheels, rudder pedals with
//!   toe brakes, the pilots' seats with harnesses, behind them the flight
//!   engineer's raised seat, the navigator's station on the right (table,
//!   radio rack, swivel seat), the crew bunk on the left, the aft bulkhead.
//! - Lining (cream insulation blankets over the interior-green structure),
//!   the glazing structure (`window_frames`): flat riveted frames round every
//!   opening `dims::window` cuts in the lining, one post between adjacent
//!   panes (the field half way between them), joined at the corners, a
//!   reveal and rubber seal at the glass; sun visors stowed over the side
//!   windows, the non-slip floor.
//!
//! # References
//!
//! - Photographs of analog H flight decks on Wikimedia Commons: USCG HC-130H
//!   1704 ("USCG Lockheed HC-130H 1704 cockpit 1-4", "... cockpit shelf",
//!   "... navigator's station": the panel, engine stack, overhead fuel and
//!   fire panels, nav station); "Lockheed C-130 Hercules flight deck.jpg"
//!   (USAF H in a museum: pedestal and quadrant); "Antwerp Lockheed C-130H
//!   Hercules BAF CH11 17/18" (Belgian H, its panel upgraded but the original
//!   pedestal: power levers left, condition levers right with numbered tags,
//!   the trim panel and the flap lever's PERCENT EXTENDED scale aft);
//!   "Flight deck of a RNZAF C-130H Hercules in 2011" (lining, overhead,
//!   steering wheel on the pilot's console).
//! - A C-130 flight engineer's systems gouge (baseops.net, enggouge99.doc):
//!   "5 [fire] handles located on fwd overhead panel"; the condition lever
//!   "actuates a switch in the control pedestal"; engine oil 50-60 psi, oil
//!   temperature 60-85 C, gearbox oil 150-250 psi, TD start limiting 830 C.
//! - Limits on the dials: T56-A-15 take-off torque 19,600 in-lb, TIT 1,083 C,
//!   overspeed 103.5% [unofficial crew references; TO 1C-130H-1 not seen].
//!
//! All positions are estimates scaled from the photographs to the flight
//! deck envelope in `dims` (`PANEL_*`, `GLARE_*`, `PED_*`, `OVHD_*`, `SEAT_*`,
//! `EYE`), which is itself an estimate.
//!
//! # Instruments
//!
//! Drawn in plane.wgsl (`gauge`, ids in `gauge_id`) from main.rs
//! `instruments` (the jet readings). The four T56s are the simulator's one
//! `Engine::Fan` (n = 4), so the four columns read alike. Mapping (plane.wgsl
//! `c130_eng`): % RPM = 100 x the simulated propeller speed: 72% at ground
//! idle (the real low-speed ground idle is 69-75.5%, TO 1C-130H-1 limits
//! via baseops.net c130h1_Ops_Limits.pdf), rising with the power lever to
//! 100% (the real engine holds 100% in flight and changes blade angle: not
//! simulated, see flight.rs); torque = 700 + 18,900 x the power setting above idle (in-lb, to 19,600 at
//! full power; slightly negative while the starter motors it); TIT from
//! main.rs (560 C at idle to 1,010 C at full power, the flight-model agent's
//! estimate); fuel flow is the engine's burn (one quarter of the total);
//! oil pressures follow the engine speed; oil temperature is the outside air
//! until it runs, then 75 C (the warm-up is not simulated); oil quantity
//! reads a constant 9.6 gal (no consumption simulated); the oil cooler flap
//! opens with power. The hydraulic gauges: utility, booster and rudder boost
//! follow the engine-driven pumps, auxiliary reads 0 (its electric pump is
//! not simulated), brakes the accumulator's 2,900 psi. The flight dials
//! (airspeed, attitude, altimeter, VSI, heading, HSI course from NAV 1,
//! bearing from the ADF, radar altimeter, clocks, flap position, the gear
//! position indicators and the elevator tab from the pitch trim) are live.
//! The airspeed's red line is `flight::VNE_KT` (320 KIAS) and it has no
//! coloured arcs (the H's military indicator has none: no flap arc). The
//! radar altimeter's dial is the HC-130H's non-linear one (0-500 ft over the
//! right half, logarithmic to 2,500 ft up the left; plane.wgsl
//! `c130_ra_ang`). The elevator tab indicator shows the pitch trim (an angle
//! of attack in the simulator, not a tab angle): zero where the reference
//! mass is trimmed at the cruise speed at sea level, its 25 stops at the
//! trim's limits (`alpha_trim` nose up, the auto-trim's -0.05 rad nose
//! down); the real tab's travel in degrees was not found. The test
//! `gauges_follow_the_flight_model` keeps these in step with flight.rs.
//! The rudder tab reads 0 (no rudder trim). Dials with no simulated reading
//! (`gauge_id::REST`: cabin pressure, the accelerometer, oxygen) rest at zero.
//!
//! # Clickable controls (`aircraft::Hotspot`, each on its own moving piece)
//!
//! - "power lever" x4: wheel = throttle; hold = reverse (pulled back into
//!   the beta range; the simulator's reversers).
//! - "condition lever" x4: click = `ignition` (the simulator's one fuel
//!   control: all four move together between GROUND STOP and RUN, driven by
//!   `Input::FuelControl(0)` so they stay put while the starter turns).
//!   Hover: "Condition levers", RUN / GROUND STOP.
//! - "engine start button" x4 (overhead): hold = `starter` (all four go in
//!   while the one simulated starter runs).
//! - "battery switch" (overhead electrical panel): click = `master`.
//! - "gear handle": click = `gear`. "flap lever": wheel, one detent a notch.
//! - "parking brake" (yellow T-handle under the panel): click.
//! - "rudder pedal" (pilot's two): hold = toe brake left / right.
//! - "yoke trim switch" (both yokes) and "elevator tab switch" (pedestal):
//!   wheel = pitch trim.
//! - "landing light switch" x2: click = `landing_lights` (the lights follow
//!   it outside, main.rs).
//! - "ramp switch" (pilot's console, aft of the steering wheel): click =
//!   `canopy` (the cargo ramp and aft door ride the canopy action; hover:
//!   "Cargo ramp and door", OPEN / CLOSED). The flap lever's hover reads
//!   percent, as its scale and the indicator do.
//! - "altimeter baro knob" x2: click = `baro_qnh`; "course knob" x2 (HSI):
//!   wheel = OBS.
//!
//! # Simplified or decorative
//!
//! The condition levers are a two-position simplification (GROUND STOP,
//! RUN; no AIR START or FEATHER: no feathering in the simulator) and the
//! order of their gate legends is an estimate. One start button set and one
//! fuel control drive all four engines. The real ramp and door controls are
//! on the loadmaster's panel in the cargo compartment, not on the flight
//! deck: the switch here is a stand-in. Decorative: the fire handles and
//! agent discharge, every overhead panel but the battery and start switches,
//! the autopilot controller (the keys work it), the synchrophaser, TD
//! valves, aileron and rudder trim knobs, radios, nav station, the second
//! pair of rudder pedals' brakes, the nose wheel steering wheel (it turns
//! with the steering but is not a mouse target: there is no steering
//! action). Panel legends that are not real words are made-up text.
//!
//! # Known departures from the real flight deck
//!
//! - The overhead panel sits further aft than on the real aircraft, where it
//!   starts just behind the windshield (it is placed by `dims::OVHD_*`).
//! - The chin windows are mostly below the deck floor here (`dims::WINDOWS`
//!   puts them low): from inside only their top edge shows, at the floor.
//! - 16 window openings (8 a side, `dims::WINDOWS`) stand for the real 23;
//!   their outlines are estimates.
//! - All four engine columns read the same (one simulated engine).
//! - The posts are as wide as `FR_POST` makes them everywhere; the real
//!   centre post and corner posts differ in width.
use super::dims::*;
use crate::aircraft::b747::deck::{legend_mat, rect, screen_quad, sid, style, Face};
use crate::aircraft::Hotspot;
use crate::airframe::*;
use glam::{Quat, Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Instrument ids (GAUGE extra); keep in step with plane.wgsl `gauge`.
pub mod gauge_id {
    pub const ASI: f32 = 0.0;
    pub const ADI: f32 = 1.0;
    pub const ALT: f32 = 2.0;
    pub const TURN: f32 = 3.0;
    pub const VSI: f32 = 5.0;
    pub const CLOCK: f32 = 14.0;
    pub const COMPASS: f32 = 17.0;
    pub const BDHI: f32 = 30.0;
    /// The engine stack, top to bottom.
    pub const ENGINE: [f32; 9] = [40.0, 41.0, 42.0, 43.0, 44.0, 45.0, 46.0, 47.0, 48.0];
    pub const RADAR_ALT: f32 = 49.0;
    pub const HYD_UTILITY: f32 = 50.0;
    pub const HYD_BOOSTER: f32 = 51.0;
    pub const HYD_AUX: f32 = 52.0;
    pub const HYD_BRAKES: f32 = 53.0;
    pub const HYD_RUDDER: f32 = 54.0;
    pub const FLAPS: f32 = 55.0;
    pub const ELEV_TAB: f32 = 56.0;
    pub const RUDDER_TAB: f32 = 57.0;
    pub const FUEL_TANK: f32 = 58.0;
    pub const FUEL_TOTAL: f32 = 59.0;
    pub const GEAR: f32 = 60.0;
    pub const HSI: f32 = 61.0;
    pub const REST: f32 = 62.0;
}
use gauge_id as gid;

// ---------------------------------------------------------------- materials

/// Dark gull grey instrument panels, the black overhead and console panels.
const fn grey() -> Vec3 {
    Vec3::new(0.105, 0.115, 0.118)
}
const fn black() -> Vec3 {
    Vec3::new(0.020, 0.020, 0.022)
}
const PANEL: Mat = mt(mat::PLASTIC, 0.105, 0.115, 0.118).pat(pat::PANEL);
const PANEL_DK: Mat = mt(mat::PLASTIC, 0.070, 0.076, 0.078).pat(pat::PANEL);
const BLACK_PANEL: Mat = mt(mat::PLASTIC, 0.020, 0.020, 0.022).pat(pat::PANEL);
const CASE: Mat = mt(mat::PLASTIC, 0.028, 0.028, 0.030);
const BEZEL: Mat = mt(mat::PLASTIC, 0.012, 0.012, 0.013);
const KNOB: Mat = mt(mat::PLASTIC, 0.018, 0.018, 0.019);
const GLARE: Mat = mt(mat::FABRIC, 0.022, 0.022, 0.023);
const LINER: Mat = mt(mat::FABRIC, 0.50, 0.46, 0.35).pat(pat::LINING);
const STRUCT: Mat = mt(mat::SATIN, 0.25, 0.33, 0.25);
const FLOOR_MAT: Mat = mt(mat::SATIN, 0.055, 0.058, 0.056).pat(pat::NONSLIP);
const TREAD: Mat = mt(mat::ALU, 0.42, 0.42, 0.43).pat(pat::KNURL);
const SEAT_FAB: Mat = mt(mat::FABRIC, 0.13, 0.13, 0.125).pat(pat::MESH);
const SEAT_BACK: Mat = mt(mat::FABRIC, 0.028, 0.028, 0.03).pat(pat::SEAT);
const SEAT_FRAME: Mat = mt(mat::SATIN, 0.20, 0.21, 0.21);
const HARNESS: Mat = mt(mat::FABRIC, 0.22, 0.22, 0.17).pat(pat::WEBBING);
const YOKE_MAT: Mat = mt(mat::PLASTIC, 0.022, 0.022, 0.024);
const COLUMN_MAT: Mat = mt(mat::SATIN, 0.045, 0.048, 0.05);
const YELLOW: Mat = mt(mat::PAINT, 0.62, 0.44, 0.03);
const RED: Mat = mt(mat::PAINT, 0.45, 0.03, 0.02);
const WOOD: Mat = mt(mat::SATIN, 0.38, 0.22, 0.09);

// ---------------------------------------------------------------- printing and small parts

/// A printed legend (white on `bg`): string `s` ("" for made-up text).
fn lbl(m: &mut Mesh, f: &Face, u: f32, v: f32, hw: f32, hh: f32, s: &str, bg: Vec3) {
    let id = if s.is_empty() { 0 } else { sid(s) };
    rect(m, &legend_mat(style::TEXT, id, bg), f.at_h(u, v, 0.0004), f.ax, f.ay, hw, hh);
}
/// A word sized to its text, glyph height h.
fn word(m: &mut Mesh, f: &Face, u: f32, v: f32, h: f32, s: &str, bg: Vec3) {
    let n = s.chars().count().max(1) as f32;
    let hw = (n * 1.45 - 0.45) * h * 0.25 + h * 0.3;
    lbl(m, f, u, v, hw, h * 0.72, s, bg);
}
/// A placard: a card of fine print.
fn placard(m: &mut Mesh, f: &Face, u: f32, v: f32, hw: f32, hh: f32, red: bool) {
    let col = if red { Vec3::new(0.5, 0.06, 0.04) } else { Vec3::new(0.75, 0.74, 0.70) };
    rect(m, &mt(mat::PLASTIC, col.x, col.y, col.z), f.at_h(u, v, 0.0006), f.ax, f.ay, hw, hh);
    rect(m, &legend_mat(style::PLACARD, 0, col), f.at_h(u, v, 0.0011), f.ax, f.ay, hw * 0.94, hh * 0.9);
}
/// A plate standing proud of its face, a screw in each corner.
fn plate(m: &mut Mesh, f: &Face, u0: f32, v0: f32, u1: f32, v1: f32, mm: &Mat, seed: f32) {
    let (cu, cv) = ((u0 + u1) * 0.5, (v0 + v1) * 0.5);
    let (hu, hv) = ((u1 - u0) * 0.5 - 0.0008, (v1 - v0) * 0.5 - 0.0008);
    m.rbox(mm, f.at_h(cu, cv, 0.0015), Vec3::new(hu, hv, 0.0025), 0.0012, f.rot());
    for (k, (du, dv)) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().enumerate() {
        m.screw(&SCREW_BLACK, f.at_h(cu + du * (hu - 0.006), cv + dv * (hv - 0.006), 0.0040), f.n(), 0.0026, seed + k as f32);
    }
}
/// A round instrument: its square case, the bezel ring, the face, the screws.
fn gauge(m: &mut Mesh, f: &Face, u: f32, v: f32, r: f32, id: f32, seed: f32) {
    let rot = f.rot();
    let cs = r + 0.0065;
    m.rbox(&CASE, f.at_h(u, v, 0.004), Vec3::new(cs, cs, 0.004), 0.004, rot);
    let c = f.at_h(u, v, 0.008);
    m.lathe(&BEZEL, c, f.n(), &[[r + 0.0045, 0.0], [r + 0.0045, 0.0032], [r + 0.0022, 0.0048], [r, 0.0042], [r, 0.0006]], 36, false);
    m.disc(&mt(mat::GAUGE, 0.02, 0.02, 0.02).x(id), f.at_h(u, v, 0.0086), f.n(), f.ay, r, 40);
    for (k, (du, dv)) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().enumerate() {
        m.screw(&SCREW_BLACK, f.at_h(u + du * (cs - 0.0035), v + dv * (cs - 0.0035), 0.008), f.n(), 0.0019, seed + k as f32);
    }
}
/// A big flight instrument (ADI, HSI): a deep rounded case with a lip.
fn big_gauge(m: &mut Mesh, f: &Face, u: f32, v: f32, r: f32, id: f32, seed: f32) {
    let rot = f.rot();
    m.rbox(&PANEL_DK, f.at_h(u, v, 0.006), Vec3::new(r + 0.016, r + 0.016, 0.006), 0.010, rot);
    let c = f.at_h(u, v, 0.012);
    m.lathe(&BEZEL, c, f.n(), &[[r + 0.007, 0.0], [r + 0.007, 0.004], [r + 0.003, 0.006], [r, 0.005], [r, 0.0005]], 44, false);
    m.disc(&mt(mat::GAUGE, 0.02, 0.02, 0.02).x(id), f.at_h(u, v, 0.0124), f.n(), f.ay, r, 48);
    for (k, (du, dv)) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().enumerate() {
        m.screw(&SCREW_BLACK, f.at_h(u + du * (r + 0.010), v + dv * (r + 0.010), 0.012), f.n(), 0.0022, seed + k as f32);
    }
    // The test button and the adjusting knob along the bottom.
    m.lathe(&KNOB, f.at_h(u - r * 0.85, v - r - 0.008, 0.012), f.n(), &[[0.0045, 0.0], [0.0045, 0.005], [0.0, 0.0055]], 10, false);
}
/// A square drum indicator (the gear position indicators).
fn drum(m: &mut Mesh, f: &Face, u: f32, v: f32, h: f32, id: f32) {
    m.rbox(&CASE, f.at_h(u, v, 0.004), Vec3::new(h + 0.004, h + 0.004, 0.004), 0.002, f.rot());
    screen_quad(m, &mt(mat::GAUGE, 0.02, 0.02, 0.02).x(id), f.at_h(u, v, 0.0082), f.ax, f.ay, h, h);
}
/// A bat-handle toggle switch.
fn toggle(m: &mut Mesh, f: &Face, u: f32, v: f32, up: bool) {
    let c = f.at(u, v);
    let n = f.n();
    m.lathe(&CHROME, c, n, &[[0.0055, 0.0], [0.0055, 0.0018], [0.0042, 0.0022], [0.0042, 0.0042], [0.0, 0.0044]], 8, false);
    let tip = c + n * 0.016 + f.ay * if up { 0.006 } else { -0.006 };
    m.tube(&CHROME, &[c + n * 0.004, tip], &|t| 0.0018 + 0.0006 * t, 8, true);
    m.dome(&CHROME, tip, (tip - c).normalize(), 0.0026, 0.002, 8, 0.0);
}
/// A guarded switch (red guard for the ones that matter).
fn guarded(m: &mut Mesh, f: &Face, u: f32, v: f32, red: bool) {
    toggle(m, f, u, v, false);
    let col = if red { RED } else { mt(mat::PLASTIC, 0.03, 0.03, 0.032) };
    m.rbox(&col, f.at_h(u, v - 0.002, 0.012), Vec3::new(0.009, 0.016, 0.009), 0.0025, f.rot() * Quat::from_rotation_x(-0.15));
}
/// A pointer knob (rotary selector) with a scale.
fn knob(m: &mut Mesh, f: &Face, u: f32, v: f32, r: f32, seed: f32, bg: Vec3) {
    let c = f.at(u, v);
    let n = f.n();
    m.lathe(&KNOB, c, n, &[[r * 1.2, 0.0], [r * 1.2, r * 0.3], [r, r * 0.4], [r, r * 1.2], [r * 0.85, r * 1.35], [0.0, r * 1.38]], 12, false);
    let a = (hash(seed) - 0.5) * 2.4;
    m.rbox(&mt(mat::PLASTIC, 0.6, 0.6, 0.58), c + n * (r * 1.38), Vec3::new(r * 0.12, r * 0.8, 0.0004), 0.0002, f.rot() * Quat::from_rotation_z(a));
    rect(m, &legend_mat(style::SCALE, (hash(seed + 3.0) * 9.0) as u32, bg), c + n * 0.0004, f.ax, f.ay, r * 2.6, r * 2.6);
}
/// An indicator light: a lens in a black housing.
fn lamp_lens(m: &mut Mesh, f: &Face, u: f32, v: f32, r: f32, col: Vec3) {
    let c = f.at(u, v);
    m.lathe(&BEZEL, c, f.n(), &[[r * 1.5, 0.0], [r * 1.5, 0.004], [r * 1.2, 0.006], [0.0, 0.006]], 12, false);
    m.dome(&mt(mat::PLASTIC, col.x, col.y, col.z), c + f.n() * 0.006, f.n(), r, r * 0.7, 10, 0.0);
}
/// Fill a panel with controls: rows of switches, knobs and lights, made up from `seed`.
fn filler(m: &mut Mesh, f: &Face, u0: f32, v0: f32, u1: f32, v1: f32, seed: f32, bg: Vec3) {
    let w = u1 - u0;
    let h = v1 - v0;
    let rows = ((h / 0.048).floor() as usize).max(1);
    let rh = h / rows as f32;
    let g = f.sub(0.0, 0.0, 0.004);
    for r in 0..rows {
        let vy = v1 - (r as f32 + 0.6) * rh;
        let kind = (hash(seed + r as f32 * 3.1) * 4.0) as u32;
        let pitch = if kind == 1 { 0.042 } else { 0.034 };
        let n = ((w - 0.016) / pitch).floor().max(1.0) as usize;
        let start = (u0 + u1) * 0.5 - (n as f32 - 1.0) * pitch * 0.5;
        for i in 0..n {
            let x = start + i as f32 * pitch;
            let s2 = seed + r as f32 * 13.0 + i as f32 * 1.37;
            let pick = hash(s2);
            if pick < 0.15 {
                continue;
            }
            match kind {
                1 => knob(m, &g, x, vy - 0.004, 0.0058, s2, bg),
                2 => {
                    if pick > 0.75 {
                        guarded(m, &g, x, vy - 0.003, pick > 0.92);
                    } else {
                        toggle(m, &g, x, vy - 0.003, pick > 0.45);
                    }
                }
                3 => lamp_lens(m, &g, x, vy - 0.002, 0.0045, if pick > 0.7 { Vec3::new(0.35, 0.2, 0.02) } else { Vec3::new(0.05, 0.22, 0.06) }),
                _ => toggle(m, &g, x, vy - 0.003, pick > 0.55),
            }
            lbl(m, &g, x, vy + 0.014, 0.013, 0.003, "", bg);
        }
    }
}
/// Rows of circuit breakers with their labels.
fn breakers(m: &mut Mesh, f: &Face, u0: f32, v0: f32, u1: f32, v1: f32, seed: f32, bg: Vec3) {
    let (pu, pv) = (0.018, 0.03);
    let nu = ((u1 - u0) / pu) as usize;
    let nv = ((v1 - v0) / pv) as usize;
    for j in 0..nv {
        let v = v1 - (j as f32 + 0.6) * pv;
        lbl(m, f, (u0 + u1) * 0.5, v + 0.014, (u1 - u0) * 0.47, 0.0026, "", bg);
        for i in 0..nu {
            let u = u0 + (i as f32 + 0.5) * pu;
            if hash(seed + (j * 31 + i) as f32) < 0.08 {
                continue;
            }
            m.lathe(&CASE, f.at(u, v), f.n(), &[[0.0052, 0.0], [0.0052, 0.0016], [0.0038, 0.0022], [0.0038, 0.0085], [0.0032, 0.0095], [0.0, 0.0095]], 8, false);
        }
    }
}

// ---------------------------------------------------------------- the envelope

const FLOOR: f32 = DECK_FLOOR_Y;

/// The lining's surface point at station z and height y on side sx (the wall).
fn wall(z: f32, y: f32, sx: f32) -> Vec3 {
    Vec3::new(sx * deck_half_w(z, y), y, z)
}
/// Whether p lies in the envelope, give or take `tol`.
#[cfg(test)]
fn in_env(p: Vec3, tol: f32) -> bool {
    let z = p.z.clamp(DECK_FWD_Z, DECK_AFT_Z);
    let y = p.y.clamp(FLOOR, deck_ceiling(z));
    p.z >= DECK_FWD_Z - tol && p.z <= DECK_AFT_Z + tol && p.y >= FLOOR - tol && p.y <= deck_ceiling(z) + tol && p.x.abs() <= deck_half_w(z, y) + tol
}
/// Clamp a point to stay `gap` inside the envelope (sideways and under the ceiling).
fn inside(p: Vec3, gap: f32) -> Vec3 {
    let y = p.y.min(deck_ceiling(p.z) - gap).max(FLOOR + gap);
    let hx = (deck_half_w(p.z, y) - gap).max(0.0);
    Vec3::new(p.x.clamp(-hx, hx), y, p.z)
}

// ---------------------------------------------------------------- the deck

pub fn build(b: &mut Builder) {
    let mut shell = Mesh::default();
    lining(&mut shell);
    floor(&mut shell);
    window_frames(&mut shell);
    b.fixed("flight deck lining", G_ENGINE, shell);

    let mut m = Mesh::default();
    let pf = main_panel(&mut m, b);
    glareshield(&mut m, &pf);
    pedestal(&mut m, b);
    overhead(&mut m, b);
    b.fixed("flight deck panels", G_ENGINE, m);

    let mut s = Mesh::default();
    seats(&mut s);
    b.fixed("crew seats", G_ENGINE, s);

    let mut fu = Mesh::default();
    side_consoles(&mut fu, b);
    aft_deck(&mut fu);
    b.fixed("flight deck furnishings", G_ENGINE, fu);

    levers(b);
    columns(b);
}

// ---------------------------------------------------------------- lining and floor

fn lining(m: &mut Mesh) {
    let (z0, z1) = (DECK_FWD_Z, DECK_AFT_Z);
    let nz = 150;
    let mut side = Mesh::default();
    let rows: Vec<Vec<SP>> = (0..=nz)
        .map(|i| {
            let z = z0 + (z1 - z0) * i as f32 / nz as f32;
            let yc = deck_ceiling(z);
            let xc = deck_half_w(z, yc);
            let mut pts: Vec<Vec3> = (0..=36).map(|k| {
                let t = k as f32 / 36.0;
                // Denser toward the top, where the wall curves in.
                let y = FLOOR - 0.01 + (yc - FLOOR + 0.01) * t.powf(0.85);
                wall(z, y, 1.0)
            }).collect();
            for k in 1..=10 {
                let x = xc * (1.0 - k as f32 / 10.0);
                pts.push(Vec3::new(x, yc, z));
            }
            let mut arc = 0.0;
            let mut prev = pts[0];
            pts.iter().map(|&p| {
                arc += (p - prev).length();
                prev = p;
                (p, [z, arc], 0.0)
            }).collect()
        })
        .collect();
    side.grid(&LINER, &rows, false, false);
    // Interior-green structure below the sills and round the floor; the cream
    // insulation blankets above.
    for v in &mut side.v {
        let y = v.pos[1];
        let c = if y < FLOOR + 0.62 { Vec3::new(0.24, 0.31, 0.24) } else { Vec3::new(0.50, 0.46, 0.35) };
        v.col = [c.x, c.y, c.z, 1.0];
        if y < FLOOR + 0.62 {
            v.mat[0] = mat::SATIN;
        }
    }
    let mut left = side.mirror_x();
    for t in left.i.chunks_mut(3) {
        t.swap(1, 2);
    }
    m.append(&left);
    m.append(&side);

    // The frames (ring stiffeners) along the walls and over the ceiling between the windows.
    for d in [4.98f32, 5.45, 5.95, 6.45] {
        let z = bz(d);
        let yc = deck_ceiling(z);
        let pts: Vec<Vec3> = (0..=24).map(|k| {
            let y = FLOOR + (yc - FLOOR) * k as f32 / 24.0;
            inside(wall(z, y, 1.0), 0.03)
        }).chain((1..=8).map(|k| Vec3::new((deck_half_w(z, yc) - 0.03) * (1.0 - k as f32 / 8.0), yc - 0.016, z))).collect();
        let mut fr = Mesh::default();
        fr.tube(&STRUCT, &pts, &|_| 0.013, 6, true);
        m.append(&fr.mirror_x());
        m.append(&fr);
    }

    // The aft bulkhead (FS 245): the envelope's section, the door to the cargo compartment.
    let zb = DECK_AFT_Z;
    let yc = deck_ceiling(zb);
    let ring: Vec<Vec3> = (0..=24).map(|k| wall(zb, FLOOR + (yc - FLOOR) * k as f32 / 24.0, 1.0))
        .chain((0..=24).map(|k| wall(zb, yc - (yc - FLOOR) * k as f32 / 24.0, -1.0)))
        .collect();
    m.fan(&mt(mat::SATIN, 0.24, 0.31, 0.24), Vec3::new(0.0, FLOOR + 0.9, zb), &ring, Vec3::NEG_Z);
    // The forward end of the lining under the glareshield (out of sight).
    let zf = DECK_FWD_Z;
    let yf = deck_ceiling(zf);
    let ring: Vec<Vec3> = (0..=16).map(|k| wall(zf, FLOOR + (yf - FLOOR) * k as f32 / 16.0, 1.0))
        .chain((0..=16).map(|k| wall(zf, yf - (yf - FLOOR) * k as f32 / 16.0, -1.0)))
        .collect();
    m.fan(&mt(mat::SATIN, 0.05, 0.055, 0.05), Vec3::new(0.0, (FLOOR + yf) * 0.5, zf), &ring, Vec3::Z);
}

fn floor(m: &mut Mesh) {
    let (z0, z1) = (DECK_FWD_Z, DECK_AFT_Z);
    m.patch(&FLOOR_MAT, 60, 24, false, |a, c| {
        let z = z0 + (z1 - z0) * a;
        let x = (c - 0.5) * 2.0 * (deck_half_w(z, FLOOR + 0.001) - 0.002);
        (Vec3::new(x, FLOOR + 0.001, z), [z, x], 0.0)
    });
    // Seat tracks either side of each pilot and the engineer, the tread
    // plates in the walkway, the hatch to the crew door stairs.
    for sx in [-1.0f32, 1.0] {
        for dx in [-0.16f32, 0.16] {
            m.rbox(&TREAD, Vec3::new(sx * SEAT_L.x.abs() + dx, FLOOR + 0.006, SEAT_L.z - 0.05), Vec3::new(0.016, 0.006, 0.36), 0.002, Quat::IDENTITY);
        }
    }
    for k in 0..3 {
        let z = PED_AFT_Z + 0.45 + k as f32 * 0.55;
        m.rbox(&TREAD, Vec3::new(0.0, FLOOR + 0.003, z), Vec3::new(0.30, 0.003, 0.25), 0.002, Quat::IDENTITY);
    }
    let hz = DECK_AFT_Z - 0.55;
    m.rbox(&mt(mat::SATIN, 0.07, 0.075, 0.072).pat(pat::NONSLIP), Vec3::new(-0.62, FLOOR + 0.004, hz), Vec3::new(0.38, 0.004, 0.40), 0.004, Quat::IDENTITY);
    for (dx, dz) in [(-0.34f32, -0.36f32), (0.34, -0.36), (-0.34, 0.36), (0.34, 0.36)] {
        m.screw(&SCREW_STEEL, Vec3::new(-0.62 + dx, FLOOR + 0.008, hz + dz), Vec3::Y, 0.004, dx + dz);
    }
    m.rbox(&YELLOW.pat(pat::STRIPE).x(0.04), Vec3::new(-0.62, FLOOR + 0.0085, hz - 0.41), Vec3::new(0.38, 0.0005, 0.012), 0.0004, Quat::IDENTITY);
}

// ---------------------------------------------------------------- window frames

// The glazing structure, seen from inside: a flat frame `FR_W` wide standing
// `FR_H` proud of the lining round every opening `dims::window` cuts in it,
// and where two openings come close (or, 10 cm inside the skin, run into
// each other) one post `2 FR_POST` wide between them, centred on the line
// half way between the two. It is one surface: the band of the lining
// within `FR_W` of the nearest opening (`frame_field`, from each opening's
// distance measured along the lining, `win_dists`), so the frames close
// round every pane, posts and sills join at the corners and nothing
// crosses. Its edges are found on a fine chart of the lining (marching
// squares): the inner one (the opening, shrunk to the posts) turned back
// into a reveal with a rubber seal, the outer one rounding off just under
// the lining. Rivets run along the middle.
const FR_W: f32 = 0.048;
const FR_POST: f32 = 0.032;
const FR_H: f32 = 0.016;
const FR_STEP: f32 = 0.006;
const FR_RIVET: f32 = 0.020;
const FR_PITCH: f32 = 0.034;
/// How far aft the chart of the lining reaches (drawing distance): past the
/// last window's frame.
const FR_AFT_D: f32 = 4.95;

/// The lining as a grid: rows at stations z (`FR_STEP` apart, from the
/// lining's forward end), each sampled every `FR_STEP` along the lining from
/// the ceiling's centreline out and down the right wall to the floor
/// (`lining` draws the same surface), with the way into the deck.
struct Chart {
    nz: usize,
    ns: usize,
    p: Vec<Vec3>,
    n: Vec<Vec3>,
    ok: Vec<bool>,
}

impl Chart {
    fn new() -> Chart {
        let (z0, z1) = (DECK_FWD_Z, bz(FR_AFT_D));
        let nz = ((z1 - z0) / FR_STEP).ceil() as usize + 1;
        let rows: Vec<Vec<Vec3>> = (0..nz).map(|i| lining_row(z0 + (z1 - z0) * i as f32 / (nz - 1) as f32)).collect();
        let ns = rows.iter().map(|r| r.len()).max().unwrap_or(2);
        let mut p = vec![Vec3::ZERO; nz * ns];
        let mut ok = vec![false; nz * ns];
        for (i, r) in rows.iter().enumerate() {
            for (j, q) in r.iter().enumerate() {
                p[i * ns + j] = *q;
                ok[i * ns + j] = true;
            }
        }
        let mut c = Chart { nz, ns, p, n: vec![Vec3::ZERO; nz * ns], ok };
        for i in 0..nz {
            for j in 0..ns {
                if !c.ok[c.k(i, j)] {
                    continue;
                }
                let g = |a: usize, b: usize| if c.ok[c.k(a, b)] { Some(c.p[c.k(a, b)]) } else { None };
                let pc = c.p[c.k(i, j)];
                let du = g((i + 1).min(nz - 1), j).unwrap_or(pc) - g(i.saturating_sub(1), j).unwrap_or(pc);
                let dv = g(i, (j + 1).min(ns - 1)).unwrap_or(pc) - g(i, j.saturating_sub(1)).unwrap_or(pc);
                let mut n = du.cross(dv).normalize_or_zero();
                // (the way in: toward the middle of the deck at the pilots'
                // station, which also holds where the ceiling slopes steeply
                // down to the windshield's foot ahead of it)
                if n.dot(Vec3::new(0.0, FLOOR + 0.6, EYE.z) - pc) < 0.0 {
                    n = -n;
                }
                let k = c.k(i, j);
                c.n[k] = n;
            }
        }
        // Blurred across the row (about 5 cm), so the frame stood off along
        // them runs smoothly over the crease where the lining's flat ceiling
        // meets the wall.
        for _ in 0..2 {
            let n0 = c.n.clone();
            for i in 0..nz {
                for j in 0..ns {
                    if !c.ok[c.k(i, j)] {
                        continue;
                    }
                    let mut a = Vec3::ZERO;
                    for jj in j.saturating_sub(4)..(j + 5).min(ns) {
                        if c.ok[c.k(i, jj)] {
                            a += n0[c.k(i, jj)];
                        }
                    }
                    let k = c.k(i, j);
                    c.n[k] = a.normalize_or_zero();
                }
            }
        }
        c
    }
    fn k(&self, i: usize, j: usize) -> usize {
        i * self.ns + j
    }
    /// The lining's point and the way in at fractional grid place (u row, v column).
    fn at(&self, u: f32, v: f32) -> (Vec3, Vec3) {
        let (i, j) = ((u.floor().max(0.0) as usize).min(self.nz - 2), (v.floor().max(0.0) as usize).min(self.ns - 2));
        let (fu, fv) = (u - i as f32, v - j as f32);
        let bl = |a: &[Vec3]| {
            let (p00, p10, p01, p11) = (a[self.k(i, j)], a[self.k(i + 1, j)], a[self.k(i, j + 1)], a[self.k(i + 1, j + 1)]);
            p00.lerp(p10, fu).lerp(p01.lerp(p11, fu), fv)
        };
        (bl(&self.p), bl(&self.n).normalize_or_zero())
    }
    /// Which way along the lining a step (du, dv) in the grid goes, and how far.
    fn dir(&self, u: f32, v: f32, d: Vec2) -> Vec3 {
        let (a, _) = self.at(u - d.x * 0.5, v - d.y * 0.5);
        let (b, _) = self.at(u + d.x * 0.5, v + d.y * 0.5);
        b - a
    }
}

/// The lining's section at station z from the ceiling's centreline out and
/// down the right wall to the floor, every `FR_STEP` along it.
fn lining_row(z: f32) -> Vec<Vec3> {
    let yc = deck_ceiling(z);
    let s = sec(z);
    let xc = deck_half_w(z, yc);
    let mut pts: Vec<Vec3> = (0..=60).map(|k| Vec3::new(xc * k as f32 / 60.0, yc, z)).collect();
    let (t0, t1) = (fuse_th(z, yc), fuse_th(z, FLOOR));
    for k in 1..=600 {
        let q = sec_pt(&s, t0 + (t1 - t0) * k as f32 / 600.0);
        pts.push(Vec3::new(q.x - LINING, q.y, z));
    }
    // Every FR_STEP along it.
    let mut out = vec![pts[0]];
    let (mut acc, mut next) = (0.0f32, FR_STEP);
    for w in pts.windows(2) {
        let l = (w[1] - w[0]).length();
        while acc + l >= next && l > 0.0 {
            out.push(w[0].lerp(w[1], (next - acc) / l));
            next += FR_STEP;
        }
        acc += l;
    }
    out
}

/// One window's opening as `dims::window` cuts it, at p: negative inside.
/// (The outline's distance in the window's own projection, or the gate's:
/// at most the distance along the lining, never more.)
fn win_raw(w: &Win, p: Vec3) -> f32 {
    let (d, h, ax) = (p.z + ZN, p.y + YG, p.x.abs());
    let (q, gate) = match w.view {
        FRONT => (Vec2::new(ax, h), (w.gate[0] - d).max(d - w.gate[1])),
        SIDE => (Vec2::new(d, h), w.gate[0] - ax),
        _ => (Vec2::new(d, ax), w.gate[0] - h),
    };
    let poly = w.pts.map(|v| Vec2::new(v[0], v[1]));
    let lo = poly.iter().copied().fold(Vec2::splat(f32::MAX), Vec2::min);
    let hi = poly.iter().copied().fold(Vec2::splat(f32::MIN), Vec2::max);
    let far = (lo - q).max(q - hi).max_element() - WIN_R;
    if far > 0.3 {
        return far.max(gate);
    }
    (sd_poly(q, &poly) - WIN_R).max(gate)
}

/// How far (m, along the lining; negative inside) every chart point is from
/// each window's opening, out to `FR_FAR`: the distance to the opening's
/// edge, found on the chart (marching squares) and measured in the deck.
const FR_FAR: f32 = 0.14;
fn win_dists(c: &Chart) -> Vec<Vec<f32>> {
    const B: usize = 8;
    WINDOWS
        .iter()
        .map(|w| {
            let raw: Vec<f32> = (0..c.nz * c.ns).map(|k| if c.ok[k] { win_raw(w, c.p[k]) } else { 1.0 }).collect();
            let edge = march(c, &raw, 0.0, f32::MAX);
            let mut blocks: std::collections::HashMap<(usize, usize), Vec<(Vec3, Vec3)>> = Default::default();
            for ((_, qa), (_, qb)) in &edge.segs {
                let m = (*qa + *qb) * 0.5;
                blocks.entry((m.x as usize / B, m.y as usize / B)).or_default().push((c.at(qa.x, qa.y).0, c.at(qb.x, qb.y).0));
            }
            (0..c.nz * c.ns)
                .map(|k| {
                    let r = raw[k];
                    if !c.ok[k] || r.abs() > FR_FAR || blocks.is_empty() {
                        return r.signum() * FR_FAR.max(r.abs());
                    }
                    let (i, j) = (k / c.ns / B, k % c.ns / B);
                    let p = c.p[k];
                    let mut best = FR_FAR;
                    for bi in i.saturating_sub(3)..=i + 3 {
                        for bj in j.saturating_sub(3)..=j + 3 {
                            for &(a, b) in blocks.get(&(bi, bj)).map(|v| v.as_slice()).unwrap_or(&[]) {
                                let ab = b - a;
                                let t = ((p - a).dot(ab) / ab.length_squared().max(1e-12)).clamp(0.0, 1.0);
                                best = best.min((p - a - ab * t).length());
                            }
                        }
                    }
                    if r < 0.0 { -best } else { best }
                })
                .collect()
        })
        .collect()
}

/// The frame's field from the windows' distances: negative in an opening
/// (each shrunk back `FR_POST` from the line half way to its nearest
/// neighbour, so adjacent panes share one post), rising across the frame.
fn frame_field(ds: &[f32]) -> f32 {
    let mut h = f32::MAX;
    for i in 0..ds.len() {
        let m = (0..ds.len()).filter(|&j| j != i).map(|j| ds[j]).fold(f32::MAX, f32::min);
        h = h.min(ds[i].max((ds[i] - m) * 0.5 + FR_POST));
    }
    h
}

/// How far the frame stands in from the lining at field value h: flat
/// across, rounding down under the lining at its outer edge.
fn fr_lift(h: f32) -> f32 {
    FR_H - (FR_H + 0.004) * smoothstep(FR_W - 0.014, FR_W, h)
}

/// How far behind the lining (up to `want`, negative) a point of the frame
/// may go at p (the way in n) and stay within the deck's envelope (as the
/// test `deck_stays_in_its_envelope` measures it, give or take 1.5 cm): at
/// the walls' shoulders, where the section narrows fast, hardly at all.
fn sink(p: Vec3, n: Vec3, want: f32) -> f32 {
    let ok = |q: Vec3| {
        let z = q.z.clamp(DECK_FWD_Z, DECK_AFT_Z);
        let y = q.y.clamp(FLOOR, deck_ceiling(z));
        q.z >= DECK_FWD_Z - 0.015 && q.y <= deck_ceiling(z) + 0.015 && q.x.abs() <= deck_half_w(z, y) + 0.015
    };
    let mut h = want;
    while h < 0.0 && !ok(p + n * h) {
        h = (h + 0.0005).min(0.0);
    }
    h
}

/// A place on the chart: a grid point, or where the field crosses a grid
/// edge (0 along a row, 1 along a column) at some level.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Node {
    Corner(usize, usize),
    Edge(usize, usize, u8),
    /// Where the frame's outer edge (`FR_W`) crosses a grid edge.
    Outer(usize, usize, u8),
}

/// The grid edge a side of a cell's polygon lies along (None: the contour).
fn grid_edge(a: Node, b: Node) -> Option<(usize, usize, u8)> {
    match (a, b) {
        (Node::Corner(i, j), Node::Corner(k, l)) => {
            let (lo, hi) = if (i, j) < (k, l) { ((i, j), (k, l)) } else { ((k, l), (i, j)) };
            Some((lo.0, lo.1, if lo.0 != hi.0 { 0 } else { 1 }))
        }
        (Node::Corner(..), Node::Edge(i, j, d)) | (Node::Edge(i, j, d), Node::Corner(..)) => Some((i, j, d)),
        _ => None,
    }
}

/// Marching squares on the chart's field `f` at `level`: the cells' parts
/// at or above it (polygons of nodes and their grid places) and the
/// contour's segments, each with the solid side on its left.
struct March {
    polys: Vec<Vec<(Node, Vec2)>>,
    segs: Vec<((Node, Vec2), (Node, Vec2))>,
}

fn march(c: &Chart, f: &[f32], level: f32, skip_above: f32) -> March {
    let mut out = March { polys: Vec::new(), segs: Vec::new() };
    for i in 0..c.nz - 1 {
        for j in 0..c.ns - 1 {
            let ks = [c.k(i, j), c.k(i + 1, j), c.k(i + 1, j + 1), c.k(i, j + 1)];
            if ks.iter().any(|&k| !c.ok[k]) {
                continue;
            }
            let v = ks.map(|k| f[k]);
            if v.iter().all(|&x| x < level) || v.iter().all(|&x| x > skip_above) {
                continue;
            }
            let corner = [(i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1)];
            let solid = v.map(|x| x >= level);
            // The crossings on the four edges (counter-clockwise from (i, j)).
            let edge_node = |e: usize| -> (Node, Vec2) {
                let (a, b) = (e, (e + 1) % 4);
                // Canonical end first, so both cells find the same place.
                let (lo, hi) = if corner[a] < corner[b] { (a, b) } else { (b, a) };
                let t = ((level - v[lo]) / (v[hi] - v[lo])).clamp(0.0, 1.0);
                let pl = Vec2::new(corner[lo].0 as f32, corner[lo].1 as f32);
                let ph = Vec2::new(corner[hi].0 as f32, corner[hi].1 as f32);
                let dir = if corner[lo].0 != corner[hi].0 { 0 } else { 1 };
                (Node::Edge(corner[lo].0, corner[lo].1, dir), pl.lerp(ph, t))
            };
            let cross: Vec<(usize, bool)> = (0..4).filter(|&e| solid[e] != solid[(e + 1) % 4]).map(|e| (e, solid[e])).collect();
            let centre = v.iter().sum::<f32>() * 0.25 >= level;
            // Each exit (solid to empty, going round) with its entry.
            let partner = |k: usize| -> usize {
                let n = cross.len();
                (1..n).map(|s| if centre { (k + s) % n } else { (k + n - s) % n }).find(|&m| !cross[m].1).unwrap()
            };
            for (k, &(e, exit)) in cross.iter().enumerate() {
                if exit {
                    out.segs.push((edge_node(e), edge_node(cross[partner(k)].0)));
                }
            }
            // The solid part(s) of the cell.
            let mut seen = [false; 4];
            for s0 in 0..4 {
                if !solid[s0] || seen[s0] {
                    continue;
                }
                let mut poly = Vec::new();
                let mut s = s0;
                for _ in 0..12 {
                    seen[s] = true;
                    poly.push((Node::Corner(corner[s].0, corner[s].1), Vec2::new(corner[s].0 as f32, corner[s].1 as f32)));
                    let e = s;
                    if let Some(k) = cross.iter().position(|&(ce, ex)| ce == e && ex) {
                        poly.push(edge_node(e));
                        let en = cross[partner(k)].0;
                        poly.push(edge_node(en));
                        s = (en + 1) % 4;
                    } else {
                        s = (s + 1) % 4;
                    }
                    if s == s0 {
                        break;
                    }
                }
                out.polys.push(poly);
            }
        }
    }
    out
}

/// The segments joined into runs (open where the chart ends), in order.
fn chains(segs: &[((Node, Vec2), (Node, Vec2))]) -> Vec<Vec<Vec2>> {
    use std::collections::HashMap;
    let mut from: HashMap<Node, usize> = HashMap::new();
    let mut ends: std::collections::HashSet<Node> = Default::default();
    for (k, s) in segs.iter().enumerate() {
        from.insert(s.0 .0, k);
        ends.insert(s.1 .0);
    }
    let mut used = vec![false; segs.len()];
    let mut out = Vec::new();
    let starts: Vec<usize> = (0..segs.len()).filter(|&k| !ends.contains(&segs[k].0 .0)).chain(0..segs.len()).collect();
    for k0 in starts {
        if used[k0] {
            continue;
        }
        let mut run = vec![segs[k0].0 .1];
        let mut k = k0;
        loop {
            used[k] = true;
            run.push(segs[k].1 .1);
            match from.get(&segs[k].1 .0) {
                Some(&n) if !used[n] => k = n,
                _ => break,
            }
        }
        out.push(run);
    }
    out
}

fn window_frames(m: &mut Mesh) {
    let frame = mt(mat::SATIN, 0.26, 0.31, 0.26);
    let seal = mt(mat::RUBBER, 0.02, 0.02, 0.022);
    let rivet = mt(mat::SCREW, 0.33, 0.37, 0.32);
    let c = Chart::new();
    let dist = win_dists(&c);
    let field: Vec<f32> = (0..c.nz * c.ns)
        .map(|k| {
            if !c.ok[k] {
                return 1.0;
            }
            let ds: Vec<f32> = dist.iter().map(|d| d[k]).collect();
            frame_field(&ds)
        })
        .collect();
    let mut side = Mesh::default();

    // The frame's face: the band from the openings out to where it rounds off.
    let band = march(&c, &field, 0.0, FR_W + 0.004);
    let mut idx: std::collections::HashMap<Node, u32> = Default::default();
    let mut verts: Vec<(Vec3, Vec3, Vec2)> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    let val = |node: Node| match node {
        Node::Corner(i, j) => field[c.k(i, j)],
        Node::Edge(..) => 0.0,
        Node::Outer(..) => FR_W,
    };
    for poly0 in &band.polys {
        // Cut at the frame's outer edge, so it ends on a smooth line.
        let mut poly: Vec<(Node, Vec2)> = Vec::new();
        for k in 0..poly0.len() {
            let (a, b) = (poly0[k], poly0[(k + 1) % poly0.len()]);
            let (va, vb) = (val(a.0), val(b.0));
            if va <= FR_W {
                poly.push(a);
            }
            if (va <= FR_W) != (vb <= FR_W) {
                if let Some((i, j, d)) = grid_edge(a.0, b.0) {
                    let ((ql, vl), (qh, vh)) = if (a.1.x, a.1.y) < (b.1.x, b.1.y) { ((a.1, va), (b.1, vb)) } else { ((b.1, vb), (a.1, va)) };
                    poly.push((Node::Outer(i, j, d), ql.lerp(qh, ((FR_W - vl) / (vh - vl)).clamp(0.0, 1.0))));
                }
            }
        }
        if poly.len() < 3 {
            continue;
        }
        let ids: Vec<u32> = poly
            .iter()
            .map(|&(node, q)| {
                *idx.entry(node).or_insert_with(|| {
                    let (p, n) = c.at(q.x, q.y);
                    let l = fr_lift(val(node));
                    verts.push((p + n * if l < 0.0 { sink(p, n, l) } else { l }, n, q));
                    (verts.len() - 1) as u32
                })
            })
            .collect();
        for k in 1..ids.len().saturating_sub(1) {
            tris.push([ids[0], ids[k], ids[k + 1]]);
        }
    }
    let mut nrm = vec![Vec3::ZERO; verts.len()];
    for t in &tris {
        let (a, b, cc) = (verts[t[0] as usize].0, verts[t[1] as usize].0, verts[t[2] as usize].0);
        let fnrm = (b - a).cross(cc - a);
        for &k in t {
            nrm[k as usize] += fnrm;
        }
    }
    let base = side.v.len() as u32;
    for (k, &(p, n, q)) in verts.iter().enumerate() {
        let nn = if nrm[k].dot(n) < 0.0 { -nrm[k] } else { nrm[k] };
        let (pp, _) = c.at(q.x, q.y);
        side.push(p, if nn.length_squared() > 1e-14 { nn } else { n }, [pp.z, q.y * FR_STEP], &frame, 0.0);
    }
    for t in &tris {
        side.i.extend(t.iter().map(|k| k + base));
    }
    // Closed where the chart ends at the lining's forward end.
    for j in 0..c.ns - 1 {
        let (a, b) = (c.k(0, j), c.k(0, j + 1));
        if !c.ok[a] || !c.ok[b] || (field[a] < 0.0 && field[b] < 0.0) || (field[a] > FR_W && field[b] > FR_W) {
            continue;
        }
        let at = |q: f32| -> (Vec3, f32) {
            let h = field[a] + (field[b] - field[a]) * q;
            (c.at(0.0, j as f32 + q).0, h)
        };
        let (t0, t1) = {
            let (fa, fb) = (field[a], field[b]);
            let cut = if (fa < 0.0) != (fb < 0.0) { fa / (fa - fb) } else { 0.0 };
            if fa < 0.0 { (cut, 1.0) } else if fb < 0.0 { (0.0, cut) } else { (0.0, 1.0) }
        };
        let (pa, ha) = at(t0);
        let (pb, hb) = at(t1);
        let n = c.at(0.0, j as f32 + 0.5).1;
        let ha = if t0 > 0.0 { 0.0 } else { ha };
        let hb = if t1 < 1.0 { 0.0 } else { hb };
        side.quad(&frame, [pa + n * sink(pa, n, -0.004), pb + n * sink(pb, n, -0.004), pb + n * fr_lift(hb), pa + n * fr_lift(ha)]);
    }

    // The reveal round each opening: the frame turned back past the lining,
    // the rubber seal at its back.
    for run in chains(&band.segs) {
        if run.len() < 2 {
            continue;
        }
        let pts: Vec<(Vec3, Vec3)> = run.iter().map(|q| c.at(q.x, q.y)).collect();
        // Into the opening: right of the way the run goes on the chart.
        let into: Vec<Vec3> = (0..run.len())
            .map(|k| {
                let (a, b) = (run[k.saturating_sub(1)], run[(k + 1).min(run.len() - 1)]);
                let d = b - a;
                let w = c.dir(run[k].x, run[k].y, Vec2::new(d.y, -d.x).normalize_or_zero());
                let n = pts[k].1;
                (w - n * w.dot(n)).normalize_or_zero()
            })
            .collect();
        // How deep it goes behind the lining: as deep as the envelope lets it
        // (the least of its neighbours', averaged, so the edge runs smooth).
        let deep: Vec<f32> = pts.iter().map(|&(p, n)| sink(p, n, -0.011)).collect();
        let near = |k: usize, r: usize, f: &dyn Fn(usize) -> f32| -> Vec<f32> { (k.saturating_sub(r)..(k + r + 1).min(run.len())).map(f).collect() };
        let least: Vec<f32> = (0..run.len()).map(|k| near(k, 3, &|i| deep[i]).into_iter().fold(f32::MIN, f32::max)).collect();
        let deep: Vec<f32> = (0..run.len()).map(|k| { let v = near(k, 2, &|i| least[i]); v.iter().sum::<f32>() / v.len() as f32 }).collect();
        for (mat, f0, f1) in [(&frame, -1.0f32, 0.4f32), (&seal, 0.4, 1.0)] {
            let rows: Vec<Vec<SP>> = (0..run.len())
                .map(|k| {
                    let (p, n) = pts[k];
                    let at = |f: f32| p + n * if f < 0.0 { FR_H } else { deep[k] * f };
                    vec![(at(f0), [k as f32, 0.0], 0.0), (at(f1), [k as f32, 1.0], 0.0)]
                })
                .collect();
            let start = side.v.len();
            side.grid(mat, &rows, false, false);
            for (k, v) in side.v[start..].iter_mut().enumerate() {
                v.nrm = into[k / 2].into();
            }
        }
    }

    // Rivets along the middle of the frames and posts.
    let mid = march(&c, &field, FR_RIVET, FR_RIVET + 0.01);
    for run in chains(&mid.segs) {
        let mut acc = FR_PITCH * 0.5;
        for w in run.windows(2) {
            let (a, _) = c.at(w[0].x, w[0].y);
            let (b, _) = c.at(w[1].x, w[1].y);
            acc += (b - a).length();
            if acc >= FR_PITCH {
                acc -= FR_PITCH;
                let (p, n) = c.at(w[1].x, w[1].y);
                side.screw(&rivet, p + n * (FR_H - 0.0004), n, 0.0024, p.x * 37.0 + p.z * 11.0);
            }
        }
    }

    m.append(&side.mirror_x());
    m.append(&side);
    sun_visors(m);
}

/// The sun visors (tinted plastic), stowed flat under the ceiling over each
/// pilot, on the rod they swing down on.
fn sun_visors(m: &mut Mesh) {
    let zv = EYE.z + 0.05;
    for sx in [-1.0f32, 1.0] {
        let p = inside(Vec3::new(sx * 0.53, deck_ceiling(zv) - 0.035, zv), 0.03);
        m.rbox(&mt(mat::PLASTIC, 0.035, 0.05, 0.04), p, Vec3::new(0.10, 0.003, 0.09), 0.003, Quat::IDENTITY);
        m.tube(&CHROME, &[p + Vec3::new(-0.11, 0.006, -0.095), p + Vec3::new(0.11, 0.006, -0.095)], &|_| 0.004, 6, true);
        for dx in [-0.11f32, 0.11] {
            m.rbox(&BLACK_PANEL, p + Vec3::new(dx, 0.012, -0.095), Vec3::new(0.008, 0.012, 0.008), 0.003, Quat::IDENTITY);
        }
    }
}

// ---------------------------------------------------------------- main instrument panel

/// The panel's lean (top aft), rad, and its face.
const PT: f32 = 0.10;
fn panel_face() -> Face {
    Face::new(Vec3::new(0.0, (PANEL_TOP_Y + PANEL_BOT_Y) * 0.5, PANEL_Z), Vec3::X, Vec3::new(0.0, PT.cos(), PT.sin()))
}
/// Half height of the panel along its face.
fn ph() -> f32 {
    (PANEL_TOP_Y - PANEL_BOT_Y) * 0.5 / PT.cos()
}
/// Half width of the main panel's flat part.
const PW: f32 = 1.0;
/// The engine stack's columns (engines 1 to 4) and the pitch of its rows.
const STACK_U: [f32; 4] = [-0.272, -0.204, -0.136, -0.068];
const STACK_PITCH: f32 = 0.0675;
const ENG_R: f32 = 0.0245;
/// The pilots' groups' centres.
const GROUP_U: [f32; 2] = [-0.62, 0.62];
/// The gear handle's place on the panel face (u, and its rest point's height).
const GEAR_U: f32 = 0.072;
/// Panel v of a body height on the face.
fn v_at(y: f32) -> f32 {
    (y - (PANEL_TOP_Y + PANEL_BOT_Y) * 0.5) / PT.cos()
}

fn main_panel(m: &mut Mesh, b: &mut Builder) -> Face {
    let f = panel_face();
    let h = ph();
    // The panel: a slab across the deck with rounded top corners, deep enough
    // behind for the instrument cases; the outboard ends turned back.
    let mut outline = vec![Vec2::new(-PW, -h), Vec2::new(PW, -h)];
    for k in 0..=6 {
        let a = k as f32 / 6.0 * FRAC_PI_2;
        outline.push(Vec2::new(PW - 0.06 + 0.06 * a.cos(), h - 0.06 + 0.06 * a.sin()));
    }
    for k in 0..=6 {
        let a = FRAC_PI_2 + k as f32 / 6.0 * FRAC_PI_2;
        outline.push(Vec2::new(-PW + 0.06 + 0.06 * a.cos(), h - 0.06 + 0.06 * a.sin()));
    }
    m.slab(&PANEL, &outline, &[], f.o, f.ax, f.ay, 0.02);
    m.rbox(&PANEL_DK, f.at_h(0.0, 0.0, -0.12), Vec3::new(PW - 0.02, h - 0.01, 0.10), 0.02, f.rot());
    for sx in [-1.0f32, 1.0] {
        side_panel(m, &f, sx);
    }
    // The divisions: the stack's frame, the centre panel's, the lower edge's lip.
    let g = f.sub(0.0, 0.0, 0.0);
    let stack_l = STACK_U[0] - 0.042;
    let stack_r = STACK_U[3] + 0.042;
    plate(m, &g, stack_l, -h + 0.008, stack_r, h - 0.008, &PANEL, 1.0);
    plate(m, &g, stack_r + 0.004, -h + 0.008, 0.32, h - 0.008, &PANEL, 2.0);
    m.tube(&PANEL_DK, &[f.at_h(-PW, -h, 0.006), f.at_h(PW, -h, 0.006)], &|_| 0.009, 8, true);
    let gs = f.sub(0.0, 0.0, 0.004);

    // The engine stack: four columns of nine.
    for (c, &u) in STACK_U.iter().enumerate() {
        for (r, &id) in gid::ENGINE.iter().enumerate() {
            let v = h - 0.045 - r as f32 * STACK_PITCH;
            gauge(m, &gs, u, v, ENG_R, id, (c * 10 + r) as f32);
        }
        // The engine numbers above each column.
        word(m, &gs, u, h - 0.008, 0.007, ["1", "2", "3", "4"][c], grey());
    }

    // The two flight instrument groups (the same order on each side).
    for (k, &pc) in GROUP_U.iter().enumerate() {
        let s = k as f32 * 100.0 + 10.0;
        plate(m, &g, pc - 0.33, -h + 0.008, pc + 0.27, h - 0.008, &PANEL, s);
        gauge(m, &gs, pc - 0.265, 0.245, 0.026, gid::CLOCK, s + 1.0);
        gauge(m, &gs, pc - 0.140, 0.230, 0.038, gid::ASI, s + 2.0);
        big_gauge(m, &gs, pc + 0.025, 0.215, 0.054, gid::ADI, s + 3.0);
        gauge(m, &gs, pc + 0.190, 0.230, 0.038, gid::ALT, s + 4.0);
        gauge(m, &gs, pc - 0.265, 0.125, 0.026, gid::TURN, s + 5.0);
        gauge(m, &gs, pc - 0.140, 0.100, 0.036, gid::BDHI, s + 6.0);
        big_gauge(m, &gs, pc + 0.025, 0.060, 0.054, gid::HSI, s + 7.0);
        gauge(m, &gs, pc + 0.190, 0.100, 0.036, gid::RADAR_ALT, s + 8.0);
        gauge(m, &gs, pc + 0.190, -0.030, 0.034, gid::VSI, s + 9.0);
        gauge(m, &gs, pc - 0.140, -0.030, 0.030, gid::REST, s + 10.0);
        word(m, &gs, pc - 0.140, -0.072, 0.0055, "", grey());
        gauge(m, &gs, pc - 0.020, -0.075, 0.021, gid::ELEV_TAB, s + 11.0);
        gauge(m, &gs, pc + 0.070, -0.075, 0.021, gid::RUDDER_TAB, s + 12.0);
        // The standby compass's window and its deviation card.
        m.rbox(&CASE, gs.at_h(pc - 0.265, 0.020, 0.012), Vec3::new(0.026, 0.020, 0.012), 0.006, gs.rot());
        rect(m, &mt(mat::GAUGE, 0.02, 0.02, 0.02).x(gid::COMPASS), gs.at_h(pc - 0.265, 0.022, 0.0245), gs.ax, gs.ay, 0.019, 0.012);
        placard(m, &gs, pc - 0.265, -0.030, 0.028, 0.014, false);
        // The fire warning light and its placard, top outboard.
        let fo = pc + if k == 0 { -0.30 } else { 0.24 };
        lamp_lens(m, &gs, fo, 0.29, 0.007, Vec3::new(0.35, 0.03, 0.02));
        word(m, &gs, fo + 0.0, 0.305 - 0.032, 0.0042, "FLASH OVHT", grey());
        // Lower: the navigation selectors and a placard under the trim tab indicators.
        filler(m, &gs, pc - 0.32, -h + 0.02, pc - 0.07, -0.11, s + 20.0, grey());
        placard(m, &gs, pc + 0.045, -0.15, 0.07, 0.025, false);
        filler(m, &gs, pc + 0.13, -h + 0.02, pc + 0.26, -0.11, s + 30.0, grey());
    }
    // Outboard of each group: the selector panels.
    for sx in [-1.0f32, 1.0] {
        let (u0, u1) = if sx < 0.0 { (-PW + 0.01, -0.955) } else { (0.895, PW - 0.01) };
        filler(m, &gs, u0, -h + 0.02, u1, h - 0.07, 300.0 + sx, grey());
    }

    // Right of the stack: the nacelle overheat lights, the gear indicators and
    // handle, the flap indicator, the landing lights, the hydraulic gauges.
    let gx = GEAR_U;
    for k in 0..4 {
        lamp_lens(m, &gs, 0.020 + k as f32 * 0.024, 0.300, 0.0055, Vec3::new(0.35, 0.03, 0.02));
    }
    word(m, &gs, 0.056, 0.272, 0.005, "NACELLE OVHT", grey());
    gauge(m, &gs, 0.200, 0.290, 0.022, gid::REST, 410.0);
    for (k, s) in ["LEFT", "NOSE", "RIGHT"].iter().enumerate() {
        let u = gx - 0.040 + k as f32 * 0.040;
        drum(m, &gs, u, 0.215, 0.012, gid::GEAR);
        word(m, &gs, u, 0.240, 0.0045, s, grey());
    }
    gauge(m, &gs, 0.245, 0.210, 0.030, gid::FLAPS, 411.0);
    // The gear handle's slot and its UP / DN legends.
    let gv = v_at(GEAR_HANDLE.y);
    m.rbox(&BEZEL, gs.at_h(gx, gv, 0.001), Vec3::new(0.011, 0.045, 0.0015), 0.003, gs.rot());
    word(m, &gs, gx + 0.030, gv + 0.035, 0.0055, "UP", grey());
    word(m, &gs, gx + 0.030, gv - 0.035, 0.0055, "DN", grey());
    word(m, &gs, gx - 0.032, gv + 0.020, 0.0042, "LOCK RELEASE", grey());
    rect(m, &mt(mat::PAINT, 0.5, 0.05, 0.03), gs.at_h(gx - 0.032, gv + 0.002, 0.004), gs.ax, gs.ay, 0.006, 0.006);
    // Landing lights: the motor switches (decorative) and the two light
    // switches (`levers`) under their legends.
    plate(m, &gs, 0.170, 0.025, 0.315, 0.165, &BLACK_PANEL, 412.0);
    let gl = gs.sub(0.0, 0.0, 0.004);
    word(m, &gl, 0.2425, 0.150, 0.0058, "LANDING LIGHTS", black());
    for (k, s) in ["LEFT", "RIGHT"].iter().enumerate() {
        let u = 0.205 + k as f32 * 0.075;
        toggle(m, &gl, u, 0.050, true);
        word(m, &gl, u, 0.068, 0.0045, "MOTOR", black());
        word(m, &gl, u, 0.033, 0.0045, s, black());
        word(m, &gl, u, 0.134, 0.0045, "LIGHT", black());
        word(m, &gl, u + 0.020, 0.128, 0.004, "ON", black());
        word(m, &gl, u + 0.020, 0.102, 0.004, "OFF", black());
    }
    word(m, &gl, 0.2425, 0.050, 0.0042, "EXTEND", black());
    // The hydraulic gauges and the pump switches under them.
    for (k, (id, s)) in [(gid::HYD_BRAKES, 0.0), (gid::HYD_AUX, 1.0), (gid::HYD_UTILITY, 2.0), (gid::HYD_BOOSTER, 3.0)].iter().enumerate() {
        gauge(m, &gs, 0.040 + k as f32 * 0.070, -0.030, 0.027, *id, 420.0 + s);
    }
    gauge(m, &gs, 0.285, -0.120, 0.022, gid::HYD_RUDDER, 425.0);
    plate(m, &gs, 0.006, -0.180, 0.245, -0.075, &BLACK_PANEL, 426.0);
    filler(m, &gs.sub(0.0, 0.0, 0.004), 0.01, -0.176, 0.24, -0.08, 427.0, black());
    plate(m, &gs, 0.006, -h + 0.012, 0.315, -0.19, &PANEL, 428.0);
    filler(m, &gs, 0.012, -h + 0.02, 0.31, -0.20, 429.0, grey());
    // The baro knobs on the altimeters, the course knobs on the HSIs.
    for &pc in &GROUP_U {
        let mut kb = Mesh::default();
        let c = gs.at_h(pc + 0.190 - 0.036, 0.230 - 0.036, 0.008);
        kb.lathe(&KNOB.pat(pat::KNURL), c, f.n(), &[[0.0075, 0.0], [0.0075, 0.011], [0.0065, 0.012], [0.0, 0.012]], 14, false);
        let i = b.fixed("altimeter baro knob", G_ENGINE, kb);
        b.pieces[i].hotspot = Some(Hotspot::click(c + f.n() * 0.012, 0.010, "baro_qnh"));
        let mut cb = Mesh::default();
        let c = gs.at_h(pc + 0.025 + 0.052, 0.060 - 0.058, 0.012);
        cb.lathe(&KNOB.pat(pat::KNURL), c, f.n(), &[[0.0075, 0.0], [0.0075, 0.011], [0.0065, 0.012], [0.0, 0.012]], 14, false);
        let i = b.fixed("course knob", G_ENGINE, cb);
        b.pieces[i].hotspot = Some(Hotspot::wheel(c + f.n() * 0.012, 0.010, "obs_right", "obs_left"));
    }
    f
}

/// The side panel each end of the main panel: angled back from the panel's
/// end to the wall, down to the side console and up to under the corner
/// pane's frame, so the panel runs wall to wall as in the photographs. On it
/// the crew member's oxygen regulator (supply, diluter and emergency
/// levers, the flow blinker, the pressure gauge) and the interphone panel.
fn side_panel(m: &mut Mesh, f: &Face, sx: f32) {
    let yb = FLOOR + 0.60;
    let z_o = PANEL_Z + 0.20;
    let top_in = PANEL_TOP_Y - 0.02;
    let top_out = PANEL_TOP_Y - 0.19;
    let pt = |a: f32, b: f32| -> Vec3 {
        let yi = yb + (top_in - yb) * b;
        let yo = yb + (top_out - yb) * b;
        let inb = f.at(sx * PW, v_at(yi));
        let out = Vec3::new(sx * (deck_half_w(z_o, yo) - 0.006), yo, z_o);
        inb.lerp(out, a)
    };
    let rows: Vec<Vec<SP>> = (0..=8).map(|i| (0..=6).map(|j| {
        let (a, b) = (i as f32 / 8.0, j as f32 / 6.0);
        (pt(a, b), [a, b], 0.0)
    }).collect()).collect();
    let start = m.v.len();
    m.grid(&PANEL, &rows, false, false);
    for v in &mut m.v[start..] {
        if v.nrm[2] < 0.0 {
            v.nrm = (-Vec3::from(v.nrm)).into();
        }
    }
    // Its edge against the wall and its top, rolled.
    let edge: Vec<Vec3> = (0..=8).map(|i| pt(i as f32 / 8.0, 1.0)).collect();
    m.tube(&PANEL_DK, &edge, &|_| 0.008, 8, true);
    // A face on it: across from the panel's end outboard, up.
    let (c0, c1) = (pt(0.0, 0.45), pt(1.0, 0.45));
    let o = pt(0.5, 0.45);
    let ax = (c1 - c0).normalize();
    let up = (pt(0.5, 0.9) - pt(0.5, 0.1)).normalize();
    let ay = (up - ax * up.dot(ax)).normalize();
    let fs = Face::new(o, ax, ay);
    let fs = if fs.n().z < 0.0 { Face::new(o, -ax, ay) } else { fs };
    // (u runs inboard to outboard on the left, outboard to inboard on the right.)
    let u = |w: f32| w * -sx;
    let g = fs.sub(0.0, 0.0, 0.004);
    // The oxygen regulator.
    let (r0, r1) = (u(-0.20).min(u(-0.03)), u(-0.20).max(u(-0.03)));
    plate(m, &fs, r0, -0.07, r1, 0.075, &BLACK_PANEL, 730.0 + sx);
    let rc = (r0 + r1) * 0.5;
    word(m, &g, rc, 0.062, 0.0055, "OXYGEN", black());
    gauge(m, &g, rc - 0.045, 0.022, 0.020, gid::REST, 731.0 + sx);
    lamp_lens(m, &g, rc + 0.040, 0.030, 0.007, Vec3::new(0.08, 0.08, 0.075));
    word(m, &g, rc + 0.040, 0.008, 0.004, "", black());
    for (k, (s, up_)) in [("ON", true), ("", false), ("EMER", false)].iter().enumerate() {
        let x = rc - 0.05 + k as f32 * 0.05;
        toggle(m, &g, x, -0.030, *up_);
        word(m, &g, x, -0.056, 0.0036, s, black());
    }
    // The interphone panel and a placard under it.
    let (i0, i1) = (u(0.0).min(u(0.20)), u(0.0).max(u(0.20)));
    plate(m, &fs, i0, -0.07, i1, 0.075, &BLACK_PANEL, 735.0 + sx);
    word(m, &g, (i0 + i1) * 0.5, 0.062, 0.0055, "INTERPHONE", black());
    filler(m, &g, i0 + 0.005, -0.065, i1 - 0.005, 0.05, 736.0 + sx, black());
    placard(m, &fs, u(-0.115), -0.105, 0.07, 0.018, false);
}

// ---------------------------------------------------------------- glareshield

fn glareshield(m: &mut Mesh, f: &Face) {
    let h = ph();
    let lip = f.at(0.0, h);
    let (z_back, z_front) = (lip.z + 0.035, GLARE_Z);
    let y_back = GLARE_Y;
    // The padded top, from the lip forward under the windshield.
    m.patch(&GLARE, 40, 10, true, |a, c| {
        let x = (a - 0.5) * 2.0;
        let z = z_back + (z_front - z_back) * c;
        let y = y_back - 0.030 * c - 0.02 * x.powi(6);
        let p = inside(Vec3::new(x * (PW + 0.04), y, z), 0.01);
        (p, [x, z], 0.0)
    });
    // The rolled lip and the hood down to the panel's top edge.
    let edge: Vec<Vec3> = (0..=30).map(|k| {
        let x = (k as f32 / 30.0 - 0.5) * 2.0;
        inside(Vec3::new(x * (PW + 0.04), y_back - 0.012 - 0.02 * x.powi(6), z_back), 0.02)
    }).collect();
    m.tube(&GLARE, &edge, &|_| 0.013, 10, true);
    m.patch(&GLARE, 30, 2, false, |a, c| {
        let x = (a - 0.5) * 2.0;
        let e = inside(Vec3::new(x * (PW + 0.04), y_back - 0.02 - 0.02 * x.powi(6), z_back), 0.02);
        let p = f.at_h(x * PW, h, 0.0);
        (e.lerp(Vec3::new(e.x, p.y, p.z - 0.01), c), [x, c], 0.0)
    });
    // The altitude alerter on top of the middle.
    let c = Vec3::new(0.0, y_back + 0.028, z_back - 0.07);
    m.rbox(&BLACK_PANEL, c, Vec3::new(0.065, 0.026, 0.05), 0.006, Quat::IDENTITY);
    let fa = Face::new(c + Vec3::new(0.0, 0.0, 0.0505), Vec3::X, Vec3::Y);
    rect(m, &mt(mat::PLASTIC, 0.004, 0.004, 0.005), fa.at_h(0.008, -0.004, 0.0002), fa.ax, fa.ay, 0.028, 0.008);
    word(m, &fa, 0.0, 0.015, 0.0045, "SET ALTITUDE", black());
    lamp_lens(m, &fa, -0.045, -0.004, 0.005, Vec3::new(0.4, 0.04, 0.02));
    m.lathe(&KNOB.pat(pat::KNURL), fa.at(0.05, -0.006), fa.n(), &[[0.006, 0.0], [0.006, 0.01], [0.0, 0.011]], 12, false);
    for sx in [-1.0f32, 1.0] {
        m.rbox(&BLACK_PANEL, c + Vec3::new(sx * 0.072, -0.02, 0.0), Vec3::new(0.008, 0.008, 0.045), 0.003, Quat::IDENTITY);
    }
}

// ---------------------------------------------------------------- pedestal

/// The throttle quadrant: its axis's height, the arc's radius, the arc's
/// half angle. A compact unit at the top of the pedestal just below the
/// centre panel, as in the photographs: its hump 0.6 m above the floor and
/// the grips about 12 cm above that, so that from the pilot's eye the whole
/// engine stack and the gear handle are seen over it. (`dims::PED_TOP_Y` is
/// the envelope's allowance; the pedestal is built lower than it.)
fn q_pivot_y() -> f32 {
    FLOOR + 0.42
}
const Q_R: f32 = 0.125;
const Q_ANG: f32 = 0.78;
/// The power levers' and the condition levers' length from the axis to the grip.
const POWER_LEN: f32 = 0.235;
const COND_LEN: f32 = 0.215;
/// The power levers (engines 1 to 4) and the condition levers.
pub const POWER_LEVER_X: [f32; 4] = [-0.150, -0.113, -0.076, -0.039];
const COND_LEVER_X: [f32; 4] = [0.045, 0.085, 0.125, 0.165];
/// The aft pedestal's top: from just behind the quadrant (level with the
/// arc's aft end) falling 5 cm to its aft end.
fn aft_top(z: f32) -> f32 {
    let z0 = PED_LEVER_Z + Q_R * Q_ANG.sin();
    q_pivot_y() + Q_R * Q_ANG.cos() - 0.004 - 0.05 * ((z - z0) / (PED_AFT_Z - z0)).clamp(0.0, 1.0)
}
/// The aft pedestal's top face (reads from aft: u = x, v forward).
fn aft_face(z: f32) -> Face {
    Face::new(Vec3::new(0.0, aft_top(z) + 0.0005, z), Vec3::X, Vec3::new(0.0, 0.05 / (PED_AFT_Z - PED_LEVER_Z - Q_R * Q_ANG.sin()), -1.0))
}

fn pedestal(m: &mut Mesh, b: &mut Builder) {
    let hw = PED_HALF_W;
    let yq = q_pivot_y();
    let zq = PED_LEVER_Z;
    let zq0 = zq - Q_R * Q_ANG.sin();
    let zq1 = zq + Q_R * Q_ANG.sin();
    let yq_end = yq + Q_R * Q_ANG.cos();
    let zf = PANEL_Z + 0.012;
    // The body: its top from the panel's foot up the quadrant's front, round
    // the arc and back along the aft top; swept between its two sides.
    let mut top: Vec<(f32, f32)> = vec![(zf, PANEL_BOT_Y - 0.01)];
    for k in 0..=3 {
        let t = k as f32 / 3.0;
        top.push((zf + (zq0 - 0.012 - zf) * t, PANEL_BOT_Y - 0.01 + (yq_end - 0.02 - PANEL_BOT_Y + 0.01) * t.powf(0.8)));
    }
    for k in 0..=16 {
        let a = -Q_ANG + 2.0 * Q_ANG * k as f32 / 16.0;
        top.push((zq + a.sin() * (Q_R - 0.008), yq + a.cos() * (Q_R - 0.008)));
    }
    for k in 0..=8 {
        let z = zq1 + 0.01 + (PED_AFT_Z - zq1 - 0.01) * k as f32 / 8.0;
        top.push((z, aft_top(z)));
    }
    let rows: Vec<Vec<SP>> = top.iter().map(|&(z, y)| {
        let pts = [Vec3::new(-hw, FLOOR, z), Vec3::new(-hw, y - 0.012, z), Vec3::new(-hw + 0.012, y, z), Vec3::new(hw - 0.012, y, z), Vec3::new(hw, y - 0.012, z), Vec3::new(hw, FLOOR, z)];
        pts.iter().enumerate().map(|(k, p)| (*p, [z, k as f32 * 0.2], 0.0)).collect()
    }).collect();
    // (Normals out of the body: up on top, outboard on the sides.)
    m.grid(&PANEL_DK, &rows, false, false);
    let (za, ya) = *top.last().unwrap();
    m.quad(&PANEL_DK, [Vec3::new(-hw, FLOOR, za), Vec3::new(hw, FLOOR, za), Vec3::new(hw, ya, za), Vec3::new(-hw, ya, za)]);

    // The quadrant's arc: grey plates with a slot for every lever, the gate legends.
    let qgrey = mt(mat::PLASTIC, 0.15, 0.155, 0.158).pat(pat::PANEL);
    for (x0, x1) in [(-hw + 0.01, -0.012), (0.012, hw - 0.01)] {
        m.patch(&qgrey, 20, 4, true, |a, c| {
            let ang = (a - 0.5) * 2.0 * Q_ANG;
            let x = x0 + (x1 - x0) * c;
            (Vec3::new(x, yq + ang.cos() * Q_R, zq + ang.sin() * Q_R), [x, ang * Q_R], 0.0)
        });
        for (x, sx) in [(x0, -1.0f32), (x1, 1.0)] {
            let ring: Vec<Vec3> = (0..=16).map(|k| { let a = -Q_ANG + 2.0 * Q_ANG * k as f32 / 16.0; Vec3::new(x, yq + a.cos() * Q_R, zq + a.sin() * Q_R) }).collect();
            m.fan(&qgrey, Vec3::new(x, yq + 0.04, zq), &ring, Vec3::X * sx);
        }
    }
    for &x in POWER_LEVER_X.iter().chain(COND_LEVER_X.iter()) {
        m.patch(&BEZEL, 14, 1, true, |a, c| {
            let ang = (a - 0.5) * 2.0 * (Q_ANG - 0.06);
            (Vec3::new(x + (c - 0.5) * 0.007, yq + ang.cos() * (Q_R + 0.0006), zq + ang.sin() * (Q_R + 0.0006)), [a, c], 0.0)
        });
    }
    // The divider between the two halves, the gate legends on the arc.
    m.rbox(&BLACK_PANEL, Vec3::new(0.0, yq + Q_R - 0.022, zq), Vec3::new(0.008, 0.02, Q_R * 0.6), 0.004, Quat::IDENTITY);
    let at_ang = |x: f32, ang: f32| -> Face {
        let p = Vec3::new(x, yq + ang.cos() * (Q_R + 0.001), zq + ang.sin() * (Q_R + 0.001));
        let n = Vec3::new(0.0, ang.cos(), ang.sin());
        Face::new(p, Vec3::X, n.cross(Vec3::X))
    };
    let qc = Vec3::new(0.15, 0.155, 0.158);
    for (s, ang) in [("TAKEOFF", -0.62f32), ("FLIGHT IDLE", 0.20), ("GROUND IDLE", 0.38), ("MAX REVERSE", 0.62)] {
        let fq = at_ang(-0.175, ang);
        word(m, &fq, 0.0, 0.0, 0.0045, s, qc);
    }
    for (s, ang) in [("RUN", -0.38f32), ("GROUND STOP", 0.36)] {
        let fq = at_ang(0.105, ang);
        word(m, &fq, 0.0, 0.0, 0.0042, s, qc);
    }
    // FEATHER, at the aft end of the condition levers' travel: a red painted
    // band across their half of the arc with the word on it (a plain band:
    // fine hazard stripes shimmer at this size).
    let ang = 0.66f32;
    let red = Vec3::new(0.45, 0.03, 0.02);
    rect(m, &mt(mat::PAINT, red.x, red.y, red.z), Vec3::new(0.105, yq + ang.cos() * (Q_R + 0.0008), zq + ang.sin() * (Q_R + 0.0008)), Vec3::X, Vec3::new(0.0, ang.sin(), -ang.cos()), 0.085, 0.009);
    word(m, &at_ang(0.105, ang), 0.0, 0.0, 0.0042, "FEATHER", red);
    // Where the four power levers and four condition levers say what they are.
    let fq = at_ang(-0.095, -0.70);
    word(m, &fq, 0.0, 0.0, 0.004, "POWER", qc);
    let fq = at_ang(0.105, -0.70);
    word(m, &fq, 0.0, 0.0, 0.004, "CONDITION", qc);

    // The quadrant's front: the friction knobs and the panel under it.
    let ff = Face::new(Vec3::new(0.0, (PANEL_BOT_Y + yq_end) * 0.5, (zf + zq0) * 0.5 + 0.004), Vec3::X, Vec3::new(0.0, yq_end - PANEL_BOT_Y, zq0 - zf - 0.012));
    for sx in [-1.0f32, 1.0] {
        m.lathe(&KNOB.pat(pat::KNURL), ff.at(sx * 0.17, 0.065), ff.n(), &[[0.012, 0.0], [0.012, 0.012], [0.009, 0.016], [0.0, 0.016]], 14, false);
    }
    plate(m, &ff, -0.15, -0.095, 0.15, 0.045, &BLACK_PANEL, 500.0);
    filler(m, &ff.sub(0.0, 0.0, 0.004), -0.145, -0.09, 0.145, 0.04, 501.0, black());

    // Behind the quadrant: the synchrophaser and temperature datum valve switches.
    let z_s = zq1 + 0.05;
    let fs = aft_face(z_s);
    plate(m, &fs, -hw + 0.008, -0.035, hw - 0.008, 0.035, &BLACK_PANEL, 510.0);
    let gs = fs.sub(0.0, 0.0, 0.004);
    toggle(m, &gs, -0.155, 0.0, true);
    toggle(m, &gs, -0.105, 0.0, true);
    word(m, &gs, -0.13, -0.022, 0.0045, "SYNCHROPHASE", black());
    for k in 0..4 {
        let u = 0.02 + k as f32 * 0.045;
        toggle(m, &gs, u, 0.0, true);
        word(m, &gs, u, 0.020, 0.005, ["1", "2", "3", "4"][k], black());
    }
    word(m, &gs, 0.088, -0.022, 0.0045, "TD VALVE", black());

    // The trim panel and the autopilot controller.
    let z_t = z_s + 0.11;
    let ft = aft_face(z_t);
    plate(m, &ft, -hw + 0.008, -0.055, -0.005, 0.055, &BLACK_PANEL, 520.0);
    let gt = ft.sub(0.0, 0.0, 0.004);
    word(m, &gt, -0.10, 0.044, 0.0055, "TRIM", black());
    knob(m, &gt, -0.15, -0.005, 0.009, 521.0, black());
    word(m, &gt, -0.15, -0.034, 0.004, "AILERON TRIM", black());
    knob(m, &gt, -0.045, -0.025, 0.008, 522.0, black());
    word(m, &gt, -0.045, -0.048, 0.004, "RUDDER TRIM", black());
    word(m, &gt, -0.06, 0.028, 0.0042, "NOSE DOWN", black());
    word(m, &gt, -0.06, -0.002, 0.0042, "NOSE UP", black());
    word(m, &gt, -0.095, 0.012, 0.0042, "ELEV TAB", black());
    plate(m, &ft, 0.005, -0.055, hw - 0.008, 0.055, &BLACK_PANEL, 523.0);
    let ga = ft.sub(0.0, 0.0, 0.004);
    word(m, &ga, 0.10, 0.044, 0.0055, "AUTOPILOT", black());
    knob(m, &ga, 0.06, 0.005, 0.011, 524.0, black());
    m.rbox(&KNOB, ga.at_h(0.13, 0.005, 0.008), Vec3::new(0.012, 0.004, 0.008), 0.003, ga.rot());
    word(m, &ga, 0.13, 0.025, 0.0042, "ENGAGE", black());
    filler(m, &ga, 0.03, -0.052, 0.19, -0.015, 525.0, black());
    // The elevator tab switch (momentary: it shows nothing; the elevator tab
    // indicator on the panel shows the trim).
    let mut es = Mesh::default();
    toggle(&mut es, &gt, -0.06, 0.013, true);
    let i = b.fixed("elevator tab switch", G_ENGINE, es);
    b.pieces[i].hotspot = Some(Hotspot::wheel(gt.at_h(-0.06, 0.016, 0.015), 0.012, "trim_down", "trim_up"));

    // The flap lever's gate and scale at the aft end.
    let z_f = PED_AFT_Z - 0.12;
    let fl = aft_face(z_f);
    plate(m, &fl, -hw + 0.008, -0.10, hw - 0.008, 0.11, &BLACK_PANEL, 530.0);
    let gf = fl.sub(0.0, 0.0, 0.004);
    m.rbox(&BEZEL, gf.at_h(FLAP_X, 0.0, 0.001), Vec3::new(0.006, 0.085, 0.0015), 0.002, gf.rot());
    for (k, s) in ["0", "50", "100"].iter().enumerate() {
        let v = 0.07 - k as f32 * 0.07;
        word(m, &gf, FLAP_X + 0.028, v, 0.0055, s, black());
        m.rbox(&mt(mat::PLASTIC, 0.7, 0.7, 0.68), gf.at_h(FLAP_X + 0.012, v, 0.0005), Vec3::new(0.006, 0.0008, 0.0005), 0.0003, gf.rot());
    }
    word(m, &gf, FLAP_X + 0.03, 0.095, 0.0042, "FLAPS", black());
    word(m, &gf, FLAP_X + 0.035, -0.088, 0.004, "PERCENT", black());
    filler(m, &gf, -hw + 0.012, -0.095, FLAP_X - 0.03, 0.10, 531.0, black());
    // Radio panels down the pedestal's sides.
    for sx in [-1.0f32, 1.0] {
        let z0 = zq1 + 0.02;
        let fsd = Face::new(Vec3::new(sx * (hw + 0.0005), (FLOOR + aft_top(PED_AFT_Z)) * 0.5 - 0.02, (z0 + PED_AFT_Z) * 0.5), Vec3::new(0.0, 0.0, -sx), Vec3::Y);
        let hz = (PED_AFT_Z - z0) * 0.5 - 0.01;
        plate(m, &fsd, -hz, -0.12, hz, 0.12, &BLACK_PANEL, 540.0 + sx);
        filler(m, &fsd.sub(0.0, 0.0, 0.004), -hz + 0.01, -0.115, hz - 0.01, 0.115, 541.0 + sx, black());
    }
}
/// The flap lever's slot across the aft pedestal.
const FLAP_X: f32 = 0.10;

// ---------------------------------------------------------------- overhead

/// The overhead's underside (text tops aft, as the crew read it looking up).
fn ovhd_face() -> Face {
    Face::new(Vec3::new(0.0, OVHD_LOW_Y, (OVHD_FWD_Z + OVHD_AFT_Z) * 0.5), Vec3::X, Vec3::Z)
}

fn overhead(m: &mut Mesh, b: &mut Builder) {
    let f = ovhd_face();
    let hw = OVHD_HALF_W;
    let hl = (OVHD_AFT_Z - OVHD_FWD_Z) * 0.5;
    // The box up to the ceiling.
    for sx in [-1.0f32, 1.0] {
        let x = sx * hw;
        let rows: Vec<Vec<SP>> = (0..=8).map(|k| {
            let z = OVHD_FWD_Z + (OVHD_AFT_Z - OVHD_FWD_Z) * k as f32 / 8.0;
            vec![(Vec3::new(x, OVHD_LOW_Y - 0.004, z), [z, 0.0], 0.0), (Vec3::new(x, deck_ceiling(z) - 0.002, z), [z, 0.2], 0.0)]
        }).collect();
        m.grid(&BLACK_PANEL, &rows, false, sx < 0.0);
    }
    for z in [OVHD_FWD_Z, OVHD_AFT_Z] {
        let yc = deck_ceiling(z) - 0.002;
        m.quad(&BLACK_PANEL, [Vec3::new(-hw, OVHD_LOW_Y - 0.004, z), Vec3::new(hw, OVHD_LOW_Y - 0.004, z), Vec3::new(hw, yc, z), Vec3::new(-hw, yc, z)]);
    }
    m.rbox(&BLACK_PANEL, f.at_h(0.0, 0.0, -0.006), Vec3::new(hw, hl, 0.006), 0.003, f.rot());
    let bg = black();
    let g = f.sub(0.0, 0.0, 0.0);

    // The fire emergency panel along the forward edge: hazard stripes, the
    // agent discharge switch, the five T-handles (engines 1 to 4, the GTC).
    let v0 = -hl;
    let stripes = mt(mat::PAINT, 0.62, 0.46, 0.03).pat(pat::STRIPE).x(0.03);
    m.rbox(&stripes, g.at_h(0.0, v0 + 0.055, 0.002), Vec3::new(hw - 0.01, 0.05, 0.002), 0.001, g.rot());
    word(m, &g.sub(0.0, 0.0, 0.0045), 0.0, v0 + 0.085, 0.0065, "FIRE EMERGENCY", Vec3::new(0.62, 0.46, 0.03));
    guarded(m, &g.sub(0.0, 0.0, 0.004), 0.0, v0 + 0.055, true);
    let clear = mt(mat::GLASS, 0.5, 0.48, 0.45);
    let _ = clear;
    for k in 0..5 {
        let u = [-0.30f32, -0.17, 0.12, 0.25, 0.36][k];
        let c = g.at_h(u, v0 + 0.03, 0.0);
        m.tube(&CHROME, &[c, c - Vec3::Y * 0.035], &|_| 0.004, 8, true);
        let hc = c - Vec3::Y * 0.042;
        m.rbox(&mt(mat::PLASTIC, 0.55, 0.52, 0.48), hc, Vec3::new(0.045, 0.009, 0.012), 0.008, Quat::IDENTITY);
        m.rbox(&mt(mat::PLASTIC, 0.40, 0.04, 0.03), hc + Vec3::new(0.0, -0.002, 0.0), Vec3::new(0.025, 0.006, 0.008), 0.004, Quat::IDENTITY);
        let hf = Face::new(hc + Vec3::new(0.0, -0.0092, 0.0), Vec3::X, Vec3::Z);
        word(m, &hf, 0.0, 0.0, 0.007, ["1", "2", "3", "4", "GTC"][k], Vec3::new(0.55, 0.52, 0.48));
    }

    // Columns: electrical (left), engine start / fuel (middle), anti-ice and air conditioning (right).
    let gi = g.sub(0.0, 0.0, 0.0045);
    // Engine start: four buttons (`levers`) and the start selector.
    plate(m, &g, -0.14, v0 + 0.11, 0.14, v0 + 0.20, &BLACK_PANEL, 600.0);
    word(m, &gi, 0.0, v0 + 0.19, 0.0055, "ENG START", bg);
    knob(m, &gi, 0.11, v0 + 0.15, 0.008, 601.0, bg);
    // Electrical: the battery switch (`levers`), generators, the meters.
    plate(m, &g, -hw + 0.01, v0 + 0.11, -0.15, v0 + 0.36, &BLACK_PANEL, 610.0);
    word(m, &gi, -0.275, v0 + 0.35, 0.0055, "ELECTRICAL", bg);
    word(m, &gi, BATT_U, v0 + 0.175, 0.0045, "BATTERY", bg);
    word(m, &gi, BATT_U, v0 + 0.12, 0.0042, "OFF", bg);
    word(m, &gi, BATT_U, v0 + 0.22, 0.0042, "ON", bg);
    for k in 0..4 {
        let u = -0.33 + k as f32 * 0.04;
        toggle(m, &gi, u, v0 + 0.28, true);
        word(m, &gi, u, v0 + 0.305, 0.0042, "GEN", bg);
        word(m, &gi, u, v0 + 0.262, 0.0045, ["1", "2", "3", "4"][k], bg);
    }
    gauge(m, &g, -0.30, v0 + 0.205, 0.022, gid::REST, 611.0);
    // Fuel: the flow lines, a quantity gauge a tank, the total, the pump switches.
    plate(m, &g, -0.145, v0 + 0.205, 0.145, v0 + 0.62, &BLACK_PANEL, 620.0);
    word(m, &gi, 0.0, v0 + 0.61, 0.0055, "FUEL", bg);
    rect(m, &legend_mat(style::LINES, 0, black()), gi.at_h(0.0, v0 + 0.41, 0.0), gi.ax, gi.ay, 0.135, 0.19);
    for (k, u) in [-0.105f32, -0.035, 0.035, 0.105].iter().enumerate() {
        gauge(m, &gi, *u, v0 + 0.53, 0.024, gid::FUEL_TANK, 621.0 + k as f32);
        word(m, &gi, *u, v0 + 0.497, 0.0045, ["1", "2", "3", "4"][k], bg);
        toggle(m, &gi, *u, v0 + 0.46, true);
        word(m, &gi, *u, v0 + 0.44, 0.004, "BOOST PUMP", bg);
        knob(m, &gi, *u, v0 + 0.39, 0.009, 625.0 + k as f32, bg);
        word(m, &gi, *u, v0 + 0.365, 0.004, "CROSSFEED", bg);
    }
    gauge(m, &gi, 0.0, v0 + 0.29, 0.028, gid::FUEL_TOTAL, 630.0);
    for sx in [-1.0f32, 1.0] {
        gauge(m, &gi, sx * 0.08, v0 + 0.27, 0.020, gid::FUEL_TANK, 631.0 + sx);
        toggle(m, &gi, sx * 0.12, v0 + 0.23, true);
        word(m, &gi, sx * 0.12, v0 + 0.215, 0.004, "DUMP", bg);
    }
    // Anti-ice, air conditioning and pressurisation on the right.
    plate(m, &g, 0.15, v0 + 0.11, hw - 0.01, v0 + 0.36, &BLACK_PANEL, 640.0);
    word(m, &gi, 0.275, v0 + 0.35, 0.0055, "ANTI-ICE", bg);
    filler(m, &gi, 0.155, v0 + 0.115, hw - 0.015, v0 + 0.335, 641.0, bg);
    plate(m, &g, 0.15, v0 + 0.36, hw - 0.01, v0 + 0.62, &BLACK_PANEL, 642.0);
    gauge(m, &g, 0.22, v0 + 0.56, 0.024, gid::REST, 643.0);
    gauge(m, &g, 0.33, v0 + 0.56, 0.024, gid::REST, 644.0);
    filler(m, &gi, 0.155, v0 + 0.365, hw - 0.015, v0 + 0.51, 645.0, bg);
    plate(m, &g, -hw + 0.01, v0 + 0.36, -0.15, v0 + 0.62, &BLACK_PANEL, 650.0);
    filler(m, &gi, -hw + 0.015, v0 + 0.365, -0.155, v0 + 0.615, 651.0, bg);
    // Aft: lights, NESA and pitot heat, circuit breakers.
    plate(m, &g, -hw + 0.01, v0 + 0.62, hw - 0.01, hl - 0.01, &BLACK_PANEL, 660.0);
    breakers(m, &gi, -hw + 0.03, v0 + 0.64, -0.06, hl - 0.03, 661.0, bg);
    filler(m, &gi, -0.05, v0 + 0.64, hw - 0.02, hl - 0.03, 662.0, bg);
    let _ = b;
}
/// The battery switch's place on the overhead's electrical panel (u).
const BATT_U: f32 = -0.195;
/// The engine start buttons (u) on the overhead.
const START_U: [f32; 4] = [-0.105, -0.050, 0.005, 0.060];

// ---------------------------------------------------------------- seats

fn seats(m: &mut Mesh) {
    for s in [SEAT_L, SEAT_R] {
        let (x, zc) = (s.x, s.z);
        let top = s.y;
        // The base on its tracks, the pan, the cushion.
        m.rbox(&SEAT_FRAME, Vec3::new(x, (FLOOR + top) * 0.5 - 0.06, zc + 0.04), Vec3::new(0.17, (top - FLOOR) * 0.5 - 0.07, 0.20), 0.02, Quat::IDENTITY);
        m.rbox(&SEAT_FRAME, Vec3::new(x, top - 0.09, zc), Vec3::new(0.25, 0.03, 0.25), 0.02, Quat::IDENTITY);
        m.rbox(&SEAT_FAB, Vec3::new(x, top - 0.035, zc - 0.01), Vec3::new(0.235, 0.04, 0.235), 0.035, Quat::from_rotation_x(-0.06));
        // The back, leaning back, with its headrest.
        let lean = Quat::from_rotation_x(0.16);
        let bc = Vec3::new(x, top + 0.36, zc + 0.30);
        m.rbox(&SEAT_FRAME, bc + Vec3::new(0.0, 0.0, 0.05), Vec3::new(0.245, 0.36, 0.035), 0.03, lean);
        m.rbox(&SEAT_BACK, bc, Vec3::new(0.23, 0.34, 0.045), 0.04, lean);
        m.rbox(&SEAT_BACK, Vec3::new(x, top + 0.80, zc + 0.36), Vec3::new(0.13, 0.08, 0.05), 0.04, lean);
        // Armrests, the outboard one hinged up out of the way.
        for ax in [-1.0f32, 1.0] {
            let p = Vec3::new(x + ax * 0.27, top + 0.17, zc + 0.05);
            m.rbox(&mt(mat::PLASTIC, 0.06, 0.065, 0.068), p, Vec3::new(0.03, 0.03, 0.15), 0.025, Quat::IDENTITY);
            m.rbox(&SEAT_FRAME, p + Vec3::new(0.0, -0.10, 0.12), Vec3::new(0.02, 0.09, 0.02), 0.01, Quat::IDENTITY);
        }
        // The harness: shoulder straps from the inertia reel over the back,
        // the lap belts, the rotary buckle.
        let reel = bc + Vec3::new(0.0, 0.37, 0.04);
        m.rbox(&SEAT_FRAME, reel, Vec3::new(0.05, 0.025, 0.03), 0.01, Quat::IDENTITY);
        let buckle = Vec3::new(x, top + 0.02, zc - 0.12);
        for dx in [-0.07f32, 0.07] {
            let a = reel + Vec3::new(dx, 0.0, -0.03);
            let pts = [a, a + Vec3::new(0.0, -0.05, -0.08), Vec3::new(x + dx * 1.3, top + 0.30, zc + 0.02), buckle + Vec3::new(dx * 0.4, 0.02, 0.0)];
            m.tube(&HARNESS, &pts, &|_| 0.011, 6, true);
            let lap = [Vec3::new(x + dx * 3.2, top - 0.02, zc + 0.08), Vec3::new(x + dx * 2.2, top + 0.03, zc - 0.05), buckle + Vec3::new(dx * 0.6, 0.0, 0.0)];
            m.tube(&HARNESS, &lap, &|_| 0.011, 6, true);
        }
        m.lathe(&CHROME, buckle, Vec3::new(0.0, 0.4, -1.0), &[[0.032, 0.0], [0.032, 0.012], [0.025, 0.016], [0.0, 0.017]], 16, false);
    }
}

// ---------------------------------------------------------------- side consoles

fn side_consoles(m: &mut Mesh, b: &mut Builder) {
    let yt = FLOOR + 0.60;
    let (z0, z1) = (PANEL_Z + 0.03, SEAT_L.z + 0.30);
    for sx in [-1.0f32, 1.0] {
        let x0 = sx * 0.93;
        // The console's top out to the wall, its inboard face.
        m.patch(&BLACK_PANEL, 16, 4, sx > 0.0, |a, c| {
            let z = z0 + (z1 - z0) * a;
            let xw = deck_half_w(z, yt) - 0.004;
            (Vec3::new(x0 + (sx * xw - x0) * c, yt, z), [z, c], 0.0)
        });
        m.quad(&PANEL_DK, if sx < 0.0 { [Vec3::new(x0, FLOOR, z1), Vec3::new(x0, FLOOR, z0), Vec3::new(x0, yt, z0), Vec3::new(x0, yt, z1)] } else { [Vec3::new(x0, FLOOR, z0), Vec3::new(x0, FLOOR, z1), Vec3::new(x0, yt, z1), Vec3::new(x0, yt, z0)] });
        m.tube(&PANEL_DK, &[Vec3::new(x0, yt - 0.008, z0), Vec3::new(x0, yt - 0.008, z1)], &|_| 0.01, 8, true);
        let xw = |z: f32| sx * (deck_half_w(z, yt) - 0.004);
        m.quad(&PANEL_DK, if sx < 0.0 { [Vec3::new(x0, FLOOR, z1), Vec3::new(x0, yt, z1), Vec3::new(xw(z1), yt, z1), Vec3::new(xw(z1), FLOOR, z1)] } else { [Vec3::new(x0, FLOOR, z1), Vec3::new(xw(z1), FLOOR, z1), Vec3::new(xw(z1), yt, z1), Vec3::new(x0, yt, z1)] });
        // Its forward face, under the side panel.
        m.quad(&PANEL_DK, if sx < 0.0 { [Vec3::new(x0, FLOOR, z0), Vec3::new(xw(z0), FLOOR, z0), Vec3::new(xw(z0), yt, z0), Vec3::new(x0, yt, z0)] } else { [Vec3::new(x0, FLOOR, z0), Vec3::new(x0, yt, z0), Vec3::new(xw(z0), yt, z0), Vec3::new(xw(z0), FLOOR, z0)] });
        // The trim panel along the wall above it, from the side panel back past
        // the seat: riveted sheet, the map case (a canvas pocket) on it.
        let (tz0, tz1, ty0, ty1) = (PANEL_Z + 0.19, z1, yt + 0.002, yt + 0.16);
        let rows: Vec<Vec<SP>> = (0..=24).map(|i| {
            let z = tz0 + (tz1 - tz0) * i as f32 / 24.0;
            (0..=4).map(|j| {
                let y = ty0 + (ty1 - ty0) * j as f32 / 4.0;
                (Vec3::new(sx * (deck_half_w(z, y) - 0.008), y, z), [z, y], 0.0)
            }).collect()
        }).collect();
        let start = m.v.len();
        m.grid(&PANEL_DK, &rows, false, false);
        for v in &mut m.v[start..] {
            if v.nrm[0] * sx > 0.0 {
                v.nrm = (-Vec3::from(v.nrm)).into();
            }
        }
        let rail: Vec<Vec3> = (0..=24).map(|i| { let z = tz0 + (tz1 - tz0) * i as f32 / 24.0; Vec3::new(sx * (deck_half_w(z, ty1) - 0.010), ty1, z) }).collect();
        m.tube(&PANEL_DK, &rail, &|_| 0.006, 8, true);
        for i in 0..=((tz1 - tz0) / 0.07) as usize {
            let z = tz0 + 0.02 + i as f32 * 0.07;
            for y in [ty0 + 0.02, ty1 - 0.02] {
                let p = Vec3::new(sx * (deck_half_w(z, y) - 0.0085), y, z);
                m.screw(&SCREW_BLACK, p, Vec3::new(-sx, 0.0, 0.0), 0.0025, z * 31.0 + y);
            }
        }
        let mz = z1 - 0.42;
        let mw = deck_half_w(mz - 0.13, yt + 0.175).min(deck_half_w(mz + 0.13, yt + 0.175));
        let mc = Vec3::new(sx * (mw - 0.03), yt + 0.10, mz);
        m.rbox(&mt(mat::FABRIC, 0.09, 0.10, 0.08).pat(pat::SEAT), mc, Vec3::new(0.018, 0.075, 0.13), 0.012, Quat::IDENTITY);
        m.rbox(&mt(mat::FABRIC, 0.07, 0.075, 0.06), mc + Vec3::new(-sx * 0.006, 0.07, 0.0), Vec3::new(0.016, 0.008, 0.125), 0.004, Quat::IDENTITY);
        // Panels on it: lights and the radio control; a map case aft.
        let f = Face::new(Vec3::new(x0 + sx * 0.20, yt + 0.0005, (z0 + z1) * 0.5), Vec3::new(sx, 0.0, 0.0), Vec3::new(0.0, 0.0, -sx));
        let f = if f.n().y < 0.0 { Face::new(f.o, -f.ax, f.ay) } else { f };
        let hz = (z1 - z0) * 0.5;
        let reach = (deck_half_w(z0 + 0.2, yt) - 0.93 - 0.04) * 0.5;
        plate(m, &f, -0.16, -hz + 0.25, 0.16, hz - 0.30, &BLACK_PANEL, 700.0 + sx);
        filler(m, &f.sub(0.0, 0.0, 0.004), -0.155, -hz + 0.255, 0.155, hz - 0.305, 701.0 + sx, black());
        m.rbox(&mt(mat::FABRIC, 0.10, 0.11, 0.09), Vec3::new(x0 + sx * 0.2, yt + 0.06, z1 - 0.15), Vec3::new(0.12, 0.06, 0.12), 0.02, Quat::IDENTITY);
        let _ = reach;
        // The oxygen hose and mask stowed on the wall.
        let hp = inside(Vec3::new(sx * 2.0, yt + 0.25, z1 - 0.25), 0.05);
        m.tube(&mt(mat::RUBBER, 0.05, 0.10, 0.06).pat(pat::HOSE), &[hp, hp + Vec3::new(-sx * 0.04, -0.15, 0.05), Vec3::new(hp.x - sx * 0.06, yt + 0.03, hp.z + 0.12)], &|_| 0.014, 10, true);
        m.dome(&mt(mat::RUBBER, 0.04, 0.12, 0.08), hp + Vec3::new(-sx * 0.02, 0.02, 0.0), Vec3::new(-sx, 0.2, 0.0), 0.045, 0.06, 14, 0.0);
    }
    // The nose wheel steering wheel on the pilot's console, forward.
    let tc = Vec3::new(-1.10, yt + 0.045, PANEL_Z + 0.22);
    let ax = Vec3::new(0.3, 1.0, 0.25).normalize();
    m.rbox(&BLACK_PANEL, tc - ax * 0.02, Vec3::new(0.06, 0.02, 0.06), 0.01, Quat::IDENTITY);
    let mut w = Mesh::default();
    let u = ax.any_orthonormal_vector();
    let v = ax.cross(u);
    let rim: Vec<Vec3> = (0..=32).map(|k| { let a = k as f32 / 32.0 * TAU; tc + ax * 0.03 + (u * a.cos() + v * a.sin()) * 0.068 }).collect();
    w.tube(&YOKE_MAT, &rim, &|_| 0.008, 8, false);
    for k in 0..3 {
        let a = k as f32 / 3.0 * TAU;
        w.tube(&YOKE_MAT, &[tc + ax * 0.01, tc + ax * 0.03 + (u * a.cos() + v * a.sin()) * 0.064], &|_| 0.005, 6, true);
    }
    w.lathe(&YOKE_MAT, tc, ax, &[[0.018, 0.0], [0.018, 0.03], [0.0, 0.032]], 12, false);
    let knobp = tc + ax * 0.03 + u * 0.068;
    w.tube(&CHROME, &[knobp, knobp + ax * 0.035], &|_| 0.006, 8, true);
    b.add("nose wheel steering", G_ENGINE, w, Anim::hinge(Drive::new(Input::Steer, -2.0)), tc, ax, None);
    word(m, &Face::new(tc + Vec3::new(0.06, -0.044, -0.09), Vec3::X, Vec3::NEG_Z), 0.0, 0.0, 0.005, "NOSE", black());
    // The ramp and aft door switch's plate (the switch itself is in `levers`).
    let rf = Face::new(Vec3::new(-1.00, yt + 0.0005, PANEL_Z + 0.50), Vec3::X, Vec3::NEG_Z);
    plate(m, &rf, -0.05, -0.04, 0.05, 0.05, &BLACK_PANEL, 720.0);
    let rg = rf.sub(0.0, 0.0, 0.004);
    word(m, &rg, 0.0, 0.032, 0.0055, "RAMP/DOOR", black());
    word(m, &rg, 0.0, -0.025, 0.0045, "OPEN", black());
    word(m, &rg, 0.0, 0.015, 0.0045, "CLOSE", black());
}

// ---------------------------------------------------------------- behind the pilots

fn aft_deck(m: &mut Mesh) {
    // The flight engineer's seat: raised on a pedestal behind the centre
    // pedestal, so he can reach the overhead.
    let fe = Vec3::new(0.12, FLOOR + 0.62, PED_AFT_Z + 0.55);
    m.rbox(&SEAT_FRAME, Vec3::new(fe.x, (FLOOR + fe.y) * 0.5, fe.z), Vec3::new(0.10, (fe.y - FLOOR) * 0.5 - 0.02, 0.12), 0.02, Quat::IDENTITY);
    m.rbox(&SEAT_FAB, fe + Vec3::new(0.0, 0.03, 0.0), Vec3::new(0.22, 0.04, 0.21), 0.035, Quat::IDENTITY);
    m.rbox(&SEAT_BACK, fe + Vec3::new(0.0, 0.40, 0.23), Vec3::new(0.21, 0.30, 0.04), 0.035, Quat::from_rotation_x(0.12));
    m.rbox(&SEAT_FRAME, Vec3::new(fe.x, FLOOR + 0.30, fe.z - 0.26), Vec3::new(0.20, 0.015, 0.06), 0.01, Quat::IDENTITY);
    for dx in [-0.06f32, 0.06] {
        m.tube(&HARNESS, &[fe + Vec3::new(dx, 0.68, 0.25), fe + Vec3::new(dx * 1.4, 0.35, 0.10), fe + Vec3::new(dx * 0.4, 0.07, -0.12)], &|_| 0.011, 6, true);
    }
    // The navigator's station on the right: the table against the wall, the
    // radio rack over it, the lamp, the seat turned outboard.
    let (nz0, nz1) = (DECK_AFT_Z - 1.75, DECK_AFT_Z - 0.85);
    let yt = FLOOR + 0.76;
    let zc = (nz0 + nz1) * 0.5;
    let xw = deck_half_w(zc, yt) - 0.01;
    let xt = xw - 0.55;
    m.rbox(&WOOD, Vec3::new((xt + xw) * 0.5, yt, zc), Vec3::new((xw - xt) * 0.5, 0.012, (nz1 - nz0) * 0.5), 0.006, Quat::IDENTITY);
    m.rbox(&CHROME, Vec3::new(xt + 0.01, yt - 0.02, zc), Vec3::new(0.008, 0.012, (nz1 - nz0) * 0.5), 0.004, Quat::IDENTITY);
    let rack = Face::new(Vec3::new(deck_half_w(zc - 0.40, yt + 0.62) - 0.17, yt + 0.36, zc), Vec3::new(0.0, 0.0, 1.0), Vec3::Y);
    let rack = if rack.n().x > 0.0 { Face::new(rack.o, -rack.ax, rack.ay) } else { rack };
    m.rbox(&mt(mat::SATIN, 0.20, 0.22, 0.22), rack.at_h(0.0, 0.0, -0.08), Vec3::new(0.40, 0.26, 0.08), 0.01, rack.rot());
    for r in 0..3 {
        let v0 = -0.25 + r as f32 * 0.165;
        plate(m, &rack, -0.38, v0, 0.38, v0 + 0.15, &BLACK_PANEL, 800.0 + r as f32);
        filler(m, &rack.sub(0.0, 0.0, 0.004), -0.37, v0 + 0.01, 0.37, v0 + 0.14, 810.0 + r as f32, black());
    }
    let lampp = rack.at_h(0.33, 0.20, 0.05);
    m.tube(&BLACK_PANEL, &[lampp, lampp + Vec3::new(-0.08, 0.02, 0.0), lampp + Vec3::new(-0.16, -0.06, 0.0)], &|_| 0.006, 6, true);
    m.lathe(&BLACK_PANEL, lampp + Vec3::new(-0.16, -0.06, 0.0), Vec3::NEG_Y, &[[0.012, 0.0], [0.03, 0.05], [0.0, 0.05]], 12, false);
    let ns = Vec3::new(xt - 0.30, FLOOR + 0.48, zc);
    m.tube(&SEAT_FRAME, &[Vec3::new(ns.x, FLOOR, ns.z), ns - Vec3::Y * 0.05], &|_| 0.03, 10, true);
    m.lathe(&SEAT_FRAME, Vec3::new(ns.x, FLOOR, ns.z), Vec3::Y, &[[0.22, 0.0], [0.22, 0.02], [0.0, 0.02]], 5, false);
    m.rbox(&SEAT_FAB, ns, Vec3::new(0.21, 0.04, 0.21), 0.035, Quat::IDENTITY);
    m.rbox(&SEAT_BACK, ns + Vec3::new(-0.22, 0.32, 0.0), Vec3::new(0.035, 0.26, 0.20), 0.03, Quat::from_rotation_z(-0.12));

    // The crew bunk along the left wall, aft, its curtain rail.
    let (bz0, bz1) = (DECK_AFT_Z - 1.90, DECK_AFT_Z - 0.06);
    let yb = FLOOR + 0.48;
    // (the wall at its narrowest from the floor up to the cushion's top)
    let xwb = -([bz0, bz1].iter().flat_map(|&z| [0.01f32, 0.3, 0.62].map(|dy| deck_half_w(z, FLOOR + dy))).fold(f32::MAX, f32::min) - 0.02);
    let xi = xwb + 0.66;
    m.rbox(&SEAT_FRAME, Vec3::new((xwb + xi) * 0.5, (FLOOR + yb) * 0.5, (bz0 + bz1) * 0.5), Vec3::new((xi - xwb) * 0.5, (yb - FLOOR) * 0.5, (bz1 - bz0) * 0.5), 0.01, Quat::IDENTITY);
    m.rbox(&mt(mat::FABRIC, 0.10, 0.12, 0.08).pat(pat::SEAT), Vec3::new((xwb + xi) * 0.5, yb + 0.05, (bz0 + bz1) * 0.5), Vec3::new((xi - xwb) * 0.5 - 0.01, 0.05, (bz1 - bz0) * 0.5 - 0.01), 0.04, Quat::IDENTITY);
    m.rbox(&mt(mat::FABRIC, 0.12, 0.13, 0.10), Vec3::new(xwb + 0.25, yb + 0.12, bz1 - 0.20), Vec3::new(0.18, 0.05, 0.12), 0.04, Quat::IDENTITY);
    let ry = FLOOR + 1.35;
    m.tube(&CHROME, &[Vec3::new(xi, ry, bz0), Vec3::new(xi, ry, bz1)], &|_| 0.007, 8, true);
    let curtain = mt(mat::FABRIC, 0.20, 0.24, 0.17).pat(pat::SEAT);
    m.rbox(&curtain, Vec3::new(xi, (ry + yb + 0.1) * 0.5, bz0 + 0.20), Vec3::new(0.01, (ry - yb - 0.1) * 0.5, 0.18), 0.008, Quat::IDENTITY);

    // On the aft bulkhead: the door to the cargo compartment, an extinguisher,
    // a portable oxygen bottle, the crash axe, placards.
    let zb = DECK_AFT_Z - 0.004;
    let door = Face::new(Vec3::new(0.55, FLOOR + 0.92, zb), Vec3::NEG_X, Vec3::Y);
    m.rbox(&mt(mat::SATIN, 0.28, 0.33, 0.28), door.at_h(0.0, 0.0, 0.012), Vec3::new(0.32, 0.90, 0.012), 0.01, door.rot());
    m.rbox(&CHROME, door.at_h(-0.26, 0.0, 0.03), Vec3::new(0.012, 0.06, 0.012), 0.008, door.rot());
    placard(m, &door, 0.0, 0.55, 0.12, 0.05, false);
    word(m, &door, 0.0, 0.70, 0.02, "EXIT", Vec3::new(0.28, 0.33, 0.28));
    let ext = Vec3::new(-0.05, FLOOR + 0.40, zb - 0.07);
    m.tube(&RED, &[ext, ext + Vec3::Y * 0.34], &|t| if t > 0.9 { 0.04 } else { 0.055 }, 16, true);
    m.lathe(&BLACK_PANEL, ext + Vec3::Y * 0.34, Vec3::Y, &[[0.03, 0.0], [0.02, 0.05], [0.012, 0.06]], 12, true);
    for t in [0.08f32, 0.24] {
        m.rbox(&ALU, ext + Vec3::new(0.0, t, 0.035), Vec3::new(0.065, 0.012, 0.012), 0.004, Quat::IDENTITY);
    }
    let ox = Vec3::new(0.20, FLOOR + 0.45, zb - 0.07);
    m.tube(&mt(mat::PAINT, 0.12, 0.30, 0.10), &[ox, ox + Vec3::Y * 0.36], &|t| if t > 0.92 { 0.03 } else { 0.05 }, 16, true);
    let axe = Vec3::new(-0.25, FLOOR + 0.95, zb - 0.02);
    m.tube(&mt(mat::PAINT, 0.6, 0.45, 0.05), &[axe, axe + Vec3::new(0.0, 0.38, 0.0)], &|_| 0.014, 10, true);
    m.rbox(&RED, axe + Vec3::new(0.05, 0.36, 0.0), Vec3::new(0.06, 0.035, 0.006), 0.004, Quat::IDENTITY);
    let fb = Face::new(Vec3::new(-0.45, FLOOR + 1.35, zb), Vec3::NEG_X, Vec3::Y);
    placard(m, &fb, 0.0, 0.0, 0.10, 0.07, true);
    // The galley's hot cup and water jug on the bulkhead, right.
    m.rbox(&mt(mat::SATIN, 0.30, 0.31, 0.30), Vec3::new(1.1, FLOOR + 0.95, zb - 0.15), Vec3::new(0.22, 0.25, 0.14), 0.01, Quat::IDENTITY);
    let _ = PI;
}

// ---------------------------------------------------------------- moving controls

fn levers(b: &mut Builder) {
    let yq = q_pivot_y();
    let zq = PED_LEVER_Z;
    let pf = panel_face();
    // The four power levers: a straight shaft and the paddle grip; forward of
    // flight idle the throttle, back past ground idle into reverse.
    for &x in &POWER_LEVER_X {
        let piv = Vec3::new(x, yq, zq);
        let mut m = Mesh::default();
        let grip = piv + Vec3::new(0.0, POWER_LEN, -0.004);
        // (The block in the slot, standing a centimetre out of the arc.)
        m.rbox(&mt(mat::PLASTIC, 0.03, 0.03, 0.032), piv + Vec3::new(0.0, Q_R - 0.017, 0.0), Vec3::new(0.0055, 0.027, 0.020), 0.004, Quat::IDENTITY);
        m.tube(&ALU, &[piv + Vec3::Y * (Q_R - 0.01), piv + Vec3::new(0.0, (Q_R + POWER_LEN) * 0.5, 0.004), grip - Vec3::Y * 0.012], &|_| 0.0055, 8, true);
        m.rbox(&mt(mat::PLASTIC, 0.025, 0.025, 0.027), grip, Vec3::new(0.010, 0.013, 0.024), 0.008, Quat::from_rotation_x(-0.15));
        m.rbox(&mt(mat::PLASTIC, 0.03, 0.03, 0.03).pat(pat::KNURL), grip + Vec3::new(0.0, 0.0, -0.022), Vec3::new(0.0105, 0.010, 0.004), 0.003, Quat::IDENTITY);
        let anim = Anim::hinge2(Drive::new(Input::Throttle, -0.70).offset(0.25), Drive::new(Input::Reverser, 0.30));
        let k = b.add("power lever", G_ENGINE, m, anim, piv, Vec3::X, None);
        b.pieces[k].hotspot = Some(Hotspot { hold: Some("reversers"), ..Hotspot::wheel(grip - piv, 0.016, "throttle_up", "throttle_down") });
    }
    // The four condition levers: curved shafts, a ball knob and the engine's
    // number tag; all four go between GROUND STOP and RUN with the fuel control.
    for (k, &x) in COND_LEVER_X.iter().enumerate() {
        let piv = Vec3::new(x, yq, zq);
        let mut m = Mesh::default();
        let knob = piv + Vec3::new(0.0, COND_LEN, -0.045);
        let pts: Vec<Vec3> = (0..=8).map(|i| { let t = i as f32 / 8.0; piv + Vec3::new(0.0, Q_R - 0.03 + (COND_LEN - Q_R + 0.03) * t, -0.045 * t * t) }).collect();
        m.tube(&CHROME, &pts, &|_| 0.0045, 8, true);
        m.lathe(&mt(mat::PLASTIC, 0.03, 0.03, 0.032), knob, Vec3::Y, &[[0.0, -0.012], [0.010, -0.010], [0.0125, -0.002], [0.011, 0.008], [0.0, 0.012]], 14, true);
        let tag = knob + Vec3::new(0.0, -0.040, 0.010);
        m.rbox(&mt(mat::PLASTIC, 0.85, 0.85, 0.82), tag, Vec3::new(0.0085, 0.010, 0.0012), 0.002, Quat::from_rotation_x(-0.4));
        let tf = Face::new(tag + Quat::from_rotation_x(-0.4) * Vec3::new(0.0, 0.0, 0.0014), Vec3::X, Quat::from_rotation_x(-0.4) * Vec3::Y);
        rect(&mut m, &legend_mat(style::TEXT, sid(["1", "2", "3", "4"][k]), Vec3::new(0.85, 0.85, 0.82)), tf.o, tf.ax, tf.ay, 0.006, 0.0075);
        // (TEXT is white ink: the tag's number in black is printed as its shadow.)
        // Driven by the fuel control alone (`FuelControl(0)`: the one
        // simulated fuel control, as `ignition` sets it), not `Input::Ignition`,
        // which also reads 1.5 while the starter turns: the levers stay at
        // GROUND STOP while the engines are motored and go to RUN when they are.
        let anim = Anim::hinge(Drive::new(Input::FuelControl(0), -0.55).smooth(0.2, 0.8).offset(0.30));
        let i = b.add("condition lever", G_ENGINE, m, anim, piv, Vec3::X, None);
        b.pieces[i].hotspot = Some(Hotspot::click(knob - piv, 0.014, "ignition"));
    }
    // The flap lever on the aft pedestal: forward UP, aft 100%.
    let fz = PED_AFT_Z - 0.12;
    let ft = aft_top(fz);
    let fp = Vec3::new(FLAP_X, ft - 0.09, fz);
    let mut fl = Mesh::default();
    let fk = fp + Vec3::Y * 0.165;
    fl.tube(&CHROME, &[fp + Vec3::Y * 0.08, fk - Vec3::Y * 0.01], &|_| 0.0045, 8, true);
    // The knob: an aerofoil section, the way flap handles are.
    fl.rbox(&mt(mat::PLASTIC, 0.72, 0.72, 0.70), fk, Vec3::new(0.022, 0.009, 0.014), 0.008, Quat::IDENTITY);
    let k = b.add("flap lever", G_ENGINE, fl, Anim::hinge(Drive::new(Input::FlapLever, 0.80).offset(-0.40)), fp, Vec3::X, None);
    b.pieces[k].hotspot = Some(Hotspot::wheel(Vec3::Y * 0.165, 0.018, "flaps_up", "flaps_down"));

    // The landing gear lever: the wheel-shaped clear knob on its arm, up and down its slot.
    let gb = pf.at(GEAR_U, v_at(GEAR_HANDLE.y));
    let n = pf.n();
    let mut g = Mesh::default();
    g.tube(&CHROME, &[gb, gb + n * 0.055], &|_| 0.0055, 8, true);
    let kc = gb + n * 0.055;
    g.lathe(&mt(mat::PLASTIC, 0.70, 0.70, 0.66), kc - Vec3::X * 0.012, Vec3::X, &[[0.0, 0.0], [0.016, 0.0], [0.019, 0.004], [0.019, 0.020], [0.016, 0.024], [0.0, 0.024]], 22, false);
    g.dome(&mt(mat::PLASTIC, 0.45, 0.04, 0.03), kc + n * 0.019, n, 0.005, 0.004, 8, 0.0);
    let k = b.add("gear handle", G_ENGINE, g, Anim::slide(Drive::new(Input::GearHandle, -0.032), pf.ay), gb, Vec3::X, None);
    b.pieces[k].hotspot = Some(Hotspot::click(n * 0.055, 0.020, "gear"));

    // The parking brake: a yellow T-handle under the pilot's panel by the
    // pedestal, pulled out (aft) to set.
    let pb = Vec3::new(-0.30, PANEL_BOT_Y - 0.035, PANEL_Z + 0.02);
    let mut p = Mesh::default();
    p.tube(&CHROME, &[pb - Vec3::Z * 0.02, pb + Vec3::Z * 0.05], &|_| 0.004, 8, true);
    p.rbox(&YELLOW, pb + Vec3::Z * 0.055, Vec3::new(0.035, 0.008, 0.007), 0.005, Quat::IDENTITY);
    let k = b.add("parking brake", G_ENGINE, p, Anim::slide(Drive::new(Input::ParkingBrake, 0.05), Vec3::Z), pb, Vec3::Z, None);
    b.pieces[k].hotspot = Some(Hotspot::click(Vec3::Z * 0.062, 0.020, "parking_brake"));

    // The two landing light switches on the centre panel.
    for k in 0..2 {
        let c = pf.at_h(0.205 + k as f32 * 0.075, 0.115, 0.008);
        let mut t = Mesh::default();
        t.lathe(&CHROME, c, n, &[[0.0055, 0.0], [0.0055, 0.0018], [0.0042, 0.0022], [0.0042, 0.0042], [0.0, 0.0044]], 8, false);
        let tip = c + n * 0.017;
        t.tube(&CHROME, &[c + n * 0.003, tip], &|t| 0.002 + 0.0006 * t, 8, true);
        t.dome(&CHROME, tip, n, 0.0028, 0.002, 8, 0.0);
        let i = b.add("landing light switch", G_ENGINE, t, Anim::hinge(Drive::new(Input::LandingLights, -0.70).offset(0.35)), c, pf.ax, None);
        b.pieces[i].hotspot = Some(Hotspot::click(n * 0.016, 0.010, "landing_lights"));
    }

    // Overhead: the battery switch and the four engine start buttons.
    let of = ovhd_face();
    let hl = (OVHD_AFT_Z - OVHD_FWD_Z) * 0.5;
    let on = of.n();
    let c = of.at_h(BATT_U, -hl + 0.17, 0.0045);
    let mut t = Mesh::default();
    t.lathe(&CHROME, c, on, &[[0.0065, 0.0], [0.0065, 0.002], [0.005, 0.0025], [0.005, 0.0048], [0.0, 0.005]], 8, false);
    let tip = c + on * 0.019;
    t.tube(&CHROME, &[c + on * 0.003, tip], &|t| 0.0022 + 0.0007 * t, 8, true);
    t.dome(&CHROME, tip, on, 0.003, 0.0022, 8, 0.0);
    // (Hinged across the panel: ON with the bat aft, OFF forward.)
    let i = b.add("battery switch", G_ENGINE, t, Anim::hinge(Drive::new(Input::Master, 0.70).offset(-0.35)), c, of.ax, None);
    b.pieces[i].hotspot = Some(Hotspot::click(on * 0.018, 0.011, "master"));
    for &u in &START_U {
        let c = of.at_h(u, -hl + 0.15, 0.0045);
        let mut s = Mesh::default();
        s.lathe(&CASE, c, on, &[[0.011, 0.0], [0.011, 0.004], [0.0, 0.004]], 14, false);
        s.lathe(&mt(mat::PLASTIC, 0.06, 0.06, 0.065), c + on * 0.004, on, &[[0.0075, 0.0], [0.0075, 0.009], [0.0065, 0.010], [0.0, 0.010]], 14, false);
        let i = b.add("engine start button", G_ENGINE, s, Anim::slide(Drive::new(Input::Ignition, -0.004).smooth(1.1, 1.4), on), c, on, None);
        b.pieces[i].hotspot = Some(Hotspot::hold(on * 0.014, 0.011, "starter"));
    }

    // The ramp and aft door switch on the pilot's console, aft of the
    // steering wheel (see the module docs).
    let yt = FLOOR + 0.60;
    let rc = Vec3::new(-1.00, yt + 0.004, PANEL_Z + 0.50);
    let mut r = Mesh::default();
    r.lathe(&CHROME, rc, Vec3::Y, &[[0.007, 0.0], [0.007, 0.002], [0.005, 0.003], [0.005, 0.005], [0.0, 0.0052]], 8, false);
    r.tube(&CHROME, &[rc + Vec3::Y * 0.004, rc + Vec3::Y * 0.022], &|t| 0.0025 + 0.0008 * t, 8, true);
    r.dome(&CHROME, rc + Vec3::Y * 0.022, Vec3::Y, 0.0034, 0.0025, 8, 0.0);
    let i = b.add("ramp switch", G_ENGINE, r, Anim::hinge(Drive::new(Input::Canopy, 0.8).offset(-0.40)), rc, Vec3::X, None);
    b.pieces[i].hotspot = Some(Hotspot::click(Vec3::Y * 0.021, 0.012, "canopy"));
}

fn columns(b: &mut Builder) {
    for sx in [-1.0f32, 1.0] {
        let x = sx * EYE.x.abs();
        // The column out of the floor ahead of the seat, the wheel on its top.
        let base = Vec3::new(x, FLOOR, EYE.z - 0.54);
        let hub = Vec3::new(x, EYE.y - 0.47, EYE.z - 0.43);
        let mut c = Mesh::default();
        c.lathe(&mt(mat::RUBBER, 0.03, 0.03, 0.03), base, Vec3::Y, &[[0.08, 0.0], [0.08, 0.02], [0.05, 0.08], [0.04, 0.12]], 16, false);
        let top = hub + Vec3::new(0.0, -0.02, 0.05);
        c.tube(&COLUMN_MAT, &[base + Vec3::Y * 0.05, base + Vec3::new(0.0, 0.45, 0.02), top], &|t| 0.036 - 0.008 * t, 14, true);
        c.rbox(&COLUMN_MAT, top + Vec3::new(0.0, 0.005, -0.01), Vec3::new(0.028, 0.03, 0.03), 0.012, Quat::IDENTITY);
        let ci = b.add("control column", G_ENGINE, c, Anim::hinge(Drive::new(Input::Pitch, 0.10)), base, Vec3::X, None);
        // The wheel: a U of two horns on a hub, the switch box on the outboard horn.
        let ax = (hub - top).normalize();
        let ax = (ax + Vec3::Z * 2.0).normalize();
        let rot = Quat::from_rotation_arc(Vec3::Z, ax);
        let mut y = Mesh::default();
        y.lathe(&YOKE_MAT, hub - ax * 0.02, ax, &[[0.045, 0.0], [0.045, 0.03], [0.04, 0.034], [0.0, 0.035]], 18, false);
        y.disc(&mt(mat::ALU, 0.45, 0.45, 0.46), hub + ax * 0.0151, ax, rot * Vec3::Y, 0.040, 18);
        for k in 0..4 {
            let a = k as f32 / 4.0 * TAU + 0.6;
            y.screw(&SCREW_STEEL, hub + ax * 0.016 + rot * (Vec3::new(a.cos(), a.sin(), 0.0) * 0.028), ax, 0.003, a);
        }
        let shape: Vec<Vec3> = [(-0.17, 0.10), (-0.175, 0.03), (-0.16, -0.06), (-0.10, -0.10), (0.0, -0.105), (0.10, -0.10), (0.16, -0.06), (0.175, 0.03), (0.17, 0.10)]
            .iter().map(|&(a, c)| hub + rot * Vec3::new(a, c, 0.0)).collect();
        y.tube(&YOKE_MAT, &shape, &|t| 0.017 + 0.003 * (t * PI * 2.0).sin().abs(), 12, true);
        for s in [-1.0f32, 1.0] {
            y.tube(&YOKE_MAT, &[hub + rot * Vec3::new(s * 0.035, -0.01, 0.0), hub + rot * Vec3::new(s * 0.08, -0.095, 0.0)], &|_| 0.014, 10, true);
        }
        let ob = sx; // the outboard horn
        let sb = hub + rot * Vec3::new(ob * 0.165, 0.10, 0.01);
        y.rbox(&YOKE_MAT, sb, Vec3::new(0.022, 0.03, 0.018), 0.008, rot);
        for (k, dy) in [0.0f32, -0.02].iter().enumerate() {
            y.dome(&RED, sb + rot * Vec3::new(-ob * 0.012, *dy, 0.019), rot * Vec3::Z, 0.0055, 0.004, 10, k as f32);
        }
        let yi = b.add("yoke", G_ENGINE, y, Anim::hinge(Drive::new(Input::Roll, -0.9)), hub, ax, Some(ci));
        // The pitch trim switch on top of the outboard horn (momentary).
        let mut t = Mesh::default();
        let tc = hub + rot * Vec3::new(ob * 0.165, 0.132, 0.0);
        t.rbox(&mt(mat::PLASTIC, 0.05, 0.05, 0.05), tc, Vec3::new(0.006, 0.004, 0.009), 0.003, rot);
        let ti = b.add("yoke trim switch", G_ENGINE, t, Anim::Fixed, Vec3::ZERO, Vec3::X, Some(yi));
        b.pieces[ti].hotspot = Some(Hotspot::wheel(tc + rot * Vec3::new(0.0, 0.004, 0.006), 0.010, "trim_down", "trim_up"));

        // Rudder pedals hung from under the panel, the toe brakes on top.
        for dx in [-0.11f32, 0.11] {
            let mut p = Mesh::default();
            let pv = Vec3::new(x + dx, PANEL_BOT_Y - 0.05, PANEL_Z - 0.10);
            let pc = Vec3::new(x + dx, FLOOR + 0.26, PANEL_Z + 0.04);
            p.tube(&STEEL, &[pv, pv + Vec3::new(0.0, -0.10, 0.05), pc + Vec3::new(0.0, 0.10, -0.02)], &|_| 0.010, 8, true);
            let tilt = Quat::from_rotation_x(-0.30);
            p.rbox(&mt(mat::ALU, 0.32, 0.32, 0.33), pc, Vec3::new(0.045, 0.09, 0.008), 0.006, tilt);
            p.rbox(&RUBBER.pat(pat::KNURL), pc + tilt * Vec3::new(0.0, 0.0, 0.009), Vec3::new(0.04, 0.08, 0.0025), 0.002, tilt);
            let anim = Anim::hinge2(Drive::new(Input::Yaw, dx.signum() * 0.16), Drive::new(Input::Brake(dx.signum()), 0.12));
            let piece = b.add("rudder pedal", G_ENGINE, p, anim, pv, Vec3::X, None);
            if sx < 0.0 {
                let toe = pc + tilt * Vec3::new(0.0, 0.065, 0.03);
                b.pieces[piece].hotspot = Some(Hotspot::hold(toe - pv, 0.032, if dx < 0.0 { "brake_left" } else { "brake_right" }));
            }
        }
    }
}

// ---------------------------------------------------------------- what the eye sees

/// Whether a look from `eye` along `d` (body frame) ends on something on the
/// flight deck (true) or goes out through a window (false).
pub fn sees_inside(eye: Vec3, d: Vec3) -> bool {
    let d = d.normalize_or_zero();
    let f = panel_face();
    let n = f.n();
    let mut t = 0.03;
    while t < 6.0 {
        let p = eye + d * t;
        if p.y < FLOOR + 0.01 || p.z > DECK_AFT_Z {
            return true;
        }
        // The panel and the glareshield ahead (as far forward as it reaches).
        if (p - f.o).dot(n) < 0.01 && p.y < GLARE_Y + 0.005 && p.x.abs() < PW + 0.15 && p.z > GLARE_Z - 0.01 {
            return true;
        }
        // The pedestal, the overhead, the seats.
        if p.x.abs() < PED_HALF_W && p.y < PED_TOP_Y && p.z > PANEL_Z && p.z < PED_AFT_Z {
            return true;
        }
        if p.x.abs() < OVHD_HALF_W && p.y > OVHD_LOW_Y && p.z > OVHD_FWD_Z && p.z < OVHD_AFT_Z {
            return true;
        }
        // Reached the lining: a window, or the wall.
        if p.y >= deck_ceiling(p.z) || p.x.abs() >= deck_half_w(p.z, p.y) || p.z < DECK_FWD_Z {
            return window(p) >= 0.0;
        }
        t += 0.012 + t * 0.01;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nothing solid of the deck pokes out of the envelope the exterior is built
    /// round, give or take 2 cm (the lining is 10 cm inside the skin, `dims::LINING`).
    const TOL: f32 = 0.02;
    #[test]
    fn deck_stays_in_its_envelope() {
        let mut b = Builder::default();
        build(&mut b);
        let mut bad = Vec::new();
        for p in &b.pieces {
            if p.glass {
                continue;
            }
            let worst = p.mesh.v.iter().map(|v| Vec3::from(v.pos)).filter(|&q| !in_env(q, TOL)).take(3).collect::<Vec<_>>();
            if !worst.is_empty() {
                bad.push(format!("{}: {:?}", p.name, worst));
            }
        }
        assert!(bad.is_empty(), "outside the flight deck envelope:\n{}", bad.join("\n"));
    }

    /// Straight ahead and over the nose the pilot sees out; the panel below.
    #[test]
    fn the_pilot_sees_out_and_the_panel() {
        assert!(!sees_inside(EYE, Vec3::NEG_Z));
        assert!(!sees_inside(EYE, Vec3::new(0.0, -0.10, -1.0)));
        assert!(sees_inside(EYE, (panel_face().at(-0.62, 0.1) - EYE).normalize()));
        assert!(sees_inside(EYE, (Vec3::new(-0.1, PED_TOP_Y - 0.05, PED_LEVER_Z) - EYE).normalize()));
    }

    /// The dials' markings are the flight model's: the airspeed's red line is
    /// `flight::VNE_KT`, the tachometer's idle the engine's ground idle, and
    /// the elevator tab indicator's span the trim's (plane.wgsl's `C130G_*`
    /// copies; the auto-trim's lower clamp, -0.05 rad, is plane.rs's).
    #[test]
    fn gauges_follow_the_flight_model() {
        use super::super::flight::{SPEC, VNE_KT};
        let src = include_str!("../../shaders/plane.wgsl");
        let has = |t: String| assert!(src.contains(&t), "plane.wgsl lacks {t:?}: keep it in step with c130/flight.rs");
        has(format!("const C130G_VNE_KT = {VNE_KT:?};"));
        assert!((SPEC.vne * 1.94384 - VNE_KT).abs() < 0.01);
        let crate::aircraft::Engine::Fan(f) = &SPEC.engine else { panic!("the C-130 flies as Engine::Fan") };
        has(format!("const C130G_IDLE = {:?};", f.idle));
        // Trimmed for the cruise speed at sea level at the reference mass.
        let q = 0.5 * 1.225 * SPEC.cruise * SPEC.cruise;
        let a0 = (SPEC.mass * 9.81 / (q * SPEC.wing_area) - SPEC.cl0) / SPEC.cl_a;
        has(format!("const C130G_TRIM = vec3f({a0:.4}, -0.05, {:?});", SPEC.alpha_trim));
        assert!(src.contains("redline = C130G_VNE_KT;") && src.contains("let rpm = 100.0 * n1;"));
    }

    /// Every C-130 dial id the deck uses is one the shader draws.
    #[test]
    fn gauges_are_drawn_by_the_shader() {
        let src = include_str!("../../shaders/plane.wgsl");
        for id in gid::ENGINE.iter().chain([gid::RADAR_ALT, gid::HYD_UTILITY, gid::FLAPS, gid::ELEV_TAB, gid::FUEL_TANK, gid::GEAR, gid::HSI, gid::REST].iter()) {
            let s = format!("{}u", *id as u32);
            assert!(src.contains(&format!("case {s}")) || src.contains(&format!(", {s}")), "plane.wgsl draws no gauge {s}");
        }
    }
}

