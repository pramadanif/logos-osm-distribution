//! P-F6 update check and hosted status, with the registry fixture captured from a real sequencer.

mod common;

use osm_sdk::geofabrik::Geofabrik;
use osm_sdk::registry::{Registry, RegistrySource};
use osm_sdk::status::{check_updates, hosted_status};
use osm_sdk::SdkError;
use std::collections::HashMap;

const PROGRAM: &str = "G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ";

struct Map(HashMap<String, Vec<u8>>);
impl RegistrySource for Map {
    fn fetch(&self, a: &str) -> Result<Option<Vec<u8>>, SdkError> {
        Ok(self.0.get(a).cloned())
    }
}

fn registry() -> Registry<Map> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/registry-v031.json")).unwrap();
    let m = v["accounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            (
                a["account"].as_str().unwrap().to_string(),
                a["bytes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|b| b.as_u64().unwrap() as u8)
                    .collect(),
            )
        })
        .collect();
    Registry {
        source: Map(m),
        program: PROGRAM.into(),
    }
}

#[test]
fn hosted_status_covers_all_72() {
    let (s, e) = hosted_status(&registry());
    assert!(e.is_empty());
    assert_eq!(s.len(), 72);
    let ph = s.iter().find(|x| x.region == "philippines").unwrap();
    assert_eq!((ph.version, ph.hosts), (Some(261101), 1));
    assert_eq!(
        s.iter().find(|x| x.region == "germany").unwrap().version,
        None
    );
}

#[test]
fn update_check_per_region() {
    let (base, _rx) = common::serve(vec![
        (
            "307 Temporary Redirect",
            vec![("Location", "{BASE}asia/philippines-261208.osm.pbf".into())],
            vec![],
        ),
        (
            "307 Temporary Redirect",
            vec![("Location", "{BASE}europe/germany-261108.osm.pbf".into())],
            vec![],
        ),
    ]);
    let g = Geofabrik::with_base(&base);
    let u = check_updates(&registry(), &g, &["philippines", "germany"]).unwrap();
    assert_eq!(
        (u[0].registered, u[0].available, u[0].update_available()),
        (Some(261101), 261201, true)
    );
    assert_eq!(
        (u[1].registered, u[1].available, u[1].update_available()),
        (None, 261101, true)
    );
    let (base, _rx) = common::serve(vec![(
        "307 Temporary Redirect",
        vec![("Location", "{BASE}asia/philippines-261115.osm.pbf".into())],
        vec![],
    )]);
    let u = check_updates(&registry(), &Geofabrik::with_base(&base), &["philippines"]).unwrap();
    assert!(!u[0].update_available(), "same month: no update");
}
