//! the run's log and crash reports, kept on disk so a crash leaves a trace.
//! game.log is this run's log, crash.log gathers every panic with its backtrace.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

static LOG: OnceLock<parking_lot::Mutex<File>> = OnceLock::new();

/// the folder both files live in.
pub fn dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("fablegame")
}

/// where panics are written.
pub fn crash_path() -> PathBuf {
    dir().join("crash.log")
}

/// log lines go to stderr and to game.log.
struct Tee;

impl Write for Tee {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stderr().write_all(buf);
        if let Some(f) = LOG.get() {
            let _ = f.lock().write_all(buf);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if let Some(f) = LOG.get() {
            let _ = f.lock().flush();
        }
        std::io::stderr().flush()
    }
}

/// starts the logger and the panic hook.
pub fn init() {
    let d = dir();
    let _ = std::fs::create_dir_all(&d);
    if let Ok(f) = File::create(d.join("game.log")) {
        let _ = LOG.set(parking_lot::Mutex::new(f));
    }
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn,fablegame=info")).target(env_logger::Target::Pipe(Box::new(Tee))).init();
    log::info!("fablegame {} on {} {}", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH);
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("unnamed").to_string();
        let msg = info.payload().downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_else(|| "(no message)".into());
        let at = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let bt = std::backtrace::Backtrace::force_capture();
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let report = format!("panic in thread '{name}' at {at}: {msg}\nunix time {secs}, fablegame {}\n{bt}\n", env!("CARGO_PKG_VERSION"));
        log::error!("panic in thread '{name}' at {at}: {msg} (backtrace in {})", crash_path().display());
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(crash_path()) {
            let _ = writeln!(f, "{report}");
        }
        eprintln!("{report}");
    }));
}

/// logs why the game is closing.
pub fn exit(reason: &str) {
    log::info!("exit: {reason}");
    if let Some(f) = LOG.get() {
        let _ = f.lock().flush();
    }
}
