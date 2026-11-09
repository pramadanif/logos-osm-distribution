//! C ABI over the consumer API of `osm_sdk` (see `include/osm_ffi.h`).
//!
//! JSON in, JSON out. Inputs are bounded (64 KiB) and validated; panics are caught and turned
//! into `{"ok":false,...}`; returned strings are owned by the caller until `osm_free`.

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::time::Duration;

use osm_sdk::geofabrik::{Geofabrik, Source as DiscoverySource};
use osm_sdk::hash::to_hex;
use osm_sdk::registry::{Registry, RpcRegistry};
use osm_sdk::rest_storage::RestStorage;
use osm_sdk::SdkError;
use serde_json::{json, Value};

/// Version of this C API. Bumped only on a breaking change.
pub const API_VERSION: u32 = 1;
const MAX_ARGS_BYTES: usize = 64 * 1024;

fn invalid(why: impl Into<String>) -> SdkError {
    SdkError::InvalidInput {
        field: "args",
        why: why.into(),
    }
}

fn parse_args(ptr: *const c_char) -> Result<Value, SdkError> {
    if ptr.is_null() {
        return Err(invalid("null pointer"));
    }
    // SAFETY: the caller passes a NUL-terminated string (header contract); we only read it.
    let bytes = unsafe { CStr::from_ptr(ptr) }.to_bytes();
    if bytes.len() > MAX_ARGS_BYTES {
        return Err(invalid(format!(
            "arguments larger than {MAX_ARGS_BYTES} bytes"
        )));
    }
    let v: Value = serde_json::from_slice(bytes).map_err(|e| invalid(e.to_string()))?;
    if !v.is_object() {
        return Err(invalid("arguments must be a JSON object"));
    }
    Ok(v)
}

pub(crate) fn str_arg<'a>(v: &'a Value, k: &'static str) -> Result<&'a str, SdkError> {
    v[k].as_str().ok_or(SdkError::InvalidInput {
        field: k,
        why: "missing or not a string".into(),
    })
}

fn out(s: String) -> *mut c_char {
    // Serialised JSON never contains a NUL byte (it would be escaped as \u0000).
    CString::new(s)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

fn respond(f: impl FnOnce() -> Result<Value, SdkError>) -> *mut c_char {
    let v = match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(data)) => json!({"ok": true, "data": data}),
        Ok(Err(e)) => json!({"ok": false, "code": e.code(), "error": e.to_string()}),
        Err(_) => {
            json!({"ok": false, "code": "INTERNAL", "error": "internal error (panic caught at the C boundary)"})
        }
    };
    out(v.to_string())
}

pub(crate) fn registry(v: &Value) -> Result<Registry<RpcRegistry>, SdkError> {
    let program = str_arg(v, "program")?;
    Ok(Registry {
        source: RpcRegistry::new(str_arg(v, "rpc")?, program)?,
        program: program.to_string(),
    })
}

pub(crate) fn geofabrik(v: &Value) -> Option<Geofabrik> {
    if v["geofabrik"].as_bool() == Some(false) {
        return None;
    }
    Some(match v["geofabrik_base"].as_str() {
        Some(b) => Geofabrik::with_base(b),
        None => Geofabrik::new(),
    })
}

/// The consumer API as Rust functions over JSON values. The C ABI below and the Basecamp
/// modules both call these, so there is one implementation.
pub mod api {
    use super::*;

    pub fn discover(v: &Value) -> Result<Value, SdkError> {
        let (d, src) = osm_sdk::geofabrik::discover(geofabrik(v).as_ref());
        let source = match src {
            DiscoverySource::Index => "geofabrik-index".to_string(),
            DiscoverySource::OfflineDisabled => "offline-disabled".to_string(),
            DiscoverySource::OfflineUnreachable(why) => format!("offline-unreachable: {why}"),
        };
        let regions: Vec<_> = d
                .regions
                .iter()
                .map(|r| json!({"region": r.def.region, "name": r.name, "parent": r.def.parent, "level": r.def.level.as_str()}))
                .collect();
        let missing: Vec<_> = d.missing.iter().map(|r| r.region).collect();
        Ok(json!({"source": source, "regions": regions, "missing": missing}))
    }

    pub fn resolve(v: &Value) -> Result<Value, SdkError> {
        let r = registry(v)?.resolve(str_arg(v, "region")?)?;
        Ok(match r {
            None => Value::Null,
            Some(r) => {
                json!({"region": r.region, "cid": r.cid, "checksum": to_hex(&r.checksum), "version": r.version,
                    "source_url": r.source_url, "size_bytes": r.size_bytes, "timestamp_ms": r.timestamp_ms, "hosts": r.hosts})
            }
        })
    }

    pub fn fetch(v: &Value) -> Result<Value, SdkError> {
        let reg = registry(v)?;
        let dir = PathBuf::from(str_arg(v, "dir")?);
        osm_sdk::datadir::ensure_private_dir(&dir)?;
        let mut node = RestStorage::new(str_arg(v, "storage")?, Duration::from_secs(120));
        let g = geofabrik(v);
        let f =
            osm_sdk::fetch::fetch_region(&reg, &mut node, g.as_ref(), str_arg(v, "region")?, &dir)?;
        let (kind, from) = match f.source {
            osm_sdk::fetch::Source::Storage { cid } => ("storage", cid),
            osm_sdk::fetch::Source::Geofabrik { url } => ("geofabrik", url),
        };
        let skipped: Vec<_> = f
            .skipped
            .iter()
            .map(|(c, w)| json!({"cid": c, "why": w}))
            .collect();
        Ok(
            json!({"path": f.path, "source": kind, "from": from, "md5": to_hex(&f.md5), "skipped": skipped}),
        )
    }

    pub fn hosted_status(v: &Value) -> Result<Value, SdkError> {
        let (s, errors) = osm_sdk::status::hosted_status(&registry(v)?);
        let rows: Vec<_> = s
            .iter()
            .map(|x| json!({"region": x.region, "version": x.version, "hosts": x.hosts}))
            .collect();
        let errs: Vec<_> = errors
            .iter()
            .map(|e| json!({"code": e.code(), "error": e.to_string()}))
            .collect();
        Ok(json!({"regions": rows, "errors": errs}))
    }

    pub fn check_updates(v: &Value) -> Result<Value, SdkError> {
        let reg = registry(v)?;
        let g = geofabrik(v).ok_or(SdkError::InvalidInput {
            field: "geofabrik",
            why: "update check needs Geofabrik".into(),
        })?;
        let regions: Vec<&str> = v["regions"]
            .as_array()
            .ok_or(SdkError::InvalidInput {
                field: "regions",
                why: "missing array".into(),
            })?
            .iter()
            .map(|r| {
                r.as_str().ok_or(SdkError::InvalidInput {
                    field: "regions",
                    why: "not a string".into(),
                })
            })
            .collect::<Result<_, _>>()?;
        if regions.len() > osm_core::PREDEFINED.len() {
            return Err(SdkError::InvalidInput {
                field: "regions",
                why: "more than 72".into(),
            });
        }
        let u = osm_sdk::status::check_updates(&reg, &g, &regions)?;
        Ok(Value::Array(
                u.iter()
                    .map(|x| json!({"region": x.region, "registered": x.registered, "available": x.available, "update_available": x.update_available()}))
                    .collect(),
            ))
    }

    /// `{"ok":true,"data":…}` or `{"ok":false,"code":…,"error":…}`, as the C ABI returns.
    pub fn envelope(r: Result<Value, SdkError>) -> String {
        match r {
            Ok(data) => json!({"ok": true, "data": data}).to_string(),
            Err(e) => json!({"ok": false, "code": e.code(), "error": e.to_string()}).to_string(),
        }
    }

    /// Parses a JSON object argument with the same size bound as the C ABI.
    pub fn parse(s: &str) -> Result<Value, SdkError> {
        if s.len() > MAX_ARGS_BYTES {
            return Err(invalid(format!(
                "arguments larger than {MAX_ARGS_BYTES} bytes"
            )));
        }
        let v: Value = serde_json::from_str(s).map_err(|e| invalid(e.to_string()))?;
        if !v.is_object() {
            return Err(invalid("arguments must be a JSON object"));
        }
        Ok(v)
    }
}

#[no_mangle]
pub extern "C" fn osm_version() -> *mut c_char {
    out(json!({"version": env!("CARGO_PKG_VERSION"), "api": API_VERSION}).to_string())
}

#[no_mangle]
pub extern "C" fn osm_discover(args_json: *const c_char) -> *mut c_char {
    respond(|| api::discover(&parse_args(args_json)?))
}

#[no_mangle]
pub extern "C" fn osm_resolve(args_json: *const c_char) -> *mut c_char {
    respond(|| api::resolve(&parse_args(args_json)?))
}

#[no_mangle]
pub extern "C" fn osm_fetch(args_json: *const c_char) -> *mut c_char {
    respond(|| api::fetch(&parse_args(args_json)?))
}

#[no_mangle]
pub extern "C" fn osm_hosted_status(args_json: *const c_char) -> *mut c_char {
    respond(|| api::hosted_status(&parse_args(args_json)?))
}

#[no_mangle]
pub extern "C" fn osm_check_updates(args_json: *const c_char) -> *mut c_char {
    respond(|| api::check_updates(&parse_args(args_json)?))
}

/// Frees a string returned by this library. Null is ignored.
///
/// # Safety
/// `s` must come from one of this library's functions and must not be used afterwards.
#[no_mangle]
pub unsafe extern "C" fn osm_free(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: per the contract, `s` was produced by CString::into_raw in `out`.
        drop(unsafe { CString::from_raw(s) });
    }
}
