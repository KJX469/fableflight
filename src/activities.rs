//! activities: the tutorials, the checkrides, the landing challenges and the
//! missions in one place, as cards with what each is, where, in what, and how
//! it went. shown full screen from the main menu and in the pause menu.

use crate::menu::{self, ACCENT, DIM, FAINT, OK, TEXT};
use crate::missions::{self, Cat, Mission};
use crate::widgets::{self, chip};
use crate::{aircraft, challenge, tr_en, Game};
use egui::{pos2, vec2, Align2, Color32, Rect, Sense, Stroke};

/// what was picked to fly.
pub enum Pick {
    Mission(&'static Mission),
    Challenge(usize),
}

/// free practice only: the career's lessons and checkrides are flown from its training page.
pub const TABS: [&str; 3] = ["Tutorials", "Landing challenges", "Missions"];

/// one card: the title, the words, where and in what, how it went, and fly.
#[allow(clippy::too_many_arguments)]
fn card(ui: &mut egui::Ui, w: f32, n: usize, title: &str, about: &str, foot: &str, status: Option<(String, Color32)>, col: Color32) -> bool {
    let (r, resp) = ui.allocate_exact_size(vec2(w, 118.0), Sense::click());
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered());
    let p = ui.painter();
    // a row of the list, a hairline over it; lit under the pointer.
    p.hline(r.x_range(), r.top(), Stroke::new(1.0, Color32::from_white_alpha(22)));
    if lit > 0.0 {
        p.rect_filled(r, menu::R_PANEL, Color32::from_white_alpha((8.0 * lit) as u8));
    }
    // the number as the list's index: quiet, in the display face, its kind's colour under the pointer.
    let nr = Rect::from_min_size(r.min + vec2(8.0, 12.0), vec2(40.0, 34.0));
    p.text(nr.left_center(), Align2::LEFT_CENTER, format!("{n:02}"), menu::bold(24.0), menu::mix(FAINT, col, lit));
    let tx = r.left() + 64.0;
    let tw = r.width() - 64.0 - 130.0;
    let tg = menu::one_line(p, title, menu::semibold(17.0), Color32::WHITE, tw);
    p.galley(pos2(tx, r.top() + 15.0), tg, Color32::WHITE);
    let mut job = egui::text::LayoutJob::simple(about.to_string(), menu::caption(), TEXT, tw);
    job.wrap.max_rows = 2;
    job.wrap.overflow_character = Some('…');
    let ag = p.layout_job(job);
    p.galley(pos2(tx, r.top() + 42.0), ag, TEXT);
    let fg = menu::one_line(p, foot, menu::caption(), DIM, tw);
    p.galley(pos2(tx, r.bottom() - 26.0), fg, DIM);
    if let Some((s, c)) = status {
        let g = p.layout_no_wrap(s.clone(), menu::semibold(11.5), c);
        chip(p, pos2(r.right() - 16.0 - g.size().x - 14.0, r.top() + 26.0), &s, c);
    }
    // the fly button, always at the right.
    let br = Rect::from_min_size(pos2(r.right() - 112.0, r.bottom() - 50.0), vec2(96.0, 34.0));
    let mut b = ui.new_child(egui::UiBuilder::new().max_rect(br));
    let fly = widgets::action(&mut b, tr_en("Fly"), lit > 0.5, true, 96.0, 34.0).clicked();
    fly || resp.double_clicked()
}

impl Game {
    /// the cards of one tab, in a grid `w` wide: what was picked.
    pub(crate) fn activities_grid(&self, ui: &mut egui::Ui, tab: usize, w: f32) -> Option<Pick> {
        let mut pick = None;
        let cols = if w > 980.0 { 2 } else { 1 };
        let cw = (w - 14.0 * (cols as f32 - 1.0)) / cols as f32;
        // (the tab names the kind; the cards say the rest)
        ui.add_space(8.0);
        let mut cards: Vec<(String, String, String, Option<(String, Color32)>, Color32, Pick)> = Vec::new();
        if tab == 1 {
            for (i, c) in challenge::ALL.iter().enumerate() {
                let plane = aircraft::by_key(c.aircraft).map_or("", |s| s.name);
                let status = self.challenge_records.top(c.id).map(|b| (format!("{} {b}", tr_en("BEST")), if b >= 90 { ACCENT } else { OK }));
                cards.push((c.name().to_string(), c.about().to_string(), format!("{} {}  ·  {plane}  ·  {}", c.airport, c.runway, c.conditions()), status, Color32::from_rgb(110, 170, 255), Pick::Challenge(i)));
            }
        } else {
            let cat = if tab == 0 { Cat::Tutorial } else { Cat::Mission };
            for m in missions::ALL.iter().filter(|m| m.cat == cat) {
                let plane = aircraft::by_key(m.aircraft).map_or(m.aircraft, |s| s.name);
                let best = self.profile.best.get(m.id).copied();
                let status = best.map(|_| (tr_en("DONE").to_string(), OK));
                let col = if cat == Cat::Tutorial { ACCENT } else { Color32::from_rgb(178, 140, 255) };
                let title = tr_en(m.title).trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ' ').to_string();
                cards.push((title, tr_en(m.brief).to_string(), format!("{}  ·  {plane}", m.airport), status, col, Pick::Mission(m)));
            }
        }
        let mut cards = cards.into_iter().enumerate().peekable();
        while cards.peek().is_some() {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 14.0;
                for _ in 0..cols {
                    let Some((k, (title, about, foot, status, col, p))) = cards.next() else { break };
                    if card(ui, cw, k + 1, &title, &about, &foot, status, col) {
                        pick = Some(p);
                    }
                }
            });
            ui.add_space(12.0);
        }
        pick
    }

    /// flies what was picked.
    pub(crate) fn fly_pick(&mut self, p: Pick) {
        match p {
            Pick::Mission(m) => {
                self.leave_challenge();
                self.start_mission(m);
            }
            Pick::Challenge(i) => {
                self.end_mission();
                self.start_challenge(i);
            }
        }
    }

    /// the main menu's activities screen.
    pub(crate) fn activities_screen(&mut self, ctx: &egui::Context) {
        widgets::backdrop(ctx, "activities");
        let screen = ctx.content_rect();
        let (x0, w) = widgets::columns(screen, 1240.0);
        let mut back = false;
        let mut pick = None;
        let tab = self.activity_tab.min(TABS.len() - 1);
        egui::Area::new(egui::Id::unique("activities")).fixed_pos(pos2(x0, 26.0)).constrain(false).show(ctx, |ui| {
            ui.set_width(w);
            let names: Vec<&str> = TABS.iter().map(|t| tr_en(t)).collect();
            let done = self.profile.best.len() + self.challenge_records.best.len();
            let (b, hit) = widgets::page_head(ui, tr_en("Activities"), &names, tab, |ui| {
                ui.label(egui::RichText::new(format!("{done} {}", tr_en("flown"))).font(menu::label()).color(DIM));
            });
            back = b;
            if let Some(i) = hit {
                self.activity_tab = i;
            }
            let h = (screen.bottom() - ui.cursor().top() - 24.0).max(300.0);
            ui.allocate_ui_with_layout(vec2(w, h), egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical().id_salt("activities_cards").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
                    pick = self.activities_grid(ui, tab, w - 16.0);
                });
            });
        });
        if back {
            self.menu.screen = menu::Screen::Main;
            self.pad_home = true;
        }
        if let Some(p) = pick {
            self.fly_pick(p);
        }
    }
}
