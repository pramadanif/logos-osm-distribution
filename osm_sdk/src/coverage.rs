//! Coverage verification as the prize evaluator does it (P-A1, H7, H24).
//!
//! For each hosted registry entry: fetch the CID from Logos Storage, hash the bytes, and compare
//! with Geofabrik's published MD5 for that snapshot (the `.md5` next to the dated source URL).
//! A region counts once, when at least one of its entries verifies. A country counts when its
//! country-level entry verifies, or, for a decomposed country, when one of its subregions does.

use std::collections::BTreeSet;
use std::path::Path;

use crate::geofabrik::Geofabrik;
use crate::hash::{md5_file, to_hex};
use crate::registry::{Registry, RegistrySource};
use crate::storage::StorageBackend;
use crate::SdkError;

/// Prize floors (upstream LP-0018 Adoption section).
pub const MIN_ENTRIES: usize = 25;
pub const MIN_COUNTRIES: usize = 15;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryStatus {
    /// Storage bytes hash to Geofabrik's published MD5 for the snapshot.
    Verified,
    /// Bytes hash to the registered checksum, but Geofabrik's published MD5 differs.
    PublishedMd5Differs { published: String },
    /// Bytes do not hash to the registered checksum.
    BytesMismatch { got: String },
    /// Geofabrik no longer serves the snapshot's `.md5` (H24); bytes match the registered checksum.
    SnapshotGone,
    /// The CID could not be fetched.
    NotRetrievable(String),
}

#[derive(Clone, Debug)]
pub struct EntryReport {
    pub region: &'static str,
    pub cid: String,
    pub source_url: String,
    pub status: EntryStatus,
}

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub entries: Vec<EntryReport>,
    pub verified_regions: BTreeSet<&'static str>,
    pub countries: BTreeSet<&'static str>,
    pub errors: Vec<String>,
}

impl Report {
    pub fn passes(&self) -> bool {
        self.verified_regions.len() >= MIN_ENTRIES && self.countries.len() >= MIN_COUNTRIES
    }
}

/// Verifies every hosted entry (or only the newest per region with `newest_only`). Downloads go
/// to `dir` and are deleted after hashing.
pub fn verify_coverage<S: RegistrySource>(
    registry: &Registry<S>,
    storage: &mut impl StorageBackend,
    geofabrik: &Geofabrik,
    dir: &Path,
    newest_only: bool,
) -> Report {
    let mut report = Report::default();
    let (mut all, errors) = registry.all();
    report.errors.extend(errors.iter().map(|e| e.to_string()));
    all.retain(|r| r.entry.hosted);
    if newest_only {
        all.sort_by_key(|r| std::cmp::Reverse(r.entry.timestamp));
        let mut seen = BTreeSet::new();
        all.retain(|r| seen.insert(r.region));
    }
    for r in all {
        let status = verify_one(storage, geofabrik, dir, r.region, &r.entry);
        if status == EntryStatus::Verified {
            report.verified_regions.insert(r.region);
            let def = osm_core::lookup(r.region).expect("registry regions are in the set");
            report.countries.insert(def.parent.unwrap_or(def.region));
        }
        report.entries.push(EntryReport {
            region: r.region,
            cid: r.entry.cid.clone(),
            source_url: r.entry.source_url.clone(),
            status,
        });
    }
    report
}

fn verify_one(
    storage: &mut impl StorageBackend,
    geofabrik: &Geofabrik,
    dir: &Path,
    region: &str,
    e: &crate::registry::Entry,
) -> EntryStatus {
    let dest = dir.join(format!(
        "verify-{}-{:06}.osm.pbf",
        region.replace('/', "_"),
        e.version
    ));
    if let Err(f) = storage.download_to_file(&e.cid, &dest) {
        return EntryStatus::NotRetrievable(format!("{f:?}"));
    }
    let got = match md5_file(&dest, |_| {}) {
        Ok(g) => g,
        Err(err) => return EntryStatus::NotRetrievable(err.to_string()),
    };
    let _ = std::fs::remove_file(&dest);
    if got != e.checksum {
        return EntryStatus::BytesMismatch { got: to_hex(&got) };
    }
    let Some(def) = osm_core::lookup(region) else {
        return EntryStatus::NotRetrievable("region not in set".into());
    };
    match geofabrik.snapshot_md5(def, e.version) {
        Ok(published) if published == got => EntryStatus::Verified,
        Ok(published) => EntryStatus::PublishedMd5Differs {
            published: to_hex(&published),
        },
        Err(SdkError::SnapshotGone { .. }) => EntryStatus::SnapshotGone,
        Err(err) => EntryStatus::NotRetrievable(format!("Geofabrik: {err}")),
    }
}
