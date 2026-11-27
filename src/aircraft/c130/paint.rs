//! The C-130H's paint schemes: what the aircraft picker offers
//! (`aircraft::liveries`). plane.wgsl paints them (`livery_c130` and the
//! `c130_*` functions after it), the style in `colours[0][3]`; its surface
//! detail (`relief_c130`) and weathering (`weather_c130`) are the same under
//! every scheme. The shader keys everything on the body-frame position, with
//! copies of the `dims` constants it needs (`C130P_*`, each named after the
//! Rust constant; `shader_mirrors_dims` below checks them).
//!
//! The schemes (colours are approximate FS 595 chips, sRGB read off colour
//! charts and linearised: [est]):
//!
//! 0. **AMC grey**, the default: the overall grey that USAF airlift C-130Hs
//!    have worn since the late 1990s (FS 36173 Neutral Gray), low-visibility
//!    markings in Gunship Gray (FS 36118) and black: a darker radome, a flat
//!    black anti-glare panel ahead of the windshield, a black line on the
//!    fuselage in the inboard propellers' plane, black walkway lines on top
//!    of the wing, the star-and-bar in Gunship Gray on the aft fuselage, the
//!    upper left and lower right wing (as the F-4's scheme carries the
//!    insignia of its period). The serial on the fin (`title`) is MADE UP;
//!    there are no unit markings (tail band, base code).
//! 1. **European One**: the "Lizard" wraparound camouflage of the 1980s
//!    (FS 34092 and 34102 greens, FS 36081 grey; some C-130 references give
//!    the earlier FS 36118 for the grey,
//!    https://www.usaf-sig.org/index.php/references/reference/112-color-schemes/143-c-130-hercules-lockheed/130-c-130-european-one-scheme )
//!    with black markings: black radome and anti-glare panel, the insignia in
//!    black outline. Its colours are the FS 595C chips as tabulated (as
//!    approximate sRGB) at https://www.w3schools.com/colors/colors_fs595.asp ,
//!    linearised and scaled by 0.8 (the ratio between the AMC grey's FS 36173
//!    above and the same table's chip, so that the schemes sit on one chart):
//!    34092 a dark grey-green, 34102 a drab olive, 36081 a dark grey, all
//!    three dull and close in value (the scheme's low contrast); an earlier
//!    set read off another chart was lighter and too yellow-green in 34102. The pattern is generated (smooth bands round the airframe), not a
//!    copy of the real pattern drawings; serial made up as above.
//! 2. **Orivante Cargo**: FICTIONAL, the made-up airline of the 747 and the
//!    G650 flying a civil freighter: white over light grey with its teal
//!    cheat line and coral pinstripe, the teal fin with coral arcs, teal
//!    nacelles, the name on the forward fuselage.
//! 3. **Rescue**: FICTIONAL, after the high-visibility search-and-rescue and
//!    polar schemes of the 1960s-70s (light grey, fluorescent red-orange on
//!    the fin, the tail cone band and the wing and tailplane tips, "RESCUE"
//!    titles); no real service's markings, no insignia.
//!
//! Not painted: rubber de-icing boots. The C-130's wing and tail leading
//! edges are anti-iced with engine bleed air inside them, not boots; they are
//! the same paint as the rest of the wing (their removable panels show as
//! seams in the relief).
//!
//! The pictures (`still`) are rendered by the game itself with
//! `assets/aircraft/liveries/stills.sh c130` (with `--graphics high` added:
//! the default preset on integrated graphics is Low, and the other types'
//! pictures show the fuller one); the type's own picture,
//! `assets/aircraft/c130.png`, is the AMC grey scheme shot the same way from
//! higher and further ahead (yaw 145°, pitch 0.6), as the other types' are.
use crate::aircraft::Livery;

pub const LIVERIES: &[Livery] = &[
    // FS 36173 / FS 36118 / black
    Livery { name: "AMC grey", colours: [[0.138, 0.150, 0.159, 0.0], [0.076, 0.089, 0.093, 0.0], [0.014, 0.014, 0.014, 0.0], [0.0; 4]], title: "AF 00-0130", paint: None, swatch: [[0.138, 0.150, 0.159], [0.076, 0.089, 0.093]], still: include_bytes!("../../../assets/aircraft/liveries/c130_amc.png") },
    // FS 34092 #46554F / FS 34102 #565845 / FS 36081 #4E4F54 (sRGB, linearised, x 0.8) / black
    Livery { name: "European One", colours: [[0.049, 0.073, 0.062, 1.0], [0.074, 0.078, 0.048, 0.0], [0.061, 0.062, 0.071, 0.0], [0.014, 0.014, 0.014, 0.0]], title: "AF 00-0131", paint: None, swatch: [[0.049, 0.073, 0.062], [0.074, 0.078, 0.048]], still: include_bytes!("../../../assets/aircraft/liveries/c130_euro1.png") },
    // Orivante's teal and coral (b747::ORIVANTE), light grey, white
    Livery { name: "Orivante Cargo", colours: [[0.004, 0.10, 0.11, 2.0], [0.80, 0.24, 0.10, 0.0], [0.352, 0.366, 0.376, 0.0], [0.80, 0.80, 0.78, 0.0]], title: "ORIVANTE CARGO", paint: None, swatch: [[0.80, 0.80, 0.78], [0.004, 0.10, 0.11]], still: include_bytes!("../../../assets/aircraft/liveries/c130_cargo.png") },
    // light grey (about FS 16473), fluorescent red-orange (about FS 28915), black
    Livery { name: "Rescue", colours: [[0.352, 0.366, 0.376, 3.0], [0.871, 0.091, 0.017, 0.0], [0.014, 0.014, 0.014, 0.0], [0.0; 4]], title: "RESCUE", paint: None, swatch: [[0.352, 0.366, 0.376], [0.871, 0.091, 0.017]], still: include_bytes!("../../../assets/aircraft/liveries/c130_rescue.png") },
];

#[cfg(test)]
mod tests {
    use super::super::dims::*;
    use super::*;

    /// The numbers after `const NAME = ` in plane.wgsl (a scalar or a vecNf).
    fn shader_const(src: &str, name: &str) -> Vec<f32> {
        let key = format!("const {name} = ");
        let at = src.find(&key).unwrap_or_else(|| panic!("plane.wgsl has no {name}")) + key.len();
        let body = &src[at..at + src[at..].find(';').unwrap()];
        let body = body.trim();
        let body = if body.starts_with("vec") { &body[body.find('(').unwrap() + 1..body.len() - 1] } else { body };
        body.split(',').map(|v| v.trim().parse::<f32>().unwrap_or_else(|_| panic!("{name}: {v:?}"))).collect()
    }

    /// plane.wgsl's copies of the dimensions its paint and relief follow.
    #[test]
    fn shader_mirrors_dims() {
        let src = include_str!("../../shaders/plane.wgsl");
        let want: &[(&str, &[f32])] = &[
            ("C130P_D", &[D_RADOME, D_DECK, D_RAMP, D_TAIL]),
            ("C130P_RADOME", &RADOME_JOINT),
            ("C130P_FUSE_H", &[FUSE_TOP_H, FUSE_MID_H, FUSE_BOT_H]),
            ("C130P_FUSE_HALF_W", &[FUSE_HALF_W]),
            ("C130P_WING", &[LE0_D, CR, CT, SPAN]),
            ("C130P_TAPER_X", &[TAPER_X]),
            ("C130P_WING_ROOT", &[WING_ROOT_X, WING_ROOT_H, DIHEDRAL]),
            ("C130P_FLAPS", &[FLAP_IB[0], FLAP_OB[1], XF_FLAP, XF_AIL]),
            ("C130P_AILERON", &AILERON),
            ("C130P_ENG0", &ENG[0]),
            ("C130P_ENG1", &ENG[1]),
            ("C130P_NAC", &[NAC_R, SPINNER_L]),
            ("C130P_EXHAUST", &[EXHAUST_OFF[0], EXHAUST_OFF[1], EXHAUST_D[0], EXHAUST_D[1]]),
            ("C130P_EXT_TANK", &[EXT_TANK[0], EXT_TANK[1], EXT_TANK[2], EXT_TANK[3]]),
            ("C130P_STAB", &[STAB_LE[0], STAB_LE[1], STAB_TE[0], STAB_TE[1]]),
            ("C130P_STAB_SH", &[STAB_SPAN, STAB_H]),
            ("C130P_FIN", &[FIN_LE[0], FIN_LE[1], FIN_TE[0], FIN_TE[1]]),
            ("C130P_FIN_H", &[FIN_ROOT_H, FIN_H, RUDDER_HINGE]),
            ("C130P_CREW_DOOR", &CREW_DOOR),
            ("C130P_PARA_DOOR", &PARA_DOOR),
            ("C130P_RAMP", &[D_RAMP, RAMP_TIP_D, AFT_DOOR_END_D, RAMP_HALF_W]),
            ("C130P_SPON", &[SPON_D[0], SPON_D[1], SPON_XO, SPON_TOP_H]),
            ("C130P_GEAR", &[TRACK * 0.5, NOSE_WHEEL_X, MAIN_TYRE_W, NOSE_TYRE_W]),
        ];
        let mut bad = Vec::new();
        for (name, v) in want {
            let got = shader_const(src, name);
            if got.len() != v.len() || got.iter().zip(v.iter()).any(|(a, b)| (a - b).abs() > 1e-4) {
                bad.push(format!("{name}: plane.wgsl {got:?}, dims {v:?}"));
            }
        }
        assert!(bad.is_empty(), "plane.wgsl's C-130 paint and relief are out of step with c130::dims:\n{}", bad.join("\n"));
    }

    #[test]
    fn schemes_are_drawable() {
        for (i, l) in LIVERIES.iter().enumerate() {
            assert_eq!(l.colours[0][3], i as f32, "{}: style", l.name);
            assert!(l.title.chars().count() <= 15 && l.title.chars().all(|c| c == ' ' || crate::aircraft::b747::deck::glyph(c) != 15), "{}: title {:?}", l.name, l.title);
            assert!(image::load_from_memory(l.still).is_ok(), "{}: picture", l.name);
        }
    }
}
