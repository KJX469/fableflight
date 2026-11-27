//! VHF airband radio: what a controller's voice sounds like by the time it
//! reaches the pilot's headset.
//!
//! The airband is AM in narrow channels, so a voice arrives as a thin,
//! nasal band from about 300 Hz to 3 kHz, squashed and gritty from a
//! close-talking mic driven hard into the transmitter, over a hiss that grows
//! and a level that wavers as the station gets weaker with distance. Each
//! message opens the squelch with a click, rides on a short bed of carrier
//! hiss, and ends with the squelch tail: the burst of noise heard when the
//! carrier drops, before the receiver mutes again.
//!
//! The chain, per sample:
//! - transmitter: mic high-pass, a presence lift, and soft clipping (`Station`
//!   sets how hard each controller drives it);
//! - path: the voice fades with a weak signal and receiver noise is added;
//! - receiver: the channel filter, steep on both sides, shapes the voice and
//!   the noise alike, so the hiss sounds like the airband and not like tape;
//!   the headset amplifier after it rounds off the loudest peaks.
//!
//! Voices come in at their own sample rate (Kokoro speaks at 24 kHz) and are
//! resampled linearly; the channel filter's 3 kHz top hides the artefacts.

use super::{Bq, DT, RATE, Rng, Slow, kf};

/// The rate `Transmission` plays at: the mixer's.
#[cfg(test)]
pub const OUT_RATE: u32 = RATE;

/// A controller's mic and transmitter: the character of one voice on the air.
#[derive(Clone, Copy, Debug)]
pub struct Station {
    /// how hard the mic drives the transmitter's clipping (1 clean .. 6 gritty)
    pub drive: f32,
    /// the channel's lower and upper edge, Hz
    pub low: f32,
    pub high: f32,
    /// the mic's presence peak, Hz, and its lift, dB
    pub presence: f32,
    pub lift: f32,
}

impl Station {
    /// Clearance and ground: an old desk mic, pushed hard.
    pub const GROUND: Station = Station { drive: 4.5, low: 380.0, high: 2700.0, presence: 1600.0, lift: 5.0 };
    /// The tower: a headset boom mic, firm but clear.
    pub const TOWER: Station = Station { drive: 3.0, low: 320.0, high: 3000.0, presence: 1900.0, lift: 3.5 };
    /// Approach and centre: the cleanest, from a radar room's remote transmitters.
    pub const RADAR: Station = Station { drive: 2.0, low: 300.0, high: 3200.0, presence: 2200.0, lift: 2.5 };
}

impl Default for Station {
    fn default() -> Self {
        Self::TOWER
    }
}

/// The radio path, one mono sample at a time.
pub struct Radio {
    mic: Bq,
    presence: Bq,
    drive: f32,
    channel: [Bq; 6],
    rng: Rng,
    fade: Slow,
}

impl Radio {
    pub fn new(st: Station, seed: u32) -> Self {
        let mut presence = Bq::default();
        presence.peak(st.presence, 1.1, st.lift);
        Radio {
            mic: Bq::new(Bq::hp, st.low * 0.6, 0.7),
            presence,
            drive: st.drive.max(0.5),
            // sixth-order Butterworth each side, like a receiver's crystal
            // filter: 36 dB/octave out of the channel
            channel: [
                Bq::new(Bq::hp, st.low, 0.518),
                Bq::new(Bq::hp, st.low, 0.707),
                Bq::new(Bq::hp, st.low, 1.932),
                Bq::new(Bq::lp, st.high, 0.518),
                Bq::new(Bq::lp, st.high, 0.707),
                Bq::new(Bq::lp, st.high, 1.932),
            ],
            rng: Rng(seed | 1),
            fade: Slow::default(),
        }
    }

    /// One sample. `voice` is the dry voice (peak about 1, or 0 with the
    /// carrier up and nobody speaking), `noise` extra receiver noise on top
    /// (the squelch tail), `signal` the station's strength: 1 close by,
    /// 0 at the edge of range.
    #[inline]
    pub fn run(&mut self, voice: f32, noise: f32, signal: f32) -> f32 {
        let weak = 1.0 - signal.clamp(0.0, 1.0);
        // transmitter
        let mut x = self.presence.run(self.mic.run(voice));
        x = (x * self.drive).tanh() / self.drive.tanh();
        // path: a weak signal wavers (AM fading, a few times a second) and sinks into the hiss
        let waver = self.fade.run(self.rng.f(), kf(4.0));
        x *= (1.0 - 0.6 * weak * weak) * (1.0 + waver * 0.35 * weak).clamp(0.2, 1.4);
        let hiss = 0.04 + 0.9 * weak * weak + noise;
        x += self.rng.f() * hiss;
        // receiver
        for b in &mut self.channel {
            x = b.run(x);
        }
        // the headset amplifier: a little headroom, and it never clips hard
        (0.6 * x).tanh()
    }
}

/// One message on the frequency: the squelch opening, the voice, the tail.
pub struct Transmission {
    voice: Vec<f32>,
    /// voice samples per output sample
    step: f64,
    pos: f64,
    radio: Radio,
    /// output samples so far
    t: usize,
    lead: usize,
    tail: usize,
    /// the station's strength, 0..1; may change while it plays (the aircraft moves)
    pub signal: f32,
}

/// Carrier hiss before the first word, s.
const LEAD: f32 = 0.09;
/// Carrier hiss after the last word, then the squelch tail, s.
const HANG: f32 = 0.07;
const CRASH: f32 = 0.11;

impl Transmission {
    /// `voice` is mono at `rate` Hz, any level: it is brought to a peak of 1.
    pub fn new(mut voice: Vec<f32>, rate: u32, st: Station, signal: f32, seed: u32) -> Self {
        let peak = voice.iter().fold(1e-6f32, |m, x| m.max(x.abs()));
        voice.iter_mut().for_each(|x| *x /= peak);
        let step = rate as f64 / RATE as f64;
        Transmission {
            step,
            pos: 0.0,
            radio: Radio::new(st, seed),
            t: 0,
            lead: (LEAD * RATE as f32) as usize,
            tail: ((HANG + CRASH) * RATE as f32) as usize,
            signal,
            voice,
        }
    }

    /// Output samples from the squelch opening to it closing.
    pub fn len(&self) -> usize {
        self.lead + (self.voice.len() as f64 / self.step).ceil() as usize + self.tail
    }

    pub fn done(&self) -> bool {
        self.t >= self.len()
    }

    /// The next mono sample at `RATE`; silence once done.
    pub fn sample(&mut self) -> f32 {
        let len = self.len();
        if self.t >= len {
            return 0.0;
        }
        let t = self.t;
        self.t += 1;
        let secs = |n: usize| n as f32 * DT;
        let mut voice = 0.0;
        let mut noise = 0.0;
        let mut gain = 1.0;
        if t < self.lead {
            // the squelch opens: a click, then the carrier comes up
            let s = secs(t);
            noise = 2.5 * (-s / 0.004).exp();
            gain = (s / 0.012).min(1.0);
        } else if t < len - self.tail {
            let n = self.voice.len();
            let i = self.pos as usize;
            if i < n {
                let a = self.voice[i];
                let b = if i + 1 < n { self.voice[i + 1] } else { 0.0 };
                voice = a + (b - a) * self.pos.fract() as f32;
            }
            self.pos += self.step;
        } else {
            // the carrier hangs on a moment, drops, and the receiver's noise
            // floods in until the squelch shuts it off
            let s = secs(t - (len - self.tail)) - HANG;
            if s > 0.0 {
                noise = 1.1 * (1.0 - s / CRASH).max(0.0).powf(0.7);
                gain = 1.0 - (s / CRASH).powi(4);
            }
        }
        self.radio.run(voice, noise, self.signal) * gain
    }

    /// All of it, rendered.
    #[cfg(test)]
    pub fn render(mut self) -> Vec<f32> {
        (0..self.len()).map(|_| self.sample()).collect()
    }
}

/// A 16-bit PCM WAV file (the voice cache's format), mixed down to mono:
/// the samples and their rate.
pub fn wav_mono(b: &[u8]) -> Option<(Vec<f32>, u32)> {
    let u16_at = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]) as usize;
    let u32_at = |i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize;
    if b.len() < 12 || &b[..4] != b"RIFF" || &b[8..12] != b"WAVE" {
        return None;
    }
    let (mut fmt, mut data) = (None, None);
    let mut at = 12;
    while at + 8 <= b.len() {
        let n = u32_at(at + 4).min(b.len() - at - 8);
        match &b[at..at + 4] {
            b"fmt " if n >= 16 => fmt = Some((u16_at(at + 8), u16_at(at + 10).max(1), u32_at(at + 12), u16_at(at + 22))),
            b"data" => data = Some(&b[at + 8..at + 8 + n]),
            _ => {}
        }
        at += 8 + n + (n & 1);
    }
    let ((1, ch, rate, 16), Some(data)) = (fmt?, data) else { return None };
    let frames = data.len() / 2 / ch;
    let s = |i: usize| i16::from_le_bytes([data[i * 2], data[i * 2 + 1]]) as f32 / 32768.0;
    Some(((0..frames).map(|f| (0..ch).map(|c| s(f * ch + c)).sum::<f32>() / ch as f32).collect(), rate as u32))
}

/// Mono samples as a 16-bit PCM WAV file.
pub fn wav_bytes(x: &[f32], rate: u32) -> Vec<u8> {
    let n = (x.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + x.len() * 2);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + n).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&n.to_le_bytes());
    x.iter().for_each(|s| b.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes()));
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn rms(x: &[f32]) -> f32 {
        (x.iter().map(|s| s * s).sum::<f32>() / x.len().max(1) as f32).sqrt()
    }

    /// The level of a quiet tone at `f` Hz after the radio, dB against 1 kHz,
    /// with a strong signal and the hiss measured apart and taken off.
    fn response(f: f32) -> f32 {
        let level = |f: f32| {
            let mut r = Radio::new(Station::TOWER, 7);
            let n = RATE as usize;
            let out: Vec<f32> = (0..n).map(|i| r.run(0.05 * (TAU * f * i as f32 * DT).sin(), 0.0, 1.0)).collect();
            let mut q = Radio::new(Station::TOWER, 7);
            let hiss: Vec<f32> = (0..n).map(|_| q.run(0.0, 0.0, 1.0)).collect();
            let tail = n / 2;
            (rms(&out[tail..]).powi(2) - rms(&hiss[tail..]).powi(2)).max(1e-12).sqrt()
        };
        20.0 * (level(f) / level(1000.0)).log10()
    }

    #[test]
    fn passes_the_voice_band_only() {
        for f in [600.0, 1500.0, 2500.0] {
            let d = response(f);
            assert!(d > -6.0, "{f} Hz is {d:.1} dB down, inside the channel");
        }
        for f in [100.0, 6000.0, 10000.0] {
            let d = response(f);
            assert!(d < -30.0, "{f} Hz is only {d:.1} dB down, outside the channel");
        }
    }

    /// A stand-in voice: a buzzy, syllable-shaped tone, 1.5 s at 24 kHz.
    fn voice() -> Vec<f32> {
        (0..36000)
            .map(|i| {
                let t = i as f32 / 24000.0;
                let syll = (TAU * 4.0 * t).sin().max(0.0);
                let buzz: f32 = (1..12).map(|k| (TAU * 140.0 * k as f32 * t).sin() / k as f32).sum();
                0.3 * syll * buzz
            })
            .collect()
    }

    #[test]
    fn stays_finite_and_in_range() {
        for st in [Station::GROUND, Station::TOWER, Station::RADAR] {
            for signal in [0.0, 0.5, 1.0] {
                let x = Transmission::new(voice(), 24000, st, signal, 3).render();
                assert!(x.iter().all(|s| s.is_finite() && s.abs() < 1.0), "{st:?} at {signal}");
            }
        }
    }

    #[test]
    fn squelch_frames_the_voice() {
        let tx = Transmission::new(voice(), 24000, Station::TOWER, 1.0, 3);
        let len = tx.len();
        let expect = ((LEAD + HANG + CRASH) * RATE as f32) as usize + (36000.0 * RATE as f32 / 24000.0) as usize;
        assert!(len.abs_diff(expect) < 4, "{len} samples, expected about {expect}");
        let x = tx.render();
        let ms = |a: f32, b: f32| rms(&x[(a * RATE as f32) as usize..(b * RATE as f32) as usize]);
        let tail_start = (len as f32 * DT) - CRASH;
        // the opening click and the squelch tail stand out of the carrier hiss
        assert!(ms(0.0, 0.006) > 3.0 * ms(0.04, 0.08), "no squelch click");
        assert!(ms(tail_start, tail_start + 0.04) > 3.0 * ms(0.04, 0.08), "no squelch tail");
        // and the receiver is quiet again once the squelch closes
        let end = &x[len - (0.005 * RATE as f32) as usize..];
        assert!(rms(end) < 0.5 * ms(0.04, 0.08), "the squelch didn't close");
    }

    #[test]
    fn weaker_further_away() {
        let snr = |signal: f32| {
            let mut tx = Transmission::new(voice(), 24000, Station::TOWER, signal, 3);
            let x: Vec<f32> = (0..tx.len()).map(|_| tx.sample()).collect();
            let lead = (0.03 * RATE as f32) as usize..(0.08 * RATE as f32) as usize;
            let speech = (0.3 * RATE as f32) as usize..(1.3 * RATE as f32) as usize;
            20.0 * (rms(&x[speech]) / rms(&x[lead])).log10()
        };
        let (near, far) = (snr(1.0), snr(0.15));
        assert!(near > 20.0, "only {near:.1} dB above the hiss close by");
        assert!(near - far > 10.0, "{near:.1} dB close by, {far:.1} dB far off");
    }

    #[test]
    fn resamples_to_the_output_rate() {
        // a 1 kHz tone at 24 kHz comes out at 1 kHz: count the zero crossings
        let tone: Vec<f32> = (0..24000).map(|i| (TAU * 1000.0 * i as f32 / 24000.0).sin()).collect();
        let mut tx = Transmission::new(tone, 24000, Station::RADAR, 1.0, 3);
        let x: Vec<f32> = (0..tx.len()).map(|_| tx.sample()).collect();
        let mid = &x[(0.3 * RATE as f32) as usize..(0.8 * RATE as f32) as usize];
        let crossings = mid.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count();
        assert!((495..=505).contains(&crossings), "{crossings} cycles in half a second");
    }

    #[test]
    fn reads_wav() {
        let mut b = Vec::new();
        let pcm: [i16; 4] = [0, 16384, -16384, 32767];
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&(36 + 8u32).to_le_bytes());
        b.extend_from_slice(b"WAVEfmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        for v in [1u16, 2] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b.extend_from_slice(&24000u32.to_le_bytes());
        b.extend_from_slice(&(24000u32 * 4).to_le_bytes());
        for v in [4u16, 16] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        b.extend_from_slice(b"data");
        b.extend_from_slice(&8u32.to_le_bytes());
        pcm.iter().for_each(|v| b.extend_from_slice(&v.to_le_bytes()));
        let (x, rate) = wav_mono(&b).unwrap();
        assert_eq!(rate, 24000);
        assert_eq!(x.len(), 2);
        assert!((x[0] - 0.25).abs() < 1e-4 && (x[1] - (32767.0 - 16384.0) / 65536.0).abs() < 1e-4);
        assert!(wav_mono(b"RIFF....WAVX").is_none());
    }

    /// Put a recording through the radio to hear it:
    /// `RADIO_IN=voice.wav RADIO_OUT=out.wav [RADIO_STATION=ground|tower|radar]
    /// [RADIO_SIGNAL=0..1] cargo test --release --bin fablegame radio_file -- --ignored`
    #[test]
    #[ignore]
    fn radio_file() {
        let (Ok(src), Ok(dst)) = (std::env::var("RADIO_IN"), std::env::var("RADIO_OUT")) else {
            panic!("set RADIO_IN and RADIO_OUT");
        };
        let (voice, rate) = wav_mono(&std::fs::read(&src).unwrap()).expect("16-bit PCM WAV");
        let st = match std::env::var("RADIO_STATION").as_deref() {
            Ok("ground") => Station::GROUND,
            Ok("radar") => Station::RADAR,
            _ => Station::TOWER,
        };
        let signal = std::env::var("RADIO_SIGNAL").ok().and_then(|s| s.parse().ok()).unwrap_or(1.0);
        let x = Transmission::new(voice, rate, st, signal, 0x5eed).render();
        std::fs::write(&dst, wav_bytes(&x, RATE)).unwrap();
    }
}
