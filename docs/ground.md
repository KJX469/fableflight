# Ground handling

`src/gear.rs` rolls every aircraft on the same model. An aircraft is only data:
each `aircraft::Leg` plus its `Tyre`.

## Rigging a leg

| Field | What it does |
| --- | --- |
| `at`, `wheel_r`, `travel` | where the tyre meets the ground fully extended, its radius, the strut's stroke |
| `k`, `c` | spring rate where it stands at rest (N/m) and damping (N s/m; for an oleo, at 0.5 m/s) |
| `steer` | share of the pedal steering it follows: 1 a nose wheel, 0 fixed, negative a body bogie steering against it at taxi speeds (747) |
| `brakes`, `brake_mu` | braked, and the brake's most force as a share of the leg's static load |
| `mu`, `roll_mu` | tyre peak friction and rolling resistance on dry concrete |
| `tyre.wheels` | wheels sharing the load (wheel inertia, spin-up) |
| `tyre.pressure` | kPa: aquaplaning speed (9 √psi kt rolling, 7.7 √psi kt spun down) and sinking into soft ground |
| `tyre.caster` | free swivel range (rad); with `steer` set, a steerable tailwheel that breaks out past its springs |
| `tyre.oleo` | gas spring and orifice damping (stiff rebound) instead of a steel spring leg |
| `tyre.antiskid` | brakes ease off short of a lock, and stay off until the wheels spin up at touchdown |

Static load per leg comes from the legs' positions and spring rates (`gear::static_loads`), so
there is nothing to tune per aircraft beyond the data above. Downloaded JSBSim aircraft
(`hangar.rs`) get castering wheels from `max_steer = 360` and tailwheel breakout for steerable
wheels aft of the centre of gravity.

## The ground

`World::surface` reads the loaded airfield (runways paved or turf, aprons concrete, taxiways
asphalt, grass between), falls back to the airport database's runway rectangles before the
airfield has loaded, and is open rough ground elsewhere. Main sets `Plane::ground_wx` each
frame: rain (not snow) falling there and lying snow. Rain builds standing water on the runway
over about two minutes and it dries over about fifteen.

## Checked by

`src/plane/ground_tests.rs`, on every wheeled aircraft: standing still, brakes against run-up
power, braking dry/wet/snow (anti-skid never locks), lock-up without anti-skid, aquaplaning,
grass drag and bumps, touchdown spin-up, takeoff tracking, taxi turns, a tailwheel and a
castering nose wheel.

## Not modelled yet

Autobrakes, brake heat and fade, tyre wear, flat spots and blowouts, ice (freezing rain on
pavement), slush depth beyond a lying-snow flag, side-load gear failure, and a tiller (pedals
steer, as before). Tyre pressures for the fighters, G650, DR400 and 172RG are estimates; the
747's and Concorde's are from Boeing and Michelin data.
