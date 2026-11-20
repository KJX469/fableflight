//! Feral GameMode on Linux: while the game runs, ask the GameMode daemon to
//! put the machine in its performance state (CPU governor, process priority
//! and, where the setup allows, the GPU's performance level), and let it go
//! again on the way out.
//!
//! GameMode's client library is opened at run time, as its own C header
//! does, so it is not needed to build or run the game: without it (or
//! without the daemon) nothing happens. `FABLE_NO_GAMEMODE=1` skips it.

/// Holds GameMode on while it lives.
pub struct GameMode {
    #[cfg(target_os = "linux")]
    lib: Option<libloading::Library>,
}

impl GameMode {
    /// Asks for GameMode; never fails (a log line says how it went).
    pub fn request() -> Self {
        #[cfg(target_os = "linux")]
        {
            if std::env::var_os("FABLE_NO_GAMEMODE").is_some() {
                return Self { lib: None };
            }
            // SAFETY: libgamemode is GameMode's own client library; its
            // request functions take nothing and return an int.
            let lib = unsafe { libloading::Library::new("libgamemode.so.0") }.ok();
            let lib = lib.and_then(|l| {
                let start = unsafe { l.get::<unsafe extern "C" fn() -> i32>(b"real_gamemode_request_start\0") }.ok()?;
                let r = unsafe { start() };
                drop(start);
                if r == 0 {
                    log::info!("GameMode on");
                    Some(l)
                } else {
                    log::info!("GameMode is installed but its daemon declined ({r})");
                    None
                }
            });
            Self { lib }
        }
        #[cfg(not(target_os = "linux"))]
        Self {}
    }
}

impl Drop for GameMode {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        if let Some(l) = &self.lib {
            // (the daemon also lets go of a process that has exited)
            if let Ok(end) = unsafe { l.get::<unsafe extern "C" fn() -> i32>(b"real_gamemode_request_end\0") } {
                unsafe { end() };
            }
        }
    }
}
