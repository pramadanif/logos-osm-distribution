//! `pramadani_osm_sdk`: the read-only consumer subset of the LP-0018 SDK as a Basecamp module
//! (no wallet, no writes). Modules that only need to find and use map files depend on this one;
//! the distribution app uses `pramadani_osm_distribution`, which adds hosting.
//!
//! Every method takes and returns JSON text, the same shapes as the C ABI in
//! `osm_ffi/include/osm_ffi.h` (`{"ok":true,"data":…}` / `{"ok":false,"code":…,"error":…}`); the
//! logic is `osm_ffi::api`, so the module, the C ABI and the CLI share one implementation.
//! Settings and downloads live in this instance's persistence folder, so two Basecamp profiles
//! never share state (H38).

use osm_ffi::api;
use serde_json::{json, Value};

/// Module API (frozen consumer part: discover, resolve, fetch, check_updates, hosted_status).
pub trait PramadaniOsmSdkModule: Send + 'static {
    /// `{"version","api"}`.
    fn version(&mut self) -> String;
    /// Current settings: `rpc`, `program`, `storage`, `geofabrik`, and where data is kept.
    fn settings(&mut self) -> String;
    /// Merge and save settings (JSON object with any of `rpc`, `program`, `storage`, `geofabrik`).
    fn configure(&mut self, settings: String) -> String;
    /// The 72 regions with names, level and parent.
    fn discover(&mut self) -> String;
    /// Newest hosted entry of a region: CID, MD5, snapshot, size, hosts.
    fn resolve(&mut self, region: String) -> String;
    /// Download a region into this instance's `maps` folder and verify its MD5.
    fn fetch(&mut self, region: String) -> String;
    /// Hosted snapshot and host count for all 72 regions.
    fn hosted_status(&mut self) -> String;
    /// Registered vs current Geofabrik snapshot; `regions` is a JSON array of region ids.
    fn check_updates(&mut self, regions: String) -> String;

    fn on_context_ready(&mut self, _ctx: &RustModuleContext) {}
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/provider_gen.rs"));

/// Defaults = networks.json profile `v03-public` at the pinned commit (rehearsal network).
fn default_settings() -> Value {
    json!({
        "rpc": "https://testnet.lez.logos.co",
        "program": "5VZeCZKSzkQyYb2nPm4ShDAKpkwncvbQh58XSg4cCAcS",
        "storage": "http://127.0.0.1:8080/api/storage/v1",
        "geofabrik": true
    })
}

struct Distribution {
    settings: Value,
    dir: Option<std::path::PathBuf>,
}

impl Default for Distribution {
    fn default() -> Self {
        Distribution { settings: default_settings(), dir: None }
    }
}

impl Distribution {
    fn settings_file(&self) -> Option<std::path::PathBuf> {
        self.dir.as_ref().map(|d| d.join("settings.json"))
    }

    /// Settings merged with call arguments (arguments win).
    fn args(&self, extra: Value) -> Value {
        let mut v = self.settings.clone();
        if let (Some(o), Some(e)) = (v.as_object_mut(), extra.as_object()) {
            for (k, x) in e {
                o.insert(k.clone(), x.clone());
            }
        }
        v
    }
}

impl PramadaniOsmSdkModule for Distribution {
    fn version(&mut self) -> String {
        json!({"ok": true, "data": {"module": env!("CARGO_PKG_VERSION"), "api": osm_ffi::API_VERSION}}).to_string()
    }

    fn settings(&mut self) -> String {
        api::envelope(Ok(json!({"settings": self.settings, "data_dir": self.dir})))
    }

    fn configure(&mut self, settings: String) -> String {
        let r = (|| {
            let new = api::parse(&settings)?;
            let allowed = ["rpc", "program", "storage", "geofabrik"];
            for k in new.as_object().into_iter().flat_map(|o| o.keys()) {
                if !allowed.contains(&k.as_str()) {
                    return Err(osm_sdk::SdkError::InvalidInput { field: "settings", why: format!("unknown key {k:?}") });
                }
            }
            let merged = self.args(new);
            if let Some(f) = self.settings_file() {
                let bytes = serde_json::to_vec_pretty(&merged).expect("settings serialize");
                osm_sdk::datadir::atomic_write(&f, &bytes, |b| serde_json::from_slice::<Value>(b).is_ok())?;
            }
            self.settings = merged;
            Ok(self.settings.clone())
        })();
        api::envelope(r)
    }

    fn discover(&mut self) -> String {
        api::envelope(api::discover(&self.args(json!({}))))
    }

    fn resolve(&mut self, region: String) -> String {
        api::envelope(api::resolve(&self.args(json!({"region": region}))))
    }

    fn fetch(&mut self, region: String) -> String {
        let r = (|| {
            let dir = self.dir.as_ref().ok_or(osm_sdk::SdkError::InvalidInput { field: "data_dir", why: "module context not ready".into() })?;
            api::fetch(&self.args(json!({"region": region, "dir": dir.join("maps")})))
        })();
        api::envelope(r)
    }

    fn hosted_status(&mut self) -> String {
        api::envelope(api::hosted_status(&self.args(json!({}))))
    }

    fn check_updates(&mut self, regions: String) -> String {
        let r = serde_json::from_str::<Value>(&regions)
            .map_err(|e| osm_sdk::SdkError::InvalidInput { field: "regions", why: e.to_string() })
            .and_then(|list| api::check_updates(&self.args(json!({"regions": list}))));
        api::envelope(r)
    }

    fn on_context_ready(&mut self, ctx: &RustModuleContext) {
        let dir = std::path::PathBuf::from(&ctx.instance_persistence_path).join("pramadani_osm_sdk");
        if osm_sdk::datadir::ensure_private_dir(&dir).is_ok() {
            if let Some(saved) = std::fs::read(dir.join("settings.json")).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
                self.settings = saved;
            }
            self.dir = Some(dir);
        }
    }
}

#[no_mangle]
pub extern "Rust" fn logos_module_install() {
    install::<Distribution>();
}
