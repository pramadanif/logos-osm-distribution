//! Against two real Logos Storage nodes (SC-−1.3, H3). Runs only when both are configured:
//!
//!     OSM_STORAGE_A=http://127.0.0.1:8081/api/storage/v1 \
//!     OSM_STORAGE_B=http://127.0.0.1:8082/api/storage/v1 cargo test -p osm_sdk --test real_storage
//!
//! When the variables are set, an unreachable node fails the test; it is never skipped silently.

use osm_sdk::hash::md5_file;
use osm_sdk::rest_storage::RestStorage;
use osm_sdk::retry::{RetryPolicy, XorShift};
use osm_sdk::storage::{upload_with_retry, StorageBackend};
use std::io::Write;
use std::time::Duration;

#[test]
fn upload_on_a_fetch_on_b_over_the_network() {
    let (Ok(a), Ok(b)) = (
        std::env::var("OSM_STORAGE_A"),
        std::env::var("OSM_STORAGE_B"),
    ) else {
        eprintln!("OSM_STORAGE_A / OSM_STORAGE_B not set: real-node test not run");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("probe.bin");
    let mut f = std::fs::File::create(&src).unwrap();
    let mut x = 0x9E37_79B9_7F4A_7C15u64;
    for _ in 0..(3 << 20) / 8 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        f.write_all(&x.to_le_bytes()).unwrap();
    }
    drop(f);

    let mut node_a = RestStorage::new(&a, Duration::from_secs(60));
    let cid = upload_with_retry(
        &mut node_a,
        &src,
        &RetryPolicy::default(),
        &mut XorShift::seeded(),
        std::thread::sleep,
    )
    .expect("upload to node A");
    assert!(
        node_a.exists(&cid).unwrap(),
        "node A must hold what it accepted"
    );

    let mut node_b = RestStorage::new(&b, Duration::from_secs(60));
    let dest = dir.path().join("fetched.bin");
    node_b
        .download_to_file(&cid, &dest)
        .expect("node B fetches from the network");
    assert_eq!(
        md5_file(&dest, |_| {}).unwrap(),
        md5_file(&src, |_| {}).unwrap()
    );
    assert!(
        node_b.exists(&cid).unwrap(),
        "node B now stores the dataset"
    );
    println!("cid={cid}");
}
