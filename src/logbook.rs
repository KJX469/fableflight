//! the pilot's screen: the profile (level, experience, figures and records),
//! the logbook (every flight from the flight log, with its best landing) and
//! the achievements. the pause menu's logbook page shows the same table.

use crate::flightlog::{self, Flight, Grade, Outcome};
use crate::menu::{self, ACCENT, BAD, DIM, FAINT, OK, TEXT};
use crate::profile::{self, Totals};
use crate::widgets::{self, card, chip, meter, meter_row};
use crate::{pilot, t, tr_en, Game};
use egui::{pos2, vec2, Align2, Color32, Layout, Rect, RichText, Sense, Stroke};

/// a place as the logbook keeps it.
fn place(s: &Option<String>) -> String {
    s.clone().unwrap_or_else(|| "—".into())
}

/// a landing grade's colour.
fn grade_colour(g: Grade) -> Color32 {
    match g {
        Grade::Butter => ACCENT,
        Grade::Smooth => OK,
        Grade::Firm => TEXT,
        Grade::Hard => Color32::from_rgb(255, 170, 90),
        Grade::VeryHard => BAD,
    }
}

/// the table's columns: where each starts as a share of the width, and
/// whether it is a number (set against its right edge); the last marks the end.
const COLS: [(f32, bool); 9] = [(0.0, false), (0.17, false), (0.34, false), (0.43, false), (0.52, true), (0.62, true), (0.73, false), (0.89, false), (1.0, false)];

fn heads() -> [&'static str; 8] {
    [t!("profile.col_date"), t!("profile.col_aircraft"), t!("profile.col_from"), t!("profile.col_to"), t!("profile.col_time"), t!("profile.col_dist"), tr_en("Best landing"), tr_en("End")]
}

/// one flight's row.
fn flight_row(ui: &mut egui::Ui, f: &Flight, odd: bool) {
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), Sense::hover());
    let p = ui.painter();
    if resp.hovered() {
        p.rect_filled(r, 3.0, Color32::from_white_alpha(10));
    } else if odd {
        p.rect_filled(r, 3.0, Color32::from_white_alpha(4));
    }
    let y = r.center().y;
    let x = |k: usize| r.left() + 12.0 + (r.width() - 24.0) * COLS[k].0;
    let room = |k: usize| (r.width() - 24.0) * (COLS[k + 1].0 - COLS[k].0) - 14.0;
    let cell = |k: usize, text: &str, col: Color32| {
        let g = menu::one_line(p, text, menu::label(), col, room(k));
        let at = if COLS[k].1 { pos2(x(k + 1) - 14.0 - g.size().x, y - g.size().y * 0.5) } else { pos2(x(k), y - g.size().y * 0.5) };
        p.galley(at, g, col);
    };
    cell(0, &profile::flight_date(f), DIM);
    cell(1, &crate::i18n::tr_en_owned(&f.aircraft), Color32::WHITE);
    cell(2, &place(&f.from), TEXT);
    cell(3, &place(&f.to), TEXT);
    cell(4, &profile::hhmm(f.air_time), Color32::WHITE);
    cell(5, &format!("{:.0} nm", f.distance / 1852.0), TEXT);
    match f.best_landing() {
        Some(l) => {
            let col = grade_colour(l.grade);
            let cr = chip(p, pos2(x(6), y), &l.grade.text().to_uppercase(), col);
            p.text(pos2(cr.right() + 8.0, y), Align2::LEFT_CENTER, l.score.to_string(), menu::label(), DIM);
        }
        None => cell(6, "—", FAINT),
    }
    match f.outcome {
        Outcome::Landed => cell(7, t!("log.landed"), TEXT),
        Outcome::Crashed => {
            chip(p, pos2(x(7), y), t!("log.crashed"), BAD);
        }
        Outcome::Unfinished => cell(7, t!("log.unfinished"), FAINT),
    }
    if let Some(l) = f.best_landing() {
        resp.on_hover_text(l.summary());
    } else {
        resp.on_hover_text(t!("profile.row_tip", alt = format!("{:.0}", f.max_alt * 3.28084)));
    }
}

/// the logbook's table: the heads and every flight, latest first.
fn table(ui: &mut egui::Ui, flights: &[Flight], h: f32) {
    let (hr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    let p = ui.painter();
    let x = |k: usize| hr.left() + 12.0 + (hr.width() - 24.0) * COLS[k].0;
    for (k, head) in heads().iter().enumerate() {
        if COLS[k].1 {
            p.text(pos2(x(k + 1) - 14.0, hr.center().y), Align2::RIGHT_CENTER, *head, menu::caption(), FAINT);
        } else {
            p.text(pos2(x(k), hr.center().y), Align2::LEFT_CENTER, *head, menu::caption(), FAINT);
        }
    }
    p.hline(hr.x_range(), hr.bottom() + 2.0, Stroke::new(1.0, Color32::from_white_alpha(16)));
    ui.add_space(6.0);
    if flights.is_empty() {
        ui.add_space(30.0);
        menu::hint(ui, t!("log.empty"));
        return;
    }
    egui::ScrollArea::vertical().id_salt("logbook_rows").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for (k, f) in flights.iter().rev().enumerate() {
            flight_row(ui, f, k % 2 == 1);
        }
    });
}

const TABS: [&str; 3] = ["Overview", "Logbook", "Achievements"];

impl Game {
    /// the pilot's screen (see the module docs).
    pub(crate) fn logbook_screen(&mut self, ctx: &egui::Context) {
        widgets::backdrop(ctx, "pilot");
        let screen = ctx.content_rect();
        let (x0, w) = widgets::columns(screen, 1340.0);
        let mut back = false;
        let tab = self.pilot_tab.min(2);
        egui::Area::new(egui::Id::unique("pilot")).fixed_pos(pos2(x0, 26.0)).constrain(false).show(ctx, |ui| {
            ui.set_width(w);
            let names: Vec<&str> = TABS.iter().map(|t| tr_en(t)).collect();
            let (b, hit) = widgets::page_head(ui, tr_en("Pilot"), &names, tab, |ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut self.profile.name).desired_width(220.0).hint_text(t!("profile.pilot_name")).font(menu::label()).margin(vec2(10.0, 7.0)));
                if r.lost_focus() {
                    self.keep_profile();
                }
            });
            back = b;
            if let Some(i) = hit {
                self.pilot_tab = i;
            }
            let h = (screen.bottom() - ui.cursor().top() - 24.0).max(320.0);
            ui.allocate_ui_with_layout(vec2(w, h), Layout::top_down(egui::Align::Min), |ui| match tab {
                // (scrolls in its own room like the other tabs, so the head never moves)
                0 => {
                    egui::ScrollArea::vertical().id_salt("pilot_overview").max_height(h).auto_shrink([false, false]).show(ui, |ui| self.pilot_overview(ui, w - 12.0, h - 8.0));
                }
                1 => self.pilot_logbook(ui, w, h),
                _ => self.pilot_badges(ui, w, h),
            });
        });
        if back {
            self.menu.screen = menu::Screen::Main;
            self.pad_home = true;
        }
    }

    fn pilot_overview(&mut self, ui: &mut egui::Ui, w: f32, h: f32) {
        let tot: Totals = profile::totals(&self.logbook.flights);
        let p = &self.profile;
        let lvl = pilot::level(p.xp);
        let lw = (w * 0.62).floor();
        let rw = w - lw - 40.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(40.0, 14.0);
            ui.allocate_ui_with_layout(vec2(lw, h), Layout::top_down(egui::Align::Min), |ui| {
                // the pilot, the level and the way to the next.
                let (r, _) = ui.allocate_exact_size(vec2(lw, 190.0), Sense::hover());
                let pa = ui.painter();
                pa.rect_filled(r, menu::R_PANEL, menu::PANEL);
                pa.rect_stroke(r, menu::R_PANEL, Stroke::new(1.0, menu::LINE_HI), egui::StrokeKind::Inside);
                let name = if p.name.trim().is_empty() { tr_en("Pilot").to_string() } else { p.name.clone() };
                widgets::avatar(pa, pos2(r.left() + 100.0, r.center().y), 62.0, &name, lvl.frac());
                let tx = r.left() + 200.0;
                pa.text(pos2(tx, r.top() + 36.0), Align2::LEFT_TOP, &name, menu::bold(40.0), Color32::WHITE);
                pa.text(pos2(tx, r.top() + 90.0), Align2::LEFT_TOP, format!("{} {}  ·  {}", tr_en("Level"), lvl.n, tr_en(pilot::rank(lvl.n))), menu::semibold(18.0), TEXT);
                // the bar, and how far along it right beside it.
                let next = if lvl.span == 0 { tr_en("Highest level").to_string() } else { format!("{} / {} XP", lvl.into, lvl.span) };
                let ng = pa.layout_no_wrap(next, menu::label(), TEXT);
                let by = r.top() + 136.0;
                let bar = Rect::from_min_max(pos2(tx, by - 4.0), pos2(r.right() - 30.0 - ng.size().x - 14.0, by + 4.0));
                meter(pa, bar, lvl.frac(), ACCENT);
                pa.galley(pos2(r.right() - 30.0 - ng.size().x, by - ng.size().y * 0.5), ng, TEXT);
                // the figures.
                widgets::figures(ui, &[
                    (profile::hours(tot.air_s), tr_en("Hours")),
                    (tot.flights.to_string(), tr_en("Flights")),
                    (tot.landings.to_string(), tr_en("Landings")),
                    (p.airports.len().to_string(), tr_en("Airports")),
                    (tot.best.map_or("—".into(), |b| b.to_string()), tr_en("Best landing")),
                ]);
                // the latest achievements.
                card(ui, 0.0, |ui| {
                    ui.horizontal(|ui| {
                        widgets::title(ui, tr_en("ACHIEVEMENTS"));
                        menu::at_right(ui, "badges_n", |ui| {
                            ui.label(RichText::new(format!("{} / {}", p.badges.len(), pilot::BADGES.len())).font(menu::label()).color(DIM));
                        });
                    });
                    let mut got: Vec<(&String, &u64)> = p.badges.iter().collect();
                    got.sort_by(|a, b| b.1.cmp(a.1));
                    if got.is_empty() {
                        widgets::empty(ui, tr_en("Fly to earn them: a first landing, a butter touchdown, new airports, the sound barrier."));
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                        for (id, _) in got.into_iter().take(8) {
                            if let Some(b) = pilot::badge(id) {
                                let (r, resp) = ui.allocate_exact_size(vec2(96.0, 92.0), Sense::hover());
                                let pp = ui.painter();
                                pilot::medal(pp, pos2(r.center().x, r.top() + 34.0), 20.0, true, 1.0);
                                let g = menu::one_line(pp, tr_en(b.name), menu::caption(), TEXT, 94.0);
                                pp.galley(pos2(r.center().x - g.size().x * 0.5, r.top() + 66.0), g, TEXT);
                                resp.on_hover_text(tr_en(b.about));
                            }
                        }
                    });
                });
            });
            ui.allocate_ui_with_layout(vec2(rw, h), Layout::top_down(egui::Align::Min), |ui| {
                card(ui, 0.0, |ui| {
                    widgets::title(ui, tr_en("RECORDS"));
                    widgets::line(ui, tr_en("Longest flight"), &profile::hhmm(tot.longest_s), Color32::WHITE);
                    widgets::line(ui, tr_en("Distance flown"), &format!("{:.0} nm", tot.dist_nm), Color32::WHITE);
                    widgets::line(ui, tr_en("Highest"), &format!("{:.0} ft", p.max_ft.max(tot.highest_ft)), Color32::WHITE);
                    widgets::line(ui, tr_en("Fastest"), &if p.max_mach > 0.0 { format!("Mach {:.2}", p.max_mach) } else { "—".into() }, Color32::WHITE);
                    widgets::line(ui, tr_en("Butter landings"), &p.butters.to_string(), ACCENT);
                    widgets::line(ui, tr_en("Night landings"), &p.night_landings.to_string(), Color32::WHITE);
                    widgets::line(ui, tr_en("Types flown"), &p.types.len().to_string(), Color32::WHITE);
                    widgets::line(ui, tr_en("Crashes"), &tot.crashes.to_string(), if tot.crashes > 0 { BAD } else { Color32::WHITE });
                });
                card(ui, 0.0, |ui| {
                    widgets::title(ui, tr_en("RECENT EXPERIENCE"));
                    if p.awards.is_empty() {
                        widgets::empty(ui, tr_en("Every take off, landing, flight, lesson and job earns experience."));
                    }
                    for a in p.awards.iter().rev().take(9) {
                        widgets::line(ui, &a.what, &format!("+{} XP", a.xp), ACCENT);
                    }
                });
            });
        });
    }

    fn pilot_logbook(&mut self, ui: &mut egui::Ui, w: f32, h: f32) {
        let tot: Totals = profile::totals(&self.logbook.flights);
        widgets::figures(ui, &[
            (profile::hours(tot.air_s), t!("profile.flight_time")),
            (tot.flights.to_string(), t!("profile.flights")),
            (tot.landings.to_string(), t!("profile.landings")),
            (format!("{:.0} nm", tot.dist_nm), t!("profile.distance")),
            (profile::hhmm(tot.longest_s), t!("profile.longest")),
            (format!("{:.0} ft", tot.highest_ft), t!("profile.highest")),
        ]);
        ui.add_space(14.0);
        let rest = (h - 100.0).max(220.0);
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = 16.0;
            let left_w = (w * 0.7).floor();
            ui.allocate_ui_with_layout(vec2(left_w, rest), Layout::top_down(egui::Align::Min), |ui| {
                card(ui, rest, |ui| {
                    widgets::title(ui, t!("profile.flights_title"));
                    table(ui, &self.logbook.flights, rest - 90.0);
                });
            });
            let right_w = w - left_w - 16.0;
            ui.allocate_ui_with_layout(vec2(right_w, rest), Layout::top_down(egui::Align::Min), |ui| {
                card(ui, rest, |ui| {
                    widgets::title(ui, t!("profile.by_aircraft"));
                    let most = tot.by_type.first().map(|b| b.1).unwrap_or(1.0).max(1.0);
                    for (name, secs, n) in tot.by_type.iter().take(8) {
                        widgets::line(ui, &crate::i18n::tr_en_owned(name), &t!("profile.short", h = profile::hours(*secs), n = n), DIM);
                        meter_row(ui, (*secs / most) as f32, ACCENT, 4.0);
                        ui.add_space(4.0);
                    }
                    if tot.by_type.is_empty() {
                        widgets::empty(ui, "—");
                    }
                    ui.add_space(14.0);
                    widgets::title(ui, t!("profile.airports"));
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
                        for (a, n) in tot.airports.iter().take(36) {
                            let g = ui.painter().layout_no_wrap(a.to_string(), menu::label(), Color32::WHITE);
                            let ng = ui.painter().layout_no_wrap(n.to_string(), menu::caption(), DIM);
                            let (r, _) = ui.allocate_exact_size(vec2(g.size().x + ng.size().x + 30.0, 30.0), Sense::hover());
                            let p = ui.painter();
                            p.rect_filled(r, menu::R_WIDGET, menu::RAISED);
                            p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, menu::LINE_HI), egui::StrokeKind::Inside);
                            p.galley(pos2(r.left() + 10.0, r.center().y - g.size().y * 0.5), g.clone(), Color32::WHITE);
                            p.galley(pos2(r.right() - 10.0 - ng.size().x, r.center().y - ng.size().y * 0.5), ng, DIM);
                        }
                    });
                    if tot.airports.is_empty() {
                        widgets::empty(ui, "—");
                    }
                    ui.add_space(14.0);
                    let since = profile::date(self.profile.since);
                    menu::hint(ui, &t!("profile.since", date = since.get(..10).unwrap_or(&since)));
                });
            });
        });
    }

    fn pilot_badges(&mut self, ui: &mut egui::Ui, w: f32, h: f32) {
        let p = &self.profile;
        egui::ScrollArea::vertical().id_salt("pilot_badges").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
            let cols = ((w + 14.0) / 314.0).floor().max(2.0) as usize;
            let cw = (w - 14.0 * (cols as f32 - 1.0)) / cols as f32;
            for row in pilot::BADGES.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    for b in row {
                        let got = p.badges.get(b.id);
                        let (r, _) = ui.allocate_exact_size(vec2(cw, 84.0), Sense::hover());
                        let pa = ui.painter();
                        // an earned one framed, the rest flat on the page.
                        if got.is_some() {
                            pa.rect_stroke(r, menu::R_PANEL, Stroke::new(1.0, ACCENT.gamma_multiply(0.45)), egui::StrokeKind::Inside);
                        } else {
                            pa.hline(r.x_range(), r.bottom() - 0.5, Stroke::new(1.0, Color32::from_white_alpha(14)));
                        }
                        pilot::medal(pa, pos2(r.left() + 40.0, r.center().y), 21.0, got.is_some(), 1.0);
                        let tw = r.width() - 90.0;
                        let ng = menu::one_line(pa, tr_en(b.name), menu::semibold(16.0), if got.is_some() { Color32::WHITE } else { TEXT }, tw);
                        pa.galley(pos2(r.left() + 76.0, r.top() + 14.0), ng, Color32::WHITE);
                        let ag = menu::one_line(pa, tr_en(b.about), menu::caption(), DIM, tw);
                        pa.galley(pos2(r.left() + 76.0, r.top() + 38.0), ag, DIM);
                        let foot = match got {
                            Some(u) => profile::date(*u).get(..10).unwrap_or("").to_string(),
                            None => format!("+{} XP", b.xp),
                        };
                        pa.text(pos2(r.left() + 76.0, r.top() + 64.0), Align2::LEFT_CENTER, foot, menu::caption(), if got.is_some() { ACCENT } else { FAINT });
                    }
                });
                ui.add_space(14.0);
            }
        });
    }

    /// the pause menu's logbook page: the totals and the same table.
    pub(crate) fn logbook_compact(&self, ui: &mut egui::Ui) {
        let tot = self.logbook.totals();
        let lvl = pilot::level(self.profile.xp);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 28.0;
            for (v, l) in [(format!("{} {}", tr_en("Level"), lvl.n), tr_en(pilot::rank(lvl.n))), (flightlog::hours_text(tot.hours * 3600.0), t!("log.hours_in_the_air")), (tot.flights.to_string(), t!("log.flights")), (tot.landings.to_string(), t!("log.landings")), (tot.best.map_or("—".into(), |b| format!("{b}/100")), t!("log.best_landing"))] {
                ui.vertical(|ui| {
                    ui.label(RichText::new(v).font(menu::bold(22.0)).color(Color32::WHITE));
                    ui.label(RichText::new(l).font(menu::caption()).color(DIM));
                });
            }
        });
        if let Some((name, h)) = self.logbook.hours_by_aircraft().first() {
            ui.add_space(6.0);
            ui.label(RichText::new(format!("{}: {}  ·  {}", t!("log.by_aircraft"), crate::i18n::tr_en_owned(name), flightlog::hours_text(h * 3600.0))).font(menu::caption()).color(DIM));
        }
        ui.add_space(12.0);
        table(ui, &self.logbook.flights, (ui.available_height() - 40.0).max(200.0));
    }
}
