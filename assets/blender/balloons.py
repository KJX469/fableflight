# Hot-air balloons for FableFlight, built procedurally in Blender: an
# envelope in eight fictional colour schemes, the skirt, the flying wires,
# two rigs (basket, burner frame, burners, passengers) and the burner flame.
#
#   blender -b --factory-startup --python assets/blender/balloons.py -- <repo>/assets [--render <dir>]
#
# Saves assets/blender/balloons.blend and exports assets/balloons.bin.gz (the
# same file format as the aircraft and the birds; read by src/balloons.rs).
# The rebuild starts from scratch, so it throws away anything edited by hand
# in the .blend. To keep hand edits, export the .blend as it stands instead:
#
#   blender -b assets/blender/balloons.blend --python assets/blender/balloons.py -- <repo>/assets --export-only
#
# Two sizes share the envelope (the large one is the same mesh scaled by
# LARGE_SCALE in the game):
#
# - a sport balloon, after the Cameron O-105 (105,000 ft³, about 3,000 m³):
#   envelope 64 ft (19.5 m) high and 61 ft (18.6 m) across, from Cameron's
#   O-type table (https://cameronballoons.com/products/hot-air-balloons/envelopes/o-type);
#   a four-person basket.
# - a passenger balloon, after the Ultramagic N-425 (12,000 m³): 31.2 m high,
#   29.5 m across, 28 gores, 20 passengers (Ultramagic catalogue; the flight
#   manual gives 35.5 m overall). The envelope here is the sport one scaled by
#   1.6 (31.2 / 19.5), with 24 gores rather than 28: a difference nobody sees
#   from another aircraft.
#
# Estimates, from photographs of balloons at launch and in flight (no maker's
# drawings were used): 24 gores on the sport envelope (common on modern
# envelopes; Cameron's O-type table lists 12), the natural shape's profile
# (widest at about 62% of the height, a throat about 21% of the diameter),
# 14 rows of panels, the skirt (scoop) about 1.3 m deep, the basket sizes
# (sport 1.25 x 1.55 x 1.12 m; passenger 4.6 x 2.2 x 1.25 m, a pilot's
# compartment in the middle and four for passengers), the heights of the
# burner frames and of the mouth above the basket floor, and the flame.
#
# The colour schemes are fictional, in the spirit of balloons seen at
# festivals (rainbow gores, harlequin blocks, bands, chevrons, checks, a
# wave, stripes, blocks on yellow). No real operator's markings or logos.
#
# Pieces:
#   "env <scheme>"       the envelope, near (24 gores with load tapes)
#   "env <scheme> far"   the same, coarse, for far away
#   "skirt"              the scoop below the mouth (dark, flame-proof fabric)
#   "wires sport"        flying wires from the mouth to the sport burner frame
#   "wires large"        the same for the passenger balloon (its own lengths)
#   "rig sport"          basket, padding, poles, burner frame, double burner,
#                        cylinders, three people
#   "rig large"          passenger basket, quad burner, pilot and passengers
#   "rig far"            a plain basket for far away
#   "flame"              the burner flame, 1 m nominal, nozzle at the origin
# Origins: the envelope, the skirt and the far envelope at the middle of the
# mouth; the rigs and wires at the middle of the basket floor; the flame at
# the burner nozzle. Custom property "pivot" on each rig: its burner nozzle
# (where the flame goes), and "axis": the mouth's position (x, y, z) above
# the basket floor, for src/balloons.rs to check its constants against.
#
# Colours: a "Col" colour attribute on the face corners (linear albedo).
#
# Blender frame: +X right, +Y forward, +Z up, metres. The game's frame is +X
# right, +Y up, -Z forward, so a point (x, y, z) here is (x, z, -y) there.
import bpy, bmesh, math, struct, gzip, sys, os, json
from mathutils import Vector as V

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
ASSETS = os.path.abspath(argv[0]) if argv else os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RENDER = argv[argv.index("--render") + 1] if "--render" in argv else None
EXPORT_ONLY = "--export-only" in argv

# ---------------------------------------------------------------- dimensions
H = 19.5            # envelope, mouth to crown (m)
R = 9.3             # envelope radius at its widest
GORES = 24
ROWS = 14           # rows of panels
LARGE_SCALE = 1.6

# The natural shape: (height share, radius share), mouth to crown.
PROFILE = [
    (0.00, 0.210), (0.06, 0.320), (0.14, 0.480), (0.24, 0.650), (0.34, 0.790),
    (0.44, 0.900), (0.53, 0.968), (0.62, 1.000), (0.70, 0.992), (0.77, 0.955),
    (0.84, 0.895), (0.90, 0.790), (0.945, 0.645), (0.975, 0.460), (0.992, 0.260),
    (1.000, 0.000),
]

# Sport rig (basket floor at z = 0).
S_BASKET = (1.25, 1.55, 1.12)       # width (x), length (y), height
S_FRAME_Z = 2.45                    # burner frame
S_NOZZLE_Z = 2.82                   # top of the burner coils
S_MOUTH_Z = 4.55                    # mouth (throat) above the basket floor
# Passenger rig.
L_BASKET = (2.2, 4.6, 1.25)
L_FRAME_Z = 2.95
L_NOZZLE_Z = 3.35
L_MOUTH_Z = 6.2

# --------------------------------------------------------------- colours (linear)
def c(r, g, b):
    return (r, g, b)
RED = c(0.55, 0.025, 0.02)
ORANGE = c(0.78, 0.20, 0.012)
YELLOW = c(0.82, 0.56, 0.02)
LIME = c(0.36, 0.62, 0.03)
GREEN = c(0.03, 0.30, 0.06)
SKY = c(0.04, 0.30, 0.66)
BLUE = c(0.02, 0.10, 0.50)
NAVY = c(0.010, 0.020, 0.10)
PURPLE = c(0.13, 0.025, 0.28)
PINK = c(0.75, 0.07, 0.24)
CREAM = c(0.78, 0.64, 0.36)
WHITE = c(0.76, 0.76, 0.73)
BLACK = c(0.018, 0.018, 0.022)
TEAL = c(0.02, 0.32, 0.32)
WICKER = c(0.60, 0.43, 0.20)
WICKER_DK = c(0.27, 0.17, 0.07)
LEATHER = c(0.10, 0.055, 0.03)
SUEDE = c(0.20, 0.10, 0.05)
STEEL = c(0.50, 0.50, 0.52)
DARK_STEEL = c(0.10, 0.10, 0.11)
NOMEX = c(0.045, 0.045, 0.05)
FLOOR = c(0.06, 0.05, 0.04)

RAINBOW = [RED, ORANGE, YELLOW, GREEN, BLUE, PURPLE]


def scheme_colour(scheme, g, k):
    """Panel colour of gore g (0..GORES-1), row k (0 at the mouth .. ROWS-1 at the crown)."""
    top = k >= ROWS - 1
    if scheme == "rainbow":
        return RAINBOW[g % 6]
    if scheme == "harlequin":
        # Diagonal steps of colour over a dark lower part (blocks offset gore by gore).
        if k < 4:
            return NAVY if (g % 2 == 0) else BLACK
        if k == 4:
            return BLUE if g % 2 == 0 else NAVY
        pal = [YELLOW, CREAM, ORANGE, PINK, PURPLE, BLUE]
        return pal[(k + g) % 6] if not top else YELLOW
    if scheme == "bands":
        band = [RED, RED, ORANGE, ORANGE, YELLOW, YELLOW, WHITE, YELLOW, YELLOW, ORANGE, ORANGE, RED, RED, RED]
        col = band[k]
        return col if g % 2 == 0 else tuple(x * 0.82 for x in col)
    if scheme == "chevron":
        if k < 3:
            return ORANGE
        if top or k >= ROWS - 3:
            return YELLOW if k < ROWS - 1 else RED
        zig = abs((g % 6) - 3)
        return LIME if ((k + zig) // 2) % 2 == 0 else BLACK
    if scheme == "blocks":
        # Yellow with black stripes every fourth gore and a band of colour.
        if k < 2:
            return ORANGE
        if g % 4 == 0:
            return BLACK
        if 6 <= k <= 8:
            return [SKY, PINK, LIME, PURPLE, RED, BLUE][(g + k) % 6]
        if k == 5 or k == 9:
            return BLACK
        return YELLOW
    if scheme == "wave":
        # Orange above, blue below, a white wave between.
        mid = 7 + (1 if (g // 2) % 2 else 0)
        if k == mid:
            return WHITE
        return ORANGE if k > mid else (BLUE if k > 2 else NAVY)
    if scheme == "checks":
        upper = k >= 7
        a, b = (BLUE, YELLOW) if upper else (GREEN, YELLOW)
        if top:
            return BLUE
        return a if (g + k) % 2 == 0 else b
    if scheme == "stripes":
        if k < 2:
            return RED
        return [NAVY, WHITE, RED][g % 3] if not top else NAVY
    raise ValueError(scheme)


SCHEMES = ["rainbow", "harlequin", "bands", "chevron", "blocks", "wave", "checks", "stripes"]

# ------------------------------------------------------------------- helpers
def pchip(xs, ys):
    """Monotone cubic interpolation (Fritsch-Carlson)."""
    n = len(xs)
    h = [xs[i + 1] - xs[i] for i in range(n - 1)]
    d = [(ys[i + 1] - ys[i]) / h[i] for i in range(n - 1)]
    m = [0.0] * n
    m[0], m[-1] = d[0], d[-1]
    for i in range(1, n - 1):
        if d[i - 1] * d[i] <= 0:
            m[i] = 0.0
        else:
            w1, w2 = 2 * h[i] + h[i - 1], h[i] + 2 * h[i - 1]
            m[i] = (w1 + w2) / (w1 / d[i - 1] + w2 / d[i])

    def f(x):
        x = min(max(x, xs[0]), xs[-1])
        i = max(0, min(n - 2, next((j for j in range(n - 1) if x <= xs[j + 1]), n - 2)))
        t = (x - xs[i]) / h[i]
        h00, h10, h01, h11 = 2 * t**3 - 3 * t**2 + 1, t**3 - 2 * t**2 + t, -2 * t**3 + 3 * t**2, t**3 - t**2
        return h00 * ys[i] + h10 * h[i] * m[i] + h01 * ys[i + 1] + h11 * h[i] * m[i + 1]
    return f


PR = pchip([p[0] for p in PROFILE], [p[1] for p in PROFILE])


def radius(t):
    return R * max(PR(t), 0.0)


def arc_stations(n_rows, sub):
    """Heights (share of H) splitting the meridian into rows of equal fabric
    length (panels are cut about the same height), each split into `sub`."""
    ts = [i / 2000 for i in range(2001)]
    s = [0.0]
    for a, b in zip(ts, ts[1:]):
        s.append(s[-1] + math.hypot((b - a) * H, radius(b) - radius(a)))
    total = s[-1]
    out = []
    for j in range(n_rows * sub + 1):
        want = total * j / (n_rows * sub)
        i = next((i for i, v in enumerate(s) if v >= want), len(s) - 1)
        out.append(ts[i])
    out[-1] = 1.0
    return out


def make_object(name, bm, coll, sharp_angle=None):
    me = bpy.data.meshes.new(name)
    bm.to_mesh(me)
    bm.free()
    ob = bpy.data.objects.new(name, me)
    coll.objects.link(ob)
    return ob


def colour_layer(bm):
    return bm.loops.layers.color.get("Col") or bm.loops.layers.color.new("Col")


# Materials, kept as each face's material index and exported as the game's
# material code (inst.wgsl draws each one: woven nylon, its seams and load
# tapes, wicker, leather, rope, metal, the skirt's Nomex, the flame).
TRIM, WICKER_M, LEATHER_M, ROPE_M, METAL_M, NOMEX_M, FABRIC_M, SEAM_M, TAPE_M, FLAME_M = range(10)
MAT_CODE = [1.0, 1.2, 1.25, 1.3, 1.36, 1.45, 1.5, 1.52, 1.54, 2.0]
_mat = [TRIM]


def use(m):
    """The material for the faces made from now on."""
    _mat[0] = m


def paint(face, layer, col):
    face.material_index = _mat[0]
    for l in face.loops:
        l[layer] = (*col, 1.0)


def frames(pts, closed):
    """A frame (tangent, u, w) at each point of a path, carried along it
    without twisting (parallel transport)."""
    n = len(pts)
    tans = []
    for i in range(n):
        a = pts[i - 1] if (i > 0 or closed) else pts[i]
        b = pts[(i + 1) % n] if (i < n - 1 or closed) else pts[i]
        t = (b - a)
        tans.append(t.normalized() if t.length > 1e-9 else V((0, 0, 1)))
    u = tans[0].orthogonal().normalized()
    out = []
    for i, t in enumerate(tans):
        if i > 0:
            u = (u - t * u.dot(t))
            u = u.normalized() if u.length > 1e-9 else t.orthogonal().normalized()
        out.append((t, u, t.cross(u)))
    return out


def sweep(bm, layer, pts, r, col, sides=6, closed=False, caps=True):
    """A round section of radius r (a number, or one per point) along a path."""
    pts = [V(p) for p in pts]
    fr = frames(pts, closed)
    rings = []
    for i, (p, (t, u, w)) in enumerate(zip(pts, fr)):
        rr = r[i] if isinstance(r, (list, tuple)) else r
        rings.append([bm.verts.new(p + (u * math.cos(2 * math.pi * k / sides) + w * math.sin(2 * math.pi * k / sides)) * rr) for k in range(sides)])
    last = len(rings) if closed else len(rings) - 1
    for i in range(last):
        a, b = rings[i], rings[(i + 1) % len(rings)]
        for k in range(sides):
            f = bm.faces.new((a[k], a[(k + 1) % sides], b[(k + 1) % sides], b[k]))
            f.smooth = True
            paint(f, layer, col)
    if caps and not closed:
        for ring in (rings[0][::-1], rings[-1]):
            paint(bm.faces.new(ring), layer, col)


def rope(bm, layer, pts, r, col, closed=False):
    """A three-strand laid rope of radius r along a path: three strands
    twisted round each other, a turn every 3.5 diameters."""
    pts = [V(p) for p in pts]
    # Resample finely enough for the twist.
    fine = []
    segs = len(pts) if closed else len(pts) - 1
    for i in range(segs):
        a, b = pts[i], pts[(i + 1) % len(pts)]
        n = max(1, int((b - a).length / (r * 1.2)))
        for k in range(n):
            fine.append(a.lerp(b, k / n))
    if not closed:
        fine.append(pts[-1])
    fr = frames(fine, closed)
    pitch = 7.0 * r
    s = 0.0
    strands = [[], [], []]
    for i, (p, (t, u, w)) in enumerate(zip(fine, fr)):
        if i > 0:
            s += (fine[i] - fine[i - 1]).length
        for k in range(3):
            a = 2 * math.pi * (s / pitch + k / 3)
            strands[k].append(p + (u * math.cos(a) + w * math.sin(a)) * r * 0.5)
    for k in range(3):
        shade = tuple(x * (0.92 + 0.08 * k) for x in col)
        sweep(bm, layer, strands[k], r * 0.55, shade, sides=5, closed=closed, caps=not closed)


def rounded_rect(hx, hy, rc, per_corner=6, step=0.08):
    """The outline of a rectangle with rounded corners, counter-clockwise,
    with points every `step` along the straight sides."""
    out = []
    corners = [(hx - rc, hy - rc, 0.0), (-hx + rc, hy - rc, 90.0), (-hx + rc, -hy + rc, 180.0), (hx - rc, -hy + rc, 270.0)]
    for ci, (cx, cy, a0) in enumerate(corners):
        for k in range(per_corner + 1):
            a = math.radians(a0 + 90.0 * k / per_corner)
            out.append(V((cx + rc * math.cos(a), cy + rc * math.sin(a), 0.0)))
        # The straight side to the next corner.
        nx, ny, na = corners[(ci + 1) % 4]
        a1 = math.radians(na)
        p0 = out[-1]
        p1 = V((nx + rc * math.cos(a1), ny + rc * math.sin(a1), 0.0))
        n = max(1, int((p1 - p0).length / step))
        for k in range(1, n):
            out.append(p0.lerp(p1, k / n))
    return out


def wall(bm, layer, outline, z0, z1, col, inward=False, closed=True):
    """A vertical wall along an outline (faces out, or in)."""
    lo = [bm.verts.new((p.x, p.y, z0)) for p in outline]
    hi = [bm.verts.new((p.x, p.y, z1)) for p in outline]
    n = len(outline)
    for i in range(n if closed else n - 1):
        j = (i + 1) % n
        q = (lo[i], lo[j], hi[j], hi[i])
        f = bm.faces.new(q if not inward else q[::-1])
        f.smooth = True
        paint(f, layer, col)


def tube(bm, layer, a, b, r, col, n=6, caps=True):  # noqa: shared helper
    """A round bar from a to b."""
    a, b = V(a), V(b)
    ax = (b - a)
    if ax.length < 1e-6:
        return
    ax.normalize()
    u = ax.orthogonal().normalized()
    w = ax.cross(u)
    ra, rb = [], []
    for i in range(n):
        ang = 2 * math.pi * i / n
        d = u * math.cos(ang) + w * math.sin(ang)
        ra.append(bm.verts.new(a + d * r))
        rb.append(bm.verts.new(b + d * r))
    for i in range(n):
        j = (i + 1) % n
        f = bm.faces.new((ra[i], ra[j], rb[j], rb[i]))
        f.smooth = True
        paint(f, layer, col)
    if caps:
        for ring in (ra[::-1], rb):
            f = bm.faces.new(ring)
            paint(f, layer, col)


def box(bm, layer, lo, hi, col, smooth=False):
    (x0, y0, z0), (x1, y1, z1) = lo, hi
    vs = [bm.verts.new(p) for p in [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0), (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]]
    for q in [(0, 3, 2, 1), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]:
        f = bm.faces.new([vs[i] for i in q])
        f.smooth = smooth
        paint(f, layer, col)


def lathe(bm, layer, prof, n, col_of, z0=0.0, centre=(0.0, 0.0), smooth=True, phase=0.0):
    """A body of revolution: prof = [(r, z)], colour per ring band (the
    first vertex of each ring at angle `phase`)."""
    cx, cy = centre
    rings = []
    for (r, z) in prof:
        if r < 1e-6:
            rings.append([bm.verts.new((cx, cy, z0 + z))])
        else:
            rings.append([bm.verts.new((cx + r * math.cos(phase + 2 * math.pi * i / n), cy + r * math.sin(phase + 2 * math.pi * i / n), z0 + z)) for i in range(n)])
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


# ---------------------------------------------------------------- the envelope
def gore_columns(near):
    # Shares across a gore: a load tape on each edge, the bulging panel between.
    # (A load tape is about 25 mm wide: some 0.5% of a gore at its widest each side.)
    return [0.0, 0.005, 0.25, 0.5, 0.75, 0.995, 1.0] if near else [0.0, 0.5, 1.0]


def envelope(name, scheme, near, coll):
    bm = bmesh.new()
    layer = colour_layer(bm)
    sub = 3 if near else 1
    ts = arc_stations(ROWS, sub)
    # Near: a narrow band at every seam between rows of panels (the two
    # panels overlap there, sewn with two rows of stitches): its own faces,
    # 20 mm wide.
    seam_rows = set()
    if near:
        out = []
        for j, t in enumerate(ts):
            if j % sub == 0 and 0 < j < len(ts) - 1 and t < 0.985:
                d = 0.010 / H
                out += [t - d, t + d]
                seam_rows.add(len(out) - 2)
            else:
                out.append(t)
        ts = out
    cols = gore_columns(near)
    rows = []
    for j, t in enumerate(ts):
        z = t * H
        r = radius(t)
        ring = []
        for g in range(GORES):
            for ci, u in enumerate(cols[:-1]):
                ang = 2 * math.pi * (g + u) / GORES
                # Between the load tapes the fabric bulges out (less at the
                # mouth and the crown, where the gores are narrow).
                bulge = 0.022 * math.sin(math.pi * u) * math.sin(math.pi * min(max(t, 0.0), 1.0)) ** 0.5
                rr = r * (1 + bulge)
                ring.append(bm.verts.new((rr * math.cos(ang), rr * math.sin(ang), z)))
        if r < 1e-6:
            for v in ring:
                bm.verts.remove(v)
            ring = [bm.verts.new((0.0, 0.0, z))]
        rows.append(ring)
    n_per = len(cols) - 1
    tape_cols = {0, n_per - 1} if near else set()
    for j in range(len(rows) - 1):
        a, b = rows[j], rows[j + 1]
        # (The row of panels a face is in, counted along the meridian.)
        tmid = 0.5 * (ts[j] + ts[j + 1])
        k = min(sum(1 for jj, tt in enumerate(arc_stations(ROWS, 1)) if 0 < jj and tt <= tmid), ROWS - 1) if near else min(j // sub, ROWS - 1)
        seam = j in seam_rows
        for g in range(GORES):
            panel = scheme_colour(scheme, g, k)
            # (Panels of one colour still differ a little from one dye lot to the next.)
            lot = 0.96 + 0.08 * (((g * 7 + k * 13) * 2654435761) % 1000) / 1000.0
            panel = tuple(x * lot for x in panel)
            tape = tuple(0.8 * x + 0.01 for x in panel)
            for ci in range(n_per):
                i = g * n_per + ci
                i2 = (i + 1) % len(a) if len(a) > 1 else 0
                if ci in tape_cols:
                    use(TAPE_M)
                    col = tape
                elif seam:
                    use(SEAM_M)
                    col = panel
                else:
                    use(FABRIC_M)
                    col = panel
                if len(b) == 1:
                    f = bm.faces.new((a[i], a[i2], b[0]))
                else:
                    f = bm.faces.new((a[i], a[i2], b[i2 if len(b) > 1 else 0], b[i]))
                f.smooth = True
                paint(f, layer, col)
    # The load tapes are creases: sharp edges along the gore seams.
    bm.edges.ensure_lookup_table()
    for e in bm.edges:
        a, b = e.verts
        if abs(a.co.z - b.co.z) > 1e-4 and math.hypot(a.co.x, a.co.y) > 1e-3:
            ang = math.atan2(a.co.y, a.co.x) % (2 * math.pi)
            frac = ang / (2 * math.pi) * GORES
            if abs(frac - round(frac)) < 1e-4:
                e.smooth = False
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    use(TRIM)
    return make_object(name, bm, coll)


def skirt(coll):
    """The scoop: flame-proof fabric hanging below the mouth, a little wider."""
    bm = bmesh.new()
    layer = colour_layer(bm)
    r0 = radius(0.0)
    prof = [(r0 * 1.0, 0.0), (r0 * 0.99, -0.45), (r0 * 1.0, -0.9), (r0 * 1.02, -1.3)]
    n = GORES * 2
    use(NOMEX_M)
    rings = [[bm.verts.new((r * math.cos(2 * math.pi * i / n), r * math.sin(2 * math.pi * i / n), z)) for i in range(n)] for r, z in prof]
    for k in range(len(rings) - 1):
        for i in range(n):
            j = (i + 1) % n
            f = bm.faces.new((rings[k][i], rings[k][j], rings[k + 1][j], rings[k + 1][i]))
            f.smooth = True
            paint(f, layer, NOMEX if k < 2 else tuple(x * 1.4 for x in NOMEX))
    # (Two-sided: the inside, seen from the basket, is a copy facing in.)
    inner = bmesh.ops.duplicate(bm, geom=list(bm.faces))["geom"]
    bmesh.ops.reverse_faces(bm, faces=[f for f in inner if isinstance(f, bmesh.types.BMFace)])
    return make_object("skirt", bm, coll)


def karabiner(bm, layer, at, r=0.045, wire=0.007):
    """A steel karabiner: an oval loop of round bar, hanging."""
    at = V(at)
    pts = [at + V((r * 0.55 * math.cos(2 * math.pi * k / 14), 0.0, r * math.sin(2 * math.pi * k / 14))) for k in range(14)]
    sweep(bm, layer, pts, wire, STEEL, sides=5, closed=True)


def wires(name, frame_half, frame_z, mouth_z, scale, coll):
    """Flying wires: steel cables from the foot of every second load tape at
    the mouth down to karabiners at the corners of the burner frame."""
    bm = bmesh.new()
    layer = colour_layer(bm)
    use(METAL_M)
    r0 = radius(0.0) * scale
    corners = [(sx * frame_half, sy * frame_half) for sx, sy in [(1, 1), (-1, 1), (-1, -1), (1, -1)]]
    for g in range(0, GORES, 2):
        ang = 2 * math.pi * g / GORES
        top = V((r0 * math.cos(ang), r0 * math.sin(ang), mouth_z))
        cx, cy = min(corners, key=lambda cc: (cc[0] - top.x) ** 2 + (cc[1] - top.y) ** 2)
        tube(bm, layer, (cx, cy, frame_z + 0.10), top, 0.005, c(0.30, 0.30, 0.32), n=5, caps=False)
    for cx, cy in corners:
        karabiner(bm, layer, (cx, cy, frame_z + 0.07), 0.05 * scale ** 0.3, 0.008)
    use(TRIM)
    return make_object(name, bm, coll)


# -------------------------------------------------------------------- the rigs
def burner_unit(bm, layer, cx, cy, z, r=0.16, h=0.30):
    """One burner: a stainless coil (a stack of turns of tube) in its can,
    the jet ring on top, the blast valve and its handle on the side, the
    pilot light."""
    use(METAL_M)
    # The coil: turns of tube round a cylinder.
    turns = 7
    for k in range(turns):
        zz = z + 0.03 + (h - 0.06) * k / (turns - 1)
        ring = [V((cx + r * math.cos(2 * math.pi * i / 16), cy + r * math.sin(2 * math.pi * i / 16), zz)) for i in range(16)]
        sweep(bm, layer, ring, 0.018, STEEL, sides=6, closed=True)
    # The can inside the coil, and the jet ring on top.
    lathe(bm, layer, [(0.0, 0.0), (r * 0.82, 0.0), (r * 0.82, h), (r * 0.6, h + 0.015), (r * 0.25, h + 0.02), (0.0, h + 0.02)], 14,
          lambda k, i: DARK_STEEL if k < 2 else STEEL, z0=z, centre=(cx, cy))
    # Blast valve, handle.
    box(bm, layer, (cx + r + 0.01, cy - 0.035, z + 0.04), (cx + r + 0.11, cy + 0.035, z + 0.13), STEEL)
    use(TRIM)
    tube(bm, layer, (cx + r + 0.09, cy, z + 0.08), (cx + r + 0.12, cy, z - 0.20), 0.013, BLACK, n=6)
    use(METAL_M)
    tube(bm, layer, (cx - r * 0.3, cy + r * 0.3, z + h), (cx - r * 0.3, cy + r * 0.3, z + h + 0.05), 0.008, STEEL, n=5)


def cylinder_tank(bm, layer, cx, cy, jacket):
    """A propane cylinder in its padded jacket, the valve and gauge on top."""
    use(LEATHER_M)
    lathe(bm, layer, [(0.0, 0.04), (0.150, 0.04), (0.165, 0.07), (0.168, 0.40), (0.165, 0.70), (0.140, 0.76), (0.0, 0.76)], 14,
          lambda k, i: jacket, centre=(cx, cy))
    use(METAL_M)
    lathe(bm, layer, [(0.11, 0.76), (0.10, 0.80), (0.06, 0.83), (0.035, 0.84), (0.035, 0.90), (0.05, 0.92), (0.0, 0.93)], 12,
          lambda k, i: STEEL, centre=(cx, cy))
    use(TRIM)
    lathe(bm, layer, [(0.0, 0.86), (0.028, 0.86), (0.028, 0.875), (0.0, 0.875)], 8, lambda k, i: WHITE if k == 2 else BLACK, centre=(cx + 0.05, cy))
    # The cylinder's carrying strap round its middle.
    use(LEATHER_M)
    ring = [V((cx + 0.172 * math.cos(2 * math.pi * i / 16), cy + 0.172 * math.sin(2 * math.pi * i / 16), 0.45)) for i in range(16)]
    sweep(bm, layer, ring, 0.012, LEATHER, sides=4, closed=True)


def basket(bm, layer, w, l, h, rc, wall_t, rim_r, handles_per_side):
    """A wicker basket: rounded corners, woven walls outside and in (the
    weave is drawn by the shader, inst.wgsl), a plywood floor on leather-shod
    runners, a padded suede rim, step holes, rope handles."""
    hx, hy = w / 2, l / 2
    out_line = rounded_rect(hx, hy, rc)
    in_line = rounded_rect(hx - wall_t, hy - wall_t, max(rc - wall_t, 0.02))
    use(WICKER_M)
    wall(bm, layer, out_line, 0.0, h - 0.03, WICKER)
    wall(bm, layer, in_line, 0.05, h - 0.03, tuple(x * 0.85 for x in WICKER), inward=True)
    # The top of the walls under the rim, and the bottom edge.
    use(TRIM)
    floor_v = [bm.verts.new((p.x, p.y, 0.05)) for p in in_line]
    paint(bm.faces.new(floor_v), layer, FLOOR)
    under = [bm.verts.new((p.x, p.y, 0.0)) for p in out_line]
    paint(bm.faces.new(under[::-1]), layer, FLOOR)
    # Runners under the floor, shod in leather.
    use(LEATHER_M)
    for sx in (-1, 1):
        box(bm, layer, (sx * (hx - 0.16) - 0.035, -hy + 0.08, -0.05), (sx * (hx - 0.16) + 0.035, hy - 0.08, 0.0), LEATHER, smooth=False)
    # The padded rim: suede over a rawhide-wrapped cane top.
    rim = [V((p.x, p.y, h - 0.02)) for p in rounded_rect(hx - wall_t * 0.5, hy - wall_t * 0.5, max(rc - wall_t * 0.5, 0.03), step=0.06)]
    sweep(bm, layer, rim, rim_r, SUEDE, sides=10, closed=True)
    # Step holes in the long sides (dark openings in the weave).
    use(TRIM)
    for sx in (-1, 1):
        for zc in (0.32, 0.72):
            for yc in (-hy * 0.45,):
                pts = [V((sx * (hx + 0.002), yc + 0.07 * math.cos(2 * math.pi * k / 10), zc + 0.045 * math.sin(2 * math.pi * k / 10))) for k in range(10)]
                vs = [bm.verts.new(p_) for p_ in pts]
                f = bm.faces.new(vs if sx > 0 else vs[::-1])
                paint(f, layer, c(0.03, 0.02, 0.012))
    # Rope handles: loops of rope through the weave below the rim.
    use(ROPE_M)
    for side in range(4):
        along_y = side in (0, 2)
        sgn = 1 if side in (0, 1) else -1
        span = l if along_y else w
        for k in range(handles_per_side):
            t = (k + 1) / (handles_per_side + 1) - 0.5
            pts = []
            for q in range(9):
                a = math.pi * q / 8
                d = 0.11 * math.cos(a)
                drop = 0.13 * math.sin(a)
                off = 0.035 * math.sin(a)
                if along_y:
                    pts.append(V((sgn * (hx + 0.012 + off), t * span + d, h - 0.16 - drop)))
                else:
                    pts.append(V((t * span + d, sgn * (hy + 0.012 + off), h - 0.16 - drop)))
            rope(bm, layer, pts, 0.011, CREAM)


def rig_sport(coll):
    bm = bmesh.new()
    layer = colour_layer(bm)
    w, l, h = S_BASKET
    hx, hy = w / 2, l / 2
    basket(bm, layer, w, l, h, 0.13, 0.05, 0.055, 2)
    # Four propane cylinders in the corners, in padded jackets.
    for (sx, sy), jacket in zip([(-1, -1), (1, -1), (-1, 1), (1, 1)], [c(0.02, 0.05, 0.20), c(0.02, 0.05, 0.20), c(0.30, 0.02, 0.02), c(0.02, 0.05, 0.20)]):
        cylinder_tank(bm, layer, sx * (hx - 0.23), sy * (hy - 0.23), jacket)
    # Fuel hoses up to the burners.
    use(TRIM)
    for k, cx in enumerate((-0.17, 0.17)):
        pts = [V((cx * 2.2, (hy - 0.23) * (1 if k else -1), 0.93)), V((cx * 1.8, 0.0, 1.6)), V((cx * 1.2, 0.05, 2.2)), V((cx + 0.05, 0.10, S_NOZZLE_Z - 0.30))]
        sweep(bm, layer, pts, 0.014, BLACK, sides=6)
    # The burner poles: nylon rods in padded suede sleeves, from the basket's
    # corners up to the frame.
    use(LEATHER_M)
    f = 0.36
    for sx in (-1, 1):
        for sy in (-1, 1):
            tube(bm, layer, (sx * (hx - 0.07), sy * (hy - 0.07), h - 0.05), (sx * f, sy * f, S_FRAME_Z), 0.035, c(0.32, 0.03, 0.03), n=8)
    # The load frame: stainless square, a cross bar carrying the burners,
    # and a small instrument pack.
    use(METAL_M)
    sq = [V((f, f, S_FRAME_Z)), V((-f, f, S_FRAME_Z)), V((-f, -f, S_FRAME_Z)), V((f, -f, S_FRAME_Z))]
    sweep(bm, layer, sq, 0.02, STEEL, sides=8, closed=True)
    tube(bm, layer, (-f, 0, S_FRAME_Z), (f, 0, S_FRAME_Z), 0.018, STEEL, n=8)
    for cx in (-0.17, 0.17):
        burner_unit(bm, layer, cx, 0.0, S_NOZZLE_Z - 0.33)
    use(TRIM)
    box(bm, layer, (f - 0.02, -0.06, S_FRAME_Z - 0.16), (f + 0.03, 0.06, S_FRAME_Z - 0.02), BLACK)
    # A drop-line bag hung outside one end: a coil of rope in a canvas bag.
    use(LEATHER_M)
    lathe(bm, layer, [(0.0, 0.0), (0.10, 0.0), (0.11, 0.06), (0.11, 0.30), (0.10, 0.32), (0.0, 0.32)], 12, lambda k, i: c(0.12, 0.10, 0.06), z0=h - 0.62, centre=(0.25, hy + 0.12))
    use(ROPE_M)
    rope(bm, layer, [V((0.25, hy + 0.10, h - 0.30)), V((0.25, hy + 0.04, h - 0.10)), V((0.25, hy - 0.02, h - 0.02))], 0.008, CREAM)
    # (The pilot and passengers: MakeHuman people, people.py, placed by the
    # game where crew_places() says.)
    ob = make_object("rig sport", bm, coll)
    ob["pivot"] = (0.0, 0.0, S_NOZZLE_Z)
    ob["axis"] = (0.0, 0.0, S_MOUTH_Z)
    return ob


def rig_large(coll):
    bm = bmesh.new()
    layer = colour_layer(bm)
    w, l, h = L_BASKET
    hx, hy = w / 2, l / 2
    basket(bm, layer, w, l, h, 0.18, 0.06, 0.065, 4)
    # Partitions: the pilot's compartment across the middle (with the
    # cylinders), the ends split along their length.
    pc = 0.55
    for sy in (-1, 1):
        use(WICKER_M)
        for z_side in (-1, 1):
            pts = [V((-hx + 0.05, sy * pc + z_side * 0.025, 0.0)), V((hx - 0.05, sy * pc + z_side * 0.025, 0.0))]
            pts = [pts[0].lerp(pts[1], k / 20) for k in range(21)]
            wall(bm, layer, pts, 0.05, h - 0.04, WICKER, inward=(z_side < 0), closed=False)
            pts2 = [V((z_side * 0.025, sy * pc, 0.0)), V((z_side * 0.025, sy * (hy - 0.05), 0.0))]
            pts2 = [pts2[0].lerp(pts2[1], k / 16) for k in range(17)]
            wall(bm, layer, pts2, 0.05, h - 0.04, WICKER, inward=(z_side > 0) == (sy > 0), closed=False)
        use(LEATHER_M)
        sweep(bm, layer, [V((-hx + 0.05, sy * pc, h - 0.02)), V((hx - 0.05, sy * pc, h - 0.02))], 0.05, SUEDE, sides=8)
        sweep(bm, layer, [V((0.0, sy * pc, h - 0.02)), V((0.0, sy * (hy - 0.05), h - 0.02))], 0.05, SUEDE, sides=8)
    for sx in (-1, 1):
        for cy in (-0.27, 0.27):
            cylinder_tank(bm, layer, sx * (hx - 0.26), cy, c(0.02, 0.05, 0.20))
    # Poles to a larger frame, quad burner.
    use(LEATHER_M)
    f = 0.62
    for sx in (-1, 1):
        for sy in (-1, 1):
            tube(bm, layer, (sx * (hx - 0.08), sy * (pc + 0.06), h - 0.05), (sx * f, sy * f, L_FRAME_Z), 0.045, c(0.05, 0.05, 0.06), n=8)
    use(METAL_M)
    sq = [V((f, f, L_FRAME_Z)), V((-f, f, L_FRAME_Z)), V((-f, -f, L_FRAME_Z)), V((f, -f, L_FRAME_Z))]
    sweep(bm, layer, sq, 0.026, STEEL, sides=8, closed=True)
    for sx in (-1, 1):
        tube(bm, layer, (sx * f, -f, L_FRAME_Z), (sx * f, f, L_FRAME_Z), 0.02, STEEL, n=8)
        tube(bm, layer, (sx * f, 0, L_FRAME_Z), (0, 0, L_FRAME_Z), 0.02, STEEL, n=8)
    for cx in (-0.2, 0.2):
        for cy in (-0.2, 0.2):
            burner_unit(bm, layer, cx, cy, L_NOZZLE_Z - 0.36, r=0.18, h=0.33)
    use(TRIM)
    for k, (cx, cy) in enumerate([(-0.2, -0.2), (0.2, 0.2)]):
        sweep(bm, layer, [V(((hx - 0.26) * (1 if k else -1), 0.27 * (1 if k else -1), 0.93)), V((cx * 1.5, cy, 1.9)), V((cx, cy, L_NOZZLE_Z - 0.34))], 0.016, BLACK, sides=6)
    ob = make_object("rig large", bm, coll)
    ob["pivot"] = (0.0, 0.0, L_NOZZLE_Z)
    ob["axis"] = (0.0, 0.0, L_MOUTH_Z)
    return ob


def crew_places():
    """Where the people stand in each basket: (x, y, floor z, facing), the
    facing in radians from +X (people.py's people face +X). The pilot first."""
    sport = [(0.28, -0.35, 0.05, 0.2), (-0.30, -0.30, 0.05, -0.4), (0.05, 0.42, 0.05, 3.0)]
    w, l, h = L_BASKET
    hx, hy = w / 2, l / 2
    large = [(0.3, 0.0, 0.06, 0.0)]
    seed = 0.13
    # Passengers in the four compartments, mostly looking out.
    for sy in (-1, 1):
        for sx in (-1, 1):
            for (px, py) in [(0.33, 0.95), (0.70, 0.95), (0.33, 1.55), (0.72, 1.60), (0.50, 2.0)]:
                seed = (seed * 9301 + 49297) % 233280 / 233280
                x, y = sx * px * (hx - 0.1) / 0.9, sy * min(py, hy - 0.25)
                large.append((x, y, 0.06, math.atan2(y, sx * px) + (seed - 0.5) * 1.6))
    # (Hands reach some 0.4 m ahead: nobody's through the wicker.)
    def inside(places, hx, hy, walls=False):
        out = []
        for k, (x, y, z, f) in enumerate(places):
            for _ in range(30):
                hx_, hy_ = x + 0.42 * math.cos(f), y + 0.42 * math.sin(f)
                ok = abs(hx_) < hx - 0.10 and abs(hy_) < hy - 0.10
                if walls and k > 0:
                    # (The passenger rig's partitions: x = 0 in the ends, y = +-0.55.)
                    ok = ok and hx_ * x > 0.0 and abs(hx_) > 0.08 and hy_ * y > 0.0 and abs(hy_) > 0.65
                if ok:
                    break
                x, y = x - 0.02 * math.cos(f), y - 0.02 * math.sin(f)
            out.append((round(x, 3), round(y, 3), z, f))
        return out
    sw, sl, _ = S_BASKET
    return {"sport": inside(sport, sw / 2, sl / 2), "large": inside(large, hx, hy, walls=True)}


def rig_far(coll):
    bm = bmesh.new()
    layer = colour_layer(bm)
    w, l, h = S_BASKET
    use(WICKER_M)
    box(bm, layer, (-w / 2, -l / 2, 0.0), (w / 2, l / 2, h), WICKER)
    use(LEATHER_M)
    f = 0.36
    for sx in (-1, 1):
        for sy in (-1, 1):
            tube(bm, layer, (sx * w / 2, sy * l / 2, h), (sx * f, sy * f, S_FRAME_Z), 0.05, SUEDE, n=3, caps=False)
    use(METAL_M)
    box(bm, layer, (-0.35, -0.2, S_FRAME_Z), (0.35, 0.2, S_NOZZLE_Z), STEEL)
    use(TRIM)
    return make_object("rig far", bm, coll)


FLAGS = ["fr", "be", "ch", "de"]


def flag_colour(kind, u, v):
    """The flag's colour at u (0 at the hoist .. 1 at the fly), v (0 bottom .. 1 top)."""
    if kind == "fr":
        return [c(0.0, 0.08, 0.40), WHITE, c(0.75, 0.02, 0.04)][min(int(u * 3), 2)]
    if kind == "be":
        return [BLACK, c(0.95, 0.70, 0.02), c(0.75, 0.02, 0.04)][min(int(u * 3), 2)]
    if kind == "de":
        return [c(0.85, 0.62, 0.0), c(0.75, 0.0, 0.0), BLACK][min(int(v * 3), 2)]
    # Switzerland (and Savoy): a white cross on red.
    x, y = u * 1.5 - 0.75, v - 0.5
    cross = (abs(x) < 0.11 and abs(y) < 0.32) or (abs(y) < 0.11 and abs(x) < 0.32)
    return WHITE if cross else c(0.80, 0.02, 0.02)


def flag(kind, coll):
    """A flag on a line hung from a corner of the basket: the line down 3.5 m,
    the flag tied along its lowest metre, drooping and rippling in still air
    (the balloon goes with the wind, so there is none to fly it). Origin at
    the line's top, tied under the basket's rim at a corner."""
    bm = bmesh.new()
    layer = colour_layer(bm)
    use(ROPE_M)
    line = [V((0.0, 0.0, -0.02 * k * k)) + V((0.03 * math.sin(k * 0.9), 0.0, -0.35 * k)) for k in range(11)]
    # (A thin line: a twist would not show at 6 mm.)
    sweep(bm, layer, line, 0.006, CREAM, sides=5)
    use(FABRIC_M)
    w, hgt = 0.9, 0.6
    nu, nv = 10, 6
    top = line[-1] + V((0.0, 0.0, 0.85))
    grid = []
    for i in range(nu + 1):
        u = i / nu
        col = []
        for j in range(nv + 1):
            v = j / nv
            # Out from the line, sagging more towards the fly, in soft folds.
            x = w * u * 0.93
            z = -hgt * (1 - v) - 0.22 * u * u
            y = 0.06 * math.sin(u * 7.0 + 0.6) * u
            col.append(bm.verts.new(top + V((x, y, z))))
        grid.append(col)
    for i in range(nu):
        for j in range(nv):
            q = (grid[i][j], grid[i + 1][j], grid[i + 1][j + 1], grid[i][j + 1])
            colr = flag_colour(kind, (i + 0.5) / nu, (j + 0.5) / nv)
            f = bm.faces.new(q)
            f.smooth = True
            paint(f, layer, colr)
            # (Two-sided.)
            g = bm.faces.new([bm.verts.new(v_.co) for v_ in q[::-1]])
            g.smooth = True
            paint(g, layer, colr)
    use(TRIM)
    return make_object(f"flag {kind}", bm, coll)


def flame(coll):
    """The burner flame, 1 m nominal (the game scales and flickers it): a
    blue root, a white-yellow core, orange, and red-tinged tips."""
    bm = bmesh.new()
    layer = colour_layer(bm)
    prof = [(0.0, 0.0), (0.05, 0.0), (0.09, 0.08), (0.14, 0.22), (0.17, 0.40), (0.16, 0.58), (0.12, 0.74), (0.075, 0.87), (0.03, 0.96), (0.0, 1.0)]
    use(FLAME_M)
    cols = [c(0.25, 0.45, 1.0), c(1.0, 0.95, 0.75), c(1.0, 0.85, 0.45), c(1.0, 0.62, 0.16), c(1.0, 0.45, 0.06), c(1.0, 0.33, 0.03), c(0.95, 0.22, 0.02), c(0.80, 0.12, 0.01), c(0.60, 0.06, 0.0)]
    lathe(bm, layer, [(r, z) for r, z in prof], 10, lambda k, i: cols[min(k, len(cols) - 1)])
    use(TRIM)
    return make_object("flame", bm, coll)


# ---------------------------------------------------------------------- build
EXPECTED = [f"env {s}" for s in SCHEMES] + [f"env {s} far" for s in SCHEMES] + ["skirt", "wires sport", "wires large", "rig sport", "rig large", "rig far", "flame"] + [f"flag {k}" for k in FLAGS]


def build():
    for ob in list(bpy.data.objects):
        bpy.data.objects.remove(ob, do_unlink=True)
    coll = bpy.context.scene.collection
    for i, s in enumerate(SCHEMES):
        ob = envelope(f"env {s}", s, True, coll)
        ob.location = (i % 4 * 24.0, i // 4 * 24.0, S_MOUTH_Z)
        far = envelope(f"env {s} far", s, False, coll)
        far.location = (i % 4 * 24.0, i // 4 * 24.0 - 70.0, S_MOUTH_Z)
    sk = skirt(coll)
    sk.location = (0.0, 0.0, S_MOUTH_Z)
    wires("wires sport", 0.36, S_FRAME_Z, S_MOUTH_Z, 1.0, coll)
    wl = wires("wires large", 0.62, L_FRAME_Z, L_MOUTH_Z, LARGE_SCALE, coll)
    wl.location = (0.0, -150.0, 0.0)
    rig_sport(coll)
    rl = rig_large(coll)
    rl.location = (0.0, -150.0, 0.0)
    rf = rig_far(coll)
    rf.location = (0.0, -70.0, 0.0)
    for i, k in enumerate(FLAGS):
        fg = flag(k, coll)
        fg.location = (3.0 + 1.5 * i, 0.0, 1.0)
    fl = flame(coll)
    fl.location = (0.0, 0.0, S_NOZZLE_Z)
    # The passenger balloon's envelope, for looking at (not exported: the
    # game scales "env <scheme>").
    big = bpy.data.objects["env harlequin"].copy()
    big.name = "preview large envelope"
    big.scale = (LARGE_SCALE,) * 3
    big.location = (0.0, -150.0, L_MOUTH_Z)
    coll.objects.link(big)
    big_skirt = sk.copy()
    big_skirt.name = "preview large skirt"
    big_skirt.scale = (LARGE_SCALE,) * 3
    big_skirt.location = (0.0, -150.0, L_MOUTH_Z)
    coll.objects.link(big_skirt)


def game(v):
    return (v[0], v[2], -v[1])


def export_crew(path):
    """The places, in the game's frame (x right, y up, z back: game())."""
    out = {k: [[*game((x, y, z)), f] for (x, y, z, f) in v] for k, v in crew_places().items()}
    with open(path, "w") as f:
        json.dump({k: [[round(a, 3) for a in p] for p in v] for k, v in out.items()}, f)
        f.write("\n")


def export(path):
    out = bytearray(b"FFAC")
    obs = [o for o in bpy.data.objects if o.type == "MESH"]
    names = {o.name for o in obs}
    missing = [n for n in EXPECTED if n not in names]
    if missing:
        raise SystemExit(f"balloons export: the game needs these pieces: {', '.join(missing)}")
    obs = [bpy.data.objects[n] for n in EXPECTED]
    out += struct.pack("<II", 1, len(obs))
    total = 0
    for ob in obs:
        me = ob.data
        me.calc_loop_triangles()
        col = me.color_attributes.get("Col")
        # Exported about the piece's own origin (its placement in the .blend
        # is only for looking at them side by side).
        nm = V((1, 1, 1))
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
        pivot = V(ob.get("pivot", (0.0, 0.0, 0.0)))
        axis = V(ob.get("axis", (0.0, 0.0, 1.0)))
        name = ob.name.encode()
        out += struct.pack("<I", len(name)) + name
        # (The rigs' "axis" is the mouth's position, not a direction: as it is.)
        out += struct.pack("<7f", *game(pivot), *game(axis), 0.0)
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


# --------------------------------------------------------------------- render
def setup_look():
    """Materials for looking at the .blend (and --render): the colour
    attribute as albedo, fabric a little translucent, the flame glowing."""
    for ob in bpy.data.objects:
        if ob.type != "MESH":
            continue
        name = "flame" if ob.name.startswith("flame") else ("fabric" if ob.name.startswith(("env", "preview", "skirt")) else "solid")
        mat = bpy.data.materials.get(name)
        if mat is None:
            mat = bpy.data.materials.new(name)
            mat.use_nodes = True
            nt = mat.node_tree
            bsdf = next(n for n in nt.nodes if n.type == "BSDF_PRINCIPLED")
            attr = nt.nodes.new("ShaderNodeVertexColor")
            attr.layer_name = "Col"
            if name == "flame":
                out = next(n for n in nt.nodes if n.type == "OUTPUT_MATERIAL")
                em = nt.nodes.new("ShaderNodeEmission")
                em.inputs["Strength"].default_value = 25.0
                nt.links.new(attr.outputs["Color"], em.inputs["Color"])
                nt.links.new(em.outputs["Emission"], out.inputs["Surface"])
            else:
                nt.links.new(attr.outputs["Color"], bsdf.inputs["Base Color"])
                bsdf.inputs["Roughness"].default_value = 0.55 if name == "fabric" else 0.8
                if name == "fabric":
                    for key in ("Subsurface Weight", "Transmission Weight"):
                        if key in bsdf.inputs:
                            pass
                    if "Sheen Weight" in bsdf.inputs:
                        bsdf.inputs["Sheen Weight"].default_value = 0.2
            mat.use_backface_culling = False
        ob.data.materials.clear()
        ob.data.materials.append(mat)


def render(outdir):
    os.makedirs(outdir, exist_ok=True)
    sc = bpy.context.scene
    engines = [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items]
    sc.render.engine = "CYCLES" if "CYCLES" in engines else engines[0]
    if sc.render.engine == "CYCLES":
        sc.cycles.samples = 48
        sc.cycles.use_denoising = True
    sc.render.resolution_x, sc.render.resolution_y = 1000, 1250
    world = bpy.data.worlds.new("sky")
    world.use_nodes = True
    nt = world.node_tree
    bg = next(n for n in nt.nodes if n.type == "BACKGROUND")
    try:
        sky = nt.nodes.new("ShaderNodeTexSky")
        nt.links.new(sky.outputs["Color"], bg.inputs["Color"])
        bg.inputs["Strength"].default_value = 0.35
    except Exception:
        bg.inputs["Color"].default_value = (0.35, 0.55, 0.9, 1)
    sc.world = world
    sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
    sun.data.energy = 4.0
    sun.rotation_euler = (math.radians(55), 0, math.radians(35))
    sc.collection.objects.link(sun)
    try:
        sc.view_settings.view_transform = "AgX"
    except Exception:
        pass
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    sc.collection.objects.link(cam)
    sc.camera = cam

    def shot(fname, loc, target, lens=35, only=None):
        if only is not None:
            for ob in bpy.data.objects:
                if ob.type == "MESH":
                    ob.hide_render = ob.name not in only
        cam.location = V(loc)
        d = V(target) - V(loc)
        cam.rotation_euler = d.to_track_quat("-Z", "Y").to_euler()
        cam.data.lens = lens
        sc.render.filepath = os.path.join(outdir, fname)
        bpy.ops.render.render(write_still=True)

    def balloon(scheme, at, large=False, flame_len=0.0, yaw=0.0):
        """A whole balloon (linked copies) with its basket floor at `at`."""
        k = LARGE_SCALE if large else 1.0
        mouth = L_MOUTH_Z if large else S_MOUTH_Z
        nozzle = L_NOZZLE_Z if large else S_NOZZLE_Z
        parts = [(f"env {scheme}", mouth, k), ("skirt", mouth, k), ("wires large" if large else "wires sport", 0.0, 1.0), ("rig large" if large else "rig sport", 0.0, 1.0)]
        if flame_len > 0:
            parts.append(("flame", nozzle, flame_len))
        made = []
        for name, z, sc_ in parts:
            ob = bpy.data.objects[name].copy()
            ob.location = (at[0], at[1], at[2] + z)
            ob.scale = (sc_ * (0.5 if name == "flame" else 1.0), sc_ * (0.5 if name == "flame" else 1.0), sc_) if name == "flame" else (sc_,) * 3
            ob.rotation_euler = (0, 0, yaw)
            sc.collection.objects.link(ob)
            made.append(ob.name)
        return made

    for ob in bpy.data.objects:
        if ob.type == "MESH":
            ob.hide_render = True
    one = balloon("harlequin", (0, 0, 0), flame_len=3.2)
    shot("sport_below.png", (10.0, -26.0, -1.0), (0.0, 0.0, 11.0), 26, one)
    shot("basket.png", (3.4, -4.2, 1.9), (0.0, 0.0, 1.9), 32, one)
    shot("burner.png", (1.2, -1.6, 2.0), (0.0, 0.0, 3.2), 24, one)
    big = balloon("rainbow", (100, 0, 0), large=True, flame_len=4.0)
    shot("large.png", (118.0, -42.0, 2.0), (100.0, 0.0, 17.0), 26, big)
    shot("large_basket.png", (104.5, -6.5, 3.6), (100.0, 0.0, 1.2), 30, big)
    # A morning launch: many balloons at different heights and distances.
    fleet = []
    import random
    rnd = random.Random(7)
    for i in range(26):
        s_ = SCHEMES[i % len(SCHEMES)]
        d = rnd.uniform(60, 700)
        az = rnd.uniform(-0.55, 0.55)
        x, y = 300 + d * math.sin(az), d * math.cos(az)
        z = rnd.uniform(20, 380)
        fleet += balloon(s_, (x, y, z), large=rnd.random() < 0.3, flame_len=3.0 if rnd.random() < 0.3 else 0.0, yaw=rnd.uniform(0, 6.28))
    sc.render.resolution_x, sc.render.resolution_y = 1400, 1400
    shot("fleet.png", (300.0, -40.0, 40.0), (300.0, 400.0, 230.0), 30, fleet)
    # All the schemes side by side.
    line = []
    for i, s_ in enumerate(SCHEMES):
        line += balloon(s_, ((i - 3.5) * 21.0 + 1000, 0.0, 0.0))
    sc.render.resolution_x, sc.render.resolution_y = 1800, 700
    shot("schemes.png", (1000.0, -150.0, 16.0), (1000.0, 0.0, 14.0), 40, line)


def main():
    blend = os.path.join(ASSETS, "blender", "balloons.blend")
    if not EXPORT_ONLY:
        build()
    # (Exported before the preview materials go on: the material index is
    # the game's material.)
    export(os.path.join(ASSETS, "balloons.bin.gz"))
    export_crew(os.path.join(ASSETS, "balloon_crew.json"))
    if not EXPORT_ONLY:
        setup_look()
        bpy.ops.wm.save_as_mainfile(filepath=blend)
    if RENDER:
        render(RENDER)


main()
