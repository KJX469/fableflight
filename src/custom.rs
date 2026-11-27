//! aircraft from files, no rust needed: a folder in `<data>/fablegame/custom/`
//! with an `aircraft.json` (docs/custom-aircraft.md) and, if it has one, its
//! model as a binary gltf (`.glb`). the json starts from one of the built-in
//! types (its `body`: cockpit, gear, sounds, handling) and sets what differs:
//! names, mass, wing, engines, speeds, flaps. the model's nodes named
//! `prop...`, `aileron_l`, `aileron_r`, `elevator`, `rudder`, `flap_l`,
//! `flap_r` and `gear...` move like ours; everything else is fixed.

use crate::aircraft::{self, Engine, Origin, Spec};
use crate::airframe::{self, Anim, Drive, Input, Mesh, Model, Piece, Role, Vis};
use crate::fgmodel::{Ext, TexSet, M_TEX, P_TEX};
use crate::render::PartVertex;
use glam::{DMat4, DVec3, Mat3, Vec3};
use serde::Deserialize;
use std::path::{Path, PathBuf};

fn one() -> f32 {
    1.0
}

/// what `aircraft.json` holds; every figure but the names is optional.
#[derive(Deserialize, Debug, Default)]
pub struct Def {
    pub name: String,
    pub key: String,
    #[serde(default)]
    pub blurb: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,
    /// the built-in type it is built on (`c172`, `dr400`, `747`, `g650`, `fa18`, `f4`, `f22`, `concorde`, `sr71`)
    pub body: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "one")]
    pub model_scale: f32,
    /// metres in our frame (+x right, +y up, +z aft) from the model's origin to the centre of gravity
    #[serde(default)]
    pub model_offset: [f32; 3],
    /// a turn about the vertical (deg): gltf faces +z, which is turned to face ahead
    #[serde(default)]
    pub model_yaw_deg: f32,
    pub mass_kg: Option<f64>,
    pub fuel_kg: Option<f64>,
    /// usual and most (kg)
    pub payload_kg: Option<[f64; 2]>,
    pub wing_area_m2: Option<f64>,
    pub span_m: Option<f64>,
    pub cd0: Option<f64>,
    pub cl_max: Option<f64>,
    /// a piston's power (kW), a jet's thrust each (kN), how many jets, afterburner thrust as a multiple of dry
    pub power_kw: Option<f64>,
    pub thrust_kn: Option<f64>,
    pub engines: Option<usize>,
    pub afterburner: Option<f64>,
    /// idle, full power, afterburner (kg/h)
    pub fuel_flow_kg_h: Option<[f64; 3]>,
    pub cruise_kt: Option<f64>,
    pub approach_kt: Option<f64>,
    pub vne_kt: Option<f64>,
    pub flaps_deg: Option<Vec<f64>>,
    pub vfe_kt: Option<Vec<f64>>,
    pub retractable: Option<bool>,
}

fn leak<T: ?Sized>(b: Box<T>) -> &'static T {
    Box::leak(b)
}

fn leak_str(s: &str) -> &'static str {
    leak(s.to_string().into_boxed_str())
}

pub fn dir() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("fablegame").join("custom")
}

const KT: f64 = 1.0 / 1.943_84;

/// the type a definition describes, built on its body.
pub fn spec(def: &Def, folder: &Path) -> Result<Spec, String> {
    let body = aircraft::BUILTIN.iter().copied().find(|s| s.keys.iter().any(|k| k.eq_ignore_ascii_case(&def.body))).ok_or_else(|| format!("no built-in type {:?} to build on", def.body))?;
    if body.rocket().is_some() {
        return Err("a rocket can't be a body".into());
    }
    let key = def.key.trim().to_ascii_lowercase();
    if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err("its key must be letters, digits, - or _".into());
    }
    let mut s = body.clone();
    s.name = leak_str(def.name.trim());
    s.keys = leak(vec![leak_str(&format!("custom:{key}"))].into_boxed_slice());
    s.blurb = leak_str(if def.blurb.is_empty() { "An aircraft made from files." } else { &def.blurb });
    s.origin = Some(leak(Box::new(Origin { license: if def.license.is_empty() { "as its folder states".into() } else { def.license.clone() }, author: def.author.clone(), body: body.keys[0] })));
    if let Some(m) = def.mass_kg {
        s.mass = m.max(100.0);
    }
    if let Some(f) = def.fuel_kg {
        s.fuel_kg = f.max(1.0);
    }
    if let Some([u, m]) = def.payload_kg {
        s.payload = [u.min(m), m, s.payload[2]];
    }
    if let Some(a) = def.wing_area_m2 {
        s.wing_area = a.max(1.0);
    }
    if let Some(b) = def.span_m {
        s.b = b.max(1.0);
        s.span = (b * 0.5) as f32;
    }
    if let Some(c) = def.cd0 {
        s.cd0 = c.clamp(0.005, 0.3);
    }
    if let Some(cl) = def.cl_max {
        s.stall_alpha = ((cl - s.cl0) / s.cl_a).clamp(0.08, 0.5);
    }
    match &mut s.engine {
        Engine::Piston(p) => {
            if let Some(kw) = def.power_kw {
                let k = kw * 1000.0 / p.power;
                p.power *= k;
                p.static_thrust *= k.powf(2.0 / 3.0);
            }
        }
        Engine::Fan(f) => {
            if let Some(kn) = def.thrust_kn {
                f.thrust = kn * 1000.0;
            }
            if let Some(n) = def.engines {
                f.n = n.clamp(1, 8);
            }
            if let Some(ab) = def.afterburner {
                f.boost = ab.clamp(1.0, 2.5);
            }
        }
        Engine::Rocket(_) => {}
    }
    if let Some([i, full, ab]) = def.fuel_flow_kg_h {
        s.fuel_flow = [i / 3600.0, full / 3600.0, ab.max(full) / 3600.0];
    }
    if let Some(v) = def.cruise_kt {
        s.cruise = v * KT;
    }
    if let Some(v) = def.approach_kt {
        s.approach.0 = v * KT;
    }
    if let Some(v) = def.vne_kt {
        s.vne = v * KT;
    }
    if let Some(f) = &def.flaps_deg {
        if !f.is_empty() {
            let vfe: Vec<f64> = (0..f.len()).map(|k| def.vfe_kt.as_ref().and_then(|v| v.get(k)).map(|v| v * KT).unwrap_or(*s.vfe.get(k).or(s.vfe.last()).unwrap_or(&80.0))).collect();
            s.flaps = leak(f.clone().into_boxed_slice());
            s.vfe = leak(vfe.into_boxed_slice());
            s.approach.1 = s.approach.1.min(s.max_notch());
        }
    }
    if def.retractable == Some(false) {
        s.gear_rate = 0.0;
    }
    if let Some(m) = &def.model {
        let path = folder.join(m);
        if !path.exists() {
            return Err(format!("its model {} isn't there", path.display()));
        }
        s.ext = Some(leak(Box::new(Ext { dir: folder.to_path_buf(), fg_dir: key.clone(), model: path, origin: Vec3::from(def.model_offset), steer_max: s.steer_max as f32, travel: s.gear.iter().map(|l| l.travel as f32).collect(), span: s.span * 2.0, gltf: [def.model_scale, def.model_yaw_deg] })));
    }
    // its picture, if the folder has one.
    for n in ["preview.png", "preview.jpg"] {
        if let Ok(img) = image::open(folder.join(n)) {
            let img = img.resize_to_fill(680, 392, image::imageops::FilterType::Triangle);
            let mut out = std::io::Cursor::new(Vec::new());
            if img.to_rgba8().write_to(&mut out, image::ImageFormat::Png).is_ok() {
                s.still = leak(out.into_inner().into_boxed_slice());
            }
            break;
        }
    }
    Ok(s)
}

/// every folder in the custom folder, onto the aircraft list (at start-up).
pub fn load_all() {
    let Ok(rd) = std::fs::read_dir(dir()) else { return };
    let mut dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.join("aircraft.json").exists()).collect();
    dirs.sort();
    for d in dirs {
        let r = std::fs::read_to_string(d.join("aircraft.json")).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<Def>(&t).map_err(|e| e.to_string())).and_then(|def| spec(&def, &d));
        match r {
            Ok(s) => {
                log::info!("custom aircraft {}: {}", d.display(), s.name);
                aircraft::register(s);
            }
            Err(e) => log::warn!("custom aircraft {}: {e}", d.display()),
        }
    }
}

// the model, from a binary gltf.

fn accessor(j: &serde_json::Value, bin: &[u8], i: usize, comps: usize) -> Result<Vec<f32>, String> {
    crate::tiles3d::accessor_f32(j, bin, i, comps).map_err(|e| e.to_string())
}

/// what a node's name makes it: a role and how it moves about its own origin.
fn moves(name: &str, origin: Vec3, x_axis: Vec3, fwd: Vec3) -> Option<(Role, Anim, Vec3, Vec3, usize)> {
    let n = name.to_ascii_lowercase();
    let side = |l: bool| if l { 0u8 } else { 1u8 };
    let r = 0.35;
    Some(if n.starts_with("prop") || n.starts_with("rotor") || n.starts_with("fan") {
        (Role::Prop, Anim::hinge(Drive::new(Input::Prop, 1.0)), origin, fwd, airframe::G_PROP)
    } else if n.starts_with("aileron_l") {
        (Role::Aileron(side(true)), Anim::hinge(Drive::new(Input::Aileron(-1.0), r)), origin, x_axis, airframe::G_LWING)
    } else if n.starts_with("aileron_r") {
        (Role::Aileron(side(false)), Anim::hinge(Drive::new(Input::Aileron(1.0), r)), origin, x_axis, airframe::G_RWING)
    } else if n.starts_with("elevator") {
        (Role::Elevator, Anim::hinge(Drive::new(Input::Elevator, r)), origin, x_axis, airframe::G_TAIL)
    } else if n.starts_with("rudder") {
        (Role::Rudder, Anim::hinge(Drive::new(Input::Rudder, r)), origin, Vec3::Y, airframe::G_TAIL)
    } else if n.starts_with("flap_l") {
        (Role::Flap(0), Anim::hinge(Drive::new(Input::Flaps, 0.6)), origin, x_axis, airframe::G_LWING)
    } else if n.starts_with("flap_r") {
        (Role::Flap(1), Anim::hinge(Drive::new(Input::Flaps, 0.6)), origin, x_axis, airframe::G_RWING)
    } else if n.starts_with("gear") {
        (Role::Gear(0), Anim::hinge(Drive::new(Input::Gear, 1.6).offset(-1.6)), origin, x_axis, airframe::G_GEAR)
    } else {
        return None;
    })
}

/// builds a model from a `.glb`: meshes, their colours and base colour textures.
pub fn build_glb(ext: &Ext) -> Result<Model, String> {
    let glb = std::fs::read(&ext.model).map_err(|e| format!("{}: {e}", ext.model.display()))?;
    let u32_at = |o: usize| glb.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).ok_or("short glb");
    if u32_at(0)? != 0x4654_6C67 {
        return Err("not a binary gltf (.glb)".into());
    }
    let (mut json, mut bin): (Option<serde_json::Value>, &[u8]) = (None, &[]);
    let mut o = 12;
    while o + 8 <= glb.len() {
        let (len, ty) = (u32_at(o)? as usize, u32_at(o + 4)?);
        let chunk = glb.get(o + 8..o + 8 + len).ok_or("short chunk")?;
        match ty {
            0x4E4F_534A => json = Some(serde_json::from_slice(chunk).map_err(|e| e.to_string())?),
            0x004E_4942 => bin = chunk,
            _ => {}
        }
        o += 8 + len;
    }
    let j = json.ok_or("glb without json")?;
    // the images, decoded; each material's colour and texture layer.
    let mut images: Vec<image::RgbaImage> = Vec::new();
    let mut image_layer: Vec<i32> = Vec::new();
    for im in j["images"].as_array().cloned().unwrap_or_default() {
        let layer = im["bufferView"].as_u64().and_then(|bv| {
            let v = &j["bufferViews"][bv as usize];
            let off = v["byteOffset"].as_u64().unwrap_or(0) as usize;
            let len = v["byteLength"].as_u64().unwrap_or(0) as usize;
            image::load_from_memory(bin.get(off..off + len)?).ok()
        });
        match layer {
            Some(img) => {
                image_layer.push(images.len() as i32);
                images.push(img.to_rgba8());
            }
            None => image_layer.push(-1),
        }
    }
    let tex_layer = |t: u64| -> i32 { j["textures"][t as usize]["source"].as_u64().and_then(|s| image_layer.get(s as usize).copied()).unwrap_or(-1) };
    let mats: Vec<([f32; 4], i32, f32, bool)> = j["materials"].as_array().cloned().unwrap_or_default().iter().map(|m| {
        let pbr = &m["pbrMetallicRoughness"];
        let f = pbr["baseColorFactor"].as_array().map(|a| [0, 1, 2, 3].map(|k| a.get(k).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32)).unwrap_or([1.0; 4]);
        let tex = pbr["baseColorTexture"]["index"].as_u64().map(tex_layer).unwrap_or(-1);
        let rough = pbr["roughnessFactor"].as_f64().unwrap_or(0.6) as f32;
        let name = m["name"].as_str().unwrap_or("").to_ascii_lowercase();
        let glass = m["alphaMode"].as_str() == Some("BLEND") || ["glass", "canopy", "window", "windshield"].iter().any(|g| name.contains(g));
        (f, tex, rough, glass)
    }).collect();
    let size = match images.len() {
        0 => 4,
        1..=3 => 2048,
        4..=12 => 1024,
        _ => 512,
    };
    let layers: Vec<Vec<u8>> = images.iter().map(|i| image::imageops::resize(i, size, size, image::imageops::FilterType::Triangle).into_raw()).collect();
    // gltf faces +z with +x to its left: turned half round (and by the json's yaw) it faces ahead (-z), scaled, put on the cg.
    let [scale, yaw] = ext.gltf;
    let to_body = DMat4::from_translation(ext.origin.as_dvec3()) * DMat4::from_rotation_y((180.0 + yaw as f64).to_radians()) * DMat4::from_scale(DVec3::splat(scale as f64));
    let mut stack: Vec<(usize, DMat4, Option<usize>)> = j["scenes"][j["scene"].as_u64().unwrap_or(0) as usize]["nodes"].as_array().map(|a| a.iter().filter_map(|n| n.as_u64()).map(|n| (n as usize, to_body, None)).collect()).unwrap_or_default();
    let mut pieces: Vec<Piece> = Vec::new();
    let mut tris = 0usize;
    while let Some((n, parent_m, parent_piece)) = stack.pop() {
        let node = &j["nodes"][n];
        let m = parent_m * crate::tiles3d::node_matrix(node);
        let name = node["name"].as_str().unwrap_or("");
        let origin = m.transform_point3(DVec3::ZERO).as_vec3();
        let x_axis = m.transform_vector3(DVec3::X).normalize_or_zero().as_vec3();
        let fwd = m.transform_vector3(DVec3::Z).normalize_or_zero().as_vec3();
        // a moving part gets its own piece; its children hang from it.
        let mut here = parent_piece;
        if let Some((role, anim, pivot, axis, group)) = moves(name, origin, x_axis, fwd) {
            pieces.push(Piece { hotspot: None, name: "custom part", mesh: Mesh::default(), group, parent: parent_piece, pivot, axis, anim, role, vis: Vis::Always, glass: false, occluder: true });
            here = Some(pieces.len() - 1);
        }
        if let Some(mi) = node["mesh"].as_u64() {
            let nm = Mat3::from_mat4(m.as_mat4()).inverse().transpose();
            for prim in j["meshes"][mi as usize]["primitives"].as_array().cloned().unwrap_or_default() {
                if prim["mode"].as_u64().unwrap_or(4) != 4 {
                    continue;
                }
                let attrs = &prim["attributes"];
                let Some(pi) = attrs["POSITION"].as_u64() else { continue };
                let pos = accessor(&j, bin, pi as usize, 3)?;
                let nrm = attrs["NORMAL"].as_u64().map(|i| accessor(&j, bin, i as usize, 3)).transpose()?;
                let uv = attrs["TEXCOORD_0"].as_u64().map(|i| accessor(&j, bin, i as usize, 2)).transpose()?;
                let col = attrs["COLOR_0"].as_u64().map(|i| accessor(&j, bin, i as usize, 4)).transpose()?;
                let idx: Vec<u32> = match prim["indices"].as_u64() {
                    Some(i) => crate::tiles3d::accessor_idx(&j, bin, i as usize).map_err(|e| e.to_string())?,
                    None => (0..(pos.len() / 3) as u32).collect(),
                };
                let (base, tex, rough, glass) = prim["material"].as_u64().and_then(|k| mats.get(k as usize).copied()).unwrap_or(([0.8, 0.8, 0.8, 1.0], -1, 0.6, false));
                let mut mesh = Mesh::default();
                for k in 0..pos.len() / 3 {
                    let p = m.transform_point3(DVec3::new(pos[3 * k] as f64, pos[3 * k + 1] as f64, pos[3 * k + 2] as f64)).as_vec3();
                    let n = nrm.as_ref().map(|v| (nm * Vec3::new(v[3 * k], v[3 * k + 1], v[3 * k + 2])).normalize_or_zero()).unwrap_or(Vec3::Y);
                    let t = uv.as_ref().map(|v| [v[2 * k], v[2 * k + 1]]).unwrap_or([0.0, 0.0]);
                    let c = col.as_ref().map(|v| [v[4 * k], v[4 * k + 1], v[4 * k + 2], v[4 * k + 3]]).unwrap_or([1.0; 4]);
                    let lin = |x: f32| x.max(0.0);
                    mesh.v.push(PartVertex { pos: p.into(), nrm: n.into(), col: [lin(base[0] * c[0]), lin(base[1] * c[1]), lin(base[2] * c[2]), 1.0], uv: t, mat: [M_TEX, P_TEX, rough.clamp(0.05, 1.0), tex as f32] });
                }
                // the turn may have mirrored it: keep the triangles facing out.
                let flip = m.determinant() < 0.0;
                for t in idx.chunks_exact(3) {
                    if flip { mesh.i.extend([t[0], t[2], t[1]]) } else { mesh.i.extend([t[0], t[1], t[2]]) }
                }
                tris += idx.len() / 3;
                if mesh.v.is_empty() {
                    continue;
                }
                match here {
                    Some(k) if !glass => pieces[k].mesh.append(&mesh),
                    _ => {
                        let c = mesh.v.iter().fold(Vec3::ZERO, |a, v| a + Vec3::from(v.pos)) / mesh.v.len() as f32;
                        let group = if c.x.abs() > ext.span * 0.12 { if c.x < 0.0 { airframe::G_LWING } else { airframe::G_RWING } } else { airframe::G_FUSE };
                        pieces.push(Piece { hotspot: None, name: if glass { "custom glass" } else { "custom mesh" }, mesh, group, parent: here, pivot: Vec3::ZERO, axis: Vec3::X, anim: Anim::Fixed, role: Role::Other, vis: Vis::Always, glass, occluder: !glass });
                    }
                }
            }
        }
        for c in node["children"].as_array().cloned().unwrap_or_default() {
            if let Some(c) = c.as_u64() {
                stack.push((c as usize, m, here));
            }
        }
    }
    if tris == 0 {
        return Err("the model has no triangles".into());
    }
    log::info!("custom model {}: {} pieces, {tris} triangles, {} textures", ext.model.display(), pieces.len(), layers.len());
    let mut model = Model::finish(pieces);
    model.tex = Some(std::sync::Arc::new(TexSet { size, layers }));
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_example_definition_builds_on_its_body() {
        let text = include_str!("../assets/custom-example/aircraft.json");
        let def: Def = serde_json::from_str(text).unwrap();
        let s = spec(&def, Path::new("assets/custom-example")).unwrap();
        assert_eq!(s.keys[0], "custom:c182");
        assert!((s.mass - 1262.0).abs() < 1e-9);
        assert!(s.ext.is_none());
    }
}
