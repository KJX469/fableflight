//! the main menu: the wordmark over the aircraft, fly and settings tiles.
//! fly asks free flight or career. Quit sits small in the bottom left corner. Behind it the renderer draws
//! space alone (the menu camera looks away from the planet, see `frame`).
use crate::menu::{self, Screen, ACCENT, DIM, FAINT, TEXT};
use crate::padui;
use crate::{aircraft, hangar_ui, settings_ui, t, tr_en, Game};
use egui::{pos2, vec2, Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke};

/// The picture behind the FLY tile, 4:3 at a little over twice the tile's
/// size: the game's own globe (sky.wgsl `globe_*`, live clouds) rendered
/// from 7600 km over the Caribbean (`--menu-view 24,-66,7600000 --time 8.35
/// --day 80`, exposure 0.3, a 3840x2160 frame with the menu hidden), cropped
/// so the planet rises off the right of the tile with the morning terminator
/// and the US's city lights along its left edge, the dark space to its left
/// left for the title. Swap the file to change it; nothing else depends on
/// its size beyond the aspect (it is cropped to fill the tile).
pub const GLOBE_PICTURE: &[u8] = include_bytes!("../assets/ui/globe.jpg");

/// the pictures behind fly's two choices: the game's own scenery, no aircraft
/// (`--no-aircraft --weather fair --tail-view`, 1600 x 1200 scaled to 1200 x 900).
/// free flight: the golden gate in the afternoon (`--lat 37.79 --lon -122.52
/// --alt 500 --heading 70 --time 16`). career: the bay at sunrise with san
/// francisco's runways (`--lat 37.56 --lon -122.30 --alt 700 --heading 330 --time 7.3`).
pub const FREE_PICTURE: &[u8] = include_bytes!("../assets/ui/free.jpg");
pub const CAREER_PICTURE: &[u8] = include_bytes!("../assets/ui/career.jpg");

/// What a tile does when pressed.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Tile {
    Aircraft,
    Fly,
    Settings,
    /// the globe, to start anywhere.
    Free,
    /// the settings' Multiplayer section
    Multiplayer,
    /// the pilot's profile and logbook.
    Logbook,
    /// the career.
    Career,
    /// tutorials, checkrides, challenges and missions.
    Activities,
}

impl Game {
    /// The main menu (see the module docs).
    pub(crate) fn main_menu(&mut self, ctx: &egui::Context) {
        let screen = ctx.content_rect();
        let globe = self.picture(ctx, GLOBE_PICTURE);
        let free_pic = self.picture(ctx, FREE_PICTURE);
        let career_pic = self.picture(ctx, CAREER_PICTURE);
        let spec = aircraft::nth(self.menu.aircraft);
        let still = self.still(ctx, spec);
        let mut press: Option<Tile> = None;
        let mut quit = false;
        let rise = entrance(ctx);
        let picking = self.menu.fly_pick;
        egui::Area::new(egui::Id::unique("main_menu")).order(egui::Order::Background).fixed_pos(screen.min).show(ctx, |ui| {
            let p = ui.painter().clone();
            // Darken space towards the foot of the screen, for depth.
            gradient(&p, Rect::from_min_max(pos2(screen.left(), screen.center().y), screen.max), Color32::TRANSPARENT, Color32::from_rgba_premultiplied(4, 6, 10, 200));

            // The tiles: FLY in the middle and larger, the other two beside it.
            let gap = 18.0;
            let (fw, fh, sw, sh) = (600.0, 450.0, 300.0, 400.0);
            let total = fw + 2.0 * sw + 2.0 * gap;
            let top = 104.0;
            let foot = 120.0;
            let k = ((screen.width() - 48.0) / total).min((screen.height() - top - foot) / fh).min(1.1).max(0.45);
            let (fw, fh, sw, sh, gap) = (fw * k, fh * k, sw * k, sh * k, gap * k);
            let cy = top + (screen.height() - top - foot) * 0.5 + 10.0;
            let x0 = screen.center().x - (fw + 2.0 * sw + 2.0 * gap) * 0.5;
            // the tiles rise in as the menu opens.
            let lift = |r: Rect, i: usize| r.translate(vec2(0.0, 26.0 * (1.0 - rise(i))));
            let bar_h = 46.0;
            // the aircraft, and the logbook as a bar under it, mirroring settings and multiplayer.
            let air = lift(Rect::from_min_size(pos2(x0, cy - sh * 0.5), vec2(sw, sh - bar_h - gap)), 1);
            let lbb = lift(Rect::from_min_size(pos2(x0, air.bottom() + gap), vec2(sw, bar_h)), 3);
            let fly = lift(Rect::from_min_size(pos2(x0 + sw + gap, cy - fh * 0.5), vec2(fw, fh)), 0);
            // Settings, and Multiplayer as a bar under it.
            let set = lift(Rect::from_min_size(pos2(x0 + sw + gap + fw + gap, cy - sh * 0.5), vec2(sw, sh - bar_h - gap)), 2);
            let mpb = lift(Rect::from_min_size(pos2(set.left(), set.bottom() + gap), vec2(sw, bar_h)), 3);
            let mut pw = p.clone();
            pw.set_opacity(rise(0));
            // the pilot, at the right end of the wordmark's line.
            let right = x0 + fw + 2.0 * sw + 2.0 * gap;
            let base = fly.top() - 26.0 * k.max(0.7);
            let chip = Rect::from_min_max(pos2(right - 290.0, base - 58.0), pos2(right, base + 4.0));
            wordmark(&pw, pos2(x0, base), chip.left() - 20.0, k);
            let r = ui.interact(chip, ui.scope_id().with("tile_pilot"), Sense::click());
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tr_en("Pilot")));
            let lvl = crate::pilot::level(self.profile.xp);
            let name = if self.profile.name.trim().is_empty() { tr_en("Pilot").to_string() } else { self.profile.name.clone() };
            pilot_chip(&pw, chip, &r, &name, lvl);
            if r.clicked() {
                press = Some(Tile::Logbook);
            }

            // Tab and the d-pad go left to right: Aircraft, FLY, Settings.
            let r = ui.interact(air, ui.scope_id().with("tile_air"), Sense::click());
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, t!("menu.aircraft")));
            let mut pa = p.clone();
            pa.set_opacity(rise(1));
            aircraft_tile(&pa, air, &r, k, &still);
            if r.clicked() {
                press = Some(Tile::Aircraft);
            }
            let r = ui.interact(fly, ui.scope_id().with("tile_fly"), Sense::click());
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, t!("menu.fly")));
            // (while fly asks, its first card is the one picked out.)
            if !picking {
                self.pad_home_focus(&r);
            }
            let mut pf = p.clone();
            pf.set_opacity(rise(0));
            fly_tile(&pf, fly, &r, k, &globe);
            if r.clicked() {
                press = Some(Tile::Fly);
            }
            let r = ui.interact(set, ui.scope_id().with("tile_set"), Sense::click());
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, t!("menu.settings")));
            let mut ps = p.clone();
            ps.set_opacity(rise(2));
            settings_tile(&ps, set, &r, k);
            if r.clicked() {
                press = Some(Tile::Settings);
            }
            let r = ui.interact(mpb, ui.scope_id().with("tile_mp"), Sense::click());
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, t!("menu.multiplayer")));
            let mut pm = p.clone();
            pm.set_opacity(rise(3));
            let (col, status) = self.mp_short();
            mp_bar(&pm, mpb, &r, &status, col);
            if r.clicked() {
                press = Some(Tile::Multiplayer);
            }

            let r = ui.interact(lbb, ui.scope_id().with("tile_acts"), Sense::click());
            r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tr_en("Activities")));
            let mut pl = p.clone();
            pl.set_opacity(rise(3));
            // the activities only: the career's lessons and checkrides are its own.
            let acts = || crate::missions::ALL.iter().filter(|m| matches!(m.cat, crate::missions::Cat::Tutorial | crate::missions::Cat::Mission));
            let all = acts().count() + crate::challenge::ALL.len();
            let flown = acts().filter(|m| self.profile.best.contains_key(m.id)).count() + crate::challenge::ALL.iter().filter(|c| self.challenge_records.top(c.id).is_some()).count();
            activities_bar(&pl, lbb, &r, &format!("{flown} / {all} {}", tr_en("flown")));
            if r.clicked() {
                press = Some(Tile::Activities);
            }

            if picking {
                // fly: free flight or career, over the dimmed menu.
                let a = ctx.animate_bool_with_time(egui::Id::unique("fly_pick_fade"), true, 0.18);
                let fg = egui::Painter::new(ctx.clone(), egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("fly_pick")), screen);
                fg.rect_filled(screen, 0.0, Color32::from_black_alpha((236.0 * a) as u8));
                let (cw, ch, cg) = (460.0 * k.max(0.7), 320.0 * k.max(0.7), 24.0 * k.max(0.7));
                let y0 = screen.center().y - ch * 0.5 + 10.0 * (1.0 - a);
                let left = Rect::from_min_size(pos2(screen.center().x - cw - cg * 0.5, y0), vec2(cw, ch));
                let right = Rect::from_min_size(pos2(screen.center().x + cg * 0.5, y0), vec2(cw, ch));
                let over = egui::Ui::new(ctx.clone(), egui::Id::unique("fly_pick_ui"), egui::UiBuilder::new().layer_id(fg.layer_id()).max_rect(screen));
                // (the cards go over it, so they take their own clicks)
                let back = over.interact(screen, egui::Id::unique("pick_back"), Sense::click());
                let r = over.interact(left, egui::Id::unique("pick_free"), Sense::click());
                r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tr_en("Free flight")));
                self.pad_home_focus(&r);
                let mut pf = fg.clone();
                pf.set_opacity(a);
                photo_card(&pf, left, &r, k, &free_pic, tr_en("Free flight"), "");
                if r.clicked() {
                    press = Some(Tile::Free);
                }
                let r = over.interact(right, egui::Id::unique("pick_career"), Sense::click());
                r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tr_en("Career")));
                photo_card(&pf, right, &r, k, &career_pic, tr_en("Career"), "");
                if r.clicked() {
                    press = Some(Tile::Career);
                }
                // a click beside the cards, or Esc, goes back.
                if press.is_none() && (back.clicked() || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))) {
                    self.menu.fly_pick = false;
                }
                let hy = left.bottom() + 40.0;
                let g = fg.layout_no_wrap(tr_en("Back").to_string(), menu::label(), DIM.gamma_multiply(a));
                let cap = fg.layout_no_wrap("Esc".to_string(), FontId::monospace(11.5), DIM.gamma_multiply(a));
                let total = cap.size().x + 14.0 + 8.0 + g.size().x;
                let x = screen.center().x - total * 0.5;
                let kr = Rect::from_min_size(pos2(x, hy - 11.0), vec2(cap.size().x + 14.0, 22.0));
                fg.rect_stroke(kr, menu::R_WIDGET, Stroke::new(1.0, Color32::from_white_alpha((46.0 * a) as u8)), egui::StrokeKind::Inside);
                fg.galley(kr.center() - cap.size() * 0.5, cap, DIM);
                fg.galley(pos2(kr.right() + 8.0, hy - g.size().y * 0.5), g, DIM);
            }

            // The foot: Quit at the left, the gamepad's buttons (or a word on the keys) in the middle.
            let foot_y = screen.bottom() - 34.0;
            let q = Rect::from_min_size(pos2(screen.left() + 24.0, foot_y - 17.0), vec2(104.0, 34.0));
            let qr = ui.interact(q, ui.scope_id().with("quit"), Sense::click());
            qr.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, t!("menu.quit")));
            let lit = menu::ease(ctx, qr.id, qr.hovered() || qr.has_focus());
            if lit > 0.0 {
                p.rect_filled(q, menu::R_WIDGET, Color32::from_white_alpha((14.0 * lit) as u8));
            }
            let qc = menu::mix(DIM, Color32::WHITE, lit);
            power_icon(&p, pos2(q.left() + 18.0, q.center().y), 14.0, qc);
            p.text(pos2(q.left() + 34.0, q.center().y), Align2::LEFT_CENTER, t!("menu.quit"), menu::medium(15.0), qc);
            if qr.clicked() {
                quit = true;
            }
            if padui::active() {
                let mut f = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_center_size(pos2(screen.center().x, foot_y), vec2(360.0, 28.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
                padui::prompts(&mut f, &[("Dpad", tr_en("move")), ("A", tr_en("press"))], 20.0, 12.5, DIM);
            }
            // the picture's credits behind a small word, not a line across the foot.
            let cr = p.text(pos2(screen.right() - 24.0, foot_y), Align2::RIGHT_CENTER, tr_en("Image credits"), FontId::proportional(11.5), FAINT);
            ui.interact(cr, ui.scope_id().with("credits"), Sense::hover()).on_hover_text(t!("menu.globe_esri_maxar_earthstar_geographics"));
        });
        // while fly asks, only its two cards answer.
        let press = if picking { press.filter(|t| matches!(t, Tile::Free | Tile::Career)) } else { press };
        match press {
            Some(Tile::Fly) => self.menu.fly_pick = true,
            Some(Tile::Free) => {
                self.menu.fly_pick = false;
                self.menu.screen = Screen::Fly;
            }
            Some(Tile::Aircraft) => self.open_aircraft(hangar_ui::Tab::Mine, Screen::Main),
            Some(Tile::Settings) => {
                self.menu.set_back = Screen::Main;
                self.menu.screen = Screen::Settings;
            }
            Some(Tile::Multiplayer) => self.open_settings(settings_ui::Section::Multiplayer, Screen::Main),
            Some(Tile::Logbook) => self.menu.screen = Screen::Logbook,
            Some(Tile::Career) => {
                self.menu.fly_pick = false;
                self.menu.screen = Screen::Career;
            }
            Some(Tile::Activities) => self.menu.screen = Screen::Activities,
            None => {}
        }
        if press.is_some() {
            self.pad_home = true;
        }
        if quit {
            self.quit_why = "quit from the main menu";
            self.quit = true;
        }
    }
}

/// how far each tile has risen since the menu opened, 0..1.
fn entrance(ctx: &egui::Context) -> impl Fn(usize) -> f32 {
    let now = ctx.input(|i| i.time);
    let id = egui::Id::unique("main_menu_entrance");
    let (start, seen) = ctx.data(|d| d.get_temp::<(f64, f64)>(id)).unwrap_or((now, f64::NEG_INFINITY));
    // away for a moment: play it again.
    let start = if now - seen > 0.3 { now } else { start };
    ctx.data_mut(|d| d.insert_temp(id, (start, now)));
    let t = (now - start) as f32;
    if t < 1.0 {
        ctx.request_repaint();
    }
    move |i: usize| {
        let x = ((t - 0.07 * i as f32) / 0.55).clamp(0.0, 1.0);
        1.0 - (1.0 - x).powi(4)
    }
}

/// Multiplayer: a slim bar under the Settings tile, its two people and
/// where it stands (a coloured dot and a few words).
fn mp_bar(p: &Painter, r: Rect, resp: &egui::Response, status: &str, col: Color32) {
    let lit = menu::ease(p.ctx(), resp.id, resp.hovered() || resp.has_focus());
    p.rect_filled(r, menu::R_WIDGET, menu::mix(Color32::from_rgba_premultiplied(12, 16, 22, 226), Color32::from_rgba_premultiplied(26, 32, 42, 238), lit));
    p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, menu::mix(Color32::from_white_alpha(26), ACCENT, lit)), egui::StrokeKind::Inside);
    let ic = menu::mix(TEXT, ACCENT, lit);
    settings_ui::Section::Multiplayer.icon(p, pos2(r.left() + 24.0, r.center().y), 18.0, ic);
    p.text(pos2(r.left() + 44.0, r.center().y), Align2::LEFT_CENTER, t!("menu.multiplayer"), menu::label(), menu::mix(TEXT, Color32::WHITE, lit));
    let g = p.layout_no_wrap(status.to_string(), menu::caption(), DIM);
    // (a long status, an error say, is cut to the room left)
    let room = r.width() - 44.0 - 120.0;
    let x = r.right() - 14.0 - g.size().x.min(room);
    let clip = p.with_clip_rect(Rect::from_min_max(pos2(x, r.top()), pos2(r.right() - 14.0, r.bottom())));
    clip.galley(pos2(x, r.center().y - g.size().y * 0.5), g, DIM);
    p.circle_filled(pos2(x - 10.0, r.center().y), 3.5, col);
}

/// activities: a slim bar under the aircraft tile, a flag and how many are flown.
fn activities_bar(p: &Painter, r: Rect, resp: &egui::Response, count: &str) {
    let lit = menu::ease(p.ctx(), resp.id, resp.hovered() || resp.has_focus());
    p.rect_filled(r, menu::R_WIDGET, menu::mix(Color32::from_rgba_premultiplied(12, 16, 22, 226), Color32::from_rgba_premultiplied(26, 32, 42, 238), lit));
    p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, menu::mix(Color32::from_white_alpha(26), ACCENT, lit)), egui::StrokeKind::Inside);
    let ic = menu::mix(TEXT, ACCENT, lit);
    // a flag on a pole.
    let c = pos2(r.left() + 22.0, r.center().y);
    let st = Stroke::new(1.6, ic);
    p.line_segment([c + vec2(-5.0, -9.0), c + vec2(-5.0, 9.0)], st);
    p.line(vec![c + vec2(-5.0, -8.0), c + vec2(7.0, -5.0), c + vec2(-5.0, -1.0)], st);
    p.text(pos2(r.left() + 44.0, r.center().y), Align2::LEFT_CENTER, tr_en("Activities"), menu::label(), menu::mix(TEXT, Color32::WHITE, lit));
    let g = p.layout_no_wrap(count.to_string(), menu::caption(), DIM);
    p.galley(pos2(r.right() - 14.0 - g.size().x, r.center().y - g.size().y * 0.5), g, DIM);
}

/// one of fly's choices: its picture filling the card (easing in on hover, as
/// fly's own tile does), a shade along the foot, the word and the line under it.
fn photo_card(p: &Painter, r: Rect, resp: &egui::Response, k: f32, pic: &egui::TextureHandle, title: &str, sub: &str) {
    let lit = menu::ease(p.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let zoom = p.ctx().animate_bool_with_time(resp.id.with("zoom"), lit > 0.5, 0.9);
    p.rect_filled(r, menu::R_PANEL, Color32::BLACK);
    let z = 1.0 - 0.04 * (zoom * zoom * (3.0 - 2.0 * zoom));
    cover(p, r.shrink(1.0), pic, vec2(0.5, 0.55), z, menu::mix(Color32::from_gray(205), Color32::WHITE, lit));
    let inner = r.shrink(1.0);
    gradient(p, Rect::from_min_max(pos2(inner.left(), inner.top() + inner.height() * 0.45), inner.max), Color32::TRANSPARENT, Color32::from_black_alpha(215));
    tile_frame(p, r, lit, false);
    card_words(p, r, k, lit, title, sub);
}

/// a choice card's word and the line under it, bottom left.
fn card_words(p: &Painter, r: Rect, k: f32, lit: f32, title: &str, sub: &str) {
    let x = r.left() + 28.0 * k.max(0.7);
    let y = r.bottom() - if sub.is_empty() { 30.0 } else { 58.0 } * k.max(0.7);
    spaced(p, pos2(x - 2.0, y), Align2::LEFT_BOTTOM, &title.to_uppercase(), menu::bold(44.0 * k.max(0.7)), menu::mix(Color32::WHITE, ACCENT, lit), 2.5 * k);
    if !sub.is_empty() {
        p.text(pos2(x, y + 10.0), Align2::LEFT_TOP, sub, menu::medium(16.0), menu::mix(TEXT, Color32::WHITE, lit));
    }
}

/// a picture filling `r`, cropped to its aspect, pushed in by `zoom` (below 1) toward `focus`.
fn cover(p: &Painter, r: Rect, tex: &egui::TextureHandle, focus: egui::Vec2, zoom: f32, tint: Color32) {
    let [iw, ih] = tex.size().map(|v| v as f32);
    let (ta, ia) = (r.width() / r.height(), iw / ih);
    let uv = if ia > ta {
        let f = ta / ia;
        Rect::from_min_max(pos2(0.5 - f * 0.5, 0.0), pos2(0.5 + f * 0.5, 1.0))
    } else {
        let f = ia / ta;
        Rect::from_min_max(pos2(0.0, 0.5 - f * 0.5), pos2(1.0, 0.5 + f * 0.5))
    };
    let at = uv.lerp_inside(focus);
    let uv = Rect::from_min_max(at + (uv.min - at) * zoom, at + (uv.max - at) * zoom);
    p.image(tex.id(), r, uv, tint);
}

/// the pilot in the top right: the avatar in its level ring, the name, the rank and level.
fn pilot_chip(p: &Painter, r: Rect, resp: &egui::Response, name: &str, lvl: crate::pilot::Level) {
    let lit = menu::ease(p.ctx(), resp.id, resp.hovered() || resp.has_focus());
    if lit > 0.0 {
        p.rect_filled(r, menu::R_PANEL, Color32::from_white_alpha((10.0 * lit) as u8));
    }
    let c = pos2(r.right() - 34.0, r.center().y);
    crate::widgets::avatar(p, c, 22.0, name, lvl.frac());
    let x = c.x - 40.0;
    let g = menu::one_line(p, name, menu::semibold(18.0), Color32::WHITE, 190.0);
    p.galley(pos2(x - g.size().x, r.center().y - 22.0), g, Color32::WHITE);
    p.text(pos2(x, r.center().y + 4.0), Align2::RIGHT_CENTER, format!("{} {}  ·  {}", tr_en("Level"), lvl.n, tr_en(crate::pilot::rank(lvl.n))), menu::caption(), menu::mix(DIM, ACCENT, lit));
    let bar = Rect::from_min_size(pos2(x - 150.0, r.center().y + 16.0), vec2(150.0, 4.0));
    crate::widgets::meter(p, bar, lvl.frac(), ACCENT);
}

/// the wordmark at `at`, a hairline running on to `right`.
pub(crate) fn wordmark(p: &Painter, at: Pos2, right: f32, k: f32) {
    let size = 60.0 * k.max(0.7);
    let mut job = egui::text::LayoutJob::default();
    let fmt = |col| egui::TextFormat { font_id: menu::bold(size), color: col, extra_letter_spacing: 3.0 * k, ..Default::default() };
    job.append("FABLE ", 0.0, fmt(Color32::WHITE));
    job.append("FLIGHT", 0.0, fmt(ACCENT));
    let g = p.layout_job(job);
    let r = Align2::LEFT_BOTTOM.anchor_size(at, g.size());
    let w = g.size().x;
    p.galley(r.min, g, Color32::WHITE);
    // a hairline from the end of the word to the row's edge.
    let y = r.center().y + 2.0;
    if right - (r.left() + w + 24.0) > 40.0 {
        p.hline(r.left() + w + 20.0..=right, y, Stroke::new(1.0, Color32::from_white_alpha(46)));
    }
}

/// the frame every tile shares, its border lit on hover.
fn tile_frame(p: &Painter, r: Rect, lit: f32, fill: bool) {
    if fill {
        p.rect_filled(r, menu::R_PANEL, menu::mix(Color32::from_rgba_premultiplied(12, 16, 22, 226), Color32::from_rgba_premultiplied(22, 28, 37, 236), lit));
    }
    p.rect_stroke(r, menu::R_PANEL, Stroke::new(1.0 + 0.5 * lit, menu::mix(Color32::from_white_alpha(26), ACCENT, lit)), egui::StrokeKind::Inside);
}

/// a side tile's name and, when there is something to say, the line under it.
fn caption(p: &Painter, r: Rect, k: f32, lit: f32, title: &str, sub: &str) {
    let y = r.bottom() - if sub.is_empty() { 40.0 } else { 74.0 } * k.max(0.7);
    spaced(p, pos2(r.left() + 24.0 * k, y), Align2::LEFT_BOTTOM, &title.to_uppercase(), menu::bold(30.0 * k.max(0.7)), menu::mix(TEXT, Color32::WHITE, lit), 2.5 * k);
    if !sub.is_empty() {
        p.text(pos2(r.left() + 25.0 * k, y + 8.0), Align2::LEFT_TOP, sub, menu::label(), menu::mix(DIM, TEXT, lit));
    }
}

/// the aircraft tile: your aircraft, easing forward on hover.
fn aircraft_tile(p: &Painter, r: Rect, resp: &egui::Response, k: f32, still: &egui::TextureHandle) {
    let lit = menu::ease(p.ctx(), resp.id, resp.hovered() || resp.has_focus());
    tile_frame(p, r, lit, true);
    // a pool of light under it.
    let c = pos2(r.center().x, r.top() + r.height() * 0.38);
    let clip = p.with_clip_rect(r.shrink(1.0));
    menu::glow(&clip, c + vec2(0.0, 30.0 * k), vec2(r.width() * 0.62, r.width() * 0.36), Color32::from_white_alpha((14.0 + 8.0 * lit) as u8));
    // the picture is 680 x 392 with the aircraft in the middle.
    let w = r.width() * 0.98;
    let pic = Rect::from_center_size(c + vec2(5.0 * lit, -4.0 * lit) * k, vec2(w, w * 392.0 / 680.0));
    clip.image(still.id(), pic, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    caption(p, r, k, lit, t!("menu.aircraft"), "");
}

/// the settings tile: a cog that turns on hover.
fn settings_tile(p: &Painter, r: Rect, resp: &egui::Response, k: f32) {
    let lit = menu::ease(p.ctx(), resp.id, resp.hovered() || resp.has_focus());
    tile_frame(p, r, lit, true);
    let c = pos2(r.center().x, r.top() + r.height() * 0.38);
    gear_icon_turned(p, c, 80.0 * k, menu::mix(Color32::from_rgb(150, 160, 174), Color32::WHITE, lit), lit * std::f32::consts::TAU / 16.0);
    caption(p, r, k, lit, t!("menu.settings"), "");
}

/// FLY: the globe picture filling it (easing in a little when hovered),
/// shades along the foot and the left for the words, and the word.
fn fly_tile(p: &Painter, r: Rect, resp: &egui::Response, k: f32, globe: &egui::TextureHandle) {
    let lit = menu::ease(p.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let zoom = p.ctx().animate_bool_with_time(resp.id.with("zoom"), lit > 0.5, 0.9);
    p.rect_filled(r, menu::R_PANEL, Color32::BLACK);
    // a slow push in toward the planet's lit face on hover (eased).
    let z = 1.0 - 0.035 * (zoom * zoom * (3.0 - 2.0 * zoom));
    cover(p, r.shrink(1.0), globe, vec2(0.72, 0.42), z, menu::mix(Color32::from_gray(222), Color32::WHITE, lit));
    // Shades for the words: across from the left, and up from the foot.
    let inner = r.shrink(1.0);
    hgradient(p, Rect::from_min_max(inner.min, pos2(inner.left() + inner.width() * 0.5, inner.bottom())), Color32::from_black_alpha(130), Color32::TRANSPARENT);
    gradient(p, Rect::from_min_max(pos2(inner.left(), inner.top() + inner.height() * 0.5), inner.max), Color32::TRANSPARENT, Color32::from_black_alpha(180));
    tile_frame(p, r, lit, false);
    let x = r.left() + 32.0 * k;
    let base = r.bottom() - 36.0 * k;
    spaced(p, pos2(x - 3.0, base), Align2::LEFT_BOTTOM, t!("menu.fly_2"), menu::bold(112.0 * k.max(0.6)), menu::mix(Color32::WHITE, ACCENT, lit), 5.0 * k);
}

/// Text with extra space between the letters.
pub(crate) fn spaced(p: &Painter, at: Pos2, align: Align2, text: &str, font: FontId, col: Color32, spacing: f32) {
    let mut job = egui::text::LayoutJob::default();
    job.append(text, 0.0, egui::TextFormat { font_id: font, color: col, extra_letter_spacing: spacing, ..Default::default() });
    let g = p.layout_job(job);
    // (the spacing trails the last letter too: leave it out of the centring)
    let size = g.size() - vec2(spacing, 0.0);
    let r = align.anchor_size(at, size);
    p.galley(r.min, g, col);
}

/// A horizontal gradient over a rectangle, `left` to `right`.
pub(crate) fn hgradient(p: &Painter, r: Rect, left: Color32, right: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(r.left_top(), left);
    mesh.colored_vertex(r.right_top(), right);
    mesh.colored_vertex(r.right_bottom(), right);
    mesh.colored_vertex(r.left_bottom(), left);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(mesh);
}

/// A vertical gradient over a rectangle.
pub(crate) fn gradient(p: &Painter, r: Rect, top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(r.left_top(), top);
    mesh.colored_vertex(r.right_top(), top);
    mesh.colored_vertex(r.right_bottom(), bottom);
    mesh.colored_vertex(r.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    p.add(mesh);
}

/// a cog turned by `turn` radians.
fn gear_icon_turned(p: &Painter, c: Pos2, s: f32, col: Color32, turn: f32) {
    let (r_out, r_in, teeth) = (s * 0.46, s * 0.36, 8);
    let mut pts = Vec::new();
    for i in 0..teeth {
        let a0 = turn + i as f32 / teeth as f32 * std::f32::consts::TAU;
        let step = std::f32::consts::TAU / teeth as f32;
        // root, rise, top across, fall
        for (f, r) in [(0.0, r_in), (0.18, r_in), (0.28, r_out), (0.62, r_out), (0.72, r_in)] {
            let a = a0 + f * step;
            pts.push(c + vec2(a.cos(), a.sin()) * r);
        }
    }
    let st = Stroke::new((s * 0.03).clamp(1.6, 3.0), col);
    p.add(egui::Shape::closed_line(pts, st));
    p.circle_stroke(c, s * 0.15, st);
}

/// The power symbol: a ring open at the top, a bar through the gap.
fn power_icon(p: &Painter, c: Pos2, s: f32, col: Color32) {
    let r = s * 0.45;
    let st = Stroke::new(1.6, col);
    let pts: Vec<Pos2> = (0..=20).map(|i| {
        let a = -std::f32::consts::FRAC_PI_2 + 0.65 + i as f32 / 20.0 * (std::f32::consts::TAU - 1.3);
        c + vec2(a.cos(), a.sin()) * r
    }).collect();
    p.line(pts, st);
    p.line_segment([c - vec2(0.0, r * 1.15), c - vec2(0.0, r * 0.2)], st);
}
