//! CLI behaviour that needs no network.

use std::process::Command;

fn osm(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_osm"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn regions_lists_72_in_text_and_json() {
    let out = osm(&["regions"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 72);
    let out = osm(&["regions", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 72);
}

#[test]
fn discover_offline_works_without_geofabrik() {
    let out = osm(&["discover", "--no-geofabrik", "--json"]);
    // (no --rpc: hosted fields are false/null, never guessed)
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["regions"].as_array().unwrap().len(), 72);
    assert_eq!(v["source"], "offline (Geofabrik disabled)");
}

#[test]
fn unknown_region_is_invalid_input_exit_2() {
    let out = osm(&["snapshot", "planet", "--json"]);
    assert_eq!(out.status.code(), Some(2));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        (v["ok"].as_bool(), v["code"].as_str()),
        (Some(false), Some("S160"))
    );
}

#[test]
fn bad_snapshot_date_rejected_before_network() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("x.osm.pbf");
    std::fs::write(&f, b"x").unwrap();
    let out = osm(&[
        "verify-file",
        f.to_str().unwrap(),
        "--region",
        "indonesia",
        "--snapshot",
        "261301",
    ]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn no_geofabrik_refuses_commands_that_need_it() {
    let out = osm(&["snapshot", "indonesia", "--no-geofabrik"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("needs Geofabrik"));
}

#[test]
fn lookup_without_selector_is_invalid_input() {
    let out = osm(&[
        "lookup",
        "--rpc",
        "http://127.0.0.1:1",
        "--program",
        "G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ",
    ]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn lookup_with_unreachable_rpc_exits_3() {
    let out = osm(&[
        "lookup",
        "--region",
        "egypt",
        "--rpc",
        "http://127.0.0.1:1",
        "--program",
        "G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ",
        "--json",
    ]);
    assert_eq!(out.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["code"], "S100");
}
