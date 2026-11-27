//! ground services for the aircraft being flown: a pushback tug at the nose
//! wheel (it pushes, the pilot picks which way the tail goes), a jet bridge
//! that drives up to the front door at a terminal stand (stairs elsewhere),
//! and fuel, baggage and boarding round it while it stands. the vehicles are
//! the airport traffic's own models (airmodels.rs).

use crate::airmodels as am;
use crate::render::Inst;
use crate::{flightui, geo, tr_en, Game};
use glam::{DVec3, Vec3};

/// a pushback: how far it has gone, which way the tail swings.
#[derive(Clone, Copy, Debug)]
pub struct Push {
    /// -1 tail left .. 1 tail right.
    pub steer: f64,
    /// metres pushed so far, and stopped by the crew.
    pub gone: f64,
    pub stopped: bool,
    /// where it started.
    pub from: DVec3,
}

#[derive(Default)]
pub struct Ground {
    pub push: Option<Push>,
    /// the jet bridge: wanted at the door, how far it has driven (0 parked, 1 at the door).
    pub bridge_on: bool,
    pub bridge: f32,
    /// the gate it swings from (planet frame), found when asked for.
    pub gate: Option<DVec3>,
    /// fuelling up to this much (kg), boarding up to this payload (kg).
    pub fuel_to: Option<f64>,
    pub board_to: Option<f64>,
    /// stairs at the door, baggage at the hold.
    pub stairs: bool,
    pub baggage: bool,
    /// a glider's winch launch: how long it has been pulling (s).
    pub winch: Option<f64>,
}

/// how fast a tug pushes (m/s), and how far before it stops by itself (m).
const PUSH_V: f64 = 1.4;
const PUSH_MAX: f64 = 90.0;

impl Ground {
    pub fn busy(&self) -> bool {
        self.push.is_some() || self.bridge > 0.0 || self.fuel_to.is_some() || self.board_to.is_some() || self.stairs || self.baggage || self.winch.is_some()
    }
}

impl Game {
    /// the front left door, in the body frame: behind the flight deck, at the sill.
    fn door_body(&self) -> Vec3 {
        let s = self.spec;
        let (hw, hh, cy) = (s.hole.fus[0], s.hole.fus[1], s.hole.fus[2]);
        let nose = s.hard[0].z as f32;
        let tail = s.hard[3].z as f32;
        Vec3::new(-hw, cy - hh * 0.35, nose + (tail - nose) * 0.12)
    }

    /// a body point put down on the ground under it (planet frame).
    fn on_ground_at(&self, b: Vec3) -> DVec3 {
        let w = self.plane.pos + self.plane.rot * b.as_dvec3();
        let (la, lo, h) = geo::pos_to_lla(w);
        let gh = self.world.ground(la, lo).map_or(h - self.plane.wheel_height(), |g| g.h);
        geo::lla_to_pos(la, lo, gh)
    }

    /// once a frame: the tug, the bridge, the fuel and the boarding move on.
    pub(crate) fn ground_step(&mut self, dt: f64) {
        let p = &mut self.plane;
        // the winch: pulls the glider ahead and up the cable for a minute, then lets go at the top.
        if let Some(t) = self.ground.winch.as_mut() {
            *t += dt;
            let (la, lo, h) = geo::pos_to_lla(p.pos);
            let agl = self.world.ground(la, lo).map_or(0.0, |g| h - g.h);
            if *t > 70.0 || agl > 450.0 || p.parking_brake {
                self.ground.winch = None;
                p.tow = None;
                self.status = tr_en("Cable released: find a thermal.").into();
                self.status_until = self.real + 5.0;
            } else {
                // (a cable pulls along the glider's axis, harder as it builds speed)
                p.tow = Some(-30.0);
            }
            return;
        }
        if let Some(push) = self.ground.push.as_mut() {
            push.gone = (p.pos - push.from).length();
            // the brakes set, or far enough: the tug lets go.
            if p.parking_brake || push.gone > PUSH_MAX || !p.on_ground {
                push.stopped = true;
            }
            p.tow = if push.stopped { None } else { Some(PUSH_V) };
            if push.stopped && p.vel.length() < 0.2 {
                self.ground.push = None;
                p.tow = None;
                self.status = tr_en("Pushback complete: set the parking brake, the tug disconnects.").into();
                self.status_until = self.real + 5.0;
            }
        } else {
            p.tow = None;
        }
        // the bridge drives over about twenty seconds.
        let want = if self.ground.bridge_on { 1.0 } else { 0.0 };
        let step = (dt / 20.0) as f32;
        self.ground.bridge = if self.ground.bridge < want { (self.ground.bridge + step).min(want) } else { (self.ground.bridge - step).max(want) };
        if self.ground.bridge <= 0.0 && !self.ground.bridge_on {
            self.ground.gate = None;
        }
        // a moving aircraft leaves the stand: everything stops.
        if p.vel.length() > 2.0 && self.ground.push.is_none() {
            self.ground.bridge_on = false;
            self.ground.fuel_to = None;
            self.ground.board_to = None;
            self.ground.stairs = false;
            self.ground.baggage = false;
        }
        // a fuel truck pumps about 600 l a minute into an airliner, a bowser 60 into a light aircraft.
        if let Some(to) = self.ground.fuel_to {
            let rate = if self.spec.mass > 20_000.0 { 8.0 } else { 0.8 };
            p.fuel = (p.fuel + rate * dt).min(to.min(self.spec.fuel_kg));
            if p.fuel >= to.min(self.spec.fuel_kg) - 0.01 {
                self.ground.fuel_to = None;
                self.status = tr_en("Refuelling complete.").into();
                self.status_until = self.real + 4.0;
            }
        }
        // passengers and bags, a couple a second.
        if let Some(to) = self.ground.board_to {
            let rate = (self.spec.payload[1] / 600.0).max(2.0);
            p.payload = if p.payload < to { (p.payload + rate * dt).min(to) } else { (p.payload - rate * dt).max(to) };
            if (p.payload - to).abs() < 0.01 {
                self.ground.board_to = None;
                self.status = tr_en("Boarding complete.").into();
                self.status_until = self.real + 4.0;
            }
        }
    }

    /// the nearest gate a bridge can swing from, on the left ahead of the wing.
    fn find_gate(&self) -> Option<DVec3> {
        let field = self.world.airfield_near(self.plane.pos, 3_000.0)?;
        let g = &field.geo;
        let door = g.local(self.on_ground_at(self.door_body()));
        let left = g.local(self.plane.pos + self.plane.rot * DVec3::NEG_X * 10.0) - g.local(self.plane.pos);
        let gp = g.gates.iter().map(|gt| gt.0).filter(|gp| {
            let run = door - *gp;
            (9.0..75.0).contains(&run.length()) && (*gp - door).dot(left.normalize_or_zero()) > 0.0
        }).min_by(|a, b| (*a - door).length().total_cmp(&(*b - door).length()))?;
        Some(g.world(gp, g.h(gp) as f64))
    }

    /// the tug, the bridge and the vehicles, for the renderer (camera relative).
    pub(crate) fn ground_instances(&self, cam: DVec3, out: &mut Vec<(u32, Inst)>) {
        if !self.plane.on_ground || self.crashed || !self.ground.busy() {
            return;
        }
        let up = self.plane.pos.normalize();
        let fwd_w = {
            let f = self.plane.forward();
            (f - up * f.dot(up)).normalize_or_zero()
        };
        let put = |out: &mut Vec<(u32, Inst)>, mesh: u32, at: DVec3, face: DVec3, scale: Vec3, tint: [f32; 3]| {
            let o = (at - cam).as_vec3();
            let u = at.normalize().as_vec3();
            let f = (face.as_vec3() - u * face.as_vec3().dot(u)).normalize_or_zero();
            let x = f.cross(u) * scale.x;
            let z = -f * scale.z;
            let u = u * scale.y;
            out.push((mesh, Inst { m: [[x.x, u.x, z.x, o.x], [x.y, u.y, z.y, o.y], [x.z, u.z, z.z, o.z]], tint: [tint[0], tint[1], tint[2], 0.0], ext: [0.0; 4] }));
        };
        let white = [0.85, 0.85, 0.82];
        let s = self.spec;
        let nose_z = s.hard[0].z as f32;
        let tail_z = s.hard[3].z as f32;
        let half = s.span;
        // the tug, nose to nose with the aircraft.
        if self.ground.push.is_some() {
            let at = self.on_ground_at(Vec3::new(0.0, 0.0, nose_z - 3.2));
            put(out, if s.mass > 20_000.0 { am::M_PUSHBACK } else { am::M_TUG }, at, -fwd_w, Vec3::ONE, [1.0; 3]);
        }
        let right_w = fwd_w.cross(up);
        // fuel at the right wing.
        if self.ground.fuel_to.is_some() {
            let at = self.on_ground_at(Vec3::new(half * 0.45 + 2.5, 0.0, nose_z * 0.2));
            put(out, am::M_FUEL, at + fwd_w * 3.0, -fwd_w, Vec3::ONE, white);
        }
        // the belt loader to the aft hold and the carts behind it.
        if self.ground.baggage || self.ground.board_to.is_some() {
            let hold = Vec3::new(s.hole.fus[0] + 4.0, 0.0, tail_z * 0.45);
            let at = self.on_ground_at(hold);
            put(out, am::M_LOADER, at, -right_w, Vec3::ONE, white);
            let tug = at + right_w * 5.0 - fwd_w * 3.0;
            put(out, am::M_TUG, tug, fwd_w, Vec3::ONE, white);
            for k in 0..3 {
                put(out, am::M_CART, tug - fwd_w * (4.6 + 3.6 * k as f64), fwd_w, Vec3::ONE, white);
            }
        }
        // stairs at the front door.
        if self.ground.stairs {
            let d = self.door_body();
            let at = self.on_ground_at(Vec3::new(d.x - 2.8, 0.0, d.z));
            put(out, am::M_STAIRS, at, right_w, Vec3::ONE, white);
        }
        // the bridge from its gate, driven out to the door.
        if let (Some(gate), true) = (self.ground.gate, self.ground.bridge > 0.0) {
            let door = self.on_ground_at(self.door_body());
            let sill = (self.plane.pos + self.plane.rot * self.door_body().as_dvec3() - door).length() as f32;
            let lift = |p: DVec3| p + p.normalize() * (sill as f64 - 0.2);
            let run = door + right_w * 0.0 - gate;
            let len = run.length();
            let parked = (len * 0.55).max(8.0);
            let reach = parked + (len - 2.6 - parked) * self.ground.bridge as f64;
            let dir = run / len.max(1.0);
            put(out, am::M_BRIDGE, lift(gate), dir, Vec3::new(1.0, 1.0, reach as f32), [1.0; 3]);
            put(out, am::M_BRIDGE_HEAD, lift(gate + dir * reach), right_w, Vec3::ONE, [1.0; 3]);
        }
    }

    /// the ground page of the flight panel: pushback, bridge or stairs, fuel and boarding.
    pub(crate) fn ground_page(&mut self, ui: &mut egui::Ui) {
        use flightui::{cells, key, note, tag, wide_tile};
        let p = &self.plane;
        let stopped = p.on_ground && p.vel.length() < 0.5;
        if !p.on_ground {
            if self.ground.winch.is_some() {
                if key(ui, tr_en("Release the cable"), false, ui.available_width(), 26.0).clicked() {
                    self.ground.winch = None;
                    self.plane.tow = None;
                }
                return;
            }
            note(ui, tr_en("Ground services come to an aircraft standing on the ground."));
            return;
        }
        let big = self.spec.mass > 20_000.0;
        let mut act: Option<&'static str> = None;
        // a glider launches on a winch cable.
        if self.spec.fuel_kg <= 0.0 && self.spec.rotor.is_none() {
            wide_tile(ui, |ui| {
                tag(ui, tr_en("Winch launch"));
                note(ui, tr_en("The winch pulls you up to about 1,500 feet and lets go. Hold the nose up gently once airborne."));
                if key(ui, if self.ground.winch.is_some() { tr_en("Launching") } else { tr_en("Take up slack, all out") }, self.ground.winch.is_some(), ui.available_width(), 26.0).clicked() && stopped && self.ground.winch.is_none() {
                    act = Some("winch");
                }
            });
        }
        wide_tile(ui, |ui| {
            tag(ui, tr_en("Pushback"));
            match self.ground.push {
                Some(push) => {
                    note(ui, &format!("{} {:.0} m", tr_en("Pushing back"), push.gone));
                    ui.horizontal(|ui| {
                        for (label, s) in [(tr_en("Tail left"), -1.0), (tr_en("Straight"), 0.0), (tr_en("Tail right"), 1.0)] {
                            if key(ui, label, (push.steer - s).abs() < 0.01, 110.0, 26.0).clicked() {
                                act = Some(if s < 0.0 { "left" } else if s > 0.0 { "right" } else { "straight" });
                            }
                        }
                        if key(ui, tr_en("Stop"), false, 80.0, 26.0).clicked() {
                            act = Some("stop");
                        }
                    });
                }
                None => {
                    if key(ui, tr_en("Request pushback"), false, ui.available_width(), 26.0).clicked() && stopped {
                        act = Some("push");
                    }
                }
            }
        });
        cells(ui, 2, 96.0, |ui, i| {
            if i == 0 {
                tag(ui, if big { tr_en("Jet bridge") } else { tr_en("Stairs") });
                if big {
                    let state = if self.ground.bridge >= 1.0 { tr_en("At the door") } else if self.ground.bridge > 0.0 { tr_en("Moving") } else { tr_en("Parked") };
                    note(ui, state);
                    if key(ui, if self.ground.bridge_on { tr_en("Retract") } else { tr_en("Connect") }, self.ground.bridge_on, ui.available_width(), 26.0).clicked() {
                        act = Some("bridge");
                    }
                }
                if key(ui, if self.ground.stairs { tr_en("Remove stairs") } else { tr_en("Stairs") }, self.ground.stairs, ui.available_width(), 26.0).clicked() {
                    act = Some("stairs");
                }
            } else {
                tag(ui, tr_en("Fuel"));
                let full = self.spec.fuel_kg.max(1.0);
                note(ui, &format!("{:.0} kg  ·  {:.0} %", self.plane.fuel, self.plane.fuel / full * 100.0));
                ui.horizontal(|ui| {
                    for (label, k) in [("50 %", 0.5), ("75 %", 0.75), ("100 %", 1.0)] {
                        if key(ui, label, self.ground.fuel_to.is_some_and(|t| (t - full * k).abs() < 1.0), 64.0, 26.0).clicked() {
                            act = Some(match k { x if x < 0.6 => "f50", x if x < 0.9 => "f75", _ => "f100" });
                        }
                    }
                });
            }
        });
        wide_tile(ui, |ui| {
            tag(ui, tr_en("Boarding and baggage"));
            note(ui, &format!("{} {:.0} kg  ·  {} {:.0} kg", tr_en("payload"), self.plane.payload, tr_en("most"), self.spec.payload[1]));
            ui.horizontal(|ui| {
                if key(ui, tr_en("Board"), self.ground.board_to.is_some(), 120.0, 26.0).clicked() {
                    act = Some("board");
                }
                if key(ui, tr_en("Deboard"), false, 120.0, 26.0).clicked() {
                    act = Some("deboard");
                }
                if key(ui, tr_en("Baggage"), self.ground.baggage, 120.0, 26.0).clicked() {
                    act = Some("bags");
                }
            });
        });
        // (the buttons only act while it stands still: the one line that says why)
        if !stopped {
            note(ui, tr_en("Stop the aircraft for the services to come."));
        }
        let Some(a) = act else { return };
        match a {
            "winch" => {
                self.plane.parking_brake = false;
                self.ground.winch = Some(0.0);
            }
            "push" => {
                self.plane.parking_brake = false;
                self.ground.bridge_on = false;
                self.ground.stairs = false;
                self.ground.push = Some(Push { steer: 0.0, gone: 0.0, stopped: false, from: self.plane.pos });
            }
            "left" | "right" | "straight" => {
                if let Some(p) = self.ground.push.as_mut() {
                    p.steer = match a { "left" => -1.0, "right" => 1.0, _ => 0.0 };
                }
            }
            "stop" => {
                if let Some(p) = self.ground.push.as_mut() {
                    p.stopped = true;
                }
            }
            "bridge" if stopped => {
                self.ground.bridge_on = !self.ground.bridge_on;
                if self.ground.bridge_on && self.ground.gate.is_none() {
                    self.ground.gate = self.find_gate();
                    if self.ground.gate.is_none() {
                        self.ground.bridge_on = false;
                        self.ground.stairs = true;
                        self.status = tr_en("No jet bridge reaches this stand: stairs instead.").into();
                        self.status_until = self.real + 4.0;
                    }
                }
            }
            "stairs" if stopped => self.ground.stairs = !self.ground.stairs,
            "f50" | "f75" | "f100" if stopped => {
                let k = match a { "f50" => 0.5, "f75" => 0.75, _ => 1.0 };
                self.ground.fuel_to = Some(self.spec.fuel_kg * k);
            }
            "board" if stopped => self.ground.board_to = Some(self.spec.payload[1] * 0.85),
            "deboard" if stopped => self.ground.board_to = Some(self.spec.payload[0].min(self.spec.payload[1]) * 0.1),
            "bags" if stopped => self.ground.baggage = !self.ground.baggage,
            _ => {}
        }
    }
}
