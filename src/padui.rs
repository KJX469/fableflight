//! Gamepad button prompts in the Xbox style (green A, red B, blue X,
//! yellow Y, bumpers, triggers, sticks, d-pad, View and Menu), and the
//! switch between keyboard hints and gamepad ones: while a pad is plugged
//! in, the menus and the HUD show its buttons instead of keys.
//!
//! Prompts are written as short specs: button names separated by spaces,
//! with `+` and `/` drawn as text between them, e.g. `"Y + View"`, `"LT / RT"`.

use egui::{pos2, vec2, Color32, CornerRadius, Painter, Pos2, Rect, Stroke, Ui};
use std::sync::atomic::{AtomicBool, Ordering};

static ACTIVE: AtomicBool = AtomicBool::new(false);

/// A gamepad is plugged in: hints show its buttons.
pub fn active() -> bool {
    ACTIVE.load(Ordering::Relaxed)
}

pub fn set_active(on: bool) {
    ACTIVE.store(on, Ordering::Relaxed);
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Btn {
    A,
    B,
    X,
    Y,
    LB,
    RB,
    LT,
    RT,
    View,
    Menu,
    /// the sticks, moved
    LS,
    RS,
    /// the sticks, clicked in
    L3,
    R3,
    /// the whole d-pad, and one direction of it
    Dpad,
    Up,
    Down,
    Left,
    Right,
}

impl Btn {
    pub fn parse(s: &str) -> Option<Btn> {
        Some(match s {
            "A" => Btn::A,
            "B" => Btn::B,
            "X" => Btn::X,
            "Y" => Btn::Y,
            "LB" => Btn::LB,
            "RB" => Btn::RB,
            "LT" => Btn::LT,
            "RT" => Btn::RT,
            "View" => Btn::View,
            "Menu" => Btn::Menu,
            "LS" => Btn::LS,
            "RS" => Btn::RS,
            "L3" => Btn::L3,
            "R3" => Btn::R3,
            "Dpad" => Btn::Dpad,
            "Up" => Btn::Up,
            "Down" => Btn::Down,
            "Left" => Btn::Left,
            "Right" => Btn::Right,
            _ => return None,
        })
    }

    /// Width of the glyph at a height.
    pub fn width(self, h: f32) -> f32 {
        match self {
            Btn::LB | Btn::RB => h * 1.65,
            Btn::LT | Btn::RT => h * 1.35,
            _ => h,
        }
    }
}

const BODY: Color32 = Color32::from_rgb(46, 52, 62);
const RIM: Color32 = Color32::from_rgb(196, 202, 212);
const LIGHT: Color32 = Color32::from_rgb(240, 242, 245);

/// a word in the display face, centred on its letters themselves rather
/// than on the font's line box, so a and lb sit dead in the middle.
fn word(p: &Painter, at: Pos2, s: &str, size: f32, col: Color32) {
    let g = p.layout_no_wrap(s.to_string(), crate::menu::bold(size), col);
    let off = at - g.mesh_bounds.center();
    p.galley(off.to_pos2(), g, col);
}

/// Draw one button glyph filling `r` (its height sets the size).
pub fn paint(p: &Painter, r: Rect, b: Btn) {
    let h = r.height();
    let c = r.center();
    let rad = h * 0.5 - 0.5;
    let shadow = |rad: f32| p.circle_filled(c + vec2(0.0, 1.0), rad, Color32::from_black_alpha(90));
    let ink = Stroke::new((h * 0.07).max(1.0), LIGHT);
    match b {
        Btn::A | Btn::B | Btn::X | Btn::Y => {
            let (fill, ink, s) = match b {
                Btn::A => (Color32::from_rgb(84, 168, 58), Color32::WHITE, "A"),
                Btn::B => (Color32::from_rgb(212, 52, 48), Color32::WHITE, "B"),
                Btn::X => (Color32::from_rgb(38, 112, 214), Color32::WHITE, "X"),
                _ => (Color32::from_rgb(236, 182, 24), Color32::from_rgb(40, 30, 0), "Y"),
            };
            shadow(rad);
            p.circle_filled(c, rad, fill);
            word(p, c, s, h * 0.56, ink);
        }
        Btn::LB | Btn::RB | Btn::LT | Btn::RT => {
            // bumpers are flat pills; triggers are rounded over the top.
            let body = if matches!(b, Btn::LB | Btn::RB) { r.shrink2(vec2(0.0, h * 0.1)) } else { r };
            let round = (body.height() * 0.5) as u8;
            let small = (h * 0.18) as u8;
            let cr = match b {
                Btn::LB | Btn::RB => CornerRadius::same(round),
                _ => CornerRadius { nw: round, ne: round, sw: small, se: small },
            };
            p.rect_filled(body.translate(vec2(0.0, 1.0)), cr, Color32::from_black_alpha(90));
            p.rect_filled(body, cr, BODY);
            p.rect_stroke(body, cr, Stroke::new(1.0, RIM), egui::StrokeKind::Inside);
            let s = match b {
                Btn::LB => "LB",
                Btn::RB => "RB",
                Btn::LT => "LT",
                _ => "RT",
            };
            word(p, body.center(), s, body.height() * 0.6, LIGHT);
        }
        Btn::View | Btn::Menu => {
            shadow(rad);
            p.circle_filled(c, rad, BODY);
            p.circle_stroke(c, rad - 0.5, Stroke::new(1.0, RIM));
            let u = h * 0.11;
            if b == Btn::Menu {
                for k in -1..=1 {
                    let y = c.y + k as f32 * u * 1.5;
                    p.line_segment([pos2(c.x - u * 2.2, y), pos2(c.x + u * 2.2, y)], ink);
                }
            } else {
                // two windows, one over the other
                let a = Rect::from_min_size(c + vec2(-u * 2.3, -u * 1.9), vec2(u * 3.0, u * 2.4));
                let b2 = a.translate(vec2(u * 1.5, u * 1.4));
                p.rect_stroke(a, 1.0, ink, egui::StrokeKind::Middle);
                p.rect_filled(b2, 1.0, BODY);
                p.rect_stroke(b2, 1.0, ink, egui::StrokeKind::Middle);
            }
        }
        Btn::LS | Btn::RS | Btn::L3 | Btn::R3 => {
            let click = matches!(b, Btn::L3 | Btn::R3);
            shadow(rad);
            p.circle_filled(c, rad, BODY);
            p.circle_stroke(c, rad - 0.5, Stroke::new(if click { 2.0 } else { 1.0 }, RIM));
            p.circle_stroke(c, rad * 0.62, Stroke::new(1.0, Color32::from_white_alpha(70)));
            let letter = if matches!(b, Btn::LS | Btn::L3) { "L" } else { "R" };
            if click {
                word(p, c, &format!("{letter}S"), h * 0.4, LIGHT);
            } else {
                word(p, c, letter, h * 0.46, LIGHT);
                // ticks all round: the stick is moved.
                let t = h * 0.08;
                for (dx, dy) in [(0.0, -1.0), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
                    let d = vec2(dx, dy);
                    let tip = c + d * (rad - 1.0);
                    let back = tip - d * t * 1.3;
                    let side = vec2(-dy, dx) * t;
                    p.add(egui::Shape::convex_polygon(vec![tip, back + side, back - side], LIGHT, Stroke::NONE));
                }
            }
        }
        Btn::Dpad | Btn::Up | Btn::Down | Btn::Left | Btn::Right => {
            let arm = h * 0.36;
            let len = h * 0.5 - 0.5;
            let v = Rect::from_center_size(c, vec2(arm, len * 2.0));
            let hz = Rect::from_center_size(c, vec2(len * 2.0, arm));
            for rr in [v, hz] {
                p.rect_filled(rr.translate(vec2(0.0, 1.0)), 2.0, Color32::from_black_alpha(90));
            }
            for rr in [v, hz] {
                p.rect_filled(rr, 2.0, BODY);
            }
            let arms = [
                (Btn::Up, Rect::from_min_max(v.min, pos2(v.right(), c.y))),
                (Btn::Down, Rect::from_min_max(pos2(v.left(), c.y), v.max)),
                (Btn::Left, Rect::from_min_max(hz.min, pos2(c.x, hz.bottom()))),
                (Btn::Right, Rect::from_min_max(pos2(c.x, hz.top()), hz.max)),
            ];
            // the direction meant is lit; the whole pad has every arm lit, softer.
            for (d, rr) in arms {
                if b == Btn::Dpad || b == d {
                    p.rect_filled(rr.shrink(1.0), 1.5, if b == Btn::Dpad { Color32::from_rgb(120, 128, 140) } else { LIGHT });
                }
            }
            for rr in [v, hz] {
                p.rect_stroke(rr, 2.0, Stroke::new(1.0, RIM), egui::StrokeKind::Inside);
            }
        }
    }
}

enum Tok<'a> {
    Btn(Btn),
    Text(&'a str),
}

fn tokens(spec: &str) -> impl Iterator<Item = Tok<'_>> {
    spec.split_whitespace().map(|t| Btn::parse(t).map(Tok::Btn).unwrap_or(Tok::Text(t)))
}

const GAP: f32 = 3.0;

fn text_w(p: &Painter, s: &str, size: f32) -> f32 {
    p.layout_no_wrap(s.to_string(), crate::menu::medium(size), Color32::WHITE).size().x
}

/// Width of a whole spec at a glyph height.
pub fn spec_width(p: &Painter, spec: &str, h: f32) -> f32 {
    let mut w = 0.0;
    for (k, t) in tokens(spec).enumerate() {
        if k > 0 {
            w += GAP;
        }
        w += match t {
            Tok::Btn(b) => b.width(h),
            Tok::Text(s) => text_w(p, s, h * 0.7),
        };
    }
    w
}

/// Paint a spec with its left edge at `x`, centred on `y`.
pub fn paint_spec(p: &Painter, x: f32, y: f32, spec: &str, h: f32, text: Color32) {
    let mut x = x;
    for (k, t) in tokens(spec).enumerate() {
        if k > 0 {
            x += GAP;
        }
        match t {
            Tok::Btn(b) => {
                let w = b.width(h);
                paint(p, Rect::from_min_size(pos2(x, y - h * 0.5), vec2(w, h)), b);
                x += w;
            }
            Tok::Text(s) => {
                let g = p.layout_no_wrap(s.to_string(), crate::menu::medium(h * 0.7), text);
                let w = g.size().x;
                p.galley(pos2(x, y - g.size().y * 0.5), g, text);
                x += w;
            }
        }
    }
}

/// A spec as a widget, `h` points tall.
pub fn glyphs(ui: &mut Ui, spec: &str, h: f32) -> egui::Response {
    let w = spec_width(ui.painter(), spec, h);
    let (r, resp) = ui.allocate_exact_size(vec2(w, h), egui::Sense::hover());
    paint_spec(ui.painter(), r.left(), r.center().y, spec, h, Color32::from_white_alpha(200));
    resp
}

/// A row of prompts: each spec's buttons, then what they do.
pub fn prompts(ui: &mut Ui, items: &[(&str, &str)], h: f32, size: f32, col: Color32) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for (k, (spec, what)) in items.iter().enumerate() {
            if k > 0 {
                ui.add_space(10.0);
            }
            glyphs(ui, spec, h);
            ui.label(egui::RichText::new(*what).size(size).color(col));
        }
    });
}

/// A hint beside a menu button: a key, gamepad buttons, plain words, or none.
#[derive(Copy, Clone)]
pub enum Hint<'a> {
    None,
    Text(&'a str),
    Pad(&'a str),
}

impl<'a> Hint<'a> {
    /// the pad's buttons while a pad is in use, else the key on a cap.
    pub fn of(key: &'a str, pad: &'a str) -> Self {
        if active() {
            if pad.is_empty() { Hint::None } else { Hint::Pad(pad) }
        } else if key.is_empty() {
            Hint::None
        } else {
            Hint::Text(key)
        }
    }
}

/// a d-pad direction.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

/// a widget the d-pad can go to, where the last pass laid it out.
#[derive(Copy, Clone, Debug)]
pub struct Spot {
    pub id: egui::Id,
    pub rect: Rect,
    pub layer: egui::LayerId,
    /// some of it is on screen, not scrolled away.
    pub shown: bool,
}

/// the widgets the d-pad can go to: focusable and enabled, drawn on the
/// last pass (a layer not drawn has none). the background only counts when
/// nothing over it takes the pad.
pub fn spots(ctx: &egui::Context) -> Vec<Spot> {
    use egui::Order;
    let screen = ctx.content_rect();
    let all: Vec<Spot> = ctx.viewport(|v| {
        v.prev_pass
            .widgets
            .layers()
            .filter(|(l, _)| !matches!(l.order, Order::Tooltip | Order::Debug))
            .flat_map(|(_, ws)| ws.iter())
            .filter(|w| w.enabled && w.sense.is_focusable() && w.rect.is_positive())
            .map(|w| Spot { id: w.id, rect: w.rect, layer: w.layer_id, shown: w.interact_rect.is_positive() && screen.intersects(w.interact_rect) })
            .collect()
    });
    // the menu's layers only: the hud and panels behind it are not for the pad.
    let top = all.iter().map(|s| s.layer.order).max();
    let floor = if top > Some(Order::Background) { Order::Middle } else { Order::Background };
    // an egui slider's number box is a text box the pad cannot type in, and
    // the slider beside it already moves the same value.
    let sliders: Vec<(egui::LayerId, Rect)> = ctx.viewport(|v| v.prev_pass.widgets.layers().flat_map(|(_, ws)| ws.iter()).filter(|w| w.sense.senses_drag() && !w.sense.senses_click()).map(|w| (w.layer_id, w.rect)).collect());
    let by_slider = |s: &Spot| sliders.iter().any(|(l, r)| *l == s.layer && (s.rect.min.x - r.max.x).abs() < 16.0 && s.rect.y_range().intersects(r.y_range()));
    all.into_iter().filter(|s| s.layer.order >= floor && !by_slider(s)).collect()
}

/// how far `b` is from `a` going `d`, or none when it is not that way, and
/// whether it is in line (overlapping across the move). going up or down
/// the gap counts once and a step sideways twice (the next row, wherever its
/// control sits); going across the gap counts half and a step sideways eight times.
fn cost(a: Rect, b: Rect, d: Dir) -> Option<(f32, bool)> {
    // turned so the move is always along +, the other axis across.
    let (a0, a1, b0, b1, ax, bx) = match d {
        Dir::Down => (a.min.y, a.max.y, b.min.y, b.max.y, a.x_range(), b.x_range()),
        Dir::Up => (-a.max.y, -a.min.y, -b.max.y, -b.min.y, a.x_range(), b.x_range()),
        Dir::Right => (a.min.x, a.max.x, b.min.x, b.max.x, a.y_range(), b.y_range()),
        Dir::Left => (-a.max.x, -a.min.x, -b.max.x, -b.min.x, a.y_range(), b.y_range()),
    };
    let mid = (a0 + a1) * 0.5;
    // past this one's middle, and not a box round it.
    if b0 < mid - 1.0 || (b0 + b1) * 0.5 <= mid + 1.0 {
        return None;
    }
    let gap = (b0 - a1).max(0.0);
    let side = if ax.max < bx.min {
        bx.min - ax.max
    } else if bx.max < ax.min {
        ax.min - bx.max
    } else {
        0.0
    };
    // within 45 degrees of the way pressed: quit, far below and a little to the left, is not left.
    if side > (b0 + b1) * 0.5 - mid {
        return None;
    }
    // across a page, the row matters more than how far over its control sits.
    let (g, k) = if matches!(d, Dir::Up | Dir::Down) { (1.0, 2.0) } else { (0.5, 8.0) };
    // lined up by the leading edge, as a list or a grid reads.
    Some((gap * g + side * k + (ax.min - bx.min).abs() * 0.05, side == 0.0))
}

/// the widget to go to from `from` going `d`: across, the nearest in line
/// with it (its own layer, then the rest of the menu) before the nearest off
/// the line anywhere; up and down, the nearest row in its own layer, then the rest.
pub fn next(all: &[Spot], from: &Spot, d: Dir) -> Option<egui::Id> {
    if matches!(d, Dir::Up | Dir::Down) {
        return nearest(all, from, d, Some(true), false).or_else(|| nearest(all, from, d, Some(false), false));
    }
    nearest(all, from, d, Some(true), true).or_else(|| nearest(all, from, d, Some(false), true)).or_else(|| nearest(all, from, d, None, false))
}

/// the cheapest widget going `d` from `from`: in its own layer, the others or
/// any (`same`), in line only or not.
fn nearest(all: &[Spot], from: &Spot, d: Dir, same: Option<bool>, in_line: bool) -> Option<egui::Id> {
    all.iter()
        .filter(|s| s.id != from.id && same.is_none_or(|k| (s.layer == from.layer) == k))
        .filter_map(|s| cost(from.rect, s.rect, d).filter(|c| c.1 || !in_line).map(|c| (c.0, s.id)))
        .min_by(|x, y| x.0.total_cmp(&y.0))
        .map(|x| x.1)
}

/// the d-pad's move from the focused widget, on the last pass's layout.
pub fn step(ctx: &egui::Context, d: Dir) -> Option<egui::Id> {
    let id = ctx.memory(|m| m.focused())?;
    let all = spots(ctx);
    let from = all.iter().find(|s| s.id == id).copied().or_else(|| {
        let r = ctx.read_response(id)?;
        Some(Spot { id, rect: r.rect, layer: r.layer_id, shown: true })
    })?;
    next(&all, &from, d)
}

/// something is focused that the pad can work: on screen and not under a
/// pop up (a screen just opened has nothing to go by yet, so it counts).
pub fn focus_usable(ctx: &egui::Context) -> bool {
    let Some(id) = ctx.memory(|m| m.focused()) else { return false };
    let all = spots(ctx);
    all.is_empty() || all.iter().any(|s| s.id == id) || !ctx.viewport(|v| v.prev_pass.widgets.contains(id))
}

/// where the d-pad starts on a screen with no first choice of its own: the
/// first widget on screen in reading order, in the topmost layer, under the
/// header strip (back and the tabs live there) when anything is.
pub fn home(ctx: &egui::Context) -> Option<egui::Id> {
    let r = ctx.content_rect();
    first(&spots(ctx), r.top() + r.height() * 0.12)
}

fn first(all: &[Spot], header: f32) -> Option<egui::Id> {
    let top = all.iter().filter(|s| s.shown).map(|s| s.layer.order).max()?;
    let pool: Vec<&Spot> = all.iter().filter(|s| s.shown && s.layer.order == top).collect();
    let body: Vec<&Spot> = pool.iter().copied().filter(|s| s.rect.min.y >= header).collect();
    (if body.is_empty() { pool } else { body })
        .into_iter()
        .min_by(|a, b| ((a.rect.min.y / 12.0).floor(), a.rect.min.x).partial_cmp(&((b.rect.min.y / 12.0).floor(), b.rect.min.x)).unwrap_or(std::cmp::Ordering::Equal))
        .map(|s| s.id)
}

thread_local! {
    /// the focused widget that takes left and right itself (a slider), this pass.
    static SIDEWAYS: std::cell::Cell<Option<egui::Id>> = const { std::cell::Cell::new(None) };
}

/// a focused widget that moves with left and right (a slider) says so each pass.
pub fn takes_sideways(id: egui::Id) {
    SIDEWAYS.with(|s| s.set(Some(id)));
}

/// the focused widget takes left and right itself: it said so, or it is
/// egui's own slider (dragged but never clicked, unlike every button).
pub fn sideways(ctx: &egui::Context) -> bool {
    let Some(f) = ctx.memory(|m| m.focused()) else { return false };
    SIDEWAYS.with(|s| s.get()) == Some(f) || ctx.viewport(|v| v.prev_pass.widgets.get(f).is_some_and(|w| w.sense.senses_drag() && !w.sense.senses_click()))
}

/// the virtual cursor: an arrow over everything, its tip at `at`.
pub fn cursor(ctx: &egui::Context, at: Pos2) {
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Debug, egui::Id::unique("pad_cursor")));
    let pts: Vec<Pos2> = [(0.0, 0.0), (0.0, 19.0), (4.6, 14.8), (7.8, 21.6), (10.6, 20.4), (7.4, 13.6), (13.4, 13.6)].iter().map(|&(x, y)| at + vec2(x, y)).collect();
    p.add(egui::Shape::convex_polygon(pts.iter().map(|q| *q + vec2(1.0, 1.5)).collect(), Color32::from_black_alpha(110), Stroke::NONE));
    p.add(egui::Shape::closed_line(pts.clone(), Stroke::new(2.4, Color32::from_rgb(10, 12, 16))));
    p.add(egui::Shape::convex_polygon(pts, Color32::WHITE, Stroke::NONE));
}

/// A clear ring round the widget the gamepad has focused, over everything.
pub fn focus_ring(ctx: &egui::Context, col: Color32) {
    if !active() {
        return;
    }
    let Some(id) = ctx.memory(|m| m.focused()) else { return };
    let Some(r) = ctx.read_response(id) else { return };
    let rect = r.interact_rect;
    if rect.width() < 2.0 || rect.height() < 2.0 {
        return;
    }
    // on the widget's own edge, where its lit border already is, so the two read as one.
    let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::unique("pad_focus")));
    p.rect_stroke(rect, crate::menu::R_WIDGET, Stroke::new(2.0, col), egui::StrokeKind::Inside);
}

/// The scroll areas and the gamepad. The right stick's scrolling: how far
/// this frame asks for (points, + moves the content down, showing what is
/// above), and which area takes it: the one holding the focused widget, else
/// the largest in view, as seen on the previous pass (an area only knows
/// what it holds once its contents are laid out). And the widget each area
/// last brought into view, to follow the focus as the d-pad moves it.
struct Scroll {
    amount: f32,
    target: Option<egui::Id>,
    /// this pass's areas: (id, visible area, holds the focus)
    seen: Vec<(egui::Id, f32, bool)>,
    /// (area, the focused widget it last scrolled to)
    followed: Vec<(egui::Id, egui::Id)>,
}

thread_local! {
    static SCROLL: std::cell::RefCell<Scroll> = const { std::cell::RefCell::new(Scroll { amount: 0.0, target: None, seen: Vec::new(), followed: Vec::new() }) };
}

/// The right stick's scroll for this frame (points; 0 when it is centred).
pub fn set_scroll(points: f32) {
    SCROLL.with(|s| s.borrow_mut().amount = points);
}

/// At the start of a UI pass: settle which scroll area the stick works.
pub fn begin_pass() {
    SCROLL.with(|s| {
        let s = &mut *s.borrow_mut();
        s.target = s.seen.iter().find(|a| a.2).or_else(|| s.seen.iter().max_by(|a, b| a.1.total_cmp(&b.1))).map(|a| a.0);
        s.seen.clear();
    });
}

/// At the end of a scroll area's contents: scroll a widget the gamepad (or
/// the keyboard) has moved the focus to into view, and let the right stick
/// scroll the area (pages of text have nothing for the d-pad to move to).
///
/// The move is noticed by the focused widget changing, not by
/// `Response::gained_focus`: egui moves the focus at the end of a pass, so
/// by the next one that already reads false.
pub fn follow_focus(ui: &Ui) {
    let ctx = ui.ctx();
    let focus = ctx.memory(|m| m.focused()).and_then(|id| ctx.read_response(id));
    let held = focus.filter(|r| r.layer_id == ui.layer_id() && ui.min_rect().contains_rect(r.rect));
    let area = ui.scope_id();
    let shown = ui.clip_rect().intersect(ui.min_rect());
    let vis = if shown.is_positive() { shown.area() } else { 0.0 };
    SCROLL.with(|s| {
        let s = &mut *s.borrow_mut();
        let last = s.followed.iter().position(|f| f.0 == area);
        match (&held, last) {
            (Some(r), Some(k)) if s.followed[k].1 == r.id => {}
            (Some(r), _) => {
                ui.scroll_to_rect(r.rect.expand(8.0), None);
                match last {
                    Some(k) => s.followed[k].1 = r.id,
                    None => s.followed.push((area, r.id)),
                }
            }
            // the focus left: coming back to the same widget scrolls to it again
            (None, Some(k)) => {
                s.followed.swap_remove(k);
            }
            (None, None) => {}
        }
        if !active() {
            return;
        }
        s.seen.push((area, vis, held.is_some()));
        if s.target == Some(area) && s.amount != 0.0 {
            ui.scroll_with_delta_animation(vec2(0.0, s.amount), egui::style::ScrollAnimation::none());
            s.amount = 0.0;
            ctx.request_repaint();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, Id, LayerId, Order};

    fn spot(n: &str, x0: f32, y0: f32, x1: f32, y1: f32, layer: &str) -> Spot {
        Spot { id: Id::new(n), rect: Rect::from_min_max(pos2(x0, y0), pos2(x1, y1)), layer: LayerId::new(Order::Middle, Id::new(layer)), shown: true }
    }

    fn go(all: &[Spot], from: &str, d: Dir) -> Option<Id> {
        let f = all.iter().find(|s| s.id == Id::new(from)).unwrap();
        next(all, f, d)
    }

    /// a settings page: a sidebar, and rows with controls of every width at the right.
    fn form() -> Vec<Spot> {
        vec![
            spot("general", 16.0, 196.0, 260.0, 240.0, "side"),
            spot("graphics", 16.0, 244.0, 260.0, 288.0, "side"),
            spot("english", 976.0, 170.0, 1060.0, 194.0, "page"),
            spot("fullscreen", 1160.0, 296.0, 1230.0, 322.0, "page"),
            spot("hud", 1160.0, 418.0, 1230.0, 444.0, "page"),
            spot("size", 860.0, 596.0, 1150.0, 622.0, "page"),
        ]
    }

    #[test]
    fn down_takes_the_next_row_not_the_next_thing_under_it() {
        let f = form();
        assert_eq!(go(&f, "english", Dir::Down), Some(Id::new("fullscreen")));
        assert_eq!(go(&f, "hud", Dir::Down), Some(Id::new("size")));
        assert_eq!(go(&f, "size", Dir::Up), Some(Id::new("hud")));
    }

    #[test]
    fn across_goes_to_the_row_at_that_height() {
        let f = form();
        assert_eq!(go(&f, "graphics", Dir::Right), Some(Id::new("fullscreen")));
        assert_eq!(go(&f, "fullscreen", Dir::Left), Some(Id::new("graphics")));
        // the sidebar keeps its own order.
        assert_eq!(go(&f, "general", Dir::Down), Some(Id::new("graphics")));
        assert_eq!(go(&f, "fullscreen", Dir::Right), None);
    }

    #[test]
    fn across_stays_on_its_line_before_going_off_it() {
        let m = vec![spot("acts", 88.0, 636.0, 436.0, 684.0, "m"), spot("quit", 20.0, 846.0, 136.0, 880.0, "m"), spot("credits", 1500.0, 850.0, 1580.0, 876.0, "m")];
        assert_eq!(go(&m, "quit", Dir::Right), Some(Id::new("credits")));
    }

    #[test]
    fn off_to_one_side_is_not_that_way() {
        // the main menu: the aircraft tile, quit far under it and a little to the left.
        let m = vec![spot("air", 88.0, 222.0, 436.0, 610.0, "m"), spot("quit", 20.0, 846.0, 136.0, 880.0, "m")];
        assert_eq!(go(&m, "air", Dir::Left), None);
        assert_eq!(go(&m, "air", Dir::Down), Some(Id::new("quit")));
    }

    #[test]
    fn a_box_round_the_widget_is_never_a_step() {
        let m = vec![spot("card", 200.0, 200.0, 600.0, 500.0, "m"), spot("back", 0.0, 0.0, 1600.0, 900.0, "m")];
        for d in [Dir::Up, Dir::Down, Dir::Left, Dir::Right] {
            assert_eq!(go(&m, "card", d), None);
        }
    }

    #[test]
    fn home_is_the_first_on_screen_in_the_top_layer() {
        let mut a = spot("under", 10.0, 10.0, 50.0, 50.0, "u");
        a.layer = LayerId::new(Order::Background, Id::new("u"));
        let mut b = spot("second", 400.0, 300.0, 500.0, 340.0, "o");
        b.layer = LayerId::new(Order::Foreground, Id::new("o"));
        let mut c = spot("first", 100.0, 300.0, 200.0, 340.0, "o");
        c.layer = b.layer;
        assert_eq!(first(&[a, b, c], 0.0), Some(Id::new("first")));
        // back, up in the header, is not where a screen starts.
        let mut back = spot("back", 10.0, 20.0, 80.0, 50.0, "o");
        back.layer = b.layer;
        assert_eq!(first(&[back, b, c], 100.0), Some(Id::new("first")));
        assert_eq!(first(&[back], 100.0), Some(Id::new("back")));
    }
}
