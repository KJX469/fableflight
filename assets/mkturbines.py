#!/usr/bin/env python3
"""Packs the world's wind turbines from OpenStreetMap into
assets/wind_turbines.bin.gz, which src/turbines.rs reads.

Data © OpenStreetMap contributors, under the Open Database License (ODbL):
every node tagged power=generator + generator:source=wind. QLever's
OpenStreetMap endpoint (https://qlever.dev/api/osm-planet) answers for the
whole planet in seconds; save the answer as tab-separated values:

    PREFIX osmkey: <https://www.openstreetmap.org/wiki/Key:>
    PREFIX geo: <http://www.opengis.net/ont/geosparql#>
    PREFIX osm: <https://www.openstreetmap.org/>
    PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
    SELECT ?osm ?wkt ?hub ?rotor ?height ?power WHERE {
      ?osm osmkey:power "generator" .
      ?osm osmkey:generator:source "wind" .
      ?osm rdf:type osm:node .
      ?osm geo:hasGeometry/geo:asWKT ?wkt .
      OPTIONAL { ?osm osmkey:height:hub ?hub }
      OPTIONAL { ?osm osmkey:rotor:diameter ?rotor }
      OPTIONAL { ?osm osmkey:height ?height }
      OPTIONAL { ?osm osmkey:generator:output:electricity ?power }
    }

    curl -sSL --post301 -H 'Accept: text/tab-separated-values' \
        --data-urlencode query@turbines.sparql https://qlever.dev/api/osm-planet -o turbines.tsv
    python3 assets/mkturbines.py turbines.tsv [assets/wind_turbines.bin.gz]

(An Overpass API answer in CSV with the columns @lat, @lon, height:hub,
rotor:diameter, height and generator:output:electricity, tab-separated,
reads as well.)

Each turbine gets its hub height and rotor diameter: from its tags where it
has them (height:hub, rotor:diameter; `height` taken as the hub's unless
that would put the rotor in the ground), else estimated from its rated
power (a rotor of about 62 m x sqrt(MW), the hub about as high as the rotor
is wide, as for the machines of the 2000s and 2010s), else a typical 2-3 MW
machine of 92 m on a 95 m tower. Small turbines (under 15 m) are left out.

Format, little-endian, gzip:
  "FFWT" u32 version (1), u32 n_cells
  per cell (1 x 1 degree): i16 lat floor, i16 lon floor, u32 count
  then the turbines, cell by cell, 6 bytes each:
      u16 lat, u16 lon (within the cell, 1/65536 degree),
      u8 hub height (m), u8 rotor diameter (m)
"""

import csv
import gzip
import math
import re
import struct
import sys
from collections import defaultdict

NUM = re.compile(r"[-+]?\d+(?:[.,]\d+)?")


def metres(s):
    """A length tag's value in metres (`117`, `117 m`, `117.5`, `384 ft`)."""
    if not s:
        return None
    m = NUM.search(s)
    if not m:
        return None
    v = float(m.group(0).replace(",", "."))
    if "ft" in s or "'" in s:
        v *= 0.3048
    return v if v > 0 else None


def megawatts(s):
    """generator:output:electricity (`3 MW`, `2300 kW`, `850000 W`, `yes`) in MW."""
    if not s:
        return None
    m = NUM.search(s)
    if not m:
        return None
    v = float(m.group(0).replace(",", "."))
    u = s.lower()
    if "gw" in u:
        v *= 1000
    elif "mw" in u:
        pass
    elif "kw" in u:
        v /= 1000
    elif "w" in u:
        v /= 1e6
    else:
        # (A bare number: kW if large, MW if small.)
        v = v / 1000 if v > 50 else v
    return v if 0.005 < v < 30 else None


def size(row):
    hub = metres(row.get("height:hub"))
    rotor = metres(row.get("rotor:diameter"))
    height = metres(row.get("height"))
    mw = megawatts(row.get("generator:output:electricity"))
    if rotor is None and mw:
        rotor = 62 * math.sqrt(mw)
    if hub is None and height:
        hub = height
        # (`height` the tip's: the hub half a rotor lower, if that leaves the
        # blades clear of the ground.)
        if rotor and height - rotor / 2 > 0.55 * rotor:
            hub = height - rotor / 2
    if rotor is None and hub is None:
        hub, rotor = 95.0, 92.0
    elif rotor is None:
        rotor = 0.95 * hub
    elif hub is None:
        hub = max(0.95 * rotor + 8, rotor / 2 + 12)
    # (No hub stands over ~170 m yet: higher figures are tip heights, or
    # feet; and blades clear of the ground, within what a byte holds.)
    if hub > 175:
        hub = max(hub - rotor / 2, 60.0) if hub - rotor / 2 <= 175 else 95.0
    rotor = min(rotor, 1.25 * hub + 30)
    hub = max(hub, rotor / 2 + 6)
    return int(round(min(hub, 255))), int(round(min(rotor, 255)))


def main():
    src = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else "assets/wind_turbines.bin.gz"
    cells = defaultdict(list)
    seen = set()
    n_in = n_tags = 0
    with open(src, newline="") as f:
        rd = csv.DictReader(f, delimiter="\t", quoting=csv.QUOTE_NONE)
        for row in rd:
            if "?wkt" in row:
                # (QLever: POINT(lon lat), values in quotes.)
                m = re.match(r"POINT\(([-\d.eE]+) ([-\d.eE]+)\)", row["?wkt"] or "")
                if not m:
                    continue
                q = lambda k: (row.get(k) or "").strip('"')
                row = {"@lat": m.group(2), "@lon": m.group(1), "height:hub": q("?hub"), "rotor:diameter": q("?rotor"),
                       "height": q("?height"), "generator:output:electricity": q("?power")}
            try:
                lat, lon = float(row["@lat"]), float(row["@lon"])
            except (KeyError, ValueError):
                continue
            n_in += 1
            hub, rotor = size(row)
            if hub < 15:
                continue
            if row.get("height:hub") or row.get("rotor:diameter"):
                n_tags += 1
            la, lo = math.floor(lat), math.floor(lon)
            q = (min(int((lat - la) * 65536), 65535), min(int((lon - lo) * 65536), 65535))
            if (la, lo, q) in seen:
                continue
            seen.add((la, lo, q))
            cells[(la, lo)].append((q[0], q[1], hub, rotor))
    b = bytearray(b"FFWT")
    b += struct.pack("<II", 1, len(cells))
    keys = sorted(cells)
    for k in keys:
        b += struct.pack("<hhI", k[0], k[1], len(cells[k]))
    total = 0
    for k in keys:
        for t in sorted(cells[k]):
            b += struct.pack("<HHBB", *t)
            total += 1
    with gzip.open(out, "wb", compresslevel=9) as f:
        f.write(bytes(b))
    print(f"{n_in} turbines read, {total} packed in {len(cells)} cells ({n_tags} with their own sizes): {out}")


if __name__ == "__main__":
    main()
