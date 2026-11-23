//! `pramadani_osm_distribution`: the LP-0018 distribution module for Logos Basecamp.
//!
//! Every method takes and returns JSON text, the same shapes as the C ABI in
//! `osm_ffi/include/osm_ffi.h` (`{"ok":true,"data":…}` / `{"ok":false,"code":…,"error":…}`); the
//! read logic is `osm_ffi::api`, so the module, the C ABI and the CLI share one implementation.
//! Writes are signed by the user's wallet in `lez_core` and confirmed by reading the region
//! account back. Settings, cache and downloads live in this instance's persistence folder, so two
//! Basecamp profiles never share state (H38).

use osm_ffi::api;
use osm_sdk::SdkError;
use serde_json::{json, Value};

/// Module API. The consumer part (discover, resolve, fetch, check_updates, hosted_status) is
/// frozen for the alpha; lookup, host, host_batch, register_batch, import_file, register_entry
/// and unhost are the distribution app's.
pub trait PramadaniOsmDistributionModule: Send + 'static {
    /// `{"version","api"}`.
    fn version(&mut self) -> String;
    /// Current settings: `rpc`, `program`, `storage`, `geofabrik`, and where data is kept.
    fn settings(&mut self) -> String;
    /// Merge and save settings (JSON object with any of `rpc`, `program`, `storage`, `geofabrik`,
    /// `host`: the default host account for writes).
    fn configure(&mut self, settings: String) -> String;
    /// The 72 regions with names, level and parent.
    /// Liveness of the configured services, each from a real request: `{"rpc": {"ok", "block" |
    /// "error"}, "storage": {"ok", "quota_max_bytes", "quota_used_bytes" | "error"}, "geofabrik"}`.
    fn status(&mut self) -> String;
    fn discover(&mut self) -> String;
    /// Newest hosted entry of a region: CID, MD5, snapshot, size, hosts.
    fn resolve(&mut self, region: String) -> String;
    /// Download a region into this instance's `maps` folder and verify its MD5.
    fn fetch(&mut self, region: String) -> String;
    /// Hosted snapshot and host count for all 72 regions.
    fn hosted_status(&mut self) -> String;
    /// Registered vs current Geofabrik snapshot; `regions` is a JSON array of region ids.
    fn check_updates(&mut self, regions: String) -> String;
    /// Host a region end to end: `{"host", "region", "snapshot"?}`. Downloads the dated Geofabrik
    /// snapshot (first of the current month by default), checks its MD5, uploads it to the
    /// configured Storage node with retries, then registers it like `register_entry`.
    fn host(&mut self, request: String) -> String;
    /// Host up to 16 regions with one registration transaction: `{"host", "regions": [...],
    /// "snapshot"?}`. Each region is downloaded, checked and uploaded like `host`; then one
    /// `register_batch` registers them all, confirmed by reading every region back.
    fn host_batch(&mut self, request: String) -> String;
    /// Register up to 16 files already in Logos Storage in one transaction (batch-register):
    /// `{"host", "entries": [{"region", "cid", "source_url", "md5", "snapshot", "size_bytes"}, ...]}`.
    /// Confirmed by reading every region back, like `register_entry`.
    fn register_batch(&mut self, request: String) -> String;
    /// Registry query by region, by parent or by CID: `{"region": …}`, `{"parent": …}` or
    /// `{"cid": …}`. Every matching entry, sorted by timestamp, and the accounts it could not read.
    fn lookup(&mut self, query: String) -> String;
    /// Import a local PBF: `{"host", "path", "region", "snapshot"}`. Checks the file against
    /// Geofabrik's published MD5 for that snapshot, uploads it, registers it.
    fn import_file(&mut self, request: String) -> String;
    /// Register a file already stored in Logos Storage, signed by the user's wallet in `lez_core`:
    /// `{"host", "region", "cid", "source_url", "md5", "snapshot", "size_bytes"}`.
    /// Answers only after the entry is visible on-chain (H33), else `S103`.
    fn register_entry(&mut self, entry: String) -> String;
    /// Stop hosting: `{"host", "region", "cid"}`; confirmed by read-back like `register_entry`.
    fn unhost(&mut self, request: String) -> String;

    fn on_context_ready(&mut self, _ctx: &RustModuleContext) {}
}

/// Events of this module.
pub trait PramadaniOsmDistributionModuleEvents {
    /// Progress of a long operation on `region`: `stage` is "download", "verify", "upload",
    /// "stored" or "register"; `done` and `total` are bytes (0 when not known).
    fn progress(&self, region: String, stage: String, done: u64, total: u64);
}

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/provider_gen.rs"));

/// Defaults = networks.json profile `v03-public` at the pinned commit (rehearsal network).
fn default_settings() -> Value {
    json!({
        "rpc": "https://testnet.lez.logos.co",
        "program": "9zuXeX1kzeukr1crbuWN92JirV3m7KMy9pvho9BCLhWw",
        "storage": "http://127.0.0.1:8080/api/storage/v1",
        "geofabrik": true
    })
}

fn invalid(field: &'static str, why: impl Into<String>) -> SdkError {
    SdkError::InvalidInput { field, why: why.into() }
}

fn str_field(e: &Value, k: &'static str) -> Result<String, SdkError> {
    e[k].as_str().map(str::to_string).ok_or_else(|| invalid(k, "missing"))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Fee the sequencer reserves up front at the SPEL client's gas limit (docs/cycles.md).
const FEE_RESERVE: u128 = (500_000 + 100_000) * 64;

/// Sends `instruction` through `lez_core.send_generic_public_transaction` with the host (signs,
/// pays) and the region accounts (not signed), in the IDL's account order: `register` and
/// `unhost` take `[region_account, host]`, `register_batch` takes `[host, region_accounts...]`.
/// Then polls every region account until `done(region, account, host)` holds. Never reports
/// success from submission alone (H33).
fn write_and_confirm(
    settings: &Value,
    host_b58: &str,
    regions: &[String],
    instruction: osm_sdk::instruction::Instruction,
    done: impl Fn(&str, &osm_sdk::registry::RegionAccount, &[u8; 32]) -> bool,
) -> Result<Value, SdkError> {
    use osm_sdk::instruction::account_hex;
    use osm_sdk::registry::{account_balance, decode_account_id, region_account_id, Registry, RpcRegistry};
    let host_b58 = host_b58.trim_start_matches("Public/");
    let rpc = settings["rpc"].as_str().ok_or_else(|| invalid("rpc", "not set"))?;
    let program = settings["program"].as_str().ok_or_else(|| invalid("program", "not set"))?;
    let host = decode_account_id(host_b58)?;
    let have = account_balance(rpc, host_b58)?;
    if have < FEE_RESERVE {
        return Err(invalid("host", format!("S105 InsufficientFunds: {have} lepta available, {FEE_RESERVE} needed for the fee reserve")));
    }
    let mut region_hex = Vec::new();
    for r in regions {
        region_hex.push(account_hex(&region_account_id(program, r)?)?);
    }
    let host_hex = account_hex(host_b58)?;
    let (accounts, signing): (Vec<String>, Vec<bool>) = match instruction {
        osm_sdk::instruction::Instruction::RegisterBatch { .. } => {
            (std::iter::once(host_hex).chain(region_hex).collect(), std::iter::once(true).chain(regions.iter().map(|_| false)).collect())
        }
        _ => (region_hex.into_iter().chain(std::iter::once(host_hex)).collect(), regions.iter().map(|_| false).chain(std::iter::once(true)).collect()),
    };
    let program_hex = account_hex(program)?;
    for r in regions {
        emit_progress(r, "register", 0, 0);
    }
    // Default-level log of every write (E82): which regions, which host, then the outcome.
    eprintln!("pramadani_osm_distribution: sending a registry write for {} as {host_b58}", regions.join(", "));
    let tx = modules()
        .lez_core
        .send_generic_public_transaction(&accounts, &signing, &instruction.to_bytes(), &program_hex, "", &[])
        .map_err(|e| invalid("transaction", format!("S104 the wallet module refused the transaction: {e:?}")))?;
    let reg = Registry { source: RpcRegistry::new(rpc, program)?, program: program.to_string() };
    let start = std::time::Instant::now();
    loop {
        let mut all = true;
        for r in regions {
            if !reg.region(r)?.is_some_and(|acc| done(r, &acc, &host)) {
                all = false;
                break;
            }
        }
        if all {
            eprintln!("pramadani_osm_distribution: confirmed on-chain: {}", regions.join(", "));
            return Ok(json!({"confirmed": true, "regions": regions, "wallet_answer": tx}));
        }
        // A public testnet has taken over 2 minutes to include a transaction.
        if start.elapsed() > std::time::Duration::from_secs(600) {
            return Err(invalid("transaction", format!("S103 TxNotIncluded: sent ({tx}) but the registry shows no change after 600 s. It may still be included later: check the region before sending again.")));
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
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

    fn data_dir(&self, sub: &str) -> Result<std::path::PathBuf, SdkError> {
        let dir = self.dir.as_ref().ok_or_else(|| invalid("data_dir", "module context not ready"))?.join(sub);
        osm_sdk::datadir::ensure_private_dir(&dir)
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

    fn geofabrik(&self) -> Result<osm_sdk::geofabrik::Geofabrik, SdkError> {
        if self.settings["geofabrik"].as_bool() == Some(false) {
            return Err(invalid("geofabrik", "disabled in settings; this action needs Geofabrik"));
        }
        Ok(osm_sdk::geofabrik::Geofabrik::new())
    }

    fn upload(&self, path: &std::path::Path) -> Result<String, SdkError> {
        let url = self.settings["storage"].as_str().ok_or_else(|| invalid("storage", "not set"))?;
        let mut node = osm_sdk::rest_storage::RestStorage::new(url, std::time::Duration::from_secs(600));
        osm_sdk::storage::upload_with_retry(
            &mut node,
            path,
            &osm_sdk::retry::RetryPolicy::default(),
            &mut osm_sdk::retry::XorShift::seeded(),
            std::thread::sleep,
        )
    }

    /// Downloads the dated snapshot of `region` (first of the current month unless `snapshot`),
    /// checks it against Geofabrik's MD5, uploads it and returns the entry to register.
    fn prepare(&self, region: &str, snapshot: Option<u64>) -> Result<Value, SdkError> {
        let def = osm_core::lookup(region).ok_or_else(|| invalid("region", "not in the predefined set"))?;
        let dir = self.data_dir("cache")?;
        let g = self.geofabrik()?;
        let snap = match snapshot {
            Some(s) => u32::try_from(s).map_err(|_| invalid("snapshot", "not YYMMDD"))?,
            None => osm_sdk::snapshot::first_of_month(g.latest_snapshot(def)?),
        };
        eprintln!("pramadani_osm_distribution: downloading {region} snapshot {snap:06} from Geofabrik");
        // Byte progress for the app, at most twice a second.
        let mut last = std::time::Instant::now() - std::time::Duration::from_secs(1);
        let (path, md5) = g.download_snapshot(def, snap, &dir, |done, total| {
            if last.elapsed() >= std::time::Duration::from_millis(500) {
                last = std::time::Instant::now();
                emit_progress(region, "download", done, total.unwrap_or(0));
            }
        })?;
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        emit_progress(region, "verify", size, size);
        eprintln!("pramadani_osm_distribution: {region}: MD5 matches Geofabrik; uploading to Logos Storage");
        emit_progress(region, "upload", 0, size);
        let cid = self.upload(&path)?;
        emit_progress(region, "stored", size, size);
        eprintln!("pramadani_osm_distribution: {region}: stored as {cid}");
        let size = std::fs::metadata(&path).map_err(|err| invalid("file", err.to_string()))?.len();
        let _ = osm_sdk::datadir::enforce_cache_cap(&dir, 20 << 30, &path);
        let url = osm_sdk::snapshot::snapshot_url(def, snap)?;
        Ok(json!({"region": region, "cid": cid, "source_url": url, "md5": osm_sdk::hash::to_hex(&md5),
            "snapshot": snap, "size_bytes": size}))
    }

    /// Registers up to 16 files already in Logos Storage in one `register_batch` transaction:
    /// each entry `{"region", "cid", "source_url", "md5", "snapshot", "size_bytes"}`. Every entry
    /// is checked with the program's own rules first, and every region is read back after.
    fn register_many(&self, host: &str, entries: Vec<Value>) -> Result<Value, SdkError> {
        let regions: Vec<String> = entries.iter().map(|x| str_field(x, "region")).collect::<Result<_, _>>()?;
        let names: Vec<&str> = regions.iter().map(String::as_str).collect();
        osm_core::entry::validate_batch_shape(&names, &[]).map_err(|err| invalid("entries", format!("{} {err:?}", err.code())))?;
        let timestamp = now_ms();
        let mut defs = Vec::new();
        for x in &entries {
            let r = str_field(x, "region")?;
            let def = osm_core::lookup(&r).ok_or_else(|| invalid("region", format!("{r:?} is not in the predefined set")))?;
            let version = x["snapshot"].as_u64().and_then(|s| u32::try_from(s).ok()).ok_or_else(|| invalid("snapshot", "not YYMMDD"))?;
            osm_core::entry::validate_entry(&osm_core::entry::EntryInput {
                region: def.region,
                parent: def.parent,
                level: def.level,
                cid: &str_field(x, "cid")?,
                source_url: &str_field(x, "source_url")?,
                checksum_hex: &str_field(x, "md5")?,
                version,
                size_bytes: x["size_bytes"].as_u64().ok_or_else(|| invalid("size_bytes", "missing"))?,
                timestamp_ms: timestamp,
            })
            .map_err(|err| invalid("entry", format!("{r}: {} {err:?}", err.code())))?;
            defs.push(def);
        }
        let s = |k: &str| entries.iter().map(|x| x[k].as_str().unwrap_or("").to_string()).collect::<Vec<_>>();
        let ix = osm_sdk::instruction::Instruction::RegisterBatch {
            regions: regions.clone(),
            parents: defs.iter().map(|d| d.parent.unwrap_or("").to_string()).collect(),
            levels: defs.iter().map(|d| u8::from(d.level == osm_core::Level::Subregion)).collect(),
            cids: s("cid"),
            source_urls: s("source_url"),
            checksums: s("md5"),
            versions: entries.iter().map(|x| x["snapshot"].as_u64().unwrap_or(0)).collect(),
            sizes: entries.iter().map(|x| x["size_bytes"].as_u64().unwrap_or(0)).collect(),
            timestamp,
        };
        let cids: std::collections::HashMap<String, String> =
            entries.iter().map(|x| (x["region"].as_str().unwrap_or("").to_string(), x["cid"].as_str().unwrap_or("").to_string())).collect();
        let mut out = write_and_confirm(&self.settings, host, &regions, ix, |r, acc, h| {
            acc.history.iter().any(|x| Some(&x.cid) == cids.get(r) && x.hosts.contains(h))
        })?;
        out["entries"] = json!(entries);
        Ok(out)
    }

    fn register_value(&self, e: &Value) -> Result<Value, SdkError> {
        let region = str_field(e, "region")?;
        let def = osm_core::lookup(&region).ok_or_else(|| invalid("region", "not in the predefined set"))?;
        let (cid, source_url, md5, host) = (str_field(e, "cid")?, str_field(e, "source_url")?, str_field(e, "md5")?, str_field(e, "host")?);
        let version = e["snapshot"].as_u64().and_then(|s| u32::try_from(s).ok()).ok_or_else(|| invalid("snapshot", "not YYMMDD"))?;
        let size_bytes = e["size_bytes"].as_u64().ok_or_else(|| invalid("size_bytes", "missing"))?;
        let timestamp = now_ms();
        // Same rules as the program, checked before anything is signed.
        osm_core::entry::validate_entry(&osm_core::entry::EntryInput {
            region: &region,
            parent: def.parent,
            level: def.level,
            cid: &cid,
            source_url: &source_url,
            checksum_hex: &md5,
            version,
            size_bytes,
            timestamp_ms: timestamp,
        })
        .map_err(|err| invalid("entry", format!("{} {err:?}", err.code())))?;
        let ix = osm_sdk::instruction::Instruction::Register {
            region: region.clone(),
            parent: def.parent.unwrap_or("").into(),
            level: u8::from(def.level == osm_core::Level::Subregion),
            cid: cid.clone(),
            source_url,
            checksum: md5,
            version,
            size_bytes,
            timestamp,
        };
        let mut out = write_and_confirm(&self.settings, &host, &[region], ix, |_, acc, h| {
            acc.history.iter().any(|x| x.cid == cid && x.hosts.contains(h))
        })?;
        out["cid"] = json!(cid);
        Ok(out)
    }
}

impl PramadaniOsmDistributionModule for Distribution {
    fn version(&mut self) -> String {
        api::envelope(Ok(json!({"module": env!("CARGO_PKG_VERSION"), "api": osm_ffi::API_VERSION})))
    }

    fn settings(&mut self) -> String {
        // Where each effective value comes from (E68): the built-in default, or the user's own.
        let defaults = default_settings();
        let sources: serde_json::Map<String, Value> = self
            .settings
            .as_object()
            .map(|o| {
                o.iter()
                    .map(|(k, v)| (k.clone(), json!(if defaults.get(k) == Some(v) { "default" } else { "custom" })))
                    .collect()
            })
            .unwrap_or_default();
        api::envelope(Ok(json!({"settings": self.settings, "sources": sources, "data_dir": self.dir})))
    }

    fn configure(&mut self, settings: String) -> String {
        let r = (|| {
            let new = api::parse(&settings)?;
            let allowed = ["rpc", "program", "storage", "geofabrik", "host"];
            for k in new.as_object().into_iter().flat_map(|o| o.keys()) {
                if !allowed.contains(&k.as_str()) {
                    return Err(invalid("settings", format!("unknown key {k:?}")));
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

    fn status(&mut self) -> String {
        // Each part is a real request; a failure is reported, never assumed away.
        let rpc = self.settings["rpc"].as_str().unwrap_or("").to_string();
        let storage = self.settings["storage"].as_str().unwrap_or("").to_string();
        let rpc_part = match osm_sdk::registry::rpc_last_block(&rpc) {
            Ok(b) => json!({"ok": true, "url": rpc, "block": b}),
            Err(e) => json!({"ok": false, "url": rpc, "code": e.code(), "error": e.to_string()}),
        };
        let storage_part = match osm_sdk::rest_storage::RestStorage::new(&storage, std::time::Duration::from_secs(10)).space() {
            Ok(v) => json!({"ok": true, "url": storage, "quota_max_bytes": v["quotaMaxBytes"], "quota_used_bytes": v["quotaUsedBytes"]}),
            Err(e) => json!({"ok": false, "url": storage, "code": "S110", "error": format!("S110 StorageUnreachable: {e:?}")}),
        };
        api::envelope(Ok(json!({"rpc": rpc_part, "storage": storage_part, "geofabrik": self.settings["geofabrik"].as_bool() != Some(false)})))
    }

    fn discover(&mut self) -> String {
        api::envelope(api::discover(&self.args(json!({}))))
    }

    fn resolve(&mut self, region: String) -> String {
        api::envelope(api::resolve(&self.args(json!({"region": region}))))
    }

    fn fetch(&mut self, region: String) -> String {
        eprintln!("pramadani_osm_distribution: fetching {region}");
        let r = self.data_dir("maps").and_then(|dir| api::fetch(&self.args(json!({"region": region, "dir": dir}))));
        match &r {
            Ok(v) => eprintln!("pramadani_osm_distribution: {region}: saved {} (MD5 verified)", v["path"]),
            Err(e) => eprintln!("pramadani_osm_distribution: {region}: {e}"),
        }
        api::envelope(r)
    }

    fn hosted_status(&mut self) -> String {
        api::envelope(api::hosted_status(&self.args(json!({}))))
    }

    fn check_updates(&mut self, regions: String) -> String {
        let r = serde_json::from_str::<Value>(&regions)
            .map_err(|e| invalid("regions", e.to_string()))
            .and_then(|list| api::check_updates(&self.args(json!({"regions": list}))));
        api::envelope(r)
    }

    fn host(&mut self, request: String) -> String {
        let r = (|| {
            let e = api::parse(&request)?;
            let mut entry = self.prepare(&str_field(&e, "region")?, e["snapshot"].as_u64())?;
            entry["host"] = json!(str_field(&e, "host")?);
            self.register_value(&entry)
        })();
        api::envelope(r)
    }

    fn host_batch(&mut self, request: String) -> String {
        let r = (|| {
            let e = api::parse(&request)?;
            let host = str_field(&e, "host")?;
            let regions: Vec<String> = e["regions"]
                .as_array()
                .ok_or_else(|| invalid("regions", "missing array"))?
                .iter()
                .map(|v| v.as_str().map(str::to_string).ok_or_else(|| invalid("regions", "not a string")))
                .collect::<Result<_, _>>()?;
            let names: Vec<&str> = regions.iter().map(String::as_str).collect();
            // Checked before any download, so a bad selection costs nothing.
            osm_core::entry::validate_batch_shape(&names, &[]).map_err(|err| invalid("regions", format!("{} {err:?}", err.code())))?;
            let mut entries = Vec::new();
            for r in &regions {
                entries.push(self.prepare(r, e["snapshot"].as_u64())?);
            }
            self.register_many(&host, entries)
        })();
        api::envelope(r)
    }

    fn register_batch(&mut self, request: String) -> String {
        let r = (|| {
            let e = api::parse(&request)?;
            let host = str_field(&e, "host")?;
            let entries = e["entries"].as_array().ok_or_else(|| invalid("entries", "missing array"))?.clone();
            self.register_many(&host, entries)
        })();
        api::envelope(r)
    }

    fn lookup(&mut self, query: String) -> String {
        let r = api::parse(&query).and_then(|q| {
            let mut v = self.args(json!({}));
            for k in ["region", "parent", "cid"] {
                if let Some(s) = q[k].as_str() {
                    v[k] = json!(s);
                }
            }
            api::lookup(&v)
        });
        api::envelope(r)
    }

    fn import_file(&mut self, request: String) -> String {
        let r = (|| {
            let e = api::parse(&request)?;
            let (region, host) = (str_field(&e, "region")?, str_field(&e, "host")?);
            let path = std::path::PathBuf::from(str_field(&e, "path")?);
            let def = osm_core::lookup(&region).ok_or_else(|| invalid("region", "not in the predefined set"))?;
            let snap = e["snapshot"].as_u64().and_then(|s| u32::try_from(s).ok()).ok_or_else(|| invalid("snapshot", "not YYMMDD"))?;
            let url = osm_sdk::snapshot::snapshot_url(def, snap)?;
            let want = self.geofabrik()?.snapshot_md5(def, snap)?;
            osm_sdk::hash::verify_import(&path, &want, &format!("{region}-{snap:06}"))?;
            let cid = self.upload(&path)?;
            let size = std::fs::metadata(&path).map_err(|err| invalid("path", err.to_string()))?.len();
            self.register_value(&json!({"host": host, "region": region, "cid": cid, "source_url": url,
                "md5": osm_sdk::hash::to_hex(&want), "snapshot": snap, "size_bytes": size}))
        })();
        api::envelope(r)
    }

    fn register_entry(&mut self, entry: String) -> String {
        api::envelope(api::parse(&entry).and_then(|e| self.register_value(&e)))
    }

    fn unhost(&mut self, request: String) -> String {
        let r = (|| {
            let e = api::parse(&request)?;
            let (region, cid, host) = (str_field(&e, "region")?, str_field(&e, "cid")?, str_field(&e, "host")?);
            osm_core::lookup(&region).ok_or_else(|| invalid("region", "not in the predefined set"))?;
            let ix = osm_sdk::instruction::Instruction::Unhost { region: region.clone(), cid: cid.clone() };
            write_and_confirm(&self.settings, &host, &[region], ix, |_, acc, h| {
                !acc.history.iter().any(|x| x.cid == cid && x.hosts.contains(h))
            })
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

#[no_mangle]
pub extern "Rust" fn logos_module_install() {
    install::<Distribution>();
}
