//! H37 / E87: arbitrary bytes into the registry decoder. It must never panic, and anything it
//! accepts must respect the program's bounds.
#![no_main]
use libfuzzer_sys::fuzz_target;
use osm_core::entry::{MAX_HISTORY, MAX_HOSTS};

fuzz_target!(|data: &[u8]| {
    // First byte picks the expected region so every region path is exercised.
    let Some((&pick, rest)) = data.split_first() else { return };
    let region = osm_core::PREDEFINED[pick as usize % osm_core::PREDEFINED.len()].region;
    if let Ok(acc) = osm_sdk::registry::decode_region(rest, region, "fuzz") {
        assert_eq!(acc.region, region);
        assert!(acc.history.len() <= MAX_HISTORY);
        assert!(acc.history.iter().all(|e| e.hosts.len() <= MAX_HOSTS));
    }
    let _ = osm_core::entry::validate_cid(&String::from_utf8_lossy(rest));
    let _ = osm_core::entry::parse_md5_hex(&String::from_utf8_lossy(rest));
});
