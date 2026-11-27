//! head tracking: opentrack's "udp over network" output (which also carries
//! trackir, webcam and phone trackers through opentrack), six little endian
//! doubles a packet: x, y, z (cm) and yaw, pitch, roll (degrees), on
//! 127.0.0.1:4242 unless FABLE_OPENTRACK_PORT says otherwise. the head in the
//! cockpit turns and moves with it.

use parking_lot::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    /// right, up, back (m)
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// rad: yaw right, pitch up, roll right
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
}

static LAST: Mutex<Option<(Pose, Instant)>> = Mutex::new(None);

/// reads a packet, if it is one.
pub fn parse(b: &[u8]) -> Option<Pose> {
    if b.len() < 48 {
        return None;
    }
    let f = |k: usize| f64::from_le_bytes(b[k * 8..k * 8 + 8].try_into().unwrap_or([0; 8]));
    let v = [f(0), f(1), f(2), f(3), f(4), f(5)];
    if v.iter().any(|x| !x.is_finite()) {
        return None;
    }
    Some(Pose { x: v[0] / 100.0, y: v[1] / 100.0, z: v[2] / 100.0, yaw: v[3].to_radians(), pitch: v[4].to_radians(), roll: v[5].to_radians() })
}

/// starts listening, once.
pub fn start() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        let port = port();
        let _ = std::thread::Builder::new().name("head tracking".into()).spawn(move || {
            let Ok(sock) = std::net::UdpSocket::bind(("127.0.0.1", port)) else {
                log::info!("head tracking: port {port} taken, not listening");
                return;
            };
            log::info!("head tracking: listening for opentrack on 127.0.0.1:{port}");
            let mut buf = [0u8; 256];
            loop {
                if let Ok(n) = sock.recv(&mut buf) {
                    if let Some(p) = parse(&buf[..n]) {
                        *LAST.lock() = Some((p, Instant::now()));
                    }
                }
            }
        });
    });
}

/// the udp port it listens on: 4242, opentrack's own default.
pub fn port() -> u16 {
    std::env::var("FABLE_OPENTRACK_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(4242)
}

/// the head's pose, while packets keep coming.
pub fn pose() -> Option<Pose> {
    let l = *LAST.lock();
    l.filter(|(_, t)| t.elapsed() < Duration::from_millis(500)).map(|(p, _)| p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_opentrack_packets() {
        let mut b = Vec::new();
        for v in [1.0f64, -2.0, 3.0, 30.0, -10.0, 5.0] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        let p = parse(&b).unwrap();
        assert!((p.x - 0.01).abs() < 1e-12 && (p.yaw - 30f64.to_radians()).abs() < 1e-12);
        assert!(parse(&b[..40]).is_none());
    }
}
