//! Dated Geofabrik snapshots (H24): URL building, `-latest` redirect parsing, `.md5` parsing.

use osm_core::entry::{parse_md5_hex, snapshot_from_source_url, GEOFABRIK_BASE};
use osm_core::RegionDef;

use crate::SdkError;

pub fn latest_url(def: &RegionDef) -> String {
    format!("{GEOFABRIK_BASE}{}-latest.osm.pbf", def.geofabrik_path)
}

/// URL of the dated snapshot `YYMMDD`.
pub fn snapshot_url(def: &RegionDef, yymmdd: u32) -> Result<String, SdkError> {
    let url = format!("{GEOFABRIK_BASE}{}-{yymmdd:06}.osm.pbf", def.geofabrik_path);
    match snapshot_from_source_url(def.region, &url) {
        Ok(v) if v == yymmdd => Ok(url),
        _ => Err(SdkError::InvalidInput {
            field: "snapshot",
            why: format!("{yymmdd} is not a YYMMDD date"),
        }),
    }
}

/// First-of-month snapshot for a date, the form coverage registers (docs/status/geofabrik.md).
pub fn first_of_month(yymmdd: u32) -> u32 {
    yymmdd / 100 * 100 + 1
}

/// Reads the snapshot date from the `Location` of a `-latest.osm.pbf` redirect.
pub fn snapshot_from_location(def: &RegionDef, location: &str) -> Result<u32, SdkError> {
    snapshot_from_source_url(def.region, location).map_err(|_| SdkError::InvalidInput {
        field: "redirect location",
        why: format!("not a dated snapshot of {}: {location:?}", def.region),
    })
}

/// Parses a Geofabrik `.md5` file (`<32 hex>  <file name>`), requiring the file name to match.
pub fn parse_md5_file(text: &str, expected_file: &str) -> Result<[u8; 16], SdkError> {
    let bad = |why: &str| SdkError::InvalidInput {
        field: "md5 file",
        why: why.to_string(),
    };
    let line = text.lines().next().ok_or_else(|| bad("empty"))?;
    let (hex, name) = line
        .split_once("  ")
        .ok_or_else(|| bad("expected '<md5>  <file>'"))?;
    if name != expected_file {
        return Err(bad(&format!("lists {name:?}, expected {expected_file:?}")));
    }
    parse_md5_hex(hex).map_err(|_| bad("checksum is not 32 lowercase hex characters"))
}

pub fn file_name(url: &str) -> &str {
    url.rsplit('/').next().unwrap_or(url)
}
