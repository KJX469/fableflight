//! Turbulence as a field in space: a frozen von Kármán gust field that drifts
//! with the wind, which the aircraft flies through. Any point can be sampled,
//! so every part of the airframe (probes at the CG, along each wing, at the
//! nose and at the tail: see `plane.rs` `gust_probes`) feels the gust where it is.
//! What a gust does to the aircraft then comes from its geometry rather than
//! from tuned gains: a vertical gust that differs from one wing tip to the
//! other rolls it, one that differs from the nose to the tail pitches it, a
//! side gust that differs along the fuselage yaws it, and a big airframe
//! spans many small eddies and averages them out.
//!
//! The field is a sum of random Fourier modes (Kraichnan's method): plane
//! waves of velocity across log-spaced wavenumber shells, each shell holding
//! two orthogonal triads so the gusts are isotropic per component, each wave's
//! velocity perpendicular to its wavevector so the air is incompressible. The
//! shells' amplitudes follow the von Kármán energy spectrum for the length
//! scale and intensity the weather asks for (MIL-HDBK-1797 / MIL-F-8785C),
//! separately for the horizontal and the vertical gusts (near the ground the
//! vertical eddies are smaller and weaker). Changing the scale or intensity
//! only changes the amplitudes, never where the waves are, so the field never
//! jumps. The eddies also turn over slowly (each wave's phase drifts at about
//! the speed of its own eddy), so air that is not moving still gusts.
//!
//! References: Kraichnan, "Diffusion by a random velocity field", Phys.
//! Fluids 13 (1970); MIL-HDBK-1797 §B.4.9 (von Kármán form, L' = 1.339 L);
//! Etkin, Dynamics of Atmospheric Flight, ch. 13 (gust gradients across the airframe).

use glam::{DQuat, DVec3};

/// Wavenumber shells, log-spaced from a 6 km wavelength to 2 m.
const SHELLS: usize = 30;
const K_MIN: f64 = std::f64::consts::TAU / 6000.0;
const K_MAX: f64 = std::f64::consts::TAU / 2.0;
/// Waves per shell: two orthogonal triads.
const PER: usize = 6;
const WAVES: usize = SHELLS * PER;

/// The waves, one array per quantity (so the sampling loop vectorises),
/// shell by shell: wave `n` is in shell `n / PER`.
#[derive(Clone, Debug)]
struct Waves {
    /// wavevector (rad/m), planet frame
    kx: [f64; WAVES],
    ky: [f64; WAVES],
    kz: [f64; WAVES],
    /// unit velocity direction, perpendicular to `k`
    pol: [DVec3; WAVES],
    phase: [f64; WAVES],
    /// phase drift per (m/s of turbulence intensity) per second: the eddy turning over
    turn: [f64; WAVES],
}

#[derive(Clone, Debug)]
pub struct GustField {
    waves: Box<Waves>,
    /// each shell's wavenumber and width (rad/m)
    k: [f64; SHELLS],
    dk: [f64; SHELLS],
    /// each wave's amplitude (m/s) for the horizontal and the vertical gusts this instant
    amp_h: [f64; SHELLS],
    amp_w: [f64; SHELLS],
    /// the lifting surfaces' mean chord (m): eddies much smaller than it
    /// barely lift a wing (Sears), so their share is tapered
    chord: f64,
    /// the last length scales `set` was given and their spectrum shares
    /// (horizontal, vertical): the scales change rarely, the intensity every step
    shares_h: (f64, [f64; SHELLS]),
    shares_w: (f64, [f64; SHELLS]),
    /// how far the air mass has moved with the wind (planet frame, m): the
    /// field is frozen in the air, so it is sampled at (position - drift)
    pub drift: DVec3,
}

/// splitmix64, for laying the waves out from a seed.
struct Mix(u64);
impl Mix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    /// A uniformly random rotation (Shoemake).
    fn rotation(&mut self) -> DQuat {
        let (u1, u2, u3) = (self.unit(), self.unit() * std::f64::consts::TAU, self.unit() * std::f64::consts::TAU);
        let (a, b) = ((1.0 - u1).sqrt(), u1.sqrt());
        DQuat::from_xyzw(a * u2.sin(), a * u2.cos(), b * u3.sin(), b * u3.cos()).normalize()
    }
}

/// The share of the variance in each shell for a von Kármán spectrum of
/// length scale `len` (m): E(k) ∝ (kL')⁴ / (1 + (kL')²)^(17/6), summed to 1.
fn shares(k: &[f64; SHELLS], dk: &[f64; SHELLS], len: f64) -> [f64; SHELLS] {
    let lp = 1.339 * len.max(1.0);
    let mut s = [0.0; SHELLS];
    let mut sum = 0.0;
    for i in 0..SHELLS {
        let x = k[i] * lp;
        let x2 = x * x;
        s[i] = x2 * x2 / (1.0 + x2).powf(17.0 / 6.0) * dk[i];
        sum += s[i];
    }
    for v in &mut s {
        *v /= sum.max(1e-30);
    }
    s
}

impl GustField {
    /// A field for an airframe whose wing has mean chord `chord` (m; 0 for
    /// the air itself, untapered).
    pub fn for_chord(seed: u64, chord: f64) -> Self {
        let mut f = Self::new(seed);
        f.chord = chord.max(0.0);
        f
    }

    pub fn new(seed: u64) -> Self {
        let mut r = Mix(seed ^ 0x6a09_e667_f3bc_c909);
        let ratio = (K_MAX / K_MIN).ln() / (SHELLS - 1) as f64;
        let mut k = [0.0; SHELLS];
        let mut dk = [0.0; SHELLS];
        let mut waves = Box::new(Waves { kx: [0.0; WAVES], ky: [0.0; WAVES], kz: [0.0; WAVES], pol: [DVec3::ZERO; WAVES], phase: [0.0; WAVES], turn: [0.0; WAVES] });
        let mut n = 0;
        for i in 0..SHELLS {
            k[i] = K_MIN * (ratio * i as f64).exp();
            // (The shell covers half a step either side, in log space.)
            dk[i] = k[i] * 2.0 * (0.5 * ratio).sinh();
            for _ in 0..PER / 3 {
                // An orthonormal triad turned at random: each wave runs along
                // one axis and moves the air along the next, so across the
                // three the velocity covers every direction equally.
                let q = r.rotation();
                let e = [q * DVec3::X, q * DVec3::Y, q * DVec3::Z];
                // Each shell's wavenumber jittered a little, so no two triads share it.
                for a in 0..3 {
                    let kk = k[i] * (ratio * (r.unit() - 0.5)).exp();
                    let kv = e[a] * kk;
                    (waves.kx[n], waves.ky[n], waves.kz[n], waves.pol[n]) = (kv.x, kv.y, kv.z, e[(a + 1) % 3]);
                    waves.phase[n] = r.unit() * std::f64::consts::TAU;
                    waves.turn[n] = kk * (0.2 + 0.3 * r.unit()) * if r.unit() < 0.5 { -1.0 } else { 1.0 };
                    n += 1;
                }
            }
        }
        Self { waves, k, dk, amp_h: [0.0; SHELLS], amp_w: [0.0; SHELLS], chord: 0.0, shares_h: (f64::NAN, [0.0; SHELLS]), shares_w: (f64::NAN, [0.0; SHELLS]), drift: DVec3::ZERO }
    }

    /// The intensity (RMS, m/s) and length scale (m) of the horizontal and
    /// the vertical gusts the weather asks for here.
    pub fn set(&mut self, sigma_h: f64, len_h: f64, sigma_w: f64, len_w: f64) {
        if self.shares_h.0 != len_h {
            self.shares_h = (len_h, shares(&self.k, &self.dk, len_h));
        }
        if self.shares_w.0 != len_w {
            self.shares_w = (len_w, shares(&self.k, &self.dk, len_w));
        }
        let (sh, sw) = (self.shares_h.1, self.shares_w.1);
        // Each wave carries a²/2 of variance along its direction; a shell's
        // two triads put 2 · a²/2 on every component.
        let per = (PER / 3) as f64;
        for i in 0..SHELLS {
            // The Sears function: a wing in a sinusoidal gust of reduced
            // frequency k c / 2 lifts as |S| ≈ 1 / √(1 + 2π k c/2) of the quasi-steady value.
            let sears = 1.0 / (1.0 + std::f64::consts::PI * self.k[i] * self.chord).sqrt();
            self.amp_h[i] = sigma_h.max(0.0) * (2.0 * sh[i] / per).sqrt() * sears;
            self.amp_w[i] = sigma_w.max(0.0) * (2.0 * sw[i] / per).sqrt() * sears;
        }
    }

    /// Carry the field along with the air (`wind`, planet frame m/s) and let
    /// its eddies turn over at the intensity `sigma` (m/s).
    pub fn advance(&mut self, dt: f64, wind: DVec3, sigma: f64) {
        // (Never reset: that would jump the gust at every probe at once. The
        // positions are planet-frame, millions of metres out, so a drift of
        // even thousands of km costs no precision in f64.)
        self.drift += wind * dt;
        let s = sigma.clamp(0.0, 15.0) * dt;
        const TAU: f64 = std::f64::consts::TAU;
        let w = &mut *self.waves;
        for (p, t) in w.phase.iter_mut().zip(&w.turn) {
            let x = *p + t * s;
            // `rem_euclid(TAU)` without the division: a phase in [0, 2π)
            // moved by less than a turn needs at most one 2π added or taken
            // away, which gives exactly the same bits (x - 2π is exact for x
            // in [2π, 4π), and rem_euclid adds 2π to a negative x itself).
            *p = if x >= TAU {
                if x < 2.0 * TAU { x - TAU } else { x.rem_euclid(TAU) }
            } else if x < 0.0 {
                if x >= -TAU { x + TAU } else { x.rem_euclid(TAU) }
            } else {
                x
            };
        }
    }

    /// The gust (planet frame, m/s) at `pos` (planet frame), `up` the local vertical.
    pub fn at(&self, pos: DVec3, up: DVec3) -> DVec3 {
        let x = pos - self.drift;
        let w = &*self.waves;
        // Each wave's cosine, all at once (this loop vectorises), then their
        // sum in the same order as ever. The fast cosine is as accurate as
        // the library's (see `cos_reduced`) for phases up to about 5e7 rad,
        // which covers every point within 8,000 km of the planet's centre
        // along each axis; further out it falls back on `f64::cos`.
        let mut c = [0.0; WAVES];
        let (kx, ky, kz, ph) = (&w.kx, &w.ky, &w.kz, &w.phase);
        if x.abs().max_element() < FAST_COS_REACH {
            cosines(w, x, &mut c);
        } else {
            for n in 0..WAVES {
                c[n] = (kx[n] * x.x + ky[n] * x.y + kz[n] * x.z + ph[n]).cos();
            }
        }
        let (mut vh, mut vw) = (DVec3::ZERO, DVec3::ZERO);
        for i in 0..SHELLS {
            let (ah, aw) = (self.amp_h[i], self.amp_w[i]);
            for n in i * PER..(i + 1) * PER {
                vh += w.pol[n] * (c[n] * ah);
                vw += w.pol[n] * (c[n] * aw);
            }
        }
        vh - up * vh.dot(up) + up * vw.dot(up)
    }

    /// Whether there is any turbulence to sample.
    pub fn calm(&self) -> bool {
        self.amp_h.iter().chain(self.amp_w.iter()).all(|a| *a == 0.0)
    }
}

/// Each wave's cosine at `x` (field frame), by `cos_reduced`. With AVX2 four
/// at a time where the processor has it (the same operations, so the same
/// bits: no fused multiply-adds), else two at a time.
fn cosines(w: &Waves, x: DVec3, c: &mut [f64; WAVES]) {
    #[inline(always)]
    fn each(w: &Waves, x: DVec3, c: &mut [f64; WAVES]) {
        for n in 0..WAVES {
            c[n] = cos_reduced(w.kx[n] * x.x + w.ky[n] * x.y + w.kz[n] * x.z + w.phase[n]);
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        #[target_feature(enable = "avx2")]
        unsafe fn wide(w: &Waves, x: DVec3, c: &mut [f64; WAVES]) {
            each(w, x, c)
        }
        if std::arch::is_x86_feature_detected!("avx2") {
            // SAFETY: the processor has just been found to support AVX2.
            return unsafe { wide(w, x, c) };
        }
    }
    each(w, x, c)
}

/// Where `at` can use `cos_reduced` (planet-frame m, per axis): the largest
/// wavenumber (K_MAX with its jitter, under 3.7 rad/m) times √3 times this
/// stays under 2^25 · π/2, the reach of the three-part reduction below.
const FAST_COS_REACH: f64 = 8.0e6;

/// cos(x) for |x| up to 2^25 · π/2, branch-free so a loop of them
/// vectorises, as accurate as the C library's (within an ulp or so).
/// Cody-Waite reduction by π/2 in three parts (the first two 26 bits long,
/// so n times them is exact for n < 2^27), then the fdlibm/musl kernels
/// (`__sin`, `__cos`) on |r| ≤ π/4, picked and signed by the quadrant.
#[inline(always)]
fn cos_reduced(x: f64) -> f64 {
    const PIO2_1: f64 = 1.570_796_310_901_641_8; // first 26 bits of π/2
    const PIO2_2: f64 = 1.589_325_471_229_585_7e-8; // next 26
    const PIO2_3: f64 = 6.123_233_995_736_766e-17; // the rest
    // 1.5 · 2^52: adding it rounds to an integer and leaves it in the low bits.
    const MAGIC: f64 = 6_755_399_441_055_744.0;
    let t = x * std::f64::consts::FRAC_2_PI + MAGIC;
    let q = t.to_bits();
    let n = t - MAGIC;
    let r = ((x - n * PIO2_1) - n * PIO2_2) - n * PIO2_3;
    let z = r * r;
    let w = z * z;
    // musl __sin(r, 0, 0)
    const S1: f64 = -1.666_666_666_666_663_243_48e-01;
    const S2: f64 = 8.333_333_333_322_489_461_24e-03;
    const S3: f64 = -1.984_126_982_985_794_931_34e-04;
    const S4: f64 = 2.755_731_370_707_006_767_89e-06;
    const S5: f64 = -2.505_076_025_340_686_341_95e-08;
    const S6: f64 = 1.589_690_995_211_550_102_21e-10;
    let rs = S2 + z * (S3 + z * S4) + z * w * (S5 + z * S6);
    let sin = r + z * r * (S1 + z * rs);
    // musl __cos(r, 0)
    const C1: f64 = 4.166_666_666_666_660_190_37e-02;
    const C2: f64 = -1.388_888_888_887_410_957_49e-03;
    const C3: f64 = 2.480_158_728_947_672_941_78e-05;
    const C4: f64 = -2.755_731_435_139_066_330_35e-07;
    const C5: f64 = 2.087_572_321_298_174_827_90e-09;
    const C6: f64 = -1.135_964_755_778_819_482_65e-11;
    let rc = z * (C1 + z * (C2 + z * C3)) + w * w * (C4 + z * (C5 + z * C6));
    let hz = 0.5 * z;
    let one = 1.0 - hz;
    let cos = one + (((1.0 - one) - hz) + z * rc);
    // cos(nπ/2 + r) = cos r, -sin r, -cos r, sin r for n = 0, 1, 2, 3 (mod 4).
    let v = if q & 1 == 0 { cos } else { sin };
    f64::from_bits(v.to_bits() ^ (((q.wrapping_add(1)) & 2) << 62))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Samples along a straight line through the field.
    fn line(f: &GustField, from: DVec3, dir: DVec3, step: f64, n: usize) -> Vec<DVec3> {
        (0..n).map(|i| f.at(from + dir * (step * i as f64), DVec3::Z)).collect()
    }

    #[test]
    fn the_gusts_have_the_intensity_asked_for() {
        // Many seeds' worth of independent stretches.
        let (mut h, mut w, mut n) = (0.0, 0.0, 0.0);
        for s in 0..40u64 {
            let mut f = GustField::new(s);
            f.set(2.0, 400.0, 1.0, 100.0);
            for u in line(&f, DVec3::new(s as f64 * 1.0e5, 0.0, 0.0), DVec3::new(0.6, 0.8, 0.0), 37.0, 800) {
                h += 0.5 * (u.x * u.x + u.y * u.y);
                w += u.z * u.z;
                n += 1.0;
            }
        }
        let (sh, sw) = ((h / n).sqrt(), (w / n).sqrt());
        println!("sigma_h {sh:.3} (2.0)  sigma_w {sw:.3} (1.0)");
        assert!((sh - 2.0).abs() < 0.2 && (sw - 1.0).abs() < 0.1, "{sh} {sw}");
    }

    #[test]
    fn the_gusts_are_correlated_over_the_length_scale() {
        // The longitudinal correlation of a von Kármán field falls to about
        // 0.5 at 0.8 L and is small by 4 L.
        let len = 300.0;
        let corr = |lag: f64| {
            let (mut c, mut v) = (0.0, 0.0);
            for s in 0..40u64 {
                let mut f = GustField::new(100 + s);
                f.set(1.0, len, 1.0, len);
                let d = DVec3::X;
                for i in 0..400 {
                    let p = DVec3::new(s as f64 * 2.0e5 + i as f64 * 97.0, 0.0, 0.0);
                    let (a, b) = (f.at(p, DVec3::Z).x, f.at(p + d * lag, DVec3::Z).x);
                    c += a * b;
                    v += a * a;
                }
            }
            c / v
        };
        let (r0, r1, r4) = (corr(10.0), corr(0.8 * len), corr(4.0 * len));
        println!("correlation: 10 m {r0:.2}, 0.8 L {r1:.2}, 4 L {r4:.2}");
        assert!(r0 > 0.9 && (0.3..0.7).contains(&r1) && r4.abs() < 0.2, "{r0} {r1} {r4}");
    }

    #[test]
    fn the_air_is_incompressible_and_continuous() {
        let mut f = GustField::new(3);
        f.set(3.0, 200.0, 3.0, 200.0);
        let h = 0.01;
        for i in 0..50 {
            let p = DVec3::new(i as f64 * 311.0, i as f64 * 53.0, 1000.0);
            let div = (f.at(p + DVec3::X * h, DVec3::Z).x - f.at(p - DVec3::X * h, DVec3::Z).x
                + f.at(p + DVec3::Y * h, DVec3::Z).y
                - f.at(p - DVec3::Y * h, DVec3::Z).y
                + f.at(p + DVec3::Z * h, DVec3::Z).z
                - f.at(p - DVec3::Z * h, DVec3::Z).z)
                / (2.0 * h);
            assert!(div.abs() < 1e-3, "divergence {div}");
            // Small steps, small changes.
            assert!((f.at(p, DVec3::Z) - f.at(p + DVec3::X * 0.05, DVec3::Z)).length() < 0.2);
        }
    }

    #[test]
    fn the_fast_cosine_matches_the_library() {
        // Across the whole range it is used on, planet-frame phases included.
        let mut r = Mix(5);
        let mut worst: f64 = 0.0;
        for i in 0..2_000_000 {
            let mag = [1.0, 10.0, 1e3, 1e5, 2.5e7, 5.0e7][i % 6];
            let x = (r.unit() * 2.0 - 1.0) * mag;
            worst = worst.max((cos_reduced(x) - x.cos()).abs());
        }
        // Special points: the quadrant edges.
        for n in -40..40 {
            let x = n as f64 * std::f64::consts::FRAC_PI_4;
            worst = worst.max((cos_reduced(x) - x.cos()).abs());
        }
        assert!(worst < 4e-16, "{worst:e}");
    }

    #[test]
    fn far_from_the_planet_it_falls_back_on_the_library() {
        let mut f = GustField::new(4);
        f.set(2.0, 300.0, 2.0, 300.0);
        // Either side of the switch-over, the field is continuous.
        let p = DVec3::new(FAST_COS_REACH - 0.001, 1.0e6, 2.0e6);
        let (a, b) = (f.at(p, DVec3::Z), f.at(p + DVec3::X * 0.002, DVec3::Z));
        assert!(a.is_finite() && (a - b).length() < 1e-2);
        assert!(f.at(DVec3::splat(4.0e7), DVec3::Z).is_finite());
    }

    #[test]
    fn changing_the_scale_never_jumps() {
        let mut f = GustField::new(9);
        f.set(2.0, 300.0, 2.0, 300.0);
        let p = DVec3::new(1234.0, 567.0, 89.0);
        let a = f.at(p, DVec3::Z);
        f.set(2.0, 301.0, 2.0, 301.0);
        assert!((f.at(p, DVec3::Z) - a).length() < 0.02);
    }
}
