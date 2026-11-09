//! Geofabrik client against a scripted server, and the offline fallback (P-R4, H9, H24).

mod common;

use common::serve;
use osm_core::lookup;
use osm_sdk::geofabrik::{discover, Geofabrik, Source};
use osm_sdk::hash::to_hex;

const FIXTURE: &[u8] = include_bytes!("fixtures/geofabrik-index-subset.json");

fn redirect(loc: &str) -> common::Reply {
    (
        "307 Temporary Redirect",
        vec![("Location", loc.to_string())],
        vec![],
    )
}

#[test]
fn latest_snapshot_reads_redirect_without_download() {
    let (base, rx) = serve(vec![redirect("{BASE}asia/indonesia-261108.osm.pbf")]);
    let g = Geofabrik::with_base(&base);
    assert_eq!(
        g.latest_snapshot(lookup("indonesia").unwrap()).unwrap(),
        261108
    );
    assert_eq!(
        rx.recv().unwrap().0,
        "HEAD /asia/indonesia-latest.osm.pbf HTTP/1.1"
    );
}

#[test]
fn path_only_redirect_resolves_against_geofabrik() {
    let (base, _rx) = serve(vec![redirect("/asia/indonesia-261108.osm.pbf")]);
    let g = Geofabrik::with_base(&base);
    assert_eq!(
        g.latest_snapshot(lookup("indonesia").unwrap()).unwrap(),
        261108
    );
}

#[test]
fn redirect_to_another_region_or_host_is_refused() {
    let id = lookup("indonesia").unwrap();
    let (base, _rx) = serve(vec![redirect("{BASE}asia/japan-261108.osm.pbf")]);
    assert_eq!(
        Geofabrik::with_base(&base)
            .latest_snapshot(id)
            .unwrap_err()
            .code(),
        "S160"
    );
    let (base, _rx) = serve(vec![redirect(
        "https://evil.example/asia/indonesia-261108.osm.pbf",
    )]);
    assert_eq!(
        Geofabrik::with_base(&base)
            .latest_snapshot(id)
            .unwrap_err()
            .code(),
        "S160"
    );
    let (base, _rx) = serve(vec![("200 OK", vec![], vec![])]);
    assert_eq!(
        Geofabrik::with_base(&base)
            .latest_snapshot(id)
            .unwrap_err()
            .code(),
        "S160"
    );
}

#[test]
fn snapshot_md5_and_gone_snapshot() {
    let body = b"ade51ccd0faa1702cbeab207f927e0a8  indonesia-261001.osm.pbf\n".to_vec();
    let (base, rx) = serve(vec![
        ("200 OK", vec![], body),
        ("404 Not Found", vec![], vec![]),
    ]);
    let g = Geofabrik::with_base(&base);
    let id = lookup("indonesia").unwrap();
    assert_eq!(
        to_hex(&g.snapshot_md5(id, 261001).unwrap()),
        "ade51ccd0faa1702cbeab207f927e0a8"
    );
    assert_eq!(
        rx.recv().unwrap().0,
        "GET /asia/indonesia-261001.osm.pbf.md5 HTTP/1.1"
    );
    let e = g.snapshot_md5(id, 240601).unwrap_err();
    assert_eq!(e.code(), "S122");
    assert!(
        e.to_string().contains("indonesia-240601.osm.pbf.md5"),
        "{e}"
    );
}

#[test]
fn subregion_md5_uses_leaf_file_name() {
    let body = b"0123456789abcdef0123456789abcdef  central-zone-261101.osm.pbf\n".to_vec();
    let (base, rx) = serve(vec![("200 OK", vec![], body)]);
    let g = Geofabrik::with_base(&base);
    g.snapshot_md5(lookup("india/central-zone").unwrap(), 261101)
        .unwrap();
    assert_eq!(
        rx.recv().unwrap().0,
        "GET /asia/india/central-zone-261101.osm.pbf.md5 HTTP/1.1"
    );
}

#[test]
fn discover_uses_index_when_reachable() {
    let (base, _rx) = serve(vec![("200 OK", vec![], FIXTURE.to_vec())]);
    let (d, src) = discover(Some(&Geofabrik::with_base(&base)));
    assert_eq!(src, Source::Index);
    assert_eq!(d.regions.len(), 72);
}

#[test]
fn discover_falls_back_when_disabled_unreachable_or_throttled() {
    let (d, src) = discover(None);
    assert_eq!((d.regions.len(), src), (72, Source::OfflineDisabled));

    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let (d, src) = discover(Some(&Geofabrik::with_base(&format!(
        "http://127.0.0.1:{port}/"
    ))));
    assert_eq!(d.regions.len(), 72);
    assert!(
        matches!(src, Source::OfflineUnreachable(ref m) if m.starts_with("S120")),
        "{src:?}"
    );

    let (base, _rx) = serve(vec![("429 Too Many Requests", vec![], vec![])]);
    let (_, src) = discover(Some(&Geofabrik::with_base(&base)));
    assert!(
        matches!(src, Source::OfflineUnreachable(ref m) if m.contains("HTTP 429")),
        "{src:?}"
    );
}
