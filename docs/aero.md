# Surface-element aerodynamics

`src/aero.rs` flies an aircraft on strips: the wing, the tailplane and the fin
cut spanwise, each in its own local airflow, the forces and moments summed
about the centre of gravity. Damping, stability, stalls, wing drop, propeller
slipstream and swirl, and what damage does all come out of the strips rather
than from tuned gains. Coefficients cover only what strips cannot see: the
fuselage's own pitching and yawing moments and side force, and the roll due to
sideslip of the wing-body junction.

An aircraft flies on it when its `Spec` has a `rig`. Without one it keeps the
older whole-aircraft model (`lift_drag` plus the `handling` gains), so aircraft
move over one at a time.

## What a rig takes

Nothing in the code is aircraft-specific: an aircraft is data.

From the `Spec` (already there for every aircraft):

- the wing: `wing_sect` (leading edge, chord, incidence and dihedral at each
  span station), `span`, `wing_area`;
- the aileron span: `aileron`; the flap notches: `flaps`;
- the parasite drag: `cd0` (what the sections' own profile drag does not
  cover is given to the body), and `cd_gear`, `cd_flap`, `spoiler_cd`;
- mass, inertia and the CG (`cg`, `mac` and the loading) — the strips' moments
  are taken about the CG the load puts there, so loading changes the handling;
- the engine: a piston's propeller diameter, for its slipstream and torque.

From the `Rig` (`aero::Rig`):

| field | what |
|---|---|
| `wing_foil` | the wing section: 2-D lift slope, zero-lift angle, cl_max, cd0, cm0 |
| `wing_strips`, `oswald` | strips per half, span efficiency |
| `flap`, `flap_dcl_max` | flap span and chord fraction; extra cl_max fully down |
| `aileron_chord`, `aileron_travel` | chord fraction; travel up, down (rad) |
| `elevons` | tailless (a delta): the trailing edge does pitch and roll |
| `htail`, `htail_foil`, `elevator` | tailplane planform (quarter-chord line, chords, incidence), section, elevator (chord 1 for a stabilator) |
| `trim` | the trim's range as elevator deflection, nose down and up |
| `fin`, `fin_foil`, `rudder` | the same for the fin |
| `fuse` | fuselage coefficients (`cm_a`, `cn_b`, `cl_b`, `cy_b`) |
| `downwash`, `eta_tail` | share of the lifting-line downwash at the tail; tail dynamic pressure ratio |
| `prop_hub`, `swirl` | the propeller's hub and its slipstream's swirl |
| `trim_stab`, `taileron` | the trim moves the whole tailplane; tailplane halves move apart for roll |
| `aileron_chord_m` | elevons' chord in metres along the span (a delta's) |
| `aileron_in`, `outboard_lock` | inboard (all-speed) ailerons; outboard ones only with flaps out |
| `aileron_droop` | ailerons droop with full flaps (rad) |
| `spoiler`, `roll_spoiler` | spoiler panels (span, chord, travel); share of their travel the roll uses |
| `slat_dcl_max` | extra cl_max with the slats out |
| `vortex`, `vortex_burst` | leading-edge vortex lift (Polhamus K_v) on strips swept past 50°; burst angle |
| `cg_from_mac`, `datum` | where the model's origin sits on the MAC, when it is not at the reference CG |
| `cg_mach` | a CG moved with Mach by fuel transfer: (Mach, fraction of MAC) pairs |
| `fcs` | the flight control system (below) |

`fuse` also takes `cl_a`, lift from the body (chines, strakes, LEX).

The aileron and rudder rigging that makes the aircraft fly straight hands off
in the cruise (its fixed tabs and fin offset) is worked out at spawn from
`Spec::cruise`, as is the trim for the speed it starts at.

## Rigging a new aircraft

1. Start from `aero::Rig::derive(spec)`. It builds a rig from what the `Spec`
   already says: the section's lift and stall from the aircraft polar, the
   tailplane and fin from `Spec::tail`'s tips sized by typical tail volume
   coefficients, typical control travel, elevons if the "stabiliser tip" is on
   the wing, a stabilator for an agile jet. The test
   `every_aircraft_can_be_rigged_from_its_spec` flies every aircraft on its
   derived rig.
2. Replace the estimates with the real aircraft's figures: tail planforms from
   drawings, sections, control travel from the type certificate data sheet.
   Record the sources in comments, as `c172::RIG` does.
3. Set `rig: Some(&RIG)` on the `Spec`.
4. Check it: the aircraft's `performance_is_*` test, the handling harness
   (`cargo test --release --bin fablegame modes:: -- --nocapture`) against the
   reference values it prints, and a stall test like
   `a_cessna_172_stalls_where_the_book_says`. Then fly it.

## Flight control system

`aero::Fcs` (start from `Fcs::NONE`) sits between the stick and the surfaces.
None of it is per aircraft: each field is a law or a gain.

| field | what |
|---|---|
| `damp`, `damp_auth`, `damp_q` | pitch, roll and yaw dampers (the yaw one washed out), their authority, and the dynamic pressure above which their gains fall |
| `gear_q`, `gear_pitch` | roll and pedal authority geared down above this dynamic pressure, as a ratio changer does; how much of that the pitch gets |
| `yaw_beta` | sideslip fed to the rudder (a lateral accelerometer) |
| `g_law`, `alpha_max` | fly-by-wire pitch: the stick asks for load factor (up to `Spec::fbw_g`), with an angle-of-attack limit |
| `speed_hold` | makes the g law speed stable (C*U): let go, it returns to the speed it was trimmed at; trim keys and the auto-trim assist move that speed |
| `flare_alpha` | below 50 ft with the gear down, the g law hands over to an angle-of-attack command from what it flew at 50 ft, so a steady pull flares |
| `roll_rate` | fly-by-wire roll: the stick asks for a roll rate |
| `beta_cmd` | fly-by-wire yaw: sideslip held where the pedals ask |

The fly-by-wire laws invert the strips themselves each step (incremental
dynamic inversion): from the angular acceleration the command needs and how
much each surface moves the moment now, they work out the deflection. So a
new fly-by-wire aircraft needs no gains, only its limits. A surface pushed
past the deflection that does the most is brought back until it bites again.
On the ground, or below 40 m/s, the stick moves the surfaces, and the
elevator the g law was holding fades out over a few seconds.

The autopilot flies a g-law aircraft through the same stick: it asks for the
load factor that holds the climb or descent it wants.

## Stall

Each strip stalls on its own: separation follows the section's angle past
its maximum lift, letting go quickly and reattaching slowly. A deflected flap
or control surface raises the section's stall angle by about two thirds of
the angle its deflection is worth (DATCOM's plain flap), and the lift it adds
acts where thin-airfoil theory puts it for its chord (`aero::flap_cm`: 0.17
of the chord behind the quarter chord for a quarter-chord flap, toward
mid-chord for a narrow one). Spoilers spoil the lift behind them without
moving the stall. `Plane::stall_alpha` with a rig is the angle at which the
first wing strip lets go, found once for each configuration (flaps, slats,
Mach) and cached; the stall warning and the autopilot's low-speed protection
use it.

## Trim

With a rig, the trim is a real elevator trim (`Plane::pitch_trim`, -1 nose down
to 1 nose up): the trim keys and the MCP `trim_deg` move it. The auto-trim
assist (Settings, on by default) winds it while the pilot flies the stick, to
where the aircraft holds 1 g hands off at its speed, as the older model's
auto-trim does. With it off, trimming is the pilot's job.

## Checking every aircraft

All eleven built-in aircraft fly on rigs. The tests that hold them to it:

- `every_aircraft_is_statically_stable_and_damped` (modes.rs): at a cruise
  point each has a lift slope, pitch and weathercock stability (a fly-by-wire
  airframe may be slightly relaxed), dihedral effect and damping on all three
  axes, from the strips' own force and moment differences;
- `handling_of_*` (modes.rs): short period, phugoid, Dutch roll, roll rate
  and spiral against MIL-F-8785C Level 1 (class I light aircraft, class III
  transports, class IV fighters);
- each aircraft's `performance_is_*`, take-off, approach and landing tests
  (plane.rs), and the autopilot's tests flying every aircraft (autopilot.rs).

## Limits

- Strip theory: no spanwise flow, no vortex lattice. The 3-D lift slope and
  induced drag come from each surface's aspect ratio.
- The downwash is the lifting-line value, lagged by the time the air takes to
  reach the tail; the slipstream is momentum theory with a fixed swirl ratio.
- Compressibility: the sections' lift slope with Mach, the buffet boundary,
  the aerodynamic centre moving aft past Mach 1 (more behind a subsonic
  leading edge), and the `Spec`'s wave drag. No shock-induced separation.
- A control surface's lift is spread over its strip's chord as 2-D thin
  airfoil theory has it; on a slender delta's long root chord that puts the
  elevons' lift further forward than a lifting-surface method would.
- Estimates still standing (see each rig's comments): most tail areas and
  sections of the jets, the fuselage coefficients, damper gains, and the
  fly-by-wire laws' limits. Known gaps: the 747's static margin comes out
  high (about 36-39%); the Hornet approaches at 12° angle of attack, not the
  real 8.1°, as its leading-edge extensions' vortex lift is not modelled;
  the G650 flies its approach at 5.1° (the book's 5.8°).
