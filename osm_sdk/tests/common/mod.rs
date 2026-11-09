//! Scripted one-shot HTTP server for client tests.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;

/// One scripted response: status line, extra headers, body.
pub type Reply = (&'static str, Vec<(&'static str, String)>, Vec<u8>);
pub type Seen = mpsc::Receiver<(String, Vec<String>, Vec<u8>)>;

/// Serves `responses` in order, one per connection; returns the base URL and what was received.
/// Each response is (status line, extra headers, body). `{BASE}` in a header value becomes the
/// server's own base URL.
pub fn serve(responses: Vec<Reply>) -> (String, Seen) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}/", l.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let me = base.clone();
    std::thread::spawn(move || {
        for (status, extra, body) in responses {
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
            let head = line.starts_with("HEAD ");
            let mut s = s;
            write!(
                s,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n",
                body.len()
            )
            .unwrap();
            for (k, v) in extra {
                let v = v.replace("{BASE}", &me);
                write!(s, "{k}: {v}\r\n").unwrap();
            }
            write!(s, "\r\n").unwrap();
            if !head {
                s.write_all(&body).unwrap();
            }
            tx.send((line.trim_end().to_string(), headers, req_body))
                .unwrap();
        }
    });
    (base, rx)
}
