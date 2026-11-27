//! Controllers' voices: Kokoro-82M (an 82M-parameter text-to-speech model)
//! run by Microsoft's ONNX Runtime, loaded from the voice pack (install.rs).
//!
//! One worker thread does all of it, so the game never waits: `Tts::say`
//! queues a message and hands back a receiver; the speech arrives on it when
//! it's ready, or at once from the cache (`<cache>/fablegame/atc/`, one WAV
//! per message, voice and speed). The model is loaded on the first message
//! and let go after a few minutes of silence on the radio (it holds about
//! 600 MB while loaded), and synthesis runs on two threads: faster than real
//! time even on an old quad-core, and leaving the rest to the game.
//!
//! Text goes to tokens through the lexicon (lexicon.rs): no phonemiser.

use super::install;
use super::lexicon::{Lexicon, MAX_TOKENS};
use crossbeam_channel::{Receiver, Sender};
use rustc_hash::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

/// Kokoro speaks at 24 kHz.
pub const RATE: u32 = 24_000;
/// Threads one synthesis may use.
const THREADS: usize = 2;
/// The model is unloaded after this long without a message.
const IDLE: Duration = Duration::from_secs(180);
/// The cache keeps at most this many messages (the oldest go first).
const CACHE_MAX: usize = 400;

/// Kokoro's English voices that sound like they belong on the radio:
/// American (a) and British (b), female (f) and male (m).
pub const VOICES: [&str; 12] = [
    "am_michael", "am_adam", "am_eric", "am_liam", "am_onyx", "bm_george", "bm_lewis", "bm_daniel", "af_sarah", "af_nicole", "bf_emma", "bf_isabella",
];

/// A synthesised message: mono samples at `RATE`.
#[derive(Clone)]
pub struct Speech {
    pub samples: Arc<[f32]>,
    pub rate: u32,
}

struct Job {
    text: String,
    voice: &'static str,
    speed: f32,
    reply: Sender<Result<Speech, String>>,
}

pub struct Tts {
    jobs: Sender<Job>,
}

impl Tts {
    /// The one speaker, its worker started on first use.
    pub fn get() -> &'static Tts {
        static TTS: OnceLock<Tts> = OnceLock::new();
        TTS.get_or_init(|| {
            let (tx, rx) = crossbeam_channel::unbounded();
            std::thread::Builder::new().name("atc-tts".into()).spawn(move || worker(rx)).expect("spawn the ATC voice thread");
            Tts { jobs: tx }
        })
    }

    /// Whether the voice pack is there to speak with.
    pub fn available() -> bool {
        install::state() == install::State::Installed
    }

    /// Queues `text` in `voice` (one of `VOICES`) at `speed` (1 is Kokoro's
    /// normal pace; controllers talk at about 1.1–1.2).
    pub fn say(&self, text: &str, voice: &'static str, speed: f32) -> Receiver<Result<Speech, String>> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let job = Job { text: text.to_owned(), voice, speed: speed.clamp(0.5, 2.0), reply: tx };
        if let Err(e) = self.jobs.send(job) {
            let _ = e.0.reply.send(Err("the ATC voice thread has stopped".into()));
        }
        rx
    }
}

/// The loaded model and the voices read so far.
struct Engine {
    session: ort::session::Session,
    voices_file: PathBuf,
    styles: FxHashMap<&'static str, Vec<f32>>,
}

fn worker(rx: Receiver<Job>) {
    let mut engine: Option<Engine> = None;
    loop {
        let job = match rx.recv_timeout(IDLE) {
            Ok(j) => j,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                if engine.take().is_some() {
                    log::info!("ATC voices: unloaded after {}s of radio silence", IDLE.as_secs());
                }
                continue;
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return,
        };
        let r = speak(&mut engine, &job);
        if let Err(e) = &r {
            log::warn!("ATC voice: {e}");
        }
        let _ = job.reply.send(r);
    }
}

fn speak(engine: &mut Option<Engine>, job: &Job) -> Result<Speech, String> {
    let lex = Lexicon::get();
    let chunks = lex.chunks(&job.text);
    if chunks.is_empty() {
        return Err(format!("nothing to say in {:?}", job.text));
    }
    let cache = cache_path(job, &chunks);
    if let Some(s) = cache.as_deref().and_then(read_cache) {
        return Ok(s);
    }
    if engine.is_none() {
        *engine = Some(load()?);
    }
    let e = engine.as_mut().unwrap();
    let mut samples = Vec::new();
    for tokens in &chunks {
        samples.extend(e.run(tokens, job.voice, job.speed)?);
    }
    if let Some(p) = &cache {
        write_cache(p, &samples);
    }
    Ok(Speech { samples: samples.into(), rate: RATE })
}

/// Loads ONNX Runtime (once per run) and the model.
fn load() -> Result<Engine, String> {
    let p = install::paths().ok_or("the ATC voice pack isn't installed")?;
    let t = std::time::Instant::now();
    let session = install::session(&p.model, THREADS).map_err(|e| format!("Kokoro: {e}"))?;
    log::info!("ATC voices: Kokoro loaded in {:.1}s", t.elapsed().as_secs_f32());
    Ok(Engine { session, voices_file: p.voices, styles: FxHashMap::default() })
}

impl Engine {
    fn run(&mut self, tokens: &[i64], voice: &'static str, speed: f32) -> Result<Vec<f32>, String> {
        use ort::value::Tensor;
        if !self.styles.contains_key(voice) {
            self.styles.insert(voice, read_voice(&self.voices_file, voice)?);
        }
        // a voice is a style vector for each length of input: the row for this one
        let n = tokens.len() - 2;
        let style = self.styles[voice][n.min(MAX_TOKENS - 1) * 256..][..256].to_vec();
        let err = |e: ort::Error| format!("Kokoro: {e}");
        let out = self
            .session
            .run(ort::inputs![
                "tokens" => Tensor::from_array(([1usize, tokens.len()], tokens.to_vec())).map_err(err)?,
                "style" => Tensor::from_array(([1usize, 256], style)).map_err(err)?,
                "speed" => Tensor::from_array(([1usize], vec![speed])).map_err(err)?,
            ])
            .map_err(err)?;
        let audio = out["audio"].try_extract_tensor::<f32>().map_err(err)?.1.to_vec();
        Ok(audio)
    }
}

/// One voice out of the voices file: a NumPy archive (a zip of `.npy`
/// arrays), each voice 510 rows of 256 little-endian f32.
fn read_voice(file: &Path, voice: &str) -> Result<Vec<f32>, String> {
    use std::io::Read;
    if !VOICES.contains(&voice) {
        return Err(format!("no voice {voice}"));
    }
    let f = std::fs::File::open(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    let mut e = zip.by_name(&format!("{voice}.npy")).map_err(|e| format!("{voice}: {e}"))?;
    let mut b = Vec::new();
    e.read_to_end(&mut b).map_err(|e| e.to_string())?;
    npy_f32(&b).ok_or_else(|| format!("{voice}: not a float32 array"))
}

/// The data of a NumPy `.npy` file holding little-endian f32.
fn npy_f32(b: &[u8]) -> Option<Vec<f32>> {
    if b.len() < 10 || &b[..6] != b"\x93NUMPY" {
        return None;
    }
    let (len, start) = match b[6] {
        1 => (u16::from_le_bytes([b[8], b[9]]) as usize, 10),
        _ => (u32::from_le_bytes(b.get(8..12)?.try_into().ok()?) as usize, 12),
    };
    let header = std::str::from_utf8(b.get(start..start + len)?).ok()?;
    if !header.contains("'descr': '<f4'") || header.contains("'fortran_order': True") {
        return None;
    }
    Some(b[start + len..].chunks_exact(4).map(|c| f32::from_le_bytes(c.try_into().unwrap())).collect())
}

/// `<cache>/fablegame/atc/<hash>.wav`, the hash of everything that shapes the sound.
fn cache_path(job: &Job, chunks: &[Vec<i64>]) -> Option<PathBuf> {
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    ctx.update(install::MODEL.sha256.as_bytes());
    ctx.update(job.voice.as_bytes());
    ctx.update(&job.speed.to_le_bytes());
    for c in chunks {
        c.iter().for_each(|t| ctx.update(&t.to_le_bytes()));
        ctx.update(b"|");
    }
    let hash: String = ctx.finish().as_ref()[..12].iter().map(|x| format!("{x:02x}")).collect();
    Some(dirs::cache_dir()?.join("fablegame").join("atc").join(format!("{hash}.wav")))
}

fn read_cache(p: &Path) -> Option<Speech> {
    let (samples, rate) = crate::audio::radio::wav_mono(&std::fs::read(p).ok()?)?;
    Some(Speech { samples: samples.into(), rate })
}

fn write_cache(p: &Path, samples: &[f32]) {
    let Some(dir) = p.parent() else { return };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let _ = std::fs::write(p, crate::audio::radio::wav_bytes(samples, RATE));
    // keep the cache bounded: the oldest messages go
    if let Ok(rd) = std::fs::read_dir(dir) {
        let mut files: Vec<(std::time::SystemTime, PathBuf)> = rd
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "wav"))
            .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
            .collect();
        if files.len() > CACHE_MAX {
            files.sort();
            for (_, f) in &files[..files.len() - CACHE_MAX] {
                let _ = std::fs::remove_file(f);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn npy(descr: &str, data: &[f32]) -> Vec<u8> {
        let header = format!("{{'descr': '{descr}', 'fortran_order': False, 'shape': ({},), }}", data.len());
        let h = format!("{header}{}\n", " ".repeat(63 - (10 + header.len()) % 64));
        let mut b = b"\x93NUMPY\x01\x00".to_vec();
        b.extend_from_slice(&(h.len() as u16).to_le_bytes());
        b.extend_from_slice(h.as_bytes());
        data.iter().for_each(|x| b.extend_from_slice(&x.to_le_bytes()));
        b
    }

    #[test]
    fn reads_npy() {
        let b = npy("<f4", &[1.5, -2.0]);
        assert_eq!((b.len() - 8) % 64, 0, "header padded to 64 bytes");
        assert_eq!(npy_f32(&b), Some(vec![1.5, -2.0]));
        assert_eq!(npy_f32(&npy("<f8", &[1.5])), None);
        assert_eq!(npy_f32(b"not numpy"), None);
    }

    #[test]
    fn cache_key_follows_the_voice_and_text() {
        let lex = Lexicon::get();
        let job = |text: &str, voice, speed| Job { text: text.into(), voice, speed, reply: crossbeam_channel::bounded(1).0 };
        let key = |j: &Job| cache_path(j, &lex.chunks(&j.text));
        let a = key(&job("cleared to land", "bf_emma", 1.1));
        assert_eq!(a, key(&job("Cleared to land", "bf_emma", 1.1)), "same sounds, same file");
        assert_ne!(a, key(&job("cleared to land", "am_adam", 1.1)));
        assert_ne!(a, key(&job("cleared to land", "bf_emma", 1.2)));
        assert_ne!(a, key(&job("cleared for takeoff", "bf_emma", 1.1)));
    }

    /// Speak with the installed voice pack (`fablegame --atc-install`), through the radio:
    /// `ATC_SAY="Fable one two three, cleared for takeoff." [ATC_VOICE=bf_emma] [ATC_OUT=atc.wav]
    /// [RADIO_STATION=ground|tower|radar] [RADIO_SIGNAL=0..1] cargo test --release --bin fablegame atc_say -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn atc_say() {
        let text = std::env::var("ATC_SAY").unwrap_or_else(|_| "Fable one two three, San Francisco Tower, runway two eight right, cleared for takeoff.".into());
        let voice = std::env::var("ATC_VOICE").ok().and_then(|v| VOICES.iter().find(|x| **x == v).copied()).unwrap_or("bf_emma");
        let t = std::time::Instant::now();
        let s = Tts::get().say(&text, voice, 1.1).recv().unwrap().expect("speech");
        let secs = s.samples.len() as f32 / s.rate as f32;
        println!("{secs:.2}s of speech in {:.2}s (model load included)", t.elapsed().as_secs_f32());
        let t = std::time::Instant::now();
        let again = Tts::get().say(&text, voice, 1.1).recv().unwrap().expect("speech");
        println!("again, from the cache: {:.3}s", t.elapsed().as_secs_f32());
        assert_eq!(again.samples.len(), s.samples.len());
        let st = match std::env::var("RADIO_STATION").as_deref() {
            Ok("ground") => crate::audio::radio::Station::GROUND,
            Ok("radar") => crate::audio::radio::Station::RADAR,
            _ => crate::audio::radio::Station::TOWER,
        };
        let signal = std::env::var("RADIO_SIGNAL").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
        let x = crate::audio::radio::Transmission::new(s.samples.to_vec(), s.rate, st, signal, 0x5eed).render();
        let out = std::env::var("ATC_OUT").unwrap_or_else(|_| "atc.wav".into());
        std::fs::write(&out, crate::audio::radio::wav_bytes(&x, crate::audio::radio::OUT_RATE)).unwrap();
        println!("written to {out}");
    }
}
