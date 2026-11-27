//! saved flights: where the aircraft is and how it is set up, the time, the
//! weather and the route, as json files in `<config>/fablegame/flights/`.
//! a flight loads back where it was, in the air or on the ground.

use crate::{aircraft, flightplan, geo, menu, profile, tr_en, Game};
use glam::{DQuat, DVec3};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Saved {
    pub name: String,
    pub unix: u64,
    pub aircraft: String,
    /// planet frame position, velocity (m, m/s), attitude (x, y, z, w) and rotation (rad/s).
    pub pos: [f64; 3],
    pub vel: [f64; 3],
    pub rot: [f64; 4],
    pub omega: [f64; 3],
    pub on_ground: bool,
    /// the ground height under it (m), for a start on the ground.
    pub ground_h: f64,
    pub fuel: f64,
    pub payload: f64,
    pub throttle: f64,
    pub flap_set: u8,
    pub gear_down: bool,
    pub running: bool,
    pub ignition: bool,
    pub master: bool,
    pub mixture: f64,
    pub parking_brake: bool,
    pub landing_lights: bool,
    pub utc: f64,
    pub doy: f64,
    /// the route as tokens the planner reads, the destination last; the cruise level.
    pub route: Vec<String>,
    pub cruise_ft: f64,
    pub clouds: f32,
    pub wind: f64,
    pub wind_from: f64,
    pub qnh: f64,
    pub live_weather: bool,
    pub com: f32,
}

pub fn dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("fablegame").join("flights"))
}

/// a name as a file name: letters, digits, spaces, dashes and underscores.
fn file_name(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' { c } else { '_' }).collect();
    let s = s.trim().to_string();
    if s.is_empty() { "flight".into() } else { s.chars().take(60).collect() }
}

/// every saved flight, the latest first.
pub fn list() -> Vec<Saved> {
    let Some(d) = dir() else { return Vec::new() };
    let mut out: Vec<Saved> = std::fs::read_dir(d).into_iter().flatten().flatten().filter(|e| e.path().extension().is_some_and(|x| x == "json")).filter_map(|e| serde_json::from_str(&std::fs::read_to_string(e.path()).ok()?).ok()).collect();
    out.sort_by(|a, b| b.unix.cmp(&a.unix));
    out
}

pub fn write(s: &Saved) -> Result<(), String> {
    let d = dir().ok_or("no config folder")?;
    std::fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    std::fs::write(d.join(format!("{}.json", file_name(&s.name))), text).map_err(|e| e.to_string())
}

pub fn delete(name: &str) {
    if let Some(d) = dir() {
        let _ = std::fs::remove_file(d.join(format!("{}.json", file_name(name))));
    }
}

/// a route's waypoints as tokens: names where they have one, else lat,lon.
fn route_tokens(p: &flightplan::Plan) -> Vec<String> {
    p.wps.iter().filter(|w| w.phase.is_none()).map(|w| match w.kind {
        flightplan::Kind::User | flightplan::Kind::Leg => format!("{:.5},{:.5}", w.lat, w.lon),
        _ => w.ident.clone(),
    }).collect()
}

impl Game {
    /// the flight as it is now.
    pub(crate) fn snapshot(&self, name: &str) -> Saved {
        let p = &self.plane;
        let (la, lo, _) = geo::pos_to_lla(p.pos);
        Saved {
            name: name.to_string(),
            unix: profile::now_unix(),
            aircraft: self.spec.keys.first().copied().unwrap_or("c172").to_string(),
            pos: p.pos.to_array(),
            vel: p.vel.to_array(),
            rot: p.rot.to_array(),
            omega: p.omega.to_array(),
            on_ground: p.on_ground,
            ground_h: self.world.ground(la, lo).map_or(0.0, |g| g.h),
            fuel: p.fuel,
            payload: p.payload,
            throttle: p.throttle,
            flap_set: p.flap_set,
            gear_down: p.gear_down,
            running: p.running,
            ignition: p.ignition,
            master: p.sys.elec.master,
            mixture: p.mixture,
            parking_brake: p.parking_brake,
            landing_lights: p.landing_lights,
            utc: self.utc,
            doy: self.doy,
            route: route_tokens(&self.plan),
            cruise_ft: self.plan.cruise_ft,
            clouds: self.wx.clouds,
            wind: self.wx.wind,
            wind_from: self.wx.wind_from,
            qnh: self.wx.qnh,
            live_weather: self.wx.live,
            com: self.atc.com,
        }
    }

    /// back into a saved flight.
    pub(crate) fn load_flight(&mut self, s: &Saved) {
        if let Some(spec) = aircraft::by_key(&s.aircraft) {
            self.set_aircraft(spec);
        }
        let pos = DVec3::from(s.pos);
        let (la, lo, alt) = geo::pos_to_lla(pos);
        let rot = DQuat::from_array(s.rot).normalize();
        let f = rot * DVec3::NEG_Z;
        let (e, n, _) = geo::enu(pos);
        let heading = f.dot(e).atan2(f.dot(n));
        if s.on_ground {
            self.respawn_ground(la, lo, s.ground_h, heading, 0.0);
        } else {
            self.respawn_at(la, lo, alt, heading, 0.0);
        }
        let p = &mut self.plane;
        (p.pos, p.vel, p.rot, p.omega) = (pos, DVec3::from(s.vel), rot, DVec3::from(s.omega));
        p.fuel = s.fuel;
        p.payload = s.payload;
        p.throttle = s.throttle;
        p.configure(s.flap_set, s.gear_down, 0.0);
        p.throttle = s.throttle;
        p.ignition = s.ignition;
        p.sys.elec.master = s.master;
        p.mixture = s.mixture;
        p.parking_brake = s.parking_brake;
        p.landing_lights = s.landing_lights;
        if !s.running {
            p.cold_and_dark();
            (p.sys.elec.master, p.parking_brake) = (s.master, s.parking_brake);
        }
        p.pos = pos;
        (self.utc, self.doy) = (s.utc, s.doy);
        if !s.live_weather {
            (self.wx.clouds, self.wx.wind, self.wx.wind_from, self.wx.qnh) = (s.clouds, s.wind, s.wind_from, s.qnh);
        }
        self.plan.clear();
        if !s.route.is_empty() {
            let here = (la.to_degrees(), lo.to_degrees());
            let _ = self.plan.add_route(&s.route.join(" "), Some(here));
            self.plan.cruise_ft = s.cruise_ft;
            self.plan.begin(here);
        }
        self.atc.reset();
        self.atc.com = s.com;
        self.rec.clear();
        self.replay = None;
        self.status = format!("{} {}", tr_en("Loaded"), s.name);
        self.status_until = self.real + 4.0;
    }

    /// the pause menu's flights page: save this one, load or delete the others, replay.
    pub(crate) fn flights_page(&mut self, ui: &mut egui::Ui) -> bool {
        use egui::{Color32, RichText};
        let mut resume = false;
        ui.label(menu::section_text(tr_en("REPLAY")));
        match self.rec.span() {
            Some((a, b)) if b - a > 5.0 && !self.crashed => {
                ui.label(RichText::new(format!("{} {}", tr_en("The last"), profile::hhmm(b - a))).font(menu::caption()).color(menu::DIM));
                if menu::choice(ui, tr_en("Watch the replay"), false, 260.0).clicked() {
                    self.start_replay();
                    resume = true;
                }
            }
            _ => {
                ui.label(RichText::new(tr_en("Nothing recorded yet: the replay keeps the last half hour of flying.")).font(menu::caption()).color(menu::DIM));
            }
        }
        ui.add_space(16.0);
        ui.label(menu::section_text(tr_en("SAVE THIS FLIGHT")));
        let (la, lo, _) = geo::pos_to_lla(self.plane.pos);
        let default = format!("{} {}", self.spec.name, Self::place_code(la.to_degrees(), lo.to_degrees()));
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.save_name).desired_width(320.0).hint_text(default.clone()).font(menu::label()).margin(egui::vec2(10.0, 7.0)));
            if menu::choice(ui, tr_en("Save"), false, 100.0).clicked() {
                let name = if self.save_name.trim().is_empty() { default.clone() } else { self.save_name.trim().to_string() };
                let s = self.snapshot(&name);
                self.status = match write(&s) {
                    Ok(()) => format!("{} {name}", tr_en("Saved")),
                    Err(e) => format!("{}: {e}", tr_en("Could not save")),
                };
                self.status_until = self.real + 4.0;
                self.save_name.clear();
            }
        });
        ui.add_space(16.0);
        ui.label(menu::section_text(tr_en("SAVED FLIGHTS")));
        let all = list();
        if all.is_empty() {
            ui.label(RichText::new(tr_en("No saved flights yet.")).font(menu::caption()).color(menu::DIM));
        }
        let mut load: Option<Saved> = None;
        let mut gone: Option<String> = None;
        for s in &all {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(360.0);
                    ui.label(RichText::new(&s.name).font(menu::label()).color(Color32::WHITE));
                    let (la, lo, alt) = geo::pos_to_lla(DVec3::from(s.pos));
                    let at = Self::place_code(la.to_degrees(), lo.to_degrees());
                    let state = if s.on_ground { tr_en("on the ground").to_string() } else { format!("{:.0} ft", alt * 3.28084) };
                    ui.label(RichText::new(format!("{}  ·  {}  ·  {at}  ·  {state}", profile::date(s.unix), s.aircraft.to_uppercase())).font(menu::caption()).color(menu::DIM));
                });
                if menu::choice(ui, tr_en("Load"), false, 90.0).clicked() {
                    load = Some(s.clone());
                }
                if menu::close_button(ui, 32.0, "").clicked() {
                    gone = Some(s.name.clone());
                }
            });
            ui.add_space(4.0);
        }
        if let Some(n) = gone {
            delete(&n);
        }
        if let Some(s) = load {
            self.load_flight(&s);
            resume = true;
        }
        resume
    }
}
