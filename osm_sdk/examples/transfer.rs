//! H27 probe: upload a file to one Storage node and fetch it through another, with the SDK's
//! streaming paths, then compare MD5s. Run under `/usr/bin/time -l` (macOS) or `-v` (Linux) to
//! see that peak memory stays far below the file size.
//!
//!     cargo run --release -p osm_sdk --example transfer -- <file> <storage-a-api> <storage-b-api> <out-dir>
use osm_sdk::hash::{md5_file, to_hex};
use osm_sdk::rest_storage::RestStorage;
use osm_sdk::retry::{RetryPolicy, XorShift};
use osm_sdk::storage::{upload_with_retry, StorageBackend};
use std::time::{Duration, Instant};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let [_, file, node_a, node_b, out] = &a[..] else {
        eprintln!("usage: transfer <file> <storage-a-api> <storage-b-api> <out-dir>");
        std::process::exit(2);
    };
    let size = std::fs::metadata(file).expect("file").len();
    let t = Instant::now();
    let mut up = RestStorage::new(node_a, Duration::from_secs(600));
    let cid = upload_with_retry(&mut up, file.as_ref(), &RetryPolicy::default(), &mut XorShift::seeded(), std::thread::sleep).expect("upload");
    let up_s = t.elapsed().as_secs_f64();
    let dest = std::path::Path::new(out).join("transfer.bin");
    let t = Instant::now();
    RestStorage::new(node_b, Duration::from_secs(600)).download_to_file(&cid, &dest).expect("download");
    let down_s = t.elapsed().as_secs_f64();
    let (x, y) = (md5_file(file.as_ref(), |_| {}).unwrap(), md5_file(&dest, |_| {}).unwrap());
    println!("size={size} cid={cid} upload_s={up_s:.1} download_s={down_s:.1} md5_src={} md5_dst={} equal={}", to_hex(&x), to_hex(&y), x == y);
    let _ = std::fs::remove_file(&dest);
    if x != y {
        std::process::exit(1);
    }
}
