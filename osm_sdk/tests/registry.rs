//! Registry decoding and queries against bytes captured from a real v0.3.1 sequencer
//! (tools/capture-registry-fixture.sh, H32).

use osm_sdk::registry::*;
use osm_sdk::SdkError;
use std::collections::HashMap;

const PROGRAM: &str = "G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ";

fn fixture() -> HashMap<String, (String, Vec<u8>)> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/registry-v031.json")).unwrap();
    assert_eq!(v["program"], PROGRAM);
    v["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            let bytes = a["bytes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|b| b.as_u64().unwrap() as u8)
                .collect();
            (
                a["region"].as_str().unwrap().to_string(),
                (a["account"].as_str().unwrap().to_string(), bytes),
            )
        })
        .collect()
}

#[test]
fn pda_matches_spel_on_real_program() {
    // Vectors from `spel pda region_account --region <r>` for program G4s9EP4n… (2026-11-09).
    for (region, want) in [
        (
            "philippines",
            "4eJ3CzRntncYM4ijNGXvVhezPLG2xKS7y8TkxnmzAt3w",
        ),
        ("egypt", "4WEu78V7KG33AbunqeWL5nVXneCb6MukV6oSnwSkRF3V"),
        (
            "russia/northwestern-fed-district",
            "B4GgKkCgMnEKova2PQGzCuVPcZc52pboguyyc1KUeWNd",
        ),
        (
            "us/new-york",
            "Gs5RKfuzmcny6qAgwcBv5z4KSqKuEbFUZytYJxs25JGJ",
        ),
    ] {
        assert_eq!(
            region_account_id(PROGRAM, region).unwrap(),
            want,
            "{region}"
        );
    }
    assert_eq!(
        region_account_id(PROGRAM, "planet").unwrap_err().code(),
        "S160"
    );
    assert_eq!(
        region_account_id("not-base58-0OIl", "egypt")
            .unwrap_err()
            .code(),
        "S160"
    );
}

#[test]
fn decodes_real_accounts() {
    let f = fixture();
    let (acct, bytes) = &f["philippines"];
    let acc = decode_region(bytes, "philippines", acct).unwrap();
    assert_eq!((acc.layout, acc.level, acc.parent.as_str()), (1, 0, ""));
    assert_eq!(acc.history.len(), 2);
    assert_eq!(
        acc.history[0].registered_by_b58(),
        "CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r"
    );
    assert_eq!(
        acc.history[0].hosts_b58(),
        ["CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r"]
    );
    assert!(acc.history[0].timestamp > 1_793_491_200_000, "milliseconds");
    let (acct, bytes) = &f["egypt"];
    let acc = decode_region(bytes, "egypt", acct).unwrap();
    assert_eq!(
        acc.history[0].source_url,
        "https://download.geofabrik.de/africa/egypt-261101.osm.pbf"
    );
    assert_eq!(
        acc.history.len(),
        2,
        "batch entry plus a later single register"
    );
}

#[test]
fn decoder_rejects_tampered_bytes() {
    let f = fixture();
    let (acct, bytes) = &f["philippines"];
    // Wrong region for this account.
    assert_eq!(
        decode_region(bytes, "egypt", acct).unwrap_err().code(),
        "S170"
    );
    // Truncation anywhere is an error, never a panic.
    for cut in [0, 1, 5, bytes.len() / 2, bytes.len() - 1] {
        assert_eq!(
            decode_region(&bytes[..cut], "philippines", acct)
                .unwrap_err()
                .code(),
            "S170",
            "cut {cut}"
        );
    }
    // Trailing bytes.
    let mut extra = bytes.clone();
    extra.push(0);
    assert_eq!(
        decode_region(&extra, "philippines", acct)
            .unwrap_err()
            .code(),
        "S170"
    );
    // Huge length prefix on the history vector: offset after layout(1) + region(4+11) + parent(4) + level(1).
    let mut huge = bytes.clone();
    huge[21..25].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        decode_region(&huge, "philippines", acct)
            .unwrap_err()
            .code(),
        "S170"
    );
    // Unknown layout.
    let mut lay = bytes.clone();
    lay[0] = 2;
    assert_eq!(
        decode_region(&lay, "philippines", acct).unwrap_err().code(),
        "S170"
    );
    // Oversized blob.
    assert_eq!(
        decode_region(&vec![0u8; MAX_ACCOUNT_BYTES + 1], "philippines", acct)
            .unwrap_err()
            .code(),
        "S170"
    );
}

struct Map(HashMap<String, Vec<u8>>);
impl RegistrySource for Map {
    fn fetch(&self, account: &str) -> Result<Option<Vec<u8>>, SdkError> {
        Ok(self.0.get(account).cloned())
    }
}

#[test]
fn queries_by_region_parent_cid_sorted() {
    let f = fixture();
    let src = Map(f.values().map(|(a, b)| (a.clone(), b.clone())).collect());
    let reg = Registry {
        source: src,
        program: PROGRAM.to_string(),
    };
    assert_eq!(reg.region("philippines").unwrap().unwrap().history.len(), 2);
    assert!(reg.region("germany").unwrap().is_none());
    let (all, errs) = reg.all();
    assert!(errs.is_empty());
    assert_eq!(all.len(), 4);
    assert!(
        all.windows(2)
            .all(|w| w[0].entry.timestamp <= w[1].entry.timestamp),
        "P-R3 order"
    );
    let cid = "zDvZRwzm3VHNvgC5Q2UWDxYHGYmMN2mcY3LniFRHvkVUYNMqiZbH";
    let (hits, _) = reg.by_cid(cid);
    assert!(!hits.is_empty() && hits.iter().all(|h| h.entry.cid == cid));
    let (china, _) = reg.by_parent("china");
    assert!(china.is_empty(), "no china leaves in this fixture");
}

#[test]
fn resolve_picks_newest_hosted_entry() {
    let f = fixture();
    let src = Map(f.values().map(|(a, b)| (a.clone(), b.clone())).collect());
    let reg = Registry {
        source: src,
        program: PROGRAM.to_string(),
    };
    let r = reg.resolve("philippines").unwrap().unwrap();
    let acc = reg.region("philippines").unwrap().unwrap();
    let newest = acc.history.iter().max_by_key(|e| e.timestamp).unwrap();
    assert_eq!(r.cid, newest.cid);
    assert_eq!(r.version, 261101);
    assert!(reg.resolve("germany").unwrap().is_none());
    assert_eq!(reg.resolve("planet").unwrap_err().code(), "S160");
}
