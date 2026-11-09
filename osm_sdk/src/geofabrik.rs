//! Geofabrik client: index, dated snapshot lookup and `.md5` files (H24, H26).
//!
//! Geofabrik is optional (P-R4). When it is disabled or unreachable, discovery falls back to the
//! bundled set and the caller is told why.

use std::io::Read;
use std::time::Duration;

use osm_core::entry::GEOFABRIK_BASE;
use osm_core::RegionDef;

use crate::discovery::{discover_from_index, discover_offline, Discovery};
use crate::snapshot::{parse_md5_file, snapshot_from_location};
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
