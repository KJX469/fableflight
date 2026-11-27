//! the pilot profile: the name and callsign, experience and level, the
//! achievements, the airports and types flown, and the best scores. the
//! flights themselves are in the flight log (flightlog.rs); flights kept here
//! by older versions move there once. saved next to the settings as profile.json.

use crate::flightlog::{self, Flight, Outcome};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

/// one flight as older versions kept it, read only to move it to the flight log.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    /// seconds since 1970, utc, when it started.
    pub unix: u64,
    pub aircraft: String,
    /// icao code, or a place name.
    pub from: String,
    pub to: String,
    /// seconds in the air.
    pub air_s: f64,
    /// distance flown, nm.
    pub dist_nm: f64,
    pub landings: u32,
    pub max_alt_ft: f64,
    #[serde(default)]
    pub crashed: bool,
    #[serde(default)]
    pub open: bool,
}

/// experience given for something, newest last.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Award {
    pub unix: u64,
    pub what: String,
    pub xp: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    /// the callsign atc knows the pilot by (empty: one made up for the aircraft).
    #[serde(default)]
    pub callsign: String,
    /// the best scores of the tutorials, exams and missions (missions.rs), by id.
    #[serde(default)]
    pub best: HashMap<String, u32>,
    /// unix seconds the profile began.
    pub since: u64,
    /// flights from before the flight log, emptied once moved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub log: Vec<Entry>,
    /// experience points, all told.
    #[serde(default)]
    pub xp: u64,
    /// the latest awards, for the profile page.
    #[serde(default)]
    pub awards: Vec<Award>,
    /// achievements (pilot.rs) by id, with when they came.
    #[serde(default)]
    pub badges: BTreeMap<String, u64>,
    /// airports landed at, and the types flown.
    #[serde(default)]
    pub airports: BTreeSet<String>,
    #[serde(default)]
    pub types: BTreeSet<String>,
    /// butter landings, night landings, the fastest mach and highest feet seen.
    #[serde(default)]
    pub butters: u32,
    #[serde(default)]
    pub night_landings: u32,
    #[serde(default)]
    pub max_mach: f64,
    #[serde(default)]
    pub max_ft: f64,
}

/// sums over the flight log.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Totals {
    pub air_s: f64,
    pub flights: usize,
    pub landings: u32,
    pub dist_nm: f64,
    pub longest_s: f64,
    pub highest_ft: f64,
    pub crashes: usize,
    /// the best landing's score.
    pub best: Option<u32>,
    /// aircraft by hours, most first: name, seconds, flights.
    pub by_type: Vec<(String, f64, usize)>,
    /// airports by visits, most first.
    pub airports: Vec<(String, usize)>,
}

pub fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// yyyy-mm-dd hh:mm utc from unix seconds.
pub fn date(unix: u64) -> String {
    let days = (unix / 86_400) as i64;
    let secs = unix % 86_400;
    // civil from days, howard hinnant's algorithm.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}", secs / 3600, (secs % 3600) / 60)
}

/// hours and minutes, like 12:05.
pub fn hhmm(secs: f64) -> String {
    let m = (secs.max(0.0) / 60.0).round() as u64;
    format!("{}:{:02}", m / 60, m % 60)
}

/// decimal hours, like 12.1.
pub fn hours(secs: f64) -> String {
    format!("{:.1}", secs.max(0.0) / 3600.0)
}

/// an icao code rather than a place name.
pub fn is_icao(s: &str) -> bool {
    (3..=4).contains(&s.len()) && s.chars().all(|c| c.is_ascii_alphanumeric()) && s.chars().any(|c| c.is_ascii_alphabetic())
}

fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("fablegame").join("profile.json"))
}

impl Profile {
    pub fn load() -> Profile {
        let mut p: Profile = path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        if p.since == 0 {
            p.since = now_unix();
        }
        p
    }

    pub fn save(&self) {
        let Some(path) = path() else { return };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let Ok(text) = serde_json::to_string_pretty(self) else { return };
        // written whole then moved, so a crash mid write keeps the old one.
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }

    /// the flights older versions kept, as the flight log keeps them; emptied here.
    pub fn take_legacy(&mut self) -> Vec<Flight> {
        std::mem::take(&mut self.log).into_iter().map(|e| Flight {
            date: e.unix as i64,
            aircraft: e.aircraft,
            from: is_icao(&e.from).then_some(e.from),
            to: is_icao(&e.to).then_some(e.to),
            air_time: e.air_s,
            distance: e.dist_nm * 1852.0,
            max_alt: e.max_alt_ft / 3.28084,
            landings: Vec::new(),
            outcome: if e.crashed { Outcome::Crashed } else if e.landings > 0 { Outcome::Landed } else { Outcome::Unfinished },
        }).collect()
    }

    /// the experience counted in, and the award kept for the profile page.
    pub fn add_xp(&mut self, what: &str, xp: u64) {
        if xp == 0 {
            return;
        }
        self.xp += xp;
        self.awards.push(Award { unix: now_unix(), what: what.into(), xp });
        if self.awards.len() > 80 {
            self.awards.remove(0);
        }
    }

    pub fn has_badge(&self, id: &str) -> bool {
        self.badges.contains_key(id)
    }
}

/// what older flights did not keep (landings in detail) counts as one landing.
pub fn totals(flights: &[Flight]) -> Totals {
    let mut t = Totals::default();
    let mut types: Vec<(String, f64, usize)> = Vec::new();
    let mut apts: Vec<(String, usize)> = Vec::new();
    for f in flights {
        t.air_s += f.air_time;
        t.flights += 1;
        t.landings += f.landings.len().max(usize::from(f.outcome == Outcome::Landed)) as u32;
        t.dist_nm += f.distance / 1852.0;
        t.longest_s = t.longest_s.max(f.air_time);
        t.highest_ft = t.highest_ft.max(f.max_alt * 3.28084);
        if f.outcome == Outcome::Crashed {
            t.crashes += 1;
        }
        if let Some(b) = f.best_landing() {
            t.best = t.best.max(Some(b.score));
        }
        match types.iter_mut().find(|x| x.0 == f.aircraft) {
            Some(x) => {
                x.1 += f.air_time;
                x.2 += 1;
            }
            None => types.push((f.aircraft.clone(), f.air_time, 1)),
        }
        for a in [&f.from, &f.to].into_iter().flatten() {
            match apts.iter_mut().find(|x| &x.0 == a) {
                Some(x) => x.1 += 1,
                None => apts.push((a.clone(), 1)),
            }
        }
    }
    types.sort_by(|a, b| b.1.total_cmp(&a.1));
    apts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    t.by_type = types;
    t.airports = apts;
    t
}

/// a flight's date as the logbook writes it.
pub fn flight_date(f: &Flight) -> String {
    flightlog::date_text(f.date)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(date(0), "1970-01-01 00:00");
        assert_eq!(date(1_795_000_000), "2026-11-18 11:06");
        assert_eq!(hhmm(3600.0 * 2.0 + 5.0 * 60.0), "2:05");
        assert_eq!(hours(5400.0), "1.5");
    }

    #[test]
    fn old_flights_move_to_the_flight_log() {
        let mut p = Profile::default();
        p.log.push(Entry { unix: 100, aircraft: "Cessna 172".into(), from: "KSFO".into(), to: "37.53,-122.58".into(), air_s: 600.0, dist_nm: 20.0, landings: 1, max_alt_ft: 3500.0, crashed: false, open: false });
        p.log.push(Entry { unix: 200, aircraft: "Boeing 747".into(), from: "KSFO".into(), to: "Crashed".into(), air_s: 60.0, dist_nm: 2.0, landings: 0, max_alt_ft: 900.0, crashed: true, open: false });
        let f = p.take_legacy();
        assert!(p.log.is_empty());
        assert_eq!(f.len(), 2);
        assert_eq!((f[0].from.as_deref(), f[0].to.as_deref(), f[0].outcome), (Some("KSFO"), None, Outcome::Landed));
        assert_eq!(f[1].outcome, Outcome::Crashed);
        let t = totals(&f);
        assert_eq!((t.flights, t.landings, t.crashes), (2, 1, 1));
        assert!((t.dist_nm - 22.0).abs() < 1e-6);
        assert_eq!(t.by_type[0].0, "Cessna 172");
        assert_eq!(t.airports, vec![("KSFO".to_string(), 2)]);
    }

    #[test]
    fn awards_add_up() {
        let mut p = Profile::default();
        p.add_xp("Take-off", 25);
        p.add_xp("nothing", 0);
        p.add_xp("Butter landing", 150);
        assert_eq!(p.xp, 175);
        assert_eq!(p.awards.len(), 2);
    }
}
