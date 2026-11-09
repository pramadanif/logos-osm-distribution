//! Region discovery from Geofabrik's `index-v1-nogeom.json`, restricted to the predefined set.

use osm_core::{RegionDef, PREDEFINED};
use serde::Deserialize;

use crate::snapshot::latest_url;
use crate::SdkError;

#[derive(Deserialize)]
struct Index {
    features: Vec<Feature>,
}

#[derive(Deserialize)]
struct Feature {
    properties: Properties,
}

#[derive(Deserialize)]
struct Properties {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    urls: Urls,
}

#[derive(Deserialize, Default)]
struct Urls {
    pbf: Option<String>,
}

/// A predefined region as Geofabrik currently lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredRegion {
    pub def: &'static RegionDef,
    /// Display name from the index. Untrusted text: render as plain text only (H21).
    pub name: String,
}

/// Result of matching the index against the predefined set.
#[derive(Debug, Default)]
pub struct Discovery {
    /// In predefined-set order.
    pub regions: Vec<DiscoveredRegion>,
    /// Predefined regions the index no longer lists (renamed or removed upstream).
    pub missing: Vec<&'static RegionDef>,
}

/// Longest display name kept from the index, in bytes.
pub const MAX_NAME_BYTES: usize = 128;

/// Matches index entries to the set by their PBF URL, never by id string slicing, because index
/// ids and prize ids differ (docs/status/geofabrik.md). Entries outside the set are ignored.
pub fn discover_from_index(json: &[u8]) -> Result<Discovery, SdkError> {
    let index: Index = serde_json::from_slice(json).map_err(|e| SdkError::InvalidInput {
        field: "geofabrik index",
        why: e.to_string(),
    })?;
    let mut out = Discovery::default();
    for def in PREDEFINED.iter() {
        let want = latest_url(def);
        let hit = index
            .features
            .iter()
            .find(|f| f.properties.urls.pbf.as_deref() == Some(want.as_str()));
        match hit {
            Some(f) => out.regions.push(DiscoveredRegion {
                def,
                name: clean_name(&f.properties.name, &f.properties.id),
            }),
            None => out.missing.push(def),
        }
    }
    Ok(out)
}

/// Offline discovery: the bundled set alone, used when Geofabrik is off or unreachable (P-R4).
pub fn discover_offline() -> Discovery {
    Discovery {
        regions: PREDEFINED
            .iter()
            .map(|def| DiscoveredRegion {
                def,
                name: def.region.to_string(),
            })
            .collect(),
        missing: Vec::new(),
    }
}

/// Drops control and bidi-override characters and caps the length. Falls back to the id when
/// nothing printable is left. Letters in any script are kept (E56).
fn clean_name(name: &str, fallback: &str) -> String {
    let mut s = String::new();
    for c in name.chars() {
        let bidi = matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}');
        if c.is_control() || bidi {
            continue;
        }
        if s.len() + c.len_utf8() > MAX_NAME_BYTES {
            break;
        }
        s.push(c);
    }
    let s = s.trim().to_string();
    if s.is_empty() {
        fallback.to_string()
    } else {
        s
    }
}
