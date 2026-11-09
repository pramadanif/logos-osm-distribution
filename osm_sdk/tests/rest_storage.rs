//! RestStorage against a scripted local HTTP server. Checks request shape and error classes; the
//! real node is exercised by the e2e job (SC-E.1), not here.

use osm_sdk::rest_storage::RestStorage;
use osm_sdk::storage::{StorageBackend, StorageFault};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

const CID: &str = "zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb"; // real v0.5.3 upload, 2026-11-09

type Seen = mpsc::Receiver<(String, Vec<String>, Vec<u8>)>;

/// Serves one request with `status` and `body`; sends back the request line, headers and body.
fn serve_once(status: &str, body: Vec<u8>) -> (String, Seen) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}/api/storage/v1", l.local_addr().unwrap());
    let status = status.to_string();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let (s, _) = l.accept().unwrap();
        let mut r = BufReader::new(s.try_clone().unwrap());
        let mut line = String::new();
        r.read_line(&mut line).unwrap();
        let mut headers = Vec::new();
        let mut len = 0usize;
        loop {
            let mut h = String::new();
            r.read_line(&mut h).unwrap();
            let h = h.trim_end().to_string();
            if h.is_empty() {
                break;
            }
            if let Some(v) = h.to_ascii_lowercase().strip_prefix("content-length: ") {
                len = v.parse().unwrap();
            }
            headers.push(h);
        }
        let mut req_body = vec![0; len];
        r.read_exact(&mut req_body).unwrap();
        let mut s = s;
        write!(
            s,
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        s.write_all(&body).unwrap();
        tx.send((line.trim_end().to_string(), headers, req_body))
            .unwrap();
    });
    (base, rx)
}

fn client(base: &str) -> RestStorage {
    RestStorage::new(base, Duration::from_secs(5))
}

#[test]
fn upload_streams_file_and_returns_cid() {
    let (base, rx) = serve_once("200 OK", format!("{CID}\n").into_bytes());
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("indonesia-261001.osm.pbf");
    std::fs::write(&f, b"pbf bytes").unwrap();
    assert_eq!(client(&base).upload_file(&f).unwrap(), CID);
    let (line, headers, body) = rx.recv().unwrap();
    assert_eq!(line, "POST /api/storage/v1/data HTTP/1.1");
    assert!(
        headers
            .iter()
            .any(|h| h == "content-disposition: attachment; filename=\"indonesia-261001.osm.pbf\""),
        "{headers:?}"
    );
    assert_eq!(body, b"pbf bytes");
}

#[test]
fn status_codes_are_classified() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("a.osm.pbf");
    std::fs::write(&f, b"x").unwrap();
    let (base, _rx) = serve_once("503 Service Unavailable", vec![]);
    assert!(
        matches!(client(&base).upload_file(&f), Err(StorageFault::Transient(m)) if m == "HTTP 503")
    );
    let (base, _rx) = serve_once("422 Unprocessable Entity", vec![]);
    assert!(
        matches!(client(&base).upload_file(&f), Err(StorageFault::Permanent(m)) if m == "HTTP 422")
    );
}

#[test]
fn unreachable_node_is_transient() {
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("a.osm.pbf");
    std::fs::write(&f, b"x").unwrap();
    let r = client(&format!("http://127.0.0.1:{port}/api/storage/v1")).upload_file(&f);
    assert!(matches!(r, Err(StorageFault::Transient(_))), "{r:?}");
}

#[test]
fn download_is_atomic_and_404_is_permanent() {
    let dir = tempfile::tempdir().unwrap();
    let dest = dir.path().join("out.osm.pbf");
    let (base, rx) = serve_once("200 OK", b"map data".to_vec());
    client(&base).download_to_file(CID, &dest).unwrap();
    assert_eq!(std::fs::read(&dest).unwrap(), b"map data");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&dest).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    assert_eq!(
        rx.recv().unwrap().0,
        format!("GET /api/storage/v1/data/{CID}/network/stream HTTP/1.1")
    );
    assert!(!dir.path().join("out.part").exists());

    let (base, _rx) = serve_once("404 Not Found", vec![]);
    let r = client(&base).download_to_file(CID, &dir.path().join("x.osm.pbf"));
    assert!(matches!(r, Err(StorageFault::Permanent(_))));
}

#[test]
fn exists_parses_json_and_rejects_bad_cid_before_any_request() {
    let (base, _rx) = serve_once("200 OK", br#"{"has":true}"#.to_vec());
    assert!(client(&base).exists(CID).unwrap());
    // Shape a real v0.5.3 node returns (captured 2026-11-09).
    let (base, _rx) = serve_once("200 OK", format!(r#"{{"{CID}":false}}"#).into_bytes());
    assert!(!client(&base).exists(CID).unwrap());
    let (base, _rx) = serve_once("200 OK", br#"{"other":true}"#.to_vec());
    assert!(matches!(
        client(&base).exists(CID),
        Err(StorageFault::Permanent(_))
    ));
    let (base, _rx) = serve_once("200 OK", b"<html>".to_vec());
    assert!(matches!(
        client(&base).exists(CID),
        Err(StorageFault::Permanent(_))
    ));
    let r = client("http://127.0.0.1:9/api/storage/v1").exists("../../etc/passwd");
    assert!(matches!(r, Err(StorageFault::Permanent(m)) if m.starts_with("invalid CID")));
}
