//! the autopilot's controls on screen, laid out as a mode control panel: a
//! column for each thing it flies (the master, heading, the courses, altitude,
//! vertical speed and speed). each column has its mode button, green while it
//! flies, outlined while armed, and under it the value it aims for, set with
//! minus and plus (held to repeat), the wheel (shift for big steps), or a
//! click that takes what the aircraft is doing now. the same panel sits at the
//! bottom of the screen and on the autopilot page.

use crate::autopilot::{self, Lat, Vert};
use crate::flightui;
use crate::menu::{self, DIM, FAINT, OK, TEXT};
use crate::{t, tr_en, Game};
use egui::{pos2, vec2, Align2, Color32, Rect, Sense, Stroke};

/// what a value window did this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Turn {
    pub fine: i32,
    pub coarse: i32,
    pub push: bool,
}

impl Turn {
    pub fn any(&self) -> bool {
        self.fine != 0 || self.coarse != 0 || self.push
    }
}

/// what one value window sets: heading, altitude, vertical speed, speed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dial {
    Hdg,
    Alt,
    Vs,
    Spd,
}

/// a mode button's light.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lamp {
    Off,
    Armed,
    On,
}

/// the panel's sizes: a mode button and a value window.
const KEY_H: f32 = 28.0;
const VALUE_H: f32 = 32.0;
const GAP: f32 = 8.0;

/// a mode button `w` wide, green while it flies, outlined while armed.
fn mode_key(ui: &mut egui::Ui, text: &str, lamp: Lamp, w: f32) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(w, KEY_H), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, lamp == Lamp::On, text));
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let p = ui.painter();
    let (fill, edge, col) = match lamp {
        Lamp::On => (OK.gamma_multiply(0.16 + 0.06 * lit), OK, OK),
        Lamp::Armed => (menu::mix(menu::RAISED, menu::RAISED_HI, lit), Color32::from_white_alpha(150), Color32::WHITE),
        Lamp::Off => (menu::mix(menu::RAISED, menu::RAISED_HI, lit), menu::mix(menu::LINE_HI, Color32::from_white_alpha(60), lit), menu::mix(DIM, TEXT, lit)),
    };
    p.rect_filled(r, menu::R_WIDGET, fill);
    p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
    p.text(r.center(), Align2::CENTER_CENTER, text, menu::semibold(13.5), col);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// a small minus or plus that steps once when pressed and repeats while held.
fn stepper(ui: &mut egui::Ui, r: Rect, plus: bool) -> i32 {
    let resp = ui.interact(r, ui.scope_id().with(("ap step", plus, r.min.x as i32)), Sense::click_and_drag());
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered());
    let p = ui.painter();
    if lit > 0.0 {
        p.rect_filled(r.shrink(2.0), menu::R_WIDGET, Color32::from_white_alpha((14.0 * lit) as u8));
    }
    let c = r.center();
    let st = Stroke::new(1.6, menu::mix(DIM, Color32::WHITE, lit));
    p.line_segment([c - vec2(4.5, 0.0), c + vec2(4.5, 0.0)], st);
    if plus {
        p.line_segment([c - vec2(0.0, 4.5), c + vec2(0.0, 4.5)], st);
    }
    // a step on the press, then every 70 ms after 400 ms held.
    let now = ui.input(|i| i.time);
    let id = resp.id.with("held");
    let down = resp.is_pointer_button_down_on();
    let mut n = 0;
    match (down, ui.data(|d| d.get_temp::<(f64, f64)>(id))) {
        (true, None) => {
            n = 1;
            ui.data_mut(|d| d.insert_temp(id, (now, now)));
        }
        (true, Some((since, last))) => {
            if now - since > 0.4 && now - last > 0.07 {
                n = 1;
                ui.data_mut(|d| d.insert_temp(id, (since, now)));
            }
            ui.ctx().request_repaint();
        }
        (false, Some(_)) => ui.data_mut(|d| d.remove::<(f64, f64)>(id)),
        (false, None) => {}
    }
    if plus { n } else { -n }
}

/// the value window: minus, the value and its unit, plus. the wheel over it
/// turns it, a click takes the aircraft's own value.
fn target(ui: &mut egui::Ui, value: &str, unit: &str, w: f32, live: bool) -> Turn {
    let (r, resp) = ui.allocate_exact_size(vec2(w, VALUE_H), Sense::click());
    let mut out = Turn::default();
    let shift = ui.input(|i| i.modifiers.shift);
    let p = ui.painter().clone();
    p.rect_filled(r, menu::R_WIDGET, Color32::from_rgba_premultiplied(4, 6, 9, 200));
    p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, Color32::from_white_alpha(18)), egui::StrokeKind::Inside);
    let (minus, plus) = (Rect::from_min_size(r.min, vec2(24.0, VALUE_H)), Rect::from_min_size(pos2(r.right() - 24.0, r.top()), vec2(24.0, VALUE_H)));
    let n = stepper(ui, minus, false) + stepper(ui, plus, true);
    if n != 0 {
        if shift { out.coarse += n } else { out.fine += n }
    }
    let mid = Rect::from_min_max(pos2(minus.right(), r.top()), pos2(plus.left(), r.bottom()));
    let over = resp.hovered() && ui.input(|i| i.pointer.hover_pos()).is_some_and(|q| mid.contains(q));
    if over {
        let wheel: f32 = ui.input(|i| {
            i.events.iter().filter_map(|e| match e {
                egui::Event::MouseWheel { unit, delta, .. } => Some(match unit {
                    egui::MouseWheelUnit::Line => delta.y + delta.x,
                    egui::MouseWheelUnit::Point => (delta.y + delta.x) / 40.0,
                    egui::MouseWheelUnit::Page => (delta.y + delta.x) * 3.0,
                }),
                _ => None,
            }).sum()
        });
        let id = resp.id.with("wheel");
        let acc = ui.data(|d| d.get_temp::<f32>(id)).unwrap_or(0.0) + wheel;
        let k = acc.trunc() as i32;
        ui.data_mut(|d| d.insert_temp(id, acc - k as f32));
        if k != 0 {
            if shift { out.coarse += k } else { out.fine += k }
        }
        // the wheel turns the value, not the page.
        ui.ctx().input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
        p.rect_filled(mid.shrink(2.0), menu::R_WIDGET, Color32::from_white_alpha(8));
    }
    if resp.clicked() && over {
        out.push = true;
    }
    // the value and its unit, centred together; where the window is narrow the unit goes, then the figure shrinks.
    let col = if live { Color32::WHITE } else { TEXT };
    let room = mid.width() - 6.0;
    let mut vg = p.layout_no_wrap(value.to_string(), menu::semibold(17.0), col);
    let ug = p.layout_no_wrap(unit.to_string(), menu::caption(), DIM);
    let unit = if vg.size().x + 3.0 + ug.size().x > room { "" } else { unit };
    if vg.size().x > room {
        vg = p.layout_no_wrap(value.to_string(), menu::semibold(14.0), col);
    }
    let total = vg.size().x + if unit.is_empty() { 0.0 } else { 3.0 + ug.size().x };
    let x = mid.center().x - total * 0.5;
    let base = mid.center().y + vg.size().y * 0.5;
    p.galley(pos2(x, mid.center().y - vg.size().y * 0.5), vg, col);
    if !unit.is_empty() {
        p.galley(pos2(x + total - ug.size().x, base - ug.size().y - 1.0), ug, DIM);
    }
    resp.on_hover_text(t!("ap.knob_hint"));
    out
}

/// what the panel asked for this frame.
#[derive(Default)]
struct Asked {
    toggle: bool,
    at: bool,
    lat: Option<Lat>,
    vert: Option<Vert>,
    turns: Vec<(Dial, Turn)>,
    /// time acceleration: steps up or down the rates, or back to 1x.
    warp: Turn,
}

/// the time acceleration rates on the panel.
const RATES: [u8; 5] = [1, 2, 4, 8, 16];

impl Game {
    /// a value window turned: fine and coarse steps, or a push to the aircraft's own value.
    pub(crate) fn ap_turn(&mut self, d: Dial, t: Turn) {
        let s = autopilot::State::of(&self.plane);
        let (f, c) = (t.fine as f64, t.coarse as f64);
        match d {
            Dial::Hdg => {
                self.ap.hdg = if t.push { s.hdg.round() } else { (self.ap.hdg.round() + f + 10.0 * c).rem_euclid(360.0) };
            }
            Dial::Alt => {
                let ft = if t.push { (s.alt * 3.28084 / 100.0).round() * 100.0 } else { ((self.ap.alt * 3.28084 / 100.0).round() * 100.0 + 100.0 * f + 1000.0 * c).clamp(0.0, 90_000.0) };
                self.ap.alt = ft / 3.28084;
            }
            Dial::Vs => {
                let fpm = if t.push { (s.vs * 196.85 / 100.0).round() * 100.0 } else { ((self.ap.vs * 196.85 / 100.0).round() * 100.0 + 100.0 * f + 500.0 * c).clamp(-6000.0, 6000.0) };
                self.ap.vs = fpm / 196.85;
            }
            Dial::Spd => {
                let kt = if t.push { (s.ias * 1.943_84).round() } else { ((self.ap.spd * 1.943_84).round() + f + 10.0 * c).max(40.0) };
                self.ap.spd = kt / 1.943_84;
            }
        }
    }

    /// the value a window sets, as shown.
    pub(crate) fn dial_text(&self, d: Dial) -> (String, &'static str) {
        match d {
            Dial::Hdg => (format!("{:03.0}°", self.ap.hdg.rem_euclid(360.0)), ""),
            Dial::Alt => (format!("{:.0}", (self.ap.alt * 3.28084 / 100.0).round() * 100.0), "ft"),
            Dial::Vs => (format!("{:+.0}", (self.ap.vs * 196.85 / 100.0).round() * 100.0), "fpm"),
            Dial::Spd => (format!("{:.0}", self.ap.spd * 1.943_84), "kt"),
        }
    }

    /// each mode button's light.
    fn lamps(&self) -> [Lamp; 8] {
        let a = &self.ap;
        let on = |b: bool| if a.on && b { Lamp::On } else { Lamp::Off };
        let lat_armed = matches!(a.lat, Lat::Nav | Lat::Apr) && !a.nav_cap || a.lat == Lat::Gps && a.gps_trk.is_none();
        let lat_lamp = |m: Lat| match (a.on, a.lat == m, lat_armed) {
            (true, true, true) => Lamp::Armed,
            (true, true, false) => Lamp::On,
            _ => Lamp::Off,
        };
        // the heading flies while a course is armed.
        let hdg = if a.on && (a.lat == Lat::Hdg || lat_armed) { Lamp::On } else { Lamp::Off };
        let alt = if a.on && a.vert == Vert::Vs && !a.alt_cap { Lamp::Armed } else { on(a.vert == Vert::Alt || a.vert == Vert::Vs && a.alt_cap) };
        let vs = on(a.vert == Vert::Vs && !a.alt_cap);
        [if a.on { Lamp::On } else { Lamp::Off }, hdg, lat_lamp(Lat::Nav), lat_lamp(Lat::Apr), lat_lamp(Lat::Gps), alt, vs, if a.at { Lamp::On } else { Lamp::Off }]
    }

    /// the panel's columns, `col` wide each, laid left to right in `ui`.
    fn ap_columns(&mut self, ui: &mut egui::Ui, col: f32) -> Asked {
        let mut ask = Asked::default();
        let l = self.lamps();
        ui.spacing_mut().item_spacing = vec2(GAP, 4.0);
        // the master, as tall as a column.
        let (r, resp) = ui.allocate_exact_size(vec2(56.0, KEY_H + 4.0 + VALUE_H), Sense::click());
        {
            let lit = menu::ease(ui.ctx(), resp.id, resp.hovered());
            let p = ui.painter();
            let on = l[0] == Lamp::On;
            p.rect_filled(r, menu::R_WIDGET, if on { OK.gamma_multiply(0.18 + 0.06 * lit) } else { menu::mix(menu::RAISED, menu::RAISED_HI, lit) });
            p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, if on { OK } else { menu::mix(menu::LINE_HI, Color32::from_white_alpha(60), lit) }), egui::StrokeKind::Inside);
            p.text(r.center() - vec2(0.0, 7.0), Align2::CENTER_CENTER, "AP", menu::semibold(17.0), if on { OK } else { TEXT });
            p.text(r.center() + vec2(0.0, 12.0), Align2::CENTER_CENTER, if on { t!("menu.on") } else { t!("menu.off") }, menu::caption(), if on { OK } else { FAINT });
        }
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, l[0] == Lamp::On, "AP"));
        if resp.on_hover_text("F5").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            ask.toggle = true;
        }
        let rule = |ui: &mut egui::Ui| {
            let x = ui.cursor().left() - GAP * 0.5 - 0.5;
            let y = ui.cursor().top();
            ui.painter().vline(x, y..=y + KEY_H + 4.0 + VALUE_H, Stroke::new(1.0, Color32::from_white_alpha(18)));
        };
        // heading, then the courses, altitude, vertical speed and speed.
        let column = |ui: &mut egui::Ui, add: &mut dyn FnMut(&mut egui::Ui)| {
            ui.allocate_ui_with_layout(vec2(col, KEY_H + 4.0 + VALUE_H), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                add(ui);
            });
        };
        let live = self.ap.on;
        let mut hdg_t = Turn::default();
        rule(ui);
        column(ui, &mut |ui| {
            if mode_key(ui, "HDG", l[1], col).on_hover_text("7").clicked() {
                ask.lat = Some(Lat::Hdg);
            }
            let (v, u) = self.dial_text(Dial::Hdg);
            hdg_t = target(ui, &v, u, col, live);
        });
        rule(ui);
        let course = match &self.radios.nav {
            Some(s) => format!("{} {}", s.name, s.ident),
            None if !self.plan.is_empty() => tr_en("Flight plan").to_string(),
            None => tr_en("No course").to_string(),
        };
        let tuned = self.radios.nav.is_some() || !self.plan.is_empty();
        let wide = col * 1.5;
        column(ui, &mut |ui| {
            ui.set_width(wide);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let kw = (wide - 8.0) / 3.0;
                for (k, m, name, key) in [(2, Lat::Nav, "NAV", "8"), (3, Lat::Apr, "APR", "9"), (4, Lat::Gps, "GPS", "")] {
                    let r = mode_key(ui, name, l[k], kw);
                    let r = if key.is_empty() { r.on_hover_text(t!("game.gps_follows_the_flight_plan")) } else { r.on_hover_text(key) };
                    if r.clicked() {
                        ask.lat = Some(m);
                    }
                }
            });
            let (r, _) = ui.allocate_exact_size(vec2(wide, VALUE_H), Sense::hover());
            let g = menu::one_line(ui.painter(), &course, menu::label(), if tuned { TEXT } else { FAINT }, wide - 8.0);
            ui.painter().galley(r.center() - g.size() * 0.5, g, TEXT);
        });
        let mut rest: [Turn; 3] = Default::default();
        for (i, (d, name, key, k)) in [(Dial::Alt, "ALT", "0", 5), (Dial::Vs, "VS", "", 6), (Dial::Spd, "A/T", "", 7)].into_iter().enumerate() {
            rule(ui);
            column(ui, &mut |ui| {
                let r = mode_key(ui, name, l[k], col);
                let r = if d == Dial::Spd { r.on_hover_text(t!("game.autothrottle")) } else if key.is_empty() { r } else { r.on_hover_text(key) };
                if r.clicked() {
                    match d {
                        Dial::Alt => ask.vert = Some(Vert::Alt),
                        Dial::Vs => ask.vert = Some(Vert::Vs),
                        _ => ask.at = true,
                    }
                }
                let (v, u) = self.dial_text(d);
                rest[i] = target(ui, &v, u, col, live || d == Dial::Spd && self.ap.at);
            });
        }
        // time acceleration, lit while the sim runs faster; only while the autopilot flies.
        rule(ui);
        let fast = self.sim_rate() > 1.0;
        column(ui, &mut |ui| {
            let lamp = if fast { Lamp::On } else { Lamp::Off };
            mode_key(ui, tr_en("Rate"), lamp, col).on_hover_text(tr_en("Time runs faster while the autopilot flies."));
            ask.warp = target(ui, &format!("{}×", self.warp), "", col, self.ap.on);
        });
        for (d, t) in [(Dial::Hdg, hdg_t), (Dial::Alt, rest[0]), (Dial::Vs, rest[1]), (Dial::Spd, rest[2])] {
            if t.any() {
                ask.turns.push((d, t));
            }
        }
        ask
    }

    /// does what the panel asked for.
    fn ap_apply(&mut self, ask: Asked) {
        // the rate steps through 1, 2, 4, 8 and 16; a click on it goes back to 1x. it waits for the autopilot.
        if ask.warp.any() && self.ap.on && !self.plane.on_ground {
            let k = RATES.iter().position(|r| *r == self.warp).unwrap_or(0) as i32;
            let n = if ask.warp.push { 0 } else { (k + ask.warp.fine + ask.warp.coarse * 2).clamp(0, RATES.len() as i32 - 1) };
            self.warp = RATES[n as usize];
        }
        for (d, t) in ask.turns {
            self.ap_turn(d, t);
        }
        if ask.at {
            self.ap_at_toggle();
        }
        if ask.toggle {
            self.ap_toggle();
        }
        if let Some(m) = ask.lat {
            self.ap_lat(m);
        }
        if let Some(m) = ask.vert {
            self.ap_vert(m);
        }
    }

    /// nothing set yet: the values start from what the aircraft is doing.
    fn ap_seed(&mut self) {
        if !self.ap.on && self.ap.alt == 0.0 {
            let s = autopilot::State::of(&self.plane);
            self.ap.sync(&s);
            self.ap.spd = s.ias;
        }
    }

    /// the panel at the bottom of the screen, up once picked or engaged.
    pub(crate) fn ap_bar(&mut self, ctx: &egui::Context) {
        if !self.ap_bar_on || self.crashed || self.spec.rocket().is_some() || self.info_open {
            return;
        }
        self.ap_seed();
        let mut ask = Asked::default();
        let mut close = false;
        let mut page = false;
        egui::Area::new(egui::Id::unique("ap_bar")).anchor(Align2::CENTER_BOTTOM, [0.0, -16.0]).show(ctx, |ui| {
            flightui::panel(ui, flightui::glass().inner_margin(egui::Margin::same(12)), |ui| {
                ui.horizontal_top(|ui| {
                    ask = self.ap_columns(ui, 120.0);
                    // hide, and the whole autopilot page, stacked at the end.
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 4.0;
                        close = flightui::icon_button(ui, menu::Icon::Close, t!("ap.hide")).clicked();
                        page = flightui::icon_button(ui, menu::Icon::Autopilot, t!("ap.more")).clicked();
                    });
                });
            });
        });
        self.ap_apply(ask);
        if close {
            self.ap_bar_on = false;
        }
        if page {
            self.toolbar_act(flightui::Bar::Page(menu::Page::Autopilot));
        }
    }

    /// the same panel on the autopilot page, as wide as the card.
    pub(crate) fn ap_page_panel(&mut self, ui: &mut egui::Ui) {
        self.ap_seed();
        // the master, the courses (one and a half) and five columns share the width.
        let col = ((ui.available_width() - 56.0 - GAP * 6.0) / 6.5).floor();
        let ask = ui.horizontal_top(|ui| self.ap_columns(ui, col)).inner;
        self.ap_apply(ask);
    }

    /// in ALT, a new altitude set in its window starts a climb or descent to it
    /// once it has been still a moment; called each step with the autopilot on.
    pub(crate) fn ap_auto_level(&mut self, s: &autopilot::State) {
        if (self.ap.alt - self.ap_alt_seen.0).abs() > 0.5 {
            self.ap_alt_seen = (self.ap.alt, self.real);
        }
        if self.ap.vert != Vert::Alt || self.plane.on_ground {
            return;
        }
        let to_go = self.ap.alt - self.ap.holding();
        if to_go.abs() > 90.0 && self.real - self.ap_alt_seen.1 > 1.2 {
            let rate = autopilot::level_rate(self.spec);
            self.ap.level_change(s, rate);
            let ft = format!("{:.0}", (self.ap.alt * 3.28084 / 100.0).round() * 100.0);
            let fpm = format!("{:.0}", rate * 196.85);
            self.status = if to_go > 0.0 { t!("ap.climbing_to", ft = ft, fpm = fpm) } else { t!("ap.descending_to", ft = ft, fpm = fpm) };
            self.status_until = self.real + 4.0;
        }
    }
}
