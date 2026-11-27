# us fixes, airways and procedures for flightplan.rs, from the faa cifp (public domain).
#   python3 assets/mkcifp.py <FAACIFP18> assets/cifp.tsv.gz
# get it from https://www.faa.gov/air_traffic/flight_info/aeronav/digital_products/cifp/download/.
# lines are tab separated, as below.
#   F ident lat lon                                   a named fix.
#   W airway fix,lat,lon;fix,lat,lon;...              one unbroken airway segment.
#   P airport kind name transition rtype legs         kind D sid, E star, F approach.
# a leg is fix,lat,lon,pt,course,altdesc,alt1,alt2,speed with an empty fix for heading legs.
# courses are true, made from the airport's magnetic variation. missed approaches are left out.
import gzip, sys

src, out = sys.argv[1], sys.argv[2]
recs = [l.rstrip('\n') for l in open(src, encoding='latin-1') if l.startswith('SUSA')]

def f(r, a, b):
    return r[a - 1:b].strip()

def latlon(s):
    # N37363403W122225483.
    if len(s) < 19 or s[0] not in 'NS' or s[9] not in 'EW':
        return None
    try:
        la = int(s[1:3]) + int(s[3:5]) / 60 + int(s[5:9]) / 360000
        lo = int(s[10:13]) + int(s[13:15]) / 60 + int(s[15:19]) / 360000
    except ValueError:
        return None
    return (-la if s[0] == 'S' else la, -lo if s[9] == 'W' else lo)

def var(s):
    # E0140 is 14.0 east.
    try:
        v = int(s[1:5]) / 10
    except ValueError:
        return 0.0
    return -v if s[:1] == 'W' else v

# fix tables, keyed the way the legs point at them.
enrt = {}      # (ident, icao) for EA.
term = {}      # (airport, ident) for PC, PN and PG.
vhf = {}       # (ident, icao) for D.
ndb = {}       # (ident, icao) for DB.
apvar = {}
appos = {}
for r in recs:
    sec, sub = r[4], r[5]
    # fix records carry their continuation number in column 22.
    if sec == 'E' and sub == 'A' and r[21] == '0':
        p = latlon(r[32:51])
        if p:
            enrt[(f(r, 14, 18), f(r, 20, 21))] = p
    elif sec == 'D' and r[21] == '0':
        p = latlon(r[32:51]) or latlon(r[55:74])
        if p:
            (ndb if sub == 'B' else vhf)[(f(r, 14, 17), f(r, 20, 21))] = p
    elif sec == 'P' and r[21] == '0':
        apt, sub = f(r, 7, 10), r[12]
        if sub in 'CNG':
            p = latlon(r[32:51])
            if p:
                term[(apt, f(r, 14, 18))] = p
        elif sub == 'A':
            apvar[apt] = var(r[51:56])
            appos[apt] = latlon(r[32:51])

# a fix in a neighbour's terminal area: the one of that name nearest the airport.
by_name = {}
for (a, ident), p in list(term.items()) + [((None, k[0]), p) for k, p in enrt.items()]:
    by_name.setdefault(ident, []).append(p)

def nearest(apt, ident):
    c = appos.get(apt)
    ps = by_name.get(ident)
    if not c or not ps:
        return None
    p = min(ps, key=lambda q: (q[0] - c[0]) ** 2 + (q[1] - c[1]) ** 2)
    return p if abs(p[0] - c[0]) + abs(p[1] - c[1]) < 3 else None

def fix(apt, ident, icao, sec, sub):
    if not ident:
        return None
    return own(apt, ident, icao, sec, sub) or nearest(apt, ident)

def own(apt, ident, icao, sec, sub):
    if sec == 'P':
        return term.get((apt, ident))
    if sec == 'E':
        return enrt.get((ident, icao))
    if sec == 'D':
        return (ndb if sub == 'B' else vhf).get((ident, icao))
    return term.get((apt, ident)) or enrt.get((ident, icao))

lines = []
seen = set()
for (ident, _), p in enrt.items():
    k = (ident, round(p[0], 2), round(p[1], 2))
    if k not in seen:
        seen.add(k)
        lines.append('F\t%s\t%.5f\t%.5f' % (ident, p[0], p[1]))
for (apt, ident), p in term.items():
    if ident.startswith('RW'):
        continue
    k = (ident, round(p[0], 2), round(p[1], 2))
    if k not in seen:
        seen.add(k)
        lines.append('F\t%s\t%.5f\t%.5f' % (ident, p[0], p[1]))

# airways, split where a segment ends.
seg, name = [], None
def flush():
    if name and len(seg) > 1:
        lines.append('W\t%s\t%s' % (name, ';'.join('%s,%.4f,%.4f' % s for s in seg)))
for r in recs:
    if r[4] != 'E' or r[5] != 'R' or r[38] != '0':
        continue
    n = f(r, 14, 18)
    if n != name:
        flush()
        seg, name = [], n
    ident = f(r, 30, 34)
    p = fix('', ident, f(r, 35, 36), r[36], r[37])
    if p:
        seg.append((ident, p[0], p[1]))
    # e in the second description column ends a continuous airway.
    if r[40] == 'E':
        flush()
        seg = []
flush()

# procedures.
procs = {}
order = []
for r in recs:
    if r[4] != 'P' or r[12] not in 'DEF' or r[38] not in '01':
        continue
    apt, kind = f(r, 7, 10), r[12]
    name, rtype, trans = f(r, 14, 19), r[19], f(r, 21, 25)
    key = (apt, kind, name, trans, rtype)
    if key not in procs:
        procs[key] = {'legs': [], 'missed': False}
        order.append(key)
    pr = procs[key]
    # the missed approach starts at the first leg flagged m.
    if kind == 'F' and r[41] == 'M':
        pr['missed'] = True
    if pr['missed']:
        continue
    ident = f(r, 30, 34)
    p = fix(apt, ident, f(r, 35, 36), r[36], r[37])
    pt = f(r, 48, 49)
    # a leg to a fix that is not in the file is left out.
    if ident and not p and pt not in ('VA', 'VM', 'VI', 'VD', 'VR', 'CA', 'CI', 'CD', 'CR'):
        continue
    crs = f(r, 71, 74)
    try:
        crs = '%.0f' % ((int(crs) / 10 + apvar.get(apt, 0.0)) % 360) if crs and 'T' not in crs else ''
    except ValueError:
        crs = ''
    alt = lambda s: s.replace('FL', '') + '00' if s.startswith('FL') else s.lstrip('0')
    leg = [ident if p else '', '%.5f' % p[0] if p else '', '%.5f' % p[1] if p else '', pt, crs, r[82].strip(), alt(f(r, 85, 89)), alt(f(r, 90, 94)), f(r, 100, 102).lstrip('0')]
    pr['legs'].append(','.join(leg))
for key in order:
    legs = procs[key]['legs']
    if legs:
        apt, kind, name, trans, rtype = key
        lines.append('P\t%s\t%s\t%s\t%s\t%s\t%s' % (apt, kind, name, trans, rtype, ';'.join(legs)))

data = ('\n'.join(lines) + '\n').encode('utf-8')
open(out, 'wb').write(gzip.compress(data, 9, mtime=0))
print(sum(1 for l in lines if l[0] == 'F'), 'fixes', sum(1 for l in lines if l[0] == 'W'), 'airway segments', sum(1 for l in lines if l[0] == 'P'), 'procedure routes', len(data), 'bytes raw')
