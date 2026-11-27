//! the season on the vegetation: how far the broadleaf trees have turned
//! (fresh in spring, coloured in autumn) and how dormant the plants are, from
//! the latitude and the day of the year. the tropics hardly change; the
//! southern hemisphere runs half a year behind. passed to the shaders in
//! globals.alto.zw (common.wgsl `season_leaf`, `season_ground`).

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// (turn, dormant): turn -1 fresh spring green .. 0 summer .. 1 full autumn
/// colour; dormant 0 growing .. 1 bare and brown. `lat` in degrees.
pub fn at(lat: f64, doy: f64) -> (f32, f32) {
    // how strong the seasons are: none near the equator, full by mid latitudes.
    let k = smooth(20.0, 38.0, lat.abs());
    if k <= 0.0 {
        return (0.0, 0.0);
    }
    // the north's calendar; the south's is half a year on.
    let d = if lat < 0.0 { (doy + 182.5).rem_euclid(365.0) } else { doy.rem_euclid(365.0) };
    // further north the autumn comes earlier and the spring later (days per degree past 45).
    let shift = (lat.abs() - 45.0).clamp(-15.0, 20.0) * 1.2;
    let (spring, autumn) = (115.0 + shift, 288.0 - shift);
    // spring: dormancy lifts over a month, then a few weeks of fresh green.
    let wake = smooth(spring - 30.0, spring + 5.0, d);
    let fresh = smooth(spring - 15.0, spring, d) * (1.0 - smooth(spring + 10.0, spring + 45.0, d));
    // autumn: colour builds to its peak, then the leaves fall and it all goes dormant.
    let colour = smooth(autumn - 35.0, autumn, d) * (1.0 - smooth(autumn + 18.0, autumn + 40.0, d) * 0.6);
    let sleep = smooth(autumn + 12.0, autumn + 42.0, d);
    let dormant = if d < 200.0 { 1.0 - wake } else { sleep };
    let turn = if d < 200.0 { -fresh } else { colour * (1.0 - sleep * 0.7) };
    ((turn * k) as f32, (dormant * k) as f32)
}

#[cfg(test)]
mod tests {
    use super::at;

    #[test]
    fn temperate_seasons_and_quiet_tropics() {
        // mid october in paris: autumn colour; mid january: dormant; july: green.
        let (t, d) = at(48.9, 288.0);
        assert!(t > 0.7 && d < 0.3, "{t} {d}");
        let (_, d) = at(48.9, 15.0);
        assert!(d > 0.95, "{d}");
        assert_eq!(at(48.9, 195.0), (0.0, 0.0));
        // spring flush in may.
        assert!(at(48.9, 120.0).0 < -0.5);
        // sydney in july is winter (a milder one at its latitude), singapore never changes.
        assert!(at(-33.9, 200.0).1 > 0.8);
        assert_eq!(at(1.3, 288.0), (0.0, 0.0));
    }
}
