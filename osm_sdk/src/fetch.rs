//! Download a region (P-F3, SC-C.6): from Logos Storage by CID when hosted, Geofabrik only as a
//! fallback for regions nobody hosts. A hosted region never needs Geofabrik (P-R4).
//!
//! Entries are tried newest first; a download counts only when its MD5 equals the registered
//! checksum, so a junk entry cannot hide a good one (ADR 0002). Mismatching files are deleted.

use std::path::{Path, PathBuf};

use crate::geofabrik::{ensure_free_space, Geofabrik};
use crate::hash::{md5_file, to_hex};
use crate::registry::{Entry, Registry, RegistrySource};
use crate::storage::StorageBackend;
use crate::SdkError;

/// Where the file came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Storage { cid: String },
    Geofabrik { url: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    pub path: PathBuf,
    pub source: Source,
    pub md5: [u8; 16],
    /// Entries tried before the one that verified, with the reason each failed.
    pub skipped: Vec<(String, String)>,
}

fn file_name(region: &str, version: u32) -> String {
    format!("{}-{version:06}.osm.pbf", region.replace('/', "_"))
}

/// Downloads `region` into `dir`. `geofabrik` is `None` when the user turned it off.
pub fn fetch_region<S: RegistrySource>(
    registry: &Registry<S>,
    storage: &mut impl StorageBackend,
    geofabrik: Option<&Geofabrik>,
    region: &str,
    dir: &Path,
) -> Result<Fetched, SdkError> {
    let def = osm_core::lookup(region).ok_or_else(|| SdkError::InvalidInput {
        field: "region",
        why: format!("{region:?} is not in the predefined set"),
    })?;
    let mut entries: Vec<Entry> = registry
        .region(region)?
        .map(|a| a.history.into_iter().filter(|e| e.hosted).collect())
        .unwrap_or_default();
    entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
    let mut skipped = Vec::new();
    for e in &entries {
        ensure_free_space(dir, e.size_bytes)?;
        let dest = dir.join(file_name(region, e.version));
        if let Err(f) = storage.download_to_file(&e.cid, &dest) {
            skipped.push((e.cid.clone(), format!("{f:?}")));
            continue;
        }
        let got = md5_file(&dest, |_| {})?;
        if got == e.checksum {
            return Ok(Fetched {
                path: dest,
                source: Source::Storage { cid: e.cid.clone() },
                md5: got,
                skipped,
            });
        }
        let _ = std::fs::remove_file(&dest);
        skipped.push((
            e.cid.clone(),
            format!("MD5 {} != registered {}", to_hex(&got), to_hex(&e.checksum)),
        ));
    }
    if !entries.is_empty() {
        return Err(SdkError::NoVerifiedEntry {
            region: region.to_string(),
            entries: entries.len(),
        });
    }
    let Some(g) = geofabrik else {
        return Err(SdkError::RegionNotHosted {
            region: region.to_string(),
        });
    };
    let snap = crate::snapshot::first_of_month(g.latest_snapshot(def)?);
    let url = crate::snapshot::snapshot_url(def, snap)?;
    let (path, md5) = g.download_snapshot(def, snap, dir, |_, _| {})?;
    Ok(Fetched {
        path,
        source: Source::Geofabrik { url },
        md5,
        skipped,
    })
}
