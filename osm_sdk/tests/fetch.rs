//! P-F3 / SC-C.6: storage first, newest verified entry wins, Geofabrik only when unhosted.

mod common;

use osm_sdk::fetch::{fetch_region, Source};
use osm_sdk::registry::{Registry, RegistrySource};
use osm_sdk::storage::{StorageBackend, StorageFault};
use osm_sdk::SdkError;
use std::collections::HashMap;
use std::path::Path;

const PROGRAM: &str = "G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ";

/// Region account bytes built the way the program writes them (layout 1, borsh).
fn account(region: &str, entries: &[(&str, [u8; 16], u64)]) -> Vec<u8> {
    let mut b = vec![1u8];
    let s = |b: &mut Vec<u8>, v: &str| {
        b.extend((v.len() as u32).to_le_bytes());
        b.extend(v.as_bytes());
    };
    let def = osm_core::lookup(region).unwrap();
    s(&mut b, region);
    s(&mut b, def.parent.unwrap_or(""));
    b.push(u8::from(def.level == osm_core::Level::Subregion));
    b.extend((entries.len() as u32).to_le_bytes());
    for (cid, md5, ts) in entries {
        s(&mut b, cid);
        s(
            &mut b,
            &format!(
                "https://download.geofabrik.de/{}-261101.osm.pbf",
                def.geofabrik_path
            ),
        );
        b.extend(md5);
        b.extend(261101u32.to_le_bytes());
        b.extend(10u64.to_le_bytes());
        b.push(1);
        b.extend(ts.to_le_bytes());
        b.extend([7u8; 32]);
        b.extend(1u32.to_le_bytes());
        b.extend([7u8; 32]);
    }
    b
}

struct Map(HashMap<String, Vec<u8>>);
impl RegistrySource for Map {
    fn fetch(&self, a: &str) -> Result<Option<Vec<u8>>, SdkError> {
        Ok(self.0.get(a).cloned())
    }
}

struct Store(HashMap<String, Vec<u8>>);
impl StorageBackend for Store {
    fn upload_file(&mut self, _: &Path) -> Result<String, StorageFault> {
        unreachable!()
    }
    fn download_to_file(&mut self, cid: &str, dest: &Path) -> Result<(), StorageFault> {
        let b = self
            .0
            .get(cid)
            .ok_or_else(|| StorageFault::Permanent(format!("{cid} not found")))?;
        std::fs::write(dest, b).unwrap();
        Ok(())
    }
    fn exists(&mut self, _: &str) -> Result<bool, StorageFault> {
        unreachable!()
    }
}

fn md5(b: &[u8]) -> [u8; 16] {
    use md5::{Digest, Md5};
    Md5::digest(b).into()
}

const GOOD: &str = "zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb";
const JUNK: &str = "zDvZRwzm3VHNvgC5Q2UWDxYHGYmMN2mcY3LniFRHvkVUYNMqiZbH";

fn registry(region: &str, bytes: Vec<u8>) -> Registry<Map> {
    let acct = osm_sdk::registry::region_account_id(PROGRAM, region).unwrap();
    Registry {
        source: Map(HashMap::from([(acct, bytes)])),
        program: PROGRAM.into(),
    }
}

#[test]
fn newer_junk_entry_cannot_hide_the_good_one() {
    let good = b"real map bytes".to_vec();
    // The junk entry is newer and claims the good checksum, but its CID serves other bytes.
    let reg = registry(
        "kenya",
        account("kenya", &[(GOOD, md5(&good), 1), (JUNK, md5(&good), 2)]),
    );
    let mut store = Store(HashMap::from([
        (GOOD.into(), good.clone()),
        (JUNK.into(), b"junk".to_vec()),
    ]));
    let dir = tempfile::tempdir().unwrap();
    let f = fetch_region(&reg, &mut store, None, "kenya", dir.path()).unwrap();
    assert_eq!(f.source, Source::Storage { cid: GOOD.into() });
    assert_eq!(std::fs::read(&f.path).unwrap(), good);
    assert_eq!(f.skipped.len(), 1);
    assert!(f.skipped[0].1.contains("MD5"), "{:?}", f.skipped);
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        1,
        "the junk download was deleted"
    );
}

#[test]
fn hosted_but_nothing_verifies_is_typed_and_never_falls_back() {
    let reg = registry("kenya", account("kenya", &[(JUNK, [1; 16], 1)]));
    let mut store = Store(HashMap::from([(JUNK.into(), b"junk".to_vec())]));
    let dir = tempfile::tempdir().unwrap();
    let e = fetch_region(&reg, &mut store, None, "kenya", dir.path()).unwrap_err();
    assert_eq!(e.code(), "S141");
}

#[test]
fn unhosted_without_geofabrik_is_typed() {
    let reg = Registry {
        source: Map(HashMap::new()),
        program: PROGRAM.into(),
    };
    let mut store = Store(HashMap::new());
    let dir = tempfile::tempdir().unwrap();
    let e = fetch_region(&reg, &mut store, None, "kenya", dir.path()).unwrap_err();
    assert_eq!(e.code(), "S140");
    assert!(e.to_string().contains("not hosted"));
}

#[test]
fn unhosted_falls_back_to_geofabrik_when_allowed() {
    use md5::{Digest, Md5};
    let body = b"geofabrik bytes".to_vec();
    let hex: String = Md5::digest(&body)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect();
    let (base, _rx) = common::serve(vec![
        (
            "307 Temporary Redirect",
            vec![("Location", "{BASE}africa/kenya-261108.osm.pbf".into())],
            vec![],
        ),
        (
            "200 OK",
            vec![],
            format!("{hex}  kenya-261101.osm.pbf\n").into_bytes(),
        ),
        ("200 OK", vec![], body.clone()),
    ]);
    let g = osm_sdk::geofabrik::Geofabrik::with_base(&base);
    let reg = Registry {
        source: Map(HashMap::new()),
        program: PROGRAM.into(),
    };
    let dir = tempfile::tempdir().unwrap();
    let f = fetch_region(
        &reg,
        &mut Store(HashMap::new()),
        Some(&g),
        "kenya",
        dir.path(),
    )
    .unwrap();
    assert_eq!(
        f.source,
        Source::Geofabrik {
            url: "https://download.geofabrik.de/africa/kenya-261101.osm.pbf".into()
        }
    );
    assert_eq!(std::fs::read(&f.path).unwrap(), body);
}
