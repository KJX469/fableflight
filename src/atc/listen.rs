//! the pilot's own calls, spoken: push to talk records the mic, whisper
//! (tiny.en as onnx, from sherpa-onnx's export, mit) writes down what was
//! said on its own thread, and `intent` matches the words to a call that makes
//! sense on the frequency.

use super::install;
use super::Req;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, OnceLock};

/// whisper hears at 16 khz, in 30 s windows of 3000 frames.
pub const RATE: u32 = 16_000;
const N_FFT: usize = 400;
/// the spectrum's bins, dc to nyquist.
const BINS: usize = N_FFT / 2 + 1;
const HOP: usize = 160;
const N_MELS: usize = 80;
const FRAMES: usize = 3000;
/// the longest call kept, s.
const LONGEST: f32 = 25.0;
/// whisper is unloaded after this long without a call.
const IDLE: std::time::Duration = std::time::Duration::from_secs(180);

/// the microphone push to talk records, by name (empty: the system's default).
static MIC_NAME: parking_lot::Mutex<String> = parking_lot::Mutex::new(String::new());
/// the microphones there are, as last looked up, and when.
static MICS: parking_lot::Mutex<(Vec<String>, Option<std::time::Instant>)> = parking_lot::Mutex::new((Vec::new(), None));

/// records from this microphone from the next call on (empty: the system's default).
pub fn set_microphone(name: &str) {
    *MIC_NAME.lock() = name.to_string();
}

/// the microphones there are, by name. looked up on a thread now and then
/// (it can take a moment), so the first call may come back empty.
pub fn microphones() -> Vec<String> {
    let mut m = MICS.lock();
    if m.1.is_none_or(|t| t.elapsed().as_secs() > 5) {
        m.1 = Some(std::time::Instant::now());
        let _ = std::thread::Builder::new().name("atc-mics".into()).spawn(|| {
            use cpal::traits::{DeviceTrait, HostTrait};
            let names: Vec<String> = cpal::default_host().input_devices().map(|l| l.filter_map(|d| d.description().ok().map(|x| x.name().to_string())).collect()).unwrap_or_default();
            MICS.lock().0 = names;
        });
    }
    m.0.clone()
}

/// the microphone, on its own thread (a stream may not leave the thread it was made on).
pub struct Mic {
    cmd: Sender<bool>,
    done: Receiver<(Vec<f32>, u32)>,
    pub on: bool,
}

impl Mic {
    pub fn new() -> Self {
        let (cmd, rx) = crossbeam_channel::unbounded::<bool>();
        let (tx, done) = crossbeam_channel::unbounded();
        let _ = std::thread::Builder::new().name("atc-mic".into()).spawn(move || mic_thread(rx, tx));
        Self { cmd, done, on: false }
    }

    /// starts or stops recording; what was said arrives from `take` after a stop.
    pub fn set(&mut self, on: bool) {
        if on != self.on {
            self.on = on;
            let _ = self.cmd.send(on);
        }
    }

    /// a finished recording, mono at its own rate.
    pub fn take(&self) -> Option<(Vec<f32>, u32)> {
        self.done.try_recv().ok()
    }
}

fn mic_thread(rx: Receiver<bool>, tx: Sender<(Vec<f32>, u32)>) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let buf: Arc<parking_lot::Mutex<Vec<f32>>> = Arc::default();
    let mut stream: Option<(cpal::Stream, u32)> = None;
    while let Ok(on) = rx.recv() {
        if on && stream.is_none() {
            buf.lock().clear();
            let open = || -> Result<(cpal::Stream, u32), String> {
                let host = cpal::default_host();
                let default = host.default_input_device().ok_or("no microphone")?;
                // the one chosen in the settings, else the default as listed, not windows'
                // "default device" handle: that one opens through an async call that fails
                // on this thread's com apartment.
                let want = MIC_NAME.lock().clone();
                let id = default.id().ok();
                let listed = |pick: &dyn Fn(&cpal::Device) -> bool| host.input_devices().ok().and_then(|mut l| l.find(|d| pick(d)));
                let named = (!want.is_empty()).then(|| listed(&|d| d.description().is_ok_and(|x| x.name() == want))).flatten();
                let dev = named.or_else(|| listed(&|d| id.is_some() && d.id().ok() == id)).unwrap_or(default);
                let cfg = dev.default_input_config().map_err(|e| e.to_string())?;
                let rate = cfg.sample_rate();
                let s = match cfg.sample_format() {
                    cpal::SampleFormat::F32 => input::<f32>(&dev, &cfg, &buf),
                    cpal::SampleFormat::I16 => input::<i16>(&dev, &cfg, &buf),
                    cpal::SampleFormat::I32 => input::<i32>(&dev, &cfg, &buf),
                    cpal::SampleFormat::U16 => input::<u16>(&dev, &cfg, &buf),
                    cpal::SampleFormat::U8 => input::<u8>(&dev, &cfg, &buf),
                    f => return Err(format!("microphone sample format {f:?}")),
                }?;
                s.play().map_err(|e| e.to_string())?;
                Ok((s, rate))
            };
            match open() {
                Ok(s) => stream = Some(s),
                Err(e) => {
                    log::warn!("push to talk: {e}");
                    let _ = tx.send((Vec::new(), RATE));
                }
            }
        } else if !on {
            if let Some((s, rate)) = stream.take() {
                drop(s);
                let _ = tx.send((std::mem::take(&mut *buf.lock()), rate));
            }
        }
    }
}

/// a stream that keeps the mic as mono f32, whatever the device sends, up to `LONGEST`.
fn input<T: cpal::SizedSample>(dev: &cpal::Device, cfg: &cpal::SupportedStreamConfig, buf: &Arc<parking_lot::Mutex<Vec<f32>>>) -> Result<cpal::Stream, String>
where
    f32: cpal::FromSample<T>,
{
    use cpal::traits::DeviceTrait;
    use cpal::Sample;
    let ch = cfg.channels().max(1) as usize;
    let max = (cfg.sample_rate() as f32 * LONGEST) as usize;
    let b = buf.clone();
    dev.build_input_stream(
        cfg.config(),
        move |d: &[T], _: &_| {
            let mut b = b.lock();
            let room = max.saturating_sub(b.len());
            b.extend(d.chunks(ch).take(room).map(|f| f.iter().map(|&x| f32::from_sample(x)).sum::<f32>() / ch as f32));
        },
        |e| log::warn!("microphone: {e}"),
        None,
    )
    .map_err(|e| e.to_string())
}

/// mono at any rate to 16 khz, linear (whisper's own front end low passes at 8 khz).
pub fn resample(x: &[f32], rate: u32) -> Vec<f32> {
    if rate == RATE || x.is_empty() {
        return x.to_vec();
    }
    // a box filter first when coming down from far above, so the top octave does not fold back.
    let k = (rate / RATE).max(1) as usize;
    let smooth: Vec<f32> = if k > 1 { x.windows(k).map(|w| w.iter().sum::<f32>() / k as f32).collect() } else { x.to_vec() };
    let step = rate as f64 / RATE as f64;
    let n = (smooth.len() as f64 / step) as usize;
    (0..n)
        .map(|i| {
            let p = i as f64 * step;
            let j = p as usize;
            let f = (p - j as f64) as f32;
            let a = smooth[j.min(smooth.len() - 1)];
            let b = smooth[(j + 1).min(smooth.len() - 1)];
            a + (b - a) * f
        })
        .collect()
}

/// whisper's log mel spectrogram: 80 bins x 3000 frames, row major by bin.
pub fn mel(audio: &[f32]) -> Vec<f32> {
    static TABLES: OnceLock<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>)> = OnceLock::new();
    let (win, cos, sin, filt) = TABLES.get_or_init(|| {
        let win: Vec<f32> = (0..N_FFT).map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / N_FFT as f32).cos()).collect();
        let mut cos = vec![0.0; BINS * N_FFT];
        let mut sin = vec![0.0; BINS * N_FFT];
        for k in 0..BINS {
            for n in 0..N_FFT {
                let a = std::f64::consts::TAU * (k * n % N_FFT) as f64 / N_FFT as f64;
                cos[k * N_FFT + n] = a.cos() as f32;
                sin[k * N_FFT + n] = a.sin() as f32;
            }
        }
        (win, cos, sin, mel_filters())
    });
    let audio = &audio[..audio.len().min(RATE as usize * 30)];
    // centred frames, the edges mirrored as torch.stft does.
    let pad = N_FFT / 2;
    let at = |i: isize| -> f32 {
        let n = audio.len() as isize;
        if n == 0 {
            return 0.0;
        }
        let mut j = i - pad as isize;
        if j < 0 {
            j = -j;
        }
        if j >= n {
            j = 2 * (n - 1) - j;
        }
        audio[j.clamp(0, n - 1) as usize]
    };
    let frames = (audio.len() / HOP).min(FRAMES);
    let mut out = vec![0.0f32; N_MELS * FRAMES];
    let mut spec = vec![0.0f32; BINS];
    let mut fr = vec![0.0f32; N_FFT];
    let mut top = f32::MIN;
    for t in 0..frames {
        for (n, v) in fr.iter_mut().enumerate() {
            *v = at((t * HOP + n) as isize) * win[n];
        }
        for (k, s) in spec.iter_mut().enumerate() {
            let (c, si) = (&cos[k * N_FFT..][..N_FFT], &sin[k * N_FFT..][..N_FFT]);
            let (mut re, mut im) = (0.0f32, 0.0f32);
            for n in 0..N_FFT {
                re += fr[n] * c[n];
                im += fr[n] * si[n];
            }
            *s = re * re + im * im;
        }
        for m in 0..N_MELS {
            let f = &filt[m * BINS..][..BINS];
            let e: f32 = f.iter().zip(&spec).map(|(a, b)| a * b).sum();
            let l = e.max(1e-10).log10();
            out[m * FRAMES + t] = l;
            top = top.max(l);
        }
    }
    // whisper's scaling; the frames past the speech stay at zero, as sherpa-onnx pads them.
    for m in 0..N_MELS {
        for t in 0..frames {
            let v = &mut out[m * FRAMES + t];
            *v = (v.max(top - 8.0) + 4.0) / 4.0;
        }
    }
    out
}

/// librosa's slaney mel filters for 16 khz, 400 point fft, 80 bins.
fn mel_filters() -> Vec<f32> {
    let to_mel = |f: f64| if f < 1000.0 { 3.0 * f / 200.0 } else { 15.0 + 27.0 * (f / 1000.0).ln() / 6.4f64.ln() };
    let to_hz = |m: f64| if m < 15.0 { 200.0 * m / 3.0 } else { 1000.0 * (6.4f64.ln() * (m - 15.0) / 27.0).exp() };
    let top = to_mel(RATE as f64 / 2.0);
    let hz: Vec<f64> = (0..N_MELS + 2).map(|i| to_hz(top * i as f64 / (N_MELS + 1) as f64)).collect();
    let mut w = vec![0.0f32; N_MELS * BINS];
    for m in 0..N_MELS {
        let norm = 2.0 / (hz[m + 2] - hz[m]);
        for k in 0..BINS {
            let f = k as f64 * RATE as f64 / N_FFT as f64;
            let lo = (f - hz[m]) / (hz[m + 1] - hz[m]);
            let hi = (hz[m + 2] - f) / (hz[m + 2] - hz[m + 1]);
            w[m * BINS + k] = (lo.min(hi).max(0.0) * norm) as f32;
        }
    }
    w
}

/// the models and what they need to run.
struct Whisper {
    enc: ort::session::Session,
    dec: ort::session::Session,
    tokens: Vec<Vec<u8>>,
    layers: usize,
    ctx: usize,
    state: usize,
    sot: Vec<i64>,
    eot: i64,
}

/// a number from the export's metadata, or `d` when it is not there.
fn meta<T: std::str::FromStr>(s: &ort::session::Session, k: &str, d: T) -> T {
    s.metadata().ok().and_then(|m| m.custom(k)).and_then(|v| v.parse().ok()).unwrap_or(d)
}

impl Whisper {
    fn load() -> Result<Whisper, String> {
        use base64::Engine;
        let p = install::paths().ok_or("the ATC voice pack isn't installed")?;
        let open = |f: &std::path::Path| install::session(f, 2).map_err(|e| format!("whisper {e}"));
        let (enc, dec) = (open(&p.ears_enc)?, open(&p.ears_dec)?);
        let (layers, ctx, state) = (meta(&enc, "n_text_layer", 4), meta(&enc, "n_text_ctx", 448), meta(&enc, "n_text_state", 384));
        let mut sot: Vec<i64> = enc.metadata().ok().and_then(|m| m.custom("sot_sequence")).map(|s| s.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_else(|| vec![50257]);
        sot.push(meta(&enc, "no_timestamps", 50362));
        let eot = meta(&enc, "eot", 50256);
        let mut tokens = Vec::new();
        for line in std::fs::read_to_string(&p.ears_tokens).map_err(|e| e.to_string())?.lines() {
            let mut it = line.split_whitespace();
            let (Some(b64), Some(id)) = (it.next(), it.next().and_then(|i| i.parse::<usize>().ok())) else { continue };
            if tokens.len() <= id {
                tokens.resize(id + 1, Vec::new());
            }
            tokens[id] = base64::engine::general_purpose::STANDARD.decode(b64).unwrap_or_default();
        }
        Ok(Whisper { enc, dec, tokens, layers, ctx, state, sot, eot })
    }

    /// 16 khz mono in, the words out.
    fn hear(&mut self, audio: &[f32]) -> Result<String, String> {
        use ort::value::{DynValue, Tensor};
        let err = |e: ort::Error| format!("whisper: {e}");
        let mel = mel(audio);
        let mut out = self.enc.run(ort::inputs!["mel" => Tensor::from_array(([1usize, N_MELS, FRAMES], mel)).map_err(err)?]).map_err(err)?;
        // the encoder's keys and values go in by reference every step, the self
        // attention cache comes back out and goes straight in again.
        let (Some(k), Some(v)) = (out.remove("n_layer_cross_k"), out.remove("n_layer_cross_v")) else { return Err("whisper: no encoder output".into()) };
        drop(out);
        let shape = [self.layers, 1, self.ctx, self.state];
        let n = self.layers * self.ctx * self.state;
        let zeros = || Tensor::from_array((shape, vec![0.0f32; n])).map(DynValue::from).map_err(err);
        let (mut sk, mut sv) = (zeros()?, zeros()?);
        let mut toks = self.sot.clone();
        let mut offset = 0i64;
        let mut said: Vec<i64> = Vec::new();
        for _ in 0..96 {
            let mut out = self
                .dec
                .run(ort::inputs![
                    "tokens" => Tensor::from_array(([1usize, toks.len()], toks.clone())).map_err(err)?,
                    "in_n_layer_self_k_cache" => sk,
                    "in_n_layer_self_v_cache" => sv,
                    "n_layer_cross_k" => &k,
                    "n_layer_cross_v" => &v,
                    "offset" => Tensor::from_array(([1usize], vec![offset])).map_err(err)?,
                ])
                .map_err(err)?;
            let (ls, logits) = out["logits"].try_extract_tensor::<f32>().map_err(err)?;
            let vocab = *ls.last().unwrap_or(&1) as usize;
            let last = &logits[logits.len() - vocab..];
            // text tokens only: timestamps and the specials past the end of text never win.
            let best = last[..=(self.eot as usize)].iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map(|x| x.0 as i64).unwrap_or(self.eot);
            (sk, sv) = match (out.remove("out_n_layer_self_k_cache"), out.remove("out_n_layer_self_v_cache")) {
                (Some(a), Some(b)) => (a, b),
                _ => return Err("whisper: no cache output".into()),
            };
            offset += toks.len() as i64;
            if best == self.eot || offset as usize >= self.ctx - 1 {
                break;
            }
            said.push(best);
            toks = vec![best];
        }
        let bytes: Vec<u8> = said.iter().flat_map(|&t| self.tokens.get(t as usize).cloned().unwrap_or_default()).collect();
        Ok(String::from_utf8_lossy(&bytes).trim().to_string())
    }
}

/// a recording at its own rate, or none to load whisper ahead of a call.
type Job = (Option<(Vec<f32>, u32)>, Sender<Result<String, String>>);

/// the listener: recordings in, words out, on its own thread so the game never waits.
pub struct Ears {
    jobs: Sender<Job>,
}

impl Ears {
    pub fn get() -> &'static Ears {
        static E: OnceLock<Ears> = OnceLock::new();
        E.get_or_init(|| {
            let (tx, rx) = crossbeam_channel::unbounded::<Job>();
            let _ = std::thread::Builder::new().name("atc-ears".into()).spawn(move || {
                let mut w: Option<Whisper> = None;
                loop {
                    let (rec, reply) = match rx.recv_timeout(IDLE) {
                        Ok(j) => j,
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                            w = None;
                            continue;
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => return,
                    };
                    let whisper = match &mut w {
                        Some(x) => x,
                        None => match Whisper::load() {
                            Ok(x) => w.insert(x),
                            Err(e) => {
                                let _ = reply.send(Err(e));
                                continue;
                            }
                        },
                    };
                    let Some((rec, rate)) = rec else { continue };
                    let t = std::time::Instant::now();
                    let r = whisper.hear(&resample(&rec, rate));
                    log::info!("push to talk: heard {:?} in {:.2}s", r, t.elapsed().as_secs_f32());
                    let _ = reply.send(r);
                }
            });
            Ears { jobs: tx }
        })
    }

    /// loads whisper while the pilot is still talking.
    pub fn warm(&self) {
        let _ = self.jobs.send((None, crossbeam_channel::bounded(1).0));
    }

    /// mono at any rate in; the words come back on the receiver.
    pub fn hear(&self, rec: Vec<f32>, rate: u32) -> Receiver<Result<String, String>> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let _ = self.jobs.send((Some((rec, rate)), tx));
        rx
    }
}

/// words as a list of plain lowercase words, numbers kept.
fn words(s: &str) -> Vec<String> {
    s.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_string).collect()
}

/// what was said, matched to one of the calls that make sense now: the one
/// whose key words were heard most. `None` when nothing fits.
pub fn intent(heard: &str, calls: &[Req]) -> Option<Req> {
    let w = words(heard);
    let has = |k: &str| {
        let k: Vec<&str> = k.split(' ').collect();
        w.windows(k.len()).any(|win| win.iter().zip(&k).all(|(a, b)| a == b))
    };
    let score = |r: Req| -> i32 {
        let keys: &[(&str, i32)] = match r {
            Req::Clearance => &[("clearance", 3), ("ifr", 2), ("cleared", 1), ("to", 0)],
            Req::Taxi => &[("taxi", 3), ("ramp", 1), ("parking", -3), ("taxiing", -1)],
            Req::Ready => &[("ready", 3), ("departure", 2), ("holding", 1), ("takeoff", 1), ("take off", 1)],
            Req::CheckIn => &[("passing", 2), ("with you", 3), ("checking in", 3), ("climbing", 2), ("level", 1), ("leveling", 1)],
            Req::FlightFollowing => &[("following", 3), ("flight following", 2), ("advisories", 2), ("vfr", 1)],
            Req::Inbound => &[("approach", 3), ("inbound", 1), ("descent", 2), ("arrival", 1)],
            Req::LandingReq => &[("landing", 3), ("inbound", 2), ("full stop", 2), ("miles", 1), ("final", 1)],
            Req::TaxiParking => &[("parking", 3), ("clear of", 2), ("ramp", 1), ("gate", 2), ("taxi", 1)],
            Req::Advisory => &[("advisory", 3), ("advisories", 1), ("traffic", 1), ("field", 1)],
            Req::AnnounceTaxi => &[("taxiing", 3), ("taxi", 1)],
            Req::AnnounceDeparture => &[("departing", 3), ("departure", 1), ("taking off", 2)],
            Req::AnnounceInbound => &[("inbound", 2), ("miles", 2), ("landing", 1)],
            Req::AnnounceFinal => &[("final", 3)],
            Req::AnnounceClear => &[("clear", 3), ("clear of", 1)],
            Req::Cancel => &[("cancel", 3), ("resume own", 3), ("own navigation", 2)],
        };
        keys.iter().map(|(k, s)| if has(k) { *s } else { 0 }).sum()
    };
    let best = calls.iter().copied().map(|r| (score(r), r)).max_by_key(|x| x.0)?;
    (best.0 >= 2).then_some(best.1)
}

/// the pilot reading back what the controller said: most of its key words heard again.
pub fn readback(heard: &str, said: &str) -> bool {
    let h = words(&super::voice::spoken(heard));
    let keys: Vec<String> = words(&super::voice::spoken(said)).into_iter().filter(|w| w.len() > 2 && !matches!(w.as_str(), "and" | "the" | "for" | "via" | "your" | "you")).collect();
    if keys.is_empty() || h.is_empty() {
        return false;
    }
    let got = keys.iter().filter(|k| h.contains(k)).count();
    got * 2 >= keys.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the real microphone opens and records (needs one; run by hand).
    #[test]
    #[ignore]
    fn atc_mic_opens() {
        let mut m = Mic::new();
        m.set(true);
        std::thread::sleep(std::time::Duration::from_millis(1200));
        m.set(false);
        let t = std::time::Instant::now();
        let got = loop {
            if let Some(r) = m.take() {
                break r;
            }
            assert!(t.elapsed().as_secs() < 5, "no recording back");
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        println!("{} samples at {} hz", got.0.len(), got.1);
        assert!(got.0.len() as f32 > 0.5 * got.1 as f32, "the mic gave nothing");
    }

    #[test]
    fn mel_matches_whisper_on_a_tone() {
        // a 1 khz tone peaks in the mel bin that holds 1 khz, and stays in whisper's range.
        let x: Vec<f32> = (0..RATE as usize).map(|i| (std::f32::consts::TAU * 1000.0 * i as f32 / RATE as f32).sin() * 0.3).collect();
        let m = mel(&x);
        let col = |t: usize| (0..N_MELS).map(|b| m[b * FRAMES + t]).collect::<Vec<f32>>();
        let c = col(50);
        let peak = c.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
        let filt = mel_filters();
        let bin1k = (1000.0 * N_FFT as f32 / RATE as f32).round() as usize;
        let best = (0..N_MELS).max_by(|&a, &b| filt[a * BINS + bin1k].total_cmp(&filt[b * BINS + bin1k])).unwrap();
        assert!((peak as i32 - best as i32).abs() <= 1, "peak {peak}, the 1 khz bin {best}");
        assert!(m.iter().all(|v| v.is_finite() && *v <= 1.6 && *v >= -1.6));
        // past the end of the audio, zero
        assert_eq!(m[FRAMES - 1], 0.0);
    }

    #[test]
    fn resamples_to_16k() {
        let x: Vec<f32> = (0..48_000).map(|i| (i as f32 * 0.001).sin()).collect();
        assert_eq!(resample(&x, 48_000).len(), 16_000 - 1);
        assert_eq!(resample(&x[..100], RATE).len(), 100);
    }

    #[test]
    fn matches_what_was_said() {
        let ground = [Req::Clearance, Req::Taxi];
        assert_eq!(intent("Ground, Fable 123, at the ramp, request taxi with information Alpha", &ground), Some(Req::Taxi));
        assert_eq!(intent("Fable one two three request IFR clearance to Sacramento", &ground), Some(Req::Clearance));
        assert_eq!(intent("nice weather today", &ground), None);
        let tower = [Req::Ready, Req::LandingReq];
        assert_eq!(intent("Tower, Fable 123, holding short runway 28 right, ready for departure", &tower), Some(Req::Ready));
        assert_eq!(intent("Fable 123, ten miles south, inbound for landing", &tower), Some(Req::LandingReq));
        let ctaf = [Req::Advisory, Req::AnnounceInbound, Req::AnnounceFinal];
        assert_eq!(intent("Half Moon Bay traffic, Cessna 123, final runway 30", &ctaf), Some(Req::AnnounceFinal));
    }

    #[test]
    fn hears_a_readback() {
        let said = "Fable One Two Three, runway two eight left (28L), cleared for takeoff.";
        assert!(readback("runway two eight left, cleared for takeoff, Fable one two three", said));
        assert!(readback("runway 28 left cleared for takeoff fable 123", said));
        assert!(!readback("request taxi", said));
    }

    /// the whole chain with the installed pack: kokoro says a call, whisper writes it down.
    /// `cargo test --release --bin fablegame atc_hear -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn atc_hear() {
        let text = std::env::var("ATC_SAY").unwrap_or_else(|_| "San Francisco Ground, Fable one two three, at the ramp, request taxi with information Alfa.".into());
        let s = super::super::tts::Tts::get().say(&text, "am_michael", 1.05).recv().unwrap().expect("speech");
        let t = std::time::Instant::now();
        let heard = Ears::get().hear(s.samples.to_vec(), s.rate).recv().unwrap().expect("words");
        println!("said:  {text}\nheard: {heard}\nin {:.2}s (model load included)", t.elapsed().as_secs_f32());
        if std::env::var("ATC_SAY").is_err() {
            assert_eq!(intent(&heard, &[Req::Clearance, Req::Taxi]), Some(Req::Taxi));
        }
    }
}
