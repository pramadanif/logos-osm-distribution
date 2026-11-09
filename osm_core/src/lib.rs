//! Shared types and validation for the LP-0018 OSM registry.
//!
//! `no_std` so the registry guest program and the SDK use one validation path.
#![no_std]

pub mod entry;
mod regions;

pub use regions::PREDEFINED;

/// Position of a region in the Geofabrik tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Country,
    Subregion,
}

impl Level {
    pub const fn as_str(self) -> &'static str {
        match self {
            Level::Country => "country",
            Level::Subregion => "subregion",
        }
    }
}

/// One leaf of the predefined set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionDef {
    /// Prize leaf id, for example `germany` or `india/central-zone`. Unique in the set.
    pub region: &'static str,
    /// Decomposed country (`us`, `india`, `china`, `russia`) for subregions, `None` for countries.
    pub parent: Option<&'static str>,
    pub level: Level,
    /// Geofabrik download path without the `-latest.osm.pbf` / `-YYMMDD.osm.pbf` suffix.
    pub geofabrik_path: &'static str,
}

/// Look up a region by its prize id.
pub fn lookup(region: &str) -> Option<&'static RegionDef> {
    PREDEFINED.iter().find(|r| r.region == region)
}

/// True when `region`, `parent` and `level` together describe a leaf of the predefined set.
pub fn is_valid_position(region: &str, parent: Option<&str>, level: Level) -> bool {
    matches!(lookup(region), Some(r) if r.parent == parent && r.level == level)
}
