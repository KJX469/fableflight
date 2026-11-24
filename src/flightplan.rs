//! flight plans made on the start screen, flown by hand or by gps.
//! waypoints are airports, fixes, vors, ndbs or lat,lon points, with airways and us procedures.
//! legs are great circles, courses true.

use crate::{airports, geo, nav, navdata};

/// what a waypoint is.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Kind {
    Airport,
    Vor,
    Ndb,
    /// a named fix.
    Fix,
    /// a picked or typed point.
    User,
    /// a leg with no fix, like a heading to an altitude.
    Leg,
}

impl Kind {
    pub fn tag(self) -> &'static str {
        match self {
            Kind::Airport => "APT",
            Kind::Vor => "VOR",
            Kind::Ndb => "NDB",
            Kind::Fix => "FIX",
            Kind::User => "WPT",
            Kind::Leg => "LEG",
        }
    }
}

/// how a leg is flown.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Leg {
    /// to the fix along the route, or straight to it when direct.
    Fix { direct: bool },
    /// a heading until an altitude in feet.
    Climb { course: f64, until: f64 },
    /// a heading until the next leg is met.
    Intercept { course: f64 },
    /// a heading until the pilot goes direct, as atc vectors.
    Vectors { course: f64 },
}

/// a procedure phase.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Phase {
    Sid,
    Star,
    App,
}

impl Phase {
    pub fn tag(self) -> &'static str {
        match self {
            Phase::Sid => "SID",
            Phase::Star => "STAR",
            Phase::App => "APP",
        }
    }

    fn kind(self) -> char {
        match self {
            Phase::Sid => 'D',
            Phase::Star => 'E',
            Phase::App => 'F',
        }
    }
}

/// an altitude constraint in feet.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Alt {
    /// + at or above, - at or below, b between, anything else at.
    pub desc: char,
    pub a1: f64,
    pub a2: Option<f64>,
}

impl Alt {
    pub fn text(&self) -> String {
        let f = |a: f64| if a >= 18000.0 { format!("FL{:03.0}", a / 100.0) } else { format!("{a:.0}") };
        match (self.desc, self.a2) {
            ('+', _) => format!("{}A", f(self.a1)),
            ('-', _) => format!("{}B", f(self.a1)),
            ('B', Some(b)) => format!("{}B {}A", f(self.a1), f(b)),
            _ => f(self.a1),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Waypoint {
    /// short name, like oak or wp3.
    pub ident: String,
    /// long name.
    pub name: String,
    pub kind: Kind,
    /// degrees. heading legs have a drawn spot only.
    pub lat: f64,
    pub lon: f64,
    pub leg: Leg,
    pub alt: Option<Alt>,
    /// speed limit in knots.
    pub speed: Option<f64>,
    /// the inbound course a coded leg asks for.
    pub course: Option<f64>,
    /// the procedure it came from, and its name.
    pub phase: Option<(Phase, String)>,
}

impl Waypoint {
    fn new(ident: String, name: String, kind: Kind, lat: f64, lon: f64) -> Self {
        Waypoint { ident, name, kind, lat, lon, leg: Leg::Fix { direct: false }, alt: None, speed: None, course: None, phase: None }
    }

    pub fn user(lat: f64, lon: f64, n: usize) -> Self {
        Waypoint::new(format!("WP{n}"), crate::menu::latlon(lat, lon), Kind::User, lat, lon)
    }

    pub fn airport(i: u32) -> Self {
        let a = &airports::Db::get().apts[i as usize];
        Waypoint::new(a.ident.clone(), a.name.clone(), Kind::Airport, a.lat, a.lon)
    }

    pub fn station(s: &nav::Station) -> Self {
        let kind = if s.kind == nav::Kind::Ndb { Kind::Ndb } else { Kind::Vor };
        Waypoint::new(s.ident.clone(), s.name.clone(), kind, s.lat, s.lon)
    }

    pub fn fix(ident: &str, lat: f64, lon: f64) -> Self {
        Waypoint::new(ident.to_string(), crate::menu::latlon(lat, lon), Kind::Fix, lat, lon)
    }

    pub fn at(&self) -> (f64, f64) {
        (self.lat, self.lon)
    }

    /// a leg flown on a heading rather than to a fix.
    pub fn is_heading(&self) -> bool {
        !matches!(self.leg, Leg::Fix { .. })
    }
}

/// great circle distance in metres.
pub fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (p1, p2) = (a.0.to_radians(), b.0.to_radians());
    let (dp, dl) = (p2 - p1, (b.1 - a.1).to_radians());
    let h = (dp * 0.5).sin().powi(2) + p1.cos() * p2.cos() * (dl * 0.5).sin().powi(2);
    2.0 * geo::R * h.sqrt().min(1.0).asin()
}

/// initial course from a to b, degrees true.
pub fn course(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (p1, p2) = (a.0.to_radians(), b.0.to_radians());
    let dl = (b.1 - a.1).to_radians();
    let y = dl.sin() * p2.cos();
    let x = p1.cos() * p2.sin() - p1.sin() * p2.cos() * dl.cos();
    y.atan2(x).to_degrees().rem_euclid(360.0)
}

/// point a fraction f of the way from a to b.
pub fn between(a: (f64, f64), b: (f64, f64), f: f64) -> (f64, f64) {
    let d = dist(a, b) / geo::R;
    if d < 1e-9 {
        return a;
    }
    let (p1, l1, p2, l2) = (a.0.to_radians(), a.1.to_radians(), b.0.to_radians(), b.1.to_radians());
    let (ka, kb) = (((1.0 - f) * d).sin() / d.sin(), (f * d).sin() / d.sin());
    let x = ka * p1.cos() * l1.cos() + kb * p2.cos() * l2.cos();
    let y = ka * p1.cos() * l1.sin() + kb * p2.cos() * l2.sin();
    let z = ka * p1.sin() + kb * p2.sin();
    (z.atan2(x.hypot(y)).to_degrees(), y.atan2(x).to_degrees())
}

/// the point d metres from a on a course.
pub fn ahead(a: (f64, f64), crs: f64, d: f64) -> (f64, f64) {
    let (p1, l1, t, k) = (a.0.to_radians(), a.1.to_radians(), crs.to_radians(), d / geo::R);
    let p2 = (p1.sin() * k.cos() + p1.cos() * k.sin() * t.cos()).asin();
    let l2 = l1 + (t.sin() * k.sin() * p1.cos()).atan2(k.cos() - p1.sin() * p2.sin());
    (p2.to_degrees(), (l2.to_degrees() + 540.0).rem_euclid(360.0) - 180.0)
}

fn wrap(a: f64) -> f64 {
    (a + 180.0).rem_euclid(360.0) - 180.0
}

/// where we are on the active leg.
#[derive(Clone, Debug, PartialEq)]
pub struct Steer {
    /// active waypoint.
    pub to: usize,
    /// metres to go and bearing to it.
    pub dist: f64,
    pub bearing: f64,
    /// leg course abeam us.
    pub course: f64,
    /// cross track in metres, right is positive.
    pub xtk: f64,
    /// track to fly.
    pub track: f64,
}

/// a chosen procedure.
#[derive(Clone, Debug, PartialEq)]
pub struct Pick {
    pub apt: String,
    pub name: String,
    /// runway transition, like rw28l.
    pub rwy: Option<String>,
    /// enroute or approach transition.
    pub trans: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    /// the route, destination last.
    pub wps: Vec<Waypoint>,
    /// active waypoint, wps.len() when done.
    pub active: usize,
    /// where the active leg starts.
    pub from: Option<(f64, f64)>,
    /// cruise altitude in feet.
    pub cruise_ft: f64,
    pub sid: Option<Pick>,
    pub star: Option<Pick>,
    pub app: Option<Pick>,
}

/// one route token to a waypoint.
pub fn resolve(token: &str, near: Option<(f64, f64)>, n: usize) -> Result<Waypoint, String> {
    let t = token.trim().trim_matches(|c| c == ',' || c == ';');
    if t.is_empty() {
        return Err(String::new());
    }
    // lat,lon or lat/lon.
    if let Some((a, b)) = t.split_once([',', '/']) {
        if let (Ok(lat), Ok(lon)) = (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
            if lat.abs() <= 90.0 && lon.abs() <= 180.0 {
                return Ok(Waypoint::user(lat, lon, n));
            }
        }
        return Err(t.to_string());
    }
    let up = t.to_ascii_uppercase();
    let db = airports::Db::get();
    let fixes = navdata::Db::get();
    // icao code first.
    if up.len() >= 4 {
        if let Some(i) = db.apts.iter().position(|a| a.ident == up) {
            return Ok(Waypoint::airport(i as u32));
        }
    }
    // five letter names are fixes.
    if up.len() == 5 {
        if let Some((la, lo)) = fixes.fix(&up, near) {
            return Ok(Waypoint::fix(&up, la, lo));
        }
    }
    // then the nearest navaid with that ident.
    let stations: Vec<&nav::Station> = nav::db().iter().filter(|s| s.ident == up).collect();
    let nearest = |within: f64| {
        let r = near?;
        stations.iter().map(|s| (*s, dist(r, (s.lat, s.lon)))).filter(|(_, d)| *d < within).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(s, _)| s)
    };
    if let Some(s) = nearest(900_000.0) {
        return Ok(Waypoint::station(s));
    }
    // then iata, then any fix or navaid.
    if let Some(i) = db.find(&up) {
        return Ok(Waypoint::airport(i));
    }
    if let Some((la, lo)) = fixes.fix(&up, near) {
        return Ok(Waypoint::fix(&up, la, lo));
    }
    if let Some(s) = nearest(f64::INFINITY).or(stations.first().copied()) {
        return Ok(Waypoint::station(s));
    }
    Err(t.to_string())
}

/// turns coded legs into waypoints. heading legs get a drawn spot a little way out.
pub fn from_legs(legs: &[navdata::RawLeg], phase: Phase, name: &str, start: Option<(f64, f64)>) -> Vec<Waypoint> {
    let mut out: Vec<Waypoint> = Vec::new();
    let mut prev = start;
    let tag = Some((phase, name.to_string()));
    for l in legs {
        let alt = l.alt1.map(|a1| Alt { desc: l.alt_desc, a1, a2: l.alt2 });
        let has_fix = !l.fix.is_empty();
        let fix_new = has_fix && out.last().is_none_or(|w| w.ident != l.fix);
        match l.pt.as_str() {
            "VA" | "CA" | "FA" | "VM" | "FM" | "VI" | "CI" | "VD" | "CD" | "VR" | "CR" => {
                if has_fix && fix_new {
                    let mut w = Waypoint::fix(&l.fix, l.lat, l.lon);
                    w.phase = tag.clone();
                    push(&mut out, &mut prev, w);
                }
                let Some(crs) = l.course else { continue };
                let Some(p) = prev else { continue };
                let (leg, ident, d) = match l.pt.as_str() {
                    "VA" | "CA" | "FA" => match l.alt1 {
                        Some(a) => (Leg::Climb { course: crs, until: a }, format!("({a:.0})"), 3700.0),
                        None => (Leg::Intercept { course: crs }, "(INTC)".into(), 7400.0),
                    },
                    "VM" | "FM" => (Leg::Vectors { course: crs }, "(VECT)".into(), 11000.0),
                    _ => (Leg::Intercept { course: crs }, "(INTC)".into(), 7400.0),
                };
                let (la, lo) = ahead(p, crs, d);
                let mut w = Waypoint::new(ident, format!("{} {crs:03.0}", t_heading()), Kind::Leg, la, lo);
                w.leg = leg;
                w.alt = alt;
                w.speed = l.speed;
                w.phase = tag.clone();
                push(&mut out, &mut prev, w);
            }
            _ if has_fix => {
                if !fix_new {
                    // the same fix twice: keep the tighter data.
                    if let Some(w) = out.last_mut() {
                        w.alt = alt.or(w.alt);
                        w.speed = l.speed.or(w.speed);
                    }
                    continue;
                }
                let mut w = Waypoint::fix(&l.fix, l.lat, l.lon);
                if l.fix.starts_with("RW") {
                    w.kind = Kind::Airport;
                    w.name = format!("runway {}", &l.fix[2..]);
                }
                w.leg = Leg::Fix { direct: l.pt == "DF" };
                w.alt = alt;
                w.speed = l.speed;
                w.course = if l.pt == "CF" { l.course } else { None };
                w.phase = tag.clone();
                push(&mut out, &mut prev, w);
            }
            _ => {}
        }
    }
    out
}

fn push(out: &mut Vec<Waypoint>, prev: &mut Option<(f64, f64)>, w: Waypoint) {
    *prev = Some(w.at());
    out.push(w);
}

fn t_heading() -> &'static str {
    crate::t!("plan.heading")
}

impl Plan {
    pub fn new() -> Self {
        Plan { cruise_ft: 5500.0, ..Default::default() }
    }

    pub fn is_empty(&self) -> bool {
        self.wps.is_empty()
    }

    /// last point of the route so far.
    fn tail(&self, dep: Option<(f64, f64)>) -> Option<(f64, f64)> {
        self.wps.last().map(|w| w.at()).or(dep)
    }

    /// adds a route string with airways, returns the tokens it could not read.
    pub fn add_route(&mut self, text: &str, dep: Option<(f64, f64)>) -> Vec<String> {
        let mut bad = Vec::new();
        let toks: Vec<String> = text.split_whitespace().map(|t| t.to_ascii_uppercase()).filter(|t| t != "DCT").collect();
        let ways = navdata::Db::get();
        // where the route goes in: before the star, approach and destination when those are set.
        let mut at = self.insert_point();
        for (k, tok) in toks.iter().enumerate() {
            // an airway between the fix before and the fix after.
            if ways.airways.contains_key(tok.as_str()) {
                let before = if at > 0 { self.wps.get(at - 1).map(|w| w.ident.clone()) } else { None };
                match (before, toks.get(k + 1)) {
                    (Some(a), Some(b)) => match ways.airway(tok, &a, b) {
                        Some(fixes) => {
                            for (id, la, lo) in fixes {
                                self.wps.insert(at, Waypoint::fix(&id, la, lo));
                                at += 1;
                            }
                        }
                        None => bad.push(format!("{tok} ({a} {b})")),
                    },
                    _ => bad.push(tok.clone()),
                }
                continue;
            }
            let near = if at > 0 { self.wps.get(at - 1).map(|w| w.at()) } else { None }.or(self.tail(dep));
            match resolve(tok, near, self.next_user()) {
                Ok(w) => {
                    self.wps.insert(at, w);
                    at += 1;
                }
                Err(e) if !e.is_empty() => bad.push(e),
                Err(_) => {}
            }
        }
        bad
    }

    /// where new enroute waypoints go.
    fn insert_point(&self) -> usize {
        self.wps.iter().position(|w| matches!(w.phase, Some((Phase::Star | Phase::App, _)))).unwrap_or_else(|| self.dest_index().unwrap_or(self.wps.len()))
    }

    /// the destination airport, the last plain airport of the route.
    pub fn dest_index(&self) -> Option<usize> {
        let i = self.wps.len().checked_sub(1)?;
        (self.wps[i].kind == Kind::Airport && self.wps[i].phase.is_none()).then_some(i)
    }

    pub fn destination(&self) -> Option<&Waypoint> {
        self.dest_index().map(|i| &self.wps[i])
    }

    /// sets or clears a procedure, putting its legs in their place.
    pub fn set_proc(&mut self, phase: Phase, pick: Option<Pick>, dep: Option<(f64, f64)>) {
        self.wps.retain(|w| w.phase.as_ref().is_none_or(|p| p.0 != phase));
        let slot = match phase {
            Phase::Sid => &mut self.sid,
            Phase::Star => &mut self.star,
            Phase::App => &mut self.app,
        };
        *slot = pick.clone();
        self.active = self.active.min(self.wps.len());
        let Some(p) = pick else { return };
        let legs = navdata::Db::get().assemble(&p.apt, phase.kind(), &p.name, p.rwy.as_deref(), p.trans.as_deref());
        let at = match phase {
            Phase::Sid => 0,
            Phase::Star => self.wps.iter().position(|w| matches!(w.phase, Some((Phase::App, _)))).unwrap_or_else(|| self.dest_index().unwrap_or(self.wps.len())),
            Phase::App => self.dest_index().unwrap_or(self.wps.len()),
        };
        let start = if at == 0 { dep } else { Some(self.wps[at - 1].at()) };
        let label = navdata::label(phase.kind(), &p.name);
        let new = from_legs(&legs, phase, &label, start);
        self.wps.splice(at..at, new);
    }

    /// next free wp number.
    pub fn next_user(&self) -> usize {
        self.wps.iter().filter_map(|w| w.ident.strip_prefix("WP").and_then(|n| n.parse::<usize>().ok())).max().unwrap_or(0) + 1
    }

    pub fn push_point(&mut self, lat: f64, lon: f64) {
        let n = self.next_user();
        let at = self.insert_point();
        self.wps.insert(at, Waypoint::user(lat, lon, n));
    }

    pub fn push(&mut self, w: Waypoint) {
        // an airport goes last as the destination, anything else before the arrival.
        let at = if w.kind == Kind::Airport { self.wps.len() } else { self.insert_point() };
        self.wps.insert(at, w);
    }

    pub fn remove(&mut self, i: usize) {
        if i < self.wps.len() {
            self.wps.remove(i);
            if self.active > i {
                self.active -= 1;
            }
            self.active = self.active.min(self.wps.len());
        }
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.wps.len() && b < self.wps.len() {
            self.wps.swap(a, b);
        }
    }

    pub fn clear(&mut self) {
        self.wps.clear();
        self.active = 0;
        self.from = None;
        self.sid = None;
        self.star = None;
        self.app = None;
    }

    /// reverses the route, dropping the procedures that no longer fit.
    pub fn reverse(&mut self) {
        self.wps.retain(|w| w.phase.is_none());
        self.sid = None;
        self.star = None;
        self.app = None;
        self.wps.reverse();
    }

    /// legs as from, to, metres, course.
    pub fn legs(&self, dep: Option<(f64, f64)>) -> Vec<((f64, f64), (f64, f64), f64, f64)> {
        let pts: Vec<(f64, f64)> = dep.into_iter().chain(self.wps.iter().map(|w| w.at())).collect();
        pts.windows(2).map(|w| (w[0], w[1], dist(w[0], w[1]), course(w[0], w[1]))).collect()
    }

    /// total length in metres.
    pub fn total(&self, dep: Option<(f64, f64)>) -> f64 {
        self.legs(dep).iter().map(|l| l.2).sum()
    }

    /// starts the plan from a point.
    pub fn begin(&mut self, from: (f64, f64)) {
        self.active = 0;
        self.from = Some(from);
        // skip fixes we start on.
        while self.active + 1 < self.wps.len() && !self.wps[self.active].is_heading() && dist(from, self.wps[self.active].at()) < 3000.0 {
            self.from = Some(self.wps[self.active].at());
            self.active += 1;
        }
        self.activate(from);
    }

    /// direct to a waypoint.
    pub fn direct(&mut self, i: usize, at: (f64, f64)) {
        if i < self.wps.len() {
            self.active = i;
            self.from = Some(at);
        }
    }

    /// sets up the leg just made active.
    fn activate(&mut self, at: (f64, f64)) {
        let Some(w) = self.wps.get(self.active) else { return };
        match (w.leg, w.course) {
            (Leg::Fix { direct: true }, _) => self.from = Some(at),
            // a course to a fix is flown along its inbound line.
            (Leg::Fix { .. }, Some(c)) => self.from = Some(ahead(w.at(), c + 180.0, dist(at, w.at()).max(9000.0))),
            (Leg::Fix { .. }, None) => {}
            _ => self.from = Some(at),
        }
    }

    pub fn done(&self) -> bool {
        self.active >= self.wps.len()
    }

    /// active waypoint.
    pub fn target(&self) -> Option<&Waypoint> {
        self.wps.get(self.active)
    }

    /// position on the leg and the track to fly.
    pub fn steer(&self, at: (f64, f64)) -> Option<Steer> {
        let to = self.target()?;
        let b = to.at();
        let d_to = dist(at, b);
        let brg = course(at, b);
        if let Leg::Climb { course: c, .. } | Leg::Intercept { course: c } | Leg::Vectors { course: c } = to.leg {
            return Some(Steer { to: self.active, dist: d_to, bearing: brg, course: c, xtk: 0.0, track: c });
        }
        let a = self.from.unwrap_or(at);
        let leg = dist(a, b);
        if leg < 50.0 {
            return Some(Steer { to: self.active, dist: d_to, bearing: brg, course: brg, xtk: 0.0, track: brg });
        }
        // cross track and along track.
        let d13 = dist(a, at) / geo::R;
        let t13 = course(a, at).to_radians();
        let t12 = course(a, b).to_radians();
        let xt = (d13.sin() * (t13 - t12).sin()).clamp(-1.0, 1.0).asin();
        let xtk = xt * geo::R;
        let along = (d13.cos() / xt.cos()).clamp(-1.0, 1.0).acos() * geo::R * if (t13 - t12).cos() < 0.0 { -1.0 } else { 1.0 };
        let abeam = between(a, b, (along / leg).clamp(0.0, 1.0));
        let crs = if along >= leg - 1.0 { brg } else { course(abeam, b) };
        // 30 degrees per nm off, max 45.
        let intercept = (xtk / 1852.0 * 30.0).clamp(-45.0, 45.0);
        let mut track = (crs - intercept).rem_euclid(360.0);
        // close or past, fly straight at it.
        if d_to < 1500.0 || wrap(brg - crs).abs() > 90.0 {
            track = brg;
        }
        Some(Steer { to: self.active, dist: d_to, bearing: brg, course: crs, xtk, track })
    }

    /// moves to the next leg when this one is done, returns the one passed.
    pub fn sequence(&mut self, at: (f64, f64), gs: f64, alt_ft: f64) -> Option<usize> {
        let s = self.steer(at)?;
        let w = &self.wps[self.active];
        let next = self.wps.get(self.active + 1);
        let done = match w.leg {
            Leg::Climb { until, .. } => alt_ft >= until - 50.0,
            // the next fix's inbound line is met, or the fix has swung well off to the side.
            Leg::Intercept { course: c } => match next {
                Some(n) => {
                    let b = course(at, n.at());
                    n.course.is_some_and(|nc| wrap(b - nc).abs() < 3.0) || wrap(b - c).abs() > 60.0
                }
                None => false,
            },
            // vectors end when the pilot goes direct.
            Leg::Vectors { .. } => false,
            Leg::Fix { .. } => {
                let turn = match next {
                    Some(n) if !n.is_heading() => {
                        let change = wrap(n.course.unwrap_or_else(|| course(w.at(), n.at())) - s.course).abs().to_radians();
                        let r = gs * 60.0 / std::f64::consts::PI;
                        (r * (change * 0.5).tan()).min(8000.0)
                    }
                    _ => 0.0,
                };
                let reach = (turn + gs * 2.0).max(if next.is_none() { 800.0 } else { 400.0 });
                // passed abeam also counts.
                let passed = wrap(s.bearing - s.course).abs() > 100.0 && s.dist < 5000.0;
                s.dist < reach || passed
            }
        };
        if !done {
            return None;
        }
        let i = self.active;
        self.from = Some(if w.is_heading() { at } else { w.at() });
        self.active += 1;
        self.activate(at);
        Some(i)
    }

    /// seconds to the next waypoint and to the end.
    pub fn ete(&self, at: (f64, f64), gs: f64) -> Option<(f64, f64)> {
        let s = self.steer(at)?;
        if gs < 5.0 {
            return None;
        }
        let rest: f64 = self.wps[self.active..].windows(2).map(|w| dist(w[0].at(), w[1].at())).sum();
        Some((s.dist / gs, (s.dist + rest) / gs))
    }

    /// route as text, procedures by name.
    pub fn route_text(&self) -> String {
        let mut out: Vec<String> = Vec::new();
        if let Some(p) = &self.sid {
            out.push(match &p.trans { Some(t) => format!("{}.{t}", p.name), None => p.name.clone() });
        }
        out.extend(self.wps.iter().filter(|w| w.phase.is_none()).map(|w| if w.kind == Kind::User { format!("{:.4},{:.4}", w.lat, w.lon) } else { w.ident.clone() }));
        for p in [&self.star, &self.app].into_iter().flatten() {
            out.push(match &p.trans { Some(t) => format!("{t}.{}", p.name), None => p.name.clone() });
        }
        out.join(" ")
    }
}

/// short duration text.
pub fn hm(secs: f64) -> String {
    let s = secs.max(0.0).round() as u64;
    if s >= 3600 {
        format!("{} h {:02}", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{} min", s / 60)
    } else {
        format!("{s} s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn great_circle() {
        // sfo to lax is about 293 nm on 137.
        let (sfo, lax) = ((37.6188, -122.3750), (33.9425, -118.4081));
        let d = dist(sfo, lax) / 1852.0;
        assert!((d - 293.0).abs() < 3.0, "{d}");
        let c = course(sfo, lax);
        assert!((c - 137.0).abs() < 2.0, "{c}");
        let m = between(sfo, lax, 0.5);
        assert!((dist(sfo, m) - dist(m, lax)).abs() < 1.0);
        let p = ahead(sfo, 90.0, 10_000.0);
        assert!((dist(sfo, p) - 10_000.0).abs() < 1.0 && (course(sfo, p) - 90.0).abs() < 0.1);
    }

    #[test]
    fn coordinates_and_unknowns() {
        let w = resolve("37.5,-122.25", None, 3).unwrap();
        assert_eq!((w.kind, w.ident.as_str(), w.lat, w.lon), (Kind::User, "WP3", 37.5, -122.25));
        assert!(resolve("ZZZZZZZ", None, 1).is_err());
        assert!(resolve("91,0", None, 1).is_err());
        let f = resolve("SAHEY", Some((37.6, -122.4)), 1).unwrap();
        assert_eq!(f.kind, Kind::Fix);
    }

    #[test]
    fn steering_and_sequencing() {
        let mut p = Plan::new();
        p.wps.push(Waypoint::user(0.0, 1.0, 1));
        p.wps.push(Waypoint::user(1.0, 1.0, 2));
        p.begin((0.0, 0.0));
        // on the leg going east.
        let s = p.steer((0.0, 0.5)).unwrap();
        assert!(s.xtk.abs() < 1.0 && (s.track - 90.0).abs() < 0.5, "{s:?}");
        // left of the leg, turn right.
        let s = p.steer((0.02, 0.5)).unwrap();
        assert!(s.xtk < -2000.0 && s.track > 90.0 && s.track < 136.0, "{s:?}");
        // turns early onto the next leg.
        assert_eq!(p.sequence((0.0, 0.5), 50.0, 3000.0), None);
        assert_eq!(p.sequence((0.0, 0.995), 50.0, 3000.0), Some(0));
        assert_eq!(p.target().unwrap().ident, "WP2");
        assert_eq!(p.sequence((0.999, 1.0), 50.0, 3000.0), Some(1));
        assert!(p.done() && p.steer((1.0, 1.0)).is_none());
    }

    #[test]
    fn heading_legs() {
        let legs = vec![
            navdata::RawLeg { fix: String::new(), lat: 0.0, lon: 0.0, pt: "VA".into(), course: Some(90.0), alt_desc: '+', alt1: Some(500.0), alt2: None, speed: None },
            navdata::RawLeg { fix: "AAAAA".into(), lat: 0.0, lon: 0.3, pt: "DF".into(), course: None, alt_desc: '+', alt1: Some(5000.0), alt2: None, speed: None },
            navdata::RawLeg { fix: "AAAAA".into(), lat: 0.0, lon: 0.3, pt: "FM".into(), course: Some(10.0), alt_desc: ' ', alt1: None, alt2: None, speed: None },
        ];
        let mut p = Plan::new();
        p.wps = from_legs(&legs, Phase::Sid, "TEST1", Some((0.0, 0.0)));
        assert_eq!(p.wps.iter().map(|w| w.ident.as_str()).collect::<Vec<_>>(), vec!["(500)", "AAAAA", "(VECT)"]);
        assert_eq!(p.wps[1].alt.unwrap().text(), "5000A");
        p.begin((0.0, 0.0));
        // climb on the heading until 500 ft.
        assert_eq!(p.steer((0.0, 0.01)).unwrap().track, 90.0);
        assert_eq!(p.sequence((0.0, 0.01), 60.0, 300.0), None);
        assert_eq!(p.sequence((0.0, 0.01), 60.0, 520.0), Some(0));
        // then direct to the fix from where we are.
        assert_eq!(p.from, Some((0.0, 0.01)));
        assert_eq!(p.sequence((0.0, 0.299), 60.0, 3000.0), Some(1));
        // vectors never end by themselves.
        assert_eq!(p.steer((0.0, 0.3)).unwrap().track, 10.0);
        assert_eq!(p.sequence((0.5, 0.3), 60.0, 3000.0), None);
    }

    #[test]
    fn procedures_and_airways() {
        let ksfo = airports::Db::get().find("KSFO").unwrap();
        let mut p = Plan::new();
        p.push(Waypoint::airport(airports::Db::get().find("KLAX").unwrap()));
        let db = navdata::Db::get();
        let sid = db.names("KSFO", 'D').into_iter().next().unwrap();
        let rw = db.runway_trans("KSFO", 'D', &sid).into_iter().next();
        let dep = Waypoint::airport(ksfo).at();
        p.set_proc(Phase::Sid, Some(Pick { apt: "KSFO".into(), name: sid.clone(), rwy: rw, trans: None }), Some(dep));
        assert!(p.wps.len() > 2 && p.wps[0].phase.is_some());
        assert_eq!(p.destination().unwrap().ident, "KLAX");
        let n = p.wps.len();
        p.set_proc(Phase::Sid, None, Some(dep));
        assert!(p.wps.len() < n && p.sid.is_none());
        // an airway expands between its fixes.
        let mut q = Plan::new();
        let bad = q.add_route("PRB V25 SNS", Some(dep));
        assert!(bad.is_empty(), "{bad:?}");
        assert!(q.wps.len() > 2, "{:?}", q.wps.iter().map(|w| &w.ident).collect::<Vec<_>>());
    }

    #[test]
    fn editing() {
        let mut p = Plan::new();
        p.push_point(1.0, 2.0);
        p.push_point(3.0, 4.0);
        p.push_point(5.0, 6.0);
        assert_eq!(p.wps[2].ident, "WP3");
        p.active = 2;
        p.remove(0);
        assert_eq!((p.active, p.wps[0].ident.as_str()), (1, "WP2"));
        assert_eq!(p.next_user(), 4);
        assert_eq!(p.route_text(), "3.0000,4.0000 5.0000,6.0000");
        assert_eq!(hm(3900.0), "1 h 05");
        assert_eq!(hm(720.0), "12 min");
    }
}
