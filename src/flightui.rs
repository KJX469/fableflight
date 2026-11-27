//! the hud over the flying, its notices, alerts and the flight panel's pieces.
use crate::menu::{self, Icon, Page, DIM, TEXT};
use crate::{padui, t, Game};
use egui::{pos2, vec2, Align2, Color32, Rect, RichText, Sense, Stroke};

// the flight panel's pieces.

/// a quiet word that acts as a button, for what is rarely wanted (abandon).
pub fn text_link(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let r = ui.add(egui::Button::new(RichText::new(text).font(menu::caption()).color(DIM)).frame(false));
    if r.hovered() {
        ui.painter().hline(r.rect.x_range(), r.rect.bottom() - 2.0, Stroke::new(1.0, DIM));
    }
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// a small button as wide as its words, lit when chosen (the menus' look).
pub fn chip(ui: &mut egui::Ui, text: &str, _mono: bool, on: bool) -> egui::Response {
    menu::choice_sized(ui, text, on, 0.0, 28.0)
}

/// a button `w` by `h`, lit when on (the menus' look).
pub fn key(ui: &mut egui::Ui, text: &str, on: bool, w: f32, h: f32) -> egui::Response {
    menu::choice_sized(ui, text, on, w, h)
}

// the flight card: one frosted card in the middle, its pages laid out in tiles.

/// the card's width.
pub const CARD_W: f32 = 660.0;
/// the gap between tiles.
pub const GAP: f32 = 8.0;

/// a row of `n` equal columns `h` tall, `add` fills column `i`. no box round
/// each: a hairline over the row (but the first) and between the columns.
pub fn cells(ui: &mut egui::Ui, n: usize, h: f32, mut add: impl FnMut(&mut egui::Ui, usize)) {
    let w = ui.available_width();
    let first = ui.min_rect().height() < 8.0;
    let (row, resp) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    if !first {
        ui.painter().hline(row.x_range(), row.top() - GAP * 0.5, Stroke::new(1.0, white(0.08)));
    }
    let cw = (w - GAP * 2.0 * (n as f32 - 1.0)) / n as f32;
    for i in 0..n {
        let r = Rect::from_min_size(pos2(row.left() + i as f32 * (cw + GAP * 2.0), row.top()), vec2(cw, h));
        if i > 0 {
            ui.painter().vline(r.left() - GAP, (row.top() + 8.0)..=(row.bottom() - 8.0), Stroke::new(1.0, white(0.08)));
        }
        let inner = Rect::from_min_max(pos2(r.left(), r.top() + 8.0), pos2(r.right(), r.bottom() - 8.0));
        let mut c = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)).id_salt(resp.id.with(i)));
        c.set_clip_rect(r.intersect(ui.clip_rect()));
        c.spacing_mut().item_spacing = vec2(6.0, 4.0);
        add(&mut c, i);
    }
    ui.add_space(GAP - ui.spacing().item_spacing.y);
}

/// one tile across the card, as tall as what it holds.
pub fn wide_tile<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    if ui.min_rect().height() >= 8.0 {
        let y = ui.cursor().top() - GAP * 0.5 + ui.spacing().item_spacing.y * 0.5;
        ui.painter().hline(ui.max_rect().x_range(), y, Stroke::new(1.0, white(0.08)));
    }
    let r = egui::Frame::new().inner_margin(egui::Margin::symmetric(0, 8)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
        add(ui)
    });
    ui.add_space(GAP - ui.spacing().item_spacing.y);
    r.inner
}

/// a part's name, dim, in the label face.
pub fn tag(ui: &mut egui::Ui, s: &str) {
    ui.label(RichText::new(s).font(menu::medium(14.0)).color(DIM));
}

/// a figure and its unit, in the instrument list's value size.
pub fn big(ui: &mut egui::Ui, value: &str, unit: &str, col: Color32) {
    let w = ui.available_width();
    ui.allocate_ui_with_layout(vec2(w, 24.0), egui::Layout::left_to_right(egui::Align::Max), |ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label(RichText::new(value).font(menu::semibold(17.0)).color(col));
        if !unit.is_empty() {
            ui.label(RichText::new(unit).font(menu::caption()).color(SOFT));
        }
    });
}

/// a short line in grey, cut at the tile's edge.
pub fn note(ui: &mut egui::Ui, s: &str) {
    ui.add(egui::Label::new(RichText::new(s).font(menu::caption()).color(SOFT)).truncate());
}

/// a thin bar `frac` full.
pub fn meter(ui: &mut egui::Ui, frac: f32, col: Color32) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
    ui.painter().rect_filled(r, 2, white(0.1));
    ui.painter().rect_filled(Rect::from_min_size(r.min, vec2(r.width() * frac.clamp(0.0, 1.0), r.height())), 2, col);
}

// the flight page's instrument list: rows with the name, the value and a detail in three columns.

/// the instrument list's columns and row height.
const NAME_W: f32 = 112.0;
const VALUE_W: f32 = 124.0;
const ROW_H: f32 = 32.0;

/// one instrument row `w` wide: the name, its value, a dim detail, and the room left at the
/// right end (`right_w`, for a button or a bar) given back.
pub fn inst_row(ui: &mut egui::Ui, w: f32, name: &str, value: &str, col: Color32, detail: &str, right_w: f32) -> Rect {
    let (r, _) = ui.allocate_exact_size(vec2(w, ROW_H), Sense::hover());
    let p = ui.painter();
    let y = r.center().y;
    let end = r.right() - right_w;
    let ng = menu::one_line(p, name, menu::label(), DIM, NAME_W - 8.0);
    p.galley(pos2(r.left(), y - ng.size().y * 0.5), ng, DIM);
    let vg = menu::one_line(p, value, menu::semibold(15.0), col, (end - r.left() - NAME_W).min(if detail.is_empty() { f32::INFINITY } else { VALUE_W - 8.0 }));
    p.galley(pos2(r.left() + NAME_W, y - vg.size().y * 0.5), vg, col);
    let dx = r.left() + NAME_W + VALUE_W;
    if !detail.is_empty() && end - dx > 24.0 {
        let dg = menu::one_line(p, detail, menu::caption(), SOFT, end - dx - 8.0);
        p.galley(pos2(dx, y - dg.size().y * 0.5), dg, SOFT);
    }
    Rect::from_min_max(pos2(end, r.top()), r.max)
}

/// a thin bar `frac` full, centred in `r`.
pub fn inst_meter(p: &egui::Painter, r: Rect, frac: f32, col: Color32) {
    let bar = Rect::from_center_size(r.center(), vec2(r.width(), 4.0));
    p.rect_filled(bar, 2, white(0.1));
    p.rect_filled(Rect::from_min_size(bar.min, vec2(bar.width() * frac.clamp(0.0, 1.0), 4.0)), 2, col);
}

/// a hairline between groups of rows.
pub fn inst_rule(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    let y = ui.cursor().top();
    ui.painter().hline(ui.max_rect().x_range(), y, Stroke::new(1.0, white(0.08)));
    ui.add_space(4.0);
}

/// a pick from a list: a code, a name and a figure, lit when it is the one in use.
pub fn pick_row(ui: &mut egui::Ui, code: &str, name: &str, right: &str, on: bool) -> egui::Response {
    let w = ui.available_width();
    let (r, resp) = ui.allocate_exact_size(vec2(w, 32.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, name));
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let p = ui.painter();
    if on {
        p.rect_filled(r, R_IN, LIT.gamma_multiply(0.16));
    } else if lit > 0.0 {
        p.rect_filled(r, R_IN, white(0.07 * lit));
    }
    let col = if on { LIT } else { Color32::WHITE };
    let cg = p.layout_no_wrap(code.to_string(), num_s(), col);
    let rg = p.layout_no_wrap(right.to_string(), menu::caption(), SOFT);
    let left = r.left() + 10.0;
    let name_x = left + cg.size().x.max(58.0) + 8.0;
    let room = (r.right() - 10.0 - rg.size().x - 8.0 - name_x).max(0.0);
    p.galley(pos2(left, r.center().y - cg.size().y * 0.5), cg, col);
    p.galley(pos2(r.right() - 10.0 - rg.size().x, r.center().y - rg.size().y * 0.5), rg, SOFT);
    let mut job = egui::text::LayoutJob::simple_singleline(name.to_string(), menu::caption(), TEXT);
    job.wrap = egui::text::TextWrapping::truncate_at_width(room);
    let ng = p.layout_job(job);
    p.galley(pos2(name_x, r.center().y - ng.size().y * 0.5), ng, TEXT);
    resp
}

/// the card's header: the page's name, the pages as icons, and a close button. the name shows
/// the page under the pointer (or the pad's focus), dimmed, so every icon reads as its page.
/// gives back the page asked for, and whether close was pressed.
pub fn card_header(ui: &mut egui::Ui, page: Page) -> (Option<Page>, bool, egui::Response) {
    let mut to = None;
    let mut close = false;
    let mut close_resp = None;
    let mut over = None;
    ui.horizontal(|ui| {
        let (title, _) = ui.allocate_exact_size(vec2(200.0, 32.0), Sense::hover());
        let slot = ui.painter().add(egui::Shape::Noop);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let x = icon_key(ui, Some(Icon::Close), "", false, t!("flight.close_tab")).on_hover_text(t!("flight.close_tab"));
            if x.clicked() {
                close = true;
            }
            close_resp = Some(x);
            ui.add_space(12.0);
            for p in Page::INFO.into_iter().rev() {
                let r = icon_key(ui, Some(page_icon(p)), "", p == page, p.label());
                if r.hovered() || r.has_focus() {
                    over = Some(p);
                }
                if r.clicked() && p != page {
                    to = Some(p);
                }
            }
        });
        let (name, col) = match over.filter(|p| *p != page) {
            Some(p) => (p.label(), DIM),
            None => (page.label(), Color32::WHITE),
        };
        let g = ui.painter().layout_no_wrap(name.to_string(), menu::page_title(), col);
        ui.painter().set(slot, egui::Shape::galley(pos2(title.left(), title.center().y - g.size().y * 0.5), g, col));
    });
    (to, close, close_resp.expect("close button"))
}

/// a square icon button with its name on hover (the autopilot panel's hide and page).
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, hint: &str) -> egui::Response {
    icon_key(ui, Some(icon), "", false, hint).on_hover_text(hint)
}

/// a square icon button, yellow when on.
fn icon_key(ui: &mut egui::Ui, icon: Option<Icon>, text: &str, on: bool, hint: &str) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, hint));
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let p = ui.painter();
    if on {
        p.rect_filled(r, R_IN, LIT);
    } else {
        p.rect_filled(r, R_IN, white(0.04 + 0.1 * lit));
    }
    let col = if on { menu::ON_ACCENT } else { menu::mix(SOFT, Color32::WHITE, lit) };
    match icon {
        Some(i) => menu::icon(p, i, r.center(), 18.0, col),
        None => {
            p.text(r.center(), Align2::CENTER_CENTER, text, menu::medium(18.0), col);
        }
    }
    resp
}

/// the icon each page goes by.
pub fn page_icon(p: Page) -> Icon {
    match p {
        Page::Places => Icon::GoTo,
        Page::Radio => Icon::Radio,
        Page::Autopilot => Icon::Autopilot,
        Page::Weather => Icon::Weather,
        Page::Checklist => Icon::Checklist,
        Page::Atc => Icon::Atc,
        Page::Ground => Icon::Ground,
        _ => Icon::Flight,
    }
}

/// what the hud's buttons asked for.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Bar {
    Page(Page),
    /// the autopilot panel at the bottom.
    Autopilot,
    Map,
    Camera,
}

const NOTE_FADE: f64 = 1.0;

impl Game {
    /// does what the hud's buttons asked for.
    pub(crate) fn toolbar_act(&mut self, b: Bar) {
        match b {
            Bar::Page(p) => {
                if self.info_open && self.info_page == p {
                    self.info_open = false;
                } else {
                    self.info_open = true;
                    self.info_page = p;
                    self.pad_home = true;
                }
            }
            Bar::Autopilot => self.ap_bar_on = !self.ap_bar_on,
            Bar::Map => self.minimap.visible = !self.minimap.visible,
            Bar::Camera => self.next_view(),
        }
    }

    /// the notices at the top centre.
    pub(crate) fn notices(&mut self, ctx: &egui::Context) {
        let mut notes: Vec<(f32, Vec<(String, egui::FontId, Color32)>)> = Vec::new();
        if self.real < self.status_until && !self.status.is_empty() {
            let a = ((self.status_until - self.real) / NOTE_FADE).clamp(0.0, 1.0) as f32;
            notes.push((a, vec![(self.status.chars().take(90).collect(), menu::label(), Color32::WHITE)]));
        }
        if !notes.is_empty() {
            // under the heading strip.
            let y = HEADING_TOP + HEADING_H + 10.0;
            egui::Area::new(egui::Id::unique("notices")).anchor(Align2::CENTER_TOP, [0.0, y]).interactable(false).show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;
                ui.vertical_centered(|ui| {
                    for (a, lines) in &notes {
                        // a scope so each notice takes its own room.
                        ui.scope(|ui| {
                            ui.multiply_opacity(*a);
                            panel(ui, glass().inner_margin(egui::Margin::symmetric(16, 8)), |ui| {
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
    }

    /// the autopilot's modes as an annunciator row: green what it flies, grey what is armed.
    /// "ap off" flashes amber for a few seconds after it lets go.
    fn ap_strip(&self, ui: &mut egui::Ui) {
        let off_for = self.real - self.ap_off_t;
        let flash = !self.ap.on && off_for < 4.0;
        if !self.ap.on && !flash {
            return;
        }
        panel(ui, glass().inner_margin(egui::Margin::symmetric(16, 8)), |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let green = Color32::from_rgb(64, 200, 96);
                if self.ap.on {
                    let (on, armed) = self.ap.annunciation();
                    ui.label(RichText::new("AP").font(menu::semibold(14.0)).color(green));
                    for m in on {
                        ui.label(RichText::new(m).font(num_s()).color(green));
                    }
                    for m in armed {
                        ui.label(RichText::new(m).font(num_s()).color(SOFT));
                    }
                    if self.sim_rate() > 1.0 {
                        ui.label(RichText::new(format!("{}×", self.warp)).font(num_s()).color(Color32::WHITE));
                    }
                } else if (off_for * 2.5).fract() < 0.6 {
                    ui.label(RichText::new("AP OFF").font(menu::semibold(14.0)).color(AMBER));
                } else {
                    ui.label(RichText::new("AP OFF").font(menu::semibold(14.0)).color(Color32::TRANSPARENT));
                }
            });
        });
    }

    /// rocket figures or weapons, none for other aircraft.
    pub(crate) fn systems_card(&self, ctx: &egui::Context, vs: f64, base_up: f64) -> Option<Rect> {
        let p = &self.plane;
        let rocket = p.engines().is_some();
        if !rocket && !self.missiles_ok() {
            return None;
        }
        let amber = Color32::from_rgb(255, 196, 80);
        let row = |ui: &mut egui::Ui, k: &str, v: String, col: Color32| {
            ui.label(RichText::new(k).font(menu::caption()).color(DIM));
            ui.label(RichText::new(v).font(num_s()).color(col));
            ui.end_row();
        };
        let r = egui::Area::new(egui::Id::unique("systems")).anchor(Align2::LEFT_TOP, [14.0, 12.0]).interactable(false).show(ctx, |ui| {
            panel(ui, glass(), |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                if let Some((name, _, _)) = p.engines() {
                    ui.label(tag_text(&format!("{}  {}", p.stage_name().unwrap_or(""), name)));
                    egui::Grid::new("rocket").num_columns(2).spacing([14.0, 3.0]).show(ui, |ui| {
                        let apo = p.apogee().map_or(t!("flight.escape").to_string(), |a| format!("{:.0} km", a.max(0.0) / 1000.0));
                        let high = if base_up < 3000.0 { format!("{:.0} m", base_up.max(0.0)) } else { format!("{:.1} km", base_up / 1000.0) };
                        row(ui, t!("flight.throttle"), format!("{:.0}%  ·  {:.0} MN", p.throttle * 100.0, p.thrust_now() / 1.0e6), TEXT);
                        row(ui, t!("flight.climb_speed"), format!("{:+.0} m/s  ·  {:.0} m/s", vs, p.vel.length()), TEXT);
                        row(ui, t!("flight.height"), high, TEXT);
                        row(ui, t!("flight.apogee"), apo, TEXT);
                        row(ui, t!("flight.downrange"), format!("{:.0} km", p.downrange() / 1000.0), TEXT);
                        if let Some(t) = p.burnout() {
                            row(ui, t!("flight.burnout"), format!("{}:{:02}", t as u32 / 60, t as u32 % 60), TEXT);
                        }
                        row(ui, t!("flight.mass_load"), format!("{:.0} t  ·  {:.1} g", p.mass() / 1000.0, p.g_load), TEXT);
                    });
                }
                if self.missiles_ok() {
                    let armed = self.weapons.armed;
                    let power = p.sys.powered();
                    ui.label(tag_text(t!("flight.weapons")));
                    egui::Grid::new("weapons").num_columns(2).spacing([14.0, 3.0]).show(ui, |ui| {
                        let arm_col = if !armed { DIM } else if power { menu::OK } else { amber };
                        row(ui, t!("flight.master_arm"), if armed { "ON".into() } else { "OFF".into() }, arm_col);
                        let (mut left, mut right) = (0u8, 0u8);
                        for (i, st) in self.weapons.stations.iter().enumerate() {
                            if self.weapons.loaded(i) {
                                if st.spec.side == 0 { left += 1 } else { right += 1 }
                            }
                        }
                        row(ui, "AIM-9", format!("L {left}  ·  R {right}"), TEXT);
                        let (line, col) = if !armed {
                            (t!("flight.safe").to_string(), DIM)
                        } else if !power {
                            (t!("flight.no_power").to_string(), amber)
                        } else if let Some(v) = &self.weapons.target_view {
                            if self.weapons.lock >= 1.0 {
                                (t!("flight.lock_km", v.range / 1000.0), menu::OK)
                            } else {
                                (t!("flight.track_km", v.range / 1000.0), amber)
                            }
                        } else {
                            (t!("flight.search").to_string(), DIM)
                        };
                        row(ui, t!("flight.seeker"), line, col);
                    });
                }
            });
        });
        Some(r.response.rect)
    }
}

// the hud sits at the edges in the menus' dark panels and leaves the middle to the world.

/// an overlay's rect from the last frame, if it was up.
pub fn shown(ctx: &egui::Context, name: &str, order: egui::Order) -> Option<Rect> {
    let id = egui::Id::unique(name);
    ctx.memory(|m| if m.areas().visible_last_frame(&egui::LayerId::new(order, id)) { m.area_rect(id) } else { None })
}

/// how far up from the screen's foot the next overlay at the bottom centre goes,
/// over the autopilot panel, the alerts, the replay bar and the crash keys.
pub fn bottom_stack(ctx: &egui::Context, skip: &str) -> f32 {
    let foot = ctx.content_rect().bottom();
    let mut lift: f32 = 16.0;
    for (name, order) in [("ap_bar", egui::Order::Middle), ("hud_alerts", egui::Order::Middle), ("replay_bar", egui::Order::Foreground), ("crash", egui::Order::Middle)] {
        if name == skip {
            continue;
        }
        if let Some(r) = shown(ctx, name, order) {
            lift = lift.max(foot - r.top() + 10.0);
        }
    }
    lift
}

/// the top of the right column: under the flight menu hint while it shows.
pub fn right_top(ctx: &egui::Context) -> f32 {
    shown(ctx, "hud_hint", egui::Order::Background).map_or(HEADING_TOP, |r| r.bottom() + 10.0)
}

/// the heading strip's height, and its gap from the top edge (layout below it uses both).
pub const HEADING_H: f32 = 46.0;
pub const HEADING_TOP: f32 = 10.0;
/// the tapes' sizes.
const TAPE_H: f32 = 352.0;
const SPEED_W: f32 = 124.0;
const ALT_W: f32 = 118.0;
/// a tape's head and foot rows, and its readout box.
const LINE_H: f32 = 28.0;
const READOUT_H: f32 = 34.0;
/// corner radii: square, like the menus.
pub(crate) const R_HUD: u8 = 0;
pub(crate) const R_IN: u8 = 0;
/// inset inside a panel.
const PAD: f32 = 12.0;

pub const AMBER: Color32 = Color32::from_rgb(255, 196, 80);
/// the hud's accent: the game's yellow.
pub const LIT: Color32 = menu::ACCENT;
/// soft white for secondary text.
pub const SOFT: Color32 = Color32::from_rgba_premultiplied(184, 184, 184, 184);
/// the readouts' boxes.
const BOX: Color32 = Color32::from_rgba_premultiplied(10, 12, 16, 225);
/// white at a share.
fn white(a: f32) -> Color32 {
    Color32::from_white_alpha((255.0 * a) as u8)
}

/// the hud's faces.
fn num_l() -> egui::FontId {
    menu::semibold(24.0)
}
fn num_m() -> egui::FontId {
    menu::medium(17.0)
}
fn num_s() -> egui::FontId {
    menu::medium(13.5)
}
fn tag_font() -> egui::FontId {
    menu::medium(11.5)
}
fn tag_text(s: &str) -> RichText {
    RichText::new(s.to_uppercase()).font(tag_font()).color(white(0.62)).extra_letter_spacing(1.1)
}
/// spaced capitals at a point.
pub(crate) fn paint_tag(p: &egui::Painter, at: egui::Pos2, align: Align2, s: &str) {
    let col = white(0.62);
    let mut job = egui::text::LayoutJob::default();
    job.append(&s.to_uppercase(), 0.0, egui::TextFormat { font_id: tag_font(), color: col, extra_letter_spacing: 1.1, ..Default::default() });
    let g = p.layout_job(job);
    let size = g.size() - vec2(1.1, 0.0);
    p.galley(align.anchor_size(at, size).min, g, col);
}

/// what the hud shows.
pub struct HudIn {
    pub hdg: f64,
    /// airspeed (m/s), heights (m), climb (m/s).
    pub ias: f64,
    pub alt: f64,
    pub agl: f64,
    pub vs: f64,
}

// the panels.

/// a rectangle's outline with corner radii `cr` (the minimap clips to it).
pub(crate) fn rounded(r: Rect, cr: egui::CornerRadius) -> Vec<egui::Pos2> {
    let mut pts = Vec::new();
    let corners = [
        (pos2(r.right() - cr.ne as f32, r.top() + cr.ne as f32), cr.ne as f32, -90.0f32),
        (pos2(r.right() - cr.se as f32, r.bottom() - cr.se as f32), cr.se as f32, 0.0),
        (pos2(r.left() + cr.sw as f32, r.bottom() - cr.sw as f32), cr.sw as f32, 90.0),
        (pos2(r.left() + cr.nw as f32, r.top() + cr.nw as f32), cr.nw as f32, 180.0),
    ];
    for (c, rad, a0) in corners {
        if rad <= 0.5 {
            pts.push(c);
            continue;
        }
        for k in 0..=8 {
            let a = (a0 + 90.0 * k as f32 / 8.0).to_radians();
            pts.push(c + vec2(a.cos(), a.sin()) * rad);
        }
    }
    pts
}

/// the panels' fill and edge, the start screen's tiles.
const PANEL_FILL: Color32 = Color32::from_rgba_premultiplied(12, 16, 22, 226);
const PANEL_EDGE: Color32 = Color32::from_rgba_premultiplied(26, 26, 26, 26);

/// a dark panel in `r` with a hairline edge.
fn panel_shapes(r: Rect) -> Vec<egui::Shape> {
    vec![
        egui::Shape::rect_filled(r, 0, PANEL_FILL),
        egui::Shape::rect_stroke(r, 0, Stroke::new(1.0, PANEL_EDGE), egui::StrokeKind::Inside),
    ]
}

/// paints a panel in `r`.
fn panel_rect(p: &egui::Painter, r: Rect) {
    for s in panel_shapes(r) {
        p.add(s);
    }
}

/// a frame for the hud's panels.
pub fn glass() -> egui::Frame {
    egui::Frame::new().inner_margin(PAD)
}

/// shows `frame` on a panel.
pub fn panel<R>(ui: &mut egui::Ui, frame: egui::Frame, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    let slot = ui.painter().add(egui::Shape::Noop);
    let resp = frame.show(ui, add);
    ui.painter().set(slot, egui::Shape::Vec(panel_shapes(resp.response.rect)));
    resp
}

/// a tape's readout: the figure in one size in a box with a notch at the scale, like the heading's.
/// `right` puts the notch on the right edge (the altitude tape).
fn readout(p: &egui::Painter, r: Rect, text: &str, col: Color32, edge: Color32, right: bool) {
    let y = r.center().y;
    let (x, tip) = if right { (r.right(), r.right() + 7.0) } else { (r.left(), r.left() - 7.0) };
    let notch = vec![pos2(x, y - 7.0), pos2(tip, y), pos2(x, y + 7.0)];
    p.rect_filled(r, 0, BOX);
    p.add(egui::Shape::convex_polygon(notch.clone(), BOX, Stroke::NONE));
    p.rect_stroke(r, 0, Stroke::new(1.0, edge), egui::StrokeKind::Inside);
    p.add(egui::Shape::line(notch, Stroke::new(1.0, edge)));
    // the notch opens into the box.
    p.line_segment([pos2(x + if right { -0.5 } else { 0.5 }, y - 6.0), pos2(x + if right { -0.5 } else { 0.5 }, y + 6.0)], Stroke::new(1.5, BOX));
    p.text(pos2(r.right() - 10.0, y), Align2::RIGHT_CENTER, text, num_l(), col);
}

/// a tag and its figure on one line inside a tape's head or foot.
fn tape_line(p: &egui::Painter, r: Rect, tag: &str, value: &str, col: Color32) {
    paint_tag(p, pos2(r.left() + 12.0, r.center().y), Align2::LEFT_CENTER, tag);
    p.text(pos2(r.right() - 12.0, r.center().y), Align2::RIGHT_CENTER, value, num_s(), col);
}

/// thousands set apart with a thin space.
fn grouped(v: f32) -> String {
    let n = v.round() as i64;
    let s = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push('\u{2009}');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

impl Game {
    /// the whole hud.
    pub(crate) fn hud(&mut self, ctx: &egui::Context, h: &HudIn) {
        let screen = ctx.content_rect();
        // the tapes' centre line, a little above the middle.
        let cy = screen.center().y - 36.0;
        self.heading_strip(ctx, h.hdg as f32);
        self.menu_hint(ctx);
        self.speed_tape(ctx, h, cy);
        // the altitude tape steps down below a tutorial or job card over it (small or scaled screens).
        let over = ["hud_hint", "mission_card", "job_card"].iter().filter_map(|n| shown(ctx, n, if *n == "hud_hint" { egui::Order::Background } else { egui::Order::Middle })).map(|r| r.bottom()).fold(f32::NEG_INFINITY, f32::max);
        let cy_alt = cy.max(over + 12.0 + TAPE_H * 0.5).min(screen.bottom() - TAPE_H * 0.5 - 16.0);
        self.altitude_tape(ctx, h, cy_alt);
        if self.path_marker {
            self.flight_path_marker(ctx);
        }
        self.minimap.top = cy + TAPE_H * 0.5 + 12.0;
        self.minimap.bottom = 14.0;
    }

    /// the flight menu's key, top right for the first seconds of a flight only.
    fn menu_hint(&self, ctx: &egui::Context) {
        let id = egui::Id::unique("hud since");
        let (seen, since) = ctx.data(|d| d.get_temp::<(f64, f64)>(id)).unwrap_or((-10.0, self.real));
        let since = if self.real - seen > 1.0 { self.real } else { since };
        ctx.data_mut(|d| d.insert_temp(id, (self.real, since)));
        let a = (1.0 - ((self.real - since - 9.0) / 1.0).clamp(0.0, 1.0)) as f32;
        if a <= 0.0 {
            return;
        }
        let key = if padui::active() { crate::padmap::spec_of("flight_panel") } else { "Tab".to_string() };
        if key.is_empty() {
            return;
        }
        egui::Area::new(egui::Id::unique("hud_hint")).anchor(Align2::RIGHT_TOP, [-14.0, HEADING_TOP]).interactable(false).order(egui::Order::Background).show(ctx, |ui| {
            ui.multiply_opacity(a);
            panel(ui, glass().inner_margin(egui::Margin::symmetric(12, 8)), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if padui::active() {
                        padui::glyphs(ui, &key, 22.0);
                    } else {
                        menu::key_cap(ui, &key);
                    }
                    ui.label(RichText::new(crate::tr_en("Flight menu")).font(menu::label()).color(TEXT));
                });
            });
        });
        ctx.request_repaint();
    }

    fn heading_strip(&self, ctx: &egui::Context, hdg: f32) {
        let screen = ctx.content_rect();
        let w = 540.0;
        let at = pos2(screen.center().x - w * 0.5, screen.top() + HEADING_TOP);
        egui::Area::new(egui::Id::unique("hud_heading")).fixed_pos(at).interactable(false).order(egui::Order::Background).show(ctx, |ui| {
            let (r, _) = ui.allocate_exact_size(vec2(w, HEADING_H), Sense::hover());
            let strip = Rect::from_min_size(r.min, vec2(w, HEADING_H));
            let p = ui.painter();
            panel_rect(p, strip);
            let clip = p.with_clip_rect(strip.shrink(1.0));
            let pl = &self.plane;
            let (e, n, _) = crate::geo::enu(pl.pos);
            let trk = pl.vel.dot(e).atan2(pl.vel.dot(n)).to_degrees().rem_euclid(360.0) as f32;
            let moving = pl.vel.length() > 3.0;
            // a scale of ticks along the foot, figures every 10°, the cardinal points in yellow.
            const PPD: f32 = 6.0;
            let cx = strip.center().x;
            let base = strip.bottom() - 1.0;
            let lo = ((hdg - 50.0) / 5.0).floor() as i32 * 5;
            for d in (lo..=lo + 100).step_by(5) {
                let x = cx + (d as f32 - hdg) * PPD;
                let edge = ((x - strip.left()).min(strip.right() - x) / 60.0).clamp(0.0, 1.0);
                if edge <= 0.0 {
                    continue;
                }
                let dd = d.rem_euclid(360);
                let big = dd % 10 == 0;
                clip.line_segment([pos2(x, base), pos2(x, base - if big { 9.0 } else { 5.0 })], Stroke::new(1.2, white(if big { 0.8 } else { 0.45 } * edge)));
                // nothing under the readout.
                if !big || (x - cx).abs() < 40.0 {
                    continue;
                }
                let (label, col) = match dd {
                    0 => ("N".to_string(), LIT),
                    90 => ("E".to_string(), LIT),
                    180 => ("S".to_string(), LIT),
                    270 => (t!("flight.w").to_string(), LIT),
                    _ => (format!("{:02}", dd / 10), Color32::WHITE),
                };
                clip.text(pos2(x, base - 20.0), Align2::CENTER_CENTER, label, num_s(), col.gamma_multiply(0.85 * edge));
            }
            // the track: a hollow yellow caret on the scale.
            if moving {
                let mut dt = (trk - hdg + 540.0).rem_euclid(360.0) - 180.0;
                dt = dt.clamp(-40.0, 40.0);
                let x = cx + dt * PPD;
                let tri = vec![pos2(x, base - 10.0), pos2(x - 5.0, base - 1.0), pos2(x + 5.0, base - 1.0)];
                clip.add(egui::Shape::closed_line(tri, Stroke::new(1.5, LIT)));
            }
            // the readout, boxed in the middle, and the index under it.
            let rb = Rect::from_center_size(pos2(cx, strip.top() + 17.0), vec2(66.0, 26.0));
            p.rect_filled(rb, 0, BOX);
            p.rect_stroke(rb, 0, Stroke::new(1.0, LIT), egui::StrokeKind::Inside);
            p.text(rb.center(), Align2::CENTER_CENTER, format!("{:03.0}°", hdg.rem_euclid(360.0).round() % 360.0), menu::semibold(17.0), Color32::WHITE);
        });
    }

    fn speed_tape(&self, ctx: &egui::Context, h: &HudIn, cy: f32) {
        let screen = ctx.content_rect();
        let p = &self.plane;
        let kt = |ms: f64| (ms * 1.943_84) as f32;
        // a quarter second low pass so the speed glides.
        let ias = {
            let raw = kt(h.ias).max(0.0);
            let id = egui::Id::unique("hud ias smooth");
            let dt = ctx.input(|i| i.stable_dt).clamp(0.0, 0.1);
            let prev = ctx.data(|d| d.get_temp::<f32>(id)).unwrap_or(raw);
            let v = if (raw - prev).abs() > 40.0 { raw } else { prev + (raw - prev) * (1.0 - (-dt / 0.22).exp()) };
            ctx.data_mut(|d| d.insert_temp(id, v));
            v
        };
        let panel = Rect::from_min_size(pos2(screen.left(), cy - TAPE_H * 0.5), vec2(SPEED_W, TAPE_H));
        egui::Area::new(egui::Id::unique("hud_speed")).fixed_pos(panel.min).interactable(false).order(egui::Order::Background).show(ctx, |ui| {
            ui.allocate_exact_size(panel.size() + vec2(8.0, 0.0), Sense::hover());
            let pt = ui.painter();
            panel_rect(pt, panel);
            // ground speed in the head, the throttle in the foot.
            let head = Rect::from_min_size(panel.min, vec2(panel.width(), LINE_H));
            let foot = Rect::from_min_max(pos2(panel.left(), panel.bottom() - LINE_H), panel.max);
            let thr = (p.throttle as f32).clamp(0.0, 1.0);
            tape_line(pt, head, "G/S", &format!("{:.0}", kt(p.vel.length())), Color32::WHITE);
            tape_line(pt, foot, t!("flight.thr"), &format!("{:.0}%", thr * 100.0), Color32::WHITE);
            // figures fade towards the ends.
            let tape = Rect::from_min_max(pos2(panel.left(), head.bottom()), pos2(panel.right() - 36.0, foot.top()));
            let clip = pt.with_clip_rect(tape);
            let c = pos2(tape.center().x, cy);
            const PPK: f32 = 6.0;
            let y_of = |v: f32| c.y - (v - ias) * PPK;
            let fade = |y: f32| ((y - tape.top()).min(tape.bottom() - y) / 36.0).clamp(0.0, 1.0);
            // the aircraft's speed limits along the edge.
            if p.spec.rocket().is_none() {
                let (vs_now, vs_clean, vs_full) = (kt(p.stall_ias_at(p.flaps)), kt(p.stall_ias_at(0.0)), kt(p.stall_ias_at(1.0)));
                let vne = kt(p.spec.vne);
                let band = |a: f32, b: f32, x: f32, w: f32, col: Color32| {
                    let (ya, yb) = (y_of(a.max(0.0)), y_of(b));
                    if ya > tape.top() && yb < tape.bottom() {
                        clip.rect_filled(Rect::from_min_max(pos2(x, yb.max(tape.top())), pos2(x + w, ya.min(tape.bottom()))), 0.0, col);
                    }
                };
                band(vs_clean, vne * 0.9, tape.left(), 5.0, Color32::from_rgb(64, 200, 96));
                band(vne * 0.9, vne, tape.left(), 5.0, AMBER);
                band(vne, vne + 500.0, tape.left(), 5.0, menu::BAD);
                if p.spec.flaps.len() > 1 {
                    if let Some(vfe) = p.spec.vfe.iter().copied().reduce(f64::max) {
                        band(vs_full, kt(vfe), tape.left() + 5.0, 3.0, white(0.9));
                    }
                }
                // red hatching below the stall.
                let (ya, yb) = (y_of(0.0).min(tape.bottom()), y_of(vs_now).max(tape.top()));
                if ya > yb {
                    let hp = clip.with_clip_rect(Rect::from_min_max(pos2(tape.left(), yb), pos2(tape.left() + 7.0, ya)));
                    let mut y = yb - 8.0;
                    while y < ya + 8.0 {
                        hp.line_segment([pos2(tape.left(), y + 7.0), pos2(tape.left() + 7.0, y)], Stroke::new(2.5, menu::BAD));
                        y += 6.0;
                    }
                }
            }
            // the scale, nothing under the readout.
            let lo = ((ias - 40.0) / 5.0).floor() as i32 * 5;
            for v in (lo.max(0)..=lo + 80).step_by(5) {
                let y = y_of(v as f32);
                let a = fade(y);
                if a <= 0.0 || (y - c.y).abs() < 30.0 {
                    continue;
                }
                let big = v % 10 == 0;
                clip.line_segment([pos2(tape.left() + 10.0, y), pos2(tape.left() + if big { 18.0 } else { 14.0 }, y)], Stroke::new(1.3, white(if big { 0.85 } else { 0.5 } * a)));
                if big {
                    clip.text(pos2(tape.left() + 26.0, y), Align2::LEFT_CENTER, format!("{v}"), num_m(), white(a));
                }
            }
            // the readout.
            let warn = p.stalled || p.overspeed() > 0.0 || (p.flaps > 0.25 && p.ias() > p.vfe());
            let col = if p.stalled { menu::BAD } else if warn { AMBER } else { Color32::WHITE };
            let rb = Rect::from_min_max(pos2(tape.left() + 12.0, c.y - READOUT_H * 0.5), pos2(tape.right(), c.y + READOUT_H * 0.5));
            readout(pt, rb, &format!("{:.0}", ias), col, if warn { col } else { LIT }, false);
            // the throttle, a bar between the head and the foot.
            let x = panel.right() - 20.0;
            let track = Rect::from_min_max(pos2(x - 3.0, head.bottom() + 8.0), pos2(x + 3.0, foot.top() - 8.0));
            pt.rect_filled(track, 3, white(0.14));
            pt.rect_filled(Rect::from_min_max(pos2(track.left(), track.bottom() - track.height() * thr), track.max), 3, if p.running { LIT } else { menu::BAD });
        });
    }

    fn altitude_tape(&self, ctx: &egui::Context, h: &HudIn, cy: f32) {
        let screen = ctx.content_rect();
        let ft = |m: f64| (m * 3.280_84) as f32;
        let alt = ft(h.alt);
        let vs_fpm = (h.vs * 196.85) as f32;
        let agl = ft(h.agl.max(0.0));
        let w = ALT_W;
        let panel = Rect::from_min_size(pos2(screen.right() - w, cy - TAPE_H * 0.5), vec2(w, TAPE_H));
        egui::Area::new(egui::Id::unique("hud_alt")).fixed_pos(panel.min - vec2(8.0, 0.0)).interactable(false).order(egui::Order::Background).show(ctx, |ui| {
            ui.allocate_exact_size(panel.size() + vec2(8.0, 0.0), Sense::hover());
            let pt = ui.painter().clone();
            panel_rect(&pt, panel);
            // climb in the head, height above the ground in the foot when low.
            let head = Rect::from_min_size(panel.min, vec2(panel.width(), LINE_H));
            let foot = Rect::from_min_max(pos2(panel.left(), panel.bottom() - LINE_H), panel.max);
            let vs_col = if vs_fpm < -1500.0 && agl < 2500.0 { menu::BAD } else { Color32::WHITE };
            tape_line(&pt, head, "V/S", &format!("{:+.0}", (vs_fpm / 10.0).round() * 10.0), vs_col);
            if agl < 2500.0 && !self.plane.on_ground {
                tape_line(&pt, foot, "AGL", &format!("{:.0}", agl), if agl < 300.0 { AMBER } else { Color32::WHITE });
            }
            let tape = Rect::from_min_max(pos2(panel.left(), head.bottom()), pos2(panel.right(), foot.top()));
            let c = pos2(tape.center().x, cy);
            let clip = pt.with_clip_rect(tape);
            const PPF: f32 = 0.6;
            let y_of = |v: f32| c.y - (v - alt) * PPF;
            let fade = |y: f32| ((y - tape.top()).min(tape.bottom() - y) / 36.0).clamp(0.0, 1.0);
            let lo = ((alt - 340.0) / 50.0).floor() as i32 * 50;
            for v in (lo..=lo + 700).step_by(50) {
                let y = y_of(v as f32);
                let a = fade(y);
                if a <= 0.0 || (v < 0 && alt >= -50.0) {
                    continue;
                }
                let big = v % 100 == 0;
                clip.line_segment([pos2(tape.right() - 4.0, y), pos2(tape.right() - if big { 14.0 } else { 9.0 }, y)], Stroke::new(1.3, white(if big { 0.85 } else { 0.5 } * a)));
                if (y - c.y).abs() < 30.0 {
                    continue;
                }
                if big {
                    clip.text(pos2(tape.right() - 20.0, y), Align2::RIGHT_CENTER, grouped(v as f32), num_m(), white(a));
                }
            }
            // the readout, to the nearest ten feet.
            let rb = Rect::from_min_max(pos2(tape.left() + 12.0, c.y - READOUT_H * 0.5), pos2(tape.right() - 18.0, c.y + READOUT_H * 0.5));
            readout(&pt, rb, &grouped((alt / 10.0).round() * 10.0), Color32::WHITE, LIT, true);
        });
    }

    /// the flight path marker: a ring where the aircraft is actually going
    /// (its velocity, not where the nose points). Off unless set in Settings.
    fn flight_path_marker(&self, ctx: &egui::Context) {
        let Some((vp, cam)) = self.live_view else { return };
        let p = &self.plane;
        if p.vel.length() < 15.0 || p.on_ground {
            return;
        }
        let target = p.pos + p.vel.normalize() * 2000.0;
        let c = vp * (target - cam).as_vec3().extend(1.0);
        if c.w <= 0.0 {
            return;
        }
        let screen = ctx.content_rect();
        let at = pos2(screen.left() + (c.x / c.w * 0.5 + 0.5) * screen.width(), screen.top() + (0.5 - c.y / c.w * 0.5) * screen.height());
        if !screen.shrink(40.0).contains(at) {
            return;
        }
        let painter = ctx.layer_painter(egui::LayerId::background());
        // a ring with a short wing each side; a dark edge keeps it legible on bright sky.
        for (w, col) in [(3.0, Color32::from_black_alpha(90)), (1.5, LIT)] {
            let st = Stroke::new(w, col);
            painter.circle_stroke(at, 6.0, st);
            painter.line_segment([at + vec2(-14.0, 0.0), at + vec2(-6.0, 0.0)], st);
            painter.line_segment([at + vec2(6.0, 0.0), at + vec2(14.0, 0.0)], st);
        }
    }

    /// the approach line, alerts and damage, bottom centre.
    pub(crate) fn alerts(&self, ctx: &egui::Context, warn: &[(&str, Color32)], report: &[String]) {
        // the panel at the bottom shows the modes itself while it is up.
        let ap = !self.ap_bar_on && (self.ap.on || self.real - self.ap_off_t < 4.0);
        if warn.is_empty() && report.is_empty() && !ap {
            return;
        }
        // above the autopilot panel while it is up.
        let lift = shown(ctx, "ap_bar", egui::Order::Middle).map_or(16.0, |r| ctx.content_rect().bottom() - r.top() + 10.0);
        egui::Area::new(egui::Id::unique("hud_alerts")).anchor(Align2::CENTER_BOTTOM, [0.0, -lift]).interactable(false).show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                if ap {
                    self.ap_strip(ui);
                }
                if !warn.is_empty() {
                    ui.horizontal(|ui| {
                        for (w, col) in warn {
                            // a lit plate.
                            egui::Frame::new().fill(col.gamma_multiply(0.92)).stroke(Stroke::new(1.0, *col)).inner_margin(egui::Margin::symmetric(16, 8)).show(ui, |ui| {
                                ui.label(RichText::new(w.to_uppercase()).font(menu::semibold(14.0)).color(Color32::WHITE).extra_letter_spacing(1.0));
                            });
                        }
                    });
                }
                if !report.is_empty() {
                    panel(ui, glass(), |ui| {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                        ui.label(tag_text(t!("flight.damage")).color(AMBER));
                        // at most six lines, then how many more.
                        for l in report.iter().take(6) {
                            ui.label(RichText::new(l.as_str()).font(num_s()).color(Color32::WHITE));
                        }
                        if report.len() > 6 {
                            ui.label(RichText::new(format!("+{}", report.len() - 6)).font(num_s()).color(SOFT));
                        }
                    });
                }
            });
        });
    }
}

// the flight menu: a ring of icons around the middle of the view.

/// the flight menu's state.
#[derive(Default)]
pub struct Radial {
    pub open: bool,
    /// when it opened (s), for its entrance and hold to pick.
    pub since: f64,
    /// the item under the pointer or the pad's focus.
    pub hover: Option<usize>,
    /// the item the gamepad points at (d-pad round the ring, or the right stick).
    pub pad: Option<usize>,
    /// A pressed: pick the pad's item on the next frame.
    pub pick: bool,
}

/// how many items the flight menu has.
pub const RADIAL_N: usize = 10;

/// ring radii: the hub, the ring's inside and outside.
const HUB_R: f32 = 58.0;
const RING_IN: f32 = 70.0;
const RING_OUT: f32 = 156.0;

/// an annulus sector from `a0` to `a1` (radians) as a triangle strip.
fn sector(c: egui::Pos2, r0: f32, r1: f32, a0: f32, a1: f32, col: Color32) -> egui::Mesh {
    let mut m = egui::Mesh::default();
    let n = 24;
    for i in 0..=n {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        let d = vec2(a.cos(), a.sin());
        for r in [r0, r1] {
            m.colored_vertex(c + d * r, col);
        }
    }
    for i in 0..n as u32 {
        let k = i * 2;
        m.add_triangle(k, k + 1, k + 3);
        m.add_triangle(k, k + 3, k + 2);
    }
    m
}

/// the outline of a sector.
fn sector_edge(c: egui::Pos2, r0: f32, r1: f32, a0: f32, a1: f32) -> Vec<egui::Pos2> {
    let n = 24;
    let mut v: Vec<egui::Pos2> = (0..=n).map(|i| { let a = a0 + (a1 - a0) * i as f32 / n as f32; c + vec2(a.cos(), a.sin()) * r1 }).collect();
    v.extend((0..=n).rev().map(|i| { let a = a0 + (a1 - a0) * i as f32 / n as f32; c + vec2(a.cos(), a.sin()) * r0 }));
    v
}

impl Game {
    /// opens or closes the flight menu, for tab and the pad's flight menu binding.
    pub(crate) fn flight_menu_key(&mut self) {
        if self.radial.open {
            self.radial.open = false;
        } else if self.info_open {
            self.info_open = false;
        } else {
            self.radial = Radial { open: true, since: self.real, ..Default::default() };
            self.pad_home = true;
            // the menu takes the mouse: a look drag in progress stops here.
            self.drag = false;
        }
    }

    /// tab let go: held long enough with an item pointed at, that item is picked.
    pub(crate) fn flight_menu_release(&mut self) -> Option<Bar> {
        if self.radial.open && self.real - self.radial.since > 0.3 {
            if let Some(k) = self.radial.hover {
                self.radial.open = false;
                return Some(self.radial_items()[k].2);
            }
        }
        None
    }

    fn radial_items(&self) -> [(Icon, &'static str, Bar, bool); RADIAL_N] {
        let page = |p: Page| self.info_open && self.info_page == p;
        [
            (Icon::Flight, t!("flight.flight"), Bar::Page(Page::Flight), page(Page::Flight)),
            (Icon::GoTo, t!("flight.go_to"), Bar::Page(Page::Places), page(Page::Places)),
            (Icon::Radio, t!("flight.radio"), Bar::Page(Page::Radio), page(Page::Radio)),
            (Icon::Autopilot, t!("flight.autopilot"), Bar::Autopilot, self.ap_bar_on),
            (Icon::Weather, t!("flight.weather"), Bar::Page(Page::Weather), page(Page::Weather)),
            (Icon::Map, t!("flight.map"), Bar::Map, self.minimap.visible),
            (Icon::Checklist, crate::tr_en("Checklist"), Bar::Page(Page::Checklist), page(Page::Checklist)),
            (Icon::Atc, crate::tr_en("ATC"), Bar::Page(Page::Atc), page(Page::Atc)),
            (Icon::Ground, crate::tr_en("Ground"), Bar::Page(Page::Ground), page(Page::Ground)),
            (Icon::Camera, match self.cam_mode { 1 => "Cockpit", crate::settings::TAIL_VIEW => t!("flight.fixed_rear"), _ => t!("flight.chase") }, Bar::Camera, false),
        ]
    }

    /// the flight menu, when open; a pick goes into `bar`.
    pub(crate) fn radial_menu(&mut self, ctx: &egui::Context, bar: &mut Option<Bar>) {
        if !self.radial.open {
            return;
        }
        let items = self.radial_items();
        let n = items.len();
        let screen = ctx.content_rect();
        let c = screen.center();
        // a short entrance: it grows and fades in.
        let t = ((self.real - self.radial.since) / 0.16).clamp(0.0, 1.0) as f32;
        let k = 1.0 - (1.0 - t).powi(3);
        if t < 1.0 {
            ctx.request_repaint();
        }
        let scale = 0.86 + 0.14 * k;
        let (hub, r0, r1) = (HUB_R * scale, RING_IN * scale, RING_OUT * scale);
        let step = std::f32::consts::TAU / n as f32;
        // item 0 sits at the top, the rest clockwise.
        let mid = |i: usize| -std::f32::consts::FRAC_PI_2 + step * i as f32;
        let gap = 0.035;
        let mut pick: Option<usize> = None;
        let mut close = false;
        egui::Area::new(egui::Id::unique("flight_menu")).order(egui::Order::Foreground).fixed_pos(screen.min).show(ctx, |ui| {
            ui.multiply_opacity(k);
            // the backdrop takes every click; the world stays visible.
            let back = ui.interact(screen, ui.scope_id().with("radial_back"), Sense::click());
            let p = ui.painter().clone();
            p.rect_filled(screen, 0.0, Color32::from_black_alpha(70));
            // which item the pointer points at, from its angle round the hub.
            let mut hover = None;
            if let Some(pos) = ctx.pointer_latest_pos() {
                let d = pos - c;
                if d.length() > hub && d.length() < r1 + 90.0 {
                    let a = (d.y.atan2(d.x) + std::f32::consts::FRAC_PI_2).rem_euclid(std::f32::consts::TAU);
                    hover = Some(((a + step * 0.5) / step) as usize % n);
                }
            }
            // one clickable spot per icon. The gamepad picks its item
            // directly (Radial::pad), not through egui's focus.
            for (i, (_, label, _, _)) in items.iter().enumerate() {
                let at = c + vec2(mid(i).cos(), mid(i).sin()) * (r0 + r1) * 0.5;
                let r = ui.interact(Rect::from_center_size(at, vec2(44.0, 44.0)), ui.scope_id().with(("radial", i)), Sense::click());
                r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, *label));
                if r.clicked() {
                    pick = Some(i);
                }
            }
            let hover = if crate::padui::active() { self.radial.pad.or(hover) } else { hover };
            self.radial.hover = hover;
            if std::mem::take(&mut self.radial.pick) {
                match hover {
                    Some(i) => pick = Some(i),
                    None => close = true,
                }
            }
            if back.clicked() {
                match hover {
                    Some(i) => pick = Some(i),
                    None => close = true,
                }
            }
            // the ring: a dark panel sector per item, the hovered one yellow.
            for (i, (ic, _, _, on)) in items.iter().enumerate() {
                let (a0, a1) = (mid(i) - step * 0.5 + gap, mid(i) + step * 0.5 - gap);
                let lit = menu::ease(ctx, ui.scope_id().with(("radial_lit", i)), hover == Some(i));
                let grow = 6.0 * lit;
                p.add(sector(c, r0, r1 + grow, a0, a1, menu::mix(PANEL_FILL, LIT, lit)));
                p.add(egui::Shape::closed_line(sector_edge(c, r0, r1 + grow, a0, a1), Stroke::new(1.0, menu::mix(PANEL_EDGE, LIT, lit))));
                let at = c + vec2(mid(i).cos(), mid(i).sin()) * ((r0 + r1 + grow) * 0.5);
                menu::icon(&p, *ic, at, 26.0 + 4.0 * lit, menu::mix(Color32::WHITE, menu::ON_ACCENT, lit));
                // what is open or on: a small yellow dot by the outer edge.
                if *on {
                    p.circle_filled(c + vec2(mid(i).cos(), mid(i).sin()) * (r1 + grow - 11.0), 3.0, menu::mix(LIT, menu::ON_ACCENT, lit));
                }
            }
            // the hub: the pointed item's name, else what this is.
            p.circle_filled(c, hub, PANEL_FILL);
            p.circle_stroke(c, hub, Stroke::new(1.0, PANEL_EDGE));
            match hover {
                Some(i) => {
                    p.text(c, Align2::CENTER_CENTER, items[i].1, menu::semibold(16.0), Color32::WHITE);
                }
                None => paint_tag(&p, c, Align2::CENTER_CENTER, t!("flight.flight")),
            }
        });
        if let Some(i) = pick {
            *bar = Some(items[i].2);
            self.radial.open = false;
        } else if close {
            self.radial.open = false;
        }
    }
}
