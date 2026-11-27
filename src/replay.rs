//! the flight replay: the last half hour of flying is kept, twenty samples a
//! second, and can be watched again from the pause menu at a few speeds,
//! scrubbed back and forth, and flown on from any moment.

use crate::plane::Plane;
use crate::{flightui, menu, profile, tr_en, Game};
use glam::{DQuat, DVec3};
use std::collections::VecDeque;

const RATE: f64 = 20.0;
const KEEP_S: f64 = 1800.0;

/// what is kept of a moment: enough to pose the aircraft and move the camera.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub t: f64,
    pub pos: DVec3,
    pub vel: DVec3,
    pub rot: DQuat,
    pub omega: DVec3,
    pub stick: DVec3,
    pub throttle: f64,
    pub flaps: f64,
    pub gear: f64,
    pub speedbrake: f64,
    pub canopy: f64,
    pub burner: f64,
    pub n1: f64,
    pub prop: f64,
    pub prop_rate: f64,
    pub on_ground: bool,
    pub gear_down: bool,
    pub flap_set: u8,
}

impl Sample {
    pub fn of(t: f64, p: &Plane) -> Self {
        Self { t, pos: p.pos, vel: p.vel, rot: p.rot, omega: p.omega, stick: p.stick, throttle: p.throttle, flaps: p.flaps, gear: p.gear, speedbrake: p.speedbrake, canopy: p.canopy, burner: p.burner, n1: p.n1, prop: p.prop, prop_rate: p.prop_rate, on_ground: p.on_ground, gear_down: p.gear_down, flap_set: p.flap_set }
    }

    pub fn apply(&self, p: &mut Plane) {
        (p.pos, p.vel, p.rot, p.omega, p.stick) = (self.pos, self.vel, self.rot, self.omega, self.stick);
        (p.throttle, p.flaps, p.gear, p.speedbrake, p.canopy) = (self.throttle, self.flaps, self.gear, self.speedbrake, self.canopy);
        (p.burner, p.n1, p.prop, p.prop_rate) = (self.burner, self.n1, self.prop, self.prop_rate);
        (p.on_ground, p.gear_down, p.flap_set) = (self.on_ground, self.gear_down, self.flap_set);
    }

    fn lerp(&self, b: &Sample, k: f64) -> Sample {
        let m = |x: f64, y: f64| x + (y - x) * k;
        Sample {
            t: m(self.t, b.t),
            pos: self.pos.lerp(b.pos, k),
            vel: self.vel.lerp(b.vel, k),
            rot: self.rot.slerp(b.rot, k),
            omega: self.omega.lerp(b.omega, k),
            stick: self.stick.lerp(b.stick, k),
            throttle: m(self.throttle, b.throttle),
            flaps: m(self.flaps, b.flaps),
            gear: m(self.gear, b.gear),
            speedbrake: m(self.speedbrake, b.speedbrake),
            canopy: m(self.canopy, b.canopy),
            burner: m(self.burner, b.burner),
            n1: m(self.n1, b.n1),
            prop: m(self.prop, b.prop),
            prop_rate: m(self.prop_rate, b.prop_rate),
            ..if k < 0.5 { *self } else { *b }
        }
    }
}

#[derive(Default)]
pub struct Recorder {
    buf: VecDeque<Sample>,
    next: f64,
}

impl Recorder {
    pub fn push(&mut self, t: f64, p: &Plane) {
        if t < self.next {
            return;
        }
        // a jump (a respawn, a teleport) starts the recording over.
        if self.buf.back().is_some_and(|b| (b.pos - p.pos).length() > 2_000.0 || t < b.t) {
            self.buf.clear();
        }
        self.next = t + 1.0 / RATE;
        self.buf.push_back(Sample::of(t, p));
        while self.buf.front().is_some_and(|f| f.t < t - KEEP_S) {
            self.buf.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.buf.clear();
        self.next = 0.0;
    }

    pub fn span(&self) -> Option<(f64, f64)> {
        Some((self.buf.front()?.t, self.buf.back()?.t))
    }

    /// the moment `t`, between the samples either side.
    pub fn at(&self, t: f64) -> Option<Sample> {
        let n = self.buf.len();
        if n == 0 {
            return None;
        }
        let i = self.buf.partition_point(|s| s.t <= t);
        if i == 0 {
            return self.buf.front().copied();
        }
        if i >= n {
            return self.buf.back().copied();
        }
        let (a, b) = (&self.buf[i - 1], &self.buf[i]);
        let k = ((t - a.t) / (b.t - a.t).max(1e-6)).clamp(0.0, 1.0);
        Some(a.lerp(b, k))
    }

    /// forgets everything after `t` (flying on from there).
    pub fn cut(&mut self, t: f64) {
        while self.buf.back().is_some_and(|b| b.t > t) {
            self.buf.pop_back();
        }
        self.next = 0.0;
    }
}

/// watching the replay.
pub struct Playback {
    pub t: f64,
    pub speed: f64,
    pub paused: bool,
    /// where the flight was when the replay began, to go back to.
    pub resume: Sample,
}

impl Game {
    pub(crate) fn start_replay(&mut self) {
        let Some((a, b)) = self.rec.span() else { return };
        self.replay = Some(Playback { t: a.max(b - 120.0), speed: 1.0, paused: false, resume: Sample::of(self.time, &self.plane) });
        self.ap.disengage();
    }

    /// one frame of the replay: the clock moves on and the aircraft goes where it was.
    pub(crate) fn replay_step(&mut self, dt: f64) {
        let Some(r) = self.replay.as_mut() else { return };
        if !r.paused {
            r.t += dt * r.speed;
        }
        let (a, b) = self.rec.span().unwrap_or((r.t, r.t));
        if r.t >= b {
            r.t = b;
            r.paused = true;
        }
        r.t = r.t.max(a);
        if let Some(s) = self.rec.at(r.t) {
            s.apply(&mut self.plane);
        }
    }

    /// leaves the replay: back where the flight was, or flying on from the moment shown.
    pub(crate) fn end_replay(&mut self, fly_from_here: bool) {
        let Some(r) = self.replay.take() else { return };
        if fly_from_here {
            self.rec.cut(r.t);
            self.acc = 0.0;
        } else {
            r.resume.apply(&mut self.plane);
        }
    }

    /// the replay's bar along the foot of the screen.
    pub(crate) fn replay_bar(&mut self, ctx: &egui::Context) {
        use egui::{Color32, RichText};
        let Some((a, b)) = self.rec.span() else { return };
        let Some(r) = self.replay.as_mut() else { return };
        let mut exit: Option<bool> = None;
        egui::Area::new(egui::Id::unique("replay_bar")).order(egui::Order::Foreground).anchor(egui::Align2::CENTER_BOTTOM, [0.0, -24.0]).show(ctx, |ui| {
            flightui::panel(ui, flightui::glass().inner_margin(12.0), |ui| {
                ui.set_width(720.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.label(RichText::new(tr_en("REPLAY")).font(menu::semibold(14.0)).color(flightui::LIT));
                    ui.label(RichText::new(format!("−{}", profile::hhmm(b - r.t).replace(':', "m ")) + "s").font(menu::caption()).color(Color32::WHITE));
                    if flightui::key(ui, if r.paused { "▶" } else { "❚❚" }, false, 36.0, 26.0).clicked() {
                        if r.t >= b - 0.01 {
                            r.t = a;
                        }
                        r.paused = !r.paused;
                    }
                    for s in [0.25, 0.5, 1.0, 2.0, 4.0] {
                        let label = if s < 1.0 { format!("{s}×") } else { format!("{s:.0}×") };
                        if flightui::key(ui, &label, (r.speed - s).abs() < 1e-6, 44.0, 26.0).clicked() {
                            r.speed = s;
                        }
                    }
                    menu::at_right(ui, "replay_exit", |ui| {
                        if flightui::key(ui, tr_en("Exit"), false, 64.0, 26.0).clicked() {
                            exit = Some(false);
                        }
                        if flightui::key(ui, tr_en("Fly from here"), false, 120.0, 26.0).clicked() {
                            exit = Some(true);
                        }
                    });
                });
                ui.add_space(4.0);
                let mut t = r.t;
                ui.spacing_mut().slider_width = 690.0;
                if ui.add(egui::Slider::new(&mut t, a..=b).show_value(false)).changed() {
                    r.t = t;
                }
            });
        });
        if let Some(fly) = exit {
            self.end_replay(fly);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_in_between_are_interpolated_and_the_tail_is_cut() {
        let mut rec = Recorder::default();
        let mut p = Plane::spawn(&crate::aircraft::c172::SPEC, 0.6, -2.1, 1000.0, 0.0, 55.0);
        for k in 0..100 {
            p.pos += DVec3::X * 10.0;
            rec.push(k as f64 * 0.05, &p);
        }
        let (a, b) = rec.span().unwrap();
        assert!((b - a - 4.95).abs() < 1e-9);
        let mid = rec.at(0.075).unwrap();
        let (s1, s2) = (rec.at(0.05).unwrap(), rec.at(0.1).unwrap());
        assert!((mid.pos - (s1.pos + s2.pos) * 0.5).length() < 1e-6);
        rec.cut(2.0);
        assert!(rec.span().unwrap().1 <= 2.0);
    }
}
