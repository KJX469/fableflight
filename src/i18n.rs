//! Interface language: English (the default), French and Korean.
//!
//! Text written in the code stays in English. Where it is shown, it goes
//! through one of:
//! - `t!("English", "Français")` for a one-off line, with format arguments
//!   when it has some: `t!("{n} tiles", "{n} tuiles", n = 3)`;
//! - `tr("English")` for text that lives in a table (the controls list,
//!   place names, presets…): it looks the French up in `fr()` below and
//!   falls back to the English;
//! - `caps(..)` for the damage messages the flight model writes in capitals.
//!
//! Korean (`Lang::Ko`) is table-driven: the plain `t!` and `tr` look the
//! English up in `ko()` and fall back to the English when it has no entry, so
//! coverage can grow one string at a time without touching the call sites.
//! The format-argument form of `t!` and the `caps`/`plural` helpers still fall
//! back to English for Korean.

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Fr,
    Ko,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::En, Lang::Fr, Lang::Ko];

    /// The language's own name, for the language picker.
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Fr => "Français",
            Lang::Ko => "한국어",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Fr => "fr",
            Lang::Ko => "ko",
        }
    }

    pub fn parse(s: &str) -> Option<Lang> {
        let s = s.trim().to_ascii_lowercase();
        if s.starts_with("fr") {
            Some(Lang::Fr)
        } else if s.starts_with("ko") {
            Some(Lang::Ko)
        } else if s.starts_with("en") {
            Some(Lang::En)
        } else {
            None
        }
    }

    /// The system's language, from the usual locale variables (Linux, macOS);
    /// English when it is not one we have.
    pub fn detect() -> Lang {
        ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
            .iter()
            .filter_map(|k| std::env::var(k).ok())
            .find(|v| !v.is_empty() && v != "C" && v != "POSIX")
            .and_then(|v| Lang::parse(&v))
            .unwrap_or(Lang::En)
    }
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn lang() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        1 => Lang::Fr,
        2 => Lang::Ko,
        _ => Lang::En,
    }
}

pub fn set_lang(l: Lang) {
    LANG.store(match l { Lang::En => 0, Lang::Fr => 1, Lang::Ko => 2 }, Ordering::Relaxed);
}

pub fn is_fr() -> bool {
    lang() == Lang::Fr
}

/// Which of the two literals a plain `t!` shows. Korean looks the English up in
/// `ko()` and falls back to the English when it has no translation yet.
pub fn pick(en: &'static str, fr: &'static str) -> &'static str {
    match lang() {
        Lang::Fr => fr,
        Lang::Ko => ko(en).unwrap_or(en),
        Lang::En => en,
    }
}

/// `t!("English", "Français")`, or with format arguments
/// `t!("{} km", "{} km", d)`: both strings get the same arguments.
macro_rules! t {
    ($en:literal, $fr:literal $(,)?) => {
        $crate::i18n::pick($en, $fr)
    };
    // With format arguments the template must be a literal, so Korean falls
    // back to the English wording here.
    ($en:literal, $fr:literal, $($arg:tt)*) => {
        if $crate::i18n::is_fr() { format!($fr, $($arg)*) } else { format!($en, $($arg)*) }
    };
}
pub(crate) use t;

/// Table text in the current language (the English itself when there is no
/// translation for it).
pub fn tr(en: &'static str) -> &'static str {
    match lang() {
        Lang::Fr => fr(en).unwrap_or(en),
        Lang::Ko => ko(en).unwrap_or(en),
        Lang::En => en,
    }
}

/// The same for text that is not `'static` (a place name kept in a `String`).
pub fn tr_owned(en: &str) -> String {
    match lang() {
        Lang::Fr => fr(en).unwrap_or(en).to_string(),
        Lang::Ko => ko(en).unwrap_or(en).to_string(),
        Lang::En => en.to_string(),
    }
}

/// French "de" before a place name, as it is written: "d'Amsterdam",
/// "du Havre", "des Sables-d'Olonne", "de la Rochelle", "de Paris".
/// (No elision before an H: "de Hambourg"; it depends on the word.)
pub fn de_place(name: &str) -> String {
    if let Some(rest) = name.strip_prefix("Le ") {
        return format!("du {rest}");
    }
    if let Some(rest) = name.strip_prefix("Les ") {
        return format!("des {rest}");
    }
    if let Some(rest) = name.strip_prefix("La ") {
        return format!("de la {rest}");
    }
    let vowel = name.chars().next().is_some_and(|c| "AEIOUYÀÂÄÉÈÊËÎÏÔÖÙÛÜaeiouyàâäéèêëîïôöùûü".contains(c));
    if vowel { format!("d'{name}") } else { format!("de {name}") }
}

/// "{n} request(s)" and the like.
pub fn plural(n: u64, en_one: &str, en_many: &str, fr_one: &str, fr_many: &str) -> String {
    let w = match (is_fr(), n) {
        (false, 1) => en_one,
        (false, _) => en_many,
        // In French, 0 and 1 take the singular.
        (true, 0 | 1) => fr_one,
        (true, _) => fr_many,
    };
    format!("{n} {w}")
}

fn fr(en: &str) -> Option<&'static str> {
    Some(match en {
        // ---- pages of the in-flight panels
        "Flight" => "Vol",
        "Go to" => "Destinations",
        "Weather" => "Météo",
        "Game" => "Jeu",
        "Settings" => "Réglages",
        "Controls" => "Commandes",
        "Bindings" => "Assignations",
        "Radio" => "Radios",
        "Autopilot" => "Pilote automatique",
        "Failures" => "Pannes",
        "battery master switch" => "batterie (interrupteur général)",
        "Ins" => "Inser",
        "start / stop the engine (automatic)" => "démarrer / couper le moteur (automatique)",
        "magnetos on / off (jets: fuel controls)" => "magnétos (jets : robinets carburant)",
        "starter (hold; jets: engage)" => "démarreur (maintenir ; jets : engager)",
        "mixture richer / leaner" => "mélange plus riche / plus pauvre",
        "Page Up / Down" => "Page préc. / suiv.",
        "navigation instruments (VOR, ILS, ADF)" => "instruments de navigation (VOR, ILS, ADF)",
        "OBS course left / right" => "OBS : course à gauche / à droite",
        "altimeter to the QNH" => "caler l'altimètre au QNH",

        // ---- controls list: headings
        "FLYING" => "PILOTAGE",
        "VIEW" => "VUE",
        "WORLD" => "MONDE",
        "GAME" => "JEU",
        "GAMEPAD (MSFS 2020 layout)" => "MANETTE (disposition MSFS 2020)",
        "GAMEPAD · FLYING" => "MANETTE · PILOTAGE",
        "GAMEPAD · MENUS" => "MANETTE · MENUS",
        "GAMEPAD · GLOBE" => "MANETTE · GLOBE",
        // keys
        "Space" => "Espace",
        "ISO (beside Left Shift)" => "< >",
        "F6 / ISO key" => "F6 / < >",
        "Mouse drag" => "Glisser souris",
        "Click / wheel" => "Clic / molette",
        "Mouse wheel" => "Molette",
        "Left stick" => "Stick gauche",
        "Right stick" => "Stick droit",
        "Left stick click" => "Clic stick gauche",
        "Right stick click" => "Clic stick droit",
        "Y + D-pad up / dn" => "Y + croix haut / bas",
        "D-pad" => "Croix",
        "A after a crash" => "A après un crash",
        "Y + Left stick click" => "Y + clic stick gauche",
        "Y + Right stick click" => "Y + clic stick droit",
        "Y + D-pad left / right" => "Y + croix gauche / droite",
        "Y (let go)" => "Y (relâché)",
        // what they do
        "pitch down / up" => "piquer / cabrer",
        "roll left / right" => "roulis gauche / droite",
        "rudder left / right" => "palonnier gauche / droite",
        "throttle up / down" => "plus / moins de gaz",
        "boost" => "boost (postcombustion)",
        "wheel brakes" => "freins de roues",
        "flaps down / up a notch" => "volets : sortir / rentrer d'un cran",
        "landing gear down / up" => "train d'atterrissage : sortir / rentrer",
        "slide the canopy open / shut (Robin)" => "ouvrir / fermer la verrière coulissante (Robin)",
        "speedbrake (airliners)" => "aérofreins (avions de ligne)",
        "thrust reversers, on the ground (hold)" => "inverseurs de poussée, au sol (maintenir)",
        "set the heading indicator to the compass" => "recaler le conservateur de cap sur le compas",
        "chase / cockpit camera" => "caméra extérieure / cockpit",
        "toggle fixed rear view" => "activer / quitter la vue arrière fixe",
        "C switches between the chase view and the cockpit; drag the mouse to look around." => "C passe de la vue extérieure au cockpit ; glissez la souris pour regarder autour.",
        "F4 starts or stops the engine; for a start by hand: Ins battery, Page Up mixture, I magnetos, hold O." => "F4 démarre ou coupe le moteur ; pour un démarrage à la main : Inser batterie, Page ↑ mélange, I magnétos, maintenir O.",
        "J shows the radio navigation instruments; the Radio page of the panel (Tab) tunes VORs and ILSs." => "J affiche les instruments de radionavigation ; la page Radios du panneau (Tab) règle VOR et ILS.",
        "X sets the altimeter to the QNH before take-off and on the way down." => "X cale l'altimètre au QNH avant le décollage et en descente.",
        "F and V move the flaps a notch; L the landing gear; B the wheel brakes." => "F et V sortent ou rentrent les volets d'un cran ; L le train ; B les freins.",
        "Tab opens the flight panel: instruments, places, weather, fuel and balance." => "Tab ouvre le panneau de vol : instruments, lieux, météo, carburant et centrage.",
        "R starts again in the air over where you are; G goes to any place or address." => "R repart en vol au-dessus de votre position ; G va à n'importe quel lieu ou adresse.",
        "The weather and the traffic are real: METAR at the airports, live aircraft around you." => "La météo et le trafic sont réels : METAR des aéroports, avions en vol autour de vous.",
        "F1 lists every key; joysticks and pedals are set up in Settings, Bindings." => "F1 liste toutes les touches ; joysticks et palonniers se règlent dans Réglages, Assignations.",
        "look around" => "regarder autour",
        "cockpit: work a switch, turn a knob (Cessna)" => "cockpit : actionner un interrupteur, tourner un bouton (Cessna)",
        "camera distance / cockpit zoom" => "distance caméra / zoom cockpit",
        "hide / show the HUD" => "masquer / afficher le HUD",
        "hide / show the minimap" => "masquer / afficher la mini-carte",
        "screenshot" => "capture d'écran",
        "time of day forward / back" => "avancer / reculer l'heure",
        "more / fewer clouds" => "plus / moins de nuages",
        "weather & time window" => "fenêtre météo et heure",
        "flight panel: instruments, places, weather" => "panneau de vol : instruments, destinations, météo",
        "pause menu: settings, globe, quit" => "menu pause : réglages, globe, quitter",
        "pause without the menu" => "pause sans le menu",
        "go to a place or address" => "aller à un lieu ou une adresse",
        "respawn in the air" => "réapparaître en vol",
        "sound on / off" => "son activé / coupé",
        "anti-aliasing (TAA) on / off" => "anticrénelage (TAA) activé / désactivé",
        "quick key reference" => "aide-mémoire des touches",
        "pitch and roll" => "tangage et roulis",
        "throttle up / down (hold)" => "plus / moins de gaz (maintenir)",
        "wheel brakes (hold)" => "freins de roues (maintenir)",
        "parking brake on / off" => "frein de parc serré / desserré",
        "trim nose down / up" => "trim à piquer / à cabrer",
        "flaps up / down a notch" => "volets : rentrer / sortir d'un cran",
        "landing gear" => "train d'atterrissage",
        "reset the view" => "réinitialiser la vue",
        "quick views" => "vues rapides",
        "fly again after a crash" => "repartir après un crash",
        "pause menu (B closes it)" => "menu pause (B pour fermer)",
        "flight panel" => "panneau de vol",
        "fly again" => "repartir",
        "afterburner / boost (hold)" => "postcombustion / boost (maintenir)",
        "thrust reversers (hold)" => "inverseurs de poussée (maintenir)",
        "speedbrake" => "aérofreins",
        "autopilot on / off" => "pilote automatique engagé / coupé",
        "start / stop the engine" => "démarrer / couper le moteur",
        "canopy open / shut" => "verrière : ouvrir / fermer",
        "master arm, lock a target" => "armement maître, verrouiller une cible",
        "fire at a locked target" => "tirer sur la cible verrouillée",
        "fire at a locked target (let go)" => "tirer sur la cible verrouillée (relâcher)",
        "scroll" => "faire défiler",
        "pause menu" => "menu pause",
        "flight panel (beside the flying)" => "panneau de vol (pendant le vol)",
        "move between choices" => "naviguer entre les choix",
        "press / choose" => "appuyer / choisir",
        "slide a slider" => "déplacer un curseur",
        "turn the page" => "changer de page",
        "back, close" => "retour, fermer",
        "turn the globe" => "tourner le globe",
        "zoom out / in" => "dézoomer / zoomer",
        "start at the crosshair" => "partir depuis le viseur",
        "take off" => "décoller",
        "settings and this controls list" => "réglages et cette liste des commandes",
        "back to the main menu" => "retour au menu principal",
        "aircraft" => "avion",
        "parking brake" => "frein de parc",
        // gamepad prompts
        "move" => "naviguer",
        "press" => "appuyer",
        "page" => "page",
        "resume" => "reprendre",
        "panel" => "panneau",
        "menu" => "menu",
        "respawn" => "réapparaître",
        "go somewhere else" => "aller ailleurs",
        "menu: respawn, globe" => "menu : réapparaître, globe",

        // ---- weather
        "DAY" => "JOUR",
        "SKY" => "CIEL",
        "WIND" => "VENT",
        "PRESETS" => "PRÉRÉGLAGES",
        "Clear" => "Dégagé",
        "Fair" => "Beau temps",
        "Hazy" => "Brumeux",
        "Overcast" => "Couvert",
        "Windy" => "Venteux",
        "Storms" => "Orages",
        "new" => "nouvelle lune",
        "waxing crescent" => "premier croissant",
        "first quarter" => "premier quartier",
        "waxing gibbous" => "gibbeuse croissante",
        "full" => "pleine lune",
        "waning gibbous" => "gibbeuse décroissante",
        "last quarter" => "dernier quartier",
        "waning crescent" => "dernier croissant",
        "SW" => "SO",
        "W" => "O",
        "NW" => "NO",

        // ---- aircraft
        "Twin-engine carrier fighter, 15.5 t. Fly-by-wire holds it to 7.5 g; Space for afterburner, / speedbrake. 135 kt approach." => "Chasseur embarqué biréacteur, 15,5 t. Les commandes de vol électriques le limitent à 7,5 g ; Espace pour la postcombustion, / aérofreins. Approche à 135 kt.",
        "Twin-engine two-seat fighter, 17 t. No fly-by-wire: mind the g and the angle of attack; Space for afterburner, / speedbrakes. 150 kt approach." => "Chasseur biplace biréacteur, 17 t. Pas de commandes de vol électriques : attention aux g et à l'incidence ; Espace pour la postcombustion, / aérofreins. Approche à 150 kt.",
        "Twin-engine stealth fighter, 25 t. Fly-by-wire holds it to 9 g; supersonic without afterburner up high. Space for afterburner, / to brake. 145 kt approach." => "Chasseur furtif biréacteur, 25 t. Les commandes de vol électriques le limitent à 9 g ; supersonique sans postcombustion en altitude. Espace pour la postcombustion, / pour freiner. Approche à 145 kt.",
        "Two-seat Mach 3 reconnaissance aircraft, 50 t. Hold Space: it cruises in afterburner, Mach 3.2 at 80,000 ft. Keep under 450 kt indicated on the way up and 3 g always. No flaps; / streams the drag chute once it is down. 175 kt approach." => "Biplace de reconnaissance à Mach 3, 50 t. Maintenez Espace : il croise en postcombustion, Mach 3,2 à 80 000 ft. Restez sous 450 kt indiqués en montée et sous 3 g en permanence. Pas de volets ; / sort le parachute-frein une fois posé. Approche à 175 kt.",
        "Two-stage rocket, 121 m and 5,000 t on the pad. Throttle up to lift off on 33 engines, then hold W a few seconds to tip it over. 1 picks which engines burn, Backspace lights the ship and drops the booster, 2 has the ship land itself." => "Fusée à deux étages, 121 m et 5 000 t sur le pas de tir. Mettez les gaz pour décoller sur 33 moteurs, puis tenez W quelques secondes pour l'incliner. 1 choisit les moteurs allumés, Retour arrière allume le vaisseau et largue le booster, 2 laisse le vaisseau se poser seul.",
        "Four-engine supersonic airliner, 137 t. Mach 2 at 55,000 ft without reheat; Space for reheat to take off. No flaps: F droops the nose (5° to take off, 12.5° to land). Rotate at 200 kt to 10° and no more until it is off (the tail touches at 11.5°); 160 kt approach, 10° nose up. Z reversers." => "Quadriréacteur de ligne supersonique, 137 t. Mach 2 à 55 000 ft sans réchauffe ; Espace pour la réchauffe au décollage. Pas de volets : F abaisse le nez (5° au décollage, 12,5° à l'atterrissage). Rotation à 200 kt jusqu'à 10°, pas plus avant l'envol (la queue touche à 11,5°) ; approche à 160 kt, 10° à cabrer. Z inverseurs.",
        "Four-seat single, 180 hp, retractable gear. Forgiving and slow: 125 kt flat out." => "Monomoteur quatre places, 180 ch, train rentrant. Indulgent et lent : 125 kt à fond.",
        "French four-seat wooden tourer and glider tug, 180 hp, fixed gear. Light on the controls; 60° flaps for steep, short landings." => "Quadriplace de voyage français en bois et remorqueur de planeurs, 180 ch, train fixe. Commandes légères ; volets à 60° pour des atterrissages courts et pentus.",
        "Four-engine wide-body airliner, 290 t. Heavy and slow to answer the controls; Mach 0.85 cruise, 150 kt approach. / speedbrake, Z reversers." => "Quadriréacteur gros-porteur, 290 t. Lourd et lent à répondre aux commandes ; croisière à Mach 0,85, approche à 150 kt. / aérofreins, Z inverseurs.",

        // ---- places (cities, one-click places, airports)
        "Mexico City" => "Mexico",
        "Havana" => "La Havane",
        "London" => "Londres",
        "Lisbon" => "Lisbonne",
        "Venice" => "Venise",
        "Athens" => "Athènes",
        "Moscow" => "Moscou",
        "Cairo" => "Le Caire",
        "Marrakesh" => "Marrakech",
        "Cape Town" => "Le Cap",
        "Dubai" => "Dubaï",
        "Tehran" => "Téhéran",
        "Kathmandu" => "Katmandou",
        "Singapore" => "Singapour",
        "Beijing" => "Pékin",
        "Seoul" => "Séoul",
        "Manila" => "Manille",
        "Yosemite Valley" => "Vallée de Yosemite",
        "Matterhorn" => "Cervin",
        "Sydney Harbour" => "Port de Sydney",
        "Black Forest" => "Forêt-Noire",
        "London Heathrow" => "Londres Heathrow",
        "St Maarten" => "Saint-Martin",

        _ => return None,
    })
}

/// Korean for the table text and the plain `t!` strings. Anything without an
/// entry here shows the English, so coverage grows one line at a time.
/// (The format-argument `t!` strings, the capitalised damage messages and the
/// long aircraft descriptions are not translated yet and stay in English.)
pub(crate) fn ko(en: &str) -> Option<&'static str> {
    Some(match en {
        // ---- flight panel pages
        "Flight" => "비행",
        "Go to" => "이동",
        "Weather" => "날씨",
        "Game" => "게임",
        "Settings" => "설정",
        "Controls" => "조작",
        "Bindings" => "키 할당",
        "Radio" => "무선",
        "Autopilot" => "오토파일럿",
        "Failures" => "고장",
        "battery master switch" => "배터리 마스터 스위치",
        "start / stop the engine (automatic)" => "엔진 시동 / 정지 (자동)",
        "magnetos on / off (jets: fuel controls)" => "마그네토 켜기 / 끄기 (제트기: 연료 밸브)",
        "starter (hold; jets: engage)" => "스타터 (길게 누름; 제트기: 작동)",
        "mixture richer / leaner" => "혼합비 농후 / 희박",
        "navigation instruments (VOR, ILS, ADF)" => "항법 계기 (VOR, ILS, ADF)",
        "OBS course left / right" => "OBS 코스 좌 / 우",
        "altimeter to the QNH" => "고도계를 QNH 로 설정",

        // ---- controls list: headings
        "FLYING" => "비행",
        "VIEW" => "시점",
        "WORLD" => "월드",
        "GAME" => "게임",
        "GAMEPAD (MSFS 2020 layout)" => "게임패드 (MSFS 2020 배열)",
        "GAMEPAD · FLYING" => "게임패드 · 비행",
        "GAMEPAD · MENUS" => "게임패드 · 메뉴",
        "GAMEPAD · GLOBE" => "게임패드 · 지구본",
        // keys
        "Space" => "스페이스",
        "Mouse drag" => "마우스 드래그",
        "Click / wheel" => "클릭 / 휠",
        "Mouse wheel" => "마우스 휠",
        "Left stick" => "왼쪽 스틱",
        "Right stick" => "오른쪽 스틱",
        "Left stick click" => "왼쪽 스틱 클릭",
        "Right stick click" => "오른쪽 스틱 클릭",
        "D-pad" => "방향 패드",
        "A after a crash" => "추락 후 A",
        "Y (let go)" => "Y (놓기)",
        // what they do
        "pitch down / up" => "피치 다운 / 업",
        "roll left / right" => "롤 좌 / 우",
        "rudder left / right" => "러더 좌 / 우",
        "throttle up / down" => "스로틀 증가 / 감소",
        "boost" => "부스트",
        "wheel brakes" => "휠 브레이크",
        "flaps down / up a notch" => "플랩 한 단 내림 / 올림",
        "landing gear down / up" => "착륙 장치 내림 / 올림",
        "speedbrake (airliners)" => "스피드 브레이크 (여객기)",
        "chase / cockpit camera" => "추적 / 조종석 카메라",
        "look around" => "주위 둘러보기",
        "camera distance / cockpit zoom" => "카메라 거리 / 조종석 확대",
        "hide / show the HUD" => "HUD 숨기기 / 표시",
        "hide / show the minimap" => "미니맵 숨기기 / 표시",
        "screenshot" => "스크린샷",
        "time of day forward / back" => "시간 앞으로 / 뒤로",
        "more / fewer clouds" => "구름 늘리기 / 줄이기",
        "weather & time window" => "날씨·시간 창",
        "flight panel: instruments, places, weather" => "비행 패널: 계기, 장소, 날씨",
        "pause menu: settings, globe, quit" => "일시정지 메뉴: 설정, 지구본, 종료",
        "pause without the menu" => "메뉴 없이 일시정지",
        "go to a place or address" => "장소나 주소로 이동",
        "respawn in the air" => "공중에서 리스폰",
        "sound on / off" => "소리 켜기 / 끄기",
        "anti-aliasing (TAA) on / off" => "안티앨리어싱 (TAA) 켜기 / 끄기",
        "quick key reference" => "빠른 키 안내",
        "pitch and roll" => "피치와 롤",
        "parking brake on / off" => "주차 브레이크 작동 / 해제",
        "trim nose down / up" => "트림 기수 내림 / 올림",
        "flaps up / down a notch" => "플랩 한 단 올림 / 내림",
        "landing gear" => "착륙 장치",
        "reset the view" => "시점 초기화",
        "quick views" => "빠른 시점",
        "fly again after a crash" => "추락 후 다시 비행",
        "flight panel" => "비행 패널",
        "fly again" => "다시 비행",
        "speedbrake" => "스피드 브레이크",
        "autopilot on / off" => "오토파일럿 작동 / 해제",
        "start / stop the engine" => "엔진 시동 / 정지",
        "scroll" => "스크롤",
        "pause menu" => "일시정지 메뉴",
        "move between choices" => "선택 이동",
        "press / choose" => "누르기 / 선택",
        "turn the page" => "페이지 넘기기",
        "back, close" => "뒤로, 닫기",
        "turn the globe" => "지구본 돌리기",
        "zoom out / in" => "축소 / 확대",
        "take off" => "이륙",
        "back to the main menu" => "메인 메뉴로",
        "aircraft" => "항공기",
        "parking brake" => "주차 브레이크",
        // gamepad prompts
        "move" => "이동",
        "press" => "누르기",
        "page" => "페이지",
        "resume" => "계속",
        "panel" => "패널",
        "menu" => "메뉴",
        "respawn" => "리스폰",
        "go somewhere else" => "다른 곳으로",

        // ---- weather
        "DAY" => "낮",
        "SKY" => "하늘",
        "WIND" => "바람",
        "PRESETS" => "프리셋",
        "Clear" => "맑음",
        "Fair" => "갬",
        "Hazy" => "연무",
        "Overcast" => "흐림",
        "Windy" => "바람 많음",
        "Storms" => "폭풍",
        "new" => "삭(그믐)",
        "waxing crescent" => "초승달",
        "first quarter" => "상현달",
        "waxing gibbous" => "차는 볼록달",
        "full" => "보름달",
        "waning gibbous" => "지는 볼록달",
        "last quarter" => "하현달",
        "waning crescent" => "그믐달",
        "SW" => "남서",
        "W" => "서",
        "NW" => "북서",

        // ---- places (cities, one-click places, airports)
        "Mexico City" => "멕시코시티",
        "Havana" => "아바나",
        "London" => "런던",
        "Lisbon" => "리스본",
        "Venice" => "베네치아",
        "Athens" => "아테네",
        "Moscow" => "모스크바",
        "Cairo" => "카이로",
        "Marrakesh" => "마라케시",
        "Cape Town" => "케이프타운",
        "Dubai" => "두바이",
        "Tehran" => "테헤란",
        "Kathmandu" => "카트만두",
        "Singapore" => "싱가포르",
        "Beijing" => "베이징",
        "Seoul" => "서울",
        "Manila" => "마닐라",
        "Yosemite Valley" => "요세미티 계곡",
        "Matterhorn" => "마터호른",
        "Sydney Harbour" => "시드니 하버",
        "Black Forest" => "슈바르츠발트",
        "London Heathrow" => "런던 히드로",
        "St Maarten" => "신트마르턴",

        // ---- globe & minimap city names (menu::CITIES; airport codes stay as codes)
        "Amsterdam" => "암스테르담",
        "Anchorage" => "앵커리지",
        "Auckland" => "오클랜드",
        "Bangkok" => "방콕",
        "Berlin" => "베를린",
        "Bogota" => "보고타",
        "Buenos Aires" => "부에노스아이레스",
        "Chicago" => "시카고",
        "Delhi" => "델리",
        "Denver" => "덴버",
        "Golden Gate" => "금문교",
        "Hong Kong" => "홍콩",
        "Honolulu" => "호놀룰루",
        "Istanbul" => "이스탄불",
        "Jakarta" => "자카르타",
        "Lagos" => "라고스",
        "Lima" => "리마",
        "Los Angeles" => "로스앤젤레스",
        "Madrid" => "마드리드",
        "Manhattan" => "맨해튼",
        "Melbourne" => "멜버른",
        "Miami" => "마이애미",
        "Mumbai" => "뭄바이",
        "Nairobi" => "나이로비",
        "New York" => "뉴욕",
        "Oslo" => "오슬로",
        "Paris" => "파리",
        "Perth" => "퍼스",
        "Queenstown" => "퀸스타운",
        "Reykjavik" => "레이캬비크",
        "Rio de Janeiro" => "리우데자네이루",
        "Rome" => "로마",
        "San Francisco" => "샌프란시스코",
        "Santiago" => "산티아고",
        "Seattle" => "시애틀",
        "Shanghai" => "상하이",
        "Stockholm" => "스톡홀름",
        "Sydney" => "시드니",
        "Tokyo" => "도쿄",
        "Toronto" => "토론토",
        "Vancouver" => "밴쿠버",
        "Zermatt" => "체르마트",

        // ---- main menu & settings (plain strings; the `{...}` ones stay English)
        "Fly" => "비행",
        "FLY" => "비행",
        "Fly again" => "다시 비행",
        "Quit" => "종료",
        "Back" => "뒤로",
        "Cancel" => "취소",
        "Change" => "변경",
        "Assign" => "할당",
        "Default" => "기본값",
        "Connect" => "연결",
        "Disconnect" => "연결 해제",
        "Connected" => "연결됨",
        "Connecting…" => "연결 중…",
        "Connection" => "연결",
        "Refresh" => "새로고침",
        "Server" => "서버",
        "Your name" => "이름",
        "Multiplayer" => "멀티플레이어",
        "Nearby" => "주변",
        "This session" => "이번 세션",
        "All time" => "전체 기간",
        "Always" => "항상",
        "Automatic" => "자동",
        "Automatic: the fastest" => "자동: 가장 빠른 서버",
        "Off" => "끄기",
        "Show" => "표시",
        "Hide" => "숨기기",
        "Invert" => "반전",
        "off" => "꺼짐",
        "on" => "켜짐",
        "offline" => "오프라인",
        "bound" => "지정됨",
        "Not bound" => "미지정",
        "Not connected" => "연결 안 됨",
        "No connection" => "연결 없음",
        "no answer" => "응답 없음",
        "on the ground" => "지상",
        "Flying now" => "비행 중",
        "General" => "일반",
        "Graphics" => "그래픽",
        "Sound" => "소리",
        "Volume" => "음량",
        "Camera" => "카메라",
        "Chase" => "추적 시점",
        "Keyboard" => "키보드",
        "Mouse" => "마우스",
        "Gamepad" => "게임패드",
        "Quality" => "품질",
        "Display" => "화면",
        "Preset" => "프리셋",
        "Scenery" => "조경",
        "Air traffic" => "항공 교통",
        "Aircraft" => "항공기",
        "Mode" => "모드",
        "Arcade mode" => "아케이드 모드",
        "Minimap" => "미니맵",
        "Minimap size" => "미니맵 크기",
        "Minimap style" => "미니맵 스타일",
        "Minimap orientation" => "미니맵 방향",
        "Satellite" => "위성",
        "Satellite imagery" => "위성 영상",
        "Plan" => "지도",
        "North up" => "북쪽 위로",
        "Heading up" => "기수 위로",
        "Landing guidance" => "착륙 유도",
        "Language" => "언어",
        "Interface language" => "인터페이스 언어",
        "HUD and minimap" => "HUD 및 미니맵",
        "HUD in the cockpit" => "조종석 HUD",
        "Live air traffic" => "실시간 항공 교통",
        "Real time" => "실시간",
        "Real weather" => "실제 날씨",
        "Real-world data" => "실제 데이터",
        "Snow on the ground" => "지면 적설",
        "Map and data" => "지도 및 데이터",
        "Map imagery" => "지도 영상",
        "Photorealistic 3D cities" => "포토리얼 3D 도시",
        "Google Maps" => "구글 지도",
        "Google usage" => "구글 사용량",
        "Google API key" => "구글 API 키",
        "Camera and view" => "카메라 및 시점",
        "Cockpit field of view" => "조종석 시야각",
        "Mouse look speed" => "마우스 시점 속도",
        "Connected devices" => "연결된 장치",
        "Joystick axes" => "조이스틱 축",
        "Buttons (joystick, throttle, wheel)" => "버튼 (조이스틱, 스로틀, 휠)",
        "Dead zone: how much travel does nothing" => "데드존: 반응하지 않는 이동 범위",
        "Key reference" => "키 안내",
        "Reset all bindings" => "모든 키 할당 초기화",
        "Remove its bindings" => "이 키 할당 제거",
        "Back to the pause menu" => "일시정지 메뉴로",
        "Apply a preset…" => "프리셋 적용…",
        "Other address…" => "다른 주소…",
        "Pick a place on the globe and take off" => "지구본에서 장소를 골라 이륙",
        "approach arrows and tunnel" => "접근 화살표와 터널",
        "key H in flight" => "비행 중 H 키",
        "key U in flight" => "비행 중 U 키",
        "key ` (left of 1) in flight" => "비행 중 ` 키 (1 왼쪽)",
        "it fits the window and keeps this size" => "창에 맞추고 이 크기를 유지",
        "real aircraft around you, from adsb.lol" => "adsb.lol 기반, 주변의 실제 항공기",
        "missiles, boost on any aircraft, respawn at will, weather and time keys" => "미사일, 모든 기체 부스트, 자유 리스폰, 날씨·시간 키",
        "off: the cockpit's own instruments only; key H in the cockpit" => "꺼짐: 조종석 자체 계기만; 조종석에서 H 키",
        // section summaries
        "Language, the HUD, the minimap, live traffic and arcade mode." => "언어, HUD, 미니맵, 실시간 교통, 아케이드 모드.",
        "Graphics · sound · controls" => "그래픽 · 소리 · 조작",
        "Engine, wind, warnings and the world around you." => "엔진, 바람, 경고, 그리고 주변 세계.",
        "Mouse, joysticks, pedals, gamepads and keys." => "마우스, 조이스틱, 페달, 게임패드, 키.",
        "Map imagery, Google's 3D cities and real-world data." => "지도 영상, 구글 3D 도시, 실제 데이터.",
        "Where the camera starts and how wide the cockpit view is." => "카메라 시작 위치와 조종석 시야 폭.",
        "How hard the renderer works. Changes apply at once." => "렌더러 부하 수준. 변경은 즉시 적용됩니다.",
        "Changes are saved as you make them." => "변경 사항은 바꾸는 즉시 저장됩니다.",

        // ---- hangar: aircraft descriptions
        "Twin-engine carrier fighter, 15.5 t. Fly-by-wire holds it to 7.5 g; Space for afterburner, / speedbrake. 135 kt approach." => "쌍발 함재 전투기, 15.5 t. 플라이바이와이어가 7.5 g 로 제한; Space 애프터버너, / 스피드브레이크. 접근 135 kt.",
        "Twin-engine two-seat fighter, 17 t. No fly-by-wire: mind the g and the angle of attack; Space for afterburner, / speedbrakes. 150 kt approach." => "쌍발 복좌 전투기, 17 t. 플라이바이와이어 없음: g 와 받음각 주의; Space 애프터버너, / 스피드브레이크. 접근 150 kt.",
        "Twin-engine stealth fighter, 25 t. Fly-by-wire holds it to 9 g; supersonic without afterburner up high. Space for afterburner, / to brake. 145 kt approach." => "쌍발 스텔스 전투기, 25 t. 플라이바이와이어가 9 g 로 제한; 고고도에선 애프터버너 없이 초음속. Space 애프터버너, / 제동. 접근 145 kt.",
        "Two-seat Mach 3 reconnaissance aircraft, 50 t. Hold Space: it cruises in afterburner, Mach 3.2 at 80,000 ft. Keep under 450 kt indicated on the way up and 3 g always. No flaps; / streams the drag chute once it is down. 175 kt approach." => "복좌 마하 3 정찰기, 50 t. Space 유지: 애프터버너로 순항, 80,000 ft 에서 마하 3.2. 상승 중 지시대기속도 450 kt 미만, 항상 3 g 이내 유지. 플랩 없음; 착지 후 / 로 드래그슈트 전개. 접근 175 kt.",
        "Two-stage rocket, 121 m and 5,000 t on the pad. Throttle up to lift off on 33 engines, then hold W a few seconds to tip it over. 1 picks which engines burn, Backspace lights the ship and drops the booster, 2 has the ship land itself." => "2단 로켓, 발사대에서 121 m · 5,000 t. 스로틀을 올려 33개 엔진으로 이륙, 이어 W 를 몇 초 유지해 기울인다. 1 로 점화 엔진 선택, Backspace 로 상단 점화·부스터 분리, 2 로 자동 착륙.",
        "Four-engine supersonic airliner, 137 t. Mach 2 at 55,000 ft without reheat; Space for reheat to take off. No flaps: F droops the nose (5° to take off, 12.5° to land). Rotate at 200 kt to 10° and no more until it is off (the tail touches at 11.5°); 160 kt approach, 10° nose up. Z reversers." => "4발 초음속 여객기, 137 t. 재연소 없이 55,000 ft 에서 마하 2; 이륙 시 Space 재연소. 플랩 없음: F 로 기수 처짐(이륙 5°, 착륙 12.5°). 200 kt 에서 10° 로 로테이트, 뜰 때까지 그 이상 금지(11.5° 에서 꼬리 접촉); 접근 160 kt, 기수 10° 업. Z 역추진.",
        "Four-seat single, 180 hp, retractable gear. Forgiving and slow: 125 kt flat out." => "4인승 단발, 180 hp, 접이식 착륙장치. 관대하고 느림: 최대 125 kt.",
        "French four-seat wooden tourer and glider tug, 180 hp, fixed gear. Light on the controls; 60° flaps for steep, short landings." => "프랑스제 4인승 목재 투어러 겸 글라이더 예인기, 180 hp, 고정식 착륙장치. 조작이 가볍고; 급강하·짧은 착륙용 60° 플랩.",
        "Four-engine wide-body airliner, 290 t. Heavy and slow to answer the controls; Mach 0.85 cruise, 150 kt approach. / speedbrake, Z reversers." => "4발 광동체 여객기, 290 t. 무겁고 조작 반응이 느림; 순항 마하 0.85, 접근 150 kt. / 스피드브레이크, Z 역추진.",

        // ---- hangar UI
        "Aircraft from" => "항공기 출처",
        "All aircraft" => "모든 항공기",
        "Download more" => "더 받기",
        "Download more aircraft" => "항공기 더 받기",
        "Downloaded" => "받음",
        "Downloaded, but it doesn't load here." => "받았지만 여기서는 로드되지 않음.",
        "can't be flown here" => "여기서는 비행할 수 없음",
        "Can't be flown here yet" => "아직 여기서 비행할 수 없음",
        "couldn't be read" => "읽을 수 없음",
        "From" => "출처",
        "FlightGear's collection" => "FlightGear 컬렉션",
        "FlightGear's collection (FGAddon)" => "FlightGear 컬렉션 (FGAddon)",
        "FlightGear's collection:" => "FlightGear 컬렉션:",
        "Get more…" => "더 받기…",
        "In Fable Flight" => "Fable Flight 수록",
        "IN USE" => "사용 중",
        "installed" => "설치됨",
        "License" => "라이선스",
        "Next aircraft" => "다음 항공기",
        "Previous aircraft" => "이전 항공기",
        "Notes from its authors" => "제작자 노트",
        "Reading FlightGear's aircraft catalogue…" => "FlightGear 항공기 목록 읽는 중…",
        "Remove" => "제거",
        "See all aircraft" => "모든 항공기 보기",
        "Select" => "선택",
        "Select this aircraft" => "이 항공기 선택",
        "Show all" => "전체 보기",
        "search" => "검색",
        "Search" => "검색",
        "Type" => "형식",
        "unpacking" => "압축 해제 중",
        "Unpacking and reading its model…" => "모델 압축 해제·읽는 중…",
        "view its files" => "파일 보기",
        "Waiting for a report…" => "보고 기다리는 중…",
        "Try again" => "다시 시도",
        "Span" => "날개폭",
        "Wingspan" => "날개폭",
        "Wing area" => "날개 면적",
        "Mass" => "질량",
        "Mass, half fuel" => "질량, 연료 절반",
        "Mass, load" => "질량, 탑재",
        "Fuel, full tanks" => "연료, 가득",
        "Top speed" => "최고 속도",
        "Never exceed (IAS)" => "초과금지 속도 (IAS)",
        "Built in" => "제작",

        // ---- flight panel & HUD
        "Back to the globe" => "지구본으로",
        "Cockpit" => "조종석",
        "Damage" => "손상",
        "Engines" => "엔진",
        "Heading" => "방위",
        "Speed" => "속도",
        "Stall" => "실속",
        "Throttle" => "스로틀",
        "Weapons" => "무기",
        "Master arm" => "마스터 암",
        "Seeker" => "탐색기",
        "Climb, speed" => "상승, 속도",
        "Apogee" => "최고 고도",
        "Burnout" => "연소 종료",
        "Downrange" => "수평 거리",
        "Clock speed" => "시간 배속",
        "Menu: pause" => "메뉴: 일시정지",
        "Tab: flight menu" => "Tab: 비행 메뉴",
        "Click or drag to choose your heading" => "클릭하거나 끌어 방위를 선택",

        // ---- weather & sky
        "Sky" => "하늘",
        "Wind" => "바람",
        "Presets" => "프리셋",
        "Cloud base" => "운저",
        "Cloud cover" => "운량",
        "Cloud depth" => "구름 두께",
        "Cloudy" => "구름 많음",
        "Some cloud" => "약간 구름",
        "Mid-level cloud" => "중층운",
        "Storm cells" => "뇌우 셀",
        "Visibility" => "시정",
        "Haze" => "연무",
        "Rain" => "비",
        "Gusts" => "돌풍",
        "no wind" => "무풍",
        "a light breeze" => "약한 바람",
        "a moderate breeze" => "보통 바람",
        "a strong wind" => "강한 바람",
        "a gale" => "강풍",
        "Moon" => "달",
        "Weather & time  (F3)" => "날씨 및 시간  (F3)",
        "the sun where you are, now" => "현재 위치의 실제 태양",

        // ---- time of day
        "Day" => "낮",
        "Night" => "밤",
        "Morning" => "아침",
        "Afternoon" => "오후",
        "Evening" => "저녁",
        "Local time" => "현지 시각",

        // ---- toggles / common
        "On" => "켜기",
        "Map" => "지도",
        "Height" => "높이",
        "no power" => "동력 없음",
        "stopped" => "정지됨",
        "safe" => "안전",

        // ---- start panel (pre-flight) in main.rs
        "Start location" => "시작 위치",
        "Start point" => "시작 지점",
        "Start here" => "여기서 시작",
        "Start there" => "거기서 시작",
        "Start the engine" => "엔진 시동",
        "Start on a final approach" => "최종 접근으로 시작",
        "Starting in the air:" => "공중에서 시작:",
        "Starting on a 3° final approach, gear and flaps down." => "3° 최종 접근으로 시작, 착륙장치·플랩 내림.",
        "Starting on the runway, ready for take-off." => "활주로에서 시작, 이륙 준비.",
        "Starting the engine…" => "엔진 시동 중…",
        "TAKE OFF" => "이륙",
        "Preparing…" => "준비 중…",
        "Take off" => "이륙",
        "Conditions" => "조건",
        "Runway" => "활주로",
        "Final" => "파이널",
        "Final approach" => "최종 접근",
        "Overhead" => "상공",
        "Into the wind" => "맞바람",
        "On the runway" => "활주로에서",
        "real weather" => "실제 날씨",
        "Places" => "장소",
        "Place not found" => "장소를 찾을 수 없음",
        "Search a place" => "장소 검색",
        "Pick a place, or click the globe." => "장소를 고르거나 지구본을 클릭.",
        "Unnamed spot" => "이름 없는 지점",
        "pick a new start" => "새 출발점 선택",
        "City, address, or airport (SFO, EGLL)" => "도시, 주소, 또는 공항 (SFO, EGLL)",
        "Go" => "이동",
        "Airport" => "공항",
        "Airports" => "공항",
        "Airports around the world" => "전 세계 공항",
        "Nearest airports" => "가장 가까운 공항",
        "No airfield within 120 km." => "120 km 내 비행장 없음.",
        "Fly to" => "비행 목적지",
        "Direct to" => "직항",
        "Loading the scenery…" => "지형 불러오는 중…",
        "Pilots nearby" => "주변 조종사",
        "Waiting for aircraft…" => "항공기 기다리는 중…",
        "Live traffic" => "실시간 교통",
        ", unpaved" => ", 비포장",

        // ---- pause / flight menu & status
        "Menu" => "메뉴",
        "Time" => "시간",
        "Resume" => "계속",
        "Paused" => "일시정지",
        "End flight" => "비행 종료",
        "Quit to desktop" => "바탕화면으로 종료",
        "Repair everything" => "전부 수리",
        "Respawn here" => "여기서 리스폰",
        "Respawn in the air" => "공중에서 리스폰",
        "Respawning in flight" => "공중 리스폰 중",
        "Ready" => "준비됨",
        "Controls reference" => "조작 안내",
        "Close (Tab)" => "닫기 (Tab)",
        "Sound on" => "소리 켬",
        "Sound off" => "소리 끔",
        "crashed: R to respawn" => "추락: R 로 리스폰",
        "Enter or click: skip" => "Enter 또는 클릭: 건너뛰기",

        // ---- flight panel: instruments & systems
        "Airspeed" => "대기속도",
        "True airspeed" => "진대기속도",
        "Altitude" => "고도",
        "True altitude" => "진고도",
        "Above ground" => "지면 위",
        "above the ground" => "지면 위",
        "Altimeter" => "고도계",
        "Altimeter reads" => "고도계 지시",
        "on QNH" => "QNH 기준",
        "Vertical speed" => "수직속도",
        "Ground speed, track" => "대지속도, 트랙",
        "Heading indicator set to the compass" => "방위 지시계를 나침반에 맞춤",
        "Angle of attack" => "받음각",
        "Course" => "코스",
        "Thrust" => "추력",
        "Engine" => "엔진",
        "Fuel" => "연료",
        "Fuel flow, endurance" => "연료 유량, 지속시간",
        "Fuel control" => "연료 조절",
        "Flaps" => "플랩",
        "Gear" => "착륙장치",
        "Electrics" => "전기계통",
        "Clouds" => "구름",
        "Payload" => "탑재물",
        "Load" => "탑재",
        "Mass, balance" => "질량, 밸런스",
        "Magnetos, mixture" => "마그네토, 혼합비",
        "Drag chute" => "드래그슈트",
        "Speedbrake" => "스피드 브레이크",
        "Stage" => "스테이지",
        "Instruments on screen (J)" => "화면 계기 (J)",
        "The weather and time keys" => "날씨·시간 키",
        "The autopilot works in flight" => "오토파일럿은 비행 중 작동",

        // ---- failures page
        "FAILURES" => "고장",
        "A failure now and then, without warning" => "가끔, 예고 없이 고장",
        "About every" => "대략 매",
        "AT RANDOM" => "무작위",
        "OUT OF LIMITS" => "한계 초과",
        "AIRCRAFT WRECKED" => "기체 파손",
        "SPLASH" => "착수(수면 충돌)",

        // ---- autopilot / radio / systems toggles
        "Autopilot on" => "오토파일럿 켜짐",
        "Autopilot off" => "오토파일럿 꺼짐",
        "Autopilot off: on the ground" => "오토파일럿 꺼짐: 지상",
        "Autopilot off: stall" => "오토파일럿 꺼짐: 실속",
        "Autopilot off: unusual attitude" => "오토파일럿 꺼짐: 비정상 자세",
        "Battery master on" => "배터리 마스터 켜짐",
        "Battery master off" => "배터리 마스터 꺼짐",
        "master off (Ins)" => "마스터 꺼짐 (Ins)",
        "Magnetos on" => "마그네토 켜짐",
        "Magnetos off" => "마그네토 꺼짐",
        "Starters engaged" => "스타터 작동",
        "Start the engine…" => "엔진 시동…",
        "Shut the engine down" => "엔진 정지",
        "Engine shut down" => "엔진 정지",
        "Engine off (cold start: F4 starts it)" => "엔진 꺼짐 (콜드 스타트: F4 시동)",
        "Fuel control switches: RUN" => "연료 차단 스위치: RUN",
        "Fuel control switches: CUTOFF" => "연료 차단 스위치: CUTOFF",
        "Parking brake set" => "주차 브레이크 설정",
        "Parking brake released" => "주차 브레이크 해제",
        "Canopy open" => "캐노피 열림",
        "Canopy shut" => "캐노피 닫힘",
        "Lower nose / visor" => "기수 / 바이저 내림",
        "Raise nose / visor" => "기수 / 바이저 올림",
        "Line up on the runway" => "활주로에 정렬",
        "Lines up on the runway." => "활주로에 정렬합니다.",
        "ground spoilers" => "지상 스포일러",
        "flight detent" => "비행 디텐트",

        // ---- radio / navigation
        "Tune" => "맞추기",
        "Tuned" => "설정됨",
        "Not tuned." => "미설정.",
        "Not tuned: pick a station below." => "미설정: 아래에서 국을 선택.",
        "armed" => "준비됨",
        "Armed: tune NAV 1 (Radio page)" => "준비됨: NAV 1 설정 (무선 페이지)",
        "on glide path" => "글라이드패스 위",
        "on the centreline" => "센터라인 위",
        "no runway on the map" => "지도에 활주로 없음",
        "no signal" => "신호 없음",

        // ---- weapons page
        "Target locked" => "표적 락온",
        "Master arm on" => "마스터 암 켜짐",
        "Master arm off" => "마스터 암 꺼짐",
        "MASTER ARM OFF" => "마스터 암 꺼짐",
        "NO TARGET" => "표적 없음",
        "NO LOCK" => "락 없음",
        "NO MISSILES" => "미사일 없음",
        "No missiles on this aircraft" => "이 기체에는 미사일 없음",
        "NO WEAPONS ON THIS AIRCRAFT" => "이 기체에는 무기 없음",
        "WEAPONS: NO POWER" => "무기: 전원 없음",
        "Missiles are for arcade mode (Settings › General)" => "미사일은 아케이드 모드 전용 (설정 › 일반)",

        // ---- messages / notes
        "Off (Esc, Settings, Multiplayer)." => "끄기 (Esc, 설정, 멀티플레이어).",
        "Off (Settings, General)." => "끄기 (설정, 일반).",
        "Menu pauses: it cannot be given anything else." => "Menu 는 일시정지 전용입니다.",
        "Some keys have been moved: see Settings › Controls." => "일부 키가 변경됨: 설정 › 조작 참고.",

        // ---- short status words
        "up" => "올림",
        "down" => "내림",
        "left" => "좌",
        "right" => "우",
        "none" => "없음",
        "gone" => "없음",
        "out" => "나감",
        "hold" => "유지",
        "moving" => "이동 중",
        "running" => "작동 중",
        "starting" => "시동 중",
        "packed" => "접힘",
        "retracted" => "접힘",
        "extended" => "전개됨",
        "calm" => "잔잔함",
        "clear" => "맑음",
        "standard" => "표준",
        "Wheel:" => "휠:",
        "Or drag the globe, scroll to zoom, and click where you want to start." => "또는 지구본을 끌고, 휠로 확대·축소하고, 시작할 곳을 클릭하세요.",
        "Or turn the globe (left stick, triggers to zoom) and press X at the crosshair." => "또는 지구본을 돌려(왼쪽 스틱, 트리거로 확대) 조준점에서 X 를 누르세요.",

        _ => return None,
    })
}

/// Damage messages ("LEFT WING TORN OFF (FLUTTER)", "R AILERON GONE",
/// "ENGINE 40%  OIL LEAK") in the current language.
pub fn caps(s: &str) -> String {
    if !is_fr() {
        return s.to_string();
    }
    // The cause, in brackets at the end.
    let (body, why) = match s.strip_suffix(')').and_then(|b| b.rsplit_once(" (")) {
        Some((b, w)) => (b, Some(w)),
        None => (s, None),
    };
    // Which side, at the front.
    let (side, body) = [("LEFT ", 0), ("RIGHT ", 1), ("L ", 0), ("R ", 1)]
        .iter()
        .find_map(|(p, k)| body.strip_prefix(p).map(|b| (Some(*k), b)))
        .unwrap_or((None, body));
    // The longest phrase it starts with.
    let found = CAPS.iter().filter(|(en, ..)| body.starts_with(en)).max_by_key(|(en, ..)| en.len());
    let mut out = match found {
        Some((en, fr, fem)) => {
            let s = match side {
                Some(0) => " GAUCHE",
                Some(_) if *fem => " DROITE",
                Some(_) => " DROIT",
                None => "",
            };
            let rest = body[en.len()..].replace("OIL LEAK", "FUITE D'HUILE");
            format!("{}{rest}", fr.replace(" {s}", s))
        }
        None => s.to_string(),
    };
    if found.is_some() {
        if let Some(w) = why {
            let w = CAUSES.iter().find(|(en, _)| *en == w).map(|(_, fr)| *fr).unwrap_or(w);
            out = format!("{out} ({w})");
        }
    }
    out
}

/// (English, French with `{s}` where the side goes, whether the French noun
/// is feminine — "AILE DROITE" but "VOLET DROIT")
const CAPS: &[(&str, &str, bool)] = &[
    ("WING BUCKLED, HANGING", "AILE {s} FLAMBÉE, PENDANTE", true),
    ("WING BUCKLED", "AILE {s} FLAMBÉE", true),
    ("WING TORN OFF", "AILE {s} ARRACHÉE", true),
    ("WING TORN AWAY", "AILE {s} ARRACHÉE", true),
    ("WING", "AILE {s}", true),
    ("FUEL TANK RUPTURED", "RÉSERVOIR {s} PERCÉ", false),
    ("TANK RUPTURED", "RÉSERVOIR {s} PERCÉ", false),
    ("AILERON HINGE BROKEN", "CHARNIÈRE D'AILERON {s} CASSÉE", false),
    ("AILERON TORN OFF", "AILERON {s} ARRACHÉ", false),
    ("AILERON GONE", "AILERON {s} PERDU", false),
    ("AILERON HANGING", "AILERON {s} PENDANT", false),
    ("FLAP TORN OFF", "VOLET {s} ARRACHÉ", false),
    ("FLAP GONE", "VOLET {s} PERDU", false),
    ("NOSE TORN OFF", "NEZ ARRACHÉ", false),
    ("NOSE GONE", "NEZ PERDU", false),
    ("VISOR TORN OFF", "VISIÈRE ARRACHÉE", true),
    ("VISOR GONE", "VISIÈRE PERDUE", true),
    ("DRAG CHUTE TORN AWAY", "PARACHUTE-FREIN ARRACHÉ", false),
    ("STABILISER TORN OFF", "STABILISATEUR {s} ARRACHÉ", false),
    ("STABILISER GONE", "STABILISATEUR {s} PERDU", false),
    ("ELEVATOR TORN OFF", "PROFONDEUR {s} ARRACHÉE", true),
    ("ELEVATOR GONE", "PROFONDEUR {s} PERDUE", true),
    ("ENGINE POD STRUCK", "NACELLE MOTEUR {s} TOUCHÉE", true),
    ("TAIL BROKEN OFF", "QUEUE ARRACHÉE", true),
    ("BOOSTER SEPARATED", "BOOSTER LARGUÉ", true),
    ("HOT STAGING", "ALLUMAGE À CHAUD DU VAISSEAU", true),
    ("AUTOMATIC LANDING ARMED", "ATTERRISSAGE AUTOMATIQUE ARMÉ", true),
    ("AUTOMATIC LANDING OFF", "ATTERRISSAGE AUTOMATIQUE COUPÉ", true),
    ("LANDED", "POSÉ", true),
    ("STAGE SEPARATION", "SÉPARATION DES ÉTAGES", true),
    ("NO STAGING ON THE GROUND", "PAS DE SÉPARATION AU SOL", true),
    ("BROKEN UP BY THE AIRSTREAM", "DISLOQUÉ PAR L'ÉCOULEMENT", true),
    ("RUDDER TORN OFF", "GOUVERNE DE DIRECTION ARRACHÉE", true),
    ("RUDDER DAMAGED", "GOUVERNE DE DIRECTION ENDOMMAGÉE", true),
    ("RUDDER GONE", "GOUVERNE DE DIRECTION PERDUE", true),
    ("RUDDER", "DIRECTION", true),
    ("FIN BROKEN", "DÉRIVE CASSÉE", true),
    ("FIN", "DÉRIVE", true),
    ("PROPELLER BLADE BROKEN", "PALE D'HÉLICE CASSÉE", true),
    ("PROP BLADE BROKEN", "PALE D'HÉLICE CASSÉE", true),
    ("PROPELLER GONE", "HÉLICE PERDUE", true),
    ("ENGINE SEIZED", "MOTEUR GRIPPÉ", false),
    ("ENGINE DAMAGED, LOSING OIL", "MOTEUR ENDOMMAGÉ, FUITE D'HUILE", false),
    ("ENGINE DAMAGED", "MOTEUR ENDOMMAGÉ", false),
    ("ENGINES FAILED", "MOTEURS EN PANNE", false),
    ("ENGINE FAILED", "MOTEUR EN PANNE", false),
    ("ENGINES DEAD", "MOTEURS HORS SERVICE", false),
    ("ENGINE DEAD", "MOTEUR HORS SERVICE", false),
    ("ENGINES", "MOTEURS", false),
    ("ENGINE", "MOTEUR", false),
    ("OIL ON WINDSHIELD", "HUILE SUR LE PARE-BRISE", true),
    ("WINDSHIELD CRACKED", "PARE-BRISE FISSURÉ", false),
    ("FUSELAGE CRUMPLED", "FUSELAGE ÉCRASÉ", false),
    ("FUSELAGE DENTED", "FUSELAGE CABOSSÉ", false),
    ("MAIN GEAR TORN OFF", "TRAIN PRINCIPAL {s} ARRACHÉ", false),
    ("MAIN GEAR GONE", "TRAIN PRINCIPAL {s} PERDU", false),
    ("WING GEAR TORN OFF", "TRAIN D'AILE {s} ARRACHÉ", false),
    ("WING GEAR GONE", "TRAIN D'AILE {s} PERDU", false),
    ("BODY GEAR TORN OFF", "TRAIN DE FUSELAGE {s} ARRACHÉ", false),
    ("BODY GEAR GONE", "TRAIN DE FUSELAGE {s} PERDU", false),
    ("NOSE GEAR TORN OFF", "TRAIN AVANT ARRACHÉ", false),
    ("NOSE GEAR GONE", "TRAIN AVANT PERDU", false),
];

/// What broke it, in brackets after a damage message.
const CAUSES: &[(&str, &str)] = &[
    ("FLUTTER", "FLOTTEMENT"),
    ("BUILDING", "BÂTIMENT"),
    ("GROUND", "SOL"),
    ("WATER", "EAU"),
    ("TREE", "ARBRE"),
    ("HARD IMPACT", "CHOC VIOLENT"),
    ("HARD LANDING", "ATTERRISSAGE DUR"),
    ("NOSE IMPACT", "CHOC DU NEZ"),
    ("ROLLED OVER", "RETOURNÉ"),
    ("OVER G", "FACTEUR DE CHARGE"),
    ("OVERSPEED", "SURVITESSE"),
    ("AIRFLOW", "ÉCOULEMENT D'AIR"),
];

#[cfg(test)]
mod tests {
    #[test]
    fn de_before_place_names() {
        assert_eq!(de_place("Amsterdam"), "d'Amsterdam");
        assert_eq!(de_place("Évreux"), "d'Évreux");
        assert_eq!(de_place("Paris"), "de Paris");
        assert_eq!(de_place("Le Havre"), "du Havre");
        assert_eq!(de_place("Les Mureaux"), "des Mureaux");
        assert_eq!(de_place("La Rochelle"), "de la Rochelle");
        assert_eq!(de_place("Hambourg"), "de Hambourg");
    }

    use super::*;

    // One test touches the global language, so the others cannot race it.
    #[test]
    fn french() {
        assert_eq!(caps("LEFT WING TORN OFF (FLUTTER)"), "LEFT WING TORN OFF (FLUTTER)");
        set_lang(Lang::Fr);
        assert_eq!(caps("LEFT WING TORN OFF (FLUTTER)"), "AILE GAUCHE ARRACHÉE (FLOTTEMENT)");
        assert_eq!(caps("R AILERON GONE"), "AILERON DROIT PERDU");
        assert_eq!(caps("RIGHT ELEVATOR TORN OFF (BUILDING)"), "PROFONDEUR DROITE ARRACHÉE (BÂTIMENT)");
        assert_eq!(caps("L WING  -1.3 m"), "AILE GAUCHE  -1.3 m");
        assert_eq!(caps("ENGINE 40%  OIL LEAK"), "MOTEUR 40%  FUITE D'HUILE");
        assert_eq!(caps("LEFT MAIN GEAR GONE"), "TRAIN PRINCIPAL GAUCHE PERDU");
        assert_eq!(caps("NOSE GEAR TORN OFF (HARD IMPACT)"), "TRAIN AVANT ARRACHÉ (CHOC VIOLENT)");
        // (Concorde's nose and visor, which its flap lever works)
        assert_eq!(caps("NOSE TORN OFF (OVERSPEED)"), "NEZ ARRACHÉ (SURVITESSE)");
        assert_eq!(caps("VISOR GONE"), "VISIÈRE PERDUE");
        // (the Blackbird's braking parachute)
        assert_eq!(caps("DRAG CHUTE TORN AWAY (OVERSPEED)"), "PARACHUTE-FREIN ARRACHÉ (SURVITESSE)");
        assert_eq!(caps("TAIL BROKEN OFF (OVER G)"), "QUEUE ARRACHÉE (FACTEUR DE CHARGE)");
        assert_eq!(caps("SOMETHING NEW"), "SOMETHING NEW");
        assert_eq!(tr("London"), "Londres");
        assert_eq!(tr("Paris"), "Paris");
        let n = 3;
        assert_eq!(t!("{n} km", "{n} kilomètres", n = n), "3 kilomètres");
        assert_eq!(plural(1, "request", "requests", "requête", "requêtes"), "1 requête");
        assert_eq!(plural(0, "request", "requests", "requête", "requêtes"), "0 requête");
        // Korean: table lookup, with a fall back to the English when missing.
        set_lang(Lang::Ko);
        assert_eq!(tr("London"), "런던");
        assert_eq!(tr("Settings"), "설정");
        assert_eq!(tr("a string with no Korean yet"), "a string with no Korean yet");
        assert_eq!(t!("Weather", "Météo"), "날씨");
        assert_eq!(t!("untranslated here", "non traduit"), "untranslated here");
        assert_eq!(Lang::parse("ko_KR.UTF-8"), Some(Lang::Ko));
        assert_eq!(Lang::Ko.code(), "ko");
        assert_eq!(Lang::Ko.name(), "한국어");
        set_lang(Lang::En);
        assert_eq!(t!("{n} km", "{n} kilomètres", n = n), "3 km");
        assert_eq!(plural(0, "request", "requests", "requête", "requêtes"), "0 requests");
        assert_eq!(Lang::parse("fr_BE.UTF-8"), Some(Lang::Fr));
    }
}
