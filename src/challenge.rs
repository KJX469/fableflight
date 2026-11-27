//! Landing challenges: a set approach (airport, runway, aircraft, time of
//! day, wind and visibility) to land as well as possible. The touchdown is
//! graded by the flight log (flightlog.rs), plus how near the aiming point it
//! came down; the best scores are kept in `challenges.json` next to
//! settings.json.

use crate::flightlog::{Grade, Landing};
use crate::{airports, geo};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct Challenge {
    /// its name in the strings ("challenge.<id>.name") and in challenges.json
    pub id: &'static str,
    pub airport: &'static str,
    pub runway: &'static str,
    /// an aircraft key (aircraft::by_key)
    pub aircraft: &'static str,
    /// local hour
    pub hour: f64,
    /// knots, and from how far right of the runway's heading (degrees; negative from the left)
    pub wind_kt: f64,
    pub wind_rel: f64,
    /// 0..1
    pub gusts: f64,
    pub clouds: f32,
    /// a multiple of a clear day's haze
    pub haze: f32,
    pub rain: f32,
}

/// From the gentlest to the hardest.
pub const ALL: [Challenge; 17] = [
    Challenge { id: "first", airport: "EBAW", runway: "29", aircraft: "c172", hour: 14.0, wind_kt: 4.0, wind_rel: 10.0, gusts: 0.0, clouds: 0.2, haze: 1.0, rain: 0.0 },
    Challenge { id: "crosswind", airport: "EBBR", runway: "25R", aircraft: "c172", hour: 11.0, wind_kt: 14.0, wind_rel: 80.0, gusts: 0.3, clouds: 0.4, haze: 1.0, rain: 0.0 },
    Challenge { id: "night", airport: "EBAW", runway: "29", aircraft: "dr400", hour: 22.5, wind_kt: 6.0, wind_rel: -20.0, gusts: 0.1, clouds: 0.3, haze: 1.2, rain: 0.0 },
    Challenge { id: "telluride", airport: "KTEX", runway: "09", aircraft: "c172", hour: 10.0, wind_kt: 8.0, wind_rel: 30.0, gusts: 0.2, clouds: 0.1, haze: 1.0, rain: 0.0 },
    Challenge { id: "stbarth", airport: "TFFJ", runway: "10", aircraft: "c172", hour: 15.0, wind_kt: 12.0, wind_rel: -15.0, gusts: 0.3, clouds: 0.3, haze: 1.0, rain: 0.0 },
    Challenge { id: "mountain", airport: "LOWI", runway: "26", aircraft: "c172", hour: 12.0, wind_kt: 10.0, wind_rel: 15.0, gusts: 0.4, clouds: 0.3, haze: 1.0, rain: 0.0 },
    Challenge { id: "courchevel", airport: "LFLJ", runway: "22", aircraft: "dr400", hour: 11.0, wind_kt: 4.0, wind_rel: 0.0, gusts: 0.0, clouds: 0.1, haze: 1.0, rain: 0.0 },
    Challenge { id: "rain", airport: "EHAM", runway: "27", aircraft: "g650", hour: 17.0, wind_kt: 16.0, wind_rel: -35.0, gusts: 0.5, clouds: 0.9, haze: 4.0, rain: 0.7 },
    Challenge { id: "london_city", airport: "EGLC", runway: "27", aircraft: "g650", hour: 19.0, wind_kt: 10.0, wind_rel: -20.0, gusts: 0.2, clouds: 0.9, haze: 2.0, rain: 0.3 },
    Challenge { id: "short", airport: "TNCS", runway: "12", aircraft: "dr400", hour: 10.0, wind_kt: 12.0, wind_rel: -30.0, gusts: 0.3, clouds: 0.3, haze: 1.0, rain: 0.0 },
    Challenge { id: "lukla", airport: "VNLK", runway: "06", aircraft: "dr400", hour: 8.0, wind_kt: 6.0, wind_rel: 20.0, gusts: 0.2, clouds: 0.3, haze: 1.0, rain: 0.0 },
    Challenge { id: "jumbo", airport: "KSFO", runway: "28L", aircraft: "747", hour: 16.0, wind_kt: 14.0, wind_rel: 10.0, gusts: 0.2, clouds: 0.2, haze: 1.5, rain: 0.0 },
    Challenge { id: "maho", airport: "TNCM", runway: "10", aircraft: "747", hour: 13.0, wind_kt: 12.0, wind_rel: -10.0, gusts: 0.2, clouds: 0.3, haze: 1.0, rain: 0.0 },
    Challenge { id: "gibraltar", airport: "LXGB", runway: "27", aircraft: "g650", hour: 12.0, wind_kt: 20.0, wind_rel: 25.0, gusts: 0.6, clouds: 0.5, haze: 1.0, rain: 0.0 },
    Challenge { id: "madeira", airport: "LPMA", runway: "05", aircraft: "g650", hour: 17.0, wind_kt: 18.0, wind_rel: 40.0, gusts: 0.6, clouds: 0.5, haze: 1.0, rain: 0.0 },
    Challenge { id: "queenstown", airport: "NZQN", runway: "23", aircraft: "g650", hour: 15.0, wind_kt: 10.0, wind_rel: 15.0, gusts: 0.4, clouds: 0.3, haze: 1.0, rain: 0.0 },
    Challenge { id: "paro", airport: "VQPR", runway: "15", aircraft: "g650", hour: 9.0, wind_kt: 12.0, wind_rel: 40.0, gusts: 0.5, clouds: 0.4, haze: 1.0, rain: 0.0 },
];

impl Challenge {
    pub fn name(&self) -> &'static str {
        crate::i18n::tr(&format!("challenge.{}.name", self.id))
    }

    pub fn about(&self) -> &'static str {
        crate::i18n::tr(&format!("challenge.{}.about", self.id))
    }

    /// The airport's index and the runway's heading (degrees true), from the database.
    pub fn runway(&self) -> Option<(u32, f64)> {
        let db = airports::Db::get();
        let i = db.find(self.airport)?;
        let r = db.apts[i as usize].rwys.iter().find(|r| r.ident.iter().any(|x| x == self.runway))?;
        let e = r.ident.iter().position(|x| x == self.runway)?;
        let heading = match (r.pos[e], r.pos[1 - e]) {
            (Some(a), Some(b)) => crate::nav::bearing(geo::lla_to_pos(a.0.to_radians(), a.1.to_radians(), 0.0), geo::lla_to_pos(b.0.to_radians(), b.1.to_radians(), 0.0)),
            _ => airports::ident_heading(self.runway)? as f64,
        };
        Some((i, heading))
    }

    /// Where the wind blows from, degrees true.
    pub fn wind_from(&self) -> f64 {
        let h = self.runway().map_or(0.0, |r| r.1);
        (h + self.wind_rel).rem_euclid(360.0)
    }

    /// "14 kt from the right, gusty · overcast, rain · 22:30"
    pub fn conditions(&self) -> String {
        let mut parts = Vec::new();
        if self.wind_kt < 5.0 {
            parts.push(crate::t!("challenge.calm").to_string());
        } else {
            let side = match self.wind_rel {
                r if r.abs() <= 30.0 => crate::t!("challenge.headwind"),
                r if r.abs() >= 150.0 => crate::t!("challenge.tailwind"),
                r if r > 0.0 => crate::t!("challenge.from_the_right"),
                _ => crate::t!("challenge.from_the_left"),
            };
            let mut w = format!("{:.0} kt {side}", self.wind_kt);
            if self.gusts >= 0.3 {
                w += &format!(", {}", crate::t!("challenge.gusty"));
            }
            parts.push(w);
        }
        if self.rain > 0.0 {
            parts.push(crate::t!("challenge.rain").to_string());
        } else if self.haze >= 3.0 {
            parts.push(crate::t!("challenge.poor_visibility").to_string());
        }
        if !(6.5..=20.0).contains(&self.hour) {
            parts.push(crate::t!("challenge.night").to_string());
        }
        parts.push(format!("{:02}:{:02}", self.hour as u32, ((self.hour.fract()) * 60.0).round() as u32));
        parts.join(" · ")
    }
}

/// A challenge being flown: which, whether its brief has shown, and how it went.
pub struct Active {
    pub idx: usize,
    pub briefed: bool,
    pub result: Option<Result>,
    /// the player's live weather and real clock, put back after the challenge
    pub kept: (bool, bool),
    /// flown from the career: a pass counts towards its licences.
    pub career: bool,
}

pub fn by_id(id: &str) -> Option<usize> {
    ALL.iter().position(|c| c.id.eq_ignore_ascii_case(id))
}

/// Why a challenge was not passed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fail {
    Crashed,
    /// down on another runway, or off the runway
    WrongPlace,
    /// a very hard touchdown
    TooHard,
}

impl Fail {
    pub fn text(self) -> &'static str {
        match self {
            Fail::Crashed => crate::t!("challenge.fail_crashed"),
            Fail::WrongPlace => crate::t!("challenge.fail_wrong_place"),
            Fail::TooHard => crate::t!("challenge.fail_too_hard"),
        }
    }
}

/// How a try went.
#[derive(Clone, Debug, PartialEq)]
pub struct Result {
    pub fail: Option<Fail>,
    pub score: u32,
    pub landing: Option<Landing>,
    /// metres from the aiming point (+ long, - short)
    pub from_aim: Option<f64>,
    /// a new best for this challenge
    pub best: bool,
}

/// The landing's score, less up to 20 for touching down far from the aiming
/// point: the first 150 m either side are free.
pub fn judge(c: &Challenge, l: &Landing) -> Result {
    let on = l.on_runway.as_ref().filter(|r| l.airport.as_deref() == Some(c.airport) && r.runway == c.runway && r.off_centre.abs() <= r.width * 0.5 + 5.0);
    let Some(on) = on else {
        return Result { fail: Some(Fail::WrongPlace), score: 0, landing: Some(l.clone()), from_aim: None, best: false };
    };
    let from_aim = on.past_threshold - crate::traffic::aim_dist(on.length as f32) as f64;
    let zone = ((from_aim.abs() - 150.0).max(0.0) / 15.0).min(20.0);
    let score = (l.score as f64 - zone).max(0.0).round() as u32;
    let fail = (l.grade == Grade::VeryHard).then_some(Fail::TooHard);
    Result { fail, score: if fail.is_some() { 0 } else { score }, landing: Some(l.clone()), from_aim: Some(from_aim), best: false }
}

pub fn crashed() -> Result {
    Result { fail: Some(Fail::Crashed), score: 0, landing: None, from_aim: None, best: false }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub score: u32,
    /// Unix seconds
    pub date: i64,
    pub aircraft: String,
}

/// The best scores of each challenge, best first.
#[derive(Default, Serialize, Deserialize)]
pub struct Records {
    pub best: BTreeMap<String, Vec<Record>>,
}

const KEEP: usize = 5;

fn path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("fablegame").join("challenges.json"))
}

impl Records {
    pub fn load() -> Self {
        path().and_then(|p| std::fs::read(p).ok()).and_then(|d| serde_json::from_slice(&d).ok()).unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = p.with_extension("json.tmp");
        if let Ok(data) = serde_json::to_vec_pretty(self) {
            if std::fs::write(&tmp, data).is_ok() {
                let _ = std::fs::rename(&tmp, &p);
            }
        }
    }

    pub fn top(&self, id: &str) -> Option<u32> {
        self.best.get(id).and_then(|v| v.first()).map(|r| r.score)
    }

    /// Adds a passed try; true when it is the best yet.
    pub fn add(&mut self, id: &str, r: Record) -> bool {
        let list = self.best.entry(id.to_string()).or_default();
        let best = list.first().is_none_or(|b| r.score > b.score);
        list.push(r);
        list.sort_by(|a, b| b.score.cmp(&a.score).then(a.date.cmp(&b.date)));
        list.truncate(KEEP);
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flightlog::OnRunway;

    fn landing(airport: &str, runway: &str, past: f64, off: f64, sink: f64) -> Landing {
        let mut l = Landing { airport: Some(airport.into()), on_runway: Some(OnRunway { runway: runway.into(), off_centre: off, past_threshold: past, length: 1510.0, width: 45.0 }), sink_fpm: sink, g: 1.2, speed_kt: 60.0, bank: 0.0, crab: 0.0, bounces: 0, grade: Grade::of(sink, 0), score: 0 };
        l.score = l.rate();
        l
    }

    #[test]
    fn every_challenge_has_its_airport_runway_and_aircraft() {
        for c in &ALL {
            let (i, h) = c.runway().unwrap_or_else(|| panic!("{}: {} {}", c.id, c.airport, c.runway));
            let want = airports::ident_heading(c.runway).unwrap() as f64;
            let off = ((h - want + 540.0).rem_euclid(360.0) - 180.0).abs();
            // (runway numbers are magnetic: San Francisco's 28L heads 298° true,
            // and Queenstown's 23 heads 257° with the 24° east variation there)
            assert!(off < 30.0, "{}: runway {} heads {h:.0}", c.id, c.runway);
            assert_eq!(airports::Db::get().apts[i as usize].ident, c.airport);
            assert!(crate::aircraft::by_key(c.aircraft).is_some(), "{}: {}", c.id, c.aircraft);
            assert!(by_id(c.id).is_some());
        }
    }

    #[test]
    fn the_aiming_point_counts_and_the_wrong_runway_fails() {
        let c = &ALL[0];
        // EBAW's 1510 m: aiming point 300 m in.
        let good = judge(c, &landing("EBAW", "29", 320.0, 1.0, 90.0));
        assert_eq!(good.fail, None);
        assert!(good.score >= 95, "{}", good.score);
        let long = judge(c, &landing("EBAW", "29", 900.0, 1.0, 90.0));
        assert_eq!(good.score - long.score, 20);
        assert_eq!(judge(c, &landing("EBAW", "11", 320.0, 1.0, 90.0)).fail, Some(Fail::WrongPlace));
        assert_eq!(judge(c, &landing("EBAW", "29", 320.0, 40.0, 90.0)).fail, Some(Fail::WrongPlace));
        assert_eq!(judge(c, &landing("EBAW", "29", 320.0, 1.0, 900.0)).fail, Some(Fail::TooHard));
    }

    #[test]
    fn records_keep_the_best_five() {
        let mut r = Records::default();
        assert!(r.add("first", Record { score: 70, date: 1, aircraft: "x".into() }));
        assert!(!r.add("first", Record { score: 60, date: 2, aircraft: "x".into() }));
        assert!(r.add("first", Record { score: 90, date: 3, aircraft: "x".into() }));
        for k in 0..5 {
            r.add("first", Record { score: 10 + k, date: 4, aircraft: "x".into() });
        }
        assert_eq!(r.best["first"].len(), 5);
        assert_eq!(r.top("first"), Some(90));
    }

    #[test]
    fn the_wind_is_set_against_the_runway() {
        let c = &ALL[1];
        let (_, h) = c.runway().unwrap();
        assert!((((c.wind_from() - h - 80.0) + 540.0).rem_euclid(360.0) - 180.0).abs() < 1e-6);
    }
}
