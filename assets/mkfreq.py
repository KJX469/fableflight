# radio frequencies for the airports in airports.tsv.gz, from ourairports
# (public domain): python mkfreq.py <dir with airport-frequencies.csv> frequencies.tsv.gz
import csv, gzip, sys
src, out = sys.argv[1], sys.argv[2]
known = set()
for line in gzip.open('airports.tsv.gz', 'rt', encoding='utf-8'):
    f = line.rstrip('\n').split('\t')
    if f[0] == 'A': known.add(f[1])
# the kinds the game's radio works with, by ourairports' type.
kinds = {'ATIS': 'ATIS', 'AWOS': 'ATIS', 'ASOS': 'ATIS', 'AFIS': 'AFIS', 'CLD': 'DEL', 'DEL': 'DEL', 'CLNC': 'DEL',
         'GND': 'GND', 'TWR': 'TWR', 'CTAF': 'CTAF', 'UNIC': 'UNIC', 'A/G': 'UNIC', 'RDO': 'UNIC', 'INFO': 'UNIC',
         'APP': 'APP', 'ARR': 'APP', 'DEP': 'DEP', 'CNTR': 'CTR', 'ACC': 'CTR', 'RDR': 'APP'}
def clean(s): return ' '.join(s.replace('\t', ' ').split())[:40]
rows = set()
for r in csv.DictReader(open(src + '/airport-frequencies.csv', encoding='utf-8')):
    k = kinds.get(r['type'].strip().upper())
    if not k or r['airport_ident'] not in known: continue
    try: mhz = float(r['frequency_mhz'])
    except: continue
    if not 108.0 <= mhz <= 137.0: continue
    rows.add((r['airport_ident'], k, '%.3f' % mhz, clean(r['description'])))
data = ''.join('\t'.join(x) + '\n' for x in sorted(rows)).encode('utf-8')
open(out, 'wb').write(gzip.compress(data, 9, mtime=0))
print(len(rows), 'frequencies', len(data), 'bytes raw')
