//! Liveness probes used by status screens: they report a real answer or a typed error.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use osm_sdk::registry::rpc_last_block;
use osm_sdk::rest_storage::RestStorage;

/// Serves one HTTP response with `body`, returns the base URL.
fn serve_once(body: &'static str) -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    std::thread::spawn(move || {
        if let Ok((mut s, _)) = l.accept() {
            // Read the whole request (headers and any body) before answering.
            let mut req = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let n = s.read(&mut buf).unwrap_or(0);
                req.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&req).to_string();
                if let Some(end) = text.find("\r\n\r\n") {
                    let len = text
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                        })
                        .unwrap_or(0);
                    if req.len() >= end + 4 + len {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            let _ = write!(
                s,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        }
    });
    format!("http://{addr}")
}

#[test]
fn rpc_probe_reads_the_block_id() {
    let rpc = serve_once(r#"{"jsonrpc":"2.0","id":1,"result":85383}"#);
    assert_eq!(rpc_last_block(&rpc).unwrap(), 85383);
}

#[test]
fn rpc_probe_errors_are_typed() {
    let rpc = serve_once(r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601}}"#);
    assert_eq!(rpc_last_block(&rpc).unwrap_err().code(), "S100");
    // Nothing listens on port 9 (discard): unreachable, not a hang.
    assert_eq!(
        rpc_last_block("http://127.0.0.1:9").unwrap_err().code(),
        "S100"
    );
}

#[test]
fn storage_probe_reads_space() {
    let base = serve_once(
        r#"{"totalBlocks":17,"quotaMaxBytes":32212254720,"quotaUsedBytes":1048662,"quotaReservedBytes":0}"#,
    );
    let v = RestStorage::new(&base, Duration::from_secs(5))
        .space()
        .unwrap();
    assert_eq!(v["quotaMaxBytes"], 32212254720u64);
}

#[test]
fn storage_probe_rejects_other_shapes() {
    let base = serve_once(r#"{"hello":"world"}"#);
    assert!(RestStorage::new(&base, Duration::from_secs(5))
        .space()
        .is_err());
    assert!(
        RestStorage::new("http://127.0.0.1:9", Duration::from_secs(5))
            .space()
            .is_err()
    );
}
