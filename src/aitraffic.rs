//! game-managed air traffic: airliners, regional jets and light aircraft
//! flying between real airports around the player, alongside the real
//! traffic (live.rs draws, lights and labels both). each climbs out to its
//! level, cruises and comes down toward its destination on a great circle; a
//! dozen or so are kept in the sky within about 80 nm, new ones appearing
//! part way along their routes. their airlines are made up.

use crate::airports::Db;
use crate::geo;
use crate::live::{AiReport, Live};

/// made-up airlines (callsign prefixes), and the types each size flies.
const AIRLINES: [&str; 8] = ["FBX", "QZA", "VRN", "NMB", "OKT", "ZLA", "TRW", "SKF"];
const TYPES: [(&str, u8, f64, f64); 5] = [("C172", 0, 110.0, 5_500.0), ("PC12", 1, 260.0, 21_000.0), ("E175", 2, 430.0, 33_000.0), ("A320", 3, 450.0, 36_000.0), ("B77W", 4, 490.0, 38_000.0)];

struct Flight {
    id: u32,
    call: String,
    ty: usize,
    dest: (f64, f64, f64),
    lat: f64,
    lon: f64,
    alt: f64,
    trk: f64,
    gs: f64,
    vs: f64,
}

pub struct AiTraffic {
    pub on: bool,
    flights: Vec<Flight>,
    next: u32,
    rng: u64,
    t_spawn: f64,
    t_report: f64,
}

impl Default for AiTraffic {
    fn default() -> Self {
        Self { on: true, flights: Vec::new(), next: 1, rng: 0x9E37_79B9_7F4A_7C15, t_spawn: 0.0, t_report: 0.0 }
    }
}

fn nm(a: (f64, f64), b: (f64, f64)) -> f64 {
    crate::flightplan::dist(a, b) / 1852.0
}

/// a point `d` nm along a course (deg) from (lat, lon).
fn ahead(lat: f64, lon: f64, d: f64, crs: f64) -> (f64, f64) {
    let r = d * 1852.0 / geo::R;
    let (p1, l1, b) = (lat.to_radians(), lon.to_radians(), crs.to_radians());
    let p2 = (p1.sin() * r.cos() + p1.cos() * r.sin() * b.cos()).asin();
    let l2 = l1 + (b.sin() * r.sin() * p1.cos()).atan2(r.cos() - p1.sin() * p2.sin());
    (p2.to_degrees(), ((l2.to_degrees() + 540.0) % 360.0) - 180.0)
}

impl AiTraffic {
    fn f(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 11) as f64 / (1u64 << 53) as f64
    }

    /// a new flight on a route through the airspace round the player, part way along.
    fn spawn(&mut self, at: (f64, f64)) {
        let db = Db::get();
        let near = db.near(at.0, at.1, 160.0 * 1852.0);
        if near.is_empty() {
            return;
        }
        let ty = match self.f() {
            x if x < 0.22 => 0,
            x if x < 0.32 => 1,
            x if x < 0.55 => 2,
            x if x < 0.90 => 3,
            _ => 4,
        };
        let big = ty >= 2;
        let fits = |i: u32| {
            let a = &db.apts[i as usize];
            if big { a.kind >= 1 && a.sched } else { a.kind <= 1 && !a.rwys.is_empty() }
        };
        let deps: Vec<u32> = near.iter().map(|x| x.0).filter(|&i| fits(i)).collect();
        if deps.is_empty() {
            return;
        }
        for _ in 0..12 {
            let d = deps[(self.f() * deps.len() as f64) as usize % deps.len()];
            let a = &db.apts[d as usize];
            // the other end: in range of the type, anywhere round.
            let reach = if big { (150.0, 900.0) } else { (20.0, 140.0) };
            let crs = self.f() * 360.0;
            let len = reach.0 + self.f() * (reach.1 - reach.0);
            let (tl, to) = ahead(a.lat, a.lon, len, crs);
            let dest = db.near(tl, to, 120.0 * 1852.0).into_iter().filter(|x| fits(x.0) && x.0 != d).min_by(|x, y| x.1.total_cmp(&y.1)).map(|x| &db.apts[x.0 as usize]);
            let Some(b) = dest else { continue };
            // somewhere along the way that is near the player.
            let total = nm((a.lat, a.lon), (b.lat, b.lon));
            let crs = crate::flightplan::course((a.lat, a.lon), (b.lat, b.lon));
            let k = 0.1 + self.f() * 0.8;
            let (lat, lon) = ahead(a.lat, a.lon, total * k, crs);
            if nm((lat, lon), at) > 80.0 || nm((lat, lon), at) < 6.0 {
                continue;
            }
            let (_, _, kt, cruise_ft) = TYPES[ty];
            // in the climb near its start, the descent near its end, else level.
            let along = total * k;
            let alt_ft = (a.elev as f64 + along * 320.0).min(cruise_ft).min(b.elev as f64 + (total - along) * 330.0).max(3000.0);
            let call = if ty == 0 { format!("N{}{}", 100 + (self.f() * 899.0) as u32, ['A', 'B', 'C', 'D', 'F', 'K', 'S'][(self.f() * 7.0) as usize % 7]) } else { format!("{}{}", AIRLINES[(self.f() * AIRLINES.len() as f64) as usize % AIRLINES.len()], 10 + (self.f() * 980.0) as u32) };
            self.flights.push(Flight { id: self.next, call, ty, dest: (b.lat, b.lon, b.elev as f64 * 0.3048), lat, lon, alt: alt_ft * 0.3048, trk: crs, gs: kt * 0.514_44, vs: 0.0 });
            self.next += 1;
            return;
        }
    }

    /// once a step: fly each one on, drop the ones that have arrived or gone, hand them to the traffic layer.
    pub fn step(&mut self, dt: f64, at: (f64, f64), live: &mut Live) {
        if !self.on {
            if !self.flights.is_empty() {
                self.flights.clear();
            }
            return;
        }
        self.t_spawn -= dt;
        if self.t_spawn <= 0.0 && self.flights.len() < 14 {
            self.spawn(at);
            self.t_spawn = if self.flights.len() < 6 { 0.5 } else { 6.0 };
        }
        for f in &mut self.flights {
            let (_, _, kt, cruise_ft) = TYPES[f.ty];
            let to_go = nm((f.lat, f.lon), (f.dest.0, f.dest.1));
            // the descent starts 3 nm a thousand feet out.
            let want = if to_go < (f.alt - f.dest.2) * 3.28084 / 1000.0 * 3.0 { f.dest.2 + 900.0 } else { cruise_ft * 0.3048 };
            f.vs = ((want - f.alt) * 0.02).clamp(-12.0, if f.ty == 0 { 3.5 } else { 11.0 });
            f.alt += f.vs * dt;
            let low = f.alt < 3_050.0;
            let v = if low { (kt * 0.514_44).min(128.0) } else { kt * 0.514_44 };
            f.gs += (v - f.gs) * (dt * 0.2).min(1.0);
            let crs = crate::flightplan::course((f.lat, f.lon), (f.dest.0, f.dest.1));
            let turn = ((crs - f.trk + 540.0) % 360.0 - 180.0).clamp(-3.0 * dt, 3.0 * dt);
            f.trk = (f.trk + turn).rem_euclid(360.0);
            (f.lat, f.lon) = ahead(f.lat, f.lon, f.gs * dt / 1852.0, f.trk);
        }
        self.flights.retain(|f| {
            let gone = nm((f.lat, f.lon), at) > 120.0;
            let landed = nm((f.lat, f.lon), (f.dest.0, f.dest.1)) < 8.0 && f.alt < f.dest.2 + 1_300.0;
            !gone && !landed
        });
        self.t_report -= dt;
        if self.t_report <= 0.0 {
            self.t_report = 1.0;
            let reports: Vec<AiReport> = self.flights.iter().map(|f| {
                let (ty, size, _, _) = TYPES[f.ty];
                AiReport { hex: format!("~ai{}", f.id), call: f.call.clone(), ty: ty.into(), size, lat: f.lat, lon: f.lon, alt_m: f.alt, gs: f.gs, trk: f.trk, vs: f.vs }
            }).collect();
            live.take_ai(&reports);
        }
    }

    pub fn count(&self) -> usize {
        self.flights.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traffic_appears_round_a_busy_airport() {
        let mut ai = AiTraffic::default();
        let mut live = Live::new(false);
        let at = (37.62, -122.38);
        for _ in 0..400 {
            ai.step(0.1, at, &mut live);
        }
        assert!(ai.count() >= 4, "{}", ai.count());
        assert!(live.planes.iter().any(|p| p.hex.starts_with("~ai")));
    }
}
