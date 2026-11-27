//! The voice pack: Kokoro-82M and the ONNX Runtime that runs it, downloaded
//! on request into the game's data folder, like AMD's FSR 4 runtime
//! (fsr4.rs). Nothing here ships with the game.
//!
//! - Kokoro-82M v1.0 (hexgrad, Apache 2.0) as ONNX, fp32, and its voices,
//!   from the kokoro-onnx project's release files.
//! - Microsoft's ONNX Runtime (MIT), the shared library only, taken out of
//!   the official `onnxruntime` wheel on PyPI for this platform. Microsoft's
//!   builds pick the CPU's instruction set at run time; the prebuilt ones
//!   other projects publish need AVX2 and crash without it.
//!
//! - Whisper tiny.en (openai, mit) as int8 onnx from sherpa-onnx's export,
//!   for the pilot's own calls by push to talk (listen.rs).
//!
//! Every download is pinned by size and SHA-256.

use parking_lot::Mutex;
use std::path::PathBuf;

/// The pack's folder name: bump it when a file changes.
const PACK: &str = "atc-voices-2";
pub const ORT_VERSION: &str = "1.31.1";

pub struct File {
    pub url: &'static str,
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

/// Kokoro's release files, and the PyPI file store.
macro_rules! kokoro {
    ($f:literal) => {
        concat!("https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/", $f)
    };
}
macro_rules! pypi {
    ($f:literal) => {
        concat!("https://files.pythonhosted.org/packages/", $f)
    };
}

pub const MODEL: File = File { url: kokoro!("kokoro-v1.0.onnx"), name: "kokoro-v1.0.onnx", size: 325_532_387, sha256: "7d5df8ecf7d4b1878015a32686053fd0eebe2bc377234608764cc0ef3636a6c5" };
/// sherpa-onnx's whisper export, pinned to one commit.
macro_rules! whisper {
    ($f:literal) => {
        concat!("https://huggingface.co/csukuangfj/sherpa-onnx-whisper-tiny.en/resolve/d026532c022fa99fd789d6b32446a1df7b6bfc43/", $f)
    };
}

pub const EARS_ENC: File = File { url: whisper!("tiny.en-encoder.int8.onnx"), name: "tiny.en-encoder.int8.onnx", size: 12_937_772, sha256: "0ce578b827c94a961aacb8fa14b02f096504b337e5c94be37c36238cbe3e8bc6" };
pub const EARS_DEC: File = File { url: whisper!("tiny.en-decoder.int8.onnx"), name: "tiny.en-decoder.int8.onnx", size: 89_853_865, sha256: "06c0e6ff6348d427e51839219d1c886c18cfdf411e629e33f5e1679bff9c1527" };
pub const EARS_TOKENS: File = File { url: whisper!("tiny.en-tokens.txt"), name: "tiny.en-tokens.txt", size: 835_554, sha256: "306cd27f03c1a714eca7108e03d66b7dc042abe8c258b44c199a7ed9838dd930" };

pub const VOICES: File = File { url: kokoro!("voices-v1.0.bin"), name: "voices-v1.0.bin", size: 28_214_398, sha256: "bca610b8308e8d99f32e6fe4197e7ec01679264efed0cac9140fe9c29f1fbf7d" };

/// ONNX Runtime for this platform: the wheel, and the library inside it.
pub struct Runtime {
    pub wheel: File,
    /// path of the library in the wheel, and the name it gets on disk
    pub lib: &'static str,
    pub lib_name: &'static str,
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const RUNTIME: Option<Runtime> = Some(Runtime {
    wheel: File { url: pypi!("fc/0a/c887bcb84a9455b2723825381a77af38611a80f996f804e20fde5f671b63/onnxruntime-1.31.1-cp313-cp313-manylinux_2_28_x86_64.whl"), name: "onnxruntime-linux-x64.whl", size: 23_760_717, sha256: "74a6a52994616c4d18b33db1e13c226333a70a6ba07480a5cab9c2a50dff1430" },
    lib: "onnxruntime/capi/libonnxruntime.so.1.31.1",
    lib_name: "libonnxruntime.so",
});
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const RUNTIME: Option<Runtime> = Some(Runtime {
    wheel: File { url: pypi!("e1/e1/633d4994db0620d31f34d9962119bc4ebd0b2ef4c9a030f91c7843fb9672/onnxruntime-1.31.1-cp313-cp313-manylinux_2_28_aarch64.whl"), name: "onnxruntime-linux-arm64.whl", size: 21_420_628, sha256: "dcaedc50333876f3829ab966b242147f0e548229e29c36fe44622ecf748c25ee" },
    lib: "onnxruntime/capi/libonnxruntime.so.1.31.1",
    lib_name: "libonnxruntime.so",
});
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const RUNTIME: Option<Runtime> = Some(Runtime {
    wheel: File { url: pypi!("59/bf/a1d282b4c753ba3e9b31f4c11abd2bc2390513dc5e3792ca3b78e42401c3/onnxruntime-1.31.1-cp313-cp313-win_amd64.whl"), name: "onnxruntime-win-x64.whl", size: 14_888_304, sha256: "0558f29d5f6ac09957390c9c5495931485ee8ae7f0ce781ccee86a0ae5c2348b" },
    lib: "onnxruntime/capi/onnxruntime.dll",
    lib_name: "onnxruntime.dll",
});
#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
pub const RUNTIME: Option<Runtime> = Some(Runtime {
    wheel: File { url: pypi!("7e/fd/529135287474dbaf19d9362ccda61998128ab18507f78bc82931e1bbaf22/onnxruntime-1.31.1-cp313-cp313-win_arm64.whl"), name: "onnxruntime-win-arm64.whl", size: 14_740_772, sha256: "075147c2e2430b4b29c436bffa78537b902a6ed58ed66f99ae3c6795e2081cb5" },
    lib: "onnxruntime/capi/onnxruntime.dll",
    lib_name: "onnxruntime.dll",
});
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const RUNTIME: Option<Runtime> = Some(Runtime {
    wheel: File { url: pypi!("b2/93/d2b855793302ef57d55cc0583b72a00a8dcb01d9bdb3adac380f09b050e8/onnxruntime-1.31.1-cp313-cp313-macosx_14_0_arm64.whl"), name: "onnxruntime-macos-arm64.whl", size: 20_881_825, sha256: "e4ad067d36e2220218e085d7223283b2ee1bc73d887e811fd14d21cf901a11ee" },
    lib: "onnxruntime/capi/libonnxruntime.1.31.1.dylib",
    lib_name: "libonnxruntime.dylib",
});
/// Intel Macs: Microsoft no longer publishes ONNX Runtime for them.
#[cfg(not(any(
    all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")),
    all(target_os = "windows", any(target_arch = "x86_64", target_arch = "aarch64")),
    all(target_os = "macos", target_arch = "aarch64")
)))]
pub const RUNTIME: Option<Runtime> = None;

/// The licences kept beside the files they cover.
const RUNTIME_NOTICES: [&str; 2] = ["onnxruntime/LICENSE", "onnxruntime/ThirdPartyNotices.txt"];

/// the pack's own files (the runtime comes out of its wheel).
const FILES: [&File; 5] = [&MODEL, &VOICES, &EARS_ENC, &EARS_DEC, &EARS_TOKENS];

/// the pack before whisper joined it: its files are taken over, not fetched again.
const OLD_PACK: &str = "atc-voices-1";

/// Bytes to download in all.
pub fn download_size() -> u64 {
    FILES.iter().map(|f| f.size).sum::<u64>() + RUNTIME.as_ref().map_or(0, |r| r.wheel.size)
}

/// `<data>/fablegame/atc-voices-2/`.
pub fn dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("fablegame").join(PACK))
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Unsupported,
    NotInstalled,
    /// bytes so far, of
    Downloading(u64, u64),
    Installed,
    Failed(String),
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

pub fn state() -> State {
    let mut s = STATE.lock();
    if s.is_none() {
        *s = Some(if RUNTIME.is_none() {
            State::Unsupported
        } else if on_disk() {
            State::Installed
        } else {
            State::NotInstalled
        });
    }
    s.clone().unwrap()
}

fn set(st: State) {
    *STATE.lock() = Some(st);
}

fn on_disk() -> bool {
    let (Some(d), Some(rt)) = (dir(), RUNTIME.as_ref()) else { return false };
    FILES.iter().all(|f| std::fs::metadata(d.join(f.name)).is_ok_and(|m| m.len() == f.size)) && d.join(rt.lib_name).is_file()
}

/// Where everything is once installed: the model, the voices, the library.
pub struct Paths {
    pub model: PathBuf,
    pub voices: PathBuf,
    pub runtime: PathBuf,
    pub ears_enc: PathBuf,
    pub ears_dec: PathBuf,
    pub ears_tokens: PathBuf,
}

pub fn paths() -> Option<Paths> {
    let (d, rt) = (dir()?, RUNTIME.as_ref()?);
    on_disk().then(|| Paths { model: d.join(MODEL.name), voices: d.join(VOICES.name), runtime: d.join(rt.lib_name), ears_enc: d.join(EARS_ENC.name), ears_dec: d.join(EARS_DEC.name), ears_tokens: d.join(EARS_TOKENS.name) })
}

/// Starts downloading the pack on a thread.
pub fn install() {
    if matches!(state(), State::Downloading(..) | State::Unsupported) {
        return;
    }
    set(State::Downloading(0, download_size()));
    std::thread::spawn(|| {
        let r = download(&|done, total| set(State::Downloading(done, total)));
        set(match r {
            Ok(()) => State::Installed,
            Err(e) => {
                log::warn!("ATC voice download failed: {e}");
                State::Failed(e)
            }
        });
    });
}

/// Downloads and checks every file, unpacks the runtime from its wheel, then
/// moves the pack into place; nothing is left half-installed on failure.
pub fn download(progress: &dyn Fn(u64, u64)) -> Result<(), String> {
    use std::io::{Read, Write};
    let rt = RUNTIME.as_ref().ok_or("ATC voices aren't available on this platform")?;
    let d = dir().ok_or("no data folder")?;
    let tmp = d.with_extension("part");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let fail = |e: String| {
        let _ = std::fs::remove_dir_all(&tmp);
        e
    };
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(20)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(600)))
        .user_agent(crate::fetch::USER_AGENT)
        .build()
        .into();
    let total = download_size();
    let mut done = 0u64;
    let old = d.with_file_name(OLD_PACK);
    for f in FILES.into_iter().chain([&rt.wheel]) {
        if reuse(&old.join(f.name), &tmp.join(f.name), f) {
            done += f.size;
            progress(done, total);
            continue;
        }
        let mut r = agent.get(f.url).call().map_err(|e| fail(format!("{}: {e}", f.name)))?;
        let mut rd = r.body_mut().with_config().limit(f.size + 1).reader();
        let mut out = std::fs::File::create(tmp.join(f.name)).map_err(|e| fail(e.to_string()))?;
        let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
        let mut buf = vec![0u8; 1 << 20];
        let mut n_file = 0u64;
        loop {
            let n = rd.read(&mut buf).map_err(|e| fail(format!("{}: {e}", f.name)))?;
            if n == 0 {
                break;
            }
            ctx.update(&buf[..n]);
            out.write_all(&buf[..n]).map_err(|e| fail(e.to_string()))?;
            n_file += n as u64;
            done += n as u64;
            progress(done, total);
        }
        if n_file != f.size {
            return Err(fail(format!("{}: {n_file} bytes, expected {}", f.name, f.size)));
        }
        if hex(ctx.finish().as_ref()) != f.sha256 {
            return Err(fail(format!("{}: checksum mismatch", f.name)));
        }
    }
    // the library and its licences out of the (checked) wheel, then the wheel goes
    let wheel = tmp.join(rt.wheel.name);
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&wheel).map_err(|e| fail(e.to_string()))?).map_err(|e| fail(e.to_string()))?;
    for (inner, name) in std::iter::once((rt.lib, rt.lib_name)).chain(RUNTIME_NOTICES.iter().map(|n| (*n, n.rsplit('/').next().unwrap()))) {
        let mut e = zip.by_name(inner).map_err(|e| fail(format!("{inner}: {e}")))?;
        let name = if name == rt.lib_name { name.to_owned() } else { format!("onnxruntime-{name}") };
        let mut out = std::fs::File::create(tmp.join(name)).map_err(|e| fail(e.to_string()))?;
        std::io::copy(&mut e, &mut out).map_err(|e| fail(e.to_string()))?;
    }
    drop(zip);
    let _ = std::fs::remove_file(&wheel);
    std::fs::write(tmp.join("NOTICE.txt"), notice()).map_err(|e| fail(e.to_string()))?;
    let _ = std::fs::remove_dir_all(&d);
    std::fs::rename(&tmp, &d).map_err(|e| fail(e.to_string()))?;
    let _ = std::fs::remove_dir_all(&old);
    Ok(())
}

/// moves a file of the old pack into the new one when it is the very file wanted.
fn reuse(from: &std::path::Path, to: &std::path::Path, f: &File) -> bool {
    use std::io::Read;
    if !std::fs::metadata(from).is_ok_and(|m| m.len() == f.size) {
        return false;
    }
    let Ok(mut rd) = std::fs::File::open(from) else { return false };
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buf = vec![0u8; 1 << 20];
    while let Ok(n) = rd.read(&mut buf) {
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
    }
    drop(rd);
    hex(ctx.finish().as_ref()) == f.sha256 && std::fs::rename(from, to).is_ok()
}

/// an onnx session on the pack's runtime.
pub fn session(model: &std::path::Path, threads: usize) -> Result<ort::session::Session, String> {
    let p = paths().ok_or("the ATC voice pack isn't installed")?;
    runtime(&p)?;
    ort::session::Session::builder()
        .and_then(|b| b.with_intra_threads(threads).map_err(Into::into))
        .and_then(|b| b.with_inter_threads(1).map_err(Into::into))
        .and_then(|mut b| b.commit_from_file(model))
        .map_err(|e| format!("{}: {e}", model.display()))
}

/// loads onnx runtime from the pack, once a run (the voices and the ears share it).
pub fn runtime(p: &Paths) -> Result<(), String> {
    static RT: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        ort::init_from(&p.runtime).map_err(|e| format!("ONNX Runtime ({}): {e}", p.runtime.display()))?.with_name("fablegame-atc").commit();
        Ok(())
    })
    .clone()
}

/// Deletes the pack, and the one before it if left behind.
pub fn remove() {
    if let Some(d) = dir() {
        let _ = std::fs::remove_dir_all(d.with_file_name(OLD_PACK));
        let _ = std::fs::remove_dir_all(d);
    }
    set(if RUNTIME.is_none() { State::Unsupported } else { State::NotInstalled });
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn notice() -> String {
    format!(
        "Fable Flight's ATC voice pack, downloaded on request; not part of the game.\n\n\
         {model}, {voices}: Kokoro-82M v1.0 by hexgrad (https://huggingface.co/hexgrad/Kokoro-82M),\n\
         Apache License 2.0, as ONNX from {url}\n\n\
         {lib}: Microsoft ONNX Runtime {ORT_VERSION} (https://onnxruntime.ai), MIT License\n\
         (onnxruntime-LICENSE, onnxruntime-ThirdPartyNotices.txt), from the official\n\
         onnxruntime wheel on PyPI.\n",
        model = MODEL.name,
        url = MODEL.url.rsplit_once('/').map_or("", |u| u.0),
        voices = VOICES.name,
        lib = RUNTIME.as_ref().map_or("-", |r| r.lib_name),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_pinned() {
        for f in [&MODEL, &VOICES].into_iter().chain(RUNTIME.as_ref().map(|r| &r.wheel)) {
            assert_eq!(f.sha256.len(), 64, "{}", f.name);
            assert!(f.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()), "{}", f.name);
            assert!(f.size > 1_000_000, "{}", f.name);
        }
        assert!(MODEL.url.ends_with("/kokoro-v1.0.onnx") && VOICES.url.ends_with("/voices-v1.0.bin"));
    }
}
