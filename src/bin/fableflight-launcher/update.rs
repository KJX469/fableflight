//! Updates from the project's GitHub releases. Each release carries one
//! archive per platform, `fableflight-<tag>-<os>-<arch>.zip` (Windows) or
//! `.tar.gz`, holding a folder with the executables, the README and the
//! licence (.github/workflows/release.yml), and `SHA256SUMS.txt`.
//!
//! An update downloads the archive, checks it against SHA256SUMS.txt, unpacks
//! the files into a staging folder beside the game, then swaps each into
//! place: the old file is renamed `<name>.old` first, which works even for
//! the launcher that is running (Windows will not let a running executable be
//! overwritten, but will let it be renamed). The `.old` files are deleted
//! the next time the launcher starts.
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const REPO: &str = "Rolandjg/fableflight";
pub const REPO_URL: &str = "https://github.com/Rolandjg/fableflight";
pub const DISCORD_URL: &str = "https://discord.gg/9akYzMRBc6";
const USER_AGENT: &str = concat!("fableflight-launcher/", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Debug)]
pub struct Release {
    pub tag: String,
    pub name: String,
    /// the release notes (GitHub markdown)
    pub body: String,
    /// "2026-11-18"
    pub date: String,
    pub url: String,
    pub assets: Vec<Asset>,
}

#[derive(Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

/// A version tag as numbers: "v1.0.1" is (1, 0, 1); a suffix ("-beta.1") is ignored.
pub fn version(tag: &str) -> Option<(u64, u64, u64)> {
    let t = tag.trim().trim_start_matches(['v', 'V']);
    let core = t.split(['-', '+']).next()?;
    let mut it = core.split('.').map(|p| p.parse::<u64>().ok());
    Some((it.next()??, it.next().flatten().unwrap_or(0), it.next().flatten().unwrap_or(0)))
}

/// This machine's name in the release archives.
pub fn platform() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "windows-x86_64",
        ("windows", "aarch64") => "windows-arm64",
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-arm64",
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-arm64",
        _ => return None,
    })
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(15)))
        .timeout_recv_body(Some(std::time::Duration::from_secs(300)))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

/// How long a fetched list is trusted before GitHub is asked again.
const FRESH_SECS: u64 = 30 * 60;

/// The release list as GitHub last sent it, kept in `<config>/fablegame/releases.json`.
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Cache {
    etag: String,
    /// seconds since 1970
    fetched: u64,
    json: String,
    /// `json` is the releases page's atom feed, not the api's list
    #[serde(default)]
    atom: bool,
}

impl Cache {
    fn releases(&self) -> Result<Vec<Release>, String> {
        if self.atom { parse_atom(&self.json) } else { parse(&self.json) }
    }
}

fn cache_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("fablegame").join("releases.json"))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn load_cache() -> Option<Cache> {
    serde_json::from_slice(&std::fs::read(cache_path()?).ok()?).ok()
}

fn save_cache(c: &Cache) {
    let Some(p) = cache_path() else { return };
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let tmp = p.with_extension("json.tmp");
    if std::fs::write(&tmp, serde_json::to_vec(c).unwrap_or_default()).is_ok() {
        let _ = std::fs::rename(&tmp, &p);
    }
}

/// The releases from the last fetch, at once and without the network.
pub fn saved_releases() -> Option<Vec<Release>> {
    load_cache()?.releases().ok()
}

/// What a check found: the releases, and a word on where they came from
/// when GitHub could not be asked (the list is then the saved one).
pub struct Fetched {
    pub list: Vec<Release>,
    pub note: Option<String>,
}

/// The published releases, newest first (drafts and pre-releases left out).
/// GitHub lets a network ask only 60 times an hour without signing in, so a
/// list fetched in the last half hour is used as it is unless `force`d, and
/// a refetch sends the saved ETag: an unchanged answer does not count.
pub fn releases(force: bool) -> Result<Fetched, String> {
    let cache = load_cache();
    if let Some(c) = cache.as_ref().filter(|c| !force && now_secs().saturating_sub(c.fetched) < FRESH_SECS) {
        if let Ok(list) = c.releases() {
            return Ok(Fetched { list, note: None });
        }
    }
    let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=30");
    let mut req = agent().get(&url).header("Accept", "application/vnd.github+json").config().http_status_as_error(false).build();
    if let Some(c) = cache.as_ref().filter(|c| !c.atom && !c.etag.is_empty()) {
        req = req.header("If-None-Match", &c.etag);
    }
    // the api refused: the releases page's feed has the notes too, and is
    // not under the api's hourly limit; else the notes saved last time.
    let failed = |why: String| {
        if let Ok((json, list)) = fetch_atom() {
            save_cache(&Cache { etag: String::new(), fetched: now_secs(), json, atom: true });
            return Ok(Fetched { list, note: None });
        }
        match cache.as_ref().and_then(|c| c.releases().ok()) {
            Some(list) => Ok(Fetched { list, note: Some(format!("{why} Showing the release notes saved from the last check.")) }),
            None => Err(why),
        }
    };
    let mut r = match req.call() {
        Ok(r) => r,
        Err(e) => return failed(format!("GitHub can't be reached ({e}).")),
    };
    let status = r.status().as_u16();
    let header = |k: &str| r.headers().get(k).and_then(|v| v.to_str().ok()).map(str::to_owned);
    match status {
        200 => {
            let etag = header("etag").unwrap_or_default();
            let json = r.body_mut().with_config().limit(8 << 20).read_to_string().map_err(|e| format!("GitHub's answer broke off: {e}"))?;
            let list = parse(&json)?;
            save_cache(&Cache { etag, fetched: now_secs(), json, atom: false });
            Ok(Fetched { list, note: None })
        }
        304 => {
            let mut c = cache.unwrap_or_default();
            c.fetched = now_secs();
            let list = c.releases()?;
            save_cache(&c);
            Ok(Fetched { list, note: None })
        }
        403 | 429 if header("x-ratelimit-remaining").as_deref() == Some("0") => {
            let wait = header("x-ratelimit-reset").and_then(|v| v.parse::<u64>().ok()).map(|t| t.saturating_sub(now_secs()).div_ceil(60).max(1));
            let when = wait.map_or(String::new(), |m| format!(", it resets in {m} min"));
            failed(format!("GitHub's limit of 60 update checks an hour for this network is used up{when}."))
        }
        404 => failed(format!("GitHub has no releases page for {REPO} (404).")),
        s => failed(format!("GitHub answered with an error ({s}).")),
    }
}

/// The releases page's atom feed: its text and the releases in it.
fn fetch_atom() -> Result<(String, Vec<Release>), String> {
    let mut r = agent().get(&format!("{REPO_URL}/releases.atom")).call().map_err(|e| format!("GitHub: {e}"))?;
    let text = r.body_mut().with_config().limit(8 << 20).read_to_string().map_err(|e| format!("GitHub: {e}"))?;
    let list = parse_atom(&text)?;
    Ok((text, list))
}

/// The text between `open` and `close` after `at` in `s`.
fn between<'a>(s: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let a = s.find(open)? + open.len();
    let b = s[a..].find(close)? + a;
    Some(&s[a..b])
}

/// The few entities a feed and github's html use.
fn unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&#x27;", "'").replace("&nbsp;", " ").replace("&amp;", "&")
}

/// Release notes as github renders them, back to the simple markdown the
/// launcher draws: headings, bullets, and links as their words (a pull
/// request as its number, a full url where the words are the url's).
fn html_to_notes(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(i) = rest.find('<') {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i..].find('>') else { break };
        let tag = &rest[i + 1..i + j];
        let name = tag.trim_start_matches('/').split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_ascii_lowercase();
        let closing = tag.starts_with('/');
        rest = &rest[i + j + 1..];
        match (name.as_str(), closing) {
            ("h1" | "h2" | "h3" | "h4", false) => out.push_str("\n## "),
            ("li", false) => out.push_str("\n- "),
            ("p" | "br" | "ul" | "ol" | "h1" | "h2" | "h3" | "h4", _) => out.push('\n'),
            ("a", false) => {
                // the words of the link, or its address where they are only a shortening of it.
                let href = between(tag, "href=\"", "\"").unwrap_or("").to_string();
                let end = rest.to_ascii_lowercase().find("</a>").unwrap_or(rest.len());
                let words = html_to_notes(&rest[..end]);
                let words = words.trim();
                let compare = href.contains("/compare/");
                out.push_str(if compare && !href.is_empty() { &href } else { words });
                rest = rest.get(end + 4..).unwrap_or("");
            }
            _ => {}
        }
    }
    out.push_str(rest);
    // one line per heading, item or paragraph, no blank lines between them.
    let text = unescape(&out);
    let tidy: Vec<&str> = text.lines().map(str::trim_end).filter(|l| !l.trim().is_empty()).collect();
    tidy.join("\n").trim().to_string()
}

/// The releases page's atom feed as releases, newest first. The feed has no
/// files, so each release's are the ones the release workflow always makes:
/// the archive for this machine and SHA256SUMS.txt (sizes come with the download).
fn parse_atom(xml: &str) -> Result<Vec<Release>, String> {
    let mut out = Vec::new();
    for entry in xml.split("<entry>").skip(1) {
        let Some(url) = between(entry, "rel=\"alternate\" type=\"text/html\" href=\"", "\"") else { continue };
        let Some(tag) = url.rsplit("/releases/tag/").next().filter(|t| *t != url) else { continue };
        if version(tag).is_none() {
            continue;
        }
        let name = between(entry, "<title>", "</title>").map(unescape).filter(|n| !n.trim().is_empty()).unwrap_or_else(|| tag.to_string());
        let date = between(entry, "<updated>", "</updated>").unwrap_or("").chars().take(10).collect();
        let body = between(entry, "<content type=\"html\">", "</content>").map(|c| html_to_notes(&unescape(c))).unwrap_or_default();
        let dl = |file: String| Asset { url: format!("{REPO_URL}/releases/download/{tag}/{file}"), name: file, size: 0 };
        let mut assets = Vec::new();
        if let Some(p) = platform() {
            let ext = if cfg!(windows) { "zip" } else { "tar.gz" };
            assets.push(dl(format!("fableflight-{tag}-{p}.{ext}")));
            assets.push(dl("SHA256SUMS.txt".to_string()));
        }
        out.push(Release { tag: tag.to_string(), name, body, date, url: url.to_string(), assets });
    }
    if out.is_empty() {
        return Err("GitHub's release feed lists no releases".into());
    }
    out.sort_by(|a, b| version(&b.tag).cmp(&version(&a.tag)));
    Ok(out)
}

/// GitHub's release list as releases, newest first.
fn parse(text: &str) -> Result<Vec<Release>, String> {
    let j: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("GitHub's answer: {e}"))?;
    let list = j.as_array().ok_or("GitHub's answer is not a list of releases")?;
    let mut out: Vec<Release> = list
        .iter()
        .filter(|r| !r["draft"].as_bool().unwrap_or(false) && !r["prerelease"].as_bool().unwrap_or(false))
        .filter_map(|r| {
            let tag = r["tag_name"].as_str()?.to_string();
            version(&tag)?;
            Some(Release {
                name: r["name"].as_str().filter(|s| !s.is_empty()).unwrap_or(&tag).to_string(),
                body: r["body"].as_str().unwrap_or("").replace("\r\n", "\n"),
                date: r["published_at"].as_str().unwrap_or("").chars().take(10).collect(),
                url: r["html_url"].as_str().unwrap_or(REPO_URL).to_string(),
                assets: r["assets"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| Some(Asset { name: x["name"].as_str()?.to_string(), url: x["browser_download_url"].as_str()?.to_string(), size: x["size"].as_u64().unwrap_or(0) }))
                            .collect()
                    })
                    .unwrap_or_default(),
                tag,
            })
        })
        .collect();
    out.sort_by(|a, b| version(&b.tag).cmp(&version(&a.tag)));
    Ok(out)
}

impl Release {
    /// The archive for this machine.
    pub fn archive(&self) -> Option<&Asset> {
        let p = platform()?;
        self.assets.iter().find(|a| a.name.contains(p) && (a.name.ends_with(".zip") || a.name.ends_with(".tar.gz")))
    }
}

/// Where an update is in its work.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    Downloading,
    Checking,
    Installing,
}

/// What an update did.
pub struct Installed {
    pub tag: String,
    /// the launcher itself was replaced: restart it to run the new one
    pub launcher_replaced: bool,
}

/// Downloads `rel`'s archive for this machine and puts its files in `dir`.
pub fn install(rel: &Release, dir: &Path, progress: &dyn Fn(Phase, u64, u64)) -> Result<Installed, String> {
    let asset = rel.archive().ok_or_else(|| format!("{} has no download for {}", rel.tag, platform().unwrap_or(std::env::consts::OS)))?;
    let staging = dir.join(".fableflight-update");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| format!("can't write to {}: {e}", dir.display()))?;
    let result = (|| {
        let archive = staging.join(&asset.name);
        let digest = download(&asset.url, asset.size, &archive, &|done, total| progress(Phase::Downloading, done, total))?;
        progress(Phase::Checking, 0, 1);
        match rel.assets.iter().find(|a| a.name == "SHA256SUMS.txt") {
            Some(sums) => {
                let mut r = agent().get(&sums.url).call().map_err(|e| format!("checksums: {e}"))?;
                let text = r.body_mut().with_config().limit(1 << 20).read_to_string().map_err(|e| format!("checksums: {e}"))?;
                let want = text
                    .lines()
                    .find_map(|l| {
                        let mut it = l.split_whitespace();
                        let (h, n) = (it.next()?, it.next()?);
                        (n.trim_start_matches('*') == asset.name).then(|| h.to_ascii_lowercase())
                    })
                    .ok_or("the archive is not in SHA256SUMS.txt")?;
                if want != digest {
                    return Err("the download is damaged (its SHA-256 does not match the release's)".to_string());
                }
            }
            None => return Err("the release has no SHA256SUMS.txt to check the download against".into()),
        }
        progress(Phase::Installing, 0, 1);
        let files = unpack(&archive, &staging)?;
        let _ = std::fs::remove_file(&archive);
        if !files.iter().any(|f| f == game_name()) {
            return Err(format!("the archive has no {}", game_name()));
        }
        let mut launcher_replaced = false;
        for f in &files {
            swap_in(&staging.join(f), &dir.join(f))?;
            launcher_replaced |= f == launcher_name();
        }
        Ok(Installed { tag: rel.tag.clone(), launcher_replaced })
    })();
    let _ = std::fs::remove_dir_all(&staging);
    result
}

/// Streams a file to disk; its SHA-256 in hex.
fn download(url: &str, size: u64, to: &Path, progress: &dyn Fn(u64, u64)) -> Result<String, String> {
    let mut r = agent().get(url).call().map_err(|e| format!("download: {e}"))?;
    let total = r.body().content_length().unwrap_or(size).max(1);
    // (from the feed the size is not known: up to 4 GB then)
    let limit = if r.body().content_length().is_none() && size == 0 { 4 << 30 } else { total.max(size) + (1 << 20) };
    let mut rd = r.body_mut().with_config().limit(limit).reader();
    let mut out = std::io::BufWriter::new(std::fs::File::create(to).map_err(|e| e.to_string())?);
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buf = vec![0u8; 256 << 10];
    let mut done = 0u64;
    loop {
        let n = rd.read(&mut buf).map_err(|e| format!("download: {e}"))?;
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
        out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as u64;
        progress(done, total);
    }
    out.flush().map_err(|e| e.to_string())?;
    Ok(ctx.finish().as_ref().iter().map(|b| format!("{b:02x}")).collect())
}

pub fn game_name() -> &'static str {
    if cfg!(windows) { "fablegame.exe" } else { "fablegame" }
}

pub fn launcher_name() -> &'static str {
    if cfg!(windows) { "fableflight-launcher.exe" } else { "fableflight-launcher" }
}

/// A file of the archive's top folder: just its name, nothing deeper and
/// nothing that climbs out.
fn top_file(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split(['/', '\\']).filter(|p| !p.is_empty() && *p != ".").collect();
    let name = match parts.as_slice() {
        [_, name] | [name] => *name,
        _ => return None,
    };
    (name != ".." && !name.contains(':')).then(|| name.to_string())
}

/// Unpacks the archive's top-level files into `dir`; their names.
fn unpack(archive: &Path, dir: &Path) -> Result<Vec<String>, String> {
    let name = archive.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let mut files = Vec::new();
    if name.ends_with(".zip") {
        let f = std::fs::File::open(archive).map_err(|e| e.to_string())?;
        let mut z = zip::ZipArchive::new(f).map_err(|e| format!("archive: {e}"))?;
        for i in 0..z.len() {
            let mut e = z.by_index(i).map_err(|e| format!("archive: {e}"))?;
            if !e.is_file() {
                continue;
            }
            let Some(n) = e.name().ok().and_then(|s| top_file(&s)) else { continue };
            let mut out = std::fs::File::create(dir.join(&n)).map_err(|e| e.to_string())?;
            std::io::copy(&mut e, &mut out).map_err(|e| format!("archive: {e}"))?;
            set_mode(&dir.join(&n), e.unix_mode().unwrap_or(0o644));
            files.push(n);
        }
    } else {
        let f = std::fs::File::open(archive).map_err(|e| e.to_string())?;
        let mut gz = std::io::BufReader::new(flate2::read::GzDecoder::new(f));
        let mut long_name: Option<String> = None;
        loop {
            let mut h = [0u8; 512];
            if read_full(&mut gz, &mut h)? < 512 || h.iter().all(|b| *b == 0) {
                break;
            }
            let field = |a: usize, b: usize| String::from_utf8_lossy(&h[a..b]).trim_end_matches('\0').to_string();
            let octal = |a: usize, b: usize| u64::from_str_radix(field(a, b).trim(), 8).unwrap_or(0);
            let size = octal(124, 136);
            let mode = octal(100, 108) as u32;
            let kind = h[156];
            let mut path = field(0, 100);
            if &h[257..262] == b"ustar" {
                let prefix = field(345, 500);
                if !prefix.is_empty() {
                    path = format!("{prefix}/{path}");
                }
            }
            if let Some(l) = long_name.take() {
                path = l;
            }
            let padded = size.div_ceil(512) * 512;
            match kind {
                b'L' => {
                    let mut data = vec![0u8; padded as usize];
                    read_full(&mut gz, &mut data)?;
                    long_name = Some(String::from_utf8_lossy(&data[..size as usize]).trim_end_matches('\0').to_string());
                    continue;
                }
                b'0' | 0 => {
                    if let Some(n) = top_file(&path) {
                        let mut out = std::fs::File::create(dir.join(&n)).map_err(|e| e.to_string())?;
                        std::io::copy(&mut (&mut gz).take(size), &mut out).map_err(|e| format!("archive: {e}"))?;
                        std::io::copy(&mut (&mut gz).take(padded - size), &mut std::io::sink()).map_err(|e| format!("archive: {e}"))?;
                        set_mode(&dir.join(&n), mode);
                        files.push(n);
                        continue;
                    }
                }
                _ => {}
            }
            std::io::copy(&mut (&mut gz).take(padded), &mut std::io::sink()).map_err(|e| format!("archive: {e}"))?;
        }
    }
    Ok(files)
}

fn read_full(r: &mut impl Read, buf: &mut [u8]) -> Result<usize, String> {
    let mut n = 0;
    while n < buf.len() {
        match r.read(&mut buf[n..]).map_err(|e| format!("archive: {e}"))? {
            0 => break,
            k => n += k,
        }
    }
    Ok(n)
}

#[cfg(unix)]
fn set_mode(p: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let exec = p.file_name().and_then(|n| n.to_str()).is_some_and(|n| !n.contains('.'));
    let m = if exec || mode & 0o111 != 0 { 0o755 } else { 0o644 };
    let _ = std::fs::set_permissions(p, std::fs::Permissions::from_mode(m));
}

#[cfg(not(unix))]
fn set_mode(_: &Path, _: u32) {}

/// Puts `new` at `dest`, the file there renamed `<name>.old` first.
fn swap_in(new: &Path, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        let mut old = old_name(dest, 0);
        let mut k = 0;
        // (an .old still in use from the last update can't be deleted: take the next name)
        while old.exists() && std::fs::remove_file(&old).is_err() {
            k += 1;
            old = old_name(dest, k);
        }
        std::fs::rename(dest, &old).map_err(|e| format!("can't replace {}: {e} (is the game still running?)", dest.display()))?;
    }
    std::fs::rename(new, dest).or_else(|_| std::fs::copy(new, dest).map(|_| ())).map_err(|e| format!("can't write {}: {e}", dest.display()))
}

fn old_name(dest: &Path, k: u32) -> PathBuf {
    let n = dest.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    dest.with_file_name(if k == 0 { format!("{n}.old") } else { format!("{n}.old{k}") })
}

/// Deletes what the last update left behind.
pub fn clean_up(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir.join(".fableflight-update"));
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            let ours = n.starts_with("fablegame") || n.starts_with("fableflight") || n.starts_with("README") || n.starts_with("LICENSE");
            if ours && (n.ends_with(".old") || n.rsplit_once(".old").is_some_and(|(_, k)| !k.is_empty() && k.chars().all(|c| c.is_ascii_digit()))) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
}

/// The installed game's version (`fablegame --version`; games before 1.0.2
/// don't answer it).
pub fn game_version(exe: &Path) -> Option<String> {
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--version").stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let out = cmd.output().ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let v = s.trim().strip_prefix("fablegame ")?;
    (out.status.success() && version(v).is_some()).then(|| format!("v{v}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert_eq!(version("v1.0.1"), Some((1, 0, 1)));
        assert_eq!(version("1.2"), Some((1, 2, 0)));
        assert_eq!(version("v1.1.0-beta.1"), Some((1, 1, 0)));
        assert_eq!(version("nightly"), None);
        assert!(version("v1.10.0") > version("v1.9.3"));
    }

    #[test]
    fn release_list() {
        let json = r#"[
            {"tag_name": "v1.1.0", "name": "", "body": "a\r\nb", "published_at": "2026-11-20T10:00:00Z", "html_url": "u0", "assets": [], "draft": false, "prerelease": false},
            {"tag_name": "v1.2.0", "name": "FableFlight v1.2.0", "body": "", "published_at": "2026-11-22T10:00:00Z", "html_url": "u1",
             "assets": [{"name": "fableflight-v1.2.0-windows-x86_64.zip", "browser_download_url": "d", "size": 5}], "draft": false, "prerelease": false},
            {"tag_name": "v1.3.0-rc1", "name": "rc", "draft": false, "prerelease": true},
            {"tag_name": "nightly", "name": "n", "draft": false, "prerelease": false}
        ]"#;
        let list = parse(json).unwrap();
        // newest first, pre-releases and odd tags left out, an empty name falls back to the tag.
        assert_eq!(list.iter().map(|r| r.tag.as_str()).collect::<Vec<_>>(), ["v1.2.0", "v1.1.0"]);
        assert_eq!(list[1].name, "v1.1.0");
        assert_eq!(list[1].body, "a\nb");
        assert_eq!(list[0].date, "2026-11-22");
        assert_eq!(list[0].assets[0].size, 5);
        assert!(parse("{\"message\": \"Not Found\"}").is_err());
    }

    #[test]
    fn release_feed() {
        let xml = r#"<feed><entry>
    <updated>2026-11-22T09:42:36Z</updated>
    <link rel="alternate" type="text/html" href="https://github.com/Rolandjg/fableflight/releases/tag/v1.1.1"/>
    <title>FableFlight v1.1.1</title>
    <content type="html">&lt;h2&gt;What&amp;#39;s Changed&lt;/h2&gt;
&lt;ul&gt;
&lt;li&gt;Fix it by &lt;a href=&quot;https://github.com/x&quot;&gt;@x&lt;/a&gt; in &lt;a href=&quot;https://github.com/Rolandjg/fableflight/pull/64&quot;&gt;#64&lt;/a&gt;&lt;/li&gt;
&lt;/ul&gt;
&lt;p&gt;&lt;strong&gt;Full Changelog&lt;/strong&gt;: &lt;a href=&quot;https://github.com/Rolandjg/fableflight/compare/v1.1.0...v1.1.1&quot;&gt;&lt;tt&gt;v1.1.0...v1.1.1&lt;/tt&gt;&lt;/a&gt;&lt;/p&gt;</content>
  </entry><entry>
    <link rel="alternate" type="text/html" href="https://github.com/Rolandjg/fableflight/releases/tag/v1.1.0"/>
    <title>FableFlight v1.1.0</title>
  </entry></feed>"#;
        let list = parse_atom(xml).unwrap();
        assert_eq!(list.iter().map(|r| r.tag.as_str()).collect::<Vec<_>>(), ["v1.1.1", "v1.1.0"]);
        assert_eq!(list[0].date, "2026-11-22");
        assert_eq!(list[0].body, "## What's Changed\n- Fix it by @x in #64\nFull Changelog: https://github.com/Rolandjg/fableflight/compare/v1.1.0...v1.1.1");
        if platform().is_some() {
            assert!(list[0].archive().is_some_and(|a| a.url.ends_with(&format!("/releases/download/v1.1.1/{}", a.name))));
        }
    }

    #[test]
    fn archive_paths() {
        assert_eq!(top_file("fableflight-v1.0.1-linux-x86_64/fablegame").as_deref(), Some("fablegame"));
        assert_eq!(top_file("./fableflight-v1/README.md").as_deref(), Some("README.md"));
        assert_eq!(top_file("a/b/c"), None);
        assert_eq!(top_file("a/.."), None);
        assert_eq!(top_file("dir\\fablegame.exe").as_deref(), Some("fablegame.exe"));
    }

    #[test]
    fn unpacks_a_tarball() {
        // a tarball as the release workflow makes it (tar czf)
        let dir = std::env::temp_dir().join(format!("ff-launcher-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src/fableflight-v9.9.9-test")).unwrap();
        std::fs::write(dir.join("src/fableflight-v9.9.9-test/fablegame"), b"#!/bin/sh\necho hi\n").unwrap();
        std::fs::write(dir.join("src/fableflight-v9.9.9-test/README.md"), vec![b'x'; 1500]).unwrap();
        let ok = std::process::Command::new("tar").args(["czf", "../a.tar.gz", "fableflight-v9.9.9-test"]).current_dir(dir.join("src")).status().is_ok_and(|s| s.success());
        if !ok {
            return; // no tar here
        }
        let out = dir.join("out");
        std::fs::create_dir_all(&out).unwrap();
        let mut files = unpack(&dir.join("a.tar.gz"), &out).unwrap();
        files.sort();
        assert_eq!(files, ["README.md", "fablegame"]);
        assert_eq!(std::fs::read(out.join("README.md")).unwrap().len(), 1500);
        assert_eq!(std::fs::read(out.join("fablegame")).unwrap(), b"#!/bin/sh\necho hi\n");
        // and swapping a file in keeps the old one beside it until the clean-up
        std::fs::write(dir.join("fablegame"), b"old").unwrap();
        swap_in(&out.join("fablegame"), &dir.join("fablegame")).unwrap();
        assert_eq!(std::fs::read(dir.join("fablegame.old")).unwrap(), b"old");
        clean_up(&dir);
        assert!(!dir.join("fablegame.old").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
