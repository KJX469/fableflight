# Engines and propellers

`src/propulsion/` simulates every engine on its own: a piston engine breathing
through its throttle and turning a propeller, or a gas turbine. Each sits on
its own mount, so an aircraft's engines can be throttled, fail, and pull
asymmetrically one by one. Nothing in the code knows any one type; an
aircraft is only data on its `Spec` (`aircraft::Piston` or `aircraft::Fan`).

| file | what |
|---|---|
| `mod.rs` | the air an engine breathes (`Ambient`), where it is (`Mount`), and building an aircraft's engines from its data (`units`) |
| `piston.rs` | a mean-value piston engine: manifold pressure, mixture, power, fuel flow, EGT, CHT, oil |
| `prop.rs` | a propeller from blade-element theory: thrust, torque, P-factor and normal force, the governor |
| `jet.rs` | a turbofan or turbojet: spools, thrust lapse with height and Mach, afterburner, reversers, fuel flow |
| `unit.rs` | one engine on the airframe: its levers, its state, the force and moment it puts on the aircraft |

All in SI units. The body frame is +X right, +Y up, −Z forward; a mount's
`axis` is the way its thrust pushes.

## Piston engines (`piston.rs`)

A mean-value model, the standard one for flight simulation:

- **Breathing.** The throttle butterfly is an orifice; the cylinders draw
  `VE · ρ_manifold · displacement · rpm / 2`. The manifold pressure is where
  the two flows agree. Full throttle drops about 3% across the butterfly. The
  idle stop holds about a third of an atmosphere at 650 rpm. The volumetric
  efficiency goes with √T, the SAE δ√(T_std/T) correction, so a hot day costs
  power.
- **Mixture.** A carburettor meters fuel by the air's density: the fuel/air
  ratio goes as 1/√ρ. Climbing without leaning runs it rich. The mixture
  lever scales that from full rich down to cut-off. Power against the
  equivalence ratio φ is a table: best power at φ 1.16, rough and dying
  outside about 0.6–2.2.
- **Auto-mixture (on by default).** An assist that holds best power. It runs
  rich (φ 1.26) at full throttle for cooling, and rich at idle as a
  carburettor's idle circuit does. Turning it off hands the lever to the
  pilot.
- **Boost.** `Boost::Turbo { max_map, critical }` holds the manifold
  pressure to a critical height, with about 2 s of turbo lag.
  `Boost::Super { ratio }` is a geared supercharger.
- **Power.** Indicated power is the air flow × fuel/air × heating value ×
  efficiency. The efficiency is worked out from the rating, so rated power
  comes out at rated rpm at sea level on a standard day. Friction
  (`fric0 + fric1·(w/w_rated)²`) and pumping losses come off it.
- **Temperatures.** EGT peaks near stoichiometric and is cooler rich and
  lean. CHT is the heat in against the cooling air (dynamic pressure plus the
  slipstream, the cowl flaps). Oil lags behind both. All three are lagged
  states, so they warm and cool at their own rates.
- **Starting and stopping.** The starter cranks it at about 200 rpm.
  Compression stops a dead engine within seconds unless the air windmills it.

## Propellers (`prop.rs`)

For each blade geometry, a blade-element-momentum analysis builds
thrust and power coefficient tables over the advance ratio J and the blade
angle β at 75% radius. The tables are cached per geometry.

- The analysis uses a Clark-Y section through the stall, Prandtl tip and hub
  losses, and a 1.15 induced-power factor for non-uniform inflow. It also
  applies an installation loss: 3% in flight, rising to 15% standing still,
  where strip theory over-predicts static thrust.
- **Loads.** The disc is taken in 8 sectors, each at its own advance ratio
  from the flow in the disc's plane (sideslip, angle of attack, the aircraft's
  rotation). The descending blade biting harder gives P-factor and the disc's
  normal force. The torque reaction rolls the airframe the other way.
- **Pitch.** `Pitch::Fixed(m)` is a fixed geometric pitch.
  `Pitch::Constant { fine, coarse, rpm, feather }` is a constant-speed
  propeller. Its governor holds the rpm the lever selects with an integral
  law plus rate damping, at about 20°/s at most. `feather` is the feathered
  angle, if it has one.

| data (`aircraft::Piston`) | what |
|---|---|
| `power`, `w_rated`, `litres` | rated power (W) at rated revs (rad/s), swept volume |
| `carb`, `boost`, `cowl_flaps` | carburetted or injected, turbo or supercharger, cowl flaps fitted |
| `prop_d`, `blades`, `chord`, `pitch` | diameter (m), blade count, chord at 75% radius over the diameter, fixed or constant-speed |
| `prop_i` | propeller, crankshaft and flywheel inertia (kg m²) |
| `fric0`, `fric1` | friction and accessories (N m): constant, and extra at rated revs |
| `mounts` | each engine's hub and turning sense (`Mount::ahead(x, y, z, spin)`) |

## Gas turbines (`jet.rs`)

- **Thrust lapse.** These are Mattingly's installed-thrust families (*Elements
  of Propulsion*), blended by bypass ratio:
  - High bypass: `δ0 (1 − k √M)`. Mattingly's k = 0.49 is for a bypass ratio
    near 8. Here it scales as (BPR/8)^¾, because ram drag is a smaller part of
    a faster jet's thrust.
  - Low bypass, mixed flow.
  - Turbojet, dry and wet.

  δ0 and θ0 are the total pressure and temperature ratios. Past the
  engine's temperature limit, `theta_break`, thrust falls away: that is a
  hot-day flat rating for an airliner, and the design Mach for a fast jet.
  Above Mach 1, the intake's total-pressure recovery follows MIL-E-5007D,
  1 − 0.075 (M − 1)^1.35.
- **Inlet fades.** `Spec::inlet_mach` and `Spec::dry_mach` cover a
  turbo-ramjet's inlet limit and a cruise-in-afterburner engine's dry fade.
- **Spools.** Two spools, N2 driving N1. Each engine accelerates on a
  schedule fitted so idle to 95% thrust takes `spool[0]` seconds and coming
  back down takes `spool[1]`. Starting uses the starter, light-off, then a
  run-up. A dead engine windmills.
- **Afterburner.** Light-off follows about 0.35 s after it is asked for. It
  comes to full in about 1.5 s and dies away in 0.5 s.
- **Fuel.** TSFC rises with Mach and falls with temperature from the static
  figures (`tsfc`, lb/lbf/h, dry and wet). Part power costs a little more per
  newton.

| data (`aircraft::Fan`) | what |
|---|---|
| `n`, `thrust` | engine count, sea-level static dry thrust of each (N) |
| `idle`, `idle_n2`, `idle_thrust` | N1 and N2 at idle, idle thrust as a fraction |
| `n1_rpm`, `spool` | fan rpm at 100%; seconds idle → 95% and back |
| `reverse`, `reverser_time` | reverse thrust fraction, deploy time |
| `boost` | afterburning thrust as a multiple of dry (≥ 1.2 for an afterburner) |
| `bpr`, `theta_break`, `tsfc` | bypass ratio, temperature limit θ0, static TSFC dry and wet |
| `nozzle_z`, `mounts` | nozzle station; each engine's nozzle and thrust line |

## On the airframe (`unit.rs`, `plane.rs`)

Each `Unit` takes the cockpit's levers (`Levers`), or its own thrust lever
if it is split (`Unit::lever`), and the air around it (`Flight`, from the
aircraft's real atmosphere). It gives back a force and a moment about the
reference CG.

- **Aircraft with a rig.** The rig is the surface-element model (`aero.rs`).
  These moments are applied as they are: the torque reaction, P-factor, the
  disc's normal force, and an off-centre engine's yaw. The thrust feeds the
  strips' slipstream. `rig_for_cruise` sets the aileron and rudder rigging
  so the aircraft flies straight, hands off, at the power that holds its
  cruise.
- **Gyroscopic moments.** The propellers' spin angular momentum goes into the
  rigid body.
- **Failures.** "Engine failure" fails engine 0 only.
- **Settling.** `Plane::thrust`, `settled_rpm` and `settle_engine` solve the
  steady state, including the governor's blade angle, for spawning in
  flight and for the performance tests.

## Not modelled yet

- Carburettor ice.
- Per-engine levers and hotspots in the cockpits: every engine follows the
  cockpit's levers unless code sets `Unit::lever`.
- A downloaded aircraft flies its engines on its base type's mounts.
