//! P-A1 counting rules and per-entry statuses (H7, H24).

mod common;

use osm_sdk::coverage::{verify_coverage, EntryStatus};
use osm_sdk::geofabrik::Geofabrik;
use osm_sdk::registry::{region_account_id, Registry, RegistrySource};
use osm_sdk::storage::{StorageBackend, StorageFault};
use osm_sdk::SdkError;
use std::collections::HashMap;
use std::path::Path;

const PROGRAM: &str = "G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ";
const C1: &str = "zDvZRwzmDhJHVcQyvkFBDXtck1bhACLTjt94z6puHN3dsfm5U7wb";
const C2: &str = "zDvZRwzm3VHNvgC5Q2UWDxYHGYmMN2mcY3LniFRHvkVUYNMqiZbH";
const C3: &str = "zDvZRwzmA5LrY5Be279q4gpJqxk6v6XrS59rHkKakguRwwFPEMfH";

fn md5(b: &[u8]) -> [u8; 16] {
    use md5::{Digest, Md5};
    Md5::digest(b).into()
}
fn hex(d: [u8; 16]) -> String {
    d.iter().map(|x| format!("{x:02x}")).collect()
}

fn account(region: &str, entries: &[(&str, [u8; 16])]) -> Vec<u8> {
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
    for (i, (cid, sum)) in entries.iter().enumerate() {
        s(&mut b, cid);
        s(
            &mut b,
            &format!(
                "https://download.geofabrik.de/{}-261101.osm.pbf",
                def.geofabrik_path
            ),
        );
        b.extend(sum);
        b.extend(261101u32.to_le_bytes());
        b.extend(10u64.to_le_bytes());
        b.push(1);
        b.extend((i as u64 + 1).to_le_bytes());
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
            .ok_or_else(|| StorageFault::Permanent("404".into()))?;
        std::fs::write(dest, b).unwrap();
        Ok(())
    }
    fn exists(&mut self, _: &str) -> Result<bool, StorageFault> {
        unreachable!()
    }
}

#[test]
fn statuses_and_counting() {
    let good = b"good".to_vec();
    let other = b"other".to_vec();
    let reg = Registry {
        source: Map(HashMap::from([
            // kenya: verifies -> 1 region, country kenya
            (
                region_account_id(PROGRAM, "kenya").unwrap(),
                account("kenya", &[(C1, md5(&good))]),
            ),
            // india/central-zone: verifies -> country india via subregion
            (
                region_account_id(PROGRAM, "india/central-zone").unwrap(),
                account("india/central-zone", &[(C1, md5(&good))]),
            ),
            // egypt: bytes do not match registered checksum
            (
                region_account_id(PROGRAM, "egypt").unwrap(),
                account("egypt", &[(C2, md5(&good))]),
            ),
            // peru: registered checksum is right for the bytes, Geofabrik's published one differs
            (
                region_account_id(PROGRAM, "peru").unwrap(),
                account("peru", &[(C2, md5(&other))]),
            ),
            // chile: CID not retrievable
            (
                region_account_id(PROGRAM, "chile").unwrap(),
                account("chile", &[(C3, md5(&good))]),
            ),
        ])),
        program: PROGRAM.into(),
    };
    let mut store = Store(HashMap::from([
        (C1.into(), good.clone()),
        (C2.into(), other.clone()),
    ]));
    // Entries are verified in registry order (timestamp, then region); only entries whose bytes
    // match their registered checksum ask Geofabrik: india/central-zone, kenya, peru.
    let (base, _rx) = common::serve(vec![
        (
            "200 OK",
            vec![],
            format!("{}  central-zone-261101.osm.pbf\n", hex(md5(&good))).into_bytes(),
        ),
        (
            "200 OK",
            vec![],
            format!("{}  kenya-261101.osm.pbf\n", hex(md5(&good))).into_bytes(),
        ),
        (
            "200 OK",
            vec![],
            format!("{}  peru-261101.osm.pbf\n", hex(md5(&good))).into_bytes(),
        ),
    ]);
    let g = Geofabrik::with_base(&base);
    let dir = tempfile::tempdir().unwrap();
    let r = verify_coverage(&reg, &mut store, &g, dir.path(), false);
    let st = |region: &str| {
        r.entries
            .iter()
            .find(|e| e.region == region)
            .unwrap()
            .status
            .clone()
    };
    assert_eq!(st("kenya"), EntryStatus::Verified);
    assert_eq!(st("india/central-zone"), EntryStatus::Verified);
    assert!(matches!(st("egypt"), EntryStatus::BytesMismatch { .. }));
    assert!(matches!(
        st("peru"),
        EntryStatus::PublishedMd5Differs { .. }
    ));
    assert!(matches!(st("chile"), EntryStatus::NotRetrievable(_)));
    assert_eq!(r.verified_regions.len(), 2);
    assert_eq!(
        r.countries.iter().copied().collect::<Vec<_>>(),
        ["india", "kenya"]
    );
    assert!(!r.passes());
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        0,
        "downloads are deleted"
    );
}
