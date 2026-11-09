//! SC-0.2: the frozen set matches the prize and the committed JSON.

use osm_core::{is_valid_position, lookup, Level, PREDEFINED};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct Doc {
    regions: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    region: String,
    parent: Option<String>,
    level: String,
    geofabrik_path: String,
}

const DECOMPOSED: [&str; 4] = ["us", "india", "china", "russia"];

fn json() -> Doc {
    let raw = include_str!("../../regions/predefined.json");
    serde_json::from_str(raw).expect("regions/predefined.json parses")
}

#[test]
fn counts_match_prize() {
    assert_eq!(PREDEFINED.len(), 72);
    let countries = PREDEFINED
        .iter()
        .filter(|r| r.level == Level::Country)
        .count();
    assert_eq!(countries, 48);
    assert_eq!(PREDEFINED.len() - countries, 24);
    let per_parent = |p: &str| PREDEFINED.iter().filter(|r| r.parent == Some(p)).count();
    assert_eq!(
        DECOMPOSED.map(per_parent),
        [8, 6, 6, 4],
        "us 8, india 6, china 6, russia 4"
    );
}

#[test]
fn ids_unique_and_positions_consistent() {
    let ids: BTreeSet<_> = PREDEFINED.iter().map(|r| r.region).collect();
    assert_eq!(ids.len(), PREDEFINED.len());
    for r in &PREDEFINED {
        match (r.level, r.parent) {
            (Level::Country, None) => assert!(!r.region.contains('/'), "{}", r.region),
            (Level::Subregion, Some(p)) => {
                assert!(DECOMPOSED.contains(&p), "{}", r.region);
                assert_eq!(r.region.split_once('/').map(|(a, _)| a), Some(p));
            }
            _ => panic!("level/parent mismatch for {}", r.region),
        }
    }
}

#[test]
fn no_ancestor_or_descendant_pair() {
    for d in DECOMPOSED {
        assert!(
            lookup(d).is_none(),
            "decomposed country {d} must not be in the set"
        );
    }
    for a in &PREDEFINED {
        for b in &PREDEFINED {
            if a.region == b.region {
                continue;
            }
            let prefix = format!("{}/", a.region);
            assert!(
                !b.region.starts_with(&prefix),
                "{} contains {}",
                a.region,
                b.region
            );
            let gprefix = format!("{}/", a.geofabrik_path);
            assert!(
                !b.geofabrik_path.starts_with(&gprefix),
                "{} contains {} on Geofabrik",
                a.geofabrik_path,
                b.geofabrik_path
            );
        }
    }
}

#[test]
fn rust_table_equals_committed_json() {
    let doc = json();
    assert_eq!(doc.regions.len(), PREDEFINED.len());
    for (e, r) in doc.regions.iter().zip(PREDEFINED.iter()) {
        assert_eq!(e.region, r.region);
        assert_eq!(e.parent.as_deref(), r.parent);
        assert_eq!(e.level, r.level.as_str());
        assert_eq!(e.geofabrik_path, r.geofabrik_path);
    }
}

#[test]
fn position_check_rejects_outside_set() {
    assert!(is_valid_position("germany", None, Level::Country));
    assert!(is_valid_position(
        "india/central-zone",
        Some("india"),
        Level::Subregion
    ));
    assert!(!is_valid_position("india", None, Level::Country));
    assert!(!is_valid_position("us", None, Level::Country));
    assert!(!is_valid_position("planet", None, Level::Country));
    assert!(!is_valid_position(
        "germany",
        Some("europe"),
        Level::Country
    ));
    assert!(!is_valid_position(
        "india/central-zone",
        Some("china"),
        Level::Subregion
    ));
    assert!(!is_valid_position(
        "india/central-zone",
        Some("india"),
        Level::Country
    ));
    assert!(!is_valid_position("Germany", None, Level::Country));
}

#[test]
fn every_region_id_fits_a_spel_string_seed() {
    // SPEL `seed_from_str` panics above 32 bytes; the registry PDA seeds on the region id.
    for r in &PREDEFINED {
        assert!(r.region.len() <= 32, "{} is {} bytes", r.region, r.region.len());
    }
    assert_eq!(PREDEFINED.iter().map(|r| r.region.len()).max(), Some(32));
}
