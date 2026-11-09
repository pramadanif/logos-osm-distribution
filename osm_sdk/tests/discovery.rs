//! SC-C.1 and P-R4 against an index subset captured from Geofabrik (tools/capture-geofabrik-fixture.sh).

use osm_core::{lookup, Level};
use osm_sdk::discovery::{discover_from_index, discover_offline, MAX_NAME_BYTES};

const FIXTURE: &[u8] = include_bytes!("fixtures/geofabrik-index-subset.json");

#[test]
fn real_index_lists_all_72_and_nothing_else() {
    let d = discover_from_index(FIXTURE).unwrap();
    assert_eq!(d.regions.len(), 72, "missing: {:?}", d.missing);
    assert!(d.missing.is_empty());
    for bad in [
        "us",
        "india",
        "china",
        "russia",
        "georgia",
        "britain-and-ireland",
        "united-kingdom",
    ] {
        assert!(
            d.regions.iter().all(|r| r.def.region != bad),
            "{bad} leaked into discovery"
        );
    }
    let ga = d
        .regions
        .iter()
        .find(|r| r.def.region == "us/georgia")
        .unwrap();
    assert_eq!(ga.def.parent, Some("us"));
    assert_eq!(ga.def.level, Level::Subregion);
    let cz = d
        .regions
        .iter()
        .find(|r| r.def.region == "india/central-zone")
        .unwrap();
    assert_eq!(cz.name, "Central Zone");
}

#[test]
fn renamed_region_is_reported_missing_not_guessed() {
    let raw = String::from_utf8(FIXTURE.to_vec()).unwrap();
    let renamed = raw.replace(
        "asia/indonesia-latest.osm.pbf",
        "asia/indonesia-renamed-latest.osm.pbf",
    );
    let d = discover_from_index(renamed.as_bytes()).unwrap();
    assert_eq!(d.regions.len(), 71);
    assert_eq!(d.missing, vec![lookup("indonesia").unwrap()]);
}

#[test]
fn hostile_names_are_cleaned_and_bounded() {
    let json = format!(
        r#"{{"features":[
        {{"properties":{{"id":"germany","name":"Ger\u202Emany\u0007<b>x</b>","urls":{{"pbf":"https://download.geofabrik.de/europe/germany-latest.osm.pbf"}}}}}},
        {{"properties":{{"id":"japan","name":"{}","urls":{{"pbf":"https://download.geofabrik.de/asia/japan-latest.osm.pbf"}}}}}},
        {{"properties":{{"id":"russia/central-fed-district","name":"Центральный","urls":{{"pbf":"https://download.geofabrik.de/russia/central-fed-district-latest.osm.pbf"}}}}}},
        {{"properties":{{"id":"france","name":"\u202E\u0000","urls":{{"pbf":"https://download.geofabrik.de/europe/france-latest.osm.pbf"}}}}}}
        ]}}"#,
        "日".repeat(200)
    );
    let d = discover_from_index(json.as_bytes()).unwrap();
    let name = |id: &str| {
        d.regions
            .iter()
            .find(|r| r.def.region == id)
            .unwrap()
            .name
            .clone()
    };
    assert_eq!(
        name("germany"),
        "Germany<b>x</b>",
        "markup is kept as text; the UI renders plain text"
    );
    assert!(name("japan").len() <= MAX_NAME_BYTES && name("japan").starts_with('日'));
    assert_eq!(name("russia/central-fed-district"), "Центральный");
    assert_eq!(name("france"), "france");
    assert_eq!(d.missing.len(), 68);
}

#[test]
fn garbage_index_is_a_typed_error() {
    let e = discover_from_index(b"<html>rate limited</html>").unwrap_err();
    assert_eq!(e.code(), "S160");
}

#[test]
fn offline_discovery_needs_no_network() {
    let d = discover_offline();
    assert_eq!(d.regions.len(), 72);
}
