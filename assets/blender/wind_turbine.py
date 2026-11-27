# A modern three-bladed wind turbine for FableFlight, built procedurally in
# Blender, in pieces the game puts together and scales to each turbine
# OpenStreetMap knows (src/turbines.rs):
#
#   blender -b --factory-startup --python assets/blender/wind_turbine.py -- <repo>/assets [--render <dir>]
#
# Saves assets/blender/wind_turbine.blend and exports assets/wind_turbine.bin.gz
# (the format of the aircraft, birds and balloons).
#
# The reference machine is a generic 3 MW onshore turbine of the 2010s
# (Vestas V117 / Nordex N117 class): hub at 100 m, rotor 120 m across.
# Figures: tubular steel tower about 4.3 m across at the base and 2.6 m at the
# top, bolted in sections of 20-30 m; nacelle some 12 m long and 4 m wide
# and high, the rotor 5 m ahead of the yaw bearing and tilted up 5 degrees;
# blades with a round root about 2.6 m across, a widest chord near 4 m at a
# fifth of the span, twisted about 14 degrees from there to the tip, which
# is cut to under a metre. They are typical values from manufacturers'
# brochures and the IEA 3.4 MW reference turbine, not one model's drawings.
# Paint: RAL 7035 light grey; on some (Germany, Denmark) two red bands near
# the blade tips, the obstruction marking for turbines over 100 m.
#
# Pieces, each about its own pivot (Blender +Y is the way the rotor faces,
# into the wind; game frame as for balloons.py):
#   tower          foot at the origin, top (the yaw bearing) at TOWER_TOP
#   nacelle        yaw bearing at the origin; the rotor's hub at ROTOR_AT
#   rotor, rotor bands
#                  hub centre at the origin, turning about +Y
#   transition     an offshore turbine's yellow transition piece and
#                  platform, standing on the sea from the origin
#   tower far, nacelle far, rotor far
#                  a few hundred triangles each, for far off
import bpy, bmesh, math, struct, gzip, sys, os
from mathutils import Vector as V, Matrix

ARGS = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
ASSETS = os.path.abspath(ARGS[0]) if ARGS and not ARGS[0].startswith("--") else os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
RENDER = ARGS[ARGS.index("--render") + 1] if "--render" in ARGS else None

# Reference dimensions (m); src/turbines.rs has the same.
HUB = 100.0
TOWER_TOP = 97.6
ROTOR_AT = (0.0, 5.2, 2.4)
TILT = math.radians(5.0)
R = 60.0
R_BASE, R_TOP = 2.15, 1.30

# Materials: the game's vertex colour alpha (inst.wgsl's vehicle branch):
# 0 paint (the instance's tint: the turbine's grey), 1 plain colour.
PAINT, PLAIN = range(2)
MAT_CODE = [0.0, 1.0]
_mat = [PAINT]


def use(m):
    _mat[0] = m


def c(r, g, b):
    return (r, g, b)


GREY = c(0.62, 0.63, 0.62)
DARK = c(0.08, 0.08, 0.09)
STEEL = c(0.35, 0.36, 0.37)
CONCRETE = c(0.42, 0.41, 0.38)
RED = c(0.55, 0.03, 0.02)
YELLOW = c(0.80, 0.55, 0.02)
GLASS = c(0.05, 0.06, 0.07)


def colour_layer(bm):
    return bm.loops.layers.color.get("Col") or bm.loops.layers.color.new("Col")


def paint(face, layer, col):
    face.material_index = _mat[0]
    for l in face.loops:
        l[layer] = (*col, 1.0)


def make_object(name, bm, coll, at=(0.0, 0.0, 0.0)):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    ob.location = at
    coll.objects.link(ob)
    return ob


def lathe(bm, layer, prof, n, col_of, z0=0.0, centre=(0.0, 0.0), smooth=True, xf=None):
    """A body of revolution about z: prof = [(r, z)], colour per band; xf maps each point."""
    cx, cy = centre
    rings = []
    for (r, z) in prof:
        if r < 1e-6:
            pts = [V((cx, cy, z0 + z))]
        else:
            pts = [V((cx + r * math.cos(2 * math.pi * i / n), cy + r * math.sin(2 * math.pi * i / n), z0 + z)) for i in range(n)]
        if xf:
            pts = [xf(p) for p in pts]
        rings.append([bm.verts.new(p) for p in pts])
    for k in range(len(rings) - 1):
        a, b = rings[k], rings[k + 1]
        for i in range(n):
            j = (i + 1) % n
            if len(a) == 1:
                f = bm.faces.new((a[0], b[i], b[j]))
            elif len(b) == 1:
                f = bm.faces.new((a[i], a[j], b[0]))
            else:
                f = bm.faces.new((a[i], a[j], b[j], b[i]))
            f.smooth = smooth
            paint(f, layer, col_of(k, i))


def box(bm, layer, lo, hi, col, xf=None):
    (x0, y0, z0), (x1, y1, z1) = lo, hi
    p = [V((x, y, z)) for z in (z0, z1) for y in (y0, y1) for x in (x0, x1)]
    if xf:
        p = [xf(q) for q in p]
    v = [bm.verts.new(q) for q in p]
    for f in [(0, 2, 3, 1), (4, 5, 7, 6), (0, 1, 5, 4), (2, 6, 7, 3), (0, 4, 6, 2), (1, 3, 7, 5)]:
        paint(bm.faces.new([v[i] for i in f]), layer, col)


def loft(bm, layer, sections, col_of, closed=True, smooth=True):
    """Faces between successive rings of points (same count each)."""
    rings = [[bm.verts.new(p) for p in s] for s in sections]
    n = len(rings[0])
    for k in range(len(rings) - 1):
        a, b = rings[k], rings[k + 1]
        for i in range(n if closed else n - 1):
            j = (i + 1) % n
            f = bm.faces.new((a[i], a[j], b[j], b[i]))
            f.smooth = smooth
            paint(f, layer, col_of(k, i))
    return rings


def cap(bm, layer, ring, col, flip=False):
    f = bm.faces.new(ring[::-1] if flip else ring)
    paint(f, layer, col)


# ---------------------------------------------------------------- the tower
def tower(coll, far=False):
    bm = bmesh.new()
    layer = colour_layer(bm)
    n = 8 if far else 28
    # Conical, in bolted sections: a flange (a slight ridge) at each joint.
    joints = [] if far else [22.0, 46.0, 72.0]

    def rad(z):
        return R_BASE + (R_TOP - R_BASE) * z / TOWER_TOP
    prof = [(rad(0.0), 0.0)]
    for z in joints:
        prof += [(rad(z - 0.15), z - 0.15), (rad(z) + 0.05, z - 0.08), (rad(z) + 0.05, z + 0.08), (rad(z + 0.15), z + 0.15)]
    prof += [(rad(TOWER_TOP - 0.3), TOWER_TOP - 0.3), (R_TOP + 0.08, TOWER_TOP - 0.25), (R_TOP + 0.08, TOWER_TOP), (0.0, TOWER_TOP)]
    use(PAINT)
    lathe(bm, layer, prof, n, lambda k, i: GREY)
    if not far:
        # Foundation: the top of the concrete slab round the foot, and the
        # anchor-bolt ring.
        use(PLAIN)
        lathe(bm, layer, [(0.0, -0.3), (R_BASE + 3.2, -0.3), (R_BASE + 3.2, 0.05), (R_BASE + 0.6, 0.25), (R_BASE + 0.25, 0.3), (0.0, 0.3)], 28, lambda k, i: CONCRETE, smooth=False)
        # The door: a steel frame on the outside, a few steps up to it.
        a0 = math.radians(-90)
        out = V((math.cos(a0), math.sin(a0), 0.0))
        side = V((-out.y, out.x, 0.0))
        z0 = 1.1
        r0 = rad(z0) + 0.04
        for (w, hgt, dz, col, dr) in [(0.55, 1.25, 0.0, STEEL, 0.06), (0.45, 1.1, 0.07, DARK, 0.08)]:
            corners = [out * (r0 + dr) + side * s * w + V((0, 0, z0 + dz + t * hgt)) for (s, t) in [(-1, 0), (1, 0), (1, 1), (-1, 1)]]
            vs = [bm.verts.new(q) for q in corners]
            paint(bm.faces.new(vs), layer, col)
        for k in range(4):
            zt = 0.3 + 0.2 * k
            box(bm, layer, (-0.6, 0, zt - 0.2), (0.6, 0.35, zt), STEEL, xf=lambda q, k=k: out * (rad(0) + 0.3 + 0.35 * (3 - k) + q.y) + side * q.x + V((0, 0, q.z)))
        # Railing posts of the stair, and a little platform at the door.
        box(bm, layer, (-0.75, 0.0, 0.95), (0.75, 1.1, 1.05), STEEL, xf=lambda q: out * (rad(0) + q.y) + side * q.x + V((0, 0, q.z)))
    return make_object("tower far" if far else "tower", bm, coll)


def transition(coll):
    """Offshore: the yellow transition piece over the monopile, its access
    platform and the boat landing."""
    bm = bmesh.new()
    layer = colour_layer(bm)
    use(PLAIN)
    lathe(bm, layer, [(0.0, -3.0), (3.2, -3.0), (3.2, 16.0), (2.4, 17.0), (0.0, 17.0)], 24, lambda k, i: YELLOW)
    # The platform: a grating ring with a handrail.
    lathe(bm, layer, [(0.0, 16.5), (6.0, 16.5), (6.0, 17.0), (0.0, 17.0)], 24, lambda k, i: c(0.25, 0.25, 0.24), smooth=False)
    for i in range(24):
        a = 2 * math.pi * i / 24
        box(bm, layer, (-0.04, -0.04, 17.0), (0.04, 0.04, 18.1), YELLOW, xf=lambda q, a=a: V((q.x + 5.9 * math.cos(a), q.y + 5.9 * math.sin(a), q.z)))
    lathe(bm, layer, [(5.85, 18.05), (5.95, 18.05), (5.95, 18.15), (5.85, 18.15), (5.85, 18.05)], 24, lambda k, i: YELLOW, smooth=False)
    # The boat landing: two fender tubes down the side, a ladder between.
    for s in (-1, 1):
        box(bm, layer, (s * 0.9 - 0.15, 3.3, -3.0), (s * 0.9 + 0.15, 3.6, 16.5), YELLOW)
    for k in range(40):
        z = -2.5 + 0.47 * k
        box(bm, layer, (-0.85, 3.35, z), (0.85, 3.45, z + 0.05), YELLOW)
    return make_object("transition", bm, coll)


# -------------------------------------------------------------- the nacelle
def nacelle(coll, far=False):
    bm = bmesh.new()
    layer = colour_layer(bm)
    use(PAINT)
    if far:
        box(bm, layer, (-1.9, -8.0, 0.4), (1.9, 3.2, 4.2), GREY)
        return make_object("nacelle far", bm, coll)
    # The housing: a long box with rounded edges, narrowing a little to the
    # back, its front rounded down to the hub.
    def section(y, w, h, z0, rr):
        pts = []
        for (cx, cz, a0) in [(w - rr, h - rr, 0), (-w + rr, h - rr, 90), (-w + rr, z0 + rr, 180), (w - rr, z0 + rr, 270)]:
            for k in range(4):
                a = math.radians(a0 + 90 * k / 3)
                pts.append(V((cx + rr * math.cos(a), y, cz + rr * math.sin(a))))
        return pts
    stations = [(-8.6, 1.75, 4.0, 0.7, 0.5), (-8.5, 1.9, 4.2, 0.45, 0.6), (-4.0, 2.05, 4.45, 0.3, 0.65), (1.5, 2.1, 4.5, 0.3, 0.65), (2.6, 2.0, 4.35, 0.45, 0.7), (3.2, 1.7, 3.9, 0.8, 0.8)]
    secs = [section(*s) for s in stations]
    rings = loft(bm, layer, secs, lambda k, i: GREY)
    cap(bm, layer, rings[0], GREY, flip=False)
    cap(bm, layer, rings[-1], GREY, flip=True)
    # The yaw ring: the nacelle's base on the tower top.
    lathe(bm, layer, [(0.0, 0.0), (R_TOP + 0.05, 0.0), (R_TOP + 0.05, 0.5), (0.0, 0.5)], 24, lambda k, i: GREY)
    # Main shaft housing in front, round, up to the hub.
    use(PAINT)
    hx = lambda q: V((q.x, ROTOR_AT[1] - 2.5 + q.z, ROTOR_AT[2] + q.y))
    lathe(bm, layer, [(0.0, 0.0), (1.55, 0.0), (1.5, 1.2), (1.3, 2.1), (0.0, 2.1)], 24, lambda k, i: GREY, xf=hx)
    # On top at the back: the cooler (fins in a frame), the wind sensors'
    # mast and the two obstruction lights.
    use(PLAIN)
    box(bm, layer, (-1.6, -8.2, 4.5), (1.6, -5.6, 6.2), c(0.5, 0.51, 0.5))
    for k in range(12):
        y = -8.1 + 0.21 * k
        box(bm, layer, (-1.5, y, 4.55), (1.5, y + 0.06, 6.15), DARK)
    box(bm, layer, (-0.05, -7.2, 6.2), (0.05, -7.1, 7.8), STEEL)
    box(bm, layer, (-0.9, -7.2, 7.75), (0.9, -7.1, 7.85), STEEL)
    for s in (-1, 1):
        lathe(bm, layer, [(0.0, 0.0), (0.06, 0.0), (0.06, 0.25), (0.0, 0.25)], 8, lambda k, i: STEEL, z0=7.85, centre=(s * 0.85, -7.15))
        # (The lights: a red dome on a short post, each side.)
        lathe(bm, layer, [(0.0, 4.5), (0.12, 4.5), (0.12, 4.9), (0.18, 5.0), (0.15, 5.15), (0.0, 5.2)], 10, lambda k, i: RED if k >= 2 else STEEL, centre=(s * 1.2, -4.8))
    # Hatches, louvres and the service crane's housing.
    box(bm, layer, (-0.9, -3.0, 4.5), (0.9, -1.0, 4.58), c(0.5, 0.51, 0.5))
    for s in (-1, 1):
        box(bm, layer, (s * 2.1 - 0.02, -6.5, 1.0), (s * 2.1 + 0.02, -4.5, 2.2), DARK)
    return make_object("nacelle", bm, coll)


# ---------------------------------------------------------------- the rotor
def airfoil(chord, thick, twist, n=14):
    """A blade section (x along the chord, y across: the wind's way), its
    pitch axis at a third of the chord; a cambered, NACA-like shape."""
    pts = []
    for k in range(n):
        t = 2 * math.pi * k / n
        # (Cosine spacing: closer at the nose and the tail.)
        u = 0.5 * (1 - math.cos(t))
        x = u
        yt = 5 * thick * (0.2969 * math.sqrt(max(x, 0)) - 0.126 * x - 0.3516 * x * x + 0.2843 * x ** 3 - 0.1036 * x ** 4)
        camber = 0.04 * thick / 0.2 * 4 * x * (1 - x)
        y = camber + (yt if t < math.pi else -yt)
        pts.append(((x - 0.33) * chord, y * chord))
    ca, sa = math.cos(twist), math.sin(twist)
    return [(x * ca - y * sa, x * sa + y * ca) for (x, y) in pts]


def blade_shape(r):
    """Chord, thickness (fraction of chord) and twist (rad) at radius r (m)."""
    s = r / R
    if r < 4.5:
        # The round root, flaring into the profile.
        k = max(0.0, (r - 2.4) / 2.1)
        chord = 2.6 + (3.6 - 2.6) * k
        return chord, 1.0 - 0.55 * k, math.radians(14.0)
    if s < 0.22:
        k = (s - 4.5 / R) / (0.22 - 4.5 / R)
        return 3.6 + 0.4 * math.sin(k * math.pi / 2), 0.45 - 0.2 * k, math.radians(14.0 - 2.0 * k)
    k = (s - 0.22) / 0.78
    chord = 4.0 - 3.15 * k ** 0.85
    thick = 0.25 - 0.07 * k
    twist = math.radians(12.0 * (1 - k) ** 1.6)
    return chord, thick, twist


def rotor(coll, bands=False, far=False):
    bm = bmesh.new()
    layer = colour_layer(bm)
    use(PAINT)
    # Spinner: along +Y (the axis), nose ahead.
    sp = lambda q: V((q.x, q.z, q.y))
    nose = [(0.0, -1.6), (1.9, -1.6), (2.05, -0.4), (1.95, 0.8), (1.6, 1.9), (1.0, 2.8), (0.4, 3.3), (0.0, 3.45)]
    lathe(bm, layer, nose if not far else [(0.0, -1.6), (1.9, -1.6), (1.6, 1.9), (0.0, 3.3)], 8 if far else 24, lambda k, i: GREY, xf=sp)
    radii = [2.4, 3.0, 3.6, 4.5, 6.0, 8.0, 10.5, 13.2, 16.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0, 44.0, 47.5, 50.5, 52.0, 53.5, 55.0, 56.5, 58.0, 59.2, 60.0]
    if far:
        radii = [2.4, 13.2, 40.0, 60.0]
    for b in range(3):
        ang = 2 * math.pi * b / 3

        def place(r, x, y, ang=ang):
            # Radial along the blade's direction in the rotor plane (x, z),
            # chord across it, y along the axis; a little prebend forward
            # (away from the tower) towards the tip.
            pre = 0.9 * (max(r - 20.0, 0.0) / 40.0) ** 2
            radial = V((math.sin(ang), 0.0, math.cos(ang)))
            tang = V((math.cos(ang), 0.0, -math.sin(ang)))
            return radial * r + tang * x + V((0.0, 1.0, 0.0)) * (y + pre)

        def col(k, i, rs=radii):
            if bands:
                # Two red bands of 6 m (and white between) from the tip.
                r = 0.5 * (rs[k] + rs[k + 1])
                if 54.0 <= r <= 60.0 or 42.0 <= r <= 48.0:
                    return RED
            return GREY

        if far:
            secs = []
            for r in radii:
                ch, _, tw = blade_shape(r)
                secs.append([place(r, -0.33 * ch, 0.0), place(r, 0.67 * ch, 0.0)])
            # (A flat strip, a face each side: its own vertices each.)
            for side in (0, 1):
                rings = [[bm.verts.new(p) for p in s] for s in secs]
                for k in range(len(rings) - 1):
                    q = (rings[k][0], rings[k][1], rings[k + 1][1], rings[k + 1][0])
                    paint(bm.faces.new(q if side else q[::-1]), layer, col(k, 0))
            continue
        secs = []
        for r in radii:
            ch, th, tw = blade_shape(r)
            if r < 4.5:
                # Round root, blending to the profile.
                k = max(0.0, (r - 2.4) / 2.1)
                foil = airfoil(ch, max(th, 0.45), tw)
                circ = [(1.3 * math.cos(2 * math.pi * i / 14 + math.pi), 1.3 * math.sin(2 * math.pi * i / 14 + math.pi)) for i in range(14)]
                pts = [((1 - k) * cx + k * fx, (1 - k) * cy + k * fy) for (cx, cy), (fx, fy) in zip(circ, foil)]
            else:
                pts = airfoil(ch, th, tw)
            secs.append([place(r, x, y) for (x, y) in pts])
        rings = loft(bm, layer, secs, col)
        # (The tip closed.)
        cap(bm, layer, rings[-1], col(len(radii) - 2, 0), flip=True)
        # Pitch bearing ring at the root, where the blade meets the hub.
        use(PLAIN)
        lathe(bm, layer, [(1.45, -0.15), (1.45, 0.25)], 20, lambda k, i: c(0.45, 0.46, 0.46),
              xf=lambda q, ang=ang: Matrix.Rotation(ang, 3, "Y") @ V((q.x, q.y, q.z + 2.15)))
        use(PAINT)
    name = "rotor far" if far else ("rotor bands" if bands else "rotor")
    return make_object(name, bm, coll)


# --------------------------------------------------------------------- build
EXPECTED = ["tower", "nacelle", "rotor", "rotor bands", "transition", "tower far", "nacelle far", "rotor far"]


def build():
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    coll = bpy.context.scene.collection
    t = tower(coll)
    n = nacelle(coll)
    n.location = (0, 0, TOWER_TOP)
    r = rotor(coll)
    r.location = (ROTOR_AT[0], ROTOR_AT[1], TOWER_TOP + ROTOR_AT[2])
    r.rotation_euler = (TILT, 0, 0)
    rb = rotor(coll, bands=True)
    rb.location = (150, ROTOR_AT[1], TOWER_TOP + ROTOR_AT[2])
    transition(coll).location = (300, 0, -17)
    tf = tower(coll, far=True)
    tf.location = (-150, 0, 0)
    nacelle(coll, far=True).location = (-150, 0, TOWER_TOP)
    rotor(coll, far=True).location = (-150, ROTOR_AT[1], TOWER_TOP + ROTOR_AT[2])
    tb = tower(coll)
    tb.location = (150, 0, 0)
    tb.name = "_preview tower"
    nb = nacelle(coll)
    nb.location = (150, 0, TOWER_TOP)
    nb.name = "_preview nacelle"


def game(v):
    return (v[0], v[2], -v[1])


def export(path):
    out = bytearray(b"FFAC")
    names = {o.name for o in bpy.data.objects if o.type == "MESH"}
    missing = [n for n in EXPECTED if n not in names]
    if missing:
        raise SystemExit(f"wind turbine export: the game needs these pieces: {', '.join(missing)}")
    obs = [bpy.data.objects[n] for n in EXPECTED]
    out += struct.pack("<II", 1, len(obs))
    total = 0
    for ob in obs:
        me = ob.data
        me.calc_loop_triangles()
        col = me.color_attributes.get("Col")
        verts, index, idx = [], [], {}
        for tri in me.loop_triangles:
            for k in range(3):
                vi, li = tri.vertices[k], tri.loops[k]
                n = V(tri.split_normals[k]).normalized()
                cc = tuple(round(x, 4) for x in col.data[li].color[:3]) if col else (0.5, 0.5, 0.5)
                mcode = MAT_CODE[min(me.polygons[tri.polygon_index].material_index, len(MAT_CODE) - 1)]
                key = (vi, round(n.x, 3), round(n.y, 3), round(n.z, 3), cc, mcode)
                if key not in idx:
                    idx[key] = len(verts)
                    verts.append((*game(me.vertices[vi].co), *game(n), *cc, mcode, 0.0, 0.0, 0.0))
                index.append(idx[key])
        name = ob.name.encode()
        out += struct.pack("<I", len(name)) + name
        out += struct.pack("<7f", 0, 0, 0, 0, 0, 1, 0)
        out += struct.pack("<I", len(verts))
        for v in verts:
            out += struct.pack("<13f", *v)
        out += struct.pack("<I", len(index))
        out += struct.pack("<%dI" % len(index), *index)
        total += len(index) // 3
        print(f"  {ob.name}: {len(index) // 3} triangles, {len(verts)} vertices")
    with gzip.open(path, "wb", compresslevel=9) as f:
        f.write(bytes(out))
    print("wrote", path, len(out), "bytes raw,", total, "triangles")


def render(outdir):
    sc = bpy.context.scene
    for ob in bpy.data.objects:
        if ob.type != "MESH":
            continue
        mat = bpy.data.materials.new(ob.name)
        mat.use_nodes = True
        nt = mat.node_tree
        bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
        attr = nt.nodes.new("ShaderNodeVertexColor")
        attr.layer_name = "Col"
        nt.links.new(attr.outputs[0], bsdf.inputs["Base Color"])
        bsdf.inputs["Roughness"].default_value = 0.45
        ob.data.materials.clear()
        ob.data.materials.append(mat)
        # (Paint faces: the grey the game tints them.)
        for p in ob.data.polygons:
            p.material_index = 0
    sc.world = bpy.data.worlds.new("w")
    sc.world.color = (0.45, 0.55, 0.7)
    sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
    sc.collection.objects.link(sun)
    sun.rotation_euler = (0.9, 0.2, 2.4)
    sun.data.energy = 4
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    sc.collection.objects.link(cam)
    sc.camera = cam
    sc.render.engine = "CYCLES"
    sc.cycles.samples = 24
    sc.render.resolution_x, sc.render.resolution_y = 900, 900
    for name, pos, tgt, lens in [("whole", (220, 160, 70), (0, 0, 70), 35), ("nacelle", (14, 18, 104), (0, 0, 100), 35),
                                 ("blade", (-12, 30, 95), (0, 5, 80), 30), ("foot", (9, -9, 3), (0, 0, 2), 30)]:
        cam.location = pos
        cam.data.lens = lens
        cam.rotation_euler = (V(tgt) - V(pos)).to_track_quat("-Z", "Y").to_euler()
        sc.render.filepath = os.path.join(outdir, f"turbine_{name}.png")
        bpy.ops.render.render(write_still=True)


def main():
    build()
    export(os.path.join(ASSETS, "wind_turbine.bin.gz"))
    bpy.ops.wm.save_as_mainfile(filepath=os.path.join(ASSETS, "blender", "wind_turbine.blend"))
    if RENDER:
        render(RENDER)


main()
