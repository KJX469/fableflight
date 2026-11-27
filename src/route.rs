//! builds a whole flight plan from two airports and their runways.
//! sid, star and approach are the real faa ones where the cifp has them,
//! picked for the runways and the least detour, generated ones elsewhere.
//! the enroute part follows us airways, and vor to vor legs everywhere,
//! found with a* over that network. not for real world navigation.

use crate::flightplan::{course, dist, Phase, Pick, Plan, Waypoint};
use crate::{airports, nav, navdata};
use std::collections::{BinaryHeap, HashMap};
use std::sync::OnceLock;

/// one end of a runway.
#[derive(Clone, Debug, PartialEq)]
pub struct RwyEnd {
    pub ident: String,
    /// threshold, degrees.
    pub thr: (f64, f64),
    /// true course down the runway.
    pub course: f64,
    /// metres.
    pub len: f64,
}

/// the runway ends of an airport, longest first.
pub fn runway_ends(apt: u32) -> Vec<RwyEnd> {
    let a = &airports::Db::get().apts[apt as usize];
    let mut out = Vec::new();
    for r in &a.rwys {
        for e in 0..2 {
            let id = r.ident[e].trim();
            if id.is_empty() {
                continue;
            }
            let (Some(p), Some(q)) = (r.pos[e], r.pos[1 - e]) else {
                out.push(RwyEnd { ident: id.to_string(), thr: (a.lat, a.lon), course: rwy_heading(id).unwrap_or(0.0), len: r.len as f64 });
                continue;
            };
            out.push(RwyEnd { ident: id.to_string(), thr: p, course: course(p, q), len: r.len as f64 });
        }
    }
    out.sort_by(|a, b| b.len.total_cmp(&a.len).then(a.ident.cmp(&b.ident)));
    out
}

/// the heading a runway number stands for.
fn rwy_heading(id: &str) -> Option<f64> {
    let n: String = id.chars().take_while(|c| c.is_ascii_digit()).collect();
    n.parse::<f64>().ok().map(|n| n * 10.0)
}

/// the runway end most into the wind (wind from, degrees), longer ones first on a tie.
pub fn into_wind(ends: &[RwyEnd], wind_from: f64) -> Option<RwyEnd> {
    ends.iter().max_by(|a, b| {
        let s = |e: &RwyEnd| (e.course - wind_from).to_radians().cos() + e.len / 40_000.0;
        s(a).total_cmp(&s(b))
    }).cloned()
}

#[derive(Clone)]
struct Node {
    ident: String,
    lat: f64,
    lon: f64,
    vor: bool,
}

struct Graph {
    nodes: Vec<Node>,
    /// to, metres, airway (u16::MAX for a direct leg).
    adj: Vec<Vec<(u32, f32, u16)>>,
    airways: Vec<String>,
    /// nodes by 2 degree cell.
    grid: HashMap<(i32, i32), Vec<u32>>,
}

const DIRECT: u16 = u16::MAX;

fn cell(lat: f64, lon: f64) -> (i32, i32) {
    ((lat / 2.0).floor() as i32, (lon / 2.0).floor() as i32)
}

impl Graph {
    fn get() -> &'static Graph {
        static G: OnceLock<Graph> = OnceLock::new();
        G.get_or_init(Graph::build)
    }

    fn build() -> Graph {
        let mut g = Graph { nodes: Vec::new(), adj: Vec::new(), airways: Vec::new(), grid: HashMap::new() };
        let mut index: HashMap<(String, i32, i32), u32> = HashMap::new();
        let mut node = |g: &mut Graph, ident: &str, lat: f64, lon: f64, vor: bool| -> u32 {
            let key = (ident.to_string(), (lat * 50.0).round() as i32, (lon * 50.0).round() as i32);
            if let Some(&i) = index.get(&key) {
                if vor {
                    g.nodes[i as usize].vor = true;
                }
                return i;
            }
            let i = g.nodes.len() as u32;
            g.nodes.push(Node { ident: ident.to_string(), lat, lon, vor });
            g.adj.push(Vec::new());
            g.grid.entry(cell(lat, lon)).or_default().push(i);
            index.insert(key, i);
            i
        };
        let vors: Vec<&nav::Station> = nav::db().iter().filter(|s| s.kind == nav::Kind::Vor).collect();
        for s in &vors {
            node(&mut g, &s.ident, s.lat, s.lon, true);
        }
        let db = navdata::Db::get();
        let mut names: Vec<&String> = db.airways.keys().collect();
        names.sort();
        for name in names {
            let aw = g.airways.len() as u16;
            g.airways.push(name.clone());
            for seg in &db.airways[name] {
                let ids: Vec<u32> = seg.iter().map(|(id, la, lo)| node(&mut g, id, *la, *lo, false)).collect();
                for w in ids.windows(2) {
                    let (a, b) = (&g.nodes[w[0] as usize], &g.nodes[w[1] as usize]);
                    let d = dist((a.lat, a.lon), (b.lat, b.lon)) as f32;
                    g.adj[w[0] as usize].push((w[1], d, aw));
                    g.adj[w[1] as usize].push((w[0], d, aw));
                }
            }
        }
        // vor to vor legs up to 300 km, the nearest ten of each.
        let vor_ids: Vec<u32> = (0..g.nodes.len() as u32).filter(|i| g.nodes[*i as usize].vor).collect();
        for &i in &vor_ids {
            let n = &g.nodes[i as usize];
            let (cy, cx) = cell(n.lat, n.lon);
            let mut near: Vec<(f32, u32)> = Vec::new();
            for dy in -2..=2 {
                for dx in -2..=2 {
                    for &j in g.grid.get(&(cy + dy, cx + dx)).into_iter().flatten() {
                        let m = &g.nodes[j as usize];
                        if j == i || !m.vor {
                            continue;
                        }
                        let d = dist((n.lat, n.lon), (m.lat, m.lon)) as f32;
                        if d > 15_000.0 && d < 300_000.0 {
                            near.push((d, j));
                        }
                    }
                }
            }
            near.sort_by(|a, b| a.0.total_cmp(&b.0));
            for (d, j) in near.into_iter().take(10) {
                g.adj[i as usize].push((j, d, DIRECT));
            }
        }
        g
    }

    /// nodes within r metres of a point, nearest first.
    fn near(&self, p: (f64, f64), r: f64, max: usize) -> Vec<(u32, f64)> {
        let (cy, cx) = cell(p.0, p.1);
        let k = (r / 222_000.0).ceil() as i32 + 1;
        let mut v: Vec<(u32, f64)> = Vec::new();
        for dy in -k..=k {
            for dx in -k..=k {
                for &j in self.grid.get(&(cy + dy, cx + dx)).into_iter().flatten() {
                    let n = &self.nodes[j as usize];
                    let d = dist(p, (n.lat, n.lon));
                    if d < r {
                        v.push((j, d));
                    }
                }
            }
        }
        v.sort_by(|a, b| a.1.total_cmp(&b.1));
        v.truncate(max);
        v
    }
}

#[derive(PartialEq)]
struct Open(f64, u32);
impl Eq for Open {}
impl Ord for Open {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        o.0.total_cmp(&self.0)
    }
}
impl PartialOrd for Open {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

/// the enroute fixes from one point to another, via airways where they help.
pub fn enroute(from: (f64, f64), to: (f64, f64)) -> Vec<Waypoint> {
    let total = dist(from, to);
    if total < 60_000.0 {
        return Vec::new();
    }
    let g = Graph::get();
    let n = g.nodes.len();
    // the virtual start and end are n and n + 1.
    let (start, end) = (n as u32, n as u32 + 1);
    let reach = (total * 0.25).clamp(40_000.0, 160_000.0);
    let firsts = g.near(from, reach, 12);
    let lasts: HashMap<u32, f64> = g.near(to, reach, 12).into_iter().collect();
    // a leg off the network costs more, a hop a little, so a short airway beats a long direct.
    let hop = 4_000.0;
    let off = 1.25;
    let limit = total * 1.35 + 150_000.0;
    let mut best: HashMap<u32, (f64, u32, u16)> = HashMap::new();
    let mut heap = BinaryHeap::new();
    best.insert(start, (0.0, start, DIRECT));
    heap.push(Open(total, start));
    let pos = |i: u32| if i == start { from } else if i == end { to } else { (g.nodes[i as usize].lat, g.nodes[i as usize].lon) };
    let mut done = false;
    let mut expanded = 0usize;
    while let Some(Open(_, i)) = heap.pop() {
        if i == end {
            done = true;
            break;
        }
        expanded += 1;
        if expanded > 200_000 {
            break;
        }
        let gi = best[&i].0;
        let mut edges: Vec<(u32, f64, u16)> = Vec::new();
        if i == start {
            edges.extend(firsts.iter().map(|(j, d)| (*j, *d * off, DIRECT)));
            edges.push((end, total * off * 1.15, DIRECT));
        } else {
            edges.extend(g.adj[i as usize].iter().map(|(j, d, a)| (*j, *d as f64 * if *a == DIRECT { 1.12 } else { 1.0 }, *a)));
            if let Some(d) = lasts.get(&i) {
                edges.push((end, d * off, DIRECT));
            }
        }
        for (j, c, a) in edges {
            let pj = pos(j);
            if j != end && dist(from, pj) + dist(pj, to) > limit {
                continue;
            }
            let cost = gi + c + hop;
            if best.get(&j).is_none_or(|b| cost < b.0) {
                best.insert(j, (cost, i, a));
                heap.push(Open(cost + dist(pj, to), j));
            }
        }
    }
    if !done {
        return Vec::new();
    }
    let mut path = Vec::new();
    let mut at = end;
    while at != start {
        let (_, prev, aw) = best[&at];
        if at != end {
            let nd = &g.nodes[at as usize];
            let mut w = match nav::db().iter().find(|s| nd.vor && s.kind == nav::Kind::Vor && s.ident == nd.ident && dist((s.lat, s.lon), (nd.lat, nd.lon)) < 2000.0) {
                Some(s) => Waypoint::station(s),
                None => Waypoint::fix(&nd.ident, nd.lat, nd.lon),
            };
            w.via = (aw != DIRECT).then(|| g.airways[aw as usize].clone());
            path.push(w);
        }
        at = prev;
    }
    path.reverse();
    path
}

/// what the auto planner chose, for the planner to show.
#[derive(Clone, Debug, Default)]
pub struct Made {
    pub dep_rwy: Option<String>,
    pub arr_rwy: Option<String>,
    pub note: Vec<String>,
}

/// the last fixed point of some legs.
fn last_fix(legs: &[navdata::RawLeg]) -> Option<(f64, f64)> {
    legs.iter().rev().find(|l| !l.fix.is_empty()).map(|l| (l.lat, l.lon))
}
fn first_fix(legs: &[navdata::RawLeg]) -> Option<(f64, f64)> {
    legs.iter().find(|l| !l.fix.is_empty()).map(|l| (l.lat, l.lon))
}

/// the sid out of a runway that turns least away from where we are going.
fn pick_sid(apt: &str, rwy: &str, dep: (f64, f64), toward: (f64, f64)) -> Option<Pick> {
    let db = navdata::Db::get();
    let total = dist(dep, toward);
    // (score, detour): generated ones score worse, so only win when nothing real fits.
    let mut best: Option<(f64, f64, Pick)> = None;
    for name in db.names(apt, 'D') {
        let rws = db.runway_trans(apt, 'D', &name);
        let rw = if rws.is_empty() { None } else { match navdata::trans_for(&rws, rwy) { Some(r) => Some(r), None => continue } };
        let mut trans: Vec<Option<String>> = vec![None];
        trans.extend(db.other_trans(apt, 'D', &name).into_iter().map(Some));
        for t in trans {
            let legs = db.assemble(apt, 'D', &name, rw.as_deref(), t.as_deref());
            let Some(exit) = last_fix(&legs) else { continue };
            // generated ones only when nothing real fits.
            let fake = if navdata::is_generated(&name) { total * 0.5 } else { 0.0 };
            let detour = dist(dep, exit) + dist(exit, toward) - total;
            if detour < total * 0.35 + 30_000.0 && best.as_ref().is_none_or(|b| detour + fake < b.0) {
                best = Some((detour + fake, detour, Pick { apt: apt.into(), name: name.clone(), rwy: rw.clone(), trans: t }));
            }
        }
    }
    best.map(|b| b.2)
}

/// the star into a runway that turns least away from the way we come.
fn pick_star(apt: &str, rwy: &str, from: (f64, f64), arr: (f64, f64)) -> Option<Pick> {
    let db = navdata::Db::get();
    let total = dist(from, arr);
    let mut best: Option<(f64, Pick)> = None;
    for name in db.names(apt, 'E') {
        let rws = db.runway_trans(apt, 'E', &name);
        let rw = if rws.is_empty() { None } else { match navdata::trans_for(&rws, rwy) { Some(r) => Some(r), None => continue } };
        let mut trans: Vec<Option<String>> = vec![None];
        trans.extend(db.other_trans(apt, 'E', &name).into_iter().map(Some));
        for t in trans {
            let legs = db.assemble(apt, 'E', &name, rw.as_deref(), t.as_deref());
            let Some(entry) = first_fix(&legs) else { continue };
            let detour = dist(from, entry) + dist(entry, arr) - total;
            if best.as_ref().is_none_or(|b| detour < b.0) {
                best = Some((detour, Pick { apt: apt.into(), name: name.clone(), rwy: rw.clone(), trans: t }));
            }
        }
    }
    best.filter(|(d, _)| *d < total * 0.3 + 30_000.0).map(|b| b.1)
}

/// the approach to a runway: ils first, then rnp and rnav, then the rest, generated last.
pub fn pick_app(apt: &str, rwy: &str, from: (f64, f64), after: Option<&str>) -> Option<Pick> {
    let db = navdata::Db::get();
    let rank = |n: &str| match n.chars().next() {
        _ if navdata::is_generated(n) => 9,
        Some('I') => 0,
        Some('H') | Some('R') => 1,
        Some('L') => 2,
        Some('P') => 3,
        _ => 5,
    };
    let num: String = rwy.chars().take_while(|c| c.is_ascii_digit()).collect();
    let mut names: Vec<String> = db.names(apt, 'F').into_iter().filter(|n| {
        let r = navdata::approach_runway(n);
        r == rwy || (r == num && !rwy.ends_with(['L', 'R', 'C']))
    }).collect();
    names.sort_by_key(|n| (rank(n), n.clone()));
    let name = names.into_iter().next()?;
    let trans = db.other_trans(apt, 'F', &name);
    // the transition that starts where the star ends, else the nearest start.
    let t = after.and_then(|a| trans.iter().find(|t| *t == a).cloned()).or_else(|| {
        trans.iter().filter_map(|t| {
            let legs = db.assemble(apt, 'F', &name, None, Some(t));
            first_fix(&legs).map(|p| (dist(from, p), t.clone()))
        }).min_by(|a, b| a.0.total_cmp(&b.0)).map(|x| x.1)
    });
    Some(Pick { apt: apt.into(), name, rwy: None, trans: t })
}

/// a cruise altitude for a trip: higher the longer, within what the type flies,
/// odd thousands eastbound and even westbound (semicircular rule, true course).
pub fn cruise_for(spec: &crate::aircraft::Spec, dep: (f64, f64), arr: (f64, f64)) -> f64 {
    let nm = dist(dep, arr) / 1852.0;
    let key = spec.keys.first().copied().unwrap_or("");
    let max = match key {
        "sr71" => 75_000.0,
        "concorde" => 58_000.0,
        "g650" => 45_000.0,
        "b747" => 37_000.0,
        "f22" | "f4" | "fa18" => 40_000.0,
        _ if spec.is_jet() => 37_000.0,
        _ => 9_500.0,
    };
    let east = course(dep, arr) < 180.0;
    if spec.is_jet() {
        let want = (nm * 80.0 + 9_000.0).min(max);
        let k = (want / 1000.0).floor() as i64;
        let k = if (k % 2 == 1) == east { k } else { k - 1 };
        ((k.max(5)) as f64) * 1000.0
    } else {
        // vfr: thousands plus 500.
        let want = (nm * 45.0 + 2_500.0).min(max);
        let k = (want / 1000.0).floor() as i64;
        let k = if (k % 2 == 1) == east { k } else { k - 1 };
        (k.max(3)) as f64 * 1000.0 + 500.0
    }
}

/// a whole plan from one airport to another. runways none: the longest.
pub fn make(dep_apt: u32, dep_rwy: Option<&str>, arr_apt: u32, arr_rwy: Option<&str>, spec: &crate::aircraft::Spec) -> (Plan, Made) {
    let adb = airports::Db::get();
    let (da, aa) = (&adb.apts[dep_apt as usize], &adb.apts[arr_apt as usize]);
    let dep = (da.lat, da.lon);
    let arr = (aa.lat, aa.lon);
    let d_ends = runway_ends(dep_apt);
    let a_ends = runway_ends(arr_apt);
    let d_rwy = dep_rwy.map(str::to_string).or_else(|| d_ends.first().map(|e| e.ident.clone()));
    let a_rwy = arr_rwy.map(str::to_string).or_else(|| a_ends.first().map(|e| e.ident.clone()));
    let mut made = Made { dep_rwy: d_rwy.clone(), arr_rwy: a_rwy.clone(), note: Vec::new() };
    let mut plan = Plan::new();
    plan.cruise_ft = cruise_for(spec, dep, arr);
    plan.push(Waypoint::airport(arr_apt));
    let db = navdata::Db::get();
    let sid = d_rwy.as_deref().and_then(|r| pick_sid(&da.ident, r, dep, arr));
    let sid_exit = sid.as_ref().and_then(|p| last_fix(&db.assemble(&p.apt, 'D', &p.name, p.rwy.as_deref(), p.trans.as_deref()))).unwrap_or(dep);
    let star = a_rwy.as_deref().and_then(|r| pick_star(&aa.ident, r, sid_exit, arr));
    let star_legs = star.as_ref().map(|p| db.assemble(&p.apt, 'E', &p.name, p.rwy.as_deref(), p.trans.as_deref())).unwrap_or_default();
    let star_end = star_legs.iter().rev().find(|l| !l.fix.is_empty()).map(|l| l.fix.clone());
    let app = a_rwy.as_deref().and_then(|r| pick_app(&aa.ident, r, last_fix(&star_legs).unwrap_or(sid_exit), star_end.as_deref()));
    if let Some(p) = &sid {
        plan.set_proc(Phase::Sid, Some(p.clone()), Some(dep));
    }
    if let Some(p) = &star {
        plan.set_proc(Phase::Star, Some(p.clone()), Some(dep));
    }
    if let Some(p) = &app {
        plan.set_proc(Phase::App, Some(p.clone()), Some(dep));
    }
    // enroute between the sid's last fix and the first fix of the arrival.
    let arr_start = plan.wps.iter().find(|w| matches!(w.phase, Some((Phase::Star | Phase::App, _))) && !w.is_heading()).map(|w| w.at()).unwrap_or(arr);
    let from_id = plan.wps.iter().rev().find(|w| matches!(w.phase, Some((Phase::Sid, _))) && !w.is_heading()).map(|w| w.ident.clone());
    let mut mid = enroute(sid_exit, arr_start);
    // the network's ends may repeat the procedure fixes.
    if mid.first().is_some_and(|w| Some(&w.ident) == from_id.as_ref()) {
        mid.remove(0);
    }
    let to_id = plan.wps.iter().find(|w| matches!(w.phase, Some((Phase::Star | Phase::App, _))) && !w.is_heading()).map(|w| w.ident.clone());
    if mid.last().is_some_and(|w| Some(&w.ident) == to_id.as_ref()) {
        mid.pop();
    }
    if mid.is_empty() && dist(sid_exit, arr_start) > 150_000.0 {
        made.note.push(crate::t!("plan.note_direct").to_string());
    }
    for w in mid {
        plan.push(w);
    }
    if sid.is_none() {
        made.note.push(crate::t!("plan.note_no_sid").to_string());
    }
    if app.is_none() {
        made.note.push(crate::t!("plan.note_no_app").to_string());
    }
    (plan, made)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sfo_to_lax() {
        let db = airports::Db::get();
        let (sfo, lax) = (db.find("KSFO").unwrap(), db.find("KLAX").unwrap());
        let (p, made) = make(sfo, Some("28R"), lax, Some("24R"), &crate::aircraft::b747::SPEC);
        let ids: Vec<&str> = p.wps.iter().map(|w| w.ident.as_str()).collect();
        println!("{ids:?} {:?} fl {}", made, p.cruise_ft);
        assert!(p.sid.is_some() && p.app.is_some(), "{made:?}");
        assert_eq!(p.destination().unwrap().ident, "KLAX");
        // a real route is not a straight line but not a long detour either.
        let len = p.total(Some((db.apts[sfo as usize].lat, db.apts[sfo as usize].lon))) / 1852.0;
        assert!(len > 290.0 && len < 520.0, "{len}");
        // southeast: odd flight level.
        assert_eq!((p.cruise_ft / 1000.0) as i64 % 2, 1, "{}", p.cruise_ft);
        assert!(p.app.as_ref().unwrap().name.contains("24R"), "{:?}", p.app);
    }

    #[test]
    fn europe_by_vors() {
        let db = airports::Db::get();
        let (lhr, cdg) = (db.find("EGLL").unwrap(), db.find("LFPG").unwrap());
        let (p, _) = make(lhr, None, cdg, None, &crate::aircraft::b747::SPEC);
        let ids: Vec<&str> = p.wps.iter().map(|w| w.ident.as_str()).collect();
        println!("{ids:?}");
        // generated departure and approach, vors between.
        assert!(p.sid.is_some() && p.app.is_some());
        assert!(p.wps.iter().any(|w| w.kind == crate::flightplan::Kind::Vor), "{ids:?}");
        assert_eq!(p.destination().unwrap().ident, "LFPG");
    }

    #[test]
    fn runways_and_wind() {
        let db = airports::Db::get();
        let ends = runway_ends(db.find("KSFO").unwrap());
        assert!(ends.iter().any(|e| e.ident == "28L"));
        let w = into_wind(&ends, 290.0).unwrap();
        assert!(w.ident.starts_with("28"), "{w:?}");
        assert_eq!(cruise_for(&crate::aircraft::c172::SPEC, (37.6, -122.4), (38.5, -121.5)) % 1000.0, 500.0);
    }
}
