//! H38 probe helper: rewrites one JSON file forever with `datadir::atomic_write`, alternating
//! between two valid 4 MiB documents. `probes/p10_atomic_kill.sh` kills it at random moments.
use osm_sdk::datadir::atomic_write;

fn main() {
    let path = std::env::args().nth(1).expect("usage: atomic_writer <file>");
    let doc = |c: char| format!("{{\"fill\":\"{}\"}}", c.to_string().repeat(4 << 20));
    let (a, b) = (doc('a'), doc('b'));
    let valid = |bytes: &[u8]| serde_json::from_slice::<serde_json::Value>(bytes).is_ok();
    loop {
        atomic_write(path.as_ref(), a.as_bytes(), valid).expect("write a");
        atomic_write(path.as_ref(), b.as_bytes(), valid).expect("write b");
    }
}
