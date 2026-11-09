//! H37 / E87: the registry decoder must never panic or accept out-of-bound data, whatever the
//! account bytes are. Deterministic mutations of accounts captured from a real sequencer, so the
//! run is reproducible and fast enough for every CI push. `cargo fuzz` covers open-ended runs.

use osm_core::entry::{MAX_HISTORY, MAX_HOSTS};
use osm_sdk::registry::decode_region;

fn seeds() -> Vec<(String, Vec<u8>)> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/registry-v031.json")).unwrap();
    v["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            (
                a["region"].as_str().unwrap().to_string(),
                a["bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|b| b.as_u64().unwrap() as u8)
                    .collect(),
            )
        })
        .collect()
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

#[test]
fn two_hundred_thousand_mutations_never_panic_or_exceed_bounds() {
    let seeds = seeds();
    let mut rng = Rng(0x5EED_1234_ABCD_0001);
    let mut accepted = 0u32;
    for i in 0..200_000u32 {
        let (region, base) = &seeds[i as usize % seeds.len()];
        let mut b = base.clone();
        for _ in 0..1 + rng.below(4) {
            match rng.below(5) {
                0 => {
                    let j = rng.below(b.len());
                    b[j] ^= 1 << rng.below(8);
                }
                1 => {
                    let j = rng.below(b.len());
                    b[j] = rng.next() as u8;
                }
                2 if b.len() > 4 => {
                    // Overwrite a 4-byte window with an extreme length prefix.
                    let j = rng.below(b.len() - 4);
                    let v = [0u32, 1, 8, 9, 16, 17, 0xFFFF, u32::MAX][rng.below(8)];
                    b[j..j + 4].copy_from_slice(&v.to_le_bytes());
                }
                3 => b.truncate(rng.below(b.len() + 1)),
                _ => {
                    let extra = rng.below(64);
                    b.extend((0..extra).map(|_| rng.next() as u8));
                }
            }
            if b.is_empty() {
                break;
            }
        }
        if let Ok(acc) = decode_region(&b, region, "mutation") {
            accepted += 1;
            assert!(acc.history.len() <= MAX_HISTORY);
            assert!(acc.history.iter().all(|e| e.hosts.len() <= MAX_HOSTS));
            assert_eq!(&acc.region, region);
        }
    }
    // Some single-bit flips inside free-form fields (e.g. a byte of a hash) still decode, which
    // is fine: the MD5 check at download time rejects wrong content.
    assert!(accepted < 200_000);
}
