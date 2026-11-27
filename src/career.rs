//! the career: a student pilot with a little money at a home airport works
//! up to airline captain and company owner. the loop: earn a licence with
//! lessons and a checkride, take jobs from the board at the airport you are
//! at, fly them, and get a debrief that pays by how it went; reputation in
//! each kind of work opens better paid jobs, money buys aircraft and hires
//! pilots who earn while you are away. the screens are in career_ui.rs.
//! saved as career.json beside the settings. the economy is a game's.

use crate::airports::{Apt, Db};
use crate::flightlog::Landing;
use crate::{aircraft, geo, profile, tr_en, Game};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// a licence or a rating.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum Lic {
    Ppl,
    Ir,
    Cpl,
    Atpl,
    BizJet,
    Airliner,
    Display,
    Night,
    Multi,
    Aero,
}

/// in the order the training page shows them.
pub const LICS: [Lic; 10] = [Lic::Ppl, Lic::Night, Lic::Ir, Lic::Multi, Lic::Cpl, Lic::Aero, Lic::BizJet, Lic::Atpl, Lic::Airliner, Lic::Display];

/// what a licence needs.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Req {
    /// one of the career's own lessons flown (missions.rs, `Cat::Lesson`).
    Lesson(&'static str),
    /// a checkride passed (missions.rs, `Cat::Exam`).
    Exam(&'static str),
    Holds(Lic),
    /// hours in the logbook.
    Hours(f64),
    Jobs(u32),
    Level(u32),
    Rep(f64),
    /// paid when the licence is taken.
    Fee(f64),
}

impl Lic {
    pub fn name(self) -> &'static str {
        match self {
            Lic::Ppl => "Private pilot",
            Lic::Ir => "Instrument rating",
            Lic::Cpl => "Commercial pilot",
            Lic::Atpl => "Airline transport pilot",
            Lic::BizJet => "Business jet type rating",
            Lic::Airliner => "Boeing 747 type rating",
            Lic::Display => "Air display team",
            Lic::Night => "Night rating",
            Lic::Multi => "Multi-engine rating",
            Lic::Aero => "Aerobatic rating",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Lic::Ppl => "PPL",
            Lic::Ir => "IR",
            Lic::Cpl => "CPL",
            Lic::Atpl => "ATPL",
            Lic::BizJet => "G650",
            Lic::Airliner => "B747",
            Lic::Display => "DISP",
            Lic::Night => "NGT",
            Lic::Multi => "ME",
            Lic::Aero => "AERO",
        }
    }

    /// what it opens, in a line.
    pub fn opens(self) -> &'static str {
        match self {
            Lic::Ppl => "Light aircraft: ferry flights, sightseeing tours and aerial photography.",
            Lic::Ir => "Flying in cloud, and air ambulance work.",
            Lic::Cpl => "Paid passengers and cargo, and skydiving drops.",
            Lic::Atpl => "Command of airliners: the way to the type ratings.",
            Lic::BizJet => "The Gulfstream G650: executive charters, long range cargo and medical flights.",
            Lic::Airliner => "The Boeing 747: airline passengers and freight.",
            Lic::Display => "Fighters at air shows.",
            Lic::Night => "Flying after dark. Needed for the commercial licence.",
            Lic::Multi => "More than one engine. Needed for jets and airliners.",
            Lic::Aero => "Loops and rolls. Needed for the air display team.",
        }
    }

    pub fn reqs(self) -> &'static [Req] {
        use Req::*;
        match self {
            Lic::Ppl => &[Lesson("c_ppl1"), Lesson("c_ppl2"), Lesson("c_ppl3"), Exam("e_ppl")],
            Lic::Ir => &[Holds(Lic::Ppl), Lesson("c_ir1"), Lesson("c_ir2"), Hours(5.0), Exam("e_ir")],
            Lic::Cpl => &[Holds(Lic::Ppl), Holds(Lic::Night), Hours(15.0), Jobs(6), Level(5), Exam("e_cpl")],
            Lic::Atpl => &[Holds(Lic::Cpl), Holds(Lic::Ir), Holds(Lic::Multi), Hours(60.0), Level(15), Rep(1500.0), Lesson("c_atpl")],
            Lic::BizJet => &[Holds(Lic::Cpl), Holds(Lic::Ir), Holds(Lic::Multi), Lesson("c_bizjet"), Fee(150_000.0)],
            Lic::Airliner => &[Holds(Lic::Atpl), Lesson("c_747"), Fee(500_000.0)],
            Lic::Display => &[Holds(Lic::Aero), Level(12), Rep(800.0), Fee(250_000.0)],
            Lic::Night => &[Holds(Lic::Ppl), Hours(3.0), Lesson("c_night"), Fee(1_500.0)],
            Lic::Multi => &[Holds(Lic::Ppl), Hours(10.0), Level(4), Fee(12_000.0)],
            Lic::Aero => &[Holds(Lic::Cpl), Level(8), Lesson("c_aero"), Fee(6_000.0)],
        }
    }

    pub fn fee(self) -> f64 {
        self.reqs().iter().map(|r| if let Req::Fee(f) = r { *f } else { 0.0 }).sum()
    }
}

/// what kind of aircraft a type is, as the career goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Class {
    Light,
    BizJet,
    Airliner,
    Fighter,
    None,
}

pub const CLASSES: [Class; 4] = [Class::Light, Class::BizJet, Class::Airliner, Class::Fighter];

pub fn class_of(spec: &aircraft::Spec) -> Class {
    let key = spec.keys.first().copied().unwrap_or("");
    let body = spec.origin.map(|o| o.body).unwrap_or(key);
    // (rockets, the blackbird and any rotorcraft or floatplane stay out of it)
    if spec.rocket().is_some() || spec.floats || spec.rotor.is_some() {
        return Class::None;
    }
    match key {
        "c172" | "dr400" | "bonanza" => return Class::Light,
        _ => {}
    }
    match spec.id {
        0 | 8 | 10 => Class::Light,
        9 => Class::BizJet,
        1 => Class::Airliner,
        // (concorde and the blackbird are not career aircraft)
        6 => Class::None,
        2..=4 => Class::Fighter,
        7 => Class::None,
        _ if body.contains("747") => Class::Airliner,
        _ if spec.is_jet() && spec.mass > 100_000.0 => Class::Airliner,
        _ if spec.is_jet() => Class::BizJet,
        _ => Class::Light,
    }
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::Light => "Light aircraft",
            Class::BizJet => "Business jet",
            Class::Airliner => "Airliner",
            Class::Fighter => "Fighter",
            Class::None => "Not in the career",
        }
    }

    pub fn licence(self) -> Option<Lic> {
        match self {
            Class::Light => Some(Lic::Ppl),
            Class::BizJet => Some(Lic::BizJet),
            Class::Airliner => Some(Lic::Airliner),
            Class::Fighter => Some(Lic::Display),
            Class::None => None,
        }
    }

    /// rental by the hour ($).
    pub fn rent(self) -> f64 {
        match self {
            Class::Light => 160.0,
            Class::BizJet => 3_200.0,
            Class::Airliner => 16_000.0,
            Class::Fighter => 9_000.0,
            Class::None => 0.0,
        }
    }

    pub fn fuel_price(self) -> f64 {
        if self == Class::Light { 2.1 } else { 0.95 }
    }

    pub fn cruise_kt(self) -> f64 {
        match self {
            Class::Light => 110.0,
            Class::Fighter => 420.0,
            _ => 460.0,
        }
    }

    /// how far its jobs go (nm), the shortest runway it takes (ft), paved only.
    fn reach(self) -> (f64, f64, f64, bool) {
        match self {
            Class::Light => (20.0, 220.0, 1_800.0, false),
            Class::BizJet => (250.0, 2_600.0, 5_000.0, true),
            Class::Airliner => (500.0, 4_500.0, 8_500.0, true),
            Class::Fighter => (40.0, 400.0, 7_000.0, true),
            Class::None => (0.0, 0.0, 0.0, true),
        }
    }

    /// the pay a nautical mile, and the start of each job.
    fn rates(self) -> (f64, f64) {
        match self {
            Class::Light => (6.0, 180.0),
            Class::BizJet => (24.0, 4_000.0),
            Class::Airliner => (95.0, 25_000.0),
            Class::Fighter => (60.0, 12_000.0),
            Class::None => (0.0, 0.0),
        }
    }

    /// what a hired pilot earns the company in an hour with one (before wages).
    pub fn hourly(self) -> f64 {
        self.rent() * 0.55
    }
}

/// the buying price of a type (none: not for sale).
pub fn price_of(spec: &aircraft::Spec) -> Option<f64> {
    let key = spec.keys.first().copied().unwrap_or("");
    Some(match key {
        "c172" => 95_000.0,
        "dr400" => 80_000.0,
        "bonanza" => 380_000.0,
        _ => match class_of(spec) {
            Class::Light => 120_000.0,
            Class::BizJet => 9_000_000.0,
            Class::Airliner => 30_000_000.0,
            _ => return None,
        },
    })
}

/// the types a career can fly: the built in ones it has a class for.
pub fn career_types() -> Vec<&'static aircraft::Spec> {
    aircraft::list().into_iter().filter(|s| s.origin.is_none() && class_of(s) != Class::None).collect()
}

/// the kinds of work.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum Kind {
    Ferry,
    Tour,
    Photo,
    Skydive,
    Passengers,
    Cargo,
    Medical,
    Charter,
    Airline,
    Display,
}

pub const KINDS: [Kind; 10] = [Kind::Ferry, Kind::Tour, Kind::Photo, Kind::Skydive, Kind::Passengers, Kind::Cargo, Kind::Medical, Kind::Charter, Kind::Airline, Kind::Display];

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Ferry => "Ferry flight",
            Kind::Tour => "Sightseeing tour",
            Kind::Photo => "Aerial photography",
            Kind::Skydive => "Skydiving drop",
            Kind::Passengers => "Passengers",
            Kind::Cargo => "Cargo",
            Kind::Medical => "Air ambulance",
            Kind::Charter => "Executive charter",
            Kind::Airline => "Airline service",
            Kind::Display => "Air display",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Kind::Ferry => "Take an empty aircraft to where its owner wants it.",
            Kind::Tour => "Show passengers a landmark from the air and bring them home.",
            Kind::Photo => "Fly low over a site for the camera and come back.",
            Kind::Skydive => "Climb over the field with jumpers aboard, let them go and land.",
            Kind::Passengers => "Fly paying passengers to another airport, smoothly.",
            Kind::Cargo => "Carry freight to another airport on time.",
            Kind::Medical => "A patient on board: get there quickly and gently.",
            Kind::Charter => "Business travellers who pay for comfort and punctuality.",
            Kind::Airline => "A scheduled service with a full cabin.",
            Kind::Display => "A low pass along the crowd line at an air show.",
        }
    }

    /// the licence the work needs, as well as the rating for the aircraft.
    pub fn lic(self) -> Lic {
        match self {
            Kind::Ferry | Kind::Tour | Kind::Photo => Lic::Ppl,
            Kind::Skydive | Kind::Passengers | Kind::Cargo => Lic::Cpl,
            Kind::Medical => Lic::Ir,
            Kind::Charter => Lic::BizJet,
            Kind::Airline => Lic::Airliner,
            Kind::Display => Lic::Display,
        }
    }

    pub fn classes(self) -> &'static [Class] {
        match self {
            Kind::Ferry => &[Class::Light, Class::BizJet],
            Kind::Tour => &[Class::Light],
            Kind::Photo => &[Class::Light],
            Kind::Skydive => &[Class::Light],
            Kind::Passengers => &[Class::Light],
            Kind::Cargo => &[Class::Light, Class::BizJet, Class::Airliner],
            Kind::Medical => &[Class::Light, Class::BizJet],
            Kind::Charter => &[Class::BizJet],
            Kind::Airline => &[Class::Airliner],
            Kind::Display => &[Class::Fighter],
        }
    }

    /// the colour of its tag.
    pub fn colour(self) -> egui::Color32 {
        use egui::Color32 as C;
        match self {
            Kind::Ferry => C::from_rgb(150, 164, 184),
            Kind::Tour => C::from_rgb(86, 204, 186),
            Kind::Photo => C::from_rgb(178, 140, 255),
            Kind::Skydive => C::from_rgb(255, 150, 70),
            Kind::Passengers => C::from_rgb(110, 170, 255),
            Kind::Cargo => C::from_rgb(214, 170, 96),
            Kind::Medical => C::from_rgb(255, 100, 110),
            Kind::Charter => C::from_rgb(245, 197, 24),
            Kind::Airline => C::from_rgb(90, 150, 255),
            Kind::Display => C::from_rgb(255, 120, 200),
        }
    }

    /// passengers aboard care how it feels.
    pub fn carries_people(self) -> bool {
        matches!(self, Kind::Tour | Kind::Passengers | Kind::Medical | Kind::Charter | Kind::Airline | Kind::Skydive)
    }

    /// comes back to where it started.
    pub fn round_trip(self) -> bool {
        matches!(self, Kind::Tour | Kind::Photo | Kind::Skydive | Kind::Display)
    }
}

/// reputation tiers in each kind of work.
pub const TIERS: [(f64, &str); 5] = [(0.0, "Newcomer"), (150.0, "Known"), (450.0, "Trusted"), (1000.0, "Renowned"), (2000.0, "Elite")];

/// the tier for a reputation, and how far to the next (0..1).
pub fn tier(rep: f64) -> (usize, f32) {
    let i = TIERS.iter().rposition(|t| rep >= t.0).unwrap_or(0);
    let frac = match TIERS.get(i + 1) {
        Some(n) => ((rep - TIERS[i].0) / (n.0 - TIERS[i].0)) as f32,
        None => 1.0,
    };
    (i, frac.clamp(0.0, 1.0))
}

/// a job on the board.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub id: u64,
    pub kind: Kind,
    pub from: String,
    pub to: String,
    /// a point to pass over on the way (tours, photos, drops, rescues): lat, lon.
    pub via: Option<(f64, f64)>,
    pub dist_nm: f64,
    pub payload_kg: f64,
    pub pax: u32,
    pub pay: f64,
    /// game seconds it should take at most.
    pub limit_s: f64,
    /// the aircraft class it is for.
    pub class: String,
    /// 1 to 3: longer, harder and better paid.
    #[serde(default = "one")]
    pub tier: u8,
    #[serde(default)]
    pub client: String,
    /// what the point on the way is.
    #[serde(default)]
    pub via_name: String,
}

fn one() -> u8 {
    1
}

/// a sale or a cost, for the statement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub unix: u64,
    pub what: String,
    pub amount: f64,
}

/// a pilot working for the company.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Hire {
    pub name: String,
    /// 1 to 5.
    pub skill: u8,
    /// the owned aircraft they fly (its key).
    pub plane: Option<String>,
    pub hired: u64,
    /// what they have earned the company, all told.
    #[serde(default)]
    pub earned: f64,
}

/// how a job came out.
#[derive(Clone, Debug)]
pub struct Debrief {
    pub ok: bool,
    pub stars: u8,
    /// what was marked: the name, out of 100, a word on it.
    pub marks: Vec<(String, u32, String)>,
    pub lines: Vec<(String, f64)>,
    pub rep: f64,
    pub xp: u64,
    pub note: String,
    pub kind: Kind,
    pub title: String,
}

impl Debrief {
    pub fn net(&self) -> f64 {
        self.lines.iter().map(|l| l.1).sum()
    }
}

/// what the career screen is showing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tab {
    #[default]
    Home,
    Jobs,
    Training,
    Fleet,
    Company,
    Finances,
}

/// the screen's own state, not saved.
#[derive(Clone, Debug, Default)]
pub struct View {
    pub tab: Tab,
    /// the job picked on the board, and the one being briefed.
    pub sel: Option<u64>,
    pub brief: Option<Job>,
    /// the aircraft picked in the briefing (its key).
    pub brief_plane: Option<String>,
    pub filter: Option<Kind>,
    /// a word after an action.
    pub note: Option<(String, bool)>,
    /// the debrief of the last job, shown on return.
    pub debrief: Option<Debrief>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Career {
    pub started: bool,
    pub name: String,
    pub base: String,
    pub money: f64,
    /// all the reputation; by kind of work in `rep_by`.
    pub rep: f64,
    pub hours: f64,
    pub jobs_done: u32,
    pub lics: Vec<Lic>,
    /// aircraft owned, by their key.
    pub owned: Vec<String>,
    pub board: Vec<Job>,
    /// what the board was made for: the airport and how many times it was refreshed.
    pub board_for: String,
    pub refresh: u32,
    pub active: Option<Job>,
    pub ledger: Vec<Entry>,
    pub next_id: u64,
    #[serde(default)]
    pub rep_by: BTreeMap<Kind, f64>,
    /// wear on owned aircraft by key, 0 new to 1 worn out.
    #[serde(default)]
    pub wear: BTreeMap<String, f64>,
    #[serde(default)]
    pub hires: Vec<Hire>,
    /// unix seconds the company's earnings were last counted.
    #[serde(default)]
    pub accrued: u64,
    /// the stars of the jobs flown, newest last.
    #[serde(default)]
    pub stars: Vec<u8>,
    /// lessons and checkrides flown from the career (activities flown outside it do not count).
    #[serde(default)]
    pub trained: std::collections::BTreeSet<String>,
    /// landing challenge scores flown from the career, the best of each.
    #[serde(default)]
    pub challenges: BTreeMap<String, u32>,
    #[serde(skip)]
    pub view: View,
}

impl Default for Career {
    fn default() -> Self {
        Self { started: false, name: String::new(), base: "KHAF".into(), money: 15_000.0, rep: 0.0, hours: 0.0, jobs_done: 0, lics: Vec::new(), owned: Vec::new(), board: Vec::new(), board_for: String::new(), refresh: 0, active: None, ledger: Vec::new(), next_id: 1, rep_by: BTreeMap::new(), wear: BTreeMap::new(), hires: Vec::new(), accrued: 0, stars: Vec::new(), trained: Default::default(), challenges: BTreeMap::new(), view: View::default() }
    }
}

fn path() -> Option<PathBuf> {
    // scripted runs (shots, recordings, benchmarks) read the career but never write it.
    static SCRIPTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let scripted = *SCRIPTED.get_or_init(|| std::env::args().any(|a| matches!(a.as_str(), "--headless" | "--shot" | "--record" | "--bench")));
    dirs::config_dir().map(|d| d.join("fablegame").join("career.json")).filter(|_| !scripted)
}

/// licences, kinds of work and aircraft from older versions that are gone
/// (the helicopter, the glider, the floatplane and the blackbird's rating),
/// taken out of a saved career so the rest of it still loads.
fn prune(mut v: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    const LIC_GONE: [&str; 4] = ["Heli", "Glider", "Recon", "Sst"];
    const KIND_GONE: [&str; 3] = ["Rescue", "Soaring", "Recon"];
    const TYPE_GONE: [&str; 4] = ["r44", "ask21", "c172f", "concorde"];
    let kind_ok = |j: &Value| !j.get("kind").and_then(Value::as_str).is_some_and(|k| KIND_GONE.contains(&k));
    let Some(o) = v.as_object_mut() else { return v };
    if let Some(Value::Array(a)) = o.get_mut("lics") {
        a.retain(|l| !l.as_str().is_some_and(|s| LIC_GONE.contains(&s)));
    }
    if let Some(Value::Object(m)) = o.get_mut("rep_by") {
        m.retain(|k, _| !KIND_GONE.contains(&k.as_str()));
    }
    if let Some(Value::Array(a)) = o.get_mut("board") {
        a.retain(kind_ok);
    }
    if o.get("active").is_some_and(|j| !kind_ok(j)) {
        o.insert("active".into(), Value::Null);
    }
    if let Some(Value::Array(a)) = o.get_mut("owned") {
        a.retain(|k| !k.as_str().is_some_and(|s| TYPE_GONE.contains(&s)));
    }
    if let Some(Value::Array(a)) = o.get_mut("hires") {
        for h in a.iter_mut() {
            if h.get("plane").and_then(Value::as_str).is_some_and(|s| TYPE_GONE.contains(&s)) {
                h["plane"] = Value::Null;
            }
        }
    }
    v
}

fn load_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("fablegame").join("career.json"))
}

/// $12,345 (a minus in front when it is owed).
pub fn money(v: f64) -> String {
    let n = v.abs().round() as u64;
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{}${out}", if v < 0.0 { "-" } else { "" })
}

/// $1.2M, $45k: short, for chips.
pub fn money_short(v: f64) -> String {
    let a = v.abs();
    let sign = if v < 0.0 { "-" } else { "" };
    if a >= 1e6 {
        format!("{sign}${:.1}M", a / 1e6)
    } else if a >= 1e4 {
        format!("{sign}${:.0}k", a / 1e3)
    } else {
        money(v)
    }
}

struct Rng(u64);
impl Rng {
    fn f(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    fn pick<T: Copy>(&mut self, v: &[T]) -> T {
        v[(self.f() * v.len() as f64) as usize % v.len()]
    }
}

fn longest_ft(a: &Apt) -> (f64, bool) {
    a.rwys.iter().map(|r| (r.len as f64, r.paved)).fold((0.0, false), |m, r| if r.0 > m.0 { r } else { m })
}

/// a point `nm` out from (lat, lon) on a bearing (deg).
fn offset(lat: f64, lon: f64, nm: f64, bearing: f64) -> (f64, f64) {
    let d = nm * 1852.0 / geo::R;
    let (p1, l1, b) = (lat.to_radians(), lon.to_radians(), bearing.to_radians());
    let p2 = (p1.sin() * d.cos() + p1.cos() * d.sin() * b.cos()).asin();
    let l2 = l1 + (b.sin() * d.sin() * p1.cos()).atan2(d.cos() - p1.sin() * p2.sin());
    (p2.to_degrees(), l2.to_degrees())
}

pub fn nm_between(a: (f64, f64), b: (f64, f64)) -> f64 {
    crate::flightplan::dist(a, b) / 1852.0
}

const CLIENTS: &[&str] = &["Coastal Air Charter", "Bluebird Aviation", "Summit Medical", "Skyline Tours", "Northwind Freight", "Harbour Air Taxi", "Valley Skydivers", "Meridian Survey", "Silverline Executive", "Atlas Cargo", "Horizon Airways", "Aurora Air Shows", "Crown Logistics", "Pioneer Flying Club"];
const NAMES: &[&str] = &["Maya Chen", "Luca Moretti", "Amara Okafor", "Jonas Berg", "Priya Nair", "Diego Alvarez", "Hana Sato", "Owen Price", "Leila Haddad", "Tomás Silva", "Ingrid Larsen", "Kwame Mensah", "Sofia Rossi", "Arjun Mehta", "Elif Kaya", "Noah Fischer"];

/// a hired pilot's wage an hour, by their skill.
pub fn wage(skill: u8) -> f64 {
    18.0 + 12.0 * skill as f64
}

/// what signing a pilot costs.
pub fn signing_fee(skill: u8) -> f64 {
    2_500.0 * skill as f64
}

impl Career {
    pub fn load() -> Career {
        let mut c: Career = load_path().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).and_then(|v| serde_json::from_value(prune(v)).ok()).unwrap_or_default();
        // a board from before jobs had clients and tiers is made again.
        if c.board.iter().any(|j| j.client.is_empty()) {
            c.board.clear();
        }
        c
    }

    pub fn save(&self) {
        let Some(p) = path() else { return };
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let tmp = p.with_extension("json.tmp");
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(&tmp, &p);
            }
        }
    }

    pub fn has(&self, l: Lic) -> bool {
        self.lics.contains(&l)
    }

    /// the highest of the main licences, for the pilot's title.
    pub fn title(&self) -> &'static str {
        [Lic::Atpl, Lic::Cpl, Lic::Ppl].into_iter().find(|l| self.has(*l)).map_or("Student pilot", |l| l.name())
    }

    pub fn pay(&mut self, what: impl Into<String>, amount: f64) {
        self.money += amount;
        self.ledger.push(Entry { unix: profile::now_unix(), what: what.into(), amount });
        if self.ledger.len() > 300 {
            self.ledger.remove(0);
        }
    }

    pub fn rep_of(&self, k: Kind) -> f64 {
        self.rep_by.get(&k).copied().unwrap_or(0.0)
    }

    /// may fly this class.
    pub fn rated(&self, c: Class) -> bool {
        c.licence().is_some_and(|l| self.has(l))
    }

    /// may take this job: the licence for the work, the rating for the aircraft, the reputation for its tier.
    pub fn can_take(&self, j: &Job) -> Result<(), String> {
        let c = class_parse(&j.class);
        if !self.has(j.kind.lic()) {
            return Err(format!("{} {}", tr_en("Needs:"), tr_en(j.kind.lic().name())));
        }
        if !self.rated(c) {
            return Err(format!("{} {}", tr_en("Needs:"), c.licence().map_or("", |l| tr_en(l.name()))));
        }
        let (t, _) = tier(self.rep_of(j.kind));
        if (j.tier as usize) > t + 1 {
            return Err(format!("{} {} {}", tr_en("Needs"), tr_en(TIERS[j.tier as usize - 1].1), tr_en("reputation")));
        }
        Ok(())
    }

    /// a fresh board of jobs out of the base: the work the pilot does, and some just out of reach.
    pub fn make_board(&mut self) {
        let db = Db::get();
        let Some(bi) = db.find(&self.base) else { return };
        let base = &db.apts[bi as usize];
        let seed = (bi as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (self.refresh as u64 + 1).wrapping_mul(0x2545_F491_4F6C_DD1D) ^ ((self.jobs_done as u64) << 7);
        let mut rng = Rng(seed | 1);
        let rated: Vec<Class> = CLASSES.into_iter().filter(|c| self.rated(*c)).collect();
        let mut board = Vec::new();
        for kind in KINDS {
            // the work for an aircraft the pilot flies, or the first steps for a student.
            let classes: Vec<Class> = kind.classes().iter().copied().filter(|c| rated.contains(c) || (rated.is_empty() && *c == Class::Light)).collect();
            if classes.is_empty() {
                continue;
            }
            let n = if self.has(kind.lic()) { 3 } else { 1 };
            for k in 0..n {
                let c = rng.pick(&classes);
                let t: u8 = match k {
                    0 => 1,
                    1 => 1 + (rng.f() < 0.5) as u8,
                    _ => 2 + (rng.f() < 0.4) as u8,
                };
                if let Some(j) = self.make_job(&mut rng, base, bi, kind, c, t) {
                    board.push(j);
                }
            }
        }
        board.sort_by(|a, b| (a.kind, a.tier).cmp(&(b.kind, b.tier)).then(a.pay.total_cmp(&b.pay)));
        self.board = board;
        self.board_for = self.base.clone();
    }

    fn make_job(&mut self, rng: &mut Rng, base: &Apt, bi: u32, kind: Kind, c: Class, t: u8) -> Option<Job> {
        let db = Db::get();
        let (lo, hi, rwy_ft, paved) = c.reach();
        // a tier's range: the first stays near, the third goes far.
        let span = hi - lo;
        let (lo_t, hi_t) = match t {
            1 => (lo, lo + span * 0.4),
            2 => (lo + span * 0.25, lo + span * 0.75),
            _ => (lo + span * 0.5, hi),
        };
        let spec = aircraft::list().into_iter().find(|s| class_of(s) == c);
        let max_payload = spec.map_or(500.0, |s| s.payload[1]);
        let (payload, pax) = match kind {
            Kind::Passengers | Kind::Charter | Kind::Airline | Kind::Tour => {
                let per = 95.0;
                let pax = ((max_payload * (0.3 + 0.6 * rng.f())) / per).clamp(1.0, 420.0).floor();
                (pax * per, pax as u32)
            }
            Kind::Skydive => {
                let pax = (max_payload / 95.0).floor().clamp(1.0, 3.0);
                (pax * 95.0, pax as u32)
            }
            Kind::Cargo => ((max_payload * (0.35 + 0.6 * rng.f())).round(), 0),
            Kind::Medical => (180.0_f64.min(max_payload), 1),
            _ => (0.0, 0),
        };
        let client = rng.pick(CLIENTS).to_string();
        if kind == Kind::Display {
            return self.display_job(rng, base, bi, c, t, client);
        }
        let bearing = rng.f() * 360.0;
        let (to, via, via_name, dist) = if kind.round_trip() {
            let r = if kind == Kind::Skydive { 0.0 } else { lo_t * 0.5 + rng.f() * (hi_t - lo_t) * 0.5 };
            let city = if kind == Kind::Tour {
                // a city near the base is the sight.
                crate::menu::CITIES.iter().map(|c| (c, nm_between((base.lat, base.lon), (c.1, c.2)))).filter(|(_, d)| *d > 4.0 && *d < hi_t.max(40.0)).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(c, _)| c)
            } else {
                None
            };
            let (lat, lon) = match city {
                Some(c) => (c.1, c.2),
                None if kind == Kind::Skydive => (base.lat, base.lon),
                None => offset(base.lat, base.lon, r.max(4.0), bearing),
            };
            let name = match kind {
                Kind::Tour => city.map_or_else(|| tr_en("the coast").to_string(), |c| c.0.to_string()),
                Kind::Skydive => tr_en("the drop zone over the field").to_string(),
                _ => {
                    let nearest = db.near(lat, lon, 40_000.0).into_iter().min_by(|a, b| a.1.total_cmp(&b.1)).map(|(i, _)| db.apts[i as usize].city.clone()).filter(|s| !s.is_empty());
                    nearest.map_or_else(|| tr_en("the survey site").to_string(), |c| format!("{} {c}", tr_en("the site near")))
                }
            };
            let out = nm_between((base.lat, base.lon), (lat, lon));
            (base.ident.clone(), Some((lat, lon)), name, (out * 2.0).max(8.0))
        } else {
            let mut cands: Vec<(u32, f64)> = db
                .apts
                .iter()
                .enumerate()
                .filter(|(i, a)| {
                    if *i as u32 == bi || a.rwys.is_empty() {
                        return false;
                    }
                    let (len, pv) = longest_ft(a);
                    len >= rwy_ft && (pv || !paved) && (c == Class::Light || a.kind >= 1)
                })
                .map(|(i, a)| (i as u32, nm_between((base.lat, base.lon), (a.lat, a.lon))))
                .filter(|(_, d)| *d >= lo_t && *d <= hi_t)
                .collect();
            if cands.is_empty() {
                return None;
            }
            cands.sort_by(|a, b| a.1.total_cmp(&b.1));
            let (ai, d) = rng.pick(&cands);
            (db.apts[ai as usize].ident.clone(), None, String::new(), d)
        };
        let (rate, start) = c.rates();
        let urgency = match kind {
            Kind::Medical => 1.6,
            Kind::Charter => 1.3,
            Kind::Skydive => 1.2,
            _ => 1.0,
        };
        let tier_k = [1.0, 1.5, 2.2][(t as usize - 1).min(2)];
        let dist = if kind == Kind::Skydive { 15.0 } else { dist };
        let pay = ((start + rate * dist * (0.85 + 0.3 * rng.f())) * urgency * tier_k / 10.0).round() * 10.0;
        let slack = match kind {
            Kind::Medical => 1.3,
            Kind::Skydive => 2.5,
            _ => 1.7 - 0.15 * (t as f64 - 1.0),
        };
        let limit_s = dist / c.cruise_kt() * 3600.0 * slack + if kind == Kind::Skydive { 1800.0 } else { 1200.0 };
        let id = self.next_id;
        self.next_id += 1;
        Some(Job { id, kind, from: base.ident.clone(), to, via, dist_nm: dist, payload_kg: payload, pax, pay, limit_s, class: format!("{c:?}"), tier: t, client, via_name })
    }

    /// an air show at another airport: a low pass over it and back.
    fn display_job(&mut self, rng: &mut Rng, base: &Apt, bi: u32, c: Class, t: u8, client: String) -> Option<Job> {
        let db = Db::get();
        let (lo, hi, _, _) = c.reach();
        let cands: Vec<(u32, f64)> = db
            .near(base.lat, base.lon, hi * 1852.0)
            .into_iter()
            .filter(|(i, _)| *i != bi && db.apts[*i as usize].kind >= 1)
            .map(|(i, _)| (i, nm_between((base.lat, base.lon), (db.apts[i as usize].lat, db.apts[i as usize].lon))))
            .filter(|(_, d)| *d >= lo * 0.5)
            .collect();
        if cands.is_empty() {
            return None;
        }
        let (ai, d) = rng.pick(&cands);
        let a = &db.apts[ai as usize];
        let (rate, start) = c.rates();
        let pay = ((start + rate * d * 2.0) * [1.0, 1.5, 2.2][(t as usize - 1).min(2)] / 10.0).round() * 10.0;
        let id = self.next_id;
        self.next_id += 1;
        Some(Job { id, kind: Kind::Display, from: base.ident.clone(), to: base.ident.clone(), via: Some((a.lat, a.lon)), dist_nm: d * 2.0, payload_kg: 0.0, pax: 0, pay, limit_s: d * 2.0 / c.cruise_kt() * 3600.0 * 1.8 + 1200.0, class: format!("{c:?}"), tier: t, client, via_name: format!("{} {}", a.ident, a.name) })
    }

    /// counts the company's earnings since last time (up to two days of them).
    pub fn accrue(&mut self) {
        let now = profile::now_unix();
        if self.accrued == 0 {
            self.accrued = now;
            return;
        }
        let hours = (now.saturating_sub(self.accrued) as f64 / 3600.0).min(48.0);
        if hours < 0.25 {
            return;
        }
        self.accrued = now;
        let mut net = 0.0;
        let mut flown = 0;
        for h in &mut self.hires {
            // they fly a third of the time, and are paid for it.
            let earn = h.plane.as_ref().and_then(|k| aircraft::by_key(k)).map_or(0.0, |s| class_of(s).hourly() * hours * 0.33 * (0.75 + 0.1 * h.skill as f64));
            let cost = wage(h.skill) * hours * 0.33;
            if earn > 0.0 {
                flown += 1;
            }
            h.earned += earn - cost;
            net += earn - cost;
        }
        if !self.hires.is_empty() && net.abs() >= 1.0 {
            self.pay(format!("{} ({} {}, {:.0} h)", tr_en("Company flying"), flown, tr_en("pilots"), hours), net.round());
        }
    }

    /// three pilots looking for work, the same until the board is refreshed.
    pub fn candidates(&self) -> Vec<Hire> {
        let seed = 0xA076_1D64_78BD_642F ^ (self.refresh as u64 + 7).wrapping_mul(0x9E37_79B9) ^ self.hires.len() as u64;
        let mut rng = Rng(seed | 1);
        let mut out: Vec<Hire> = Vec::new();
        for _ in 0..12 {
            if out.len() == 3 {
                break;
            }
            let name = rng.pick(NAMES).to_string();
            let skill = 1 + (rng.f() * 4.99) as u8;
            if !self.hires.iter().any(|h| h.name == name) && !out.iter().any(|h| h.name == name) {
                out.push(Hire { name, skill, plane: None, hired: 0, earned: 0.0 });
            }
        }
        out
    }

    /// the owned aircraft nobody else is flying.
    pub fn free_planes(&self) -> Vec<String> {
        self.owned.iter().filter(|k| !self.hires.iter().any(|h| h.plane.as_ref() == Some(*k))).cloned().collect()
    }

    pub fn condition(&self, key: &str) -> f64 {
        1.0 - self.wear.get(key).copied().unwrap_or(0.0)
    }
}

pub fn class_parse(s: &str) -> Class {
    match s {
        "BizJet" => Class::BizJet,
        "Airliner" => Class::Airliner,
        "Fighter" => Class::Fighter,
        _ => Class::Light,
    }
}

/// the job being flown.
pub struct Flight {
    pub job: Job,
    pub spec_key: String,
    pub rented: bool,
    pub t0: f64,
    pub fuel0: f64,
    pub violations0: u32,
    /// it went over the point on the way.
    pub via_done: bool,
    pub airborne: bool,
    pub landing: Option<Landing>,
    pub max_g: f64,
    pub max_bank: f64,
    pub max_sink: f64,
    still: f64,
    pub done: bool,
}

impl Flight {
    /// a job not yet flown, for the briefing's list of objectives.
    pub fn preview(job: Job) -> Flight {
        let via_done = job.via.is_none();
        Flight { job, spec_key: String::new(), rented: true, t0: 0.0, fuel0: 0.0, violations0: 0, via_done, airborne: false, landing: None, max_g: 1.0, max_bank: 0.0, max_sink: 0.0, still: 0.0, done: false }
    }
}

/// the objectives of a job, for the card: the words, done.
pub fn objectives(f: &Flight) -> Vec<(String, bool)> {
    let j = &f.job;
    let mut v = vec![(tr_en("Take off").to_string(), f.airborne || f.landing.is_some())];
    if j.via.is_some() {
        let what = match j.kind {
            Kind::Tour => format!("{} {}", tr_en("Fly the passengers over"), j.via_name),
            Kind::Photo => format!("{} {}", tr_en("Photograph from below 3,000 ft:"), j.via_name),
            Kind::Skydive => tr_en("Climb to 7,500 ft above the field and drop the jumpers").to_string(),
            Kind::Display => format!("{} {}", tr_en("Low pass below 1,000 ft over"), j.via_name),
            _ => format!("{} {}", tr_en("Pass over"), j.via_name),
        };
        v.push((what, f.via_done));
    }
    v.push((format!("{} {}", tr_en("Land at"), j.to), f.landing.is_some()));
    v.push((tr_en("Stop and park").to_string(), f.done));
    v
}

/// a mark out of 100 from how far over a limit something went: full marks at `ok`, none at `bad`.
fn mark(v: f64, ok: f64, bad: f64) -> u32 {
    (100.0 * (1.0 - ((v - ok) / (bad - ok)).clamp(0.0, 1.0))).round() as u32
}

/// stars from the marks' average.
pub fn stars_of(avg: f64) -> u8 {
    match avg {
        a if a >= 92.0 => 5,
        a if a >= 80.0 => 4,
        a if a >= 62.0 => 3,
        a if a >= 40.0 => 2,
        _ => 1,
    }
}

impl Game {
    /// the aircraft the career flies a job's class in: one owned and free, else one to rent.
    pub(crate) fn career_spec(&self, c: Class) -> Option<&'static aircraft::Spec> {
        let owned = self.career.free_planes().iter().filter_map(|k| aircraft::by_key(k)).find(|s| class_of(s) == c);
        owned.or_else(|| career_types().into_iter().find(|s| class_of(s) == c))
    }

    /// takes a job: off to the base with the aircraft loaded and the route set.
    pub(crate) fn fly_job(&mut self, job: Job, plane: Option<&str>) {
        let c = class_parse(&job.class);
        let spec = plane.and_then(aircraft::by_key).filter(|s| class_of(s) == c).or_else(|| self.career_spec(c));
        let Some(spec) = spec else { return };
        let db = Db::get();
        let Some(bi) = db.find(&job.from) else { return };
        self.set_aircraft(spec);
        self.menu.aircraft = aircraft::index_of(spec);
        // the load, and fuel for the trip with 45 minutes (an hour in a jet) to spare.
        let most = spec.payload[1].max(1.0);
        self.menu.payload = Some((job.payload_kg + 85.0).min(most) / most);
        let reserve = if spec.is_jet() { 1.0 } else { 0.75 };
        let need = spec.fuel_flow[1] * 0.7 * (job.dist_nm / c.cruise_kt() + reserve) * 3600.0;
        self.menu.fuel = (need / spec.fuel_kg.max(1.0)).clamp(0.25, 1.0);
        self.menu.cold = false;
        self.stand_start = true;
        self.end_mission();
        self.atc.reset();
        self.start_airport(bi, None, false, true);
        self.plan.clear();
        let a = &db.apts[bi as usize];
        let route = match job.via {
            Some((lat, lon)) if job.kind != Kind::Skydive => format!("{lat:.5},{lon:.5} {}", job.to),
            _ => job.to.clone(),
        };
        let _ = self.plan.add_route(&route, Some((a.lat, a.lon)));
        self.plan.cruise_ft = match c {
            Class::Light => 4_500.0,
            Class::Fighter => 15_000.0,
            _ => 35_000.0_f64,
        }
        .min(5_000.0 + job.dist_nm * 150.0);
        self.plan.begin((a.lat, a.lon));
        let key = spec.keys.first().copied().unwrap_or("").to_string();
        let owned = self.career.owned.contains(&key);
        self.career_flight = Some(Flight { job: job.clone(), spec_key: key, rented: !owned, t0: self.time, fuel0: spec.fuel_kg * self.menu.fuel, violations0: 0, via_done: job.via.is_none(), airborne: false, landing: None, max_g: 1.0, max_bank: 0.0, max_sink: 0.0, still: 0.0, done: false });
        self.career.active = Some(job);
        self.career.view.brief = None;
        self.career.view.debrief = None;
        self.career.save();
    }

    /// a graded touchdown from the flight log: the one at the destination counts.
    pub(crate) fn career_landed(&mut self, l: &Landing) {
        let Some(f) = self.career_flight.as_mut() else { return };
        if f.done || f.landing.is_some() || !f.airborne || !f.via_done {
            return;
        }
        let db = Db::get();
        let (la, lo, _) = geo::pos_to_lla(self.plane.pos);
        let near = db.find(&f.job.to).map(|i| &db.apts[i as usize]).is_some_and(|a| nm_between((la.to_degrees(), lo.to_degrees()), (a.lat, a.lon)) < 3.0);
        if l.airport.as_deref() == Some(f.job.to.as_str()) || near {
            f.landing = Some(l.clone());
        }
    }

    /// once a step of a job: the objectives, the comfort, the stop at the end.
    pub(crate) fn career_step(&mut self, dt: f64) {
        let Some(mut f) = self.career_flight.take() else { return };
        if f.done {
            self.career_flight = Some(f);
            return;
        }
        let p = &self.plane;
        let (la, lo, h) = geo::pos_to_lla(p.pos);
        let (lat, lon) = (la.to_degrees(), lo.to_degrees());
        let agl_ft = self.world.ground(la, lo).map_or(h, |g| h - g.h) * 3.28084;
        let (_, _, u) = geo::enu(p.pos);
        if !p.on_ground {
            if agl_ft > 50.0 {
                f.airborne = true;
            }
            f.max_g = f.max_g.max(p.g_load.abs());
            f.max_bank = f.max_bank.max((p.rot * glam::DVec3::X).dot(u).clamp(-1.0, 1.0).asin().to_degrees().abs());
            f.max_sink = f.max_sink.max(-p.vel.dot(u) * 196.85);
        }
        if let (Some((vl, vo)), false) = (f.job.via, f.via_done) {
            let d = nm_between((lat, lon), (vl, vo));
            f.via_done = match f.job.kind {
                Kind::Photo => d < 1.5 && agl_ft < 3000.0 && !p.on_ground,
                Kind::Skydive => d < 2.5 && agl_ft >= 7_500.0,
                Kind::Display => d < 1.0 && agl_ft < 1000.0 && !p.on_ground,
                _ => d < 1.5 && !p.on_ground,
            };
            if f.via_done {
                self.status = match f.job.kind {
                    Kind::Skydive => tr_en("Jumpers away! Head back and land.").to_string(),
                    _ => format!("{}: {}", tr_en("Objective done"), f.job.via_name),
                };
                self.status_until = self.real + 6.0;
            }
        }
        f.still = if p.on_ground && p.vel.length() < 0.5 { f.still + dt } else { 0.0 };
        if self.crashed || (f.landing.is_some() && f.still > 6.0) {
            let d = self.settle_job(&f, self.crashed);
            f.done = true;
            self.career.view.debrief = Some(d);
        }
        self.career_flight = Some(f);
    }

    /// marks, pays out and files a job, or charges for it gone wrong.
    fn settle_job(&mut self, f: &Flight, crashed: bool) -> Debrief {
        let c = class_parse(&f.job.class);
        let j = &f.job;
        let title = match j.kind {
            k if k.round_trip() => format!("{}  ·  {}", tr_en(k.name()), j.via_name),
            k => format!("{}  ·  {} → {}", tr_en(k.name()), j.from, j.to),
        };
        let mut lines: Vec<(String, f64)> = Vec::new();
        let hours = ((self.time - f.t0) / 3600.0).max(0.0);
        let fuel_used = (f.fuel0 - self.plane.fuel).max(0.0);
        let fuel_cost = fuel_used * c.fuel_price();
        let rent = if f.rented { c.rent() * hours.max(0.25) } else { 0.0 };
        let violations = self.atc.violations.saturating_sub(f.violations0);
        if crashed {
            let repair = aircraft::by_key(&f.spec_key).and_then(price_of).unwrap_or(c.rent() * 200.0) * 0.15;
            lines.push((tr_en("Damage to the aircraft").into(), -repair.round()));
            lines.push((tr_en("Fuel").into(), -fuel_cost.round()));
            let rep = -60.0 * j.tier as f64;
            let net: f64 = lines.iter().map(|l| l.1).sum();
            self.career.pay(format!("{} {} {} ({})", tr_en("Failed:"), tr_en(j.kind.name()), j.to, f.spec_key.to_uppercase()), net);
            let k = self.career.rep_of(j.kind);
            self.career.rep_by.insert(j.kind, (k + rep).max(0.0));
            self.career.rep = (self.career.rep + rep).max(0.0);
            self.career.active = None;
            self.career.board.clear();
            self.career.save();
            return Debrief { ok: false, stars: 0, marks: Vec::new(), lines, rep, xp: 0, note: tr_en("The aircraft was lost. The job is failed.").into(), kind: j.kind, title };
        }
        // the marks.
        let mut marks: Vec<(String, u32, String)> = Vec::new();
        let landing = f.landing.as_ref();
        let lm = landing.map_or(40, |l| l.score);
        marks.push((tr_en("Landing").into(), lm, landing.map_or_else(|| tr_en("not seen").to_string(), |l| format!("{}  ·  {:.0} fpm", l.grade.text(), l.sink_fpm))));
        let late = hours * 3600.0 - j.limit_s;
        let tm = if late <= 0.0 { 100 } else { mark(late, 0.0, j.limit_s) };
        marks.push((tr_en("Timing").into(), tm, if late <= 0.0 { format!("{} {}", profile::hhmm(-late), tr_en("to spare")) } else { format!("{} {}", profile::hhmm(late), tr_en("late")) }));
        if j.kind.carries_people() {
            let g = mark(f.max_g, 1.6, 2.6);
            let b = mark(f.max_bank, 35.0, 65.0);
            let s = mark(f.max_sink, 1_800.0, 4_000.0);
            marks.push((tr_en("Passenger comfort").into(), g.min(b).min(s), format!("{:.1} g  ·  {:.0}° {}  ·  {:.0} fpm", f.max_g, f.max_bank, tr_en("bank"), f.max_sink)));
        }
        let pm = 100u32.saturating_sub(violations * 25);
        marks.push((tr_en("Procedures").into(), pm, if violations == 0 { tr_en("no deviations").to_string() } else { format!("{violations} {}", tr_en("ATC deviations")) }));
        let key = f.spec_key.clone();
        let cond = if f.rented { 1.0 } else { self.career.condition(&key) };
        let care = if landing.is_some_and(|l| l.grade >= crate::flightlog::Grade::Hard) { 55 } else { 100 };
        marks.push((tr_en("Aircraft care").into(), care, if care < 100 { tr_en("a hard landing").to_string() } else { format!("{:.0}% {}", cond * 100.0, tr_en("condition")) }));
        let avg = marks.iter().map(|m| m.1 as f64).sum::<f64>() / marks.len() as f64;
        let stars = stars_of(avg);
        // the money.
        let quality = 0.55 + 0.14 * stars as f64;
        lines.push((format!("{}  ·  {}", tr_en(j.kind.name()), j.client), j.pay));
        if (quality - 1.0).abs() > 0.005 {
            lines.push((format!("{} ({}★)", tr_en("Client rating"), stars), (j.pay * (quality - 1.0)).round()));
        }
        lines.push((tr_en("Fuel").into(), -fuel_cost.round()));
        if rent > 0.0 {
            lines.push((format!("{} {:.1} h", tr_en("Rental"), hours), -rent.round()));
        }
        let fee = match c {
            Class::Light => 15.0,
            Class::BizJet => 350.0,
            Class::Fighter => 0.0,
            _ => 2_500.0,
        };
        if fee > 0.0 && !j.kind.round_trip() {
            lines.push((tr_en("Landing fee").into(), -fee));
        }
        // an owned aircraft wears: an hour's use, and more for a hard landing.
        if !f.rented {
            let w = 0.008 * hours.max(0.2) + if care < 100 { 0.04 } else { 0.0 };
            let e = self.career.wear.entry(key.clone()).or_insert(0.0);
            *e = (*e + w).min(1.0);
        }
        let rep = ((8.0 + 14.0 * stars as f64) * j.tier as f64 - violations as f64 * 10.0).round();
        let xp = ((80.0 + j.dist_nm.min(800.0) * 0.6 + 30.0 * stars as f64) * (1.0 + 0.25 * (j.tier as f64 - 1.0))) as u64;
        let net: f64 = lines.iter().map(|l| l.1).sum();
        self.career.pay(format!("{} {} → {} ({})", tr_en(j.kind.name()), j.from, j.to, key.to_uppercase()), net);
        *self.career.rep_by.entry(j.kind).or_default() += rep.max(0.0);
        self.career.rep = (self.career.rep + rep).max(0.0);
        self.career.hours += hours;
        self.career.jobs_done += 1;
        self.career.stars.push(stars);
        if self.career.stars.len() > 200 {
            self.career.stars.remove(0);
        }
        self.career.base = j.to.clone();
        self.career.active = None;
        self.career.board.clear();
        self.career.save();
        let note = match stars {
            5 => tr_en("Outstanding: the client asks for you by name."),
            4 => tr_en("A very good job."),
            3 => tr_en("Job done."),
            2 => tr_en("The client was not impressed."),
            _ => tr_en("The client will think twice."),
        };
        self.award(&format!("{}: {}", tr_en("Job"), tr_en(j.kind.name())), xp);
        self.unlock("first_job");
        if stars == 5 {
            self.unlock("flawless");
        }
        self.sweep_badges();
        Debrief { ok: true, stars, marks, lines, rep, xp, note: note.into(), kind: j.kind, title }
    }

    /// whether a requirement is met, and how far along it is in words.
    pub(crate) fn req_state(&self, r: Req) -> (bool, String) {
        // only what was flown in the career counts: activities and free flight do not open licences.
        let hours = self.career.hours;
        let lvl = crate::pilot::level(self.profile.xp).n;
        let title = |id: &str| crate::missions::by_id(id).map_or(id.to_string(), |m| tr_en(m.title).to_string());
        match r {
            // (the button beside it says lesson, checkride or mission)
            Req::Lesson(id) | Req::Exam(id) => (self.career.trained.contains(id), title(id)),
            Req::Holds(l) => (self.career.has(l), tr_en(l.name()).to_string()),
            Req::Hours(h) => (hours >= h, format!("{:.1} / {h:.0} {}", hours.min(h), tr_en("hours flown"))),
            Req::Jobs(n) => (self.career.jobs_done >= n, format!("{} / {n} {}", self.career.jobs_done.min(n), tr_en("jobs done"))),
            Req::Level(n) => (lvl >= n, format!("{} {n} ({} {lvl})", tr_en("Pilot level"), tr_en("now"))),
            Req::Rep(v) => (self.career.rep >= v, format!("{:.0} / {v:.0} {}", self.career.rep.min(v), tr_en("reputation"))),
            Req::Fee(v) => (self.career.money >= v, format!("{} {}", tr_en("Training fee"), money(v))),
        }
    }

    /// whether a licence can be had now.
    pub(crate) fn can_get(&self, l: Lic) -> bool {
        !self.career.has(l) && l.reqs().iter().all(|r| self.req_state(*r).0)
    }

    /// takes a licence: the fee paid, and the jobs it opens on the board.
    pub(crate) fn get_licence(&mut self, l: Lic) {
        if !self.can_get(l) {
            return;
        }
        if l.fee() > 0.0 {
            self.career.pay(format!("{} {}", tr_en(l.name()), tr_en("training")), -l.fee());
        }
        self.career.lics.push(l);
        self.career.board.clear();
        self.career.save();
        self.award(&format!("{} {}", tr_en("Licence:"), tr_en(l.name())), 500);
        match l {
            Lic::Ppl => self.unlock("ppl"),
            Lic::Cpl => self.unlock("cpl"),
            Lic::Atpl => self.unlock("atpl"),
            _ => {}
        }
    }

    /// the next thing worth doing, for the career's home page: a heading, a line, and what it is.
    pub(crate) fn next_step(&self) -> (String, String, Next) {
        if let Some(j) = &self.career.active {
            return (tr_en("Fly your job").into(), format!("{}  ·  {} → {}", tr_en(j.kind.name()), j.from, j.to), Next::Job);
        }
        if !self.career.has(Lic::Ppl) {
            for r in Lic::Ppl.reqs() {
                let (ok, what) = self.req_state(*r);
                if !ok {
                    let next = match r {
                        Req::Lesson(id) | Req::Exam(id) => Next::Mission(id),
                        _ => Next::Training,
                    };
                    return (tr_en("Earn your private pilot licence").into(), what, next);
                }
            }
            return (tr_en("Your private pilot licence is ready").into(), tr_en("Every requirement is met: take it on the Training page.").into(), Next::Training);
        }
        if let Some(l) = LICS.iter().copied().find(|l| self.can_get(*l)) {
            return (format!("{} {}", tr_en("Ready:"), tr_en(l.name())), tr_en(l.opens()).into(), Next::Training);
        }
        let n = self.career.board.iter().filter(|j| self.career.can_take(j).is_ok()).count();
        (tr_en("Take a job").into(), format!("{n} {} {}", tr_en("jobs open to you at"), self.career.base), Next::Jobs)
    }
}

/// where the next step leads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Next {
    Job,
    Jobs,
    Training,
    Mission(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_board_has_jobs_in_reach_and_money_reads_well() {
        let mut c = Career { started: true, base: "KHAF".into(), lics: vec![Lic::Ppl], ..Default::default() };
        c.make_board();
        assert!(!c.board.is_empty());
        for j in &c.board {
            assert!(j.pay > 0.0 && j.dist_nm > 0.0 && j.limit_s > 0.0, "{j:?}");
            assert!((1..=3).contains(&j.tier));
        }
        // a private pilot may take the ferry, tour and photo work, not the commercial work.
        assert!(c.board.iter().any(|j| j.kind == Kind::Ferry && j.tier == 1 && c.can_take(j).is_ok()));
        assert!(c.board.iter().filter(|j| j.kind == Kind::Passengers).all(|j| c.can_take(j).is_err()));
        assert_eq!(money(1_234_567.4), "$1,234,567");
        assert_eq!(money(-50.0), "-$50");
        assert_eq!(money_short(2_500_000.0), "$2.5M");
    }

    #[test]
    fn reputation_tiers_and_stars() {
        assert_eq!(tier(0.0).0, 0);
        assert_eq!(tier(150.0).0, 1);
        assert_eq!(tier(5000.0), (4, 1.0));
        assert!((tier(300.0).1 - 0.5).abs() < 1e-6);
        assert_eq!(stars_of(95.0), 5);
        assert_eq!(stars_of(70.0), 3);
        assert_eq!(stars_of(10.0), 1);
        assert_eq!(mark(1.5, 1.6, 2.6), 100);
        assert_eq!(mark(2.6, 1.6, 2.6), 0);
    }

    #[test]
    fn every_licence_asks_for_real_things() {
        for l in LICS {
            for r in l.reqs() {
                // the career's own lessons and checkrides, never the activities' tutorials.
                use crate::missions::Cat;
                match r {
                    Req::Lesson(id) => assert!(crate::missions::by_id(id).is_some_and(|m| m.cat == Cat::Lesson), "{l:?}: {id}"),
                    Req::Exam(id) => assert!(crate::missions::by_id(id).is_some_and(|m| m.cat == Cat::Exam), "{l:?}: {id}"),
                    _ => {}
                }
            }
        }
        // each kind of work has an aircraft that can fly it.
        for k in KINDS {
            assert!(k.classes().iter().any(|c| career_types().iter().any(|s| class_of(s) == *c)), "{k:?}");
        }
    }

    #[test]
    fn an_old_save_with_the_helicopter_and_glider_still_loads() {
        let mut v = serde_json::to_value(Career { started: true, lics: vec![Lic::Ppl], ..Default::default() }).unwrap();
        v["lics"] = serde_json::json!(["Ppl", "Heli", "Glider", "Recon"]);
        v["rep_by"] = serde_json::json!({ "Ferry": 10.0, "Rescue": 40.0 });
        v["owned"] = serde_json::json!(["c172", "r44"]);
        let c: Career = serde_json::from_value(prune(v)).unwrap();
        assert_eq!(c.lics, vec![Lic::Ppl]);
        assert_eq!(c.owned, vec!["c172".to_string()]);
        assert_eq!(c.rep_by.len(), 1);
    }

    #[test]
    fn hired_pilots_earn_while_away() {
        let mut c = Career { owned: vec!["c172".into()], ..Default::default() };
        c.hires.push(Hire { name: "A".into(), skill: 3, plane: Some("c172".into()), hired: 0, earned: 0.0 });
        c.accrued = profile::now_unix() - 10 * 3600;
        let m0 = c.money;
        c.accrue();
        assert!(c.money > m0, "{} {}", c.money, m0);
        assert!(c.free_planes().is_empty());
        assert_eq!(c.candidates().len(), 3);
    }
}
