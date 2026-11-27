//! the career's screens: home (the next step, reputation, the pilot), the
//! job board, training, the fleet, the company and the finances; the
//! briefing before a job, the card while flying it and the debrief after.

use crate::airports::Db;
use crate::career::{self, class_of, class_parse, money, money_short, tier, Career, Debrief, Hire, Job, Kind, Lic, Next, Req, Tab, KINDS, LICS, TIERS};
use crate::menu::{self, ACCENT, BAD, DIM, FAINT, OK, TEXT};
use crate::widgets::{self, card, chip, chip_ui, meter, meter_row};
use crate::{aircraft, flightui, geo, pilot, profile, tr_en, Game};
use egui::{pos2, vec2, Align2, Color32, Layout, Rect, RichText, Sense, Stroke};

/// what a press on the career screens asks for, done once drawing is over.
enum Act {
    Back,
    Tab(Tab),
    Select(u64),
    Brief(Job),
    CloseBrief,
    Fly(Job, Option<String>),
    GiveUp,
    Refresh,
    Filter(Option<Kind>),
    Mission(&'static str),
    Licence(Lic),
    Buy(&'static str, f64),
    Sell(String, f64),
    Repair(String, f64),
    Hire(Hire),
    Fire(usize),
    Assign(usize, Option<String>),
    CloseDebrief,
    NewCareer,
}

const TABS: [Tab; 6] = [Tab::Home, Tab::Jobs, Tab::Training, Tab::Fleet, Tab::Company, Tab::Finances];

fn tab_name(t: Tab) -> &'static str {
    match t {
        Tab::Home => tr_en("Home"),
        Tab::Jobs => tr_en("Jobs"),
        Tab::Training => tr_en("Training"),
        Tab::Fleet => tr_en("Fleet"),
        Tab::Company => tr_en("Company"),
        Tab::Finances => tr_en("Finances"),
    }
}

/// the airport's name for a code.
fn apt_name(code: &str) -> String {
    let db = Db::get();
    db.find(code).map(|i| db.apts[i as usize].name.clone()).unwrap_or_default()
}

fn apt_pos(code: &str) -> Option<(f64, f64)> {
    let db = Db::get();
    db.find(code).map(|i| (db.apts[i as usize].lat, db.apts[i as usize].lon))
}

/// a job's title in a line.
pub fn job_title(j: &Job) -> String {
    match j.kind {
        k if k.round_trip() => format!("{}  ·  {}", tr_en(k.name()), j.via_name),
        k => format!("{} {} {}", tr_en(k.name()), tr_en("to"), apt_name(&j.to)),
    }
}

/// the load in words.
fn load_text(j: &Job) -> String {
    if j.pax > 0 {
        format!("{} {}", j.pax, if j.pax == 1 { tr_en("person") } else { tr_en("people") })
    } else if j.payload_kg > 0.0 {
        format!("{:.0} kg", j.payload_kg)
    } else {
        tr_en("no load").into()
    }
}

/// the tier as filled dots.
fn tier_dots(p: &egui::Painter, at: egui::Pos2, t: u8, col: Color32) {
    for i in 0..3u8 {
        let c = at + vec2(i as f32 * 9.0, 0.0);
        if i < t {
            p.circle_filled(c, 3.2, col);
        } else {
            p.circle_stroke(c, 3.0, Stroke::new(1.0, Color32::from_white_alpha(50)));
        }
    }
}

/// a small padlock.
fn lock(p: &egui::Painter, c: egui::Pos2, s: f32, col: Color32) {
    let body = Rect::from_center_size(c + vec2(0.0, s * 0.25), vec2(s * 1.1, s * 0.85));
    p.rect_filled(body, 2.0, col);
    let pts: Vec<egui::Pos2> = (0..=12).map(|i| {
        let a = std::f32::consts::PI + std::f32::consts::PI * i as f32 / 12.0;
        c + vec2(a.cos() * s * 0.34, -s * 0.12 + a.sin() * s * 0.45)
    }).collect();
    p.add(egui::Shape::line(pts, Stroke::new(1.6, col)));
}

/// a sketch of the route: the start, the point on the way and the end, to scale.
fn route_sketch(p: &egui::Painter, r: Rect, j: &Job) {
    p.rect_filled(r, menu::R_WIDGET, Color32::from_rgb(12, 17, 24));
    // a faint grid.
    for k in 1..6 {
        let x = r.left() + r.width() * k as f32 / 6.0;
        p.vline(x, r.y_range(), Stroke::new(1.0, Color32::from_white_alpha(6)));
        let y = r.top() + r.height() * k as f32 / 6.0;
        p.hline(r.x_range(), y, Stroke::new(1.0, Color32::from_white_alpha(6)));
    }
    let Some(a) = apt_pos(&j.from) else { return };
    let mut pts: Vec<(f64, f64, String)> = vec![(a.0, a.1, j.from.clone())];
    if let Some((la, lo)) = j.via {
        pts.push((la, lo, String::new()));
    }
    if let Some(b) = apt_pos(&j.to) {
        if j.to != j.from || j.via.is_none() {
            pts.push((b.0, b.1, j.to.clone()));
        } else {
            pts.push((a.0, a.1, String::new()));
        }
    }
    let k = a.0.to_radians().cos();
    let xy: Vec<(f64, f64)> = pts.iter().map(|q| ((q.1 - a.1) * k, q.0 - a.0)).collect();
    let (mut x0, mut x1, mut y0, mut y1) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for q in &xy {
        x0 = x0.min(q.0);
        x1 = x1.max(q.0);
        y0 = y0.min(q.1);
        y1 = y1.max(q.1);
    }
    let span = (x1 - x0).max(y1 - y0).max(1e-3);
    let inner = r.shrink(30.0);
    let s = (inner.width().min(inner.height()) as f64) / span;
    let cx = (x0 + x1) * 0.5;
    let cy = (y0 + y1) * 0.5;
    let to_screen = |q: (f64, f64)| pos2(inner.center().x + ((q.0 - cx) * s) as f32, inner.center().y - ((q.1 - cy) * s) as f32);
    let sp: Vec<egui::Pos2> = xy.iter().map(|q| to_screen(*q)).collect();
    let col = j.kind.colour();
    for w in sp.windows(2) {
        p.line_segment([w[0], w[1]], Stroke::new(4.0, Color32::from_black_alpha(140)));
        p.line_segment([w[0], w[1]], Stroke::new(2.0, col));
    }
    for (i, (q, pt)) in sp.iter().zip(pts.iter()).enumerate() {
        let via = j.via.is_some() && i == 1;
        if via {
            let d = 6.5;
            p.add(egui::Shape::convex_polygon(vec![*q + vec2(0.0, -d), *q + vec2(d, 0.0), *q + vec2(0.0, d), *q + vec2(-d, 0.0)], col, Stroke::new(1.5, Color32::BLACK)));
        } else {
            p.circle_filled(*q, 6.0, Color32::WHITE);
            p.circle_stroke(*q, 6.0, Stroke::new(2.0, Color32::BLACK));
        }
        if !pt.2.is_empty() {
            p.text(*q + vec2(10.0, -10.0), Align2::LEFT_BOTTOM, &pt.2, menu::semibold(13.0), Color32::WHITE);
        }
    }
    p.text(r.right_bottom() + vec2(-10.0, -8.0), Align2::RIGHT_BOTTOM, format!("{:.0} nm", j.dist_nm), menu::caption(), DIM);
}

impl Game {
    /// the career (see the module docs).
    pub(crate) fn career_screen(&mut self, ctx: &egui::Context) {
        widgets::backdrop(ctx, "career");
        if !self.career.started {
            self.career_start(ctx);
            return;
        }
        self.career.accrue();
        if self.career.board.is_empty() || self.career.board_for != self.career.base {
            self.career.make_board();
            self.career.save();
        }
        let screen = ctx.content_rect();
        let (x0, w) = widgets::columns(screen, 1340.0);
        let mut act: Option<Act> = None;
        let tab = self.career.view.tab;
        let lvl = pilot::level(self.profile.xp);
        egui::Area::new(egui::Id::unique("career")).fixed_pos(pos2(x0, 26.0)).constrain(false).show(ctx, |ui| {
            ui.set_width(w);
            let names: Vec<&str> = TABS.iter().map(|t| tab_name(*t)).collect();
            let cur = TABS.iter().position(|t| *t == tab).unwrap_or(0);
            let money_now = self.career.money;
            let (back, hit) = widgets::page_head(ui, tr_en("Career"), &names, cur, |ui| {
                // the level and the money, on the right of the head.
                let (r, _) = ui.allocate_exact_size(vec2(150.0, 40.0), Sense::hover());
                let p = ui.painter();
                pilot::level_ring(p, pos2(r.left() + 18.0, r.center().y), 15.0, lvl.frac(), lvl.n, 1.0);
                p.text(pos2(r.left() + 42.0, r.center().y - 8.0), Align2::LEFT_CENTER, tr_en(pilot::rank(lvl.n)), menu::semibold(13.5), Color32::WHITE);
                p.text(pos2(r.left() + 42.0, r.center().y + 9.0), Align2::LEFT_CENTER, if lvl.span == 0 { format!("{} XP", lvl.into) } else { format!("{} / {} XP", lvl.into, lvl.span) }, menu::caption(), DIM);
                let (m, _) = ui.allocate_exact_size(vec2(130.0, 40.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(m, menu::R_WIDGET, Color32::from_white_alpha(8));
                p.text(m.center(), Align2::CENTER_CENTER, money_short(money_now), menu::bold(21.0), if money_now < 0.0 { BAD } else { ACCENT });
            });
            if back {
                act = Some(Act::Back);
            }
            if let Some(i) = hit {
                act = Some(Act::Tab(TABS[i]));
            }
            let h = (screen.bottom() - ui.cursor().top() - 24.0).max(320.0);
            // (an area's own height is only what it had last time: the page gets the rest of the screen)
            let a = ui.allocate_ui_with_layout(vec2(w, h), Layout::top_down(egui::Align::Min), |ui| match tab {
                Tab::Home => self.career_home(ui, w, h),
                Tab::Jobs => self.career_jobs(ui, w, h),
                Tab::Training => self.career_training(ui, w, h),
                Tab::Fleet => self.career_fleet(ui, w, h),
                Tab::Company => self.career_company(ui, w, h),
                Tab::Finances => self.career_finances(ui, w, h),
            }).inner;
            if a.is_some() {
                act = a;
            }
        });
        if self.career.view.brief.is_some() {
            if let Some(a) = self.career_brief(ctx) {
                act = Some(a);
            }
        }
        if let Some(d) = self.career.view.debrief.clone() {
            if self.career_flight.is_none() && debrief_window(ctx, &d, false) {
                act = Some(Act::CloseDebrief);
            }
        }
        if let Some(a) = act {
            self.career_do(a);
        }
    }

    fn career_do(&mut self, a: Act) {
        let c = &mut self.career;
        c.view.note = None;
        match a {
            Act::Back => {
                self.menu.screen = crate::menu::Screen::Main;
                self.pad_home = true;
            }
            Act::Tab(t) => c.view.tab = t,
            Act::Select(id) => c.view.sel = Some(id),
            Act::Brief(j) => {
                c.view.brief_plane = None;
                c.view.brief = Some(j);
            }
            Act::CloseBrief => c.view.brief = None,
            Act::Fly(j, plane) => {
                // a job off the board is taken as it is flown.
                c.board.retain(|b| b.id != j.id);
                self.fly_job(j, plane.as_deref());
            }
            Act::GiveUp => {
                if let Some(j) = c.active.take() {
                    let k = c.rep_of(j.kind);
                    c.rep_by.insert(j.kind, (k - 20.0).max(0.0));
                    c.rep = (c.rep - 20.0).max(0.0);
                    c.save();
                }
            }
            Act::Refresh => {
                c.refresh += 1;
                c.make_board();
                c.view.sel = None;
                c.save();
            }
            Act::Filter(k) => c.view.filter = k,
            Act::Mission(id) => {
                // flown from here, it counts for the career.
                if let Some(m) = crate::missions::by_id(id) {
                    self.start_mission(m);
                    if let Some(r) = self.mission.as_mut() {
                        r.career = true;
                    }
                }
            }
            Act::Licence(l) => {
                self.get_licence(l);
                self.career.view.note = Some((format!("{} {}", tr_en(l.name()), tr_en("earned.")), true));
            }
            Act::Buy(key, price) => {
                if c.money >= price && !c.owned.iter().any(|k| k == key) {
                    let name = aircraft::by_key(key).map_or(key, |s| s.name);
                    c.pay(format!("{} {name}", tr_en("Bought")), -price);
                    c.owned.push(key.to_string());
                    c.wear.insert(key.to_string(), 0.0);
                    c.save();
                    self.unlock("owner");
                }
            }
            Act::Sell(key, value) => {
                c.owned.retain(|k| *k != key);
                for h in &mut c.hires {
                    if h.plane.as_deref() == Some(key.as_str()) {
                        h.plane = None;
                    }
                }
                let name = aircraft::by_key(&key).map_or(key.clone(), |s| s.name.to_string());
                c.pay(format!("{} {name}", tr_en("Sold")), value);
                c.wear.remove(&key);
                c.save();
            }
            Act::Repair(key, cost) => {
                if c.money >= cost {
                    let name = aircraft::by_key(&key).map_or(key.clone(), |s| s.name.to_string());
                    c.pay(format!("{} {name}", tr_en("Maintenance:")), -cost);
                    c.wear.insert(key, 0.0);
                    c.save();
                }
            }
            Act::Hire(h) => {
                let fee = career::signing_fee(h.skill);
                if c.money >= fee {
                    c.pay(format!("{} {}", tr_en("Hired"), h.name), -fee);
                    let plane = c.free_planes().into_iter().next();
                    c.hires.push(Hire { hired: profile::now_unix(), plane, ..h });
                    if c.accrued == 0 {
                        c.accrued = profile::now_unix();
                    }
                    c.save();
                    self.unlock("chief_pilot");
                }
            }
            Act::Fire(i) => {
                if i < c.hires.len() {
                    let h = c.hires.remove(i);
                    c.pay(format!("{} {}", tr_en("Severance:"), h.name), -(career::wage(h.skill) * 8.0).round());
                    c.save();
                }
            }
            Act::Assign(i, plane) => {
                if let Some(h) = c.hires.get_mut(i) {
                    h.plane = plane;
                    c.save();
                }
            }
            Act::CloseDebrief => c.view.debrief = None,
            Act::NewCareer => {
                *c = Career::default();
                c.save();
            }
        }
    }

    /// a new career: the pilot's name and a home airport, and what lies ahead.
    fn career_start(&mut self, ctx: &egui::Context) {
        let mut start = false;
        let mut back = false;
        egui::Area::new(egui::Id::unique("career_new")).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            menu::panel().inner_margin(24.0).show(ui, |ui| {
                ui.set_width(640.0);
                menu::screen_heading(ui, tr_en("From student to captain"));
                ui.add_space(10.0);
                ui.add(egui::Label::new(RichText::new(tr_en("Start with $15,000 at a home airport. Fly the lessons and pass a checkride for your private licence, then take jobs from the board wherever you are: each is marked on the landing, the timing, the comfort and the procedures, and paid by its stars. Reputation in each line of work opens better jobs; licences and type ratings open bigger aircraft. Buy your own, then hire pilots to fly them while you are away.")).font(menu::body()).color(TEXT)).wrap());
                ui.add_space(16.0);
                // the path, at a glance.
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::hover());
                let p = ui.painter();
                let steps = [tr_en("Lessons"), tr_en("Checkride"), tr_en("Jobs"), tr_en("Reputation"), tr_en("Ratings"), tr_en("Company")];
                let dx = r.width() / steps.len() as f32;
                p.hline((r.left() + dx * 0.5)..=(r.right() - dx * 0.5), r.top() + 16.0, Stroke::new(2.0, Color32::from_white_alpha(30)));
                for (i, s) in steps.iter().enumerate() {
                    let c = pos2(r.left() + dx * (i as f32 + 0.5), r.top() + 16.0);
                    p.circle_filled(c, 9.0, if i == 0 { ACCENT } else { menu::RAISED_HI });
                    p.text(c, Align2::CENTER_CENTER, (i + 1).to_string(), menu::bold(12.0), if i == 0 { menu::ON_ACCENT } else { TEXT });
                    p.text(c + vec2(0.0, 22.0), Align2::CENTER_TOP, *s, menu::caption(), DIM);
                }
                ui.add_space(14.0);
                menu::form_row(ui, tr_en("Pilot"), "", |ui| {
                    if self.career.name.is_empty() && !self.profile.name.is_empty() {
                        self.career.name = self.profile.name.clone();
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.career.name).desired_width(280.0).hint_text(tr_en("Your name")).font(menu::label()).margin(vec2(10.0, 7.0)));
                });
                menu::form_row(ui, tr_en("Home airport"), tr_en("Any airport by its ICAO code. Half Moon Bay (KHAF) is a good first base."), |ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.career.base).desired_width(110.0).hint_text("KHAF").font(menu::label()).margin(vec2(10.0, 7.0)));
                    let db = Db::get();
                    let found = db.find(&self.career.base).map(|i| db.apts[i as usize].name.clone());
                    ui.label(RichText::new(found.clone().unwrap_or_else(|| tr_en("Unknown code").into())).font(menu::caption()).color(if found.is_some() { DIM } else { BAD }));
                });
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    let ok = Db::get().find(&self.career.base).is_some();
                    if widgets::action(ui, tr_en("Begin career"), true, ok, 220.0, 44.0).clicked() {
                        start = true;
                    }
                    ui.add_space(8.0);
                    if widgets::action(ui, tr_en("Back"), false, true, 120.0, 44.0).clicked() {
                        back = true;
                    }
                });
            });
        });
        if start {
            let db = Db::get();
            if let Some(i) = db.find(&self.career.base) {
                self.career.base = db.apts[i as usize].ident.clone();
                if self.career.name.trim().is_empty() {
                    self.career.name = if self.profile.name.is_empty() { "Pilot".into() } else { self.profile.name.clone() };
                }
                if self.profile.name.trim().is_empty() {
                    self.profile.name = self.career.name.clone();
                    self.keep_profile();
                }
                self.career.started = true;
                let base = self.career.base.clone();
                self.career.pay(format!("{} {base}", tr_en("Career started at")), 0.0);
                self.career.make_board();
                self.career.save();
            }
        }
        if back {
            self.menu.screen = crate::menu::Screen::Main;
        }
    }

    /// home: the next step, the reputation, the pilot, the base, the latest.
    fn career_home(&mut self, ui: &mut egui::Ui, w: f32, h: f32) -> Option<Act> {
        let mut act = None;
        let lw = (w * 0.6).floor();
        let rw = w - lw - 40.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(40.0, 14.0);
            ui.allocate_ui_with_layout(vec2(lw, h), Layout::top_down(egui::Align::Min), |ui| {
                // the next step, large.
                let (head, line, next) = self.next_step();
                let (r, _) = ui.allocate_exact_size(vec2(lw, 150.0), Sense::hover());
                let p = ui.painter();
                p.rect_filled(r, menu::R_PANEL, Color32::from_rgb(20, 18, 8));
                p.rect_stroke(r, menu::R_PANEL, Stroke::new(1.0, ACCENT.gamma_multiply(0.45)), egui::StrokeKind::Inside);
                let hg = menu::one_line(p, &head, menu::bold(32.0), Color32::WHITE, r.width() - 260.0);
                let hh = hg.size().y;
                p.galley(pos2(r.left() + 24.0, r.center().y - hh + 2.0), hg, Color32::WHITE);
                let lg = menu::one_line(p, &line, menu::label(), TEXT, r.width() - 260.0);
                p.galley(pos2(r.left() + 24.0, r.center().y + 10.0), lg, TEXT);
                let br = Rect::from_min_size(pos2(r.right() - 214.0, r.center().y - 22.0), vec2(190.0, 44.0));
                let mut bu = ui.new_child(egui::UiBuilder::new().max_rect(br));
                let label = match next {
                    Next::Job => tr_en("Fly it"),
                    Next::Jobs => tr_en("Job board"),
                    Next::Training => tr_en("Training"),
                    Next::Mission(id) if id.starts_with("e_") => tr_en("Take checkride"),
                    Next::Mission(_) => tr_en("Fly lesson"),
                };
                if widgets::action(&mut bu, label, true, true, 190.0, 44.0).clicked() {
                    act = match next {
                        Next::Job => self.career.active.clone().map(Act::Brief),
                        Next::Jobs => Some(Act::Tab(Tab::Jobs)),
                        Next::Training => Some(Act::Tab(Tab::Training)),
                        Next::Mission(id) => Some(Act::Mission(id)),
                    };
                }
                if self.career.active.is_some() {
                    ui.horizontal(|ui| {
                        if widgets::action(ui, tr_en("Give up the job (-20 reputation)"), false, true, 280.0, 32.0).clicked() {
                            act = Some(Act::GiveUp);
                        }
                    });
                }
                // the reputation in each line of work, once there is some work.
                let kinds: Vec<Kind> = KINDS.iter().copied().filter(|k| self.career.has(k.lic()) || self.career.rep_of(*k) > 0.0).collect();
                if !kinds.is_empty() {
                    card(ui, 0.0, |ui| {
                        widgets::title(ui, tr_en("REPUTATION"));
                        let cols = 2;
                        let cw = (ui.available_width() - 20.0) / cols as f32;
                        for row in kinds.chunks(cols) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 20.0;
                                for k in row {
                                    let (r, _) = ui.allocate_exact_size(vec2(cw, 46.0), Sense::hover());
                                    let p = ui.painter();
                                    let rep = self.career.rep_of(*k);
                                    let (t, f) = tier(rep);
                                    p.circle_filled(pos2(r.left() + 5.0, r.top() + 11.0), 4.0, k.colour());
                                    p.text(pos2(r.left() + 16.0, r.top() + 11.0), Align2::LEFT_CENTER, tr_en(k.name()), menu::medium(14.5), Color32::WHITE);
                                    p.text(pos2(r.right(), r.top() + 11.0), Align2::RIGHT_CENTER, format!("{}  ·  {:.0}", tr_en(TIERS[t].1), rep), menu::caption(), DIM);
                                    meter(p, Rect::from_min_size(pos2(r.left() + 16.0, r.top() + 28.0), vec2(r.width() - 16.0, 5.0)), f, k.colour());
                                }
                            });
                        }
                    });
                }
                // the licences held, then the next one to work on.
                let next = LICS.into_iter().find(|l| !self.career.has(*l) && l.reqs().iter().all(|r| !matches!(r, Req::Holds(h) if !self.career.has(*h))));
                card(ui, 0.0, |ui| {
                    widgets::title(ui, tr_en("LICENCES AND RATINGS"));
                    ui.spacing_mut().item_spacing.y = 4.0;
                    for l in LICS.into_iter().filter(|l| self.career.has(*l) || Some(*l) == next) {
                        let has = self.career.has(l);
                        let state = if has {
                            None
                        } else if self.can_get(l) {
                            Some((tr_en("Ready").to_string(), OK))
                        } else {
                            let done = l.reqs().iter().filter(|r| self.req_state(**r).0).count();
                            Some((format!("{} {done} / {}", tr_en("Next"), l.reqs().len()), DIM))
                        };
                        if lic_row(ui, l, has, state).clicked() {
                            act = Some(Act::Tab(Tab::Training));
                        }
                    }
                });
            });
            ui.allocate_ui_with_layout(vec2(rw, h), Layout::top_down(egui::Align::Min), |ui| {
                // the pilot.
                card(ui, 0.0, |ui| {
                    // the licence is the career's one rank; the level and its experience are in the head.
                    let lvl = pilot::level(self.profile.xp);
                    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 72.0), Sense::hover());
                    let p = ui.painter();
                    widgets::avatar(p, pos2(r.left() + 32.0, r.center().y), 30.0, &self.career.name, lvl.frac());
                    p.text(pos2(r.left() + 80.0, r.center().y - 12.0), Align2::LEFT_CENTER, &self.career.name, menu::bold(22.0), Color32::WHITE);
                    p.text(pos2(r.left() + 80.0, r.center().y + 12.0), Align2::LEFT_CENTER, tr_en(self.career.title()), menu::label(), TEXT);
                    ui.add_space(8.0);
                    let hours = profile::totals(&self.logbook.flights).air_s / 3600.0;
                    let avg = if self.career.stars.is_empty() { 0.0 } else { self.career.stars.iter().map(|s| *s as f64).sum::<f64>() / self.career.stars.len() as f64 };
                    widgets::line(ui, tr_en("Hours flown"), &format!("{hours:.1}"), Color32::WHITE);
                    widgets::line(ui, tr_en("Jobs done"), &self.career.jobs_done.to_string(), Color32::WHITE);
                    widgets::line(ui, tr_en("Average rating"), &if avg > 0.0 { format!("{avg:.1} ★") } else { "—".into() }, ACCENT);
                    widgets::line(ui, tr_en("Reputation"), &format!("{:.0}", self.career.rep), Color32::WHITE);
                });
                // the latest money in and out, once there is any.
                let rows: Vec<_> = self.career.ledger.iter().rev().filter(|e| e.amount != 0.0).take(6).cloned().collect();
                if !rows.is_empty() {
                    card(ui, 0.0, |ui| {
                        widgets::title(ui, tr_en("RECENT"));
                        for e in rows {
                            widgets::line(ui, &e.what, &money(e.amount), if e.amount < 0.0 { BAD } else { OK });
                        }
                    });
                }
            });
        });
        act
    }

    /// the job board: the kinds of work down the side, the jobs, and the one picked.
    fn career_jobs(&mut self, ui: &mut egui::Ui, w: f32, h: f32) -> Option<Act> {
        let mut act = None;
        let fw = 210.0;
        let dw = 380.0;
        let lw = w - fw - dw - 32.0;
        let filter = self.career.view.filter;
        let board = self.career.board.clone();
        let shown: Vec<&Job> = board.iter().filter(|j| filter.is_none_or(|k| j.kind == k)).collect();
        // what can be flown first; the rest fold into one line under it.
        let (open, out): (Vec<&Job>, Vec<&Job>) = shown.iter().copied().partition(|j| self.career.can_take(j).is_ok());
        let fold_id = egui::Id::unique("career_board_locked");
        let unfolded = ui.ctx().data(|d| d.get_temp::<bool>(fold_id)).unwrap_or(false);
        let sel = self.career.view.sel.and_then(|id| board.iter().find(|j| j.id == id)).or(open.first().copied()).cloned();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 6.0);
            // the kinds of work.
            ui.allocate_ui_with_layout(vec2(fw, h), Layout::top_down(egui::Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                let mut kinds: Vec<Option<Kind>> = vec![None];
                for k in KINDS {
                    if board.iter().any(|j| j.kind == k) {
                        kinds.push(Some(k));
                    }
                }
                for k in kinds {
                    let n = board.iter().filter(|j| k.is_none_or(|k| j.kind == k)).count();
                    let on = filter == k;
                    let (r, resp) = ui.allocate_exact_size(vec2(fw, 38.0), Sense::click());
                    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered());
                    let p = ui.painter();
                    if on {
                        p.rect_filled(r, menu::R_WIDGET, menu::CHOSEN);
                    } else if lit > 0.0 {
                        p.rect_filled(r, menu::R_WIDGET, Color32::from_white_alpha((10.0 * lit) as u8));
                    }
                    let name = k.map_or(tr_en("All work"), |k| tr_en(k.name()));
                    let locked = k.is_some_and(|k| !self.career.has(k.lic()));
                    let g = menu::one_line(p, name, menu::medium(14.5), if locked { DIM } else { Color32::WHITE }, fw - 60.0);
                    p.galley(pos2(r.left() + 14.0, r.center().y - g.size().y * 0.5), g, Color32::WHITE);
                    if locked {
                        lock(p, pos2(r.right() - 34.0, r.center().y), 9.0, FAINT);
                    }
                    p.text(pos2(r.right() - 10.0, r.center().y), Align2::RIGHT_CENTER, n.to_string(), menu::caption(), DIM);
                    if resp.clicked() {
                        act = Some(Act::Filter(k));
                    }
                }
                ui.add_space(12.0);
                if widgets::action(ui, tr_en("New jobs"), false, true, fw, 34.0).clicked() {
                    act = Some(Act::Refresh);
                }
                ui.add_space(4.0);
                ui.label(RichText::new(format!("{} {}", tr_en("Jobs out of"), self.career.base)).font(menu::caption()).color(DIM));
            });
            // the jobs.
            ui.allocate_ui_with_layout(vec2(lw, h), Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical().id_salt("career_board").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    if shown.is_empty() {
                        widgets::empty(ui, tr_en("No work of this kind here today. Try new jobs, or move your base."));
                    } else if open.is_empty() {
                        widgets::empty(ui, tr_en("Nothing you can fly here yet."));
                    }
                    for j in &open {
                        let on = sel.as_ref().is_some_and(|s| s.id == j.id);
                        let resp = job_row(ui, j, on, None);
                        if resp.clicked() {
                            act = Some(Act::Select(j.id));
                        }
                        if resp.double_clicked() && self.career.active.is_none() {
                            act = Some(Act::Brief((*j).clone()));
                        }
                    }
                    if !out.is_empty() {
                        let n = out.len();
                        let words = if open.is_empty() { tr_en(if n == 1 { "job out of reach" } else { "jobs out of reach" }) } else { tr_en("more out of reach") };
                        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{n} {words}")));
                        let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
                        let p = ui.painter();
                        let col = menu::mix(DIM, TEXT, lit);
                        lock(p, pos2(r.left() + 8.0, r.center().y), 9.0, col);
                        let g = p.layout_no_wrap(format!("{n} {words}"), menu::label(), col);
                        let gx = r.left() + 24.0;
                        p.galley(pos2(gx, r.center().y - g.size().y * 0.5), g.clone(), col);
                        menu::chevron(p, pos2(gx + g.size().x + 14.0, r.center().y), 5.0, if unfolded { 3.0 } else { 1.0 }, col);
                        if resp.clicked() {
                            ui.ctx().data_mut(|d| d.insert_temp(fold_id, !unfolded));
                        }
                        if unfolded {
                            for j in &out {
                                let on = sel.as_ref().is_some_and(|s| s.id == j.id);
                                let why = self.career.can_take(j).err();
                                if job_row(ui, j, on, why.as_ref()).clicked() {
                                    act = Some(Act::Select(j.id));
                                }
                            }
                        }
                    }
                });
            });
            // the job picked, none while nothing is.
            let Some(j) = sel.as_ref() else { return };
            ui.allocate_ui_with_layout(vec2(dw, h), Layout::top_down(egui::Align::Min), |ui| {
                card(ui, h, |ui| {
                    let col = j.kind.colour();
                    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
                    let cr = chip(ui.painter(), pos2(r.left(), r.center().y), &tr_en(j.kind.name()).to_uppercase(), col);
                    tier_dots(ui.painter(), pos2(cr.right() + 12.0, r.center().y), j.tier, col);
                    ui.add_space(4.0);
                    ui.add(egui::Label::new(RichText::new(job_title(j)).font(menu::semibold(19.0)).color(Color32::WHITE)).wrap());
                    ui.label(RichText::new(format!("{}  ·  {}", j.client, tr_en(j.kind.blurb()))).font(menu::caption()).color(DIM));
                    ui.add_space(10.0);
                    let (sr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 170.0), Sense::hover());
                    route_sketch(ui.painter(), sr, j);
                    ui.add_space(10.0);
                    let c = class_parse(&j.class);
                    let plane = self.career_spec(c).map_or("", |s| s.name);
                    widgets::line(ui, tr_en("Pays"), &money(j.pay), ACCENT);
                    widgets::line(ui, tr_en("Distance"), &format!("{:.0} nm", j.dist_nm), Color32::WHITE);
                    widgets::line(ui, tr_en("Load"), &load_text(j), Color32::WHITE);
                    widgets::line(ui, tr_en("Time allowed"), &profile::hhmm(j.limit_s), Color32::WHITE);
                    widgets::line(ui, tr_en("Aircraft"), plane, Color32::WHITE);
                    ui.add_space(12.0);
                    match self.career.can_take(j) {
                        Err(why) => {
                            ui.horizontal(|ui| {
                                let (lr, _) = ui.allocate_exact_size(vec2(16.0, 18.0), Sense::hover());
                                lock(ui.painter(), lr.center(), 10.0, FAINT);
                                ui.label(RichText::new(why).font(menu::label()).color(DIM));
                            });
                        }
                        Ok(()) if self.career.active.is_some() => {
                            ui.label(RichText::new(tr_en("Finish or give up your current job first.")).font(menu::label()).color(DIM));
                        }
                        Ok(()) => {
                            if widgets::action(ui, tr_en("Accept job"), true, true, ui.available_width(), 46.0).clicked() {
                                act = Some(Act::Brief(j.clone()));
                            }
                        }
                    }
                });
            });
        });
        act
    }

    /// training: every licence, what it opens and what it takes.
    fn career_training(&mut self, ui: &mut egui::Ui, w: f32, h: f32) -> Option<Act> {
        let mut act = None;
        egui::ScrollArea::vertical().id_salt("career_training").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
            if let Some((note, good)) = self.career.view.note.clone() {
                ui.label(RichText::new(note).font(menu::label()).color(if good { OK } else { BAD }));
                ui.add_space(6.0);
            }
            let cw = (w - 16.0) / 2.0;
            // only what can be worked on now: not held, and nothing else needed first.
            let open: Vec<Lic> = LICS.iter().copied().filter(|l| !self.career.has(*l) && l.reqs().iter().all(|r| !matches!(r, Req::Holds(h) if !self.career.has(*h)))).collect();
            if open.is_empty() {
                widgets::empty(ui, tr_en("Every licence is earned."));
            }
            for row in open.chunks(2) {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    for l in row {
                        ui.allocate_ui_with_layout(vec2(cw, 10.0), Layout::top_down(egui::Align::Min), |ui| {
                            if let Some(a) = self.licence_card(ui, *l) {
                                act = Some(a);
                            }
                        });
                    }
                });
                ui.add_space(14.0);
            }
        });
        act
    }

    fn licence_card(&self, ui: &mut egui::Ui, l: Lic) -> Option<Act> {
        let mut act = None;
        let has = self.career.has(l);
        let states: Vec<(Req, bool, String)> = l.reqs().iter().map(|r| {
            let (ok, s) = self.req_state(*r);
            (*r, ok, s)
        }).collect();
        let done = states.iter().filter(|s| s.1).count();
        let ready = !has && done == states.len();
        // a licence still waiting on another shows only what it waits on.
        let waits: Vec<&str> = states.iter().filter_map(|s| match s.0 { Req::Holds(h) if !s.1 => Some(tr_en(h.name())), _ => None }).collect();
        let locked = !has && !waits.is_empty();
        let (word, col) = if has {
            (tr_en("EARNED"), ACCENT)
        } else if ready {
            (tr_en("READY"), OK)
        } else if locked {
            (tr_en("LOCKED"), FAINT)
        } else {
            ("", menu::LINK)
        };
        card(ui, 0.0, |ui| {
            let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 50.0), Sense::hover());
            let p = ui.painter();
            let bc = pos2(r.left() + 24.0, r.center().y);
            p.circle_filled(bc, 23.0, if has { Color32::from_rgb(52, 42, 8) } else { menu::RAISED });
            p.circle_stroke(bc, 23.0, Stroke::new(2.0, col));
            p.text(bc, Align2::CENTER_CENTER, l.short(), menu::bold(12.5), if has { ACCENT } else { TEXT });
            p.text(pos2(r.left() + 60.0, r.top() + 14.0), Align2::LEFT_CENTER, tr_en(l.name()), menu::semibold(18.0), Color32::WHITE);
            let og = menu::one_line(p, tr_en(l.opens()), menu::caption(), DIM, r.width() - 160.0);
            p.galley(pos2(r.left() + 60.0, r.top() + 28.0), og, DIM);
            if !word.is_empty() {
                let g = p.layout_no_wrap(word.to_string(), menu::semibold(11.5), col);
                chip(p, pos2(r.right() - g.size().x - 14.0, r.top() + 14.0), word, col);
            } else {
                let g = p.layout_no_wrap(format!("{done} / {}", states.len()), menu::label(), DIM);
                p.galley(pos2(r.right() - g.size().x - 14.0, r.top() + 14.0 - g.size().y * 0.5), g, DIM);
            }
            if locked {
                ui.add_space(4.0);
                ui.label(RichText::new(format!("{} {}", tr_en("Needs"), waits.join(&format!(" {} ", tr_en("and"))))).font(menu::label()).color(DIM));
            } else if !has {
                ui.add_space(6.0);
                meter_row(ui, done as f32 / states.len().max(1) as f32, col, 4.0);
                ui.add_space(8.0);
                for (req, ok, text) in &states {
                    ui.horizontal(|ui| {
                        ui.set_min_height(28.0);
                        let (tr, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                        let p = ui.painter();
                        if *ok {
                            p.circle_filled(tr.center(), 8.0, OK.gamma_multiply(0.85));
                            crate::missions::tick(p, tr.center(), 4.2, Color32::from_rgb(10, 30, 16));
                        } else {
                            p.circle_stroke(tr.center(), 7.5, Stroke::new(1.5, Color32::from_white_alpha(70)));
                        }
                        ui.label(RichText::new(text).font(menu::label()).color(if *ok { DIM } else { TEXT }));
                        if !ok {
                            let fly = match req {
                                Req::Lesson(id) | Req::Exam(id) => Some((*id, Act::Mission(id))),
                                _ => None,
                            };
                            if let Some((id, go)) = fly {
                                menu::at_right(ui, ("lic_fly", id), |ui| {
                                    let word = if id.starts_with("e_") { tr_en("Checkride") } else { tr_en("Fly") };
                                    if widgets::action(ui, word, false, true, 104.0, 28.0).clicked() {
                                        act = Some(go);
                                    }
                                });
                            }
                        }
                    });
                }
                // the button only once it can be had.
                if ready {
                    ui.add_space(8.0);
                    let label = if l.fee() > 0.0 { format!("{} ({})", tr_en("Pay and get licence"), money(l.fee())) } else { tr_en("Get licence").to_string() };
                    if widgets::action(ui, &label, true, true, ui.available_width(), 40.0).clicked() {
                        act = Some(Act::Licence(l));
                    }
                }
            }
        });
        act
    }

    /// the fleet: every type the career flies, owned or to rent.
    fn career_fleet(&mut self, ui: &mut egui::Ui, w: f32, h: f32) -> Option<Act> {
        let mut act = None;
        let ctx = ui.ctx().clone();
        let types = career::career_types();
        let stills: Vec<egui::TextureHandle> = types.iter().map(|s| self.still(&ctx, s)).collect();
        egui::ScrollArea::vertical().id_salt("career_fleet").max_height(h).auto_shrink([false, false]).show(ui, |ui| {
            let cols = if w > 1150.0 { 4 } else { 3 };
            let cw = (w - 16.0 * (cols as f32 - 1.0)) / cols as f32;
            for (row_i, row) in types.chunks(cols).enumerate() {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 16.0;
                    for (k, s) in row.iter().enumerate() {
                        let tex = &stills[row_i * cols + k];
                        ui.allocate_ui_with_layout(vec2(cw, 10.0), Layout::top_down(egui::Align::Min), |ui| {
                            if let Some(a) = self.fleet_card(ui, s, tex, cw) {
                                act = Some(a);
                            }
                        });
                    }
                });
                ui.add_space(16.0);
            }
        });
        act
    }

    fn fleet_card(&self, ui: &mut egui::Ui, s: &'static aircraft::Spec, tex: &egui::TextureHandle, cw: f32) -> Option<Act> {
        let mut act = None;
        let key = s.keys.first().copied().unwrap_or("");
        let c = class_of(s);
        let own = self.career.owned.iter().any(|k| k == key);
        let rated = self.career.rated(c);
        let price = career::price_of(s);
        card(ui, 0.0, |ui| {
            let (r, _) = ui.allocate_exact_size(vec2(cw - 40.0, (cw - 40.0) * 0.5), Sense::hover());
            let p = ui.painter();
            p.rect_filled(r, menu::R_WIDGET, Color32::from_rgb(14, 19, 27));
            let ts = tex.size_vec2();
            let k = (r.width() / ts.x).min(r.height() / ts.y);
            let ir = Rect::from_center_size(r.center(), ts * k);
            p.image(tex.id(), ir, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), if rated || own { Color32::WHITE } else { Color32::from_white_alpha(110) });
            if own {
                chip(p, pos2(r.left() + 8.0, r.top() + 16.0), tr_en("OWNED"), ACCENT);
            } else if !rated {
                lock(p, pos2(r.right() - 16.0, r.top() + 16.0), 11.0, DIM);
            }
            ui.add_space(8.0);
            ui.label(RichText::new(s.name).font(menu::semibold(16.5)).color(Color32::WHITE));
            ui.label(RichText::new(format!("{}  ·  {}", tr_en(c.name()), c.licence().map_or("", |l| l.short()))).font(menu::caption()).color(DIM));
            ui.add_space(6.0);
            if own {
                let cond = self.career.condition(key);
                let flying = self.career.hires.iter().find(|h| h.plane.as_deref() == Some(key)).map(|h| h.name.clone());
                widgets::line(ui, tr_en("Condition"), &format!("{:.0}%", cond * 100.0), if cond < 0.6 { BAD } else { Color32::WHITE });
                meter_row(ui, cond as f32, if cond < 0.6 { BAD } else { OK }, 4.0);
                if let Some(n) = flying {
                    widgets::line(ui, tr_en("Flown by"), &n, DIM);
                }
                ui.add_space(6.0);
                let value = price.unwrap_or(0.0) * 0.65 * (0.5 + 0.5 * cond);
                let repair = (price.unwrap_or(0.0) * (1.0 - cond) * 0.3).round();
                ui.horizontal(|ui| {
                    let bw = (ui.available_width() - 8.0) / 2.0;
                    if widgets::action(ui, &format!("{} {}", tr_en("Repair"), money_short(repair)), false, repair > 0.0 && self.career.money >= repair, bw, 30.0).clicked() {
                        act = Some(Act::Repair(key.to_string(), repair));
                    }
                    if widgets::action(ui, &format!("{} {}", tr_en("Sell"), money_short(value)), false, true, bw, 30.0).clicked() {
                        act = Some(Act::Sell(key.to_string(), value.round()));
                    }
                });
            } else {
                widgets::line(ui, tr_en("Rent"), &format!("{}/h", money(c.rent())), Color32::WHITE);
                widgets::line(ui, tr_en("Price"), &price.map_or_else(|| tr_en("Not for sale").to_string(), money), if price.is_some() { ACCENT } else { DIM });
                ui.add_space(6.0);
                // a buy button only when it can be bought: rated, and the money there.
                if let Some(pr) = price.filter(|pr| rated && self.career.money >= *pr) {
                    if widgets::action(ui, tr_en("Buy"), false, true, ui.available_width(), 32.0).on_hover_text(tr_en("No rental to pay, and it can earn for your company.")).clicked() {
                        act = Some(Act::Buy(key, pr));
                    }
                }
            }
        });
        act
    }

    /// the company: the pilots hired and those looking for work.
    fn career_company(&mut self, ui: &mut egui::Ui, w: f32, h: f32) -> Option<Act> {
        let mut act = None;
        let lw = (w * 0.6).floor();
        let rw = w - lw - 16.0;
        let can_hire = self.career.has(Lic::Cpl) && !self.career.owned.is_empty();
        // nothing to run yet: one line on what starts it, not a page of zeros.
        if self.career.hires.is_empty() && !can_hire {
            card(ui, 0.0, |ui| widgets::empty(ui, tr_en("Your company starts with a commercial pilot licence and an aircraft of your own.")));
            return None;
        }
        let hourly: f64 = self.career.hires.iter().map(|hh| hh.plane.as_ref().and_then(|k| aircraft::by_key(k)).map_or(0.0, |s| class_of(s).hourly() * 0.33 * (0.75 + 0.1 * hh.skill as f64)) - career::wage(hh.skill) * 0.33).sum();
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 14.0);
            ui.allocate_ui_with_layout(vec2(lw, h), Layout::top_down(egui::Align::Min), |ui| {
                widgets::figures(ui, &[(self.career.hires.len().to_string(), tr_en("Pilots")), (self.career.owned.len().to_string(), tr_en("Aircraft owned")), (format!("{}/h", money(hourly)), tr_en("Company earnings"))]);
                card(ui, 0.0, |ui| {
                    widgets::title(ui, tr_en("YOUR PILOTS"));
                    if self.career.hires.is_empty() {
                        widgets::empty(ui, tr_en("No pilots yet. Pilots you hire fly your aircraft and earn while you are away."));
                    }
                    let free = self.career.free_planes();
                    for (i, hh) in self.career.hires.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.set_min_height(54.0);
                            let (r, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                            widgets::avatar(ui.painter(), r.center(), 18.0, &hh.name, hh.skill as f32 / 5.0);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.label(RichText::new(&hh.name).font(menu::semibold(15.5)).color(Color32::WHITE));
                                let plane = hh.plane.as_deref().and_then(aircraft::by_key).map_or(tr_en("No aircraft"), |s| s.name);
                                ui.label(RichText::new(format!("{} {}  ·  {plane}  ·  {} {}", tr_en("Skill"), hh.skill, money(hh.earned), tr_en("earned"))).font(menu::caption()).color(DIM));
                            });
                            menu::at_right(ui, ("hire", i), |ui| {
                                if widgets::action(ui, tr_en("Let go"), false, true, 90.0, 30.0).clicked() {
                                    act = Some(Act::Fire(i));
                                }
                                ui.add_space(6.0);
                                if hh.plane.is_some() {
                                    if widgets::action(ui, tr_en("Ground"), false, true, 90.0, 30.0).clicked() {
                                        act = Some(Act::Assign(i, None));
                                    }
                                } else if let Some(k) = free.first() {
                                    let name = aircraft::by_key(k).map_or(k.as_str(), |s| s.name);
                                    if widgets::action(ui, &format!("{} {}", tr_en("Fly the"), name), false, true, 170.0, 30.0).clicked() {
                                        act = Some(Act::Assign(i, Some(k.clone())));
                                    }
                                }
                            });
                        });
                        menu::rule(ui);
                    }
                });
            });
            ui.allocate_ui_with_layout(vec2(rw, h), Layout::top_down(egui::Align::Min), |ui| {
                card(ui, 0.0, |ui| {
                    widgets::title(ui, tr_en("LOOKING FOR WORK"));
                    // no candidates to look at until hiring is possible.
                    if !can_hire {
                        widgets::empty(ui, tr_en("To hire pilots, you need a commercial pilot licence and an aircraft of your own."));
                        return;
                    }
                    for cand in self.career.candidates() {
                        ui.horizontal(|ui| {
                            ui.set_min_height(50.0);
                            let (r, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
                            widgets::avatar(ui.painter(), r.center(), 16.0, &cand.name, 0.0);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 2.0;
                                ui.label(RichText::new(&cand.name).font(menu::semibold(15.0)).color(Color32::WHITE));
                                ui.label(RichText::new(format!("{} {}  ·  {} {}", tr_en("Skill"), cand.skill, money(career::wage(cand.skill)), tr_en("an hour"))).font(menu::caption()).color(DIM));
                            });
                            menu::at_right(ui, ("cand", cand.name.as_str()), |ui| {
                                let fee = career::signing_fee(cand.skill);
                                if widgets::action(ui, &format!("{} {}", tr_en("Hire"), money_short(fee)), false, self.career.money >= fee, 110.0, 30.0).clicked() {
                                    act = Some(Act::Hire(cand.clone()));
                                }
                            });
                        });
                    }
                });
            });
        });
        act
    }

    /// the finances: what came in, what went out, every line.
    fn career_finances(&mut self, ui: &mut egui::Ui, _w: f32, h: f32) -> Option<Act> {
        let mut act = None;
        let income: f64 = self.career.ledger.iter().filter(|e| e.amount > 0.0).map(|e| e.amount).sum();
        let spent: f64 = self.career.ledger.iter().filter(|e| e.amount < 0.0).map(|e| -e.amount).sum();
        widgets::figures(ui, &[(money(self.career.money), tr_en("Balance")), (money(income), tr_en("Income")), (money(spent), tr_en("Spending")), (self.career.jobs_done.to_string(), tr_en("Jobs"))]);
        ui.add_space(14.0);
        // the statement as tall as its lines, starting again a quiet link under it.
        card(ui, 0.0, |ui| {
            widgets::title(ui, tr_en("STATEMENT"));
            egui::ScrollArea::vertical().id_salt("career_ledger").max_height(h - 230.0).auto_shrink([false, true]).show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                for (k, e) in self.career.ledger.iter().rev().enumerate() {
                    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 34.0), Sense::hover());
                    let p = ui.painter();
                    if k % 2 == 1 {
                        p.rect_filled(r, 3.0, Color32::from_white_alpha(4));
                    }
                    p.text(pos2(r.left() + 10.0, r.center().y), Align2::LEFT_CENTER, profile::date(e.unix), menu::caption(), DIM);
                    let g = menu::one_line(p, &e.what, menu::label(), TEXT, r.width() - 360.0);
                    p.galley(pos2(r.left() + 170.0, r.center().y - g.size().y * 0.5), g, TEXT);
                    p.text(pos2(r.right() - 10.0, r.center().y), Align2::RIGHT_CENTER, money(e.amount), menu::semibold(14.5), if e.amount < 0.0 { BAD } else if e.amount > 0.0 { OK } else { DIM });
                }
            });
        });
        ui.add_space(8.0);
        if flightui::text_link(ui, tr_en("Start a new career")).on_hover_text(tr_en("Money, licences, aircraft and reputation start again. Your pilot level and logbook stay.")).clicked() {
            act = Some(Act::NewCareer);
        }
        act
    }

    /// the briefing before a job: the work, the route, the aircraft to take.
    fn career_brief(&mut self, ctx: &egui::Context) -> Option<Act> {
        let j = self.career.view.brief.clone()?;
        let mut act = None;
        let c = class_parse(&j.class);
        // the aircraft: owned and free ones, then the ones to rent.
        let free = self.career.free_planes();
        let mut opts: Vec<(&'static aircraft::Spec, bool)> = career::career_types().into_iter().filter(|s| class_of(s) == c && free.iter().any(|k| s.keys.first() == Some(&k.as_str()))).map(|s| (s, true)).collect();
        for s in career::career_types().into_iter().filter(|s| class_of(s) == c) {
            if !opts.iter().any(|o| std::ptr::eq(o.0, s)) {
                opts.push((s, false));
            }
        }
        let picked = self.career.view.brief_plane.clone().or_else(|| opts.first().map(|o| o.0.keys[0].to_string()));
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::unique("brief_dim")).order(egui::Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
            if ui.allocate_response(screen.size(), Sense::click()).clicked() {
                act = Some(Act::CloseBrief);
            }
            ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(170));
        });
        egui::Area::new(egui::Id::unique("brief")).order(egui::Order::Foreground).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            menu::panel().inner_margin(24.0).show(ui, |ui| {
                ui.set_width(760.0);
                chip_ui(ui, &tr_en(j.kind.name()).to_uppercase(), j.kind.colour());
                ui.add(egui::Label::new(RichText::new(job_title(&j)).font(menu::bold(28.0)).color(Color32::WHITE)).wrap());
                ui.label(RichText::new(format!("{}  ·  {}", j.client, tr_en(j.kind.blurb()))).font(menu::label()).color(DIM));
                ui.add_space(14.0);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 20.0;
                    ui.allocate_ui_with_layout(vec2(340.0, 10.0), Layout::top_down(egui::Align::Min), |ui| {
                        let (sr, _) = ui.allocate_exact_size(vec2(340.0, 200.0), Sense::hover());
                        route_sketch(ui.painter(), sr, &j);
                        ui.add_space(10.0);
                        widgets::line(ui, tr_en("Pays"), &money(j.pay), ACCENT);
                        widgets::line(ui, tr_en("Distance"), &format!("{:.0} nm", j.dist_nm), Color32::WHITE);
                        widgets::line(ui, tr_en("Load"), &load_text(&j), Color32::WHITE);
                        widgets::line(ui, tr_en("Time allowed"), &profile::hhmm(j.limit_s), Color32::WHITE);
                    });
                    ui.allocate_ui_with_layout(vec2(352.0, 10.0), Layout::top_down(egui::Align::Min), |ui| {
                        widgets::title(ui, tr_en("OBJECTIVES"));
                        let f = career::Flight::preview(j.clone());
                        for (k, (what, _)) in career::objectives(&f).into_iter().enumerate() {
                            ui.horizontal_top(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(22.0, 20.0), Sense::hover());
                                ui.painter().text(r.center(), Align2::CENTER_CENTER, (k + 1).to_string(), menu::bold(13.0), ACCENT);
                                ui.add(egui::Label::new(RichText::new(what).font(menu::label()).color(TEXT)).wrap());
                            });
                        }
                        ui.add_space(6.0);
                        menu::hint(ui, tr_en("Marked on the landing, the timing, passenger comfort, procedures and care of the aircraft. Stars set the pay."));
                        ui.add_space(12.0);
                        widgets::title(ui, tr_en("AIRCRAFT"));
                        for (s, owned) in &opts {
                            let key = s.keys[0];
                            let on = picked.as_deref() == Some(key);
                            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
                            let p = ui.painter();
                            p.rect_filled(r, menu::R_WIDGET, if on { menu::CHOSEN } else { menu::RAISED });
                            p.rect_stroke(r, menu::R_WIDGET, Stroke::new(1.0, if on { ACCENT } else { menu::LINE_HI }), egui::StrokeKind::Inside);
                            p.circle_stroke(pos2(r.left() + 16.0, r.center().y), 6.0, Stroke::new(1.5, if on { ACCENT } else { DIM }));
                            if on {
                                p.circle_filled(pos2(r.left() + 16.0, r.center().y), 3.0, ACCENT);
                            }
                            p.text(pos2(r.left() + 32.0, r.center().y), Align2::LEFT_CENTER, s.name, menu::medium(14.5), Color32::WHITE);
                            let est = (j.dist_nm / c.cruise_kt() + 0.25) * c.rent();
                            let right = if *owned { format!("{}  ·  {:.0}%", tr_en("Yours"), self.career.condition(key) * 100.0) } else { format!("{} ~{}", tr_en("Rent"), money(est)) };
                            p.text(pos2(r.right() - 12.0, r.center().y), Align2::RIGHT_CENTER, right, menu::caption(), if *owned { ACCENT } else { DIM });
                            if resp.clicked() {
                                self.career.view.brief_plane = Some(key.to_string());
                            }
                        }
                    });
                });
                ui.add_space(18.0);
                ui.horizontal(|ui| {
                    if widgets::action(ui, tr_en("Start flight"), true, !opts.is_empty(), 260.0, 48.0).clicked() {
                        act = Some(Act::Fly(j.clone(), picked.clone()));
                    }
                    ui.add_space(8.0);
                    if widgets::action(ui, tr_en("Cancel"), false, true, 130.0, 48.0).clicked() {
                        act = Some(Act::CloseBrief);
                    }
                    ui.add_space(12.0);
                    menu::hint(ui, tr_en("You start at a stand, engine running, fuel and load aboard, the route in the flight plan."));
                });
            });
        });
        act
    }

    /// the job card in flight: the objectives, where to, the time left; the debrief at the end.
    pub(crate) fn job_card(&mut self, ctx: &egui::Context) {
        let Some(f) = self.career_flight.as_ref() else { return };
        if f.done {
            let Some(d) = self.career.view.debrief.clone() else {
                self.career_flight = None;
                return;
            };
            if debrief_window(ctx, &d, true) {
                // back to the career's screen, where the debrief waits.
                self.to_globe();
                self.menu.screen = crate::menu::Screen::Career;
                self.career.view.tab = Tab::Home;
            } else if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.career_flight = None;
                self.career.view.debrief = None;
            }
            return;
        }
        let mut abandon = false;
        let db = Db::get();
        let (la, lo, _) = geo::pos_to_lla(self.plane.pos);
        let here = (la.to_degrees(), lo.to_degrees());
        let target = match (f.job.via, f.via_done) {
            (Some(v), false) => Some(v),
            _ => db.find(&f.job.to).map(|i| (db.apts[i as usize].lat, db.apts[i as usize].lon)),
        };
        let left_nm = target.map(|t| career::nm_between(here, t));
        let left_s = f.job.limit_s - (self.time - f.t0);
        let top = if self.hud_on() { flightui::right_top(ctx) } else { 24.0 };
        let objs = career::objectives(f);
        let job = f.job.clone();
        egui::Area::new(egui::Id::unique("job_card")).anchor(Align2::RIGHT_TOP, [-14.0, top]).show(ctx, |ui| {
            flightui::panel(ui, flightui::glass().inner_margin(egui::Margin::symmetric(16, 12)), |ui| {
                ui.set_width(300.0);
                ui.spacing_mut().item_spacing.y = 4.0;
                ui.horizontal(|ui| {
                    flightui::tag(ui, &tr_en(job.kind.name()).to_uppercase());
                    menu::at_right(ui, "job_pay", |ui| {
                        ui.label(RichText::new(money(job.pay)).font(menu::bold(17.0)).color(ACCENT));
                    });
                });
                ui.add(egui::Label::new(RichText::new(job_title(&job)).font(menu::semibold(16.0)).color(Color32::WHITE)).wrap());
                ui.add_space(4.0);
                // only the objective to do now, and a bar for the rest.
                let done_n = objs.iter().filter(|o| o.1).count();
                if let Some((what, _)) = objs.iter().find(|o| !o.1) {
                    ui.add(egui::Label::new(RichText::new(what).font(menu::medium(15.0)).color(Color32::WHITE)).wrap());
                    ui.add_space(4.0);
                }
                flightui::meter(ui, done_n as f32 / objs.len().max(1) as f32, ACCENT);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if let Some(d) = left_nm {
                        ui.label(RichText::new(format!("{d:.1} nm")).font(menu::semibold(15.0)).color(Color32::WHITE));
                        ui.label(RichText::new(tr_en("to go")).font(menu::caption()).color(flightui::SOFT));
                        ui.add_space(10.0);
                    }
                    let col = if left_s < 0.0 { BAD } else if left_s < 600.0 { flightui::AMBER } else { Color32::WHITE };
                    let t = if left_s >= 0.0 { profile::hhmm(left_s) } else { format!("+{}", profile::hhmm(-left_s)) };
                    ui.label(RichText::new(t).font(menu::semibold(15.0)).color(col));
                    ui.label(RichText::new(if left_s >= 0.0 { tr_en("left") } else { tr_en("late") }).font(menu::caption()).color(flightui::SOFT));
                    menu::at_right(ui, "job_abandon", |ui| {
                        if flightui::text_link(ui, tr_en("Abandon job")).clicked() {
                            abandon = true;
                        }
                    });
                });
            });
        });
        if abandon {
            let k = self.career.rep_of(job.kind);
            self.career.rep_by.insert(job.kind, (k - 20.0).max(0.0));
            self.career.rep = (self.career.rep - 20.0).max(0.0);
            self.career.active = None;
            self.career.save();
            self.career_flight = None;
        }
    }
}

/// a licence as a row: its badge, its name and what it opens, and for one not held yet how far along it is.
fn lic_row(ui: &mut egui::Ui, l: Lic, has: bool, state: Option<(String, Color32)>) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, tr_en(l.name())));
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered() || resp.has_focus());
    let p = ui.painter();
    if lit > 0.0 {
        p.rect_filled(r.expand2(vec2(8.0, 0.0)), menu::R_WIDGET, Color32::from_white_alpha((8.0 * lit) as u8));
    }
    let c = pos2(r.left() + 20.0, r.center().y);
    p.circle_filled(c, 18.0, if has { Color32::from_rgb(52, 42, 8) } else { menu::SURFACE });
    p.circle_stroke(c, 18.0, Stroke::new(1.5, if has { ACCENT } else { menu::LINE_HI }));
    p.text(c, Align2::CENTER_CENTER, l.short(), menu::bold(11.0), if has { ACCENT } else { TEXT });
    let right_w = state.as_ref().map_or(0.0, |(s, _)| p.layout_no_wrap(s.clone(), menu::label(), DIM).size().x + 16.0);
    let room = r.width() - 52.0 - right_w;
    let ng = menu::one_line(p, tr_en(l.name()), menu::semibold(15.0), Color32::WHITE, room);
    p.galley(pos2(r.left() + 52.0, r.center().y - 2.0 - ng.size().y), ng, Color32::WHITE);
    let og = menu::one_line(p, tr_en(l.opens()), menu::caption(), DIM, room);
    p.galley(pos2(r.left() + 52.0, r.center().y + 2.0), og, DIM);
    if let Some((s, col)) = state {
        p.text(pos2(r.right(), r.center().y), Align2::RIGHT_CENTER, s, menu::label(), col);
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// one job on the board: its kind, where, the load, the pay.
fn job_row(ui: &mut egui::Ui, j: &Job, on: bool, locked: Option<&String>) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 80.0), Sense::click());
    let lit = menu::ease(ui.ctx(), resp.id, resp.hovered());
    let p = ui.painter();
    p.rect_filled(r, menu::R_PANEL, if on { menu::mix(menu::PANEL, menu::CHOSEN, 0.6) } else { menu::mix(menu::PANEL, menu::RAISED, lit) });
    p.rect_stroke(r, menu::R_PANEL, Stroke::new(1.0, if on { ACCENT.gamma_multiply(0.7) } else { menu::LINE_HI }), egui::StrokeKind::Inside);
    let col = j.kind.colour();
    let fade = if locked.is_some() { 0.5 } else { 1.0 };
    let cr = chip(p, pos2(r.left() + 18.0, r.top() + 20.0), &tr_en(j.kind.name()).to_uppercase(), col.gamma_multiply(fade));
    tier_dots(p, pos2(cr.right() + 12.0, r.top() + 20.0), j.tier, col.gamma_multiply(fade));
    let tg = menu::one_line(p, &job_title(j), menu::semibold(16.0), Color32::WHITE.gamma_multiply(fade), r.width() - 200.0);
    p.galley(pos2(r.left() + 18.0, r.top() + 34.0), tg, Color32::WHITE);
    let sub = format!("{:.0} nm  ·  {}  ·  {}  ·  {}", j.dist_nm, load_text(j), profile::hhmm(j.limit_s), j.client);
    let sg = menu::one_line(p, &sub, menu::caption(), DIM, r.width() - 200.0);
    p.galley(pos2(r.left() + 18.0, r.top() + 56.0), sg, DIM);
    p.text(pos2(r.right() - 16.0, r.top() + 28.0), Align2::RIGHT_CENTER, money(j.pay), menu::bold(21.0), if locked.is_some() { DIM } else { ACCENT });
    if let Some(why) = locked {
        let g = menu::one_line(p, why, menu::caption(), DIM, 170.0);
        let gw = g.size().x;
        p.galley(pos2(r.right() - 16.0 - gw, r.top() + 50.0), g, DIM);
        lock(p, pos2(r.right() - 26.0 - gw, r.top() + 57.0), 8.0, FAINT);
    }
    resp
}

/// the debrief: stars, the marks, the money, the reputation and experience.
/// true when the pilot is done with it.
fn debrief_window(ctx: &egui::Context, d: &Debrief, in_flight: bool) -> bool {
    let mut close = false;
    let screen = ctx.content_rect();
    egui::Area::new(egui::Id::unique("debrief_dim")).order(egui::Order::Middle).fixed_pos(screen.min).show(ctx, |ui| {
        ui.allocate_response(screen.size(), Sense::click());
        ui.painter().rect_filled(screen, 0.0, Color32::from_black_alpha(175));
    });
    egui::Area::new(egui::Id::unique("debrief")).order(egui::Order::Foreground).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        menu::panel().inner_margin(24.0).show(ui, |ui| {
            ui.set_width(720.0);
            chip_ui(ui, &tr_en(d.kind.name()).to_uppercase(), d.kind.colour());
            ui.label(RichText::new(if d.ok { tr_en("Job complete") } else { tr_en("Job failed") }).font(menu::bold(34.0)).color(if d.ok { Color32::WHITE } else { BAD }));
            ui.label(RichText::new(&d.title).font(menu::label()).color(DIM));
            ui.add_space(10.0);
            if d.ok {
                let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
                pilot::stars(ui.painter(), pos2(r.left() + 16.0, r.center().y), 15.0, d.stars);
                ui.painter().text(pos2(r.left() + 200.0, r.center().y), Align2::LEFT_CENTER, &d.note, menu::medium(16.0), TEXT);
            } else {
                ui.label(RichText::new(&d.note).font(menu::medium(16.0)).color(TEXT));
            }
            ui.add_space(14.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 28.0;
                ui.allocate_ui_with_layout(vec2(330.0, 10.0), Layout::top_down(egui::Align::Min), |ui| {
                    widgets::title(ui, tr_en("MARKS"));
                    for (name, m, note) in &d.marks {
                        let (r, _) = ui.allocate_exact_size(vec2(330.0, 44.0), Sense::hover());
                        let p = ui.painter();
                        let col = if *m >= 85 { OK } else if *m >= 60 { ACCENT } else { BAD };
                        p.text(pos2(r.left(), r.top() + 9.0), Align2::LEFT_CENTER, name, menu::medium(14.5), Color32::WHITE);
                        p.text(pos2(r.right(), r.top() + 9.0), Align2::RIGHT_CENTER, m.to_string(), menu::bold(16.0), col);
                        meter(p, Rect::from_min_size(pos2(r.left(), r.top() + 22.0), vec2(r.width(), 4.0)), *m as f32 / 100.0, col);
                        let g = menu::one_line(p, note, menu::caption(), DIM, r.width());
                        p.galley(pos2(r.left(), r.top() + 29.0), g, DIM);
                    }
                    if d.marks.is_empty() {
                        widgets::empty(ui, "—");
                    }
                });
                ui.allocate_ui_with_layout(vec2(300.0, 10.0), Layout::top_down(egui::Align::Min), |ui| {
                    widgets::title(ui, tr_en("PAY"));
                    for (what, amt) in &d.lines {
                        widgets::line(ui, what, &money(*amt), if *amt < 0.0 { BAD } else { Color32::WHITE });
                    }
                    menu::rule(ui);
                    let net = d.net();
                    widgets::line(ui, tr_en("Net"), &money(net), if net < 0.0 { BAD } else { ACCENT });
                    ui.add_space(12.0);
                    let (r, _) = ui.allocate_exact_size(vec2(300.0, 56.0), Sense::hover());
                    let p = ui.painter();
                    widgets::figure(p, r.left_top(), &format!("{:+.0}", d.rep), tr_en("reputation"), 24.0, if d.rep < 0.0 { BAD } else { Color32::WHITE });
                    widgets::figure(p, r.left_top() + vec2(150.0, 0.0), &format!("+{}", d.xp), "XP", 24.0, ACCENT);
                });
            });
            ui.add_space(18.0);
            let label = if in_flight { tr_en("Back to career") } else { tr_en("Continue") };
            if widgets::action(ui, label, true, true, 260.0, 46.0).clicked() {
                close = true;
            }
            if in_flight {
                menu::hint(ui, tr_en("Esc: keep flying here."));
            }
        });
    });
    close
}
