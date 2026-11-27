//! Who may not use this relay: addresses, networks (CIDR) and pilot names,
//! one to a line in the file `FFMP_BANS` names. The relay reads the file
//! again when it changes, so a ban takes effect within a second, without a
//! restart. For example:
//!
//! ```text
//! # an address, a network, a name (any case)
//! 203.0.113.7
//! 198.51.100.0/24
//! 2001:db8::/32
//! name: Some Griefer
//! ```
//!
//! Anything after `#` on a line is a comment; lines that make no sense are
//! reported and skipped.
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A network (address and prefix length), as `parse` reads it.
pub type Net = (IpAddr, u8);

#[derive(Default)]
pub struct Bans {
    nets: Vec<Net>,
    /// lower case
    names: Vec<String>,
    /// the file, and its modification time when last read
    path: Option<PathBuf>,
    read: Option<SystemTime>,
}

impl Bans {
    pub fn new(path: Option<PathBuf>) -> Self {
        let mut b = Bans { path, ..Default::default() };
        b.reload();
        b
    }

    /// From the text of a bans file; the lines it could not read, with their numbers.
    pub fn parse(text: &str) -> (Vec<Net>, Vec<String>, Vec<(usize, String)>) {
        let (mut nets, mut names, mut bad) = (Vec::new(), Vec::new(), Vec::new());
        for (i, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(n) = line.strip_prefix("name:") {
                let n = n.trim().to_lowercase();
                if n.is_empty() {
                    bad.push((i + 1, line.to_string()));
                } else {
                    names.push(n);
                }
                continue;
            }
            match net(line) {
                Some(n) => nets.push(n),
                None => bad.push((i + 1, line.to_string())),
            }
        }
        (nets, names, bad)
    }

    /// Reads the file again if it changed; true if the bans did (a missing
    /// file bans nobody).
    pub fn reload(&mut self) -> bool {
        let Some(path) = self.path.clone() else { return false };
        let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if modified == self.read && (modified.is_some() || self.nets.is_empty() && self.names.is_empty()) {
            return false;
        }
        self.read = modified;
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let (nets, names, bad) = Self::parse(&text);
        for (n, l) in bad {
            eprintln!("bans: {}:{n}: not an address, a network or \"name: ...\": {l}", show(&path));
        }
        let changed = nets != self.nets || names != self.names;
        if changed {
            println!("bans: {} addresses or networks, {} names", nets.len(), names.len());
        }
        (self.nets, self.names) = (nets, names);
        changed
    }

    pub fn ip(&self, ip: IpAddr) -> bool {
        let ip = canonical(ip);
        self.nets.iter().any(|&(n, bits)| within(ip, n, bits))
    }

    pub fn name(&self, name: &str) -> bool {
        !self.names.is_empty() && {
            let n = name.trim().to_lowercase();
            self.names.contains(&n)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nets.is_empty() && self.names.is_empty()
    }
}

fn show(p: &Path) -> String {
    p.display().to_string()
}

/// An IPv4-mapped IPv6 address as the IPv4 one.
fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        v4 => v4,
    }
}

/// "addr" or "addr/bits".
fn net(s: &str) -> Option<Net> {
    let (a, bits) = match s.split_once('/') {
        Some((a, b)) => (a.trim(), Some(b.trim().parse::<u8>().ok()?)),
        None => (s, None),
    };
    let ip = canonical(a.parse::<IpAddr>().ok()?);
    let max = if ip.is_ipv4() { 32 } else { 128 };
    let bits = bits.unwrap_or(max);
    (bits <= max).then_some((ip, bits))
}

fn within(ip: IpAddr, net: IpAddr, bits: u8) -> bool {
    match (ip, net) {
        (IpAddr::V4(a), IpAddr::V4(n)) => {
            let mask = if bits == 0 { 0 } else { u32::MAX << (32 - bits as u32) };
            u32::from(a) & mask == u32::from(n) & mask
        }
        (IpAddr::V6(a), IpAddr::V6(n)) => {
            let mask = if bits == 0 { 0 } else { u128::MAX << (128 - bits as u32) };
            u128::from(a) & mask == u128::from(n) & mask
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bans(text: &str) -> Bans {
        let (nets, names, _) = Bans::parse(text);
        Bans { nets, names, ..Default::default() }
    }

    #[test]
    fn addresses_networks_and_names() {
        let b = bans("# comment\n203.0.113.7\n198.51.100.0/24  # a network\n2001:db8::/32\nname:  Some Griefer \n\n");
        assert!(b.ip("203.0.113.7".parse().unwrap()));
        assert!(!b.ip("203.0.113.8".parse().unwrap()));
        assert!(b.ip("198.51.100.250".parse().unwrap()));
        assert!(!b.ip("198.51.101.1".parse().unwrap()));
        assert!(b.ip("2001:db8:1::5".parse().unwrap()));
        assert!(!b.ip("2001:db9::5".parse().unwrap()));
        // (An IPv4 address as an IPv6 socket sees it.)
        assert!(b.ip("::ffff:203.0.113.7".parse().unwrap()));
        assert!(b.name("some griefer"));
        assert!(b.name("SOME GRIEFER"));
        assert!(!b.name("some griefer 2"));
    }

    #[test]
    fn nonsense_is_reported_not_applied() {
        let (nets, names, bad) = Bans::parse("10.0.0.0/33\nnot an address\nname:\n10.1.2.3/8\n");
        assert_eq!(nets.len(), 1);
        assert!(names.is_empty());
        assert_eq!(bad.iter().map(|b| b.0).collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn the_file_is_read_again_when_it_changes() {
        let dir = std::env::temp_dir().join(format!("ffmp-bans-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bans.txt");
        let _ = std::fs::remove_file(&path);
        let mut b = Bans::new(Some(path.clone()));
        assert!(b.is_empty());
        std::fs::write(&path, "192.0.2.1\n").unwrap();
        assert!(b.reload());
        assert!(b.ip("192.0.2.1".parse().unwrap()));
        assert!(!b.reload(), "unchanged: not read again");
        std::fs::remove_file(&path).unwrap();
        assert!(b.reload(), "gone: nobody banned");
        assert!(b.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
