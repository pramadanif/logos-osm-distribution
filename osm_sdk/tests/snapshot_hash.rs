//! H24 snapshot handling and P-F5 / P-R2 import checks.

use osm_core::lookup;
use osm_sdk::hash::{md5_file, to_hex, verify_import};
use osm_sdk::snapshot::*;
use std::io::Write;

#[test]
fn urls_and_redirects() {
    let id = lookup("indonesia").unwrap();
    assert_eq!(
        latest_url(id),
        "https://download.geofabrik.de/asia/indonesia-latest.osm.pbf"
    );
    assert_eq!(
        snapshot_url(id, 261001).unwrap(),
        "https://download.geofabrik.de/asia/indonesia-261001.osm.pbf"
    );
    assert!(snapshot_url(id, 261301).is_err());
    assert!(snapshot_url(id, 1_000_000).is_err());
    assert_eq!(first_of_month(261108), 261101);
    // Location value observed on 2026-11-09 (docs/status/geofabrik.md).
    let loc = "https://download.geofabrik.de/asia/indonesia-261108.osm.pbf";
    assert_eq!(snapshot_from_location(id, loc).unwrap(), 261108);
    assert!(snapshot_from_location(
        id,
        "https://download.geofabrik.de/asia/japan-261108.osm.pbf"
    )
    .is_err());
    assert!(
        snapshot_from_location(id, "https://evil.example/asia/indonesia-261108.osm.pbf").is_err()
    );
}

#[test]
fn md5_file_format() {
    // Body of indonesia-261001.osm.pbf.md5 as served on 2026-11-09.
    let real = "ade51ccd0faa1702cbeab207f927e0a8  indonesia-261001.osm.pbf\n";
    let d = parse_md5_file(real, "indonesia-261001.osm.pbf").unwrap();
    assert_eq!(to_hex(&d), "ade51ccd0faa1702cbeab207f927e0a8");
    assert!(
        parse_md5_file(real, "indonesia-261108.osm.pbf").is_err(),
        "md5 for another snapshot"
    );
    assert!(parse_md5_file("", "x").is_err());
    assert!(parse_md5_file("<html>404</html>", "x").is_err());
    assert_eq!(
        file_name("https://download.geofabrik.de/asia/indonesia-261001.osm.pbf"),
        "indonesia-261001.osm.pbf"
    );
}

fn pbf_like(bytes: &[u8]) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(&[0, 0, 0, 14, 0x0a, 0x09]).unwrap();
    f.write_all(b"OSMHeader").unwrap();
    f.write_all(bytes).unwrap();
    f
}

#[test]
fn md5_streams_and_matches_known_vector() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(b"The quick brown fox jumps over the lazy dog")
        .unwrap();
    let d = md5_file(f.path(), |_| {}).unwrap();
    assert_eq!(
        to_hex(&d),
        "9e107d9d372bb6826bd81d3542a419d6",
        "RFC 1321 style vector"
    );
    // 3 MiB crosses the 1 MiB buffer; progress must be monotonic and end at the size.
    let big = pbf_like(&vec![7u8; 3 << 20]);
    let mut seen = Vec::new();
    md5_file(big.path(), |n| seen.push(n)).unwrap();
    assert!(seen.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(
        *seen.last().unwrap(),
        std::fs::metadata(big.path()).unwrap().len()
    );
}

#[test]
fn import_checks_fail_gracefully() {
    let f = pbf_like(b"payload");
    let good = md5_file(f.path(), |_| {}).unwrap();
    verify_import(f.path(), &good, "indonesia-261001").unwrap();

    let e = verify_import(f.path(), &[0u8; 16], "indonesia-261001").unwrap_err();
    assert_eq!(e.code(), "S130");
    let msg = e.to_string();
    assert!(
        msg.contains(&to_hex(&good)) && msg.contains("indonesia-261001"),
        "{msg}"
    );

    let mut not_pbf = tempfile::NamedTempFile::new().unwrap();
    not_pbf
        .write_all(b"<html>Too many requests</html>")
        .unwrap();
    assert_eq!(
        verify_import(not_pbf.path(), &good, "x")
            .unwrap_err()
            .code(),
        "S132"
    );

    let gone = std::path::Path::new("/nonexistent/lp18/file.osm.pbf");
    assert_eq!(verify_import(gone, &good, "x").unwrap_err().code(), "I/O");
}
