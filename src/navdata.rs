//! us fixes, airways and procedures from the faa cifp, built by assets/mkcifp.py.
//! the faa marks the cifp not for real world navigation, it is for the sim only.
//! airports without real procedures get generated ones for every runway.

use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex, OnceLock};

/// one coded leg as the file has it.
#[derive(Clone, Debug, PartialEq)]
pub struct RawLeg {
    /// empty for heading legs.
    pub fix: String,
    pub lat: f64,
    pub lon: f64,
    /// arinc path terminator, like tf or va.
    pub pt: String,
    /// true course, if the leg has one.
    pub course: Option<f64>,
    /// altitude constraint: +, -, b, @ (blank) and so on, with its two altitudes in feet.
    pub alt_desc: char,
    pub alt1: Option<f64>,
    pub alt2: Option<f64>,
    /// speed limit in knots.
    pub speed: Option<f64>,
}

/// one route of a procedure, a transition or its common part.
#[derive(Clone, Debug)]
pub struct Route {
    /// d sid, e star, f approach.
    pub kind: char,
    pub name: String,
    /// rw10l, rw10b, all, a fix name, or empty for an approach's final.
    pub trans: String,
    pub legs: Vec<RawLeg>,
}

pub struct Db {
    pub fixes: HashMap<String, Vec<(f64, f64)>>,
    /// airway name to its segments, each a list of fixes in order.
    pub airways: HashMap<String, Vec<Vec<(String, f64, f64)>>>,
    /// airport to its procedure routes.
    pub procs: HashMap<String, Vec<Route>>,
}

fn num(s: &str) -> Option<f64> {
    s.parse().ok()
}

fn leg(s: &str) -> Option<RawLeg> {
    let f: Vec<&str> = s.split(',').collect();
    if f.len() < 9 {
        return None;
    }
    Some(RawLeg {
        fix: f[0].to_string(),
        lat: num(f[1]).unwrap_or(0.0),
        lon: num(f[2]).unwrap_or(0.0),
        pt: f[3].to_string(),
        course: num(f[4]),
        alt_desc: f[5].chars().next().unwrap_or(' '),
        alt1: num(f[6]),
        alt2: num(f[7]),
        speed: num(f[8]),
    })
}

impl Db {
    fn parse(text: &str) -> Db {
        let mut db = Db { fixes: HashMap::new(), airways: HashMap::new(), procs: HashMap::new() };
        for l in text.lines() {
            let f: Vec<&str> = l.split('\t').collect();
            match (f.first().copied(), f.len()) {
                (Some("F"), 4) => {
                    if let (Some(a), Some(b)) = (num(f[2]), num(f[3])) {
                        db.fixes.entry(f[1].to_string()).or_default().push((a, b));
                    }
                }
                (Some("W"), 3) => {
                    let seg: Vec<(String, f64, f64)> = f[2]
                        .split(';')
                        .filter_map(|p| {
                            let q: Vec<&str> = p.split(',').collect();
                            Some((q.first()?.to_string(), num(q.get(1)?)?, num(q.get(2)?)?))
                        })
                        .collect();
                    db.airways.entry(f[1].to_string()).or_default().push(seg);
                }
                (Some("P"), 7) => {
                    let legs = f[6].split(';').filter_map(leg).collect();
                    let r = Route { kind: f[2].chars().next().unwrap_or('?'), name: f[3].to_string(), trans: f[4].to_string(), legs };
                    db.procs.entry(f[1].to_string()).or_default().push(r);
                }
                _ => {}
            }
        }
        db
    }

    pub fn get() -> &'static Db {
        static DB: OnceLock<Db> = OnceLock::new();
        DB.get_or_init(|| {
            let mut text = String::new();
            let _ = flate2::read::GzDecoder::new(&include_bytes!("../assets/cifp.tsv.gz")[..]).read_to_string(&mut text);
            Db::parse(&text)
        })
    }

    /// the fix of that name nearest a point.
    pub fn fix(&self, ident: &str, near: Option<(f64, f64)>) -> Option<(f64, f64)> {
        let ps = self.fixes.get(ident)?;
        match near {
            Some(r) => ps.iter().copied().min_by(|a, b| crate::flightplan::dist(r, *a).total_cmp(&crate::flightplan::dist(r, *b))),
            None => ps.first().copied(),
        }
    }

    /// the fixes between two on an airway, both ends left out.
    pub fn airway(&self, name: &str, from: &str, to: &str) -> Option<Vec<(String, f64, f64)>> {
        for seg in self.airways.get(name)? {
            let (Some(a), Some(b)) = (seg.iter().position(|p| p.0 == from), seg.iter().position(|p| p.0 == to)) else { continue };
            return Some(if a < b { seg[a + 1..b].to_vec() } else { seg[b + 1..a].iter().rev().cloned().collect() });
        }
        None
    }

    /// the real routes at an airport, with generated ones for the kinds it has none of.
    pub fn routes_at(&self, apt: &str) -> Arc<Vec<Route>> {
        static CACHE: OnceLock<Mutex<HashMap<String, Arc<Vec<Route>>>>> = OnceLock::new();
        let mut cache = CACHE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
        if let Some(r) = cache.get(apt) {
            return r.clone();
        }
        let mut v: Vec<Route> = self.procs.get(apt).cloned().unwrap_or_default();
        let made = generated(apt);
        for kind in ['D', 'F'] {
            if !v.iter().any(|r| r.kind == kind) {
                v.extend(made.iter().filter(|r| r.kind == kind).cloned());
            }
        }
        let v = Arc::new(v);
        cache.insert(apt.to_string(), v.clone());
        v
    }

    /// the procedure names of a kind at an airport, sorted.
    pub fn names(&self, apt: &str, kind: char) -> Vec<String> {
        let mut v: Vec<String> = self.routes_at(apt).iter().filter(|r| r.kind == kind).map(|r| r.name.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    fn routes(&self, apt: &str, kind: char, name: &str) -> Vec<Route> {
        self.routes_at(apt).iter().filter(|r| r.kind == kind && r.name == name).cloned().collect()
    }

    /// a procedure's runway transitions, like rw10l.
    pub fn runway_trans(&self, apt: &str, kind: char, name: &str) -> Vec<String> {
        let mut v: Vec<String> = self.routes(apt, kind, name).into_iter().filter(|r| r.trans.starts_with("RW")).map(|r| r.trans).collect();
        v.sort();
        v
    }

    /// its other transitions: enroute ones for a sid or star, approach ones for an approach.
    pub fn other_trans(&self, apt: &str, kind: char, name: &str) -> Vec<String> {
        let mut v: Vec<String> = self.routes(apt, kind, name).into_iter().filter(|r| !r.trans.is_empty() && r.trans != "ALL" && !r.trans.starts_with("RW")).map(|r| r.trans).collect();
        v.sort();
        v
    }

    /// the legs of a whole procedure, in flying order.
    pub fn assemble(&self, apt: &str, kind: char, name: &str, rwy: Option<&str>, trans: Option<&str>) -> Vec<RawLeg> {
        let routes = self.routes(apt, kind, name);
        let pick = |t: &str| routes.iter().find(|r| r.trans == t).map(|r| r.legs.clone()).unwrap_or_default();
        let common = routes.iter().find(|r| r.trans == "ALL" || (kind == 'F' && r.trans.is_empty())).map(|r| r.legs.clone()).unwrap_or_default();
        let rw = rwy.map(pick).unwrap_or_default();
        let tr = trans.map(pick).unwrap_or_default();
        let parts = match kind {
            'D' => [rw, common, tr],
            'E' => [tr, common, rw],
            _ => [tr, common, Vec::new()],
        };
        let mut out: Vec<RawLeg> = Vec::new();
        for l in parts.into_iter().flatten() {
            // one route ends where the next starts.
            if !l.fix.is_empty() && out.last().is_some_and(|p| p.fix == l.fix && p.pt != "FM") && l.pt == "IF" {
                continue;
            }
            out.push(l);
        }
        out
    }
}

/// generated procedures for each runway end with known thresholds.
/// approach g28l is an rnav t: iaf straight in 16 nm out, iafs 5 nm either side of the if at 10 nm,
/// faf at 5 nm on a 3 degree path, threshold at 50 ft. departure gen28l climbs on the runway
/// heading to 500 ft then goes to a fix 8 nm out.
pub fn generated(apt: &str) -> Vec<Route> {
    use crate::flightplan::{ahead, course};
    let db = crate::airports::Db::get();
    let Some(i) = db.find(apt) else { return Vec::new() };
    let a = &db.apts[i as usize];
    let elev = a.elev as f64;
    let ft = |agl: f64| ((elev + agl) / 100.0).round() * 100.0;
    let nm = 1852.0;
    let fix = |id: String, p: (f64, f64), pt: &str, crs: Option<f64>, desc: char, alt: Option<f64>| RawLeg { fix: id, lat: p.0, lon: p.1, pt: pt.into(), course: crs, alt_desc: desc, alt1: alt, alt2: None, speed: None };
    let mut out = Vec::new();
    for r in a.rwys.iter().filter(|r| r.len >= 500.0) {
        let (Some(p0), Some(p1)) = (r.pos[0], r.pos[1]) else { continue };
        for e in 0..2 {
            let id = r.ident[e].trim().to_string();
            if id.is_empty() || id.len() > 3 {
                continue;
            }
            let (thr, far) = if e == 0 { (p0, p1) } else { (p1, p0) };
            let c = course(thr, far);
            let back = c + 180.0;
            let faf = ahead(thr, back, 5.0 * nm);
            let iff = ahead(thr, back, 10.0 * nm);
            let iaf = ahead(thr, back, 16.0 * nm);
            let left = ahead(iff, c - 90.0, 5.0 * nm);
            let right = ahead(iff, c + 90.0, 5.0 * nm);
            let name = format!("G{id}");
            let route = |trans: String, legs: Vec<RawLeg>| Route { kind: 'F', name: name.clone(), trans, legs };
            for (t, p) in [(format!("IA{id}"), iaf), (format!("IL{id}"), left), (format!("IR{id}"), right)] {
                out.push(route(t.clone(), vec![fix(t, p, "IF", None, '+', Some(ft(4000.0))), fix(format!("IF{id}"), iff, "TF", None, '+', Some(ft(3000.0)))]));
            }
            out.push(route(String::new(), vec![
                fix(format!("IF{id}"), iff, "IF", None, '+', Some(ft(3000.0))),
                fix(format!("FF{id}"), faf, "CF", Some(c), ' ', Some(ft(1600.0))),
                fix(format!("RW{id}"), thr, "CF", Some(c), ' ', Some(ft(50.0))),
            ]));
            let dp = ahead(thr, c, (r.len as f64) + 8.0 * nm);
            out.push(Route { kind: 'D', name: format!("GEN{id}"), trans: format!("RW{id}"), legs: vec![
                RawLeg { fix: String::new(), lat: 0.0, lon: 0.0, pt: "VA".into(), course: Some(c), alt_desc: '+', alt1: Some(ft(500.0)), alt2: None, speed: None },
                fix(format!("DP{id}"), dp, "DF", None, '+', Some(ft(3000.0))),
            ] });
        }
    }
    out
}

/// whether a procedure is a generated one.
pub fn is_generated(name: &str) -> bool {
    name.starts_with("GEN") || (name.starts_with('G') && name[1..].starts_with(|c: char| c.is_ascii_digit()))
}

/// a readable name for any procedure.
pub fn label(kind: char, name: &str) -> String {
    match (kind, is_generated(name)) {
        ('F', true) => crate::t!("plan.rnav_generated", &name[1..]),
        ('D', true) => crate::t!("plan.departure_generated", &name[3..]),
        ('F', false) => approach_label(name),
        _ => name.to_string(),
    }
}

/// the runway transition that serves a runway: rw28l, or rw28b for both of a pair, or rw28.
pub fn trans_for(list: &[String], rwy: &str) -> Option<String> {
    let num: String = rwy.chars().take_while(|c| c.is_ascii_digit()).collect();
    let want = format!("RW{rwy}");
    list.iter().find(|t| **t == want).or_else(|| list.iter().find(|t| **t == format!("RW{num}B"))).or_else(|| list.iter().find(|t| **t == format!("RW{num}"))).cloned()
}

/// the runway an approach is for: i28l is 28l.
pub fn approach_runway(name: &str) -> String {
    name.chars().skip(1).take_while(|c| c.is_ascii_digit() || matches!(c, 'L' | 'R' | 'C')).collect()
}

/// a readable approach name: i28l is ils 28l.
pub fn approach_label(name: &str) -> String {
    let mut c = name.chars();
    let kind = match c.next() {
        Some('I') => "ILS",
        Some('L') => "LOC",
        Some('B') => "LOC BC",
        Some('H') => "RNP",
        Some('R') => "RNAV",
        Some('P') => "GPS",
        Some('V') => "VOR",
        Some('D') => "VOR/DME",
        Some('S') => "VOR",
        Some('N') => "NDB",
        Some('Q') => "NDB/DME",
        Some('X') => "LDA",
        Some('U') => "SDF",
        _ => return name.to_string(),
    };
    let rest: String = c.collect();
    let rest = rest.trim_end_matches('-');
    // the letter after the runway, like z or y, is the variant.
    let (rwy, var) = match rest.char_indices().find(|(_, ch)| ch.is_ascii_alphabetic() && !matches!(*ch, 'L' | 'R' | 'C')) {
        Some((i, _)) if i > 0 => (&rest[..i], &rest[i..]),
        _ => (rest, ""),
    };
    if var.is_empty() { format!("{kind} {rwy}") } else { format!("{kind} {var} {rwy}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ksfo() {
        let db = Db::get();
        assert!(db.fix("SAHEY", None).is_some());
        let sids = db.names("KSFO", 'D');
        assert!(!sids.is_empty());
        let apps = db.names("KSFO", 'F');
        assert!(apps.iter().any(|a| a.starts_with('I')), "{apps:?}");
        // every leg with a fix has a position.
        for r in db.procs.get("KSFO").unwrap() {
            for l in &r.legs {
                assert!(l.fix.is_empty() || (l.lat != 0.0 && l.lon != 0.0), "{r:?}");
            }
        }
        let name = &apps.iter().find(|a| a.starts_with('I')).unwrap().clone();
        let legs = db.assemble("KSFO", 'F', name, None, None);
        assert!(legs.last().is_some_and(|l| l.fix.starts_with("RW")), "{legs:?}");
    }

    #[test]
    fn generated_procedures() {
        // london heathrow has no faa data: every runway end gets an approach and a departure.
        let db = Db::get();
        let apps = db.names("EGLL", 'F');
        assert!(apps.iter().any(|a| a == "G27L"), "{apps:?}");
        assert!(db.names("EGLL", 'D').iter().any(|a| a == "GEN27L"));
        let legs = db.assemble("EGLL", 'F', "G27L", None, Some("IL27L"));
        let ids: Vec<&str> = legs.iter().map(|l| l.fix.as_str()).collect();
        assert_eq!(ids, vec!["IL27L", "IF27L", "FF27L", "RW27L"]);
        // the final is flown on the runway course, about 270 true at heathrow.
        assert!((legs[3].course.unwrap() - 270.0).abs() < 5.0, "{:?}", legs[3]);
        // real data stays real.
        assert!(!db.names("KSFO", 'F').iter().any(|a| is_generated(a)));
        assert_eq!(approach_runway("G27L"), "27L");
        assert!(label('F', "G27L").contains("27L"));
    }

    #[test]
    fn labels_and_runways() {
        assert_eq!(approach_label("I28L"), "ILS 28L");
        assert_eq!(approach_label("H10RZ"), "RNP Z 10R");
        assert_eq!(approach_label("R28RY"), "RNAV Y 28R");
        let l = vec!["RW10B".to_string(), "RW28L".to_string()];
        assert_eq!(trans_for(&l, "10R").as_deref(), Some("RW10B"));
        assert_eq!(trans_for(&l, "28L").as_deref(), Some("RW28L"));
        assert_eq!(trans_for(&l, "01L"), None);
    }
}
