# JSBSim in FableFlight

FableFlight flies every aircraft with its own handling model (`src/plane.rs`).
JSBSim comes in three ways today:

1. **Reading JSBSim aircraft.** `src/jsbsim.rs` reads JSBSim's XML flight
   models directly: metrics, mass and inertia, control surface travel and the
   aerodynamic functions, evaluated the way JSBSim evaluates them. Nothing of
   JSBSim's code is used; the file format is the documented one.
2. **Flying FlightGear aircraft.** The hangar (`src/hangar.rs`) downloads
   FlightGear aircraft, or imports them from a folder on your computer, and
   *fits* our model to their JSBSim (or YASim) files: mass, inertia, wing,
   lift and drag curves, flaps, engines and fuel.
3. **Checking our aircraft.** `fablegame --jsbsim-compare path/to/aircraft.xml
   --aircraft c172` prints our lift, drag and moments beside the JSBSim
   file's over a sweep of angles of attack and speeds.

## The JSBSim Rust wrapper (issue #30)

[jsbsim-ffi](https://github.com/jonititan/jsbsim-ffi) binds JSBSim's C++
library to Rust. Running real JSBSim inside the game would mean:

- building JSBSim's C++ library (CMake) for each of the six release targets
  (Windows, macOS and Linux, x86-64 and ARM), which the release workflow
  does not do today;
- handing JSBSim the game's terrain under each contact point every step
  (JSBSim's ground callback), and its wind, gusts and the weather;
- keeping our damage model, cockpits, sounds and the clickable controls in
  step with JSBSim's own property tree.

That is a large piece of work for every aircraft, and it would make building
from source much harder. So the wrapper is not bundled. The comparison tool
above is where it would pay off first: a `jsbsim-ffi` cargo feature that runs
the same JSBSim file in real JSBSim, to check `src/jsbsim.rs`'s readings and
our fits against it. That is left for a contributor with JSBSim built on
their machine.
