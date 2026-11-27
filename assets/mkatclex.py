#!/usr/bin/env python3
"""Builds assets/atc_lexicon.tsv.gz: the pronunciations the ATC voices use.

Kokoro (src/atc/tts.rs) reads phonemes, not text. The usual way to get them
is espeak-ng at run time, a GPL C library that is awkward to build on every
platform. But ATC speech is a closed vocabulary: the phraseology, numbers,
the phonetic alphabet, and the names of the places controllers call
themselves after. So this script asks espeak-ng once, here, for every word
the game can say, and the game looks them up. Anything else is spelled out.

The file is gzipped text:
    #vocab            then one "<symbol>\t<token id>" per line: Kokoro's vocabulary
    #words            then one "<word>\t<phonemes>" per line, the word in lower case
Letters spelled out are the words "letter:a" .. "letter:z".

Needs `pip install kokoro-onnx` (which brings phonemizer and espeak-ng).
Run from the repository root:  python assets/mkatclex.py
"""
import gzip
import re
import sys

from kokoro_onnx.config import DEFAULT_VOCAB
from kokoro_onnx.tokenizer import Tokenizer
import phonemizer

# The phraseology (ICAO Doc 4444 / FAA JO 7110.65 wording the game uses),
# numbers and units, and common words around them.
PHRASEOLOGY = """
zero one two three four five six seven eight nine niner ten eleven twelve hundred thousand
decimal point flight level heading degrees knots feet miles altitude
alfa alpha bravo charlie delta echo foxtrot golf hotel india juliett juliet kilo lima mike
november oscar papa quebec romeo sierra tango uniform victor whiskey xray x-ray yankee zulu
tower ground approach departure center centre control clearance delivery radio information
atis traffic advisory unicom radar contact identified services terminated
runway runways taxiway taxi via hold short of position line up and wait cross
cleared for takeoff land landing option touch go low approach go around missed
the a an to at on in into from with without after before behind follow
left right center straight ahead turn climb descend maintain expedite reduce increase speed
wind calm gusting variable visibility ceiling scattered broken overcast few clear sky
altimeter qnh qfe squawk ident mode code transponder emergency mayday pan
contact monitor frequency say again correction affirm affirmative negative roger wilco
unable standby report established localizer glideslope glide path ils rnav vor ndb gps
visual instrument vectors vector direct proceed resume own navigation sid star
expect further clearance when ready leaving passing reaching
good day evening morning afternoon night welcome thanks thank you bye
caution wake turbulence heavy super vehicle ramp apron gate stand parking
holding pattern inbound outbound course final base downwind crosswind upwind leg
extend make three sixty traffic in sight not number sequence behind company
continue approved request requested present position airborne
fable please check stop immediately
able advise below call cancel change departed departing deviation deviations favoured filed
flights following landed minutes off out pilot possible reported shortly taxiing taxiways this
took use vacate vfr am over your time zulu degrees by ready niner
norcal socal potomac chicago new york boston jacksonville
"""

LETTER_NAMES = {
    "a": "ay", "b": "bee", "c": "see", "d": "dee", "e": "ee", "f": "eff", "g": "gee",
    "h": "aitch", "i": "eye", "j": "jay", "k": "kay", "l": "el", "m": "em", "n": "en",
    "o": "oh", "p": "pee", "q": "cue", "r": "ar", "s": "ess", "t": "tee", "u": "you",
    "v": "vee", "w": "double you", "x": "ex", "y": "why", "z": "zee",
}


def station_words():
    """Words in the names and towns of medium and large airports: what
    controllers call themselves ("San Francisco Tower")."""
    words = set()
    with gzip.open("assets/airports.tsv.gz", "rt", encoding="utf-8") as f:
        for line in f:
            c = line.rstrip("\n").split("\t")
            if c[0] == "A" and c[3] in ("M", "L"):
                for field in (c[7], c[8]):
                    words.update(w.lower() for w in re.findall(r"[^\W\d_]+", field))
    # a lone letter is spelled out ("letter:l"), never read as a word
    return {w for w in words if len(w) > 1}


def main():
    Tokenizer()  # points phonemizer at kokoro-onnx's espeak-ng
    base = set(re.findall(r"[a-z]+", PHRASEOLOGY))
    words = sorted(base | station_words())
    keys = words + [f"letter:{c}" for c in LETTER_NAMES]
    texts = words + list(LETTER_NAMES.values())
    ph = phonemizer.phonemize(texts, "en-us", preserve_punctuation=False, with_stress=True, njobs=4)
    with gzip.open("assets/atc_lexicon.tsv.gz", "wt", encoding="utf-8", compresslevel=9) as out:
        out.write("#vocab\n")
        for sym, i in sorted(DEFAULT_VOCAB.items(), key=lambda kv: kv[1]):
            out.write(f"{sym}\t{i}\n")
        out.write("#words\n")
        n = 0
        for k, p in zip(keys, ph):
            p = "".join(ch for ch in p.strip() if ch in DEFAULT_VOCAB)
            # alone, some phraseology words ("one") get only a secondary stress
            # that they carry in full in a sentence: give them it back (place
            # names keep theirs: "San" is unstressed in "San Francisco")
            if k in base and "ˈ" not in p and "ˌ" in p:
                p = p.replace("ˌ", "ˈ", 1)
            if p:
                out.write(f"{k}\t{p}\n")
                n += 1
    print(f"{n} words, {len(DEFAULT_VOCAB)} symbols", file=sys.stderr)


if __name__ == "__main__":
    main()
