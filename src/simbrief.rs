//! simbrief: fetches the pilot's latest operational flight plan and turns it
//! into a route with every fix where simbrief puts it.
//! api: https://www.simbrief.com/api/xml.fetcher.php?username=NAME&json=1 (or userid=ID).
//! the json mirrors the ofp xml, a list of one comes back as a bare object.

use crate::flightplan::{Kind, Phase, Waypoint};
use serde_json::Value;

/// what the game takes from an ofp.
#[derive(Clone, Debug, Default)]
pub struct Ofp {
    pub dep: String,
    pub dep_rwy: Option<String>,
    pub arr: String,
    pub arr_rwy: Option<String>,
    pub altn: Option<String>,
    pub route: String,
    pub cruise_ft: f64,
    /// enroute fixes in order, departure and destination left out.
    pub fixes: Vec<Waypoint>,
    pub sid: Option<String>,
    pub star: Option<String>,
    /// block fuel and payload, kg.
    pub ramp_kg: Option<f64>,
    pub payload_kg: Option<f64>,
    pub ac_type: String,
    pub callsign: String,
    /// planned time enroute, s, and distance, nm.
    pub ete: Option<f64>,
    pub dist_nm: Option<f64>,
}

/// a field as text, numbers and strings alike.
fn text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn num(v: &Value) -> Option<f64> {
    text(v)?.parse().ok()
}

/// a list, or a single object as a list of one.
fn list(v: &Value) -> Vec<&Value> {
    match v {
        Value::Array(a) => a.iter().collect(),
        Value::Null => Vec::new(),
        o => vec![o],
    }
}

/// the request url for a username or a numeric pilot id.
pub fn url(who: &str) -> String {
    let who = who.trim();
    let key = if !who.is_empty() && who.chars().all(|c| c.is_ascii_digit()) { "userid" } else { "username" };
    let enc: String = who.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect();
    format!("https://www.simbrief.com/api/xml.fetcher.php?{key}={enc}&json=1")
}

/// the dispatch page with the route filled in, to plan a new one on simbrief.
pub fn dispatch_url(dep: &str, arr: &str, ty: &str) -> String {
    let mut u = format!("https://dispatch.simbrief.com/options/custom?orig={dep}&dest={arr}");
    if !ty.is_empty() {
        u.push_str(&format!("&type={ty}"));
    }
    u
}

/// parses an ofp, or simbrief's own reason it has none.
pub fn parse(body: &str) -> Result<Ofp, String> {
    let j: Value = serde_json::from_str(body).map_err(|e| format!("not a simbrief answer ({e})"))?;
    let status = text(&j["fetch"]["status"]).unwrap_or_default();
    if !status.eq_ignore_ascii_case("success") {
        return Err(if status.is_empty() { "no flight plan in the answer".into() } else { status });
    }
    let kg = match text(&j["params"]["units"]).unwrap_or_default().to_ascii_lowercase().as_str() {
        "lbs" | "lb" => 0.453_592,
        _ => 1.0,
    };
    let rwy = |v: &Value| text(v).map(|r| r.to_ascii_uppercase());
    let mut o = Ofp {
        dep: text(&j["origin"]["icao_code"]).ok_or("the plan has no departure")?.to_ascii_uppercase(),
        dep_rwy: rwy(&j["origin"]["plan_rwy"]),
        arr: text(&j["destination"]["icao_code"]).ok_or("the plan has no destination")?.to_ascii_uppercase(),
        arr_rwy: rwy(&j["destination"]["plan_rwy"]),
        altn: text(&j["alternate"]["icao_code"]),
        route: text(&j["general"]["route"]).unwrap_or_default(),
        cruise_ft: num(&j["general"]["initial_altitude"]).unwrap_or(0.0),
        ramp_kg: num(&j["fuel"]["plan_ramp"]).map(|f| f * kg),
        payload_kg: num(&j["weights"]["payload"]).map(|f| f * kg),
        ac_type: text(&j["aircraft"]["icaocode"]).or_else(|| text(&j["aircraft"]["icao_code"])).unwrap_or_default(),
        callsign: text(&j["atc"]["callsign"]).or_else(|| text(&j["general"]["icao_airline"]).zip(text(&j["general"]["flight_number"])).map(|(a, n)| format!("{a}{n}"))).unwrap_or_default(),
        ete: num(&j["times"]["est_time_enroute"]),
        dist_nm: num(&j["general"]["route_distance"]),
        ..Default::default()
    };
    let fixes = list(&j["navlog"]["fix"]);
    let n = fixes.len();
    for (i, f) in fixes.into_iter().enumerate() {
        let ident = text(&f["ident"]).unwrap_or_default().to_ascii_uppercase();
        let ty = text(&f["type"]).unwrap_or_default().to_ascii_lowercase();
        // top of climb and descent are not places to fly to.
        if ident.is_empty() || ident == "TOC" || ident == "TOD" {
            continue;
        }
        let (Some(lat), Some(lon)) = (num(&f["pos_lat"]), num(&f["pos_long"])) else { continue };
        // the destination is added by the planner from the airport database.
        if i + 1 == n && ident == o.arr {
            continue;
        }
        let kind = match ty.as_str() {
            "vor" => Kind::Vor,
            "ndb" => Kind::Ndb,
            "apt" => Kind::Airport,
            "ltlg" => Kind::User,
            _ => Kind::Fix,
        };
        let name = text(&f["name"]).filter(|n| !n.eq_ignore_ascii_case(&ident)).unwrap_or_else(|| crate::menu::latlon(lat, lon));
        let mut w = Waypoint::fix(&ident, lat, lon);
        w.kind = kind;
        w.name = name;
        let via = text(&f["via_airway"]).map(|v| v.to_ascii_uppercase());
        if text(&f["is_sid_star"]).as_deref() == Some("1") {
            let stage = text(&f["stage"]).unwrap_or_default();
            let phase = if stage.eq_ignore_ascii_case("CLB") { Phase::Sid } else { Phase::Star };
            let name = via.clone().unwrap_or_default();
            match phase {
                Phase::Sid => o.sid.get_or_insert(name.clone()),
                _ => o.star.get_or_insert(name.clone()),
            };
            w.phase = Some((phase, name));
        }
        w.via = via;
        o.fixes.push(w);
    }
    if o.fixes.is_empty() && o.route.is_empty() {
        return Err("the plan has no route".into());
    }
    Ok(o)
}

/// fetches the latest ofp, blocking: run it off the main thread.
pub fn fetch(who: &str) -> Result<Ofp, String> {
    if who.trim().is_empty() {
        return Err("enter your simbrief username or pilot id".into());
    }
    let agent: ureq::Agent = ureq::Agent::config_builder().http_status_as_error(false).timeout_global(Some(std::time::Duration::from_secs(20))).build().into();
    let mut res = agent.get(&url(who)).call().map_err(|e| format!("could not reach simbrief ({e})"))?;
    let body = res.body_mut().with_config().limit(16 << 20).read_to_string().map_err(|e| format!("could not read simbrief's answer ({e})"))?;
    parse(&body)
}

/// opens a web page in the system browser.
pub fn open_in_browser(url: &str) {
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let r = std::process::Command::new("xdg-open").arg(url).spawn();
    if let Err(e) = r {
        log::warn!("could not open {url}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OFP: &str = r#"{
      "fetch": {"userid": "1", "static_id": {}, "status": "Success"},
      "params": {"units": "lbs"},
      "general": {"route": "SSTIK5 SSTIK DCT ENI J3 FMG DCT INYOE DYAMD5", "initial_altitude": "36000", "route_distance": "1850", "icao_airline": "UAL", "flight_number": "123"},
      "origin": {"icao_code": "KSFO", "plan_rwy": "28R"},
      "destination": {"icao_code": "KDEN", "plan_rwy": "34L"},
      "alternate": {"icao_code": "KCOS"},
      "aircraft": {"icaocode": "B738"},
      "fuel": {"plan_ramp": "20000"},
      "weights": {"payload": "30000"},
      "times": {"est_time_enroute": "8000"},
      "navlog": {"fix": [
        {"ident": "SSTIK", "name": "SSTIK", "type": "wpt", "pos_lat": "37.9", "pos_long": "-122.9", "via_airway": "SSTIK5", "is_sid_star": "1", "stage": "CLB"},
        {"ident": "TOC", "type": "ltlg", "pos_lat": "38.0", "pos_long": "-122.0", "via_airway": "DCT", "is_sid_star": "0", "stage": "CLB"},
        {"ident": "ENI", "name": "UKIAH", "type": "vor", "pos_lat": "39.05", "pos_long": "-123.27", "via_airway": "DCT", "is_sid_star": "0", "stage": "CRZ"},
        {"ident": "FMG", "name": "MUSTANG", "type": "vor", "pos_lat": "39.53", "pos_long": "-119.65", "via_airway": "J3", "is_sid_star": "0", "stage": "CRZ"},
        {"ident": "DYAMD", "name": "DYAMD", "type": "wpt", "pos_lat": "40.0", "pos_long": "-105.5", "via_airway": "DYAMD5", "is_sid_star": "1", "stage": "DSC"},
        {"ident": "KDEN", "name": "DENVER", "type": "apt", "pos_lat": "39.86", "pos_long": "-104.67", "via_airway": "DCT", "is_sid_star": "0", "stage": "DSC"}
      ]}
    }"#;

    #[test]
    fn reads_an_ofp() {
        let o = parse(OFP).unwrap();
        assert_eq!((o.dep.as_str(), o.dep_rwy.as_deref(), o.arr.as_str(), o.arr_rwy.as_deref()), ("KSFO", Some("28R"), "KDEN", Some("34L")));
        assert_eq!(o.cruise_ft, 36000.0);
        let ids: Vec<&str> = o.fixes.iter().map(|w| w.ident.as_str()).collect();
        assert_eq!(ids, vec!["SSTIK", "ENI", "FMG", "DYAMD"]);
        assert_eq!(o.fixes[1].kind, Kind::Vor);
        assert_eq!(o.fixes[2].via.as_deref(), Some("J3"));
        assert_eq!(o.fixes[0].phase.as_ref().map(|p| p.0), Some(Phase::Sid));
        assert_eq!(o.sid.as_deref(), Some("SSTIK5"));
        assert_eq!(o.star.as_deref(), Some("DYAMD5"));
        assert!((o.ramp_kg.unwrap() - 9071.8).abs() < 1.0);
        assert_eq!(o.callsign, "UAL123");
    }

    #[test]
    fn errors_and_single_fixes() {
        let e = parse(r#"{"fetch": {"status": "Error: Unknown UserID"}}"#).unwrap_err();
        assert!(e.contains("Unknown"), "{e}");
        let one = r#"{"fetch": {"status": "Success"}, "origin": {"icao_code": "EGLL"}, "destination": {"icao_code": "LFPG"},
          "navlog": {"fix": {"ident": "BIG", "type": "vor", "pos_lat": 51.33, "pos_long": 0.03, "via_airway": "DCT"}}}"#;
        let o = parse(one).unwrap();
        assert_eq!(o.fixes.len(), 1);
        assert!(url("123456").contains("userid=123456"));
        assert!(url("jane doe").contains("username=jane%20doe"));
    }
}
