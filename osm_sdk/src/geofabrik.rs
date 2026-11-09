//! Geofabrik client: index, dated snapshot lookup and `.md5` files (H24, H26).
//!
//! Geofabrik is optional (P-R4). When it is disabled or unreachable, discovery falls back to the
//! bundled set and the caller is told why.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use osm_core::entry::GEOFABRIK_BASE;
use osm_core::RegionDef;

use crate::discovery::{discover_from_index, discover_offline, Discovery};
use crate::snapshot::{parse_md5_file, snapshot_from_location, snapshot_url};
use crate::SdkError;

/// The index is about 0.5 MB today; anything above this is refused.
const MAX_INDEX_BYTES: u64 = 16 * 1024 * 1024;
const MAX_MD5_BYTES: u64 = 1024;

pub struct Geofabrik {
    base: String,
    agent: ureq::Agent,
}

/// Where a discovery result came from.
#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    Index,
    /// Geofabrik turned off in settings.
    OfflineDisabled,
    /// Geofabrik could not be reached; the cause is kept for the status line.
    OfflineUnreachable(String),
}

impl Geofabrik {
    pub fn new() -> Self {
        Self::with_base(GEOFABRIK_BASE)
    }

    /// Test hook: another server with Geofabrik's layout.
    pub fn with_base(base: &str) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(15))
            .timeout_read(Duration::from_secs(60))
            .redirects(0)
            .user_agent(concat!(
                "logos-osm-distribution/",
                env!("CARGO_PKG_VERSION")
            ))
            .build();
        Geofabrik {
            base: base.to_string(),
            agent,
        }
    }

    fn url(&self, tail: &str) -> String {
        format!("{}{tail}", self.base)
    }

    fn get_capped(&self, url: &str, cap: u64) -> Result<Vec<u8>, SdkError> {
        let resp = self
            .agent
            .get(url)
            .call()
            .map_err(|e| unreachable_or_gone(url, e))?;
        let mut body = Vec::new();
        resp.into_reader()
            .take(cap + 1)
            .read_to_end(&mut body)
            .map_err(|e| SdkError::GeofabrikUnreachable {
                cause: format!("{url}: {e}"),
            })?;
        if body.len() as u64 > cap {
            return Err(SdkError::InvalidInput {
                field: "geofabrik response",
                why: format!("{url} is larger than {cap} bytes"),
            });
        }
        Ok(body)
    }

    pub fn fetch_index(&self) -> Result<Vec<u8>, SdkError> {
        self.get_capped(&self.url("index-v1-nogeom.json"), MAX_INDEX_BYTES)
    }

    /// Snapshot date of today's `-latest` file, read from its redirect without downloading it.
    pub fn latest_snapshot(&self, def: &RegionDef) -> Result<u32, SdkError> {
        let url = self.url(&format!("{}-latest.osm.pbf", def.geofabrik_path));
        let resp = match self.agent.head(&url).call() {
            Ok(r) => r,
            Err(ureq::Error::Status(_, r)) => r,
            Err(e) => return Err(unreachable_or_gone(&url, e)),
        };
        let loc = resp
            .header("location")
            .ok_or_else(|| SdkError::InvalidInput {
                field: "geofabrik redirect",
                why: format!("{url} returned HTTP {} without a Location", resp.status()),
            })?;
        // Validation accepts the real host only, so a test base is mapped back to it, and a
        // path-only Location is resolved against Geofabrik.
        let loc = match loc.strip_prefix('/') {
            Some(path) => format!("{GEOFABRIK_BASE}{path}"),
            None => loc.replacen(&self.base, GEOFABRIK_BASE, 1),
        };
        snapshot_from_location(def, &loc)
    }

    /// Geofabrik's published MD5 for the dated snapshot `yymmdd`.
    pub fn snapshot_md5(&self, def: &RegionDef, yymmdd: u32) -> Result<[u8; 16], SdkError> {
        let file = format!(
            "{}-{yymmdd:06}.osm.pbf",
            def.geofabrik_path
                .rsplit('/')
                .next()
                .unwrap_or(def.geofabrik_path)
        );
        let url = self.url(&format!("{}-{yymmdd:06}.osm.pbf.md5", def.geofabrik_path));
        let body = self.get_capped(&url, MAX_MD5_BYTES)?;
        let text = String::from_utf8(body).map_err(|_| SdkError::InvalidInput {
            field: "md5 file",
            why: "not UTF-8".into(),
        })?;
        parse_md5_file(&text, &file)
    }
}

/// Space kept free on top of the file itself (index, logs, the OS).
pub const DISK_MARGIN_BYTES: u64 = 512 * 1024 * 1024;

/// Fails with `DiskFull` unless `dir` has `need` bytes plus the margin free.
pub fn ensure_free_space(dir: &Path, need: u64) -> Result<(), SdkError> {
    let have = fs2::available_space(dir).map_err(|e| SdkError::io(dir, e))?;
    let want = need.saturating_add(DISK_MARGIN_BYTES);
    if have < want {
        return Err(SdkError::DiskFull {
            path: dir.to_path_buf(),
            need: want,
            have,
        });
    }
    Ok(())
}

impl Geofabrik {
    /// Streams the dated snapshot into `dir`, resuming a previous `.part` with a `Range` request,
    /// then checks it against Geofabrik's MD5 for that snapshot. Returns the file path and MD5.
    /// Nothing is held in memory beyond a 1 MiB buffer (H27).
    pub fn download_snapshot(
        &self,
        def: &RegionDef,
        yymmdd: u32,
        dir: &Path,
        mut progress: impl FnMut(u64, Option<u64>),
    ) -> Result<(PathBuf, [u8; 16]), SdkError> {
        let want_md5 = self.snapshot_md5(def, yymmdd)?;
        let public_url = snapshot_url(def, yymmdd)?;
        let name = crate::snapshot::file_name(&public_url).to_string();
        let url = self.url(&public_url[GEOFABRIK_BASE.len()..]);
        let dest = dir.join(&name);
        let part = dir.join(format!("{name}.part"));
        // A finished, verified copy is reused instead of downloading again (H26).
        if dest.exists() && crate::hash::md5_file(&dest, |_| {})? == want_md5 {
            return Ok((dest, want_md5));
        }
        let have = fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        let mut req = self.agent.get(&url);
        if have > 0 {
            req = req.set("range", &format!("bytes={have}-"));
        }
        let resp = req.call().map_err(|e| unreachable_or_gone(&url, e))?;
        let resumed = resp.status() == 206;
        let start = if resumed { have } else { 0 };
        let total = resp
            .header("content-length")
            .and_then(|v| v.parse::<u64>().ok())
            .map(|n| n.saturating_add(start));
        ensure_free_space(dir, total.unwrap_or(0).saturating_sub(start))?;
        let mut opts = OpenOptions::new();
        opts.create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600); // owner-only, like the rest of the data dir (H38)
        }
        let mut out = opts.open(&part).map_err(|e| SdkError::io(&part, e))?;
        let mut reader = resp.into_reader();
        let mut buf = vec![0u8; 1 << 20];
        let mut done = start;
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|e| SdkError::GeofabrikUnreachable {
                    cause: format!("{url}: {e} after {done} bytes; run again to resume"),
                })?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])
                .map_err(|e| SdkError::io(&part, e))?;
            done = done.saturating_add(n as u64);
            progress(done, total);
        }
        out.sync_all().map_err(|e| SdkError::io(&part, e))?;
        if let Some(t) = total {
            if done != t {
                return Err(SdkError::GeofabrikUnreachable {
                    cause: format!("{url}: got {done} of {t} bytes; run again to resume"),
                });
            }
        }
        let got = crate::hash::md5_file(&part, |_| {})?;
        if got != want_md5 {
            let _ = fs::remove_file(&part);
            return Err(SdkError::ImportChecksumMismatch {
                got: crate::hash::to_hex(&got),
                want: crate::hash::to_hex(&want_md5),
                snapshot: name,
            });
        }
        fs::rename(&part, &dest).map_err(|e| SdkError::io(&dest, e))?;
        Ok((dest, want_md5))
    }
}

impl Default for Geofabrik {
    fn default() -> Self {
        Self::new()
    }
}

fn unreachable_or_gone(url: &str, e: ureq::Error) -> SdkError {
    match e {
        ureq::Error::Status(404, _) | ureq::Error::Status(410, _) => SdkError::SnapshotGone {
            file: url.rsplit('/').next().unwrap_or(url).to_string(),
        },
        ureq::Error::Status(code, _) => SdkError::GeofabrikUnreachable {
            cause: format!("{url}: HTTP {code}"),
        },
        ureq::Error::Transport(t) => SdkError::GeofabrikUnreachable {
            cause: format!("{url}: {t}"),
        },
    }
}

/// Discovery that never fails: the live index when allowed and reachable, else the bundled set.
pub fn discover(geofabrik: Option<&Geofabrik>) -> (Discovery, Source) {
    let Some(g) = geofabrik else {
        return (discover_offline(), Source::OfflineDisabled);
    };
    match g.fetch_index().and_then(|b| discover_from_index(&b)) {
        Ok(d) => (d, Source::Index),
        Err(e) => (
            discover_offline(),
            Source::OfflineUnreachable(e.to_string()),
        ),
    }
}
