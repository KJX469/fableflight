//! The interface over the flying itself, beside the instruments (`flight_hud`):
//!
//! - the flight toolbar, a row of icons that slides down when the pointer
//!   reaches the top edge (or while the flight panel is open) and opens the
//!   panel's pages, the minimap, the next camera and the pause menu;
//! - notices at the top centre: an airport's name, runway and wind for a few
//!   seconds when you arrive at it, and the game's status messages, fading;
//! - the guidance line above the instruments: on a runway, at a holding
//!   point, or lined up on an approach (live, so it stays while it applies);
//! - a systems card at the top left, only for the aircraft that have
//!   something to show there: a rocket's flight figures, a fighter's weapons.
use crate::menu::{self, Icon, Page, ACCENT, DIM, TEXT};
use crate::{padui, t, Game};
use egui::{pos2, vec2, Align2, Color32, Rect, RichText, Sense, Stroke};

/// What the toolbar asked for this frame.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Bar {
    Page(Page),
    Map,
    Camera,
    Pause,
}

/// How long the toolbar stays after the pointer leaves the top edge (s), and
/// how near the edge counts (points).
const BAR_LINGER: f64 = 1.2;
const BAR_EDGE: f32 = 64.0;
/// How long an airport notice shows (s), the last of it fading.
const NOTE_SHOW: f64 = 7.0;
const NOTE_FADE: f64 = 1.0;

/// A glass panel for anything over the flying.
pub fn glass() -> egui::Frame {
    egui::Frame::new().fill(menu::GLASS).stroke(Stroke::new(1.0, menu::LINE)).corner_radius(menu::R_PANEL).inner_margin(egui::Margin::symmetric(14, 10))
}

impl Game {
    /// Draws the toolbar when it is wanted; a press goes into `bar`.
    pub(crate) fn toolbar(&mut self, ctx: &egui::Context, bar: &mut Option<Bar>) {
        let screen = ctx.content_rect();
        let near_top = ctx.pointer_latest_pos().is_some_and(|p| p.y < BAR_EDGE && screen.contains(p));
        if near_top || self.info_open {
            self.toolbar_t = self.real;
        }
        let want = self.real - self.toolbar_t < BAR_LINGER;
        let show = ctx.animate_bool_with_time_and_easing(egui::Id::unique("toolbar_show"), want, 0.2, egui::emath::easing::cubic_out);
        if show <= 0.0 {
            return;
        }
        if want && show < 1.0 {
            ctx.request_repaint();
        }
        let items: [(Icon, &str, Bar, bool); 8] = [
            (Icon::Flight, t!("Flight", "Vol"), Bar::Page(Page::Flight), self.info_open && self.info_page == Page::Flight),
            (Icon::GoTo, t!("Go to", "Aller à"), Bar::Page(Page::Places), self.info_open && self.info_page == Page::Places),
            (Icon::Radio, t!("Radio", "Radio"), Bar::Page(Page::Radio), self.info_open && self.info_page == Page::Radio),
            (Icon::Autopilot, t!("Autopilot", "Pilote auto"), Bar::Page(Page::Autopilot), self.info_open && self.info_page == Page::Autopilot),
            (Icon::Weather, t!("Weather", "Météo"), Bar::Page(Page::Weather), self.info_open && self.info_page == Page::Weather),
            (Icon::Map, t!("Map", "Carte"), Bar::Map, self.minimap.visible),
            (Icon::Camera, if self.cam_mode == 1 { t!("Cockpit", "Cockpit") } else { t!("Chase", "Extérieur") }, Bar::Camera, false),
            (Icon::Pause, t!("Pause", "Pause"), Bar::Pause, false),
        ];
        let (bw, bh, gap) = (68.0, 54.0, 2.0);
        // a small gap before the camera and pause, which are not pages
        let w = bw * items.len() as f32 + gap * (items.len() - 1) as f32 + 2.0 * 16.0 + 16.0;
        let y = 10.0 - (bh + 24.0) * (1.0 - show);
        egui::Area::new(egui::Id::unique("toolbar")).order(egui::Order::Foreground).fixed_pos(pos2(screen.center().x - w * 0.5, screen.top() + y)).show(ctx, |ui| {
            ui.multiply_opacity(show);
            glass().inner_margin(egui::Margin::symmetric(8, 6)).show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                ui.horizontal(|ui| {
                    for (k, (ic, label, act, on)) in items.into_iter().enumerate() {
                        if k == 4 || k == 6 {
                            ui.add_space(6.0);
                            let r = ui.available_rect_before_wrap();
                            ui.painter().vline(r.left() - 4.0, r.top() + 10.0..=r.top() + bh - 10.0, Stroke::new(1.0, menu::LINE_HI));
                            ui.add_space(2.0);
                        }
                        let (rect, resp) = ui.allocate_exact_size(vec2(bw, bh), Sense::click());
                        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, label));
                        let lit = menu::ease(ctx, resp.id, resp.hovered() || resp.has_focus());
                        let p = ui.painter();
                        if lit > 0.0 || on {
                            p.rect_filled(rect, menu::R_WIDGET, Color32::from_white_alpha(if on { 16 } else { (12.0 * lit) as u8 }));
                        }
                        if on {
                            p.rect_filled(Rect::from_center_size(pos2(rect.center().x, rect.bottom() - 1.5), vec2(rect.width() * 0.5, 3.0)), 1.0, ACCENT);
                        }
                        let col = if on { Color32::WHITE } else { menu::mix(Color32::from_rgb(200, 207, 218), Color32::WHITE, lit) };
                        menu::icon(p, ic, pos2(rect.center().x, rect.top() + 20.0), 20.0, if on { ACCENT } else { col });
                        p.text(pos2(rect.center().x, rect.bottom() - 13.0), Align2::CENTER_CENTER, label, menu::caption(), col);
                        if resp.clicked() {
                            *bar = Some(act);
                        }
                    }
                });
            });
            // what opens it from the keyboard or the gamepad, under the bar
            let hint = if padui::active() {
                match crate::padmap::spec_of("flight_panel") {
                    k if k.is_empty() => t!("Menu: pause", "Menu : pause").to_string(),
                    k => t!("{}: flight panel   Menu: pause", "{} : panneau de vol   Menu : pause", k),
                }
            } else { t!("Tab: flight panel   Esc: pause", "Tab : panneau de vol   Échap : pause").to_string() };
            let r = ui.min_rect();
            ui.painter().text(pos2(r.center().x, r.bottom() + 12.0), Align2::CENTER_CENTER, hint, menu::caption(), Color32::from_white_alpha(170));
        });
    }

    /// Carries out what the toolbar asked for.
    pub(crate) fn toolbar_act(&mut self, b: Bar) {
        match b {
            Bar::Page(p) => {
                if self.info_open && self.info_page == p {
                    self.info_open = false;
                } else {
                    self.info_open = true;
                    self.info_page = p;
                }
            }
            Bar::Map => self.minimap.visible = !self.minimap.visible,
            Bar::Camera => self.next_view(),
            Bar::Pause => self.set_pause_menu(true),
        }
    }

    /// The notices at the top centre, and the guidance line over the
    /// instruments. `airport`: `airport_hud`'s lines (name, runway and wind,
    /// then the live line, coloured 1 good, 2 caution, 3 warning).
    pub(crate) fn notices(&mut self, ctx: &egui::Context, airport: Option<&[(String, u8)]>) {
        // A new airport (or a new runway in use) brings its notice back.
        let key = airport.map(|l| l.iter().take(2).map(|x| x.0.as_str()).collect::<Vec<_>>().join("|")).unwrap_or_default();
        if key != self.airport_note.0 {
            self.airport_note = (key.clone(), if key.is_empty() { -100.0 } else { self.real });
        }
        let fade = |since: f64, show: f64| -> f32 { (1.0 - ((self.real - since - (show - NOTE_FADE)) / NOTE_FADE).max(0.0)).clamp(0.0, 1.0) as f32 };
        let mut notes: Vec<(f32, Vec<(String, egui::FontId, Color32)>)> = Vec::new();
        if let (Some(lines), a) = (airport, fade(self.airport_note.1, NOTE_SHOW)) {
            if a > 0.0 && !lines.is_empty() {
                let mut v = vec![(lines[0].0.clone(), menu::item_title(), Color32::WHITE)];
                if let Some(l) = lines.get(1) {
                    v.push((l.0.clone(), menu::caption(), TEXT));
                }
                notes.push((a, v));
            }
        }
        // (a status that only repeats the airport the notice above already names is left out)
        let airport_code = airport.and_then(|l| l.first()).and_then(|l| l.0.split_whitespace().next()).unwrap_or("\u{0}");
        let repeats = !notes.is_empty() && self.status.starts_with(airport_code);
        if self.real < self.status_until && !self.status.is_empty() && !repeats {
            let a = ((self.status_until - self.real) / NOTE_FADE).clamp(0.0, 1.0) as f32;
            notes.push((a, vec![(self.status.chars().take(90).collect(), menu::label(), Color32::WHITE)]));
        }
        if !notes.is_empty() {
            // under the toolbar while it is down
            let top = if self.real - self.toolbar_t < BAR_LINGER { 104.0 } else { 14.0 };
            let y = ctx.animate_value_with_time(egui::Id::unique("notes_y"), top, 0.2);
            egui::Area::new(egui::Id::unique("notices")).anchor(Align2::CENTER_TOP, [0.0, y]).interactable(false).show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.vertical_centered(|ui| {
                    for (a, lines) in &notes {
                        // (a scope, so each notice takes its own room in the stack)
                        ui.scope(|ui| {
                            ui.multiply_opacity(*a);
                            glass().inner_margin(egui::Margin::symmetric(18, 10)).show(ui, |ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                for (text, font, col) in lines {
                                    ui.label(RichText::new(text).font(font.clone()).color(*col));
                                }
                            });
                        });
                    }
                });
            });
            ctx.request_repaint();
        }
        // The live line: on a runway, at a holding point, or on an approach.
        let guidance = airport.and_then(|l| l.get(2)).is_some();
        self.ap_strip(ctx, guidance);
        if let Some((text, col)) = airport.and_then(|l| l.get(2)) {
            let col = match col {
                1 => menu::OK,
                2 => Color32::from_rgb(255, 196, 80),
                3 => menu::BAD,
                _ => TEXT,
            };
            egui::Area::new(egui::Id::unique("guidance")).anchor(Align2::CENTER_BOTTOM, [0.0, -14.0 - crate::HUD_H - 10.0]).interactable(false).show(ctx, |ui| {
                glass().inner_margin(egui::Margin::symmetric(14, 6)).show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                        ui.painter().circle_filled(r.center(), 4.0, col);
                        ui.label(RichText::new(text).font(menu::label()).color(TEXT));
                    });
                });
            });
        }
    }

    /// The autopilot's modes over the instruments, as an annunciator strip:
    /// green what it flies, grey what is armed; "AP OFF" flashing amber a
    /// few seconds after it lets go. Above the guidance line when that shows.
    fn ap_strip(&self, ctx: &egui::Context, guidance: bool) {
        let off_for = self.real - self.ap_off_t;
        let flash = !self.ap.on && off_for < 4.0;
        if !self.ap.on && !flash {
            return;
        }
        let lift = if guidance { 44.0 } else { 0.0 };
        egui::Area::new(egui::Id::unique("ap_strip")).anchor(Align2::CENTER_BOTTOM, [0.0, -14.0 - crate::HUD_H - 10.0 - lift]).interactable(false).show(ctx, |ui| {
            glass().inner_margin(egui::Margin::symmetric(14, 6)).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.horizontal(|ui| {
                    let mono = egui::FontId::monospace(14.0);
                    if self.ap.on {
                        let (on, armed) = self.ap.annunciation();
                        ui.label(RichText::new("AP").font(mono.clone()).strong().color(menu::OK));
                        for m in on {
                            ui.add_space(6.0);
                            ui.label(RichText::new(m).font(mono.clone()).color(menu::OK));
                        }
                        for m in armed {
                            ui.add_space(6.0);
                            ui.label(RichText::new(m).font(mono.clone()).color(DIM));
                        }
                    } else if (off_for * 2.5).fract() < 0.6 {
                        ui.label(RichText::new("AP OFF").font(mono).strong().color(Color32::from_rgb(255, 196, 80)));
                    } else {
                        ui.label(RichText::new("      ").font(mono));
                    }
                });
            });
        });
    }

    /// The systems card at the top left: a rocket's flight figures, or a
    /// fighter's weapons. Nothing (None) for every other aircraft.
    pub(crate) fn systems_card(&self, ctx: &egui::Context, vs: f64, base_up: f64) -> Option<Rect> {
        let p = &self.plane;
        let rocket = p.engines().is_some();
        if !rocket && !self.missiles_ok() {
            return None;
        }
        let amber = Color32::from_rgb(255, 196, 80);
        let row = |ui: &mut egui::Ui, k: &str, v: String, col: Color32| {
            ui.label(RichText::new(k).font(menu::caption()).color(DIM));
            ui.label(RichText::new(v).font(menu::data()).color(col));
            ui.end_row();
        };
        let r = egui::Area::new(egui::Id::unique("systems")).anchor(Align2::LEFT_TOP, [14.0, 14.0]).interactable(false).show(ctx, |ui| {
            glass().show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                if let Some((name, _, _)) = p.engines() {
                    ui.label(menu::section_text(&format!("{}  {}", p.stage_name().unwrap_or(""), name)).color(DIM));
                    egui::Grid::new("rocket").num_columns(2).spacing([14.0, 3.0]).show(ui, |ui| {
                        let apo = p.apogee().map_or(t!("escape", "libération").to_string(), |a| format!("{:.0} km", a.max(0.0) / 1000.0));
                        let high = if base_up < 3000.0 { format!("{:.0} m", base_up.max(0.0)) } else { format!("{:.1} km", base_up / 1000.0) };
                        row(ui, t!("Throttle", "Poussée"), format!("{:.0}%  ·  {:.0} MN", p.throttle * 100.0, p.thrust_now() / 1.0e6), TEXT);
                        row(ui, t!("Climb, speed", "Montée, vitesse"), format!("{:+.0} m/s  ·  {:.0} m/s", vs, p.vel.length()), TEXT);
                        row(ui, t!("Height", "Hauteur"), high, TEXT);
                        row(ui, t!("Apogee", "Apogée"), apo, TEXT);
                        row(ui, t!("Downrange", "Distance"), format!("{:.0} km", p.downrange() / 1000.0), TEXT);
                        if let Some(t) = p.burnout() {
                            row(ui, t!("Burnout", "Fin de combustion"), format!("{}:{:02}", t as u32 / 60, t as u32 % 60), TEXT);
                        }
                        row(ui, t!("Mass, load", "Masse, facteur"), format!("{:.0} t  ·  {:.1} g", p.mass() / 1000.0, p.g_load), TEXT);
                    });
                }
                if self.missiles_ok() {
                    let armed = self.weapons.armed;
                    let power = p.sys.powered();
                    ui.label(menu::section_text(t!("Weapons", "Armement")).color(DIM));
                    egui::Grid::new("weapons").num_columns(2).spacing([14.0, 3.0]).show(ui, |ui| {
                        let arm_col = if !armed { DIM } else if power { menu::OK } else { amber };
                        row(ui, t!("Master arm", "Armement général"), if armed { "ON".into() } else { "OFF".into() }, arm_col);
                        let (mut left, mut right) = (0u8, 0u8);
                        for (i, st) in self.weapons.stations.iter().enumerate() {
                            if self.weapons.loaded(i) {
                                if st.spec.side == 0 { left += 1 } else { right += 1 }
                            }
                        }
                        row(ui, "AIM-9", format!("L {left}  ·  R {right}"), TEXT);
                        let (line, col) = if !armed {
                            (t!("safe", "sécurisé").to_string(), DIM)
                        } else if !power {
                            (t!("no power", "pas d'alimentation").to_string(), amber)
                        } else if let Some(v) = &self.weapons.target_view {
                            if self.weapons.lock >= 1.0 {
                                (t!("lock  {:.1} km", "verrouillé  {:.1} km", v.range / 1000.0), menu::OK)
                            } else {
                                (t!("track  {:.1} km", "poursuite  {:.1} km", v.range / 1000.0), amber)
                            }
                        } else {
                            (t!("search", "recherche").to_string(), DIM)
                        };
                        row(ui, t!("Seeker", "Autodirecteur"), line, col);
                    });
                }
            });
        });
        Some(r.response.rect)
    }
}
