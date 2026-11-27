//! The flight log: what happens in a flight (take-off, each touchdown and how
//! it went, a crash), and the pilot's logbook of past flights, kept in
//! `logbook.json` next to settings.json.
//!
//! A `Tracker` is fed the aircraft at every physics step. It knows when the
//! wheels leave the ground and come back, grades each touchdown on how hard
//! it was (the sink rate just before the wheels touched), the load, the
//! bounces, the bank, the crab and where on the runway it came down, and puts
//! the flight in the logbook when it ends: parked after landing, crashed,
//! or left (a respawn, a new start, quitting).

use crate::{airports, geo};
use glam::DVec3;
use serde::{Deserialize, Serialize};

const FPM: f64 = 196.850_394;
const KT: f64 = 1.943_844;

/// The aircraft at one physics step.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub pos: DVec3,
    pub vel: DVec3,
    pub forward: DVec3,
    pub right: DVec3,
    pub on_ground: bool,
    pub g: f64,
}

impl Sample {
    fn lat_lon(&self) -> (f64, f64) {
        let (la, lo, _) = geo::pos_to_lla(self.pos);
        (la.to_degrees(), lo.to_degrees())
    }
    fn vs(&self) -> f64 {
        self.vel.dot(self.pos.normalize())
    }
    fn ground_speed(&self) -> f64 {
        let up = self.pos.normalize();
        (self.vel - up * self.vel.dot(up)).length()
    }
    /// degrees true, 0..360
    fn heading(&self) -> f64 {
        let (e, n, _) = geo::enu(self.pos);
        self.forward.dot(e).atan2(self.forward.dot(n)).to_degrees().rem_euclid(360.0)
    }
    /// degrees, right wing down positive
    fn bank(&self) -> f64 {
        (-self.right.dot(self.pos.normalize())).asin().to_degrees()
    }
    /// how far the nose points off the way it moves over the ground, degrees
    fn crab(&self) -> f64 {
        let up = self.pos.normalize();
        let v = self.vel - up * self.vel.dot(up);
        if v.length() < 3.0 {
            return 0.0;
        }
        let f = (self.forward - up * self.forward.dot(up)).normalize_or_zero();
        let v = v.normalize();
        f.cross(v).dot(up).atan2(f.dot(v)).to_degrees()
    }
}

/// How a touchdown felt, softest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    Butter,
    Smooth,
    Firm,
    Hard,
    VeryHard,
}

impl Grade {
    /// From the sink rate (fpm) and the bounces: each bounce costs a step.
    pub fn of(sink_fpm: f64, bounces: u32) -> Grade {
        let base = match sink_fpm {
            s if s <= 60.0 => 0,
            s if s <= 180.0 => 1,
            s if s <= 360.0 => 2,
            s if s <= 600.0 => 3,
            _ => 4,
        };
        [Grade::Butter, Grade::Smooth, Grade::Firm, Grade::Hard, Grade::VeryHard][(base + bounces as usize).min(4)]
    }

    pub fn text(self) -> &'static str {
        match self {
            Grade::Butter => crate::t!("log.grade_butter"),
            Grade::Smooth => crate::t!("log.grade_smooth"),
            Grade::Firm => crate::t!("log.grade_firm"),
            Grade::Hard => crate::t!("log.grade_hard"),
            Grade::VeryHard => crate::t!("log.grade_very_hard"),
        }
    }
}

/// Where on a runway the aircraft touched down.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnRunway {
    /// "25R"
    pub runway: String,
    /// metres right (+) or left (-) of the centreline
    pub off_centre: f64,
    /// metres past the threshold
    pub past_threshold: f64,
    pub length: f64,
    pub width: f64,
}

/// One touchdown, graded.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Landing {
    /// the airport's identifier, if it was at one
    pub airport: Option<String>,
    pub on_runway: Option<OnRunway>,
    pub sink_fpm: f64,
    /// the highest load in the second after
    pub g: f64,
    pub speed_kt: f64,
    pub bank: f64,
    pub crab: f64,
    pub bounces: u32,
    pub grade: Grade,
    /// out of 100, for comparing landings
    pub score: u32,
}

impl Landing {
    /// A score out of 100: the sink rate counts most, then the bounces, the
    /// centreline, the bank and the crab, and touching down before the runway.
    pub fn rate(&self) -> u32 {
        let mut s = 100.0;
        s -= ((self.sink_fpm - 60.0).max(0.0) / 540.0 * 50.0).min(60.0);
        s -= self.bounces as f64 * 12.0;
        s -= ((self.bank.abs() - 2.0).max(0.0) * 2.0).min(10.0);
        s -= ((self.crab.abs() - 3.0).max(0.0) * 1.5).min(10.0);
        match &self.on_runway {
            Some(r) => {
                s -= ((r.off_centre.abs() / (r.width * 0.5).max(5.0)) * 15.0).min(15.0);
                if r.past_threshold < 0.0 {
                    s -= 20.0;
                }
            }
            None => s -= 25.0,
        }
        s.clamp(0.0, 100.0).round() as u32
    }

    /// "−142 fpm · 1.3 g · 4 m right · Smooth (88)"
    pub fn summary(&self) -> String {
        let mut parts = vec![format!("{:.0} fpm", -self.sink_fpm), format!("{:.1} g", self.g)];
        if let Some(r) = &self.on_runway {
            let side = if r.off_centre >= 0.0 { crate::t!("log.right_of_centre") } else { crate::t!("log.left_of_centre") };
            parts.push(format!("{:.0} m {side}", r.off_centre.abs()));
        }
        if self.bounces > 0 {
            parts.push(crate::t!("log.bounces", n = self.bounces));
        }
        parts.push(format!("{} ({})", self.grade.text(), self.score));
        parts.join(" · ").replace('.', crate::i18n::lang().text("number.decimal_point"))
    }
}

/// How a flight ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Landed,
    Crashed,
    /// left in the air, or on the ground away from where it took off and not yet parked
    Unfinished,
}

/// One flight in the logbook.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flight {
    /// when it began (Unix seconds, UTC)
    pub date: i64,
    pub aircraft: String,
    /// the airport it took off from, if any
    pub from: Option<String>,
    /// where it ended, if at an airport
    pub to: Option<String>,
    /// seconds in the air
    pub air_time: f64,
    /// metres flown over the ground
    pub distance: f64,
    /// highest altitude, metres above sea level
    pub max_alt: f64,
    pub landings: Vec<Landing>,
    pub outcome: Outcome,
}

impl Flight {
    pub fn best_landing(&self) -> Option<&Landing> {
        self.landings.iter().max_by_key(|l| l.score)
    }
}

/// What a step brought.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    TakeOff { airport: Option<String> },
    /// a touchdown, graded once the aircraft has settled (or flown off again)
    Landed(Landing),
}

/// A touchdown being watched for bounces before it is graded.
#[derive(Clone, Debug)]
struct Touch {
    first: Sample,
    sink: f64,
    g: f64,
    bounces: u32,
    /// seconds on the ground since the last contact, and in the air since the last bounce
    ground_t: f64,
    air_t: f64,
    since: f64,
}

/// Seconds off the ground before it counts as flying.
const LIFT_OFF: f64 = 2.0;
/// Seconds on the ground before a touchdown is graded.
const SETTLE: f64 = 2.5;
/// Seconds in the air after a touchdown that make it a touch-and-go rather than a bounce.
const BOUNCE_MAX: f64 = 4.0;
/// Parked: this slow (m/s) for this long after landing ends the flight.
const PARKED_SPEED: f64 = 1.0;
const PARKED_TIME: f64 = 20.0;

/// Follows one flight.
pub struct Tracker {
    aircraft: String,
    date: i64,
    from: Option<String>,
    flying: bool,
    took_off: bool,
    /// seconds off the ground so far (counting up to LIFT_OFF before `flying`)
    off_t: f64,
    air_time: f64,
    distance: f64,
    max_alt: f64,
    last: Option<Sample>,
    /// the sink rate the step before the wheels touched
    sink: f64,
    touch: Option<Touch>,
    landings: Vec<Landing>,
    parked_t: f64,
}

pub fn now_unix() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

impl Tracker {
    /// A new flight in `aircraft`, starting now.
    pub fn new(aircraft: &str) -> Self {
        Tracker { aircraft: aircraft.into(), date: now_unix(), from: None, flying: false, took_off: false, off_t: 0.0, air_time: 0.0, distance: 0.0, max_alt: 0.0, last: None, sink: 0.0, touch: None, landings: Vec::new(), parked_t: 0.0 }
    }

    /// One physics step of `dt` seconds.
    pub fn step(&mut self, s: Sample, dt: f64) -> Vec<Event> {
        let mut out = Vec::new();
        let first = self.last.is_none();
        if let Some(l) = self.last {
            self.distance += (s.pos - l.pos).length().min(s.vel.length() * dt * 2.0 + 1.0);
        }
        self.max_alt = self.max_alt.max(s.pos.length() - geo::R);
        // Spawned in the air: flying from the start, from nowhere.
        if first && !s.on_ground {
            self.flying = true;
            self.off_t = LIFT_OFF;
        }
        if s.on_ground {
            self.off_t = 0.0;
        } else {
            self.off_t += dt;
            self.sink = -s.vs();
        }
        if self.flying && !s.on_ground {
            self.air_time += dt;
        }
        if !self.flying && !s.on_ground && self.off_t >= LIFT_OFF {
            self.flying = true;
            self.took_off = true;
            let (la, lo) = s.lat_lon();
            let airport = airport_at(la, lo);
            if self.from.is_none() && self.landings.is_empty() {
                self.from = airport.clone();
            }
            out.push(Event::TakeOff { airport });
        }
        // a touchdown, or a bounce of one being watched.
        let was_ground = self.last.is_some_and(|l| l.on_ground);
        if s.on_ground && !was_ground && !first {
            match &mut self.touch {
                Some(t) => {
                    t.bounces += 1;
                    t.ground_t = 0.0;
                    t.g = t.g.max(s.g);
                }
                None if self.flying => {
                    self.touch = Some(Touch { first: s, sink: self.sink.max(0.0), g: s.g, bounces: 0, ground_t: 0.0, air_t: 0.0, since: 0.0 });
                }
                None => {}
            }
        }
        if let Some(t) = &mut self.touch {
            t.since += dt;
            if t.since < 1.0 {
                t.g = t.g.max(s.g);
            }
            if s.on_ground {
                t.ground_t += dt;
                t.air_t = 0.0;
            } else {
                t.air_t += dt;
            }
            if t.ground_t >= SETTLE || t.air_t >= BOUNCE_MAX {
                let t = self.touch.take().unwrap();
                let bounces = t.bounces;
                let l = grade(&t.first, t.sink, t.g, bounces);
                self.landings.push(l.clone());
                if t.air_t < BOUNCE_MAX {
                    self.flying = false;
                }
                out.push(Event::Landed(l));
            }
        }
        if !self.flying && s.on_ground && !self.landings.is_empty() && s.ground_speed() < PARKED_SPEED {
            self.parked_t += dt;
        } else {
            self.parked_t = 0.0;
        }
        self.last = Some(s);
        out
    }

    /// Stopped after landing: the flight is over.
    pub fn parked(&self) -> bool {
        self.parked_t >= PARKED_TIME
    }

    /// The flight as it stands, ended now by `outcome`; None when there was
    /// no flight to speak of (never left the ground).
    pub fn finish(mut self, crashed: bool) -> Option<Flight> {
        // A touchdown still settling counts, unless it ended in the crash.
        if let Some(t) = self.touch.take().filter(|_| !crashed) {
            self.landings.push(grade(&t.first, t.sink, t.g, t.bounces));
            self.flying = false;
        }
        if !self.took_off && self.landings.is_empty() && !crashed && self.air_time < 30.0 {
            return None;
        }
        let outcome = if crashed {
            Outcome::Crashed
        } else if !self.flying && !self.landings.is_empty() && self.last.is_some_and(|l| l.on_ground) {
            Outcome::Landed
        } else {
            Outcome::Unfinished
        };
        let to = match outcome {
            Outcome::Landed => self.landings.last().and_then(|l| l.airport.clone()),
            _ => self.last.and_then(|l| {
                let (la, lo) = l.lat_lon();
                if l.on_ground || crashed { airport_at(la, lo) } else { None }
            }),
        };
        Some(Flight { date: self.date, aircraft: self.aircraft, from: self.from, to, air_time: self.air_time, distance: self.distance, max_alt: self.max_alt, landings: self.landings, outcome })
    }
}

/// The airport whose ground this is, if any (degrees).
fn airport_at(lat: f64, lon: f64) -> Option<String> {
    let db = airports::Db::get();
    db.near(lat, lon, 4000.0).into_iter().find(|(i, d)| *d <= db.apts[*i as usize].reach().max(1500.0)).map(|(i, _)| db.apts[i as usize].ident.clone())
}

fn grade(s: &Sample, sink_ms: f64, g: f64, bounces: u32) -> Landing {
    let (la, lo) = s.lat_lon();
    let (airport, on_runway) = runway_at(la, lo, s.heading());
    let sink_fpm = sink_ms * FPM;
    let mut l = Landing { airport, on_runway, sink_fpm, g, speed_kt: s.ground_speed() * KT, bank: s.bank(), crab: s.crab(), bounces, grade: Grade::of(sink_fpm, bounces), score: 0 };
    l.score = l.rate();
    l
}

/// The runway (and its airport) a point is on, landing on `heading`.
fn runway_at(lat: f64, lon: f64, heading: f64) -> (Option<String>, Option<OnRunway>) {
    let db = airports::Db::get();
    let p = geo::lla_to_pos(lat.to_radians(), lon.to_radians(), 0.0);
    let (e, n, _) = geo::enu(p);
    let local = |q: (f64, f64)| {
        let d = geo::lla_to_pos(q.0.to_radians(), q.1.to_radians(), 0.0) - p;
        (d.dot(e), d.dot(n))
    };
    let mut best: Option<(f64, String, OnRunway)> = None;
    let near = db.near(lat, lon, 6000.0);
    for &(i, _) in &near {
        let a = &db.apts[i as usize];
        for r in &a.rwys {
            let [Some(p0), Some(p1)] = r.pos else { continue };
            let (a0, a1) = (local(p0), local(p1));
            let (dx, dy) = (a1.0 - a0.0, a1.1 - a0.1);
            let len = dx.hypot(dy);
            if len < 50.0 {
                continue;
            }
            // which way it is landing: from end 0 to end 1, or back.
            let brg = dx.atan2(dy).to_degrees();
            let forward = ((heading - brg + 540.0).rem_euclid(360.0) - 180.0).abs() < 90.0;
            let (thr, dir, ident) = if forward { (a0, (dx / len, dy / len), &r.ident[0]) } else { (a1, (-dx / len, -dy / len), &r.ident[1]) };
            let rel = (-thr.0, -thr.1);
            let along = rel.0 * dir.0 + rel.1 * dir.1;
            let right = rel.0 * dir.1 - rel.1 * dir.0;
            let w = r.width as f64;
            if along < -300.0 || along > len + 100.0 || right.abs() > w * 0.5 + 40.0 {
                continue;
            }
            let miss = right.abs() + (-along).max(0.0) + (along - len).max(0.0);
            if best.as_ref().is_none_or(|b| miss < b.0) {
                best = Some((miss, a.ident.clone(), OnRunway { runway: ident.clone(), off_centre: right, past_threshold: along, length: len, width: w }));
            }
        }
    }
    match best {
        Some((_, apt, r)) => (Some(apt), Some(r)),
        None => (airport_at(lat, lon), None),
    }
}

/// The logbook: every flight, newest last.
#[derive(Default, Serialize, Deserialize)]
pub struct Logbook {
    pub flights: Vec<Flight>,
}

const MAX_FLIGHTS: usize = 5000;

fn path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("fablegame").join("logbook.json"))
}

impl Logbook {
    pub fn load() -> Self {
        path().and_then(|p| std::fs::read(p).ok()).and_then(|d| serde_json::from_slice(&d).ok()).unwrap_or_default()
    }

    pub fn add(&mut self, f: Flight) {
        self.flights.push(f);
        if self.flights.len() > MAX_FLIGHTS {
            let extra = self.flights.len() - MAX_FLIGHTS;
            self.flights.drain(..extra);
        }
    }

    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = p.with_extension("json.tmp");
        if let Ok(data) = serde_json::to_vec(self) {
            if std::fs::write(&tmp, data).is_ok() {
                let _ = std::fs::rename(&tmp, &p);
            }
        }
    }

    /// Hours in the air, landings, and the best landing's score.
    pub fn totals(&self) -> Totals {
        let mut t = Totals::default();
        for f in &self.flights {
            t.flights += 1;
            t.hours += f.air_time / 3600.0;
            t.landings += f.landings.len();
            if f.outcome == Outcome::Crashed {
                t.crashes += 1;
            }
            if let Some(b) = f.best_landing() {
                t.best = t.best.max(Some(b.score));
            }
        }
        t
    }

    /// Hours in the air by aircraft, most first.
    pub fn hours_by_aircraft(&self) -> Vec<(String, f64)> {
        let mut m: Vec<(String, f64)> = Vec::new();
        for f in &self.flights {
            match m.iter_mut().find(|(a, _)| *a == f.aircraft) {
                Some(e) => e.1 += f.air_time / 3600.0,
                None => m.push((f.aircraft.clone(), f.air_time / 3600.0)),
            }
        }
        m.sort_by(|a, b| b.1.total_cmp(&a.1));
        m
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct Totals {
    pub flights: usize,
    pub hours: f64,
    pub landings: usize,
    pub crashes: usize,
    pub best: Option<u32>,
}

/// "2026-11-23 21:40Z" (UTC, as in aviation) from Unix seconds.
pub fn date_text(unix: i64) -> String {
    let days = unix.div_euclid(86400);
    let secs = unix.rem_euclid(86400);
    // days to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}Z", secs / 3600, secs / 60 % 60)
}

/// "1:25" from seconds.
pub fn hours_text(secs: f64) -> String {
    let m = (secs / 60.0).round() as i64;
    format!("{}:{:02}", m / 60, m % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An aircraft over `lat, lon` (degrees), `agl` metres up, flying `heading`
    /// at `speed` m/s and sinking at `sink` m/s.
    fn at(lat: f64, lon: f64, agl: f64, heading: f64, speed: f64, sink: f64, ground: bool) -> Sample {
        let pos = geo::lla_to_pos(lat.to_radians(), lon.to_radians(), agl);
        let (e, n, u) = geo::enu(pos);
        let h = heading.to_radians();
        let fwd = e * h.sin() + n * h.cos();
        Sample { pos, vel: fwd * speed - u * sink, forward: fwd, right: fwd.cross(u), on_ground: ground, g: if ground { 1.2 } else { 1.0 }, }
    }

    /// EBBR 25R's threshold and its far end, from the database.
    fn ebbr_25r() -> ((f64, f64), (f64, f64), f64) {
        let db = airports::Db::get();
        let a = &db.apts[db.find("EBBR").unwrap() as usize];
        let r = a.rwys.iter().find(|r| r.ident.contains(&"25R".to_string())).unwrap();
        let e = r.ident.iter().position(|i| i == "25R").unwrap();
        let (thr, far) = (r.pos[e].unwrap(), r.pos[1 - e].unwrap());
        let brg = nav_bearing(thr, far);
        (thr, far, brg)
    }

    fn nav_bearing(a: (f64, f64), b: (f64, f64)) -> f64 {
        crate::nav::bearing(geo::lla_to_pos(a.0.to_radians(), a.1.to_radians(), 0.0), geo::lla_to_pos(b.0.to_radians(), b.1.to_radians(), 0.0))
    }

    fn along(a: (f64, f64), b: (f64, f64), f: f64) -> (f64, f64) {
        (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f)
    }

    #[test]
    fn grades_follow_the_sink_rate_and_bounces() {
        assert_eq!(Grade::of(40.0, 0), Grade::Butter);
        assert_eq!(Grade::of(150.0, 0), Grade::Smooth);
        assert_eq!(Grade::of(150.0, 1), Grade::Firm);
        assert_eq!(Grade::of(500.0, 0), Grade::Hard);
        assert_eq!(Grade::of(900.0, 3), Grade::VeryHard);
    }

    #[test]
    fn a_touchdown_on_ebbr_25r_is_found_and_graded() {
        let (thr, far, brg) = ebbr_25r();
        let p = along(thr, far, 0.12);
        let (airport, on) = runway_at(p.0, p.1, brg);
        assert_eq!(airport.as_deref(), Some("EBBR"));
        let on = on.unwrap();
        assert_eq!(on.runway, "25R");
        assert!(on.off_centre.abs() < 5.0, "{}", on.off_centre);
        assert!((on.past_threshold - on.length * 0.12).abs() < 20.0, "{} of {}", on.past_threshold, on.length);
        // the other way it is 07L, far down the runway.
        let (_, back) = runway_at(p.0, p.1, brg + 180.0);
        assert_eq!(back.unwrap().runway, "07L");
    }

    #[test]
    fn a_flight_takes_off_lands_and_parks() {
        let (thr, far, brg) = ebbr_25r();
        let mut t = Tracker::new("Cessna 172RG");
        let dt = 1.0 / 60.0;
        let mut events = Vec::new();
        // on the runway, rolling
        for _ in 0..60 {
            events.extend(t.step(at(far.0, far.1, 0.0, brg + 180.0, 20.0, 0.0, true), dt));
        }
        // airborne for a minute
        for k in 0..3600 {
            events.extend(t.step(at(far.0, far.1, 10.0 + k as f64 * 0.1, brg + 180.0, 50.0, -1.0, false), dt));
        }
        assert!(matches!(events.first(), Some(Event::TakeOff { airport: Some(a) }) if a == "EBBR"), "{events:?}");
        // a final approach to 25R, touching down sinking 2 m/s (394 fpm), then a bounce
        let p = along(thr, far, 0.1);
        for _ in 0..30 {
            t.step(at(p.0, p.1, 2.0, brg, 33.0, 2.0, false), dt);
        }
        for _ in 0..20 {
            t.step(at(p.0, p.1, 0.0, brg, 32.0, 0.0, true), dt);
        }
        for _ in 0..40 {
            t.step(at(p.0, p.1, 0.5, brg, 31.0, 0.2, false), dt);
        }
        let mut landed = Vec::new();
        for _ in 0..300 {
            landed.extend(t.step(at(p.0, p.1, 0.0, brg, 20.0, 0.0, true), dt));
        }
        let [Event::Landed(l)] = &landed[..] else { panic!("{landed:?}") };
        assert_eq!(l.airport.as_deref(), Some("EBBR"));
        assert_eq!(l.on_runway.as_ref().unwrap().runway, "25R");
        assert_eq!(l.bounces, 1);
        assert!((l.sink_fpm - 394.0).abs() < 5.0, "{}", l.sink_fpm);
        assert_eq!(l.grade, Grade::VeryHard, "hard, and a bounce");
        // stopped for 20 s: parked
        for _ in 0..(21 * 60) {
            t.step(at(p.0, p.1, 0.0, brg, 0.0, 0.0, true), dt);
        }
        assert!(t.parked());
        let f = t.finish(false).unwrap();
        assert_eq!((f.from.as_deref(), f.to.as_deref(), f.outcome), (Some("EBBR"), Some("EBBR"), Outcome::Landed));
        assert!((f.air_time - 60.0).abs() < 2.0, "{}", f.air_time);
        assert_eq!(f.landings.len(), 1);
    }

    #[test]
    fn taxiing_is_no_flight_and_a_spawn_in_the_air_is_one() {
        let (_, far, brg) = ebbr_25r();
        let mut t = Tracker::new("x");
        for _ in 0..600 {
            t.step(at(far.0, far.1, 0.0, brg, 8.0, 0.0, true), 1.0 / 60.0);
        }
        assert!(t.finish(false).is_none());
        let mut t = Tracker::new("x");
        for _ in 0..(40 * 60) {
            t.step(at(50.0, 4.0, 500.0, 90.0, 50.0, 0.0, false), 1.0 / 60.0);
        }
        let f = t.finish(false).unwrap();
        assert_eq!((f.from, f.outcome), (None, Outcome::Unfinished));
        // into the ground: a crash, not a landing.
        let mut t = Tracker::new("x");
        t.step(at(50.0, 4.0, 500.0, 90.0, 50.0, 0.0, false), 1.0 / 60.0);
        t.step(at(50.0, 4.0, 0.0, 90.0, 50.0, 30.0, true), 1.0 / 60.0);
        let f = t.finish(true).unwrap();
        assert_eq!((f.outcome, f.landings.len()), (Outcome::Crashed, 0));
    }

    #[test]
    fn a_soft_landing_on_the_centreline_scores_high() {
        let mut l = Landing { airport: None, on_runway: Some(OnRunway { runway: "25R".into(), off_centre: 1.0, past_threshold: 300.0, length: 3200.0, width: 45.0 }), sink_fpm: 90.0, g: 1.1, speed_kt: 60.0, bank: 0.5, crab: 1.0, bounces: 0, grade: Grade::Smooth, score: 0 };
        assert!(l.rate() >= 95, "{}", l.rate());
        l.sink_fpm = 700.0;
        l.bounces = 2;
        assert!(l.rate() < 40, "{}", l.rate());
    }

    #[test]
    fn dates_and_hours_read_right() {
        assert_eq!(date_text(0), "1970-01-01 00:00Z");
        assert_eq!(date_text(1_795_470_000), "2026-11-23 21:40Z");
        assert_eq!(hours_text(5100.0), "1:25");
    }

    #[test]
    fn the_logbook_round_trips_and_totals() {
        let mut b = Logbook::default();
        b.add(Flight { date: 1, aircraft: "A".into(), from: Some("EBAW".into()), to: Some("EBBR".into()), air_time: 1800.0, distance: 40_000.0, max_alt: 900.0, landings: vec![], outcome: Outcome::Landed });
        b.add(Flight { date: 2, aircraft: "B".into(), from: None, to: None, air_time: 3600.0, distance: 0.0, max_alt: 0.0, landings: vec![], outcome: Outcome::Crashed });
        let back: Logbook = serde_json::from_slice(&serde_json::to_vec(&b).unwrap()).unwrap();
        assert_eq!(back.flights, b.flights);
        let t = b.totals();
        assert_eq!((t.flights, t.crashes), (2, 1));
        assert!((t.hours - 1.5).abs() < 1e-9);
        assert_eq!(b.hours_by_aircraft()[0].0, "B");
    }
}
