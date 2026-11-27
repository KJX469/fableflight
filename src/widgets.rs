//! widgets for the full screen pages (career, pilot profile, activities):
//! the page frame with its back button and tabs, cards, meters, chips and
//! figures, all in the menu's look (menu.rs).

use crate::menu::{self, ACCENT, DIM, FAINT, TEXT};
use egui::{pos2, vec2, Align2, Color32, Rect, Sense, Stroke};

/// the dark backdrop of a full screen page: the scenery behind, darker to the foot, no colour of its own.
pub fn backdrop(ctx: &egui::Context, id: &str) {
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::unique(("page_bg", id))).order(egui::Order::Background).fixed_pos(screen.min).show(ctx, |ui| {
        let p = ui.painter();
        p.rect_filled(screen, 0.0, Color32::from_rgba_premultiplied(5, 7, 11, 222));
        let mut m = egui::Mesh::default();
        let (top, foot) = (Color32::TRANSPARENT, Color32::from_black_alpha(90));
        m.colored_vertex(screen.left_top(), top);
        m.colored_vertex(screen.right_top(), top);
        m.colored_vertex(screen.right_bottom(), foot);
        m.colored_vertex(screen.left_bottom(), foot);
        m.add_triangle(0, 1, 2);
        m.add_triangle(0, 2, 3);
        p.add(m);
    });
}

/// the page's width and left edge, centred, at most `max` wide.
pub fn columns(screen: Rect, max: f32) -> (f32, f32) {
    let m = if screen.width() < 1100.0 { 20.0 } else { 36.0 };
    let w = (screen.width() - 2.0 * m).min(max);
    (screen.center().x - w * 0.5, w)
}

/// the head of a page: back, the title, the tabs, and whatever goes on the right.
/// returns whether back was pressed and the tab clicked.
pub fn page_head(ui: &mut egui::Ui, title: &str, tabs: &[&str], current: usize, right: impl FnOnce(&mut egui::Ui)) -> (bool, Option<usize>) {
    let mut back = false;
    let mut hit = None;
    ui.horizontal(|ui| {
        ui.set_min_height(46.0);
        if back_button(ui).clicked() {
            back = true;
        }
        ui.add_space(14.0);
        menu::screen_heading(ui, title);
        ui.add_space(26.0);
        if !tabs.is_empty() {
            hit = tab_strip(ui, tabs, current);
        }
        menu::at_right(ui, ("page_right", title), right);
    });
    // a hairline under the head.
    let y = ui.cursor().top() + 6.0;
    ui.painter().hline(ui.max_rect().x_range(), y, Stroke::new(1.0, Color32::from_white_alpha(16)));
    ui.add_space(18.0);
    (back, hit)
}

/// back: a chevron and the word, a box only under the pointer.
pub fn back_button(ui: &mut egui::Ui) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(88.0, 36.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, crate::tr_en("Back")));
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let p = ui.painter();
    if lit > 0.0 {
        p.rect_filled(r, menu::R_WIDGET, Color32::from_white_alpha((12.0 * lit) as u8));
    }
    let col = menu::mix(DIM, Color32::WHITE, lit);
    menu::chevron(p, pos2(r.left() + 18.0, r.center().y), 9.0, 2.0, col);
    p.text(pos2(r.left() + 32.0, r.center().y), Align2::LEFT_CENTER, crate::tr_en("Back"), menu::medium(15.0), col);
    resp
}

/// words on a line with the chosen one underlined in the accent.
pub fn tab_strip(ui: &mut egui::Ui, tabs: &[&str], current: usize) -> Option<usize> {
    let n = tabs.len().max(1);
    let mut hit = match menu::page_turn(ui.ctx()) {
        0 => None,
        d => Some((current as i32 + d).rem_euclid(n as i32) as usize),
    };
    ui.spacing_mut().item_spacing.x = 4.0;
    menu::page_prompt(ui, false);
    ui.add_space(6.0);
    for (i, t) in tabs.iter().enumerate() {
        let on = i == current;
        let font = menu::semibold(15.5);
        let g = ui.painter().layout_no_wrap(t.to_uppercase(), font, TEXT);
        let (r, resp) = ui.allocate_exact_size(vec2(g.size().x + 26.0, 40.0), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Tab, true, on, *t));
        let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
        let p = ui.painter();
        if lit > 0.0 && !on {
            p.rect_filled(r.shrink2(vec2(2.0, 4.0)), menu::R_WIDGET, Color32::from_white_alpha((10.0 * lit) as u8));
        }
        let col = if on { Color32::WHITE } else { menu::mix(DIM, TEXT, lit) };
        p.galley_with_override_text_color(r.center() - g.size() * 0.5, g, col);
        if on {
            p.rect_filled(Rect::from_min_size(pos2(r.left() + 10.0, r.bottom() - 3.0), vec2(r.width() - 20.0, 3.0)), 1.5, ACCENT);
        }
        if resp.clicked() {
            hit = Some(i);
        }
    }
    ui.add_space(6.0);
    menu::page_prompt(ui, true);
    hit
}

/// a section of a page, `h` high (0: as tall as it grows), filling the width.
/// no box round it: a hairline over it and the page's own dark under it, as a
/// game's menus group by type and rules, not a card for everything.
pub fn card<R>(ui: &mut egui::Ui, h: f32, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let top = ui.cursor().top();
    let x = ui.available_rect_before_wrap().x_range();
    ui.painter().hline(x, top, Stroke::new(1.0, Color32::from_white_alpha(26)));
    egui::Frame::new()
        .inner_margin(egui::Margin { left: 0, right: 0, top: 16, bottom: 16 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            if h > 0.0 {
                ui.set_min_height(h - 32.0);
            }
            add(ui)
        })
        .inner
}

/// a section's title: the small capitals, dim, over its rows.
pub fn title(ui: &mut egui::Ui, s: &str) {
    ui.label(menu::section_text(s).color(DIM));
    ui.add_space(12.0);
}

/// a bar filled to `frac` in `col`.
pub fn meter(p: &egui::Painter, r: Rect, frac: f32, col: Color32) {
    p.rect_filled(r, r.height() * 0.5, Color32::from_white_alpha(16));
    let f = frac.clamp(0.0, 1.0);
    if f > 0.0 {
        p.rect_filled(Rect::from_min_size(r.min, vec2((r.width() * f).max(r.height()), r.height())), r.height() * 0.5, col);
    }
}

/// a meter across the width, `h` high, with room under it.
pub fn meter_row(ui: &mut egui::Ui, frac: f32, col: Color32, h: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), h), Sense::hover());
    meter(ui.painter(), r, frac, col);
}

/// a small rounded tag, painted at `at` (left centre): returns its rect.
pub fn chip(p: &egui::Painter, at: egui::Pos2, text: &str, col: Color32) -> Rect {
    // nearly square, as every control: an outline in the colour, no pill.
    let g = p.layout_no_wrap(text.to_string(), menu::semibold(11.5), col);
    let r = Rect::from_min_size(pos2(at.x, at.y - 10.0), vec2(g.size().x + 12.0, 20.0));
    p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, col.gamma_multiply(0.6)), egui::StrokeKind::Inside);
    p.galley(r.center() - g.size() * 0.5, g, col);
    r
}

/// a chip laid out in a ui.
pub fn chip_ui(ui: &mut egui::Ui, text: &str, col: Color32) {
    let w = ui.painter().layout_no_wrap(text.to_string(), menu::semibold(11.5), col).size().x + 14.0;
    let (r, _) = ui.allocate_exact_size(vec2(w, 22.0), Sense::hover());
    chip(ui.painter(), pos2(r.left(), r.center().y), text, col);
}

/// a figure: a big value and a small label under it.
pub fn figure(p: &egui::Painter, at: egui::Pos2, value: &str, label: &str, size: f32, col: Color32) {
    let g = p.layout_no_wrap(value.to_string(), menu::bold(size), col);
    let h = g.size().y;
    p.galley(at, g, col);
    p.text(pos2(at.x + 1.0, at.y + h - 2.0), Align2::LEFT_TOP, label, menu::caption(), DIM);
}

/// a row of figures across the page between two hairlines, no box.
pub fn figures(ui: &mut egui::Ui, items: &[(String, &str)]) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 86.0), Sense::hover());
    let p = ui.painter();
    p.hline(r.x_range(), r.top(), Stroke::new(1.0, Color32::from_white_alpha(26)));
    p.hline(r.x_range(), r.bottom(), Stroke::new(1.0, Color32::from_white_alpha(14)));
    let step = r.width() / items.len().max(1) as f32;
    for (k, (v, l)) in items.iter().enumerate() {
        let left = r.left() + step * k as f32 + if k > 0 { 20.0 } else { 0.0 };
        if k > 0 {
            p.vline(left - 20.0, (r.top() + 22.0)..=(r.bottom() - 22.0), Stroke::new(1.0, Color32::from_white_alpha(14)));
        }
        let g = menu::one_line(p, v, menu::bold(28.0), Color32::WHITE, step - 24.0);
        let h = g.size().y;
        p.galley(pos2(left, r.center().y - h * 0.5 - 8.0), g, Color32::WHITE);
        p.text(pos2(left + 1.0, r.center().y + h * 0.5 - 6.0), Align2::LEFT_TOP, *l, menu::caption(), DIM);
    }
}

/// a line of words left and a value right, across the width.
pub fn line(ui: &mut egui::Ui, left: &str, right: &str, right_col: Color32) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::hover());
    let p = ui.painter();
    let rg = p.layout_no_wrap(right.to_string(), menu::label(), right_col);
    let lg = menu::one_line(p, left, menu::label(), TEXT, r.width() - rg.size().x - 16.0);
    p.galley(pos2(r.left(), r.center().y - lg.size().y * 0.5), lg, TEXT);
    p.galley(pos2(r.right() - rg.size().x, r.center().y - rg.size().y * 0.5), rg, right_col);
}

/// a dim note in a card that is empty.
pub fn empty(ui: &mut egui::Ui, text: &str) {
    ui.add_space(14.0);
    ui.label(egui::RichText::new(text).font(menu::body()).color(FAINT));
}

/// a primary (accent filled) or quiet button `w` wide, `h` high.
pub fn action(ui: &mut egui::Ui, text: &str, primary: bool, enabled: bool, w: f32, h: f32) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(w, h), if enabled { Sense::click() } else { Sense::hover() });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, text));
    let lit = menu::ease(ui.ctx(), resp.id, enabled && (resp.hovered() || resp.has_focus()));
    let p = ui.painter();
    let col = if primary && enabled {
        p.rect_filled(r, menu::R_WIDGET, menu::mix(ACCENT, menu::ACCENT_HI, lit));
        menu::ON_ACCENT
    } else if primary {
        p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, ACCENT.gamma_multiply(0.3)), egui::StrokeKind::Inside);
        ACCENT.gamma_multiply(0.45)
    } else {
        p.rect_filled(r, menu::R_WIDGET, menu::mix(menu::RAISED, menu::RAISED_HI, lit));
        p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, menu::mix(menu::LINE_HI, Color32::from_white_alpha(60), lit)), egui::StrokeKind::Inside);
        if enabled { menu::mix(TEXT, Color32::WHITE, lit) } else { FAINT }
    };
    let font = if primary { menu::bold(16.0) } else { menu::medium(14.5) };
    let g = menu::one_line(p, &if primary { text.to_uppercase() } else { text.to_string() }, font, col, w - 16.0);
    p.galley(r.center() - g.size() * 0.5, g, col);
    resp
}

/// the initials of a name, for an avatar.
pub fn initials(name: &str) -> String {
    let s: String = name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase();
    if s.is_empty() { "P".into() } else { s }
}

/// a round avatar with the initials, and the level ring round it.
pub fn avatar(p: &egui::Painter, c: egui::Pos2, r: f32, name: &str, frac: f32) {
    p.circle_filled(c, r, Color32::from_rgb(28, 36, 50));
    p.circle_stroke(c, r + 4.0, Stroke::new(3.0, Color32::from_white_alpha(22)));
    let f = frac.clamp(0.0, 1.0);
    if f > 0.0 {
        let a0 = -std::f32::consts::FRAC_PI_2;
        let n = (64.0 * f).ceil().max(2.0) as usize;
        let pts: Vec<egui::Pos2> = (0..=n).map(|i| {
            let a = a0 + std::f32::consts::TAU * f * i as f32 / n as f32;
            c + vec2(a.cos(), a.sin()) * (r + 4.0)
        }).collect();
        p.add(egui::Shape::line(pts, Stroke::new(3.0, ACCENT)));
    }
    p.text(c, Align2::CENTER_CENTER, initials(name), menu::bold(r * 0.8), Color32::WHITE);
}
