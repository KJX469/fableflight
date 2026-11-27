//! discord rich presence: what the pilot is doing, on their discord profile,
//! through the discord app's local ipc. it needs the game's application id
//! from discord's developer portal (`APP_ID`, or `FABLE_DISCORD_APP` to try
//! one); without one, or without discord running, it does nothing.

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

/// the game's application on discord's developer portal (empty: off).
pub const APP_ID: &str = "1575464405456388266";

/// what the profile says under the game's name (empty: nothing), and since when (unix s).
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub details: String,
    pub since: i64,
}

/// "flying a cessna 172rg", "flying an sr-71a blackbird": an before a vowel
/// sound, which a leading letter read out by name (f/a-18, sr-71) has too.
pub fn flying(plane: &str) -> String {
    let mut c = plane.chars();
    let (a, b) = (c.next().unwrap_or(' '), c.next().unwrap_or(' '));
    let vowel = "AEIOUaeiou".contains(a) || ("FHLMNRSX".contains(a) && !b.is_ascii_lowercase());
    format!("Flying {} {plane}", if vowel { "an" } else { "a" })
}

/// the link to discord, on its own thread so a slow or missing discord never stalls a frame.
pub struct Presence {
    tx: Sender<Option<Status>>,
    last: Option<Option<Status>>,
}

impl Presence {
    /// none without an application id.
    pub fn start() -> Option<Presence> {
        let id = std::env::var("FABLE_DISCORD_APP").ok().filter(|s| !s.trim().is_empty()).or_else(|| (!APP_ID.is_empty()).then(|| APP_ID.to_string()))?;
        let (tx, rx) = crossbeam_channel::unbounded();
        std::thread::Builder::new().name("discord".into()).spawn(move || run(id, rx)).ok()?;
        Some(Presence { tx, last: None })
    }

    /// what to show now (none: nothing); sent on only when it changes.
    pub fn set(&mut self, s: Option<Status>) {
        if self.last.as_ref() != Some(&s) {
            let _ = self.tx.send(s.clone());
            self.last = Some(s);
        }
    }
}

fn run(id: String, rx: Receiver<Option<Status>>) {
    use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
    let mut client: Option<DiscordIpcClient> = None;
    let mut want: Option<Status> = None;
    let mut dirty = false;
    // discord takes about five updates in twenty seconds: one every four at most.
    let gap = Duration::from_secs(4);
    loop {
        match rx.recv_timeout(gap) {
            Ok(s) => {
                want = s;
                dirty = true;
                // only the newest of a burst matters.
                while let Ok(s) = rx.try_recv() {
                    want = s;
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(c) = client.as_mut() {
                    let _ = c.close();
                }
                return;
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
        if !dirty {
            continue;
        }
        // (re)connect while discord is not there yet, or has gone and come back.
        if client.is_none() {
            let mut c = DiscordIpcClient::new(&id);
            if c.connect().is_err() {
                continue;
            }
            log::info!("discord: connected");
            client = Some(c);
        }
        let Some(c) = client.as_mut() else { continue };
        let r = match &want {
            Some(s) => {
                let mut a = activity::Activity::new().timestamps(activity::Timestamps::new().start(s.since * 1000)).assets(activity::Assets::new().large_image("logo").large_text("FableFlight"));
                if !s.details.is_empty() {
                    a = a.details(s.details.as_str());
                }
                c.set_activity(a)
            }
            None => c.clear_activity(),
        };
        match r {
            Ok(()) => dirty = false,
            Err(e) => {
                log::info!("discord: {e}");
                client = None;
            }
        }
    }
}

/// now, in unix seconds.
pub fn now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_or_an() {
        assert_eq!(flying("Cessna 172RG"), "Flying a Cessna 172RG");
        assert_eq!(flying("F/A-18C Hornet"), "Flying an F/A-18C Hornet");
        assert_eq!(flying("SR-71A Blackbird"), "Flying an SR-71A Blackbird");
        assert_eq!(flying("SpaceX Starship"), "Flying a SpaceX Starship");
        assert_eq!(flying("Airbus A320"), "Flying an Airbus A320");
    }

    /// shows a status on the discord running here for a few seconds, then clears it (run by hand).
    #[test]
    #[ignore]
    fn discord_shows_a_status() {
        use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
        let mut c = DiscordIpcClient::new(APP_ID);
        c.connect().expect("discord running here");
        let a = activity::Activity::new().details(flying("Cessna 172RG")).timestamps(activity::Timestamps::new().start(now() * 1000)).assets(activity::Assets::new().large_image("logo").large_text("FableFlight"));
        c.set_activity(a).expect("the status taken");
        std::thread::sleep(Duration::from_secs(8));
        c.clear_activity().expect("the status cleared");
        let _ = c.close();
    }
}
