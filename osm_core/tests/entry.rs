//! ADR 0002 validation: one accept case and one reject case per rule.

use osm_core::entry::*;
use osm_core::Level;

// Syntactic example only, not a captured Storage CID. Replaced by a CID from a real upload once
// SC-−1.3 runs (H32). MD5 and URL are real: `curl .../asia/indonesia-261001.osm.pbf.md5` on 2026-11-09.
const CID: &str = "zDvZRwzmAkhzDRPH5EW242gJBNZ2T7aoH2v1fVH66FxXL4kSbvyM";
const MD5: &str = "ade51ccd0faa1702cbeab207f927e0a8";
const URL: &str = "https://download.geofabrik.de/asia/indonesia-261001.osm.pbf";

fn ok() -> EntryInput<'static> {
    EntryInput {
        region: "indonesia",
        parent: None,
        level: Level::Country,
        cid: CID,
        source_url: URL,
        checksum_hex: MD5,
        version: 261001,
        size_bytes: 1_741_362_950,
        timestamp: MIN_TIMESTAMP + 1,
    }
}

fn err(e: EntryInput<'_>) -> EntryError {
    validate_entry(&e).expect_err("must be rejected")
}

#[test]
fn accepts_valid_country_and_subregion() {
    let v = validate_entry(&ok()).unwrap();
    assert_eq!(v.version, 261001);
    assert_eq!(v.checksum[0], 0xad);
    let sub = EntryInput {
        region: "india/central-zone",
        parent: Some("india"),
        level: Level::Subregion,
        source_url: "https://download.geofabrik.de/asia/india/central-zone-261101.osm.pbf",
        version: 261101,
        ..ok()
    };
    validate_entry(&sub).unwrap();
}

#[test]
fn rejects_region_and_position() {
    assert_eq!(
        err(EntryInput {
            region: "planet",
            ..ok()
        }),
        EntryError::RegionNotInSet
    );
    assert_eq!(
        err(EntryInput {
            region: "india",
            ..ok()
        }),
        EntryError::RegionNotInSet
    );
    assert_eq!(
        err(EntryInput {
            parent: Some("asia"),
            ..ok()
        }),
        EntryError::PositionMismatch
    );
    assert_eq!(
        err(EntryInput {
            level: Level::Subregion,
            ..ok()
        }),
        EntryError::PositionMismatch
    );
}

#[test]
fn rejects_bad_cids() {
    for cid in [
        "",
        "Qm",
        "QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5", // 45 chars
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi", // base32, not base58btc
        "zDvZRwzmAkhzDRPH5EW242gJBNZ2T7aoH2v1fVH66FxXL4kSbvy0", // '0' not base58
        "zDvZRwzmAkhzDRPH5EW242gJBNZ2T7aoH2v1fVH66FxXL4kSbvy<b>",
    ] {
        assert_eq!(validate_cid(cid), Err(EntryError::BadCid), "{cid}");
    }
    validate_cid("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N").unwrap();
    assert_eq!(
        validate_cid(&format!("z{}", "1".repeat(MAX_CID_LEN))),
        Err(EntryError::BadCid)
    );
}

#[test]
fn rejects_bad_source_urls() {
    for url in [
        "https://download.geofabrik.de/asia/indonesia-latest.osm.pbf",
        "http://download.geofabrik.de/asia/indonesia-261001.osm.pbf",
        "https://download.geofabrik.de.evil.example/asia/indonesia-261001.osm.pbf",
        "https://download.geofabrik.de/asia/malaysia-singapore-brunei-261001.osm.pbf",
        "https://download.geofabrik.de/asia/indonesia-261301.osm.pbf",
        "https://download.geofabrik.de/asia/indonesia-261000.osm.pbf",
        "https://download.geofabrik.de/asia/indonesia-26100.osm.pbf",
        "https://download.geofabrik.de/asia/indonesia-261001.osm.pbf?x=1",
        "https://download.geofabrik.de/asia/indonesia-2610٠1.osm.pbf",
    ] {
        assert_eq!(
            err(EntryInput {
                source_url: url,
                ..ok()
            }),
            EntryError::BadSourceUrl,
            "{url}"
        );
    }
    assert_eq!(
        err(EntryInput {
            version: 261108,
            ..ok()
        }),
        EntryError::VersionMismatch
    );
}

#[test]
fn rejects_bad_checksums_without_normalising() {
    for md5 in [
        "",
        "ade51ccd0faa1702cbeab207f927e0a",
        "ADE51CCD0FAA1702CBEAB207F927E0A8",
        " ade51ccd0faa1702cbeab207f927e0a8",
        "ade51ccd0faa1702cbeab207f927e0ag",
    ] {
        assert_eq!(
            err(EntryInput {
                checksum_hex: md5,
                ..ok()
            }),
            EntryError::BadChecksum,
            "{md5:?}"
        );
    }
}

#[test]
fn rejects_sizes_and_timestamps_at_the_edges() {
    assert_eq!(
        err(EntryInput {
            size_bytes: 0,
            ..ok()
        }),
        EntryError::SizeOutOfRange
    );
    assert_eq!(
        err(EntryInput {
            size_bytes: MAX_SIZE_BYTES + 1,
            ..ok()
        }),
        EntryError::SizeOutOfRange
    );
    assert_eq!(
        err(EntryInput {
            size_bytes: u64::MAX,
            ..ok()
        }),
        EntryError::SizeOutOfRange
    );
    validate_entry(&EntryInput {
        size_bytes: MAX_SIZE_BYTES,
        ..ok()
    })
    .unwrap();
    for ts in [0, 1, MIN_TIMESTAMP - 1] {
        assert_eq!(
            err(EntryInput {
                timestamp: ts,
                ..ok()
            }),
            EntryError::TimestampOutOfRange
        );
    }
}

#[test]
fn timestamp_window_is_symmetric_and_overflow_safe() {
    let now = MIN_TIMESTAMP + 10_000;
    assert!(timestamp_in_window(now, now));
    assert!(timestamp_in_window(now - TIMESTAMP_WINDOW_SECS, now));
    assert!(timestamp_in_window(now + TIMESTAMP_WINDOW_SECS, now));
    assert!(!timestamp_in_window(now - TIMESTAMP_WINDOW_SECS - 1, now));
    assert!(!timestamp_in_window(now + TIMESTAMP_WINDOW_SECS + 1, now));
    assert!(!timestamp_in_window(u64::MAX, now));
    assert!(!timestamp_in_window(now, u64::MAX));
    assert!(!timestamp_in_window(0, 0));
}

#[test]
fn error_codes_match_docs() {
    let doc = include_str!("../../docs/error-codes.md");
    for e in [
        EntryError::RegionNotInSet,
        EntryError::PositionMismatch,
        EntryError::BadCid,
        EntryError::BadSourceUrl,
        EntryError::VersionMismatch,
        EntryError::BadChecksum,
        EntryError::SizeOutOfRange,
        EntryError::TimestampOutOfRange,
    ] {
        let row = format!("| {} | `{:?}` |", e.code(), e);
        assert!(doc.contains(&row), "docs/error-codes.md lacks {row}");
    }
}
