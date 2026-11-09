//! `pramadani_osm_distribution`: the LP-0018 distribution module for Logos Basecamp.
//!
//! Every method takes and returns JSON text, the same shapes as the C ABI in
//! `osm_ffi/include/osm_ffi.h` (`{"ok":true,"data":…}` / `{"ok":false,"code":…,"error":…}`); the
//! logic is `osm_ffi::api`, so the module, the C ABI and the CLI share one implementation.
//! Settings and downloads live in this instance's persistence folder, so two Basecamp profiles
//! never share state (H38).

use osm_ffi::api;
use serde_json::{json, Value};

/// Module API (frozen consumer part: discover, resolve, fetch, check_updates, hosted_status).
pub trait PramadaniOsmDistributionModule: Send + 'static {
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
    /// Register a file already stored in Logos Storage, signed by the user's wallet in `lez_core`:
    /// `{"host": account, "region", "cid", "source_url", "md5", "snapshot", "size_bytes"}`.
    /// Answers only after the entry is visible on-chain (H33), else `S103`.
    fn register(&mut self, entry: String) -> String;
    /// Stop hosting: `{"host", "region", "cid"}`; confirmed by read-back like `register`.
    fn unhost(&mut self, request: String) -> String;

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

impl PramadaniOsmDistributionModule for Distribution {
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

    fn register(&mut self, entry: String) -> String {
        let r = (|| {
            let e = api::parse(&entry)?;
            let s = |k: &'static str| e[k].as_str().map(str::to_string).ok_or(osm_sdk::SdkError::InvalidInput { field: k, why: "missing".into() });
            let n = |k: &'static str| e[k].as_u64().ok_or(osm_sdk::SdkError::InvalidInput { field: k, why: "missing".into() });
            let region = s("region")?;
            let def = osm_core::lookup(&region).ok_or(osm_sdk::SdkError::InvalidInput { field: "region", why: "not in the predefined set".into() })?;
            let (cid, source_url, md5) = (s("cid")?, s("source_url")?, s("md5")?);
            let version = u32::try_from(n("snapshot")?).map_err(|_| osm_sdk::SdkError::InvalidInput { field: "snapshot", why: "not YYMMDD".into() })?;
            let size_bytes = n("size_bytes")?;
            let timestamp = now_ms();
            // Same rules as the program, checked before anything is signed.
            osm_core::entry::validate_entry(&osm_core::entry::EntryInput {
                region: &region, parent: def.parent, level: def.level, cid: &cid, source_url: &source_url,
                checksum_hex: &md5, version, size_bytes, timestamp_ms: timestamp,
            })
            .map_err(|err| osm_sdk::SdkError::InvalidInput { field: "entry", why: format!("{} {err:?}", err.code()) })?;
            let ix = osm_sdk::instruction::Instruction::Register {
                region: region.clone(), parent: def.parent.unwrap_or("").into(), level: u8::from(def.level == osm_core::Level::Subregion),
                cid: cid.clone(), source_url, checksum: md5, version, size_bytes, timestamp,
            };
            let host = s("host")?;
            write_and_confirm(&self.settings, &host, &region, ix, |acc, h| acc.history.iter().any(|x| x.cid == cid && x.hosts.contains(h)))
        })();
        api::envelope(r)
    }

    fn unhost(&mut self, request: String) -> String {
        let r = (|| {
            let e = api::parse(&request)?;
            let s = |k: &'static str| e[k].as_str().map(str::to_string).ok_or(osm_sdk::SdkError::InvalidInput { field: k, why: "missing".into() });
            let (region, cid, host) = (s("region")?, s("cid")?, s("host")?);
            osm_core::lookup(&region).ok_or(osm_sdk::SdkError::InvalidInput { field: "region", why: "not in the predefined set".into() })?;
            let ix = osm_sdk::instruction::Instruction::Unhost { region: region.clone(), cid: cid.clone() };
            write_and_confirm(&self.settings, &host, &region, ix, |acc, h| !acc.history.iter().any(|x| x.cid == cid && x.hosts.contains(h)))
        })();
        api::envelope(r)
    }

    fn on_context_ready(&mut self, ctx: &RustModuleContext) {
        let dir = std::path::PathBuf::from(&ctx.instance_persistence_path).join("pramadani_osm_distribution");
        if osm_sdk::datadir::ensure_private_dir(&dir).is_ok() {
            if let Some(saved) = std::fs::read(dir.join("settings.json")).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok()) {
                self.settings = saved;
            }
            self.dir = Some(dir);
        }
    }
}

/// Sends `instruction` for `region` through `lez_core.send_generic_public_transaction` with the
/// region account (not signed) and the host (signs, pays), then polls the region account until
/// `done` holds. Never reports success from submission alone (H33).
fn write_and_confirm(
    settings: &Value,
    host_b58: &str,
    region: &str,
    instruction: osm_sdk::instruction::Instruction,
    done: impl Fn(&osm_sdk::registry::RegionAccount, &[u8; 32]) -> bool,
) -> Result<Value, osm_sdk::SdkError> {
    use osm_sdk::instruction::account_hex;
    use osm_sdk::registry::{decode_account_id, region_account_id, Registry, RpcRegistry};
    let invalid = |why: String| osm_sdk::SdkError::InvalidInput { field: "register", why };
    let rpc = settings["rpc"].as_str().ok_or_else(|| invalid("no rpc in settings".into()))?;
    let program = settings["program"].as_str().ok_or_else(|| invalid("no program in settings".into()))?;
    let host = decode_account_id(host_b58.trim_start_matches("Public/"))?;
    let need = (500_000u128 + 100_000) * 64; // fee reserve at the SPEL client's gas limit (docs/cycles.md)
    let have = osm_sdk::registry::account_balance(rpc, host_b58.trim_start_matches("Public/"))?;
    if have < need {
        return Err(invalid(format!("S105 InsufficientFunds: {have} lepta available, {need} needed for the fee reserve")));
    }
    let account = region_account_id(program, region)?;
    let tx = modules()
        .lez_core
        .send_generic_public_transaction(
            vec![account_hex(&account)?, account_hex(host_b58)?],
            vec![false, true],
            instruction.to_bytes(),
            account_hex(program)?,
            String::new(),
            Vec::new(),
        )
        .map_err(|e| invalid(format!("S104 the wallet module refused the transaction: {e:?}")))?;
    let reg = Registry { source: RpcRegistry::new(rpc, program)?, program: program.to_string() };
    let start = std::time::Instant::now();
    loop {
        if let Some(acc) = reg.region(region)? {
            if done(&acc, &host) {
                return Ok(json!({"confirmed": true, "wallet_answer": tx}));
            }
        }
        if start.elapsed() > std::time::Duration::from_secs(120) {
            return Err(invalid(format!("S103 TxNotIncluded: sent ({tx}) but the registry shows no change after 120 s")));
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[no_mangle]
pub extern "Rust" fn logos_module_install() {
    install::<Distribution>();
}
