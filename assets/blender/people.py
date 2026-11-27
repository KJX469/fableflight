"""The people in the balloon baskets: MakeHuman characters, textured.

Built with MPFB (the MakeHuman plugin for Blender) from the MakeHuman system
assets: bodies, skins, clothes, hair, eyebrows and eyes, all released as CC0
by the MakeHuman project (Data Collection AB, Joel Palmius, Jonas Hauquier;
http://www.makehumancommunity.org). Each person is posed for a basket
(forearms on the rim, or arms down), cut off below the rim (nobody sees the
legs) and written to assets/balloon_people.bin.gz, a pack in the scenery
format (FFSC, see ebaw/07_game_pack.py) that src/balloons.rs places in the
baskets and scenery.wgsl draws. Hair and eyebrows take the instance's tint
(material flag 8), so each balloon can give its people their own hair colour.

Needs MPFB installed in Blender and the MakeHuman system assets pack (CC0) in
its user data folder:

    http://files.makehumancommunity.org/asset_packs/makehuman_system_assets/makehuman_system_assets_cc0.zip

    blender -b --python assets/blender/people.py -- [--tex 512] [--preview DIR]

Every person faces +X with the feet at the origin (balloons.py: facing 0).
"""

import gzip
import io
import math
import struct
import sys
from importlib import import_module
from pathlib import Path

import addon_utils
import bpy
import numpy as np
from mathutils import Matrix, Vector

HERE = Path(__file__).resolve().parent
OUT = HERE.parent / "balloon_people.bin.gz"

# Who: body (MakeHuman macros), skin, outfit, hair, pose. Pose "rim": the
# forearms forward onto the basket's rim; "down": arms down; "turn" is the
# head's turn to the left (degrees).
PEOPLE = [
    dict(name="pilot", gender=1.0, age=0.6, race="caucasian", weight=0.55, height=0.6, skin="middleage_caucasian_male", clothes="male_casualsuit05", hair="short02", pose="rim", turn=0),
    dict(name="anna", gender=0.0, age=0.4, race="caucasian", weight=0.45, height=0.45, skin="young_caucasian_female", clothes="female_casualsuit01", top=(0.50, 0.07, 0.08), hair="bob02", pose="rim", turn=15),
    dict(name="marc", gender=1.0, age=0.55, race="caucasian", weight=0.6, height=0.55, skin="middleage_caucasian_male", clothes="male_casualsuit02", top=(0.30, 0.30, 0.31), hair="short04", pose="down", turn=-20),
    dict(name="amara", gender=0.0, age=0.45, race="african", weight=0.5, height=0.5, skin="young_african_female", clothes="female_casualsuit01", top=(0.80, 0.56, 0.10), hair="afro01", pose="rim", turn=-10),
    dict(name="kenji", gender=1.0, age=0.45, race="asian", weight=0.45, height=0.4, skin="young_asian_male", clothes="male_casualsuit01", hair="short01", pose="rim", turn=25),
    dict(name="lea", gender=0.0, age=0.35, race="caucasian", weight=0.4, height=0.5, skin="young_caucasian_female2", clothes="female_casualsuit01", top=(0.78, 0.77, 0.74), hair="ponytail01", pose="down", turn=10),
    dict(name="omar", gender=1.0, age=0.6, race="african", weight=0.6, height=0.6, skin="middleage_african_male", clothes="male_casualsuit04", top=(0.05, 0.07, 0.18), hair="short03", pose="rim", turn=-25),
    dict(name="mei", gender=0.0, age=0.6, race="asian", weight=0.5, height=0.35, skin="middleage_asian_female", clothes="female_elegantsuit01", hair="bob01", pose="rim", turn=0),
    dict(name="jan", gender=1.0, age=0.4, race="caucasian", weight=0.5, height=0.7, skin="young_caucasian_male2", clothes="male_casualsuit06", hair="short02", pose="down", turn=30),
    dict(name="rose", gender=0.0, age=0.85, race="caucasian", weight=0.6, height=0.35, skin="old_caucasian_female", clothes="female_casualsuit01", top=(0.62, 0.52, 0.40), hair="short04", pose="rim", turn=-15),
    dict(name="paul", gender=1.0, age=0.85, race="caucasian", weight=0.55, height=0.45, skin="old_caucasian_male", clothes="male_casualsuit03", hair="", hat="fedora01", pose="rim", turn=10),
    dict(name="sofia", gender=0.0, age=0.45, race="caucasian", weight=0.5, height=0.5, skin="middleage_caucasian_female", clothes="female_casualsuit01", top=(0.08, 0.28, 0.14), hair="long01", pose="down", turn=-30),
]
# The MakeHuman T-shirts are all one blue with the project's logo: each
# person's own plain colour instead (`top`), the weave kept. Per outfit, the
# part of the picture left as it is (the trousers): x0, y0, x1, y1, top row 0.
KEEP = {"female_casualsuit01": (0.0, 0.45, 0.72, 1.0), "female_casualsuit02": (0.0, 0.47, 0.72, 1.0),
        "male_casualsuit02": (0.0, 0.415, 0.66, 1.0), "male_casualsuit04": (0.0, 0.415, 0.72, 1.0)}
# (Hidden by the basket: the rim stands ~1.1 m over the floor.)
CUT_Z = 0.72
# The far versions: this share of the triangles (balloons.rs PEOPLE_NEAR).
FAR_RATIO = 0.12
DATA = Path(bpy.utils.resource_path("USER")) / "mpfb" / "data"


def opts():
    a = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    o = {"tex": "512", "preview": "", "dump": ""}
    for i in range(0, len(a) - 1, 2):
        o[a[i].lstrip("-")] = a[i + 1]
    return o


def mpfb():
    mod = next(m.__name__ for m in addon_utils.modules() if m.__name__.endswith("mpfb"))
    addon_utils.enable(mod, default_set=True)
    return import_module(mod + ".services.humanservice").HumanService


def mhmat(path):
    """The textures a MakeHuman material names (absolute paths); `path`, or
    the first material in its folder."""
    path = Path(path)
    if not path.exists():
        path = sorted(path.parent.glob("*.mhmat"))[0]
    out = {}
    for line in Path(path).read_text(errors="replace").splitlines():
        k, _, v = line.strip().partition(" ")
        if k in ("diffuseTexture", "normalmapTexture", "aomapTexture"):
            out[k] = Path(path).parent / v.strip()
    return out


def build(hs, p):
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    female = p["gender"] < 0.5
    race = {"asian": 0.0, "caucasian": 0.0, "african": 0.0}
    race[p["race"]] = 1.0
    clothes = [f"{p['clothes']}/{p['clothes']}.mhclo"] + ([f"{p['hat']}/{p['hat']}.mhclo"] if p.get("hat") else [])
    info = {
        "phenotype": {"gender": p["gender"], "age": p["age"], "muscle": 0.5, "weight": p["weight"], "proportions": 0.5, "height": p["height"],
                      "cupsize": 0.5, "firmness": 0.5, "race": race},
        "rig": "default", "eyes": "low-poly/low-poly.mhclo", "eyebrows": "eyebrow001/eyebrow001.mhclo" if female else "eyebrow006/eyebrow006.mhclo",
        "eyelashes": "", "tongue": "", "teeth": "",
        "hair": f"{p['hair']}/{p['hair']}.mhclo" if p["hair"] else "",
        "proxy": "female1605/female1605.proxy" if female else "male1591/male1591.proxy",
        "clothes": clothes,
        "skin_mhmat": f"{p['skin']}/{p['skin']}.mhmat", "skin_material_type": "MAKESKIN",
        "eyes_material_type": "MAKESKIN", "clothes_material_type": "MAKESKIN", "name": p["name"], "alternative_materials": {},
    }
    s = hs.get_default_deserialization_settings()
    s["subdiv_levels"] = 0
    hs.deserialize_from_dict(info, s)
    pose(bpy.data.objects[p["name"]], p)
    # What each mesh is made of: (object, material key, textures, kind).
    parts = []
    for ob in bpy.data.objects:
        if ob.type != "MESH" or ob.name.endswith(".body"):
            continue
        asset = ob.name.split(".")[-1]
        if asset.startswith(("female1605", "male1591")):
            parts.append((ob, "skin " + p["skin"], mhmat(DATA / "skins" / p["skin"] / f"{p['skin']}.mhmat"), "skin"))
        elif asset == "low-poly":
            parts.append((ob, "eyes", mhmat(DATA / "eyes" / "materials" / "brown.mhmat"), "eyes"))
        elif asset.startswith("eyebrow"):
            parts.append((ob, asset, mhmat(DATA / "eyebrows" / asset / f"{asset}.mhmat"), "hair"))
        elif (DATA / "hair" / asset).is_dir():
            parts.append((ob, asset, mhmat(DATA / "hair" / asset / f"{asset}.mhmat"), "hair"))
        elif (DATA / "clothes" / asset).is_dir():
            top = p.get("top") if asset in KEEP else None
            key = asset + (" %.2f %.2f %.2f" % top if top else "")
            parts.append((ob, key, mhmat(DATA / "clothes" / asset / f"{asset}.mhmat"), ("cloth", asset, top) if top else "cloth"))
        else:
            print("people: what is", ob.name)
    return parts


def pose(arm, p):
    """Arms for a basket, the head turned a little."""
    vl = bpy.context.view_layer
    vl.objects.active = arm
    bpy.ops.object.mode_set(mode="POSE")

    def turn(name, R):
        pb = arm.pose.bones[name]
        vl.update()
        m = pb.matrix.copy()
        pb.matrix = Matrix.Translation(m.translation) @ R @ Matrix.Translation(-m.translation) @ m
        vl.update()

    def aim(name, target):
        pb = arm.pose.bones[name]
        vl.update()
        cur = (arm.matrix_world @ pb.tail - arm.matrix_world @ pb.head).normalized()
        turn(name, cur.rotation_difference(target.normalized()).to_matrix().to_4x4())

    for s in (1, -1):
        side = ".L" if s > 0 else ".R"
        if p["pose"] == "rim":
            aim("upperarm01" + side, Vector((0.08 * s, -0.30, -1.0)))
            aim("lowerarm01" + side, Vector((-0.40 * s, -0.80, 0.05)))
        else:
            aim("upperarm01" + side, Vector((0.16 * s, 0.02, -1.0)))
            aim("lowerarm01" + side, Vector((0.10 * s, -0.25, -1.0)))
    turn("head", Matrix.Rotation(math.radians(p["turn"]), 4, "Z"))
    bpy.ops.object.mode_set(mode="OBJECT")


def layer_images(key, tex, kind, size, cache):
    """A material's two texture layers: base colour + alpha (PNG) and normal
    xy + roughness + metallic (PNG), size x size; shared by key."""
    if key in cache:
        return cache[key]

    def load(path, srgb=True):
        img = bpy.data.images.load(str(path))
        img.colorspace_settings.name = "sRGB" if srgb else "Non-Color"
        img.scale(size, size)
        px = np.array(img.pixels[:], dtype=np.float32).reshape(size, size, 4)
        bpy.data.images.remove(img)
        return px

    kind, recolour = (kind[0], kind[1:]) if isinstance(kind, tuple) else (kind, None)
    base = load(tex["diffuseTexture"])
    if recolour:
        top(base, *recolour)
    if "aomapTexture" in tex:
        ao = load(tex["aomapTexture"], False)
        base[..., :3] *= 0.35 + 0.65 * ao[..., :1]
    if kind != "hair" and kind != "eyes":
        base[..., 3] = 1.0
    if kind == "hair":
        # Grey, as light as each other: the instance's tint gives the colour
        # (x2 in balloons.rs: a tint of 0.5 grey leaves it as it is).
        lum = base[..., :3] @ np.array([0.3, 0.55, 0.15], np.float32)
        on = base[..., 3] > 0.5
        mean = float(lum[on].mean()) if on.any() else 0.5
        base[..., :3] = np.clip(lum / max(mean, 1e-3) * 0.5, 0.0, 1.0)[..., None]
    nrm = load(tex["normalmapTexture"], False) if "normalmapTexture" in tex else np.tile(np.array([0.5, 0.5, 1.0, 1.0], np.float32), (size, size, 1))
    rm = np.zeros((size, size, 4), np.float32)
    rm[..., 0], rm[..., 1] = nrm[..., 0], nrm[..., 1]
    rm[..., 2] = {"skin": 0.55, "cloth": 0.85, "hair": 0.5, "eyes": 0.15}[kind]
    rm[..., 3] = 0.0
    files = []
    # (JPEG for the pictures without see-through parts, PNG for the rest.)
    jpeg = kind not in ("hair", "eyes")
    for px, nm, colour, fmt in ((base, "base", "sRGB", "JPEG" if jpeg else "PNG"), (rm, "nrm", "Non-Color", "PNG")):
        img = bpy.data.images.new(f"{nm}_{len(cache)}", size, size, alpha=True)
        img.colorspace_settings.name = colour
        img.pixels[:] = px.reshape(-1).tolist()
        path = Path(bpy.app.tempdir) / f"people_{nm}.{fmt.lower()}"
        img.filepath_raw = str(path)
        img.file_format = fmt
        if fmt == "PNG":
            img.alpha_mode = "STRAIGHT"
        bpy.context.scene.render.image_settings.quality = 88
        img.save()
        bpy.data.images.remove(img)
        files.append(path.read_bytes())
        path.unlink()
    cache[key] = (len(cache), tuple(files), kind)
    return cache[key]


def top(px, outfit, rgb):
    """A T-shirt in its own plain colour: the weave's light and shade kept,
    the logo and trims gone. px: Blender's pixels (bottom row first)."""
    h, w = px.shape[:2]
    x0, y0, x1, y1 = KEEP[outfit]
    ys, xs = np.mgrid[0:h, 0:w]
    yt = (h - 1 - ys) / h
    keep = (xs / w >= x0) & (xs / w < x1) & (yt >= y0) & (yt < y1)
    r, g, b = px[..., 0], px[..., 1], px[..., 2]
    lum = 0.3 * r + 0.55 * g + 0.15 * b
    mx, mn = px[..., :3].max(-1), px[..., :3].min(-1)
    sat = (mx - mn) / np.maximum(mx, 1e-4)
    blue = (b > r * 1.3) & (b >= g) & (sat > 0.35) & ~keep
    ref = float(np.median(lum[blue])) if blue.any() else 0.3
    # (Not the shirt's blue: logo, trims; as the cloth round them.)
    shade = np.where(blue, lum / ref, 1.0)
    shade = np.clip(shade, 0.4, 1.4)
    for c in range(3):
        px[..., c] = np.where(keep, px[..., c], np.clip(rgb[c] * shade, 0.0, 1.0))


def mesh_blob(name, parts, mat_of, far=False):
    """The person's pieces as one pack mesh, facing +X, feet at the origin,
    cut off below the rim; `far`: a light version (no eyes, a tenth of the
    triangles) for beyond a few tens of metres."""
    mods = []
    if far:
        parts = [pt for pt in parts if pt[3] != "eyes"]
        for ob, _, _, _ in parts:
            md = ob.modifiers.new("far", "DECIMATE")
            md.ratio = FAR_RATIO
            mods.append((ob, md))
    dg = bpy.context.evaluated_depsgraph_get()
    # MakeHuman people face -Y: a quarter turn to the left faces them +X.
    rot = Matrix.Rotation(math.pi / 2, 4, "Z")
    verts, index, idx = [], {}, []

    def q(v):
        return max(-127, min(127, int(round(v * 127.0))))
    for ob, key, _, _ in parts:
        ev = ob.evaluated_get(dg)
        me = bpy.data.meshes.new_from_object(ev)
        me.transform(rot @ ob.matrix_world)
        me.calc_loop_triangles()
        uvl = me.uv_layers.active
        try:
            me.calc_tangents(uvmap=uvl.name)
        except RuntimeError:
            pass
        mat = mat_of[key]
        for tri in me.loop_triangles:
            if max(me.vertices[v].co.z for v in tri.vertices) < CUT_Z:
                continue
            for li in tri.loops:
                lp = me.loops[li]
                co, n, t = me.vertices[lp.vertex_index].co, lp.normal, lp.tangent
                uv = uvl.data[li].uv
                # (Pictures are stored top row first: v down.)
                k = (round(co.x, 4), round(co.y, 4), round(co.z, 4), q(n.x), q(n.y), q(n.z), q(t.x), q(t.y), q(t.z),
                     1 if lp.bitangent_sign >= 0 else -1, round(uv[0], 5), round(1.0 - uv[1], 5), mat)
                vi = index.get(k)
                if vi is None:
                    vi = index[k] = len(verts)
                    verts.append(k)
                idx.append(vi)
        bpy.data.meshes.remove(me)
    for ob, md in mods:
        ob.modifiers.remove(md)
    radius = max(math.sqrt(v[0] ** 2 + v[1] ** 2 + v[2] ** 2) for v in verts)
    b = io.BytesIO()
    nm = name.encode()
    b.write(struct.pack("<I", len(nm)) + nm)
    b.write(struct.pack("<fI", radius, len(verts)))
    for v in verts:
        b.write(struct.pack("<3f4b4b2fI", v[0], v[1], v[2], v[3], v[4], v[5], 127, v[6], v[7], v[8], v[9], v[10], v[11], v[12]))
    b.write(struct.pack("<I", len(idx)))
    b.write(np.array(idx, dtype="<u4").tobytes())
    top = max(v[2] for v in verts)
    return b.getvalue(), len(idx) // 3, top


def preview(folder, name):
    sc = bpy.context.scene
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    sc.collection.objects.link(cam)
    sc.camera = cam
    cam.data.lens = 50
    sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
    sc.collection.objects.link(sun)
    sun.rotation_euler = (0.8, 0.2, 0.6)
    sun.data.energy = 4
    sc.world = bpy.data.worlds.new("w")
    sc.world.color = (0.5, 0.55, 0.6)
    sc.render.engine = "CYCLES"
    sc.cycles.samples = 16
    sc.render.resolution_x, sc.render.resolution_y = 400, 500
    cam.location = (0.8, -2.0, 1.6)
    cam.rotation_euler = (Vector((0, 0, 1.25)) - cam.location).to_track_quat("-Z", "Y").to_euler()
    sc.render.filepath = str(Path(folder) / f"{name}.png")
    bpy.ops.render.render(write_still=True)


def main():
    o = opts()
    size = int(o["tex"])
    hs = mpfb()
    cache, mats, mat_index, blobs, far_blobs, total = {}, [], {}, [], [], 0
    for p in PEOPLE:
        parts = build(hs, p)
        if o["preview"]:
            preview(o["preview"], p["name"])
        for _, key, tex, kind in parts:
            if key not in mat_index:
                layer, _, _ = layer_images(key, tex, kind, size, cache)
                kind = kind[0] if isinstance(kind, tuple) else kind
                flags = (2 | 8) if kind == "hair" else 0
                rough = {"skin": 0.55, "cloth": 0.85, "hair": 0.5, "eyes": 0.15}[kind]
                mat_index[key] = len(mats)
                mats.append(struct.pack("<4f2f3ff2fiIf", 1.0, 1.0, 1.0, 1.0, rough, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, layer, flags, 0.0))
        blob, tris, top = mesh_blob(p["name"], parts, mat_index)
        blobs.append(blob)
        fblob, ftris, _ = mesh_blob(p["name"] + " far", parts, mat_index, far=True)
        far_blobs.append(fblob)
        total += tris + ftris
        print(f"  {p['name']}: {tris} triangles ({ftris} far), {top:.2f} m tall above the cut")
    b = io.BytesIO()
    b.write(b"FFSC" + struct.pack("<I", 3))
    b.write(struct.pack("<ddfI", 0.0, 0.0, 0.0, 0))
    layers = sorted(cache.values())
    if o["dump"]:
        for k, (base, _), _ in layers:
            (Path(o["dump"]) / f"layer{k:02d}.png").write_bytes(base)
    b.write(struct.pack("<II", size, len(layers)))
    for _, (base, nrm), _ in layers:
        b.write(struct.pack("<I", len(base)) + base)
        b.write(struct.pack("<I", len(nrm)) + nrm)
    b.write(struct.pack("<I", len(mats)) + b"".join(mats))
    # (The people, then their far versions in the same order.)
    b.write(struct.pack("<I", 2 * len(blobs)) + b"".join(blobs) + b"".join(far_blobs))
    # (No instances, floodlights, masks or ground patches: the game places them.)
    b.write(struct.pack("<IIII", 0, 0, 0, 0))
    raw = b.getvalue()
    OUT.write_bytes(gzip.compress(raw, 9))
    print(f"wrote {OUT}: {len(PEOPLE)} people, {len(layers)} texture layers, {total} triangles, {len(raw)} bytes raw")


main()
