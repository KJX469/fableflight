//! Text to Kokoro's tokens, from the pronunciations in
//! `assets/atc_lexicon.tsv.gz` (built by `assets/mkatclex.py`), with no
//! phonemiser at run time.
//!
//! The text is ATC speech as the phrase builder writes it: words in plain
//! letters ("one two three", "niner"), with commas and full stops for the
//! pauses. Each word is looked up; a word that isn't there is spelled out
//! letter by letter, and digits are read one at a time, as controllers do.

use rustc_hash::FxHashMap;
use std::io::Read;
use std::sync::OnceLock;

/// Kokoro takes at most this many tokens at once, padding included.
pub const MAX_TOKENS: usize = 510;

pub struct Lexicon {
    vocab: FxHashMap<char, i64>,
    words: FxHashMap<String, String>,
}

pub(crate) const DIGITS: [&str; 10] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "niner"];

impl Lexicon {
    pub fn get() -> &'static Lexicon {
        static LEX: OnceLock<Lexicon> = OnceLock::new();
        LEX.get_or_init(|| {
            let mut text = String::new();
            let _ = flate2::read::GzDecoder::new(&include_bytes!("../../assets/atc_lexicon.tsv.gz")[..]).read_to_string(&mut text);
            Lexicon::parse(&text)
        })
    }

    fn parse(text: &str) -> Lexicon {
        let (mut vocab, mut words) = (FxHashMap::default(), FxHashMap::default());
        let mut in_words = false;
        for line in text.lines() {
            match line {
                "#vocab" => in_words = false,
                "#words" => in_words = true,
                _ => {
                    let Some((k, v)) = line.split_once('\t') else { continue };
                    if in_words {
                        words.insert(k.to_owned(), v.to_owned());
                    } else if let (Some(c), Ok(id)) = (k.chars().next(), v.parse()) {
                        vocab.insert(c, id);
                    }
                }
            }
        }
        Lexicon { vocab, words }
    }

    #[cfg(test)]
    fn knows(&self, word: &str) -> bool {
        self.words.contains_key(&word.to_lowercase())
    }

    /// The phonemes for a message: words separated by spaces, with its
    /// punctuation (, . ? !) kept for the pauses and intonation.
    pub fn phonemes(&self, text: &str) -> String {
        let mut out = String::new();
        let mut word = String::new();
        let flush = |word: &mut String, out: &mut String| {
            if word.is_empty() {
                return;
            }
            for p in self.word(word) {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                out.push_str(p);
            }
            word.clear();
        };
        for c in text.chars() {
            if c.is_alphanumeric() || c == '\'' {
                // a digit after letters (or letters after digits) starts a new word: "FL350" reads "F L three five zero"
                if word.chars().last().is_some_and(|l| l.is_ascii_digit() != c.is_ascii_digit()) {
                    flush(&mut word, &mut out);
                }
                word.extend(c.to_lowercase());
            } else {
                flush(&mut word, &mut out);
                if matches!(c, ',' | '.' | '?' | '!' | ';' | ':') {
                    out.truncate(out.trim_end().len());
                    out.push(c);
                    out.push(' ');
                } else if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
            }
        }
        flush(&mut word, &mut out);
        out.trim().to_owned()
    }

    /// One word's phonemes, or the phonemes of its letters or digits.
    fn word<'a>(&'a self, w: &str) -> Vec<&'a str> {
        let w = w.trim_matches('\'');
        if let Some(p) = self.words.get(w) {
            return vec![p];
        }
        w.chars()
            .filter_map(|c| match c.to_digit(10) {
                Some(d) => self.words.get(DIGITS[d as usize]),
                None => self.words.get(&format!("letter:{}", c.to_ascii_lowercase())),
            })
            .map(String::as_str)
            .collect()
    }

    /// The token ids for phonemes (symbols outside the vocabulary dropped),
    /// without the padding.
    pub fn tokens(&self, phonemes: &str) -> Vec<i64> {
        phonemes.chars().filter_map(|c| self.vocab.get(&c).copied()).collect()
    }

    /// A message as Kokoro inputs: one or more runs of tokens, each padded
    /// with a 0 at each end and short enough for one inference. A long message
    /// is split after a sentence or a comma.
    pub fn chunks(&self, text: &str) -> Vec<Vec<i64>> {
        let room = MAX_TOKENS - 2;
        let ph = self.phonemes(text);
        let mut out: Vec<Vec<i64>> = Vec::new();
        let mut cur: Vec<i64> = Vec::new();
        // pieces end at a pause; a piece too long on its own is cut at a space
        let mut pieces: Vec<String> = Vec::new();
        let mut piece = String::new();
        for c in ph.chars() {
            piece.push(c);
            if matches!(c, ',' | '.' | '?' | '!' | ';' | ':') {
                pieces.push(std::mem::take(&mut piece));
            }
        }
        pieces.push(piece);
        for p in pieces {
            let mut t = self.tokens(&p);
            while t.len() > room {
                let cut = t[..room].iter().rposition(|&id| Some(&id) == self.vocab.get(&' ')).filter(|&i| i > 0).unwrap_or(room);
                let rest = t.split_off(cut);
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                out.push(t);
                t = rest;
            }
            if cur.len() + t.len() > room {
                out.push(std::mem::take(&mut cur));
            }
            cur.extend(t);
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        let space = self.vocab.get(&' ').copied();
        out.into_iter()
            .map(|mut t| {
                while t.first().is_some() && t.first().copied() == space {
                    t.remove(0);
                }
                while t.last().is_some() && t.last().copied() == space {
                    t.pop();
                }
                let mut padded = Vec::with_capacity(t.len() + 2);
                padded.push(0);
                padded.extend(t);
                padded.push(0);
                padded
            })
            .filter(|t| t.len() > 2)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_lexicon() {
        let l = Lexicon::get();
        assert!(l.vocab.len() > 100, "{} symbols", l.vocab.len());
        for w in ["tower", "niner", "cleared", "takeoff", "runway", "francisco", "letter:z", "heathrow"] {
            assert!(l.knows(w), "no {w}");
        }
    }

    #[test]
    fn phonemes_keep_the_pauses() {
        let l = Lexicon::get();
        let p = l.phonemes("Fable one two three, San Francisco Tower, cleared for takeoff.");
        assert!(p.starts_with(&l.words["fable"]), "{p}");
        assert_eq!(p.matches(',').count(), 2, "{p}");
        assert!(p.ends_with('.'), "{p}");
        assert!(!p.contains(" ,") && !p.contains("  "), "{p}");
    }

    #[test]
    fn spells_what_it_doesnt_know() {
        let l = Lexicon::get();
        // digits one at a time, unknown words by their letters, and letters after digits split off
        assert_eq!(l.phonemes("28"), format!("{} {}", l.words["two"], l.words["eight"]));
        assert_eq!(l.phonemes("KZQX"), ["k", "z", "q", "x"].map(|c| l.words[&format!("letter:{c}")].clone()).join(" "));
        assert_eq!(l.phonemes("28L"), format!("{} {} {}", l.words["two"], l.words["eight"], l.words["letter:l"]));
    }

    #[test]
    fn chunks_fit_one_inference() {
        let l = Lexicon::get();
        let short = l.chunks("Fable one two three, runway two eight right, cleared to land.");
        assert_eq!(short.len(), 1);
        assert!(short[0][0] == 0 && *short[0].last().unwrap() == 0);
        let long = "Fable one two three, climb and maintain one zero thousand, turn left heading two four zero. ".repeat(12);
        let chunks = l.chunks(&long);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|c| c.len() <= MAX_TOKENS && c[0] == 0 && *c.last().unwrap() == 0));
        let all: usize = chunks.iter().map(|c| c.len() - 2).sum();
        let one = l.tokens(&l.phonemes(&long)).len();
        assert!(one - all < chunks.len() * 2, "{all} of {one} tokens kept");
    }
}
