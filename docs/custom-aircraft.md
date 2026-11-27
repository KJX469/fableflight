# Custom aircraft

You can add an aircraft without writing Rust: a folder with a JSON file that
describes it and, if you like, its own 3D model as a binary glTF (`.glb`).

## Where

Put each aircraft in its own folder inside the custom aircraft folder:

| Platform | Folder |
| --- | --- |
| Linux | `~/.local/share/fablegame/custom/` |
| macOS | `~/Library/Application Support/fablegame/custom/` |
| Windows | `%APPDATA%\fablegame\custom\` |

```text
custom/
  c182/
    aircraft.json
    model.glb       (optional)
    preview.png     (optional, its picture in the aircraft list, 680 x 392)
```

The game reads them when it starts. They show up in the aircraft list with a
**Custom** badge, and answer to `--aircraft custom:<key>` on the command line.
`assets/custom-example/` in the source tree is a complete example (a Cessna
182T on the 172's body); copy it into the folder to try it.

## aircraft.json

Every aircraft is built on one of the built-in types, its **body**. It
borrows the body's cockpit, landing gear, sounds, damage model and handling,
and then takes whatever the file sets. Everything except the names and the
body is optional: leave a figure out to keep the body's.

```json
{
  "name": "Cessna 182T Skylane",
  "key": "c182",
  "blurb": "One line on what it is.",
  "author": "you",
  "license": "CC BY 4.0",
  "body": "c172",
  "model": "model.glb",
  "model_scale": 1.0,
  "model_offset": [0.0, -0.6, 0.4],
  "model_yaw_deg": 0,
  "mass_kg": 1262,
  "fuel_kg": 237,
  "payload_kg": [250, 420],
  "wing_area_m2": 16.2,
  "span_m": 11.0,
  "cd0": 0.031,
  "cl_max": 1.6,
  "power_kw": 172,
  "fuel_flow_kg_h": [8, 41, 41],
  "cruise_kt": 140,
  "approach_kt": 70,
  "vne_kt": 175,
  "flaps_deg": [0, 10, 20, 30],
  "vfe_kt": [175, 140, 100, 100],
  "retractable": false
}
```

| Field | Meaning |
| --- | --- |
| `name`, `key` | What it is called, and its short name (letters, digits, `-`, `_`). |
| `body` | `c172`, `dr400`, `747`, `g650`, `fa18`, `f4`, `f22`, `concorde` or `sr71`. Pick one of about the same size and kind: the landing gear and the points that touch the ground are the body's. |
| `model`, `model_scale`, `model_offset`, `model_yaw_deg` | Its own model (a `.glb` in the folder); the scale; where the model's origin is from the centre of gravity, in metres (+x right, +y up, +z aft); a turn about the vertical. glTF models face +z, and are turned to face ahead. |
| `mass_kg` | Mass with the tanks half full and the usual payload. |
| `fuel_kg`, `fuel_flow_kg_h` | Usable fuel; the burn at idle, full power and in afterburner. |
| `payload_kg` | The usual payload and the most it may carry. |
| `wing_area_m2`, `span_m`, `cd0`, `cl_max` | The wing, the drag with everything up, the best lift (sets the stall). |
| `power_kw` | A piston engine's power. |
| `thrust_kn`, `engines`, `afterburner` | A jet's thrust each, how many, and the afterburner's thrust as a multiple of the dry thrust (1 for none). |
| `cruise_kt`, `approach_kt`, `vne_kt` | Speeds: a start in the air, a start on final, never exceed. |
| `flaps_deg`, `vfe_kt` | The flap notches and the fastest each may be out at. |
| `retractable` | `false` keeps the gear down (a retractable body's gear stops retracting). |

## The model

Export it from Blender (or anything else) as **glTF Binary (.glb)**, with +Y
up. Textures are the materials' base colour textures, embedded in the file;
the base colour factor and vertex colours tint them. Materials named with
`glass`, `canopy`, `window` or `windshield`, or set to blend, are drawn as
glass.

Nodes with these names move, about the node's own origin:

| Node name starts with | Moves with |
| --- | --- |
| `prop`, `rotor`, `fan` | the engine, spinning about the node's z axis |
| `aileron_l`, `aileron_r` | the ailerons, about the node's x axis |
| `elevator` | the elevator |
| `rudder` | the rudder, about the vertical |
| `flap_l`, `flap_r` | the flaps |
| `gear` | the landing gear, folding about the node's x axis |

Everything else is fixed. The cockpit view still shows the body's cockpit.

## Limits

- The flight model is the game's own: these figures tune it, they do not
  replace it with the aircraft's real aerodynamics.
- The gear, the contact points and the cockpit are the body's.
- Only binary glTF is read (no separate `.gltf` and `.bin` files, no Draco compression).
