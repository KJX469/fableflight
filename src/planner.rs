//! the flight planner: from one airport and runway to another, a route made
//! from the real procedures and airways or imported from simbrief, its fixes
//! on the globe and in a navigation log.

use crate::flightplan::{self, Kind, Leg, Phase, Pick, Waypoint};
use crate::menu::{self, ACCENT, DIM, FAINT, TEXT};
use crate::padui::Hint;
use crate::{airports, navdata, route, simbrief, t, tr_en, Game, ROUTE_COLOUR};
use egui::{pos2, vec2, Align2, Color32, Rect, RichText, Sense, Stroke};
use std::sync::mpsc::Receiver;

/// sid legs on the globe and in the log.
pub const SID_COLOUR: Color32 = Color32::from_rgb(92, 222, 170);
/// star and approach legs.
pub const ARR_COLOUR: Color32 = Color32::from_rgb(118, 192, 255);

/// the colour a waypoint and the leg to it are drawn in.
pub fn wp_colour(w: &Waypoint) -> Color32 {
    match w.phase.as_ref().map(|p| p.0) {
        Some(Phase::Sid) => SID_COLOUR,
        Some(Phase::Star | Phase::App) => ARR_COLOUR,
        None => ROUTE_COLOUR,
    }
}

/// a waypoint's chart symbol: airports a ring, vors a hexagon, ndbs a dot in a
/// ring, fixes a triangle, user points a diamond, heading legs a small square.
pub fn symbol(p: &egui::Painter, c: egui::Pos2, kind: Kind, col: Color32, big: bool) {
    let s = if big { 7.0 } else { 5.5 };
    let shade = Stroke::new(3.5, Color32::from_black_alpha(150));
    let line = Stroke::new(1.8, col);
    let poly = |n: usize, r: f32, turn: f32| -> Vec<egui::Pos2> { (0..n).map(|i| c + egui::Vec2::angled(turn + i as f32 * std::f32::consts::TAU / n as f32) * r).collect() };
    for st in [shade, line] {
        match kind {
            Kind::Airport => {
                p.circle_stroke(c, s, st);
            }
            Kind::Vor => {
                p.add(egui::Shape::closed_line(poly(6, s, 0.0), st));
            }
            Kind::Ndb => {
                p.circle_stroke(c, s, st);
            }
            Kind::Fix => {
                p.add(egui::Shape::closed_line(poly(3, s * 1.05, -std::f32::consts::FRAC_PI_2), st));
            }
            Kind::User => {
                p.add(egui::Shape::closed_line(poly(4, s, 0.0), st));
            }
            Kind::Leg => {
                p.rect_stroke(egui::Rect::from_center_size(c, vec2(s, s) * 1.2), 1.0, st, egui::StrokeKind::Middle);
            }
        }
    }
    if matches!(kind, Kind::Vor | Kind::Ndb | Kind::Airport) {
        p.circle_filled(c, 1.8, col);
    }
}

#[derive(Default)]
pub struct Planner {
    pub open: bool,
    /// the arrival airport chosen, and its runway.
    pub to: Option<u32>,
    pub arr_rwy: Option<String>,
    from_text: String,
    to_text: String,
    /// which airport box is being typed in.
    editing: Option<bool>,
    pub sb_user: String,
    sb_rx: Option<Receiver<Result<simbrief::Ofp, String>>>,
    /// the plan last imported from simbrief.
    pub ofp: Option<simbrief::Ofp>,
    /// notes on the plan made, and what went wrong.
    notes: Vec<String>,
    err: String,
    route_edit: String,
    route_seen: String,
    /// turn the globe to the route next frame.
    frame: bool,
    /// clicks on the globe add waypoints.
    pub pick_on_globe: bool,
    /// show the top of the list next frame.
    top: bool,
    /// the simbrief tab rather than making the route here.
    simbrief_tab: bool,
    /// the route text being typed.
    route_editing: bool,
}

impl Planner {
    pub fn busy(&self) -> bool {
        self.sb_rx.is_some()
    }
}



impl Game {
    /// opens the planner on the globe.
    pub(crate) fn open_planner(&mut self) {
        self.planner.open = true;
        self.planner.frame = true;
        self.planner.top = true;
        self.planner.err.clear();
        if self.planner.sb_user.is_empty() {
            self.planner.sb_user = self.prefs_simbrief.clone();
        }
        if self.planner.to.is_none() {
            self.planner.to = self.plan.destination().and_then(|w| airports::Db::get().find(&w.ident));
        }
    }

    /// turns the globe so the whole route shows beside the planner.
    fn frame_route(&mut self) {
        let db = airports::Db::get();
        let mut pts: Vec<(f64, f64)> = self.plan.wps.iter().filter(|w| !w.is_heading()).map(|w| w.at()).collect();
        pts.extend(self.menu.pick);
        pts.extend(self.planner.to.map(|i| (db.apts[i as usize].lat, db.apts[i as usize].lon)));
        if pts.is_empty() {
            return;
        }
        let a = pts[0];
        let b = *pts.iter().max_by(|x, y| flightplan::dist(a, **x).total_cmp(&flightplan::dist(a, **y))).unwrap_or(&a);
        let mid = flightplan::between(a, b, 0.5);
        let span = pts.iter().map(|p| flightplan::dist(mid, *p)).fold(0.0, f64::max);
        // the planner covers the left of the screen: look a little left of the middle.
        let c = flightplan::ahead(mid, 270.0, span * 0.45);
        self.menu.look_at(c.0, c.1, (span * 2.8).clamp(40_000.0, 9.0e6));
    }

    /// an airport clicked on the globe: the departure first, then the arrival.
    pub(crate) fn planner_click_airport(&mut self, i: u32) {
        match (self.menu.airport, self.planner.to) {
            (None, _) => self.planner_set_from(i, None),
            (Some(f), _) if f == i => {}
            (Some(_), _) => {
                self.planner.to = Some(i);
                self.planner.arr_rwy = None;
            }
        }
        self.planner.editing = None;
    }

    /// sets the departure airport and its runway, as starting on that runway.
    fn planner_set_from(&mut self, i: u32, rwy: Option<String>) {
        self.choose_airport(i);
        self.menu.start = menu::Start::Runway;
        self.menu.runway = rwy;
        self.planner.from_text.clear();
    }

    /// builds the route from the two airports.
    pub(crate) fn planner_make(&mut self) {
        let (Some(from), Some(to)) = (self.menu.airport, self.planner.to) else { return };
        let (plan, made) = route::make(from, self.menu.runway.as_deref(), to, self.planner.arr_rwy.as_deref(), self.spec);
        self.plan = plan;
        self.menu.runway = made.dep_rwy;
        self.menu.start = menu::Start::Runway;
        self.planner.arr_rwy = made.arr_rwy;
        self.planner.notes = made.note;
        self.planner.err.clear();
        self.planner.ofp = None;
        self.planner.frame = true;
    }

    /// starts a fetch of the latest simbrief plan.
    pub(crate) fn planner_fetch(&mut self) {
        let who = self.planner.sb_user.trim().to_string();
        self.prefs_simbrief = who.clone();
        self.keep_settings(false);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(simbrief::fetch(&who));
        });
        self.planner.sb_rx = Some(rx);
        self.planner.err.clear();
    }

    /// puts an imported plan in place.
    fn planner_take(&mut self, o: simbrief::Ofp) {
        let db = airports::Db::get();
        let (Some(dep), Some(arr)) = (db.find(&o.dep), db.find(&o.arr)) else {
            self.planner.err = t!("plan.unknown_airport", dep = o.dep, arr = o.arr);
            return;
        };
        let ends = route::runway_ends(dep);
        let rwy = o.dep_rwy.clone().filter(|r| ends.iter().any(|e| &e.ident == r));
        self.planner_set_from(dep, rwy);
        self.planner.to = Some(arr);
        let a_ends = route::runway_ends(arr);
        self.planner.arr_rwy = o.arr_rwy.clone().filter(|r| a_ends.iter().any(|e| &e.ident == r));
        let mut plan = flightplan::Plan::new();
        plan.wps = o.fixes.clone();
        plan.wps.push(Waypoint::airport(arr));
        if o.cruise_ft > 0.0 {
            plan.cruise_ft = o.cruise_ft;
        }
        // an approach to the planned runway where there is one.
        if let Some(r) = &self.planner.arr_rwy {
            let last = plan.wps.iter().rev().find(|w| w.kind != Kind::Airport).map(|w| w.at()).unwrap_or((db.apts[arr as usize].lat, db.apts[arr as usize].lon));
            let star_end = plan.wps.iter().rev().find(|w| matches!(w.phase, Some((Phase::Star, _)))).map(|w| w.ident.clone());
            if let Some(app) = route::pick_app(&o.arr, r, last, star_end.as_deref()) {
                plan.set_proc(Phase::App, Some(app), None);
            }
        }
        self.plan = plan;
        // fuel and payload as planned, within what the aircraft takes.
        if let Some(f) = o.ramp_kg {
            self.menu.fuel = (f / self.spec.fuel_kg.max(1.0)).clamp(0.05, 1.0);
        }
        let most = self.spec.payload[1];
        if let (Some(p), true) = (o.payload_kg, most > 0.0) {
            self.menu.payload = Some((p / most).clamp(0.0, 1.0));
        }
        self.planner.notes.clear();
        let mine = self.spec.name;
        if !o.ac_type.is_empty() && !self.spec.keys.iter().any(|k| k.eq_ignore_ascii_case(&o.ac_type)) {
            self.planner.notes.push(t!("plan.planned_for_type", ty = o.ac_type, mine = mine));
        }
        self.planner.ofp = Some(o);
        self.planner.frame = true;
    }

    /// the planner sidebar, in place of the start screen's two panels.
    pub(crate) fn planner_ui(&mut self, ctx: &egui::Context, start: &mut bool) {
        if let Some(rx) = &self.planner.sb_rx {
            match rx.try_recv() {
                Ok(Ok(o)) => {
                    self.planner.sb_rx = None;
                    self.planner_take(o);
                }
                Ok(Err(e)) => {
                    self.planner.sb_rx = None;
                    self.planner.err = e;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint(),
                Err(_) => self.planner.sb_rx = None,
            }
        }
        if self.planner.frame {
            self.planner.frame = false;
            self.frame_route();
        }
        const M: f32 = 22.0;
        let screen = ctx.content_rect();
        let w = (screen.width() * 0.34).clamp(460.0, 540.0);
        let top = M + 34.0 + 10.0;
        let h = screen.bottom() - M - top;
        let mut close = false;
        egui::Area::new(egui::Id::unique("planner")).anchor(Align2::LEFT_TOP, [M, top]).show(ctx, |ui| {
            ui.set_max_height(h);
            menu::panel().inner_margin(egui::Margin::symmetric(24, 16)).show(ui, |ui| {
                ui.set_width(w - 40.0);
                // the title, the aircraft beside it, done on the right.
                ui.horizontal(|ui| {
                    ui.label(RichText::new(t!("plan.flight_planner")).font(menu::semibold(22.0)).color(Color32::WHITE));
                    ui.add_space(4.0);
                    ui.label(RichText::new(self.spec.name).font(menu::label()).color(DIM));
                    menu::at_right(ui, "planner done", |ui| {
                        if menu::choice(ui, t!("plan.done"), false, 80.0).clicked() {
                            close = true;
                        }
                    });
                });
                ui.add_space(12.0);
                // made here, or brought from simbrief.
                if let Some(k) = tabs(ui, &[t!("plan.tab_build"), "SimBrief"], usize::from(self.planner.simbrief_tab)) {
                    self.planner.simbrief_tab = k == 1;
                }
                ui.add_space(16.0);
                if !self.plan.is_empty() {
                    self.planner_summary(ui);
                    ui.add_space(16.0);
                }
                // what sits outside the scrolling part: the title, the tabs, the summary, the foot and the margins.
                let foot = if self.plan.is_empty() { 60.0 } else { 120.0 };
                let used = if self.plan.is_empty() { 140.0 } else { 250.0 };
                let mut area = egui::ScrollArea::vertical().max_height((h - foot - used).max(140.0)).auto_shrink([false, true]);
                if std::mem::take(&mut self.planner.top) {
                    area = area.vertical_scroll_offset(0.0);
                }
                let out = area.show(ui, |ui| {
                    ui.set_width(ui.available_width() - 10.0);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    if self.planner.simbrief_tab {
                        self.planner_simbrief(ui);
                    } else {
                        self.planner_trip(ui);
                        self.planner_source(ui);
                    }
                    if !self.plan.is_empty() {
                        gap(ui);
                        self.planner_procs(ui);
                        gap(ui);
                        self.planner_route_text(ui);
                        gap(ui);
                        self.planner_log(ui);
                    }
                    ui.add_space(6.0);
                });
                // more below: the cut edge fades into the panel instead of slicing a row in half.
                let view = out.inner_rect;
                if out.content_size.y - out.state.clamped_offset().y > view.height() + 1.0 {
                    let band = Rect::from_min_max(pos2(view.left(), view.bottom() - 28.0), view.max);
                    let mut m = egui::Mesh::default();
                    let (clear, solid) = (Color32::TRANSPARENT, menu::PANEL);
                    m.colored_vertex(band.left_top(), clear);
                    m.colored_vertex(band.right_top(), clear);
                    m.colored_vertex(band.right_bottom(), solid);
                    m.colored_vertex(band.left_bottom(), solid);
                    m.add_triangle(0, 1, 2);
                    m.add_triangle(0, 2, 3);
                    ui.painter().add(m);
                }
                ui.add_space(12.0);
                self.planner_foot(ui, start);
            });
        });
        if close {
            self.planner.open = false;
        }
    }

    /// the plan at a glance: from and to, then distance, time and level.
    fn planner_summary(&mut self, ui: &mut egui::Ui) {
        let db = airports::Db::get();
        let dep = self.menu.pick;
        let from = self.menu.airport.map_or("—".to_string(), |i| db.apts[i as usize].ident.clone());
        let to = self.plan.destination().map_or("—".to_string(), |w| w.ident.clone());
        let total = self.plan.total(dep);
        let gs = self.spec.cruise.max(20.0);
        egui::Frame::new().fill(Color32::from_white_alpha(6)).stroke(Stroke::new(1.0, Color32::from_white_alpha(16))).inner_margin(egui::Margin::same(16)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                ui.label(RichText::new(&from).font(menu::bold(30.0)).color(Color32::WHITE));
                // a drawn arrow between them.
                let (r, _) = ui.allocate_exact_size(vec2(28.0, 30.0), Sense::hover());
                let (a, b) = (pos2(r.left(), r.center().y), pos2(r.right(), r.center().y));
                ui.painter().line_segment([a, b], Stroke::new(2.0, ACCENT));
                ui.painter().line_segment([b, b + vec2(-6.0, -5.0)], Stroke::new(2.0, ACCENT));
                ui.painter().line_segment([b, b + vec2(-6.0, 5.0)], Stroke::new(2.0, ACCENT));
                ui.label(RichText::new(&to).font(menu::bold(30.0)).color(Color32::WHITE));
            });
            let facts = [format!("{:.0} nm", total / 1852.0), flightplan::hm(total / gs), level(self.plan.cruise_ft), format!("{} {}", self.plan.wps.len(), tr_en("fixes"))];
            ui.label(RichText::new(facts.join("   ·   ")).font(menu::label()).color(TEXT));
        });
    }

    /// from and to, one under the other, each with its runways.
    fn planner_trip(&mut self, ui: &mut egui::Ui) {
        let mut pick_from = None;
        let mut pick_to = None;
        let mut rw_from = None;
        let mut rw_to = None;
        for from in [true, false] {
            let cur = if from { self.menu.airport } else { self.planner.to };
            section_title(ui, if from { tr_en("From") } else { tr_en("To") });
            let pick = self.planner_airport_box(ui, cur, from);
            let rw = cur.and_then(|i| {
                let now = if from { self.menu.runway.clone() } else { self.planner.arr_rwy.clone() };
                self.planner_runways(ui, i, now.as_deref())
            });
            if from {
                (pick_from, rw_from) = (pick, rw);
            } else {
                (pick_to, rw_to) = (pick, rw);
            }
            ui.add_space(16.0);
        }
        if let Some(i) = pick_from {
            self.planner_set_from(i, None);
            self.planner.frame = true;
        }
        if let Some(i) = pick_to {
            self.planner.to = Some(i);
            self.planner.arr_rwy = None;
            self.planner.frame = true;
        }
        if let Some(r) = rw_from {
            self.menu.runway = Some(r.clone());
            self.menu.start = menu::Start::Runway;
            self.planner_retarget(Phase::Sid, &r);
        }
        if let Some(r) = rw_to {
            self.planner.arr_rwy = Some(r.clone());
            self.planner_retarget(Phase::App, &r);
        }
    }

    /// a runway changed under a plan: its procedures follow the new runway.
    fn planner_retarget(&mut self, phase: Phase, rwy: &str) {
        if self.plan.is_empty() {
            return;
        }
        let db = navdata::Db::get();
        let dep = self.menu.pick;
        match phase {
            Phase::Sid => {
                let Some(p) = self.plan.sid.clone() else { return };
                let rws = db.runway_trans(&p.apt, 'D', &p.name);
                let pick = if rws.is_empty() { Some(p) } else { navdata::trans_for(&rws, rwy).map(|r| Pick { rwy: Some(r), ..p }) };
                self.plan.set_proc(Phase::Sid, pick, dep);
            }
            _ => {
                let Some(apt) = self.plan.destination().map(|w| w.ident.clone()) else { return };
                if let Some(s) = self.plan.star.clone() {
                    let rws = db.runway_trans(&s.apt, 'E', &s.name);
                    if let Some(r) = navdata::trans_for(&rws, rwy) {
                        self.plan.set_proc(Phase::Star, Some(Pick { rwy: Some(r), ..s }), dep);
                    }
                }
                let from = self.plan.wps.iter().rev().find(|w| w.phase.is_none() && w.kind != Kind::Airport).map(|w| w.at()).or(dep).unwrap_or((0.0, 0.0));
                let app = route::pick_app(&apt, rwy, from, None);
                self.plan.set_proc(Phase::App, app, dep);
            }
        }
        self.planner.frame = true;
    }

    /// an airport: the one chosen as a row to click to change, or a search box with matches under it.
    fn planner_airport_box(&mut self, ui: &mut egui::Ui, cur: Option<u32>, from: bool) -> Option<u32> {
        let db = airports::Db::get();
        let editing = self.planner.editing == Some(from) || cur.is_none();
        if let (Some(i), false) = (cur, editing) {
            let a = &db.apts[i as usize];
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
            let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
            let p = ui.painter();
            p.rect_filled(r, menu::R_WIDGET, menu::mix(menu::RAISED, menu::RAISED_HI, lit));
            p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, menu::mix(menu::LINE_HI, Color32::from_white_alpha(60), lit)), egui::StrokeKind::Inside);
            let code = p.layout_no_wrap(a.ident.clone(), menu::semibold(16.0), Color32::WHITE);
            let change = p.layout_no_wrap(t!("plan.change").to_string(), menu::caption(), menu::mix(DIM, ACCENT, lit));
            let cw = code.size().x;
            p.galley(pos2(r.left() + 12.0, r.center().y - code.size().y * 0.5), code, Color32::WHITE);
            p.galley(pos2(r.right() - 12.0 - change.size().x, r.center().y - change.size().y * 0.5), change.clone(), DIM);
            let name = menu::one_line(p, &a.name, menu::caption(), TEXT, r.width() - cw - change.size().x - 48.0);
            p.galley(pos2(r.left() + 24.0 + cw, r.center().y - name.size().y * 0.5), name, TEXT);
            let resp = resp.on_hover_text(format!("{}\n{}, {}", a.name, a.city, a.country));
            if resp.clicked() {
                self.planner.editing = Some(from);
            }
            return None;
        }
        let text = if from { &mut self.planner.from_text } else { &mut self.planner.to_text };
        let hint = if from { t!("plan.from_hint") } else { t!("plan.to_hint") };
        let resp = ui.add(egui::TextEdit::singleline(text).desired_width(ui.available_width()).hint_text(hint).margin(vec2(10.0, 9.0)).font(menu::label()));
        // clicked away with nothing typed: back to the airport there was.
        if cur.is_some() && text.is_empty() && !resp.has_focus() && !resp.clicked() && ui.input(|i| i.pointer.any_click()) {
            self.planner.editing = None;
        }
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let q = text.trim().to_string();
        let mut picked = None;
        if q.len() >= 2 {
            let hits = db.search(&q, 5);
            if enter {
                picked = db.find(&q).or(hits.first().copied());
            }
            ui.add_space(4.0);
            for i in hits {
                let a = &db.apts[i as usize];
                let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
                let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
                let p = ui.painter();
                if lit > 0.0 {
                    p.rect_filled(r, menu::R_WIDGET, Color32::from_white_alpha((14.0 * lit) as u8));
                }
                let code = p.layout_no_wrap(a.ident.clone(), menu::semibold(14.5), Color32::WHITE);
                let cw = code.size().x.max(44.0);
                p.galley(pos2(r.left() + 10.0, r.center().y - code.size().y * 0.5), code, Color32::WHITE);
                let name = menu::one_line(p, &a.name, menu::caption(), DIM, r.width() - cw - 30.0);
                p.galley(pos2(r.left() + 20.0 + cw, r.center().y - name.size().y * 0.5), name, DIM);
                if resp.clicked() {
                    picked = Some(i);
                }
            }
        }
        if picked.is_some() {
            if from { self.planner.from_text.clear() } else { self.planner.to_text.clear() }
            self.planner.editing = None;
        }
        picked
    }

    /// the runways as their pairs of ends (10L | 28R), four to a row in cells of
    /// one width, so the columns line up whatever the airport. the end most into
    /// the wind carries a small windsock.
    fn planner_runways(&mut self, ui: &mut egui::Ui, apt: u32, cur: Option<&str>) -> Option<String> {
        let ends = route::runway_ends(apt);
        if ends.is_empty() {
            return None;
        }
        let wind = route::into_wind(&ends, self.wx.wind_from).map(|e| e.ident);
        let a = &airports::Db::get().apts[apt as usize];
        let mut pairs: Vec<Vec<&route::RwyEnd>> = Vec::new();
        for r in &a.rwys {
            let mut v: Vec<&route::RwyEnd> = r.ident.iter().filter_map(|id| ends.iter().find(|e| e.ident == id.trim())).collect();
            v.sort_by_key(|e| rwy_num(&e.ident));
            if !v.is_empty() && !pairs.iter().any(|p| p[0].ident == v[0].ident) {
                pairs.push(v);
            }
        }
        pairs.sort_by(|x, y| rwy_num(&x[0].ident).cmp(&rwy_num(&y[0].ident)).then(x[0].ident.cmp(&y[0].ident)));
        let mut out = None;
        let w = ui.available_width();
        let cw = ((w - RW_GAP * (RW_COLS as f32 - 1.0)) / RW_COLS as f32).floor();
        ui.add_space(8.0);
        for row in pairs.chunks(RW_COLS) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = RW_GAP;
                for pair in row {
                    let (cell, _) = ui.allocate_exact_size(vec2(cw, RW_H), Sense::hover());
                    let ew = cell.width() / pair.len() as f32;
                    for (k, e) in pair.iter().enumerate() {
                        let r = Rect::from_min_size(pos2(cell.left() + ew * k as f32, cell.top()), vec2(ew, RW_H));
                        let resp = ui.interact(r, ui.scope_id().with(("planner rwy", &e.ident)), Sense::click());
                        let on = cur == Some(e.ident.as_str());
                        let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
                        let p = ui.painter();
                        let round = match (pair.len(), k) {
                            (1, _) => egui::CornerRadius::same(menu::R_WIDGET),
                            (_, 0) => egui::CornerRadius { nw: menu::R_WIDGET, sw: menu::R_WIDGET, ne: 0, se: 0 },
                            _ => egui::CornerRadius { ne: menu::R_WIDGET, se: menu::R_WIDGET, nw: 0, sw: 0 },
                        };
                        p.rect_filled(r, round, if on { menu::CHOSEN } else { menu::mix(menu::RAISED, menu::RAISED_HI, lit) });
                        let col = if on { Color32::WHITE } else { menu::mix(TEXT, Color32::WHITE, lit) };
                        p.text(r.center(), Align2::CENTER_CENTER, &e.ident, menu::medium(14.5), col);
                        if wind.as_deref() == Some(e.ident.as_str()) {
                            let c = r.right_top() + vec2(-9.0, 9.0);
                            let wc = if on { ACCENT } else { menu::LINK };
                            p.add(egui::Shape::convex_polygon(vec![c + vec2(-4.0, -2.5), c + vec2(3.0, -1.2), c + vec2(3.0, 1.2), c + vec2(-4.0, 2.5)], wc, Stroke::NONE));
                        }
                        let mut tip = t!("plan.runway_tip", len = format!("{:.0}", e.len), crs = format!("{:03.0}", e.course));
                        if wind.as_deref() == Some(e.ident.as_str()) {
                            tip = format!("{tip}\n{}", tr_en("Into the wind."));
                        }
                        if resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            out = Some(e.ident.clone());
                        }
                    }
                    // one outline round the pair, a hairline between its ends, the chosen end outlined in the accent.
                    let p = ui.painter();
                    p.rect_stroke(cell, menu::R_WIDGET, Stroke::new(1.0, menu::LINE_HI), egui::StrokeKind::Inside);
                    if pair.len() == 2 {
                        p.vline(cell.left() + ew, (cell.top() + 6.0)..=(cell.bottom() - 6.0), Stroke::new(1.0, Color32::from_white_alpha(24)));
                    }
                    for (k, e) in pair.iter().enumerate() {
                        if cur == Some(e.ident.as_str()) {
                            let r = Rect::from_min_size(pos2(cell.left() + ew * k as f32, cell.top()), vec2(ew, RW_H));
                            p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, ACCENT), egui::StrokeKind::Inside);
                        }
                    }
                }
            });
            ui.add_space(RW_GAP);
        }
        out
    }

    /// making the plan from the two airports.
    fn planner_source(&mut self, ui: &mut egui::Ui) {
        let ready = self.menu.airport.is_some() && self.planner.to.is_some() && self.menu.airport != self.planner.to;
        if !ready {
            return;
        }
        if self.plan.is_empty() {
            if menu::wide_button(ui, t!("plan.create_flight_plan"), Hint::None, false, true).on_hover_text(t!("plan.create_hint")).clicked() {
                self.planner_make();
            }
        } else if crate::flightui::text_link(ui, t!("plan.create_it_again")).clicked() {
            self.planner_make();
        }
    }

    /// simbrief: username, import the latest plan, or plan a new one there.
    fn planner_simbrief(&mut self, ui: &mut egui::Ui) {
        let busy = self.planner.busy();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let resp = ui.add_enabled(!busy, egui::TextEdit::singleline(&mut self.planner.sb_user).desired_width(ui.available_width() - 128.0).hint_text(t!("plan.simbrief_user_hint")).margin(vec2(10.0, 8.0)).font(menu::label()));
            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let word = if busy { t!("plan.importing") } else { t!("plan.import") };
            let ok = !busy && !self.planner.sb_user.trim().is_empty();
            let go = ui.add_enabled_ui(ok, |ui| menu::choice(ui, word, false, 120.0)).inner.on_hover_text(t!("plan.import_hint")).clicked();
            if (go || enter) && ok {
                self.planner_fetch();
            }
        });
        ui.add_space(10.0);
        if !self.planner.err.is_empty() {
            ui.label(RichText::new(&self.planner.err).font(menu::caption()).color(Color32::from_rgb(255, 130, 90)));
            ui.add_space(6.0);
        }
        if let Some(o) = &self.planner.ofp {
            let mut bits = vec![format!("{} → {}", o.dep, o.arr)];
            if !o.callsign.is_empty() {
                bits.insert(0, o.callsign.clone());
            }
            if !o.ac_type.is_empty() {
                bits.push(o.ac_type.clone());
            }
            if let Some(d) = o.dist_nm {
                bits.push(format!("{d:.0} nm"));
            }
            if let Some(e) = o.ete {
                bits.push(flightplan::hm(e));
            }
            if let Some(a) = &o.altn {
                bits.push(format!("ALTN {a}"));
            }
            ui.label(RichText::new(bits.join("   ·   ")).font(menu::label()).color(Color32::WHITE));
            ui.add_space(6.0);
        }
        for n in &self.planner.notes {
            ui.label(RichText::new(n).font(menu::caption()).color(Color32::from_rgb(255, 190, 110)));
            ui.add_space(4.0);
        }
        let db = airports::Db::get();
        let (dep, arr) = (self.menu.airport.map(|i| db.apts[i as usize].ident.clone()), self.planner.to.map(|i| db.apts[i as usize].ident.clone()));
        let word = if dep.is_some() && arr.is_some() { t!("plan.plan_this_on_simbrief") } else { t!("plan.open_simbrief") };
        if link(ui, word) {
            let ty = self.spec.keys.first().map(|k| icao_type(k)).unwrap_or_default();
            let url = match (dep, arr) {
                (Some(a), Some(b)) => simbrief::dispatch_url(&a, &b, ty),
                _ => "https://dispatch.simbrief.com/options/new".into(),
            };
            simbrief::open_in_browser(&url);
        }
    }

    /// departure, arrival and approach procedures in one aligned grid.
    fn planner_procs(&mut self, ui: &mut egui::Ui) {
        let db = navdata::Db::get();
        let dep_apt = self.menu.airport.map(|i| airports::Db::get().apts[i as usize].ident.clone());
        let dest_apt = self.plan.destination().map(|w| w.ident.clone());
        let mut change: Option<(Phase, Option<Pick>)> = None;
        section_title(ui, t!("game.procedures"));
        {
            let rows = [(Phase::Sid, tr_en("Departure"), dep_apt.clone(), self.menu.runway.clone()), (Phase::Star, tr_en("Arrival"), dest_apt.clone(), self.planner.arr_rwy.clone()), (Phase::App, tr_en("Approach"), dest_apt.clone(), None)];
            for (k, (phase, tag, apt, rwy)) in rows.into_iter().enumerate() {
                if k > 0 {
                    ui.add_space(4.0);
                }
                let kind = match phase {
                    Phase::Sid => 'D',
                    Phase::Star => 'E',
                    Phase::App => 'F',
                };
                let cur = match phase {
                    Phase::Sid => self.plan.sid.clone(),
                    Phase::Star => self.plan.star.clone(),
                    Phase::App => self.plan.app.clone(),
                };
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.set_min_height(32.0);
                    label_col(ui, tag, phase_colour(phase));
                    let col_w = ((ui.available_width() - 6.0) * 0.5).floor();
                    let Some(apt) = apt else {
                        placeholder(ui, col_w, t!("game.none"));
                        placeholder(ui, col_w, "");
                        return;
                    };
                    // only the ones that serve the chosen runway.
                    let names: Vec<String> = db.names(&apt, kind).into_iter().filter(|n| match (&rwy, phase) {
                        (Some(r), Phase::Sid | Phase::Star) => {
                            let rws = db.runway_trans(&apt, kind, n);
                            rws.is_empty() || navdata::trans_for(&rws, r).is_some()
                        }
                        _ => true,
                    }).collect();
                    if names.is_empty() {
                        placeholder(ui, col_w, t!("game.none"));
                        placeholder(ui, col_w, "");
                        return;
                    }
                    let name_w = col_w;
                    let none = t!("game.none");
                    let sel = cur.as_ref().map(|p| navdata::label(kind, &p.name)).unwrap_or_else(|| none.to_string());
                    egui::ComboBox::from_id_salt(("pl_proc", tag)).width(name_w).selected_text(sel).show_ui(ui, |ui| {
                        if ui.selectable_label(cur.is_none(), none).clicked() {
                            change = Some((phase, None));
                        }
                        for n in &names {
                            if ui.selectable_label(cur.as_ref().is_some_and(|p| &p.name == n), navdata::label(kind, n)).clicked() {
                                let rws = db.runway_trans(&apt, kind, n);
                                let rw = rwy.as_deref().and_then(|r| navdata::trans_for(&rws, r)).or_else(|| rws.first().cloned());
                                change = Some((phase, Some(Pick { apt: apt.clone(), name: n.clone(), rwy: if phase == Phase::App { None } else { rw }, trans: None })));
                            }
                        }
                    });
                    let Some(p) = cur else {
                        placeholder(ui, col_w, "");
                        return;
                    };
                    let others = db.other_trans(&apt, kind, &p.name);
                    if others.is_empty() {
                        placeholder(ui, col_w, "");
                        return;
                    }
                    let none = if phase == Phase::App { t!("game.vectors") } else { t!("game.no_transition") };
                    egui::ComboBox::from_id_salt(("pl_tr", tag)).width(col_w).selected_text(p.trans.clone().unwrap_or_else(|| none.to_string())).show_ui(ui, |ui| {
                        if ui.selectable_label(p.trans.is_none(), none).clicked() {
                            change = Some((phase, Some(Pick { trans: None, ..p.clone() })));
                        }
                        for t in &others {
                            if ui.selectable_label(p.trans.as_ref() == Some(t), t).clicked() {
                                change = Some((phase, Some(Pick { trans: Some(t.clone()), ..p.clone() })));
                            }
                        }
                    });
                });
            }
        }
        if let Some((phase, pick)) = change {
            let dep = self.menu.pick;
            self.plan.set_proc(phase, pick, dep);
            self.planner.frame = true;
        }
    }

    /// the route as text; edit to retype the enroute part.
    fn planner_route_text(&mut self, ui: &mut egui::Ui) {
        let now = self.plan.route_text();
        if now != self.planner.route_seen && !self.planner.route_editing {
            self.planner.route_seen = now.clone();
            self.planner.route_edit = now.clone();
        }
        let mut copy = false;
        header(ui, t!("plan.route"), |ui| {
            copy = link(ui, t!("plan.copy"));
            if !self.planner.route_editing && link(ui, t!("plan.edit")) {
                self.planner.route_editing = true;
            }
        });
        let mut apply = false;
        card(ui, |ui| {
            if self.planner.route_editing {
                ui.add(egui::TextEdit::multiline(&mut self.planner.route_edit).desired_rows(3).desired_width(ui.available_width()).font(menu::label()).frame(egui::Frame::NONE));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    apply = menu::choice(ui, t!("plan.apply"), false, 90.0).clicked();
                    if menu::choice(ui, t!("plan.cancel"), false, 90.0).clicked() {
                        self.planner.route_editing = false;
                        self.planner.route_edit = self.planner.route_seen.clone();
                    }
                });
                ui.add_space(6.0);
                menu::hint(ui, t!("plan.route_hint"));
            } else {
                ui.add(egui::Label::new(RichText::new(&now).font(menu::label()).color(Color32::WHITE)).wrap());
            }
        });
        if copy {
            ui.ctx().copy_text(now);
        }
        if apply {
            // the enroute part is typed again; procedures and the destination stay.
            let dest = self.plan.dest_index();
            let keep: Vec<bool> = self.plan.wps.iter().enumerate().map(|(i, w)| w.phase.is_some() || Some(i) == dest).collect();
            let mut k = keep.into_iter();
            self.plan.wps.retain(|_| k.next().unwrap_or(true));
            let names: Vec<String> = [&self.plan.sid, &self.plan.star, &self.plan.app].into_iter().flatten().flat_map(|p| [p.name.clone(), format!("{}.{}", p.name, p.trans.clone().unwrap_or_default()), format!("{}.{}", p.trans.clone().unwrap_or_default(), p.name)]).collect();
            let text: Vec<&str> = self.planner.route_edit.split_whitespace().filter(|t| !names.iter().any(|n| n.eq_ignore_ascii_case(t))).collect();
            let bad = self.plan.add_route(&text.join(" "), self.menu.pick);
            self.planner.err = if bad.is_empty() { String::new() } else { t!("game.not_found", bad.join(", ")) };
            self.planner.route_editing = false;
            self.planner.route_seen.clear();
            self.planner.frame = true;
        }
    }

    /// the navigation log: each fix, how it is reached, course, distance, constraints.
    fn planner_log(&mut self, ui: &mut egui::Ui) {
        header(ui, t!("plan.navlog"), |ui| {
            let word = if self.planner.pick_on_globe { t!("plan.adding_on_globe") } else { t!("plan.add_on_globe") };
            if link(ui, word) {
                self.planner.pick_on_globe = !self.planner.pick_on_globe;
            }
        });
        let dep = self.menu.pick;
        let legs = self.plan.legs(dep);
        let gs = self.spec.cruise.max(20.0);
        let mut act: Option<(usize, i8)> = None;
        let mut look: Option<(f64, f64)> = None;
        card(ui, |ui| {
            let wd = ui.available_width();
            // fix, via, course, distance, constraint as shares of the width.
            let cols = [0.0, 22.0, wd * 0.25, wd * 0.52, wd * 0.64, wd * 0.76];
            let head = [t!("plan.col_fix"), t!("plan.col_via"), t!("plan.col_crs"), t!("plan.col_dist"), t!("plan.col_alt")];
            let (r, _) = ui.allocate_exact_size(vec2(wd, 20.0), Sense::hover());
            for (k, hd) in head.iter().enumerate() {
                ui.painter().text(pos2(r.left() + cols[k + 1], r.top() + 8.0), Align2::LEFT_CENTER, *hd, menu::caption(), FAINT);
            }
            let n = self.plan.wps.len();
            let mut run = 0.0;
            for i in 0..n {
                let wp = &self.plan.wps[i];
                let leg = if dep.is_some() { legs.get(i) } else { i.checked_sub(1).and_then(|k| legs.get(k)) };
                let (r, resp) = ui.allocate_exact_size(vec2(wd, 32.0), Sense::click());
                let p = ui.painter();
                p.hline(r.x_range(), r.top(), Stroke::new(1.0, Color32::from_white_alpha(10)));
                if resp.hovered() {
                    p.rect_filled(r, 3.0, Color32::from_white_alpha(9));
                }
                let col = wp_colour(wp);
                let y = r.center().y;
                symbol(p, pos2(r.left() + 7.0, y), wp.kind, col, false);
                p.text(pos2(r.left() + cols[1], y), Align2::LEFT_CENTER, &wp.ident, menu::semibold(14.5), Color32::WHITE);
                let via = match (&wp.phase, &wp.via) {
                    (Some((_, name)), _) => name.clone(),
                    (None, Some(v)) if v != "DCT" => v.clone(),
                    _ => "DCT".into(),
                };
                let vg = p.layout_no_wrap(via, menu::caption(), col);
                p.with_clip_rect(Rect::from_min_max(pos2(r.left() + cols[2], r.top()), pos2(r.left() + cols[3] - 6.0, r.bottom()))).galley(pos2(r.left() + cols[2], y - vg.size().y * 0.5), vg, col);
                match (wp.leg, leg) {
                    (Leg::Fix { .. }, Some(l)) => {
                        run += l.2;
                        p.text(pos2(r.left() + cols[3], y), Align2::LEFT_CENTER, format!("{:03.0}°", l.3), menu::label(), TEXT);
                        p.text(pos2(r.left() + cols[4], y), Align2::LEFT_CENTER, format!("{:.0}", l.2 / 1852.0), menu::label(), TEXT);
                    }
                    (Leg::Climb { course, .. } | Leg::Intercept { course } | Leg::Vectors { course }, _) => {
                        p.text(pos2(r.left() + cols[3], y), Align2::LEFT_CENTER, format!("H{course:03.0}"), menu::label(), TEXT);
                    }
                    _ => {}
                }
                let alt = wp.alt.map(|a| a.text()).unwrap_or_default();
                let spd = wp.speed.map(|s| format!("  {s:.0}K")).unwrap_or_default();
                p.text(pos2(r.left() + cols[5], y), Align2::LEFT_CENTER, format!("{alt}{spd}"), menu::caption(), ACCENT);
                let tip = format!("{}  ·  {}\n{}\n{}", wp.kind.tag(), wp.name, menu::latlon(wp.lat, wp.lon), t!("plan.row_tip", nm = format!("{:.0}", run / 1852.0), time = flightplan::hm(run / gs)));
                let resp = resp.on_hover_text(tip);
                if resp.hovered() {
                    let bw = 24.0;
                    let mut x = r.right() - bw;
                    for (k, turn, en) in [(0i8, 0.0f32, true), (1, 1.0, i + 1 < n), (-1, -1.0, i > 0)] {
                        let b = Rect::from_min_size(pos2(x, r.top() + 3.0), vec2(bw - 3.0, r.height() - 6.0));
                        let br = ui.interact(b, resp.id.with(k), Sense::click());
                        let p = ui.painter();
                        let hot = br.hovered();
                        p.rect_filled(b, 3.0, if hot { Color32::from_rgb(64, 70, 80) } else { Color32::from_rgb(40, 45, 54) });
                        let c = if en { Color32::WHITE } else { DIM };
                        if k == 0 {
                            menu::icon(p, menu::Icon::Close, b.center(), 12.0, c);
                        } else {
                            menu::chevron(p, b.center(), 7.0, turn, c);
                        }
                        if en && br.clicked() {
                            act = Some((i, k));
                        }
                        x -= bw;
                    }
                }
                if resp.double_clicked() {
                    look = Some(wp.at());
                }
            }
        });
        if let Some((la, lo)) = look {
            self.menu.look_at(la, lo, 60_000.0);
        }
        match act {
            Some((i, 0)) => self.plan.remove(i),
            Some((i, 1)) => self.plan.swap(i, i + 1),
            Some((i, _)) => self.plan.swap(i, i - 1),
            None => {}
        }
    }

    /// the figures, the cruise level, and the way into the air.
    fn planner_foot(&mut self, ui: &mut egui::Ui, start: &mut bool) {
        if !self.plan.is_empty() {
            menu::slider_inline(ui, t!("game.cruise"), &mut self.plan.cruise_ft, menu::Scale::new(1000.0, 60000.0).step(1000.0), level);
            ui.add_space(12.0);
        }
        let ready = self.menu.pick.is_some() && !self.menu.starting;
        let db = airports::Db::get();
        let label = match (self.menu.airport, &self.menu.runway) {
            (Some(i), Some(r)) => t!("plan.fly_from", apt = db.apts[i as usize].ident, rwy = r),
            _ => t!("game.take_off").to_string(),
        };
        if menu::wide_button(ui, &label, Hint::of("Enter", "Menu"), true, ready).clicked() {
            self.planner.open = false;
            *start = true;
        }
    }
}

/// the colour of a procedure phase.
fn phase_colour(p: Phase) -> Color32 {
    match p {
        Phase::Sid => SID_COLOUR,
        _ => ARR_COLOUR,
    }
}

/// the width of the procedures' label column.
const LABEL_W: f32 = 104.0;

/// a procedure's name in the label column, a dot in its colour on the map before it.
fn label_col(ui: &mut egui::Ui, text: &str, col: Color32) {
    let (r, _) = ui.allocate_exact_size(vec2(LABEL_W - 6.0, 32.0), Sense::hover());
    ui.painter().circle_filled(pos2(r.left() + 4.0, r.center().y), 4.0, col);
    let g = menu::one_line(ui.painter(), text, menu::label(), TEXT, r.width() - 18.0);
    ui.painter().galley(pos2(r.left() + 16.0, r.center().y - g.size().y * 0.5), g, TEXT);
}

/// the runway grid: pairs to a row, the gap, a cell's height.
const RW_COLS: usize = 4;
const RW_GAP: f32 = 8.0;
const RW_H: f32 = 32.0;

/// a runway's number, for ordering ("09L" is 9).
fn rwy_num(id: &str) -> u32 {
    id.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(99)
}

/// an empty field as wide as a choice beside it, so the procedure rows keep their columns.
fn placeholder(ui: &mut egui::Ui, w: f32, text: &str) {
    let h = ui.spacing().interact_size.y.max(24.0);
    let (r, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let p = ui.painter();
    p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, Color32::from_white_alpha(14)), egui::StrokeKind::Inside);
    if !text.is_empty() {
        p.text(pos2(r.left() + 8.0, r.center().y), Align2::LEFT_CENTER, text, menu::label(), FAINT);
    }
}

/// a cruise level as pilots say it.
fn level(fl: f64) -> String {
    if fl >= 18_000.0 { format!("FL{:03.0}", fl / 100.0) } else { format!("{fl:.0} ft") }
}

/// underline tabs: the words, the chosen one lit with a bar under it.
fn tabs(ui: &mut egui::Ui, names: &[&str], cur: usize) -> Option<usize> {
    let mut hit = None;
    let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::hover());
    ui.painter().hline(row.x_range(), row.bottom() - 0.5, Stroke::new(1.0, menu::LINE_HI));
    let mut x = row.left();
    for (k, name) in names.iter().enumerate() {
        let g = ui.painter().layout_no_wrap(name.to_string(), menu::medium(15.5), TEXT);
        let r = Rect::from_min_max(pos2(x, row.top()), pos2(x + g.size().x, row.bottom()));
        let resp = ui.interact(r.expand2(vec2(8.0, 0.0)), ui.scope_id().with(("plan_tab", k)), Sense::click());
        let lit = menu::ease(ui.ctx(), resp.id, resp.hovered());
        let col = if k == cur { Color32::WHITE } else { menu::mix(DIM, TEXT, lit) };
        ui.painter().galley(pos2(r.left(), r.top() + 2.0), g, col);
        if k == cur {
            ui.painter().rect_filled(Rect::from_min_max(pos2(r.left(), row.bottom() - 2.0), pos2(r.right(), row.bottom())), 0.0, ACCENT);
        }
        if resp.clicked() {
            hit = Some(k);
        }
        x = r.right() + 28.0;
    }
    hit
}

/// space between the planner's sections.
fn gap(ui: &mut egui::Ui) {
    ui.add_space(24.0);
}

/// a title over a section.
fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(sentence(title)).font(menu::semibold(16.0)).color(Color32::WHITE));
    ui.add_space(10.0);
}

/// a title in sentence case: the first letter up, the rest as written unless it was all caps.
fn sentence(s: &str) -> String {
    if s.chars().any(|c| c.is_lowercase()) {
        return s.to_string();
    }
    let low = s.to_lowercase();
    let mut c = low.chars();
    c.next().map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
}

/// a section title with links on the right, all on one line.
fn header(ui: &mut egui::Ui, title: &str, links: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(sentence(title)).font(menu::semibold(16.0)).color(Color32::WHITE));
        menu::at_right(ui, title, |ui| {
            ui.spacing_mut().item_spacing.x = 14.0;
            links(ui);
        });
    });
    ui.add_space(10.0);
}

/// a quiet box around a section.
fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new().fill(Color32::from_white_alpha(5)).stroke(Stroke::new(1.0, Color32::from_white_alpha(14))).corner_radius(4).inner_margin(egui::Margin::symmetric(12, 12)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.spacing_mut().item_spacing.y = 0.0;
        add(ui);
    });
}

/// a word that acts as a button, in the accent colour.
fn link(ui: &mut egui::Ui, text: &str) -> bool {
    ui.add(egui::Button::new(RichText::new(text).font(menu::label()).color(ACCENT)).frame(false)).clicked()
}


/// the icao type designator simbrief knows an aircraft by.
fn icao_type(key: &str) -> &'static str {
    match key {
        "c172" => "C172",
        "dr400" => "DR40",
        "b747" => "B744",
        "g650" => "GLF6",
        "concorde" => "CONC",
        "fa18" => "F18S",
        "f22" => "F22",
        "f4" => "F4",
        "sr71" => "SR71",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_by_phase() {
        let mut w = Waypoint::fix("ABCDE", 0.0, 0.0);
        assert_eq!(wp_colour(&w), ROUTE_COLOUR);
        w.phase = Some((Phase::Sid, "X1".into()));
        assert_eq!(wp_colour(&w), SID_COLOUR);
        assert_eq!(icao_type("b747"), "B744");
    }
}
