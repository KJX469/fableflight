//! the normal checklists, one set per family of aircraft, after the order of
//! the makers' handbooks (cessna 172rg poh section 4, the robin dr400 flight
//! manual, boeing's 747-400 normal checklist, the f/a-18 natops pocket
//! checklist) and cut down to what the sim models. items the sim can see tick
//! themselves; the rest are ticked by hand. for the game, not for real flying.

use crate::aircraft::Spec;
use crate::plane::{FuelSelector, Plane};
use crate::{flightui, menu, tr_en, Game};
use egui::{pos2, vec2, Color32, Rect, Sense, Stroke};

/// what the sim can tell of an item.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Is {
    /// ticked by hand.
    Hand,
    ParkingBrake(bool),
    Master(bool),
    Ignition(bool),
    Running(bool),
    MixtureRich,
    MixtureCutoff,
    FuelOn,
    ThrottleIdle,
    ThrottleFull,
    FlapsUp,
    /// a takeoff setting: past up, short of full.
    FlapsTakeoff,
    FlapsFull,
    GearDown,
    GearUp,
    Lights(bool),
    CanopyOpen(bool),
    SpeedbrakeIn,
    Autopilot(bool),
    PropHigh,
    CowlOpen,
    Airborne,
    Stopped,
    ReversersStowed,
    CarbHeat(bool),
    PitotHeat(bool),
}

/// what an item is checked against.
pub struct Ctx<'a> {
    pub plane: &'a Plane,
    pub ap: bool,
}

impl Is {
    /// whether the sim sees the item done; none for one ticked by hand.
    pub fn met(self, c: &Ctx) -> Option<bool> {
        let p = c.plane;
        let max = p.spec.max_notch();
        Some(match self {
            Is::Hand => return None,
            Is::ParkingBrake(on) => p.parking_brake == on,
            Is::Master(on) => p.sys.elec.master == on,
            Is::Ignition(on) => p.ignition == on,
            Is::Running(on) => p.running == on,
            Is::MixtureRich => p.mixture > 0.9,
            Is::MixtureCutoff => p.mixture < 0.05,
            Is::FuelOn => p.fuel_selector != FuelSelector::Off,
            Is::ThrottleIdle => p.throttle < 0.08,
            Is::ThrottleFull => p.throttle > 0.95,
            Is::FlapsUp => p.flap_set == 0,
            Is::FlapsTakeoff => p.flap_set > 0 && (p.flap_set < max || max == 1),
            Is::FlapsFull => p.flap_set == max,
            Is::GearDown => p.gear_down || p.spec.fixed_gear(),
            Is::GearUp => !p.gear_down || p.spec.fixed_gear(),
            Is::Lights(on) => p.landing_lights == on,
            Is::CanopyOpen(open) => p.canopy_open == open,
            Is::SpeedbrakeIn => p.speedbrake < 0.05,
            Is::Autopilot(on) => c.ap == on,
            Is::PropHigh => p.propeller > 0.95,
            Is::CowlOpen => p.cowl_flaps > 0.9,
            Is::Airborne => !p.on_ground,
            Is::Stopped => p.on_ground && p.vel.length() < 0.5,
            Is::ReversersStowed => p.reverser < 0.05,
            Is::CarbHeat(on) => p.carb_heat == on,
            Is::PitotHeat(on) => p.pitot_heat == on,
        })
    }
}

/// one line: the item, what it should be, and how the sim tells.
pub struct Item {
    pub what: &'static str,
    pub state: &'static str,
    pub is: Is,
}

pub struct List {
    pub name: &'static str,
    pub items: &'static [Item],
}

const fn i(what: &'static str, state: &'static str, is: Is) -> Item {
    Item { what, state, is }
}

/// the 172rg cutlass (poh section 4).
static PISTON_RG: &[List] = &[
    List { name: "Before starting engine", items: &[
        i("Preflight inspection", "COMPLETE", Is::Hand),
        i("Seats and belts", "ADJUST, LOCK", Is::Hand),
        i("Brakes", "TEST, SET", Is::ParkingBrake(true)),
        i("Landing gear lever", "DOWN", Is::GearDown),
        i("Fuel selector", "BOTH", Is::FuelOn),
        i("Avionics", "OFF", Is::Hand),
    ] },
    List { name: "Starting engine", items: &[
        i("Mixture", "RICH", Is::MixtureRich),
        i("Propeller", "HIGH RPM", Is::PropHigh),
        i("Throttle", "OPEN 1/4 INCH", Is::Hand),
        i("Master switch", "ON", Is::Master(true)),
        i("Propeller area", "CLEAR", Is::Hand),
        i("Ignition", "START, then BOTH", Is::Running(true)),
        i("Oil pressure", "CHECK", Is::Hand),
    ] },
    List { name: "Before takeoff", items: &[
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Flight controls", "FREE AND CORRECT", Is::Hand),
        i("Flight instruments", "SET", Is::Hand),
        i("Fuel selector", "BOTH", Is::FuelOn),
        i("Mixture", "RICH", Is::MixtureRich),
        i("Elevator trim", "TAKEOFF", Is::Hand),
        i("Run up, magnetos", "CHECK at 1700 RPM", Is::Hand),
        i("Propeller", "HIGH RPM", Is::PropHigh),
        i("Cowl flaps", "OPEN", Is::CowlOpen),
        i("Wing flaps", "UP to 10°", Is::Hand),
        i("Pitot heat", "ON in visible moisture", Is::Hand),
        i("Landing light", "ON", Is::Lights(true)),
    ] },
    List { name: "Takeoff", items: &[
        i("Parking brake", "RELEASE", Is::ParkingBrake(false)),
        i("Power", "FULL THROTTLE, 2700 RPM", Is::ThrottleFull),
        i("Rotate", "55 KIAS", Is::Airborne),
        i("Landing gear", "RETRACT in climb", Is::GearUp),
    ] },
    List { name: "Climb", items: &[
        i("Airspeed", "75 to 85 KIAS", Is::Hand),
        i("Wing flaps", "UP", Is::FlapsUp),
        i("Power", "25 IN, 2500 RPM", Is::Hand),
        i("Cowl flaps", "OPEN as required", Is::Hand),
    ] },
    List { name: "Cruise", items: &[
        i("Power", "15 to 25 IN, 2100 to 2600 RPM", Is::Hand),
        i("Cowl flaps", "CLOSED", Is::Hand),
        i("Mixture", "LEAN", Is::Hand),
    ] },
    List { name: "Descent", items: &[
        i("Altimeter", "SET", Is::Hand),
        i("Mixture", "ENRICHEN as needed", Is::Hand),
        i("Cowl flaps", "CLOSED", Is::Hand),
    ] },
    List { name: "Before landing", items: &[
        i("Seats and belts", "SECURE", Is::Hand),
        i("Fuel selector", "BOTH", Is::FuelOn),
        i("Landing gear", "DOWN, green light", Is::GearDown),
        i("Mixture", "RICH", Is::MixtureRich),
        i("Propeller", "HIGH RPM", Is::PropHigh),
        i("Autopilot", "OFF", Is::Autopilot(false)),
        i("Landing light", "ON", Is::Lights(true)),
        i("Wing flaps", "AS DESIRED", Is::Hand),
    ] },
    List { name: "After landing", items: &[
        i("Wing flaps", "UP", Is::FlapsUp),
        i("Cowl flaps", "OPEN", Is::CowlOpen),
    ] },
    List { name: "Securing airplane", items: &[
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Throttle", "IDLE", Is::ThrottleIdle),
        i("Mixture", "IDLE CUT OFF", Is::MixtureCutoff),
        i("Ignition", "OFF", Is::Ignition(false)),
        i("Master switch", "OFF", Is::Master(false)),
    ] },
];

/// the robin dr400 (flight manual, normal procedures).
static PISTON_FIXED: &[List] = &[
    List { name: "Before starting engine", items: &[
        i("Preflight inspection", "COMPLETE", Is::Hand),
        i("Harnesses", "FASTENED", Is::Hand),
        i("Parking brake", "ON", Is::ParkingBrake(true)),
        i("Fuel selector", "MAIN TANK", Is::FuelOn),
        i("Mixture", "RICH", Is::MixtureRich),
    ] },
    List { name: "Starting engine", items: &[
        i("Throttle", "OPEN 1 CM", Is::Hand),
        i("Master switch", "ON", Is::Master(true)),
        i("Propeller area", "CLEAR", Is::Hand),
        i("Starter", "ENGAGE", Is::Running(true)),
        i("Oil pressure", "RISING", Is::Hand),
    ] },
    List { name: "Before takeoff", items: &[
        i("Run up, magnetos", "CHECK at 2000 RPM", Is::Hand),
        i("Carburettor heat", "CHECK, then COLD", Is::CarbHeat(false)),
        i("Flaps", "TAKEOFF", Is::FlapsTakeoff),
        i("Trim", "TAKEOFF", Is::Hand),
        i("Fuel pump", "ON", Is::Hand),
        i("Canopy", "CLOSED AND LOCKED", Is::CanopyOpen(false)),
        i("Landing light", "ON", Is::Lights(true)),
    ] },
    List { name: "Takeoff", items: &[
        i("Parking brake", "OFF", Is::ParkingBrake(false)),
        i("Throttle", "FULL", Is::ThrottleFull),
        i("Rotate", "55 KT", Is::Airborne),
        i("Flaps", "UP above 300 FT", Is::FlapsUp),
    ] },
    List { name: "Before landing", items: &[
        i("Fuel selector", "FULLEST TANK", Is::FuelOn),
        i("Fuel pump", "ON", Is::Hand),
        i("Mixture", "RICH", Is::MixtureRich),
        i("Carburettor heat", "HOT", Is::CarbHeat(true)),
        i("Flaps", "LANDING", Is::FlapsFull),
    ] },
    List { name: "After landing", items: &[
        i("Flaps", "UP", Is::FlapsUp),
        i("Carburettor heat", "COLD", Is::CarbHeat(false)),
        i("Fuel pump", "OFF", Is::Hand),
    ] },
    List { name: "Engine shutdown", items: &[
        i("Parking brake", "ON", Is::ParkingBrake(true)),
        i("Throttle", "IDLE", Is::ThrottleIdle),
        i("Mixture", "IDLE CUT OFF", Is::MixtureCutoff),
        i("Magnetos", "OFF", Is::Ignition(false)),
        i("Master switch", "OFF", Is::Master(false)),
    ] },
];

/// airliners and business jets (boeing's normal checklist order).
static JET: &[List] = &[
    List { name: "Before start", items: &[
        i("Flight deck preparation", "COMPLETE", Is::Hand),
        i("Battery", "ON", Is::Master(true)),
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Gear lever", "DOWN", Is::GearDown),
        i("Fuel control switches", "CUTOFF", Is::Running(false)),
        i("Takeoff speeds", "SET", Is::Hand),
    ] },
    List { name: "Engine start", items: &[
        i("Engine area", "CLEAR", Is::Hand),
        i("Starters", "START", Is::Running(true)),
        i("Engine indications", "STABLE AT IDLE", Is::Hand),
    ] },
    List { name: "Before taxi", items: &[
        i("Flaps", "TAKEOFF", Is::FlapsTakeoff),
        i("Flight controls", "CHECKED", Is::Hand),
        i("Autobrakes", "RTO", Is::Hand),
        i("Transponder", "TA/RA", Is::Hand),
    ] },
    List { name: "Before takeoff", items: &[
        i("Probe heat", "ON", Is::PitotHeat(true)),
        i("Anti ice", "AS REQUIRED", Is::Hand),
        i("Speedbrake", "DOWN", Is::SpeedbrakeIn),
        i("Landing lights", "ON", Is::Lights(true)),
        i("Parking brake", "RELEASED", Is::ParkingBrake(false)),
        i("Takeoff clearance", "RECEIVED", Is::Hand),
    ] },
    List { name: "After takeoff", items: &[
        i("Landing gear", "UP", Is::GearUp),
        i("Flaps", "UP", Is::FlapsUp),
        i("Autopilot", "ENGAGE", Is::Autopilot(true)),
    ] },
    List { name: "Descent", items: &[
        i("Arrival and approach", "BRIEFED", Is::Hand),
        i("Landing speeds", "SET", Is::Hand),
        i("Altimeters", "SET", Is::Hand),
    ] },
    List { name: "Landing", items: &[
        i("Landing gear", "DOWN", Is::GearDown),
        i("Flaps", "LANDING", Is::FlapsFull),
        i("Speedbrake", "ARMED", Is::Hand),
        i("Autobrakes", "SET", Is::Hand),
    ] },
    List { name: "After landing", items: &[
        i("Speedbrake", "DOWN", Is::SpeedbrakeIn),
        i("Reversers", "STOWED", Is::ReversersStowed),
        i("Flaps", "UP", Is::FlapsUp),
        i("Landing lights", "OFF", Is::Lights(false)),
    ] },
    List { name: "Shutdown", items: &[
        i("Aircraft", "STOPPED at the stand", Is::Stopped),
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Fuel control switches", "CUTOFF", Is::Running(false)),
        i("Battery", "OFF", Is::Master(false)),
    ] },
];

/// concorde: the flap lever is its nose and visor (up, 5°, 12.5°).
static SST: &[List] = &[
    List { name: "Before start", items: &[
        i("Flight deck preparation", "COMPLETE", Is::Hand),
        i("Battery", "ON", Is::Master(true)),
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Fuel control switches", "CUTOFF", Is::Running(false)),
    ] },
    List { name: "Engine start", items: &[
        i("Engines 3, 2, 4, 1", "START", Is::Running(true)),
        i("Reheat", "SELECTED for takeoff", Is::Hand),
    ] },
    List { name: "Before takeoff", items: &[
        i("Visor and nose", "DOWN, 5°", Is::FlapsTakeoff),
        i("Landing lights", "ON", Is::Lights(true)),
        i("Parking brake", "RELEASED", Is::ParkingBrake(false)),
    ] },
    List { name: "Takeoff", items: &[
        i("Throttles", "FULL, reheat lit", Is::ThrottleFull),
        i("Rotate", "195 KT", Is::Airborne),
        i("Landing gear", "UP", Is::GearUp),
    ] },
    List { name: "Climb", items: &[
        i("Visor and nose", "UP", Is::FlapsUp),
        i("Reheat", "OFF until transonic", Is::Hand),
        i("Autopilot", "ENGAGE", Is::Autopilot(true)),
    ] },
    List { name: "Landing", items: &[
        i("Visor and nose", "DOWN, 12.5°", Is::FlapsFull),
        i("Landing gear", "DOWN", Is::GearDown),
        i("Autopilot", "OFF by 500 FT", Is::Hand),
    ] },
    List { name: "After landing", items: &[
        i("Reversers", "STOWED", Is::ReversersStowed),
        i("Visor and nose", "5°", Is::FlapsTakeoff),
        i("Landing lights", "OFF", Is::Lights(false)),
    ] },
    List { name: "Shutdown", items: &[
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Fuel control switches", "CUTOFF", Is::Running(false)),
        i("Battery", "OFF", Is::Master(false)),
    ] },
];

/// fighters (the hornet's pocket checklist, kept to what they share).
static FIGHTER: &[List] = &[
    List { name: "Before start", items: &[
        i("Ejection seat", "ARMED", Is::Hand),
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Battery", "ON", Is::Master(true)),
        i("Gear handle", "DOWN", Is::GearDown),
    ] },
    List { name: "Engine start", items: &[
        i("Intakes", "CLEAR", Is::Hand),
        i("Engines", "CRANK, then IDLE", Is::Running(true)),
        i("Warnings", "CHECKED", Is::Hand),
    ] },
    List { name: "Before taxi", items: &[
        i("Flaps", "HALF", Is::FlapsTakeoff),
        i("Trim", "TAKEOFF", Is::Hand),
        i("Canopy", "CLOSED", Is::CanopyOpen(false)),
        i("Speedbrake", "IN", Is::SpeedbrakeIn),
    ] },
    List { name: "Takeoff", items: &[
        i("Parking brake", "OFF", Is::ParkingBrake(false)),
        i("Throttles", "MIL or MAX", Is::ThrottleFull),
        i("Rotate", "ON SPEED", Is::Airborne),
        i("Landing gear", "UP", Is::GearUp),
        i("Flaps", "AUTO", Is::FlapsUp),
    ] },
    List { name: "Landing", items: &[
        i("Landing gear", "DOWN", Is::GearDown),
        i("Flaps", "FULL", Is::FlapsFull),
        i("Speedbrake", "IN", Is::SpeedbrakeIn),
        i("Approach speed", "ON SPEED AOA", Is::Hand),
    ] },
    List { name: "After landing", items: &[
        i("Flaps", "AUTO", Is::FlapsUp),
        i("Canopy", "OPEN", Is::CanopyOpen(true)),
    ] },
    List { name: "Shutdown", items: &[
        i("Parking brake", "SET", Is::ParkingBrake(true)),
        i("Throttles", "OFF", Is::Running(false)),
        i("Battery", "OFF", Is::Master(false)),
    ] },
];

/// a glider (the ask 21's manual and the cb sift cbe drill before a launch).
static GLIDER: &[List] = &[
    List { name: "Before launch", items: &[
        i("Controls", "FREE AND CORRECT", Is::Hand),
        i("Ballast", "CHECKED", Is::Hand),
        i("Straps", "TIGHT", Is::Hand),
        i("Instruments", "SET", Is::Hand),
        i("Trim", "SET for the launch", Is::Hand),
        i("Canopy", "CLOSED AND LOCKED", Is::CanopyOpen(false)),
        i("Airbrakes", "CLOSED AND LOCKED", Is::SpeedbrakeIn),
        i("Eventualities", "BRIEFED: a cable break", Is::Hand),
    ] },
    List { name: "Launch", items: &[
        i("Wheel brake", "OFF", Is::ParkingBrake(false)),
        i("Airborne", "HOLD the attitude", Is::Airborne),
        i("Release", "AT THE TOP", Is::Hand),
    ] },
    List { name: "Before landing", items: &[
        i("Approach speed", "55 KT plus half the wind", Is::Hand),
        i("Airbrakes", "UNLOCKED, as needed", Is::Hand),
        i("Field", "CHOSEN, circuit planned", Is::Hand),
    ] },
    List { name: "After landing", items: &[
        i("Stopped", "WINGS LEVEL as it slows", Is::Stopped),
        i("Canopy", "OPEN", Is::CanopyOpen(true)),
    ] },
];

/// a piston helicopter (the r44's pilot's operating handbook, section 4, cut down).
static HELI: &[List] = &[
    List { name: "Before starting engine", items: &[
        i("Seat belts", "FASTENED", Is::Hand),
        i("Collective", "FULL DOWN, friction on", Is::ThrottleIdle),
        i("Cyclic and pedals", "NEUTRAL", Is::Hand),
        i("Battery", "ON", Is::Master(true)),
    ] },
    List { name: "Starting engine", items: &[
        i("Rotor area", "CLEAR", Is::Hand),
        i("Key", "START, release when it fires", Is::Running(true)),
        i("Clutch", "ENGAGE, rotor turning", Is::Hand),
        i("Governor", "ON, rotor 101 to 102 %", Is::Hand),
    ] },
    List { name: "Before takeoff", items: &[
        i("Doors", "LATCHED", Is::Hand),
        i("Warning lights", "OUT", Is::Hand),
        i("Rotor speed", "102 %", Is::Hand),
    ] },
    List { name: "Takeoff", items: &[
        i("Collective", "RAISE SLOWLY to a hover", Is::Airborne),
        i("Hover check", "WEIGHT, BALANCE, POWER", Is::Hand),
        i("Transition", "NOSE DOWN, accelerate through 15 KT", Is::Hand),
    ] },
    List { name: "Shutdown", items: &[
        i("Collective", "FULL DOWN", Is::ThrottleIdle),
        i("Key", "OFF", Is::Ignition(false)),
        i("Battery", "OFF", Is::Master(false)),
    ] },
];

/// the checklists for a type: by its own kind, or the body it borrows.
pub fn lists(spec: &Spec) -> &'static [List] {
    let body = spec.origin.map(|o| o.body).unwrap_or(spec.keys.first().copied().unwrap_or(""));
    match spec.id {
        _ if spec.rocket().is_some() => &[],
        _ if spec.rotor.is_some() => HELI,
        _ if spec.fuel_kg <= 0.0 => GLIDER,
        0 => PISTON_RG,
        8 => PISTON_FIXED,
        1 | 9 => JET,
        6 => SST,
        2 | 3 | 4 | 7 => FIGHTER,
        _ if body.contains("747") || body.contains("650") => JET,
        _ if spec.is_jet() => FIGHTER,
        _ if spec.fixed_gear() => PISTON_FIXED,
        _ => PISTON_RG,
    }
}

/// the ticks made by hand, and the list showing.
#[derive(Default)]
pub struct Checklists {
    pub list: usize,
    ticks: Vec<Vec<bool>>,
    /// the lists the ticks belong to (their address).
    of: usize,
}

impl Checklists {
    /// starts over for another type, or a new flight.
    pub fn reset(&mut self, lists: &'static [List]) {
        self.list = 0;
        self.ticks = lists.iter().map(|l| vec![false; l.items.len()]).collect();
        self.of = lists.as_ptr() as usize;
    }

    fn sync(&mut self, lists: &'static [List]) {
        if self.of != lists.as_ptr() as usize || self.ticks.len() != lists.len() {
            self.reset(lists);
        }
        self.list = self.list.min(lists.len().saturating_sub(1));
    }

    fn ticked(&self, l: usize, k: usize) -> bool {
        self.ticks.get(l).and_then(|t| t.get(k)).copied().unwrap_or(false)
    }

    fn toggle(&mut self, l: usize, k: usize) {
        if let Some(t) = self.ticks.get_mut(l).and_then(|t| t.get_mut(k)) {
            *t = !*t;
        }
    }
}

impl Game {
    fn check_ctx(&self) -> Ctx<'_> {
        Ctx { plane: &self.plane, ap: self.ap.on }
    }

    /// the checklist page of the flight panel: the lists down the left, the
    /// one open on the right with the next item to do lit.
    pub(crate) fn checklist_page(&mut self, ui: &mut egui::Ui) {
        use flightui::{key, note, LIT};
        let lists = lists(self.spec);
        self.checks.sync(lists);
        if lists.is_empty() {
            note(ui, tr_en("This aircraft flies without a checklist here."));
            return;
        }
        let ctx = self.check_ctx();
        // what is done in every list, for the sidebar's counts.
        let state: Vec<Vec<(bool, bool)>> = lists.iter().enumerate().map(|(l, list)| {
            list.items.iter().enumerate().map(|(k, it)| {
                let met = it.is.met(&ctx);
                (met == Some(true) || self.checks.ticked(l, k), met.is_some())
            }).collect()
        }).collect();
        let l = self.checks.list;
        let list = &lists[l];
        let done = &state[l];
        let n_done = done.iter().filter(|d| d.0).count();
        let next = done.iter().position(|d| !d.0);
        let mut go: Option<usize> = None;
        let mut flip: Option<usize> = None;
        let mut reset = false;
        let side_w = 210.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            ui.allocate_ui_with_layout(vec2(side_w, 10.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                for (j, li) in lists.iter().enumerate() {
                    let n = state[j].iter().filter(|d| d.0).count();
                    if list_row(ui, tr_en(li.name), n, li.items.len(), j == l, side_w).clicked() {
                        go = Some(j);
                    }
                }
            });
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                // the list's name, how far along, and a bar.
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tr_en(list.name)).font(menu::semibold(20.0)).color(Color32::WHITE));
                    menu::at_right(ui, "check_n", |ui| {
                        let col = if n_done == list.items.len() { LIT } else { DIM };
                        ui.label(egui::RichText::new(format!("{n_done} / {}", list.items.len())).font(menu::label()).color(col));
                    });
                });
                ui.add_space(4.0);
                flightui::meter(ui, n_done as f32 / list.items.len().max(1) as f32, if n_done == list.items.len() { GOOD } else { LIT });
                ui.add_space(8.0);
                for (k, it) in list.items.iter().enumerate() {
                    let r = item_row(ui, tr_en(it.what), tr_en(it.state), done[k].0, Some(k) == next);
                    let r = if done[k].1 { r.on_hover_text(tr_en("Ticks itself from the cockpit")) } else { r };
                    if r.clicked() {
                        flip = Some(k);
                    }
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if flightui::text_link(ui, tr_en("Reset")).clicked() {
                        reset = true;
                    }
                    if l + 1 < lists.len() {
                        menu::at_right(ui, "check_next", |ui| {
                            let all = n_done == list.items.len();
                            let word = format!("{}: {}", tr_en("Next"), tr_en(lists[l + 1].name));
                            if key(ui, &word, all, 210.0, 30.0).clicked() {
                                go = Some(l + 1);
                            }
                        });
                    }
                });
            });
        });
        if let Some(k) = flip {
            self.checks.toggle(l, k);
        }
        if reset {
            self.checks.reset(lists);
        }
        if let Some(j) = go {
            self.checks.list = j;
        }
    }
}

/// the green of a done item.
const GOOD: Color32 = Color32::from_rgb(110, 210, 130);
const DIM: Color32 = menu::DIM;

/// a list in the sidebar: its name and how many of its items are done, a tick when all are.
fn list_row(ui: &mut egui::Ui, name: &str, done: usize, of: usize, on: bool, w: f32) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(w, 32.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, name));
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let p = ui.painter();
    if on {
        p.rect_filled(r, 0.0, flightui::LIT.gamma_multiply(0.14));
        p.rect_stroke(r, 0.0, Stroke::new(1.0, flightui::LIT.gamma_multiply(0.5)), egui::StrokeKind::Inside);
    } else if lit > 0.0 {
        p.rect_filled(r, 0.0, Color32::from_white_alpha((8.0 * lit) as u8));
    }
    let all = done == of && of > 0;
    let col = if on { Color32::WHITE } else if all { DIM } else { menu::TEXT };
    let right = if all { String::new() } else { format!("{done}/{of}") };
    let rg = p.layout_no_wrap(right, menu::caption(), DIM);
    let ng = menu::one_line(p, name, menu::label(), col, w - 50.0);
    p.galley(pos2(r.left() + 14.0, r.center().y - ng.size().y * 0.5), ng, col);
    if all {
        tick(p, pos2(r.right() - 16.0, r.center().y), GOOD);
    } else {
        p.galley(pos2(r.right() - 10.0 - rg.size().x, r.center().y - rg.size().y * 0.5), rg, DIM);
    }
    resp
}

/// a tick mark centred at `c`.
fn tick(p: &egui::Painter, c: egui::Pos2, col: Color32) {
    p.line(vec![c + vec2(-5.0, 0.0), c + vec2(-1.5, 3.5), c + vec2(5.0, -4.0)], Stroke::new(2.0, col));
}

/// one item: a tick box, the item, a dotted leader and what it should be.
/// the next item to do is lit; done ones step back.
fn item_row(ui: &mut egui::Ui, what: &str, state: &str, done: bool, next: bool) -> egui::Response {
    let w = ui.available_width();
    let (r, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, done, what));
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let p = ui.painter();
    if next {
        p.rect_filled(r, 0.0, flightui::LIT.gamma_multiply(0.10));
    }
    if lit > 0.0 {
        p.rect_filled(r, 0.0, Color32::from_white_alpha((10.0 * lit) as u8));
    }
    let bx = Rect::from_center_size(pos2(r.left() + 22.0, r.center().y), vec2(18.0, 18.0));
    if done {
        p.rect_filled(bx, 3.0, GOOD.gamma_multiply(0.22));
        p.rect_stroke(bx, 3.0, Stroke::new(1.4, GOOD), egui::StrokeKind::Inside);
        tick(p, bx.center(), GOOD);
    } else {
        p.rect_stroke(bx, 3.0, Stroke::new(1.3, Color32::from_white_alpha(if next { 200 } else { 110 })), egui::StrokeKind::Inside);
    }
    let col = if done { DIM } else { Color32::WHITE };
    let scol = if done { GOOD.gamma_multiply(0.8) } else if next { flightui::LIT } else { menu::TEXT };
    let right = p.layout_no_wrap(state.to_uppercase(), menu::semibold(14.0), scol);
    let room = (r.width() - 44.0 - right.size().x - 30.0).max(40.0);
    let left = menu::one_line(p, what, menu::label(), col, room);
    let lw = left.size().x;
    p.galley(pos2(r.left() + 42.0, r.center().y - left.size().y * 0.5), left, col);
    // the dotted leader between the item and its state.
    let x1 = r.right() - 12.0 - right.size().x - 10.0;
    let mut x = r.left() + 42.0 + lw + 10.0;
    while x < x1 {
        p.circle_filled(pos2(x, r.center().y + 5.0), 0.9, Color32::from_white_alpha(if done { 30 } else { 60 }));
        x += 6.0;
    }
    let rw = right.size();
    p.galley(pos2(r.right() - 12.0 - rw.x, r.center().y - rw.y * 0.5), right, scol);
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_type_has_lists_but_the_rocket() {
        for s in crate::aircraft::BUILTIN {
            assert_eq!(lists(s).is_empty(), s.rocket().is_some(), "{}", s.name);
            for l in lists(s) {
                assert!(!l.items.is_empty(), "{}: {}", s.name, l.name);
            }
        }
    }
}
