//! The encoder must produce the bytes the SPEL-generated client produced for the same call
//! (`spel --dry-run register …` on 2026-11-09: 194 bytes, prefix below).

use osm_sdk::instruction::{account_hex, Instruction};

#[test]
fn register_matches_spel_client_bytes() {
    let ix = Instruction::Register {
        region: "indonesia".into(),
        parent: "".into(),
        level: 0,
        cid: "zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb".into(),
        source_url: "https://download.geofabrik.de/asia/indonesia-261001.osm.pbf".into(),
        checksum: "ade51ccd0faa1702cbeab207f927e0a8".into(),
        version: 261001,
        size_bytes: 1741362950,
        timestamp: 1794197873,
    };
    let b = ix.to_bytes();
    assert_eq!(b.len(), 194, "spel reported 194 bytes for this call");
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    let spel_prefix = "0009000000696e646f6e657369610000000000340000007a44765a52777a6d44684a4856635179766b4642445874636b31626841434c546a7439347a367075484e336473666d35553777623b00000068747470733a2f2f646f776e6c6f61642e67656f66";
    assert!(hex.starts_with(spel_prefix), "{hex}");
}

#[test]
fn variant_indices_follow_the_idl() {
    let idl: serde_json::Value =
        serde_json::from_str(include_str!("../../idl/osm_registry.json")).unwrap();
    let names: Vec<_> = idl["instructions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, ["register", "register_batch", "unhost"]);
    assert_eq!(
        Instruction::Unhost {
            region: "kenya".into(),
            cid: "x".into()
        }
        .to_bytes()[0],
        2
    );
}

#[test]
fn account_hex_round_trip() {
    assert_eq!(
        account_hex("Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r").unwrap(),
        account_hex("CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r").unwrap()
    );
    assert_eq!(
        account_hex("CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r")
            .unwrap()
            .len(),
        64
    );
    assert!(account_hex("0OIl").is_err());
}
