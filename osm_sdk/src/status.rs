//! Hosted status and update check (P-F6, P-U4, SC-C.5).

use osm_core::PREDEFINED;

use crate::geofabrik::Geofabrik;
use crate::registry::{Registry, RegistrySource};
use crate::snapshot::first_of_month;
use crate::SdkError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostedStatus {
    pub region: &'static str,
    /// Newest hosted snapshot (YYMMDD), if any.
    pub version: Option<u32>,
    /// Distinct hosting accounts of that entry.
    pub hosts: usize,
}

/// Hosted status of every predefined region, in set order. Errors on single regions are
/// returned next to the list, never silently dropped.
pub fn hosted_status<S: RegistrySource>(
    registry: &Registry<S>,
) -> (Vec<HostedStatus>, Vec<SdkError>) {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    for def in PREDEFINED.iter() {
        match registry.resolve(def.region) {
            Ok(r) => out.push(HostedStatus {
                region: def.region,
                version: r.as_ref().map(|r| r.version),
                hosts: r.map(|r| r.hosts).unwrap_or(0),
            }),
            Err(e) => errors.push(e),
        }
    }
    (out, errors)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateStatus {
    pub region: &'static str,
    /// Newest registered snapshot, if any.
    pub registered: Option<u32>,
    /// Newest first-of-month snapshot Geofabrik serves.
    pub available: u32,
}

impl UpdateStatus {
    pub fn update_available(&self) -> bool {
        self.registered.is_none_or(|r| r < self.available)
    }
}

/// Compares, per region path, the newest registered snapshot with Geofabrik's current
/// first-of-month snapshot. Each region is tracked on its own (country and subregions apart).
pub fn check_updates<S: RegistrySource>(
    registry: &Registry<S>,
    geofabrik: &Geofabrik,
    regions: &[&str],
) -> Result<Vec<UpdateStatus>, SdkError> {
    let mut out = Vec::new();
    for region in regions {
        let def = osm_core::lookup(region).ok_or_else(|| SdkError::InvalidInput {
            field: "region",
            why: format!("{region:?} is not in the predefined set"),
        })?;
        let registered = registry.resolve(def.region)?.map(|r| r.version);
        let available = first_of_month(geofabrik.latest_snapshot(def)?);
        out.push(UpdateStatus {
            region: def.region,
            registered,
            available,
        });
    }
    Ok(out)
}
