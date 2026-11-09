//! H38: modes and atomic writes.

use osm_sdk::datadir::{atomic_write, enforce_cache_cap, ensure_private_dir};

#[cfg(unix)]
#[test]
fn modes_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("a/b");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o775)).unwrap();
    ensure_private_dir(&dir).unwrap();
    assert_eq!(
        std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let f = dir.join("settings.json");
    atomic_write(&f, b"{}", |_| true).unwrap();
    assert_eq!(
        std::fs::metadata(&f).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn private_dir_is_created_on_every_platform() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("x/y");
    assert_eq!(ensure_private_dir(&dir).unwrap(), dir);
    assert!(dir.is_dir());
}

#[test]
fn invalid_content_never_replaces_good_file() {
    let root = tempfile::tempdir().unwrap();
    let f = root.path().join("settings.json");
    let is_json = |b: &[u8]| serde_json::from_slice::<serde_json::Value>(b).is_ok();
    atomic_write(&f, br#"{"ok":true}"#, is_json).unwrap();
    let e = atomic_write(&f, b"{trunc", is_json).unwrap_err();
    assert_eq!(e.code(), "S160");
    assert_eq!(std::fs::read(&f).unwrap(), br#"{"ok":true}"#);
    let leftovers: Vec<_> = std::fs::read_dir(root.path()).unwrap().collect();
    assert_eq!(leftovers.len(), 1, "no temp files left behind");
}

#[test]
fn cache_cap_evicts_oldest_but_never_the_kept_file() {
    let d = tempfile::tempdir().unwrap();
    let mk = |name: &str, n: usize, age_s: u64| {
        let p = d.path().join(name);
        std::fs::write(&p, vec![0u8; n]).unwrap();
        let t = std::time::SystemTime::now() - std::time::Duration::from_secs(age_s);
        std::fs::File::options()
            .write(true)
            .open(&p)
            .unwrap()
            .set_modified(t)
            .unwrap();
        p
    };
    let old = mk("a-261001.osm.pbf", 400, 300);
    let mid = mk("b-261001.osm.pbf", 400, 200);
    let new = mk("c-261101.osm.pbf", 400, 100);
    let part = mk("d-261101.osm.pbf.part", 400, 400);
    // Cap 500: the two oldest PBFs go, the newest stays, .part is untouched.
    let gone = enforce_cache_cap(d.path(), 500, &new).unwrap();
    assert_eq!(gone, vec![old.clone(), mid.clone()]);
    assert!(new.exists() && part.exists());
    // The kept file survives even when it alone exceeds the cap.
    assert!(enforce_cache_cap(d.path(), 10, &new).unwrap().is_empty());
    assert!(new.exists());
}
