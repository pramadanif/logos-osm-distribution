//! The C ABI as a C caller sees it: pointers in, owned strings out, errors as JSON.

use osm_ffi::*;
use std::ffi::{CStr, CString};

fn call(
    f: extern "C" fn(*const std::ffi::c_char) -> *mut std::ffi::c_char,
    args: &str,
) -> serde_json::Value {
    let a = CString::new(args).unwrap();
    let p = f(a.as_ptr());
    assert!(!p.is_null());
    let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string();
    unsafe { osm_free(p) };
    serde_json::from_str(&s).unwrap()
}

#[test]
fn version_and_offline_discovery() {
    let p = osm_version();
    let v: serde_json::Value =
        serde_json::from_str(unsafe { CStr::from_ptr(p) }.to_str().unwrap()).unwrap();
    unsafe { osm_free(p) };
    assert_eq!(v["api"], 1);
    let d = call(osm_discover, r#"{"geofabrik":false}"#);
    assert_eq!(d["ok"], true);
    assert_eq!(d["data"]["regions"].as_array().unwrap().len(), 72);
    assert_eq!(d["data"]["source"], "offline-disabled");
}

#[test]
fn bad_inputs_are_errors_not_crashes() {
    assert_eq!(call(osm_discover, "not json")["code"], "S160");
    assert_eq!(call(osm_discover, "[1,2]")["code"], "S160");
    assert_eq!(
        call(osm_resolve, r#"{"rpc":"http://127.0.0.1:1"}"#)["code"],
        "S160"
    );
    let r = call(
        osm_resolve,
        r#"{"rpc":"http://127.0.0.1:1","program":"G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ","region":"planet"}"#,
    );
    assert_eq!(r["code"], "S160");
    let r = call(
        osm_resolve,
        r#"{"rpc":"http://127.0.0.1:1","program":"G4s9EP4n9DbaD8HdfbyFYDTSVnHSnfsoP3SEPxVQACJ","region":"kenya"}"#,
    );
    assert_eq!(r["code"], "S100", "{r}");
    let big = format!(r#"{{"x":"{}"}}"#, "a".repeat(70 * 1024));
    assert_eq!(call(osm_discover, &big)["code"], "S160");
    // Null pointer.
    let p = osm_discover(std::ptr::null());
    let s = unsafe { CStr::from_ptr(p) }.to_str().unwrap().to_string();
    unsafe { osm_free(p) };
    assert!(s.contains("null pointer"));
    unsafe { osm_free(std::ptr::null_mut()) };
}

#[test]
fn header_lists_every_exported_function() {
    let h = include_str!("../include/osm_ffi.h");
    for f in [
        "osm_version",
        "osm_discover",
        "osm_resolve",
        "osm_fetch",
        "osm_hosted_status",
        "osm_check_updates",
        "osm_free",
    ] {
        assert!(h.contains(&format!("{f}(")), "{f} missing from osm_ffi.h");
    }
}
