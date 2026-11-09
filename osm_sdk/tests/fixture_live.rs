//! H32: the decoder against accounts captured from the sequencer the CI e2e job just ran
//! (tools/capture-registry-fixture.sh writes OSM_LIVE_FIXTURE). Without the variable the test
//! checks the committed fixture, so it is never empty; with OSM_REQUIRE_LIVE_FIXTURE set, a
//! missing variable fails.

use osm_sdk::registry::decode_region;

#[test]
fn every_captured_account_decodes_and_respects_bounds() {
    let path = match std::env::var("OSM_LIVE_FIXTURE") {
        Ok(p) => p,
        Err(_) => {
            assert!(
                std::env::var_os("OSM_REQUIRE_LIVE_FIXTURE").is_none(),
                "OSM_LIVE_FIXTURE not set"
            );
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/registry-v031.json"
            )
            .to_string()
        }
    };
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let accounts = v["accounts"].as_array().unwrap();
    assert!(!accounts.is_empty(), "{path} has no accounts");
    for a in accounts {
        let region = a["region"].as_str().unwrap();
        let bytes: Vec<u8> = a["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b.as_u64().unwrap() as u8)
            .collect();
        let acc = decode_region(&bytes, region, a["account"].as_str().unwrap())
            .unwrap_or_else(|e| panic!("{region}: {e}"));
        assert!(!acc.history.is_empty(), "{region}: empty history");
        for e in &acc.history {
            assert!(
                e.timestamp > 1_793_491_200_000,
                "{region}: timestamp not in ms"
            );
            assert!(
                !e.hosts.is_empty() || !e.hosted,
                "{region}: hosted without hosts"
            );
        }
    }
}
