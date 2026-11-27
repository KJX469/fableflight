//! The launcher's look, the game's own (src/menu.rs): the Barlow faces, the
//! dark panels and the aviation-yellow accent, with a few widgets drawn to match.
use egui::{Color32, CornerRadius, FontFamily, FontId, Rect, RichText, Sense, Stroke, Vec2, pos2, vec2};

pub const ACCENT: Color32 = Color32::from_rgb(245, 197, 24);
pub const ACCENT_HI: Color32 = Color32::from_rgb(255, 214, 72);
pub const ON_ACCENT: Color32 = Color32::from_rgb(26, 20, 2);
pub const TEXT: Color32 = Color32::from_rgb(232, 236, 242);
pub const DIM: Color32 = Color32::from_rgb(146, 156, 171);
pub const FAINT: Color32 = Color32::from_rgb(98, 108, 123);
pub const RAISED: Color32 = Color32::from_rgb(28, 34, 44);
pub const RAISED_HI: Color32 = Color32::from_rgb(42, 50, 63);
pub const CHOSEN: Color32 = Color32::from_rgb(66, 78, 98);
pub const LINE: Color32 = Color32::from_rgba_premultiplied(30, 30, 30, 30);
pub const LINK: Color32 = Color32::from_rgb(132, 184, 255);
pub const OK: Color32 = Color32::from_rgb(112, 214, 138);
pub const WARN: Color32 = Color32::from_rgb(255, 196, 92);
pub const BAD: Color32 = Color32::from_rgb(255, 122, 92);
pub const PANEL: Color32 = Color32::from_rgba_premultiplied(9, 12, 17, 214);
pub const R: u8 = 4;

const BOLD: &str = "bold";
const MEDIUM: &str = "medium";
const SEMIBOLD: &str = "semibold";

pub fn fonts(ctx: &egui::Context) {
    let mut f = egui::FontDefinitions::empty();
    let add = |f: &mut egui::FontDefinitions, name: &str, data: &'static [u8]| {
        f.font_data.insert(name.into(), std::sync::Arc::new(egui::FontData::from_static(data)));
    };
    add(&mut f, "regular", include_bytes!("../../../assets/fonts/Barlow-Regular.ttf"));
    add(&mut f, MEDIUM, include_bytes!("../../../assets/fonts/Barlow-Medium.ttf"));
    add(&mut f, SEMIBOLD, include_bytes!("../../../assets/fonts/Barlow-SemiBold.ttf"));
    add(&mut f, BOLD, include_bytes!("../../../assets/fonts/BarlowCondensed-Bold.ttf"));
    add(&mut f, "mono", include_bytes!("../../../assets/fonts/IBMPlexMono-Medium.ttf"));
    f.families.insert(FontFamily::Proportional, vec!["regular".into(), "mono".into()]);
    f.families.insert(FontFamily::Monospace, vec!["mono".into(), "regular".into()]);
    for n in [BOLD, MEDIUM, SEMIBOLD] {
        f.families.insert(FontFamily::Name(n.into()), vec![n.into(), "regular".into()]);
    }
    ctx.set_fonts(f);
}

pub fn visuals(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = Color32::TRANSPARENT;
    v.window_fill = Color32::from_rgb(15, 19, 26);
    v.override_text_color = Some(TEXT);
    v.hyperlink_color = LINK;
    v.selection.bg_fill = ACCENT.gamma_multiply(0.45);
    v.widgets.inactive.bg_fill = RAISED;
    v.widgets.inactive.weak_bg_fill = RAISED;
    v.widgets.hovered.bg_fill = RAISED_HI;
    v.widgets.hovered.weak_bg_fill = RAISED_HI;
    v.widgets.active.bg_fill = CHOSEN;
    v.widgets.active.weak_bg_fill = CHOSEN;
    v.slider_trailing_fill = true;
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        // no text selection anywhere.
        s.interaction.selectable_labels = false;
        s.spacing.item_spacing = vec2(8.0, 8.0);
        s.spacing.slider_width = 220.0;
        s.text_styles.insert(egui::TextStyle::Body, FontId::proportional(15.0));
        s.text_styles.insert(egui::TextStyle::Button, FontId::new(15.0, FontFamily::Name(MEDIUM.into())));
        s.text_styles.insert(egui::TextStyle::Small, FontId::proportional(13.0));
    });
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(BOLD.into()))
}
pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MEDIUM.into()))
}
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

pub fn ease(ctx: &egui::Context, id: egui::Id, on: bool) -> f32 {
    ctx.animate_bool_with_time_and_easing(id.with("ease"), on, 0.14, egui::emath::easing::cubic_out)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

/// a section of the page's one glass sheet: a hairline over it, no box of its own.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let top = ui.cursor().top();
    ui.painter().hline(ui.available_rect_before_wrap().x_range(), top, Stroke::new(1.0, Color32::from_white_alpha(22)));
    egui::Frame::new()
        .inner_margin(egui::Margin { left: 0, right: 0, top: 16, bottom: 16 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// the page's one glass sheet, every section on it.
pub fn sheet<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(PANEL)
        .stroke(Stroke::new(1.0, LINE))
        .corner_radius(R + 2)
        .inner_margin(egui::Margin { left: 24, right: 24, top: 8, bottom: 8 })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            add(ui)
        })
        .inner
}

/// a section's heading in display capitals, dim, over its rows.
pub fn heading(ui: &mut egui::Ui, s: &str) {
    ui.label(RichText::new(s.to_uppercase()).font(bold(15.0)).color(DIM).extra_letter_spacing(1.0));
    ui.add_space(4.0);
}

/// a page's title, alone: no line under it saying what the page is for.
pub fn page_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).font(semibold(30.0)).color(Color32::WHITE));
    ui.add_space(8.0);
}

/// one line of an instrument list: the name, its value, a dim detail, in three aligned columns,
/// and the room at the right end (`right_w` wide, for a button) given back.
pub fn list_row(ui: &mut egui::Ui, name: &str, value: &str, detail: &str, right_w: f32) -> Rect {
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 48.0), Sense::hover());
    let p = ui.painter();
    let y = r.center().y;
    let (name_w, value_w) = (150.0, (w * 0.32).clamp(180.0, 300.0));
    let end = r.right() - right_w;
    let one = |s: &str, font: FontId, col: Color32, width: f32| {
        let mut job = egui::text::LayoutJob::simple_singleline(s.to_string(), font, col);
        job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(8.0));
        p.layout_job(job)
    };
    let ng = one(name, medium(15.0), DIM, name_w - 12.0);
    p.galley(pos2(r.left(), y - ng.size().y * 0.5), ng, DIM);
    let vg = one(value, semibold(16.0), Color32::WHITE, value_w - 12.0);
    p.galley(pos2(r.left() + name_w, y - vg.size().y * 0.5), vg, Color32::WHITE);
    let dx = r.left() + name_w + value_w;
    if !detail.is_empty() && end - dx > 40.0 {
        let dg = one(detail, FontId::proportional(13.5), DIM, end - dx - 12.0);
        p.galley(pos2(dx, y - dg.size().y * 0.5), dg, DIM);
    }
    Rect::from_min_max(pos2(end, r.top()), r.max)
}

/// a hairline across a card, between rows.
pub fn rule(ui: &mut egui::Ui) {
    let y = ui.cursor().top() - ui.spacing().item_spacing.y * 0.5;
    ui.painter().hline(ui.max_rect().x_range(), y, Stroke::new(1.0, Color32::from_white_alpha(12)));
}

/// Small grey text.
pub fn note(ui: &mut egui::Ui, s: impl Into<String>) {
    ui.label(RichText::new(s.into()).size(13.0).color(DIM));
}

/// A label with a value beside it, for the cards' facts.
pub fn fact(ui: &mut egui::Ui, k: &str, v: impl Into<RichText>) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(vec2(90.0, 18.0), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_width(90.0);
            ui.label(RichText::new(k).size(13.0).color(DIM));
        });
        ui.label(v.into());
    });
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    /// the one thing to press: yellow
    Primary,
    /// everything else
    Plain,
    /// stops something: red outline
    Danger,
}

/// A button, with an icon before its text if given.
pub fn button(ui: &mut egui::Ui, text: &str, icon: Option<&egui::TextureHandle>, kind: Kind, size: Vec2) -> egui::Response {
    // the yellow one in the display face, in capitals, as the game's take off.
    let (font, text) = match kind {
        Kind::Primary => (bold(if size.y >= 44.0 { 26.0 } else { 18.0 }), text.to_uppercase()),
        _ => (medium(if size.y >= 44.0 { 17.0 } else { 15.0 }), text.to_string()),
    };
    let text = text.as_str();
    let enabled = ui.is_enabled();
    let (fg, bg, bg_hi) = match kind {
        Kind::Primary => (ON_ACCENT, ACCENT, ACCENT_HI),
        Kind::Plain => (TEXT, RAISED, RAISED_HI),
        Kind::Danger => (BAD, Color32::from_rgba_premultiplied(40, 12, 8, 120), Color32::from_rgba_premultiplied(70, 20, 12, 160)),
    };
    let g = ui.painter().layout_no_wrap(text.to_string(), font, if enabled { fg } else { FAINT });
    let icon_w = if icon.is_some() { 18.0 + 8.0 } else { 0.0 };
    let w = size.x.max(g.size().x + icon_w + 28.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, size.y), Sense::click());
    let lit = ease(ui.ctx(), resp.id, resp.hovered() && enabled);
    let p = ui.painter();
    let fill = if enabled { mix(bg, bg_hi, lit) } else { RAISED.gamma_multiply(0.6) };
    let rect_drawn = if resp.is_pointer_button_down_on() && enabled { rect.shrink(1.0) } else { rect };
    p.rect_filled(rect_drawn, R, fill);
    match kind {
        Kind::Danger => {
            p.rect_stroke(rect_drawn, R, Stroke::new(1.0, BAD.gamma_multiply(0.6)), egui::StrokeKind::Inside);
        }
        Kind::Plain => {
            p.rect_stroke(rect_drawn, R, Stroke::new(1.0, mix(Color32::from_white_alpha(22), Color32::from_white_alpha(60), lit)), egui::StrokeKind::Inside);
        }
        _ => {}
    }
    let total = g.size().x + icon_w;
    let x0 = rect.center().x - total * 0.5;
    if let Some(t) = icon {
        let ir = Rect::from_center_size(pos2(x0 + 9.0, rect.center().y), vec2(18.0, 18.0));
        let tint = if kind == Kind::Plain { TEXT } else { fg };
        p.image(t.id(), ir, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), if enabled { tint } else { FAINT });
    }
    p.galley(pos2(x0 + icon_w, rect.center().y - g.size().y * 0.5), g, fg);
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

/// A segmented choice: one of `options`, the chosen one lit.
pub fn segments<T: PartialEq + Copy>(ui: &mut egui::Ui, current: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    egui::Frame::new().fill(Color32::from_rgba_premultiplied(0, 0, 0, 80)).stroke(Stroke::new(1.0, LINE)).corner_radius(R + 1).inner_margin(3).show(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.horizontal(|ui| {
            for (v, label) in options {
                let on = *current == *v;
                let g = ui.painter().layout_no_wrap(label.to_string(), medium(14.5), TEXT);
                let (rect, resp) = ui.allocate_exact_size(vec2(g.size().x + 22.0, 26.0), Sense::click());
                let lit = ease(ui.ctx(), resp.id, resp.hovered() && !on);
                let p = ui.painter();
                if on {
                    p.rect_filled(rect, R, CHOSEN);
                } else if lit > 0.0 {
                    p.rect_filled(rect, R, Color32::from_white_alpha((12.0 * lit) as u8));
                }
                let col = if !ui.is_enabled() { FAINT } else if on { Color32::WHITE } else { mix(DIM, TEXT, lit) };
                p.galley(rect.center() - g.size() * 0.5, g, col);
                if resp.clicked() && !on {
                    *current = *v;
                    changed = true;
                }
            }
        });
    });
    changed
}

/// An on/off switch.
pub fn switch(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let (w, h) = (36.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 24.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    let t = ui.ctx().animate_bool_with_time_and_easing(resp.id.with("knob"), *on, 0.16, egui::emath::easing::cubic_out);
    let lit = ease(ui.ctx(), resp.id, resp.hovered());
    let track = Rect::from_center_size(rect.center(), vec2(w, h));
    let p = ui.painter();
    p.rect_filled(track, h * 0.5, mix(mix(RAISED, RAISED_HI, lit), mix(ACCENT, ACCENT_HI, lit), t));
    let knob = pos2(track.left() + h * 0.5 + (w - h) * t, track.center().y);
    p.circle_filled(knob, h * 0.5 - 3.0, mix(DIM, ON_ACCENT, t));
    resp.clicked()
}

/// one setting: its name (and a "?" with the hint, when there is one) on the
/// left, centred on the control against the right edge (in reading order: it
/// is placed by last frame's size), a faint line above all but a card's first row.
pub fn row<R>(ui: &mut egui::Ui, name: &str, hint: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let w = ui.available_width();
    let id = egui::Id::unique(("launcher row", name));
    if ui.min_rect().height() > 40.0 {
        let y = ui.cursor().top() + 1.0;
        ui.painter().hline(ui.cursor().left()..=ui.cursor().left() + w, y, Stroke::new(1.0, Color32::from_white_alpha(12)));
        ui.add_space(8.0);
    }
    let lw = (w * 0.45).clamp(200.0, 340.0);
    let last = ui.ctx().data(|d| d.get_temp::<Vec2>(id)).unwrap_or(vec2(f32::INFINITY, 32.0));
    let out = ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(vec2(lw, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(lw - 16.0);
            let lh = ui.painter().layout_no_wrap("Ag".into(), medium(15.5), TEXT).size().y.max(if hint.is_empty() { 0.0 } else { 18.0 });
            // (a row is as tall as the interact height, which would push the name down)
            ui.spacing_mut().interact_size.y = lh;
            ui.add_space(((last.y.min(34.0) - lh) * 0.5).max(0.0));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(RichText::new(name).font(medium(15.5)).color(TEXT));
                if !hint.is_empty() {
                    help(ui, hint);
                }
            });
        });
        let cw = ui.available_width().max(80.0);
        let at = Rect::from_min_size(pos2(ui.max_rect().right() - last.x.min(cw), ui.cursor().top()), vec2(cw, f32::INFINITY));
        let r = ui.scope_builder(egui::UiBuilder::new().max_rect(at).layout(egui::Layout::top_down(egui::Align::Min)), add);
        let got = r.response.rect.size();
        if (got - last).abs().max_elem() > 0.5 {
            ui.ctx().data_mut(|d| d.insert_temp(id, got));
            ui.ctx().request_discard("launcher row size");
        }
        r.inner
    });
    ui.add_space(8.0);
    out.inner
}

/// `s` as a sentence: a capital first letter and a full stop.
pub fn sentence(s: &str) -> String {
    let s = s.trim();
    let mut c = s.chars();
    let Some(first) = c.next() else { return String::new() };
    let mut out: String = first.to_uppercase().chain(c).collect();
    if out.chars().last().is_some_and(|l| l.is_alphanumeric() || l == ')') {
        out.push('.');
    }
    out
}

/// The small round "?" beside a setting: what it does, on hover.
pub fn help(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
    let lit = ease(ui.ctx(), resp.id, resp.hovered());
    let p = ui.painter();
    let c = rect.center();
    p.circle_filled(c, 7.5, Color32::from_white_alpha((6.0 + 14.0 * lit) as u8));
    p.circle_stroke(c, 7.5, Stroke::new(1.0, mix(FAINT, ACCENT, lit)));
    p.text(c + vec2(0.0, 0.5), egui::Align2::CENTER_CENTER, "?", semibold(11.5), mix(DIM, ACCENT, lit));
    let text = sentence(text);
    resp.on_hover_ui(|ui| {
        ui.set_max_width(300.0);
        ui.label(RichText::new(text).size(14.5).color(TEXT));
    })
}

/// The title bar's height.
pub const TITLE_H: f32 = 34.0;

/// What the title bar asks of the window.
#[derive(Clone, Copy, PartialEq)]
pub enum TitleAction {
    Minimize,
    Maximize,
    Close,
}

/// The window's own title bar: a strip to drag the window by (twice to
/// maximise), and minimise, maximise and close at the right, drawn as the
/// rest of the launcher is, in place of the system's.
pub fn title_bar(ui: &mut egui::Ui, maximized: bool) -> Option<TitleAction> {
    let rect = ui.max_rect();
    let mut act = None;
    let bw = 46.0;
    let drag = Rect::from_min_max(rect.min, pos2(rect.right() - 3.0 * bw, rect.bottom()));
    let resp = ui.interact(drag, egui::Id::unique("launcher title drag"), Sense::click_and_drag());
    if resp.drag_started() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if resp.double_clicked() {
        act = Some(TitleAction::Maximize);
    }
    let p = ui.painter();
    // the two words in one face and size, the second in the accent.
    let mut job = egui::text::LayoutJob::default();
    let fmt = |col| egui::TextFormat { font_id: bold(15.0), color: col, extra_letter_spacing: 1.2, ..Default::default() };
    job.append("FABLE FLIGHT", 0.0, fmt(Color32::WHITE));
    job.append("LAUNCHER", 8.0, fmt(ACCENT));
    let g = p.layout_job(job);
    p.galley(pos2(rect.left() + 16.0, rect.center().y - g.size().y * 0.5), g, Color32::WHITE);
    for (k, what) in [TitleAction::Minimize, TitleAction::Maximize, TitleAction::Close].into_iter().enumerate() {
        let r = Rect::from_min_size(pos2(rect.right() - bw * (3 - k) as f32, rect.top()), vec2(bw, rect.height()));
        let b = ui.interact(r, egui::Id::unique(("launcher title button", k)), Sense::click());
        let lit = ease(ui.ctx(), b.id, b.hovered());
        let close = what == TitleAction::Close;
        let p = ui.painter();
        if lit > 0.0 {
            p.rect_filled(r, 0.0, if close { Color32::from_rgb(196, 43, 28).gamma_multiply(lit) } else { Color32::from_white_alpha((16.0 * lit) as u8) });
        }
        let col = mix(DIM, Color32::WHITE, lit);
        let st = Stroke::new(1.0, col);
        let c = r.center();
        match what {
            TitleAction::Minimize => {
                p.hline((c.x - 5.0)..=(c.x + 5.0), c.y, st);
            }
            TitleAction::Maximize if maximized => {
                // two squares, one behind the other: back to the size before.
                p.rect_stroke(Rect::from_center_size(c + vec2(-1.5, 1.5), vec2(8.0, 8.0)), 1.0, st, egui::StrokeKind::Middle);
                p.line(vec![c + vec2(-2.0, -3.0), c + vec2(-2.0, -5.0), c + vec2(5.5, -5.0), c + vec2(5.5, 2.5), c + vec2(3.5, 2.5)], st);
            }
            TitleAction::Maximize => {
                p.rect_stroke(Rect::from_center_size(c, vec2(10.0, 10.0)), 1.0, st, egui::StrokeKind::Middle);
            }
            TitleAction::Close => {
                p.line_segment([c + vec2(-5.0, -5.0), c + vec2(5.0, 5.0)], st);
                p.line_segment([c + vec2(-5.0, 5.0), c + vec2(5.0, -5.0)], st);
            }
        }
        if b.clicked() {
            act = Some(what);
        }
    }
    p.hline(rect.x_range(), rect.bottom() - 0.5, Stroke::new(1.0, Color32::from_white_alpha(12)));
    act
}

/// Without the system's frame the window is resized from its edges here:
/// the pointer near one shows the resize arrows, and pressing starts it.
pub fn edge_resize(ctx: &egui::Context, maximized: bool) {
    if maximized {
        return;
    }
    let screen = ctx.content_rect();
    let Some(p) = ctx.input(|i| i.pointer.hover_pos()) else { return };
    let m = 6.0;
    let (l, r, t, b) = (p.x < screen.left() + m, p.x > screen.right() - m, p.y < screen.top() + m, p.y > screen.bottom() - m);
    use egui::viewport::ResizeDirection as D;
    let dir = match (l, r, t, b) {
        (true, _, true, _) => D::NorthWest,
        (_, true, true, _) => D::NorthEast,
        (true, _, _, true) => D::SouthWest,
        (_, true, _, true) => D::SouthEast,
        (true, ..) => D::West,
        (_, true, ..) => D::East,
        (_, _, true, _) => D::North,
        (_, _, _, true) => D::South,
        _ => return,
    };
    ctx.set_cursor_icon(match dir {
        D::North | D::South => egui::CursorIcon::ResizeVertical,
        D::East | D::West => egui::CursorIcon::ResizeHorizontal,
        D::NorthWest | D::SouthEast => egui::CursorIcon::ResizeNwSe,
        D::NorthEast | D::SouthWest => egui::CursorIcon::ResizeNeSw,
    });
    if ctx.input(|i| i.pointer.primary_pressed()) {
        ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
    }
}

/// The pictures beside the sidebar's pages.
#[derive(Clone, Copy)]
pub enum Icon {
    Home,
    Graphics,
    Updates,
    Settings,
}

/// Draws `icon` in a square of side `s` around `c`, in lines like the game's settings icons.
pub fn icon(p: &egui::Painter, icon: Icon, c: egui::Pos2, s: f32, col: Color32) {
    let k = s / 20.0;
    let at = |x: f32, y: f32| pos2(c.x + x * k, c.y + y * k);
    let st = Stroke::new(1.6, col);
    match icon {
        Icon::Home => {
            p.line(vec![at(-8.0, -1.0), at(0.0, -8.0), at(8.0, -1.0)], st);
            p.line(vec![at(-6.0, -3.0), at(-6.0, 8.0), at(6.0, 8.0), at(6.0, -3.0)], st);
            p.line(vec![at(-2.0, 8.0), at(-2.0, 3.0), at(2.0, 3.0), at(2.0, 8.0)], st);
        }
        Icon::Graphics => {
            p.rect_stroke(Rect::from_min_max(at(-9.0, -7.0), at(9.0, 5.0)), 2.0 * k, st, egui::StrokeKind::Middle);
            p.line_segment([at(0.0, 5.0), at(0.0, 8.0)], st);
            p.line_segment([at(-4.0, 8.0), at(4.0, 8.0)], st);
        }
        Icon::Updates => {
            p.line_segment([at(0.0, -8.0), at(0.0, 3.0)], st);
            p.line(vec![at(-4.5, -1.5), at(0.0, 3.0), at(4.5, -1.5)], st);
            p.line(vec![at(-8.0, 3.0), at(-8.0, 8.0), at(8.0, 8.0), at(8.0, 3.0)], st);
        }
        Icon::Settings => {
            for (y, x) in [(-5.5, -3.0), (0.0, 4.0), (5.5, -1.0)] {
                p.line_segment([at(-8.5, y), at(8.5, y)], st);
                p.circle_filled(at(x, y), 2.4 * k, col);
            }
        }
    }
}

/// A page in the sidebar: its icon and name, lit when `on`, a dot when `badge`.
pub fn nav_item(ui: &mut egui::Ui, name: &str, ic: Icon, on: bool, badge: bool) -> egui::Response {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(vec2(w, 40.0), Sense::click());
    let lit = ease(ui.ctx(), resp.id, resp.hovered() || on);
    let p = ui.painter();
    if lit > 0.0 {
        p.rect_filled(rect, R, Color32::from_white_alpha((if on { 14.0 } else { 9.0 } * lit) as u8));
    }
    // the chosen page outlined, as the game's sidebars, never a bar down one side.
    let pick = ui.ctx().animate_bool_with_time(resp.id.with("pick"), on, 0.15);
    if pick > 0.0 {
        p.rect_stroke(rect, R, Stroke::new(1.0, ACCENT.gamma_multiply(0.7 * pick)), egui::StrokeKind::Inside);
    }
    let col = if on { Color32::WHITE } else { mix(DIM, TEXT, lit) };
    icon(p, ic, pos2(rect.left() + 26.0, rect.center().y), 19.0, if on { ACCENT } else { col });
    let g = p.layout_no_wrap(name.to_string(), medium(16.0), col);
    p.galley(pos2(rect.left() + 48.0, rect.center().y - g.size().y * 0.5), g, col);
    if badge {
        p.circle_filled(pos2(rect.right() - 16.0, rect.center().y), 4.0, ACCENT);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A progress bar the launcher's way: a thin track, the yellow fill with a
/// sheen running along it while something is under way.
pub fn progress_bar(ui: &mut egui::Ui, rect: Rect, frac: Option<f32>, busy: bool, col: Color32) {
    let p = ui.painter();
    let r = CornerRadius::same((rect.height() * 0.5) as u8);
    p.rect_filled(rect, r, Color32::from_white_alpha(22));
    let t = ui.input(|i| i.time) as f32;
    match frac {
        Some(f) => {
            let f = f.clamp(0.0, 1.0);
            if f > 0.0 {
                let fill = Rect::from_min_size(rect.min, vec2((rect.width() * f).max(rect.height()), rect.height()));
                p.rect_filled(fill, r, col);
                if busy {
                    // the sheen
                    let x = fill.left() + ((t * 0.6).fract() * (fill.width() + 160.0)) - 80.0;
                    let sheen = Rect::from_center_size(pos2(x, fill.center().y), vec2(80.0, fill.height())).intersect(fill);
                    if sheen.width() > 0.0 {
                        p.rect_filled(sheen, r, Color32::from_white_alpha(46));
                    }
                }
            }
        }
        None if busy => {
            // waiting on something of unknown length: a block sweeping across
            let w = rect.width() * 0.25;
            let x = rect.left() - w + (t * 0.5).fract() * (rect.width() + w);
            let block = Rect::from_min_size(pos2(x, rect.top()), vec2(w, rect.height())).intersect(rect);
            p.rect_filled(block, r, col);
        }
        None => {}
    }
}

/// Seconds as "12.4 s" or "1 min 05 s".
pub fn secs(s: f64) -> String {
    if s < 60.0 { format!("{s:.1} s") } else { format!("{} min {:02} s", (s / 60.0) as u64, (s % 60.0) as u64) }
}

pub fn bytes(n: u64) -> String {
    if n >= 1 << 30 {
        format!("{:.2} GB", n as f64 / (1u64 << 30) as f64)
    } else if n >= 1 << 20 {
        format!("{:.1} MB", n as f64 / (1u64 << 20) as f64)
    } else {
        format!("{:.0} KB", n as f64 / 1024.0)
    }
}
