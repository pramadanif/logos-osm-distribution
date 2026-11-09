//! `osm`: command line for the LP-0018 OSM registry. Commands that need the chain are added with
//! the LEZ adapter; everything here works without it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use osm_core::{lookup, RegionDef, PREDEFINED};
use osm_sdk::geofabrik::{discover, Geofabrik, Source};
use osm_sdk::hash::{to_hex, verify_import};
use osm_sdk::registry::{RegisteredEntry, Registry, RpcRegistry};
use osm_sdk::rest_storage::RestStorage;
use osm_sdk::retry::{RetryPolicy, XorShift};
use osm_sdk::snapshot::{first_of_month, snapshot_url};
use osm_sdk::storage::upload_with_retry;
use osm_sdk::SdkError;
use serde_json::json;

#[derive(Parser)]
#[command(
    name = "osm",
    version,
    about = "LP-0018 OSM data distribution on Logos"
)]
struct Cli {
    /// Print machine-readable JSON instead of text.
    #[arg(long, global = true)]
    json: bool,
    /// Never contact Geofabrik.
    #[arg(long, global = true)]
    no_geofabrik: bool,
    /// Server with Geofabrik's file layout to fetch from (tests and mirrors). Registered source
    /// URLs always name download.geofabrik.de.
    #[arg(long, global = true, env = "OSM_GEOFABRIK_BASE", hide = true)]
    geofabrik_base: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List the 72 predefined regions (no network).
    Regions,
    /// Discover regions from the Geofabrik index, restricted to the predefined set.
    Discover,
    /// Show today's dated snapshot of a region and Geofabrik's MD5 for it.
    Snapshot { region: String },
    /// Host regions: download the dated Geofabrik snapshot, check its MD5, upload it to Logos
    /// Storage, and register it on-chain (one region, or several in one batch transaction).
    Host {
        /// Region to host; repeat for a bulk host (up to 16).
        #[arg(long = "region", required = true)]
        regions: Vec<String>,
        /// Snapshot date YYMMDD; default: first of the current month.
        #[arg(long)]
        snapshot: Option<u32>,
        /// Storage node REST API root.
        #[arg(
            long,
            env = "OSM_STORAGE",
            default_value = "http://127.0.0.1:8080/api/storage/v1"
        )]
        storage: String,
        /// Folder for downloaded PBFs (created owner-only).
        #[arg(long, env = "OSM_CACHE_DIR")]
        cache_dir: PathBuf,
        /// Cache size limit in GiB; oldest PBFs are deleted first (H39).
        #[arg(long, env = "OSM_CACHE_CAP_GIB", default_value_t = 20)]
        cache_cap_gib: u64,
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
        /// Hosting account (signs and pays).
        #[arg(long, env = "OSM_HOST_ACCOUNT")]
        host: String,
        /// The `osm-chain` binary that signs with the LEZ wallet (LEE_WALLET_HOME_DIR).
        #[arg(long, env = "OSM_CHAIN", default_value = "osm-chain")]
        chain_bin: PathBuf,
    },
    /// Import a local PBF: check it against Geofabrik's published MD5 for its snapshot (import
    /// time only), store it in Logos Storage, and register it.
    Import {
        file: PathBuf,
        #[arg(long)]
        region: String,
        /// Snapshot date YYMMDD the file was downloaded as.
        #[arg(long)]
        snapshot: u32,
        #[arg(
            long,
            env = "OSM_STORAGE",
            default_value = "http://127.0.0.1:8080/api/storage/v1"
        )]
        storage: String,
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
        #[arg(long, env = "OSM_HOST_ACCOUNT")]
        host: String,
        #[arg(long, env = "OSM_CHAIN", default_value = "osm-chain")]
        chain_bin: PathBuf,
    },
    /// Register entries whose files are already in Logos Storage, from a coverage manifest, in
    /// batches of up to 16 (used to replay coverage onto a new network after a reset).
    BulkRegister {
        /// JSON: {"entries": [{"region", "cid", "source_url", "md5", "snapshot", "size_bytes"}, ...]}
        #[arg(long)]
        manifest: PathBuf,
        /// Skip these regions (per-region opt-out).
        #[arg(long = "skip")]
        skip: Vec<String>,
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
        #[arg(long, env = "OSM_HOST_ACCOUNT")]
        host: String,
        #[arg(long, env = "OSM_CHAIN", default_value = "osm-chain")]
        chain_bin: PathBuf,
    },
    /// Stop hosting: remove your account from the hosts of a CID.
    Unhost {
        #[arg(long)]
        region: String,
        #[arg(long)]
        cid: String,
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
        #[arg(long, env = "OSM_HOST_ACCOUNT")]
        host: String,
        #[arg(long, env = "OSM_CHAIN", default_value = "osm-chain")]
        chain_bin: PathBuf,
    },
    /// Verify coverage like the prize evaluator: fetch each hosted CID from Logos Storage, hash
    /// it, compare with Geofabrik's published MD5 for its snapshot, count entries and countries.
    /// Exit 0 only when at least 25 regions and 15 countries verify.
    VerifyCoverage {
        #[arg(
            long,
            env = "OSM_STORAGE",
            default_value = "http://127.0.0.1:8080/api/storage/v1"
        )]
        storage: String,
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
        /// Scratch folder for downloads (each file is deleted after hashing).
        #[arg(long)]
        dir: PathBuf,
        /// Only check the newest hosted entry of each region.
        #[arg(long)]
        newest_only: bool,
    },
    /// Download a region: from Logos Storage by CID when hosted (newest entry whose bytes match
    /// the registered MD5), from Geofabrik only when nobody hosts it.
    Download {
        #[arg(long)]
        region: String,
        /// Folder to write the PBF into.
        #[arg(long)]
        dir: PathBuf,
        #[arg(
            long,
            env = "OSM_STORAGE",
            default_value = "http://127.0.0.1:8080/api/storage/v1"
        )]
        storage: String,
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
    },
    /// Compare each region's newest registered snapshot with Geofabrik's current one.
    UpdateCheck {
        /// Regions to check; default: every region that is hosted.
        #[arg(long = "region")]
        regions: Vec<String>,
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
    },
    /// Look up registry entries by region, parent or CID (sorted by timestamp).
    Lookup {
        #[arg(long, conflicts_with_all = ["parent", "cid"])]
        region: Option<String>,
        #[arg(long, conflicts_with = "cid")]
        parent: Option<String>,
        #[arg(long)]
        cid: Option<String>,
        /// Sequencer JSON-RPC URL.
        #[arg(long, env = "OSM_RPC")]
        rpc: String,
        /// Registry program id (header account, base58).
        #[arg(long, env = "OSM_PROGRAM")]
        program: String,
    },
    /// Check a local PBF against Geofabrik's published MD5 for a snapshot (import-time check).
    VerifyFile {
        file: PathBuf,
        #[arg(long)]
        region: String,
        /// Snapshot date YYMMDD, for example 261101.
        #[arg(long)]
        snapshot: u32,
    },
}

/// Exit status per docs/error-codes.md.
fn exit_for(e: &SdkError) -> u8 {
    match e.code() {
        "S110" | "S111" | "S112" | "S113" => 5,
        "S120" | "S121" | "S122" => 6,
        "S140" | "S141" => 8,
        "S130" | "S131" | "S132" => 7,
        "S150" | "S151" | "S152" => 9,
        "S100" | "S170" => 3,
        "S160" => 2,
        _ => 1,
    }
}

fn make_geofabrik(cli: &Cli) -> Geofabrik {
    match &cli.geofabrik_base {
        Some(base) => Geofabrik::with_base(base),
        None => Geofabrik::new(),
    }
}

fn region(id: &str) -> Result<&'static RegionDef, SdkError> {
    lookup(id).ok_or_else(|| SdkError::InvalidInput {
        field: "region",
        why: format!("{id:?} is not in the predefined set; run `osm regions`"),
    })
}

fn geofabrik(cli: &Cli) -> Result<Geofabrik, SdkError> {
    if cli.no_geofabrik {
        return Err(SdkError::InvalidInput {
            field: "--no-geofabrik",
            why: "this command needs Geofabrik".into(),
        });
    }
    Ok(make_geofabrik(cli))
}

/// Runs `osm-chain register` with an argument array (no shell) and returns its JSON answer.
fn chain_register(
    bin: &Path,
    rpc: &str,
    program: &str,
    host: &str,
    entries: &[String],
) -> Result<serde_json::Value, SdkError> {
    let mut cmd = Command::new(bin);
    cmd.args([
        "register",
        "--rpc",
        rpc,
        "--program",
        program,
        "--host",
        host,
    ]);
    for e in entries {
        cmd.args(["--entry", e]);
    }
    let out = cmd.output().map_err(|e| SdkError::Io {
        path: bin.to_path_buf(),
        source: e,
    })?;
    // The LEZ wallet library may print its own lines on stdout; the answer is the last JSON line.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or("");
    let v: serde_json::Value = serde_json::from_str(last).map_err(|_| SdkError::InvalidInput {
        field: "osm-chain",
        why: format!(
            "unexpected output (exit {:?}): {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ),
    })?;
    Ok(v)
}

fn entry_json(r: &RegisteredEntry) -> serde_json::Value {
    let e = &r.entry;
    json!({
        "region": r.region, "cid": e.cid, "source_url": e.source_url, "checksum": to_hex(&e.checksum),
        "version": e.version, "size_bytes": e.size_bytes, "hosted": e.hosted, "timestamp_ms": e.timestamp,
        "registered_by": e.registered_by_b58(), "hosts": e.hosts_b58(),
    })
}

fn run(cli: &Cli) -> Result<(), SdkError> {
    match &cli.cmd {
        Cmd::Regions => {
            if cli.json {
                let rows: Vec<_> = PREDEFINED
                    .iter()
                    .map(|r| json!({"region": r.region, "parent": r.parent, "level": r.level.as_str()}))
                    .collect();
                println!("{}", serde_json::Value::Array(rows));
            } else {
                for r in PREDEFINED.iter() {
                    println!(
                        "{:<34} {:<9} {}",
                        r.region,
                        r.level.as_str(),
                        r.parent.unwrap_or("-")
                    );
                }
            }
        }
        Cmd::Discover => {
            let g = (!cli.no_geofabrik).then(|| make_geofabrik(cli));
            let (d, src) = discover(g.as_ref());
            let source = match &src {
                Source::Index => "geofabrik-index".to_string(),
                Source::OfflineDisabled => "offline (Geofabrik disabled)".to_string(),
                Source::OfflineUnreachable(why) => format!("offline ({why})"),
            };
            if cli.json {
                let rows: Vec<_> = d
                    .regions
                    .iter()
                    .map(|r| json!({"region": r.def.region, "name": r.name, "parent": r.def.parent, "level": r.def.level.as_str()}))
                    .collect();
                let missing: Vec<_> = d.missing.iter().map(|r| r.region).collect();
                println!(
                    "{}",
                    json!({"source": source, "regions": rows, "missing": missing})
                );
            } else {
                eprintln!("source: {source}");
                for r in &d.regions {
                    println!(
                        "{:<34} {:<9} {}",
                        r.def.region,
                        r.def.level.as_str(),
                        r.name
                    );
                }
                for m in &d.missing {
                    eprintln!("warning: {} is no longer in the Geofabrik index", m.region);
                }
            }
        }
        Cmd::Snapshot { region: id } => {
            let def = region(id)?;
            let g = geofabrik(cli)?;
            let snap = g.latest_snapshot(def)?;
            let md5 = g.snapshot_md5(def, snap)?;
            let url = snapshot_url(def, snap)?;
            if cli.json {
                println!(
                    "{}",
                    json!({"region": def.region, "snapshot": snap, "source_url": url, "md5": to_hex(&md5)})
                );
            } else {
                println!("{} {snap:06} {} {url}", def.region, to_hex(&md5));
            }
        }
        Cmd::Host {
            regions,
            snapshot,
            storage,
            cache_dir,
            cache_cap_gib,
            rpc,
            program,
            host,
            chain_bin,
        } => {
            if regions.len() > osm_core::entry::MAX_BATCH {
                return Err(SdkError::InvalidInput {
                    field: "--region",
                    why: format!("at most {} per run", osm_core::entry::MAX_BATCH),
                });
            }
            let defs: Vec<_> = regions
                .iter()
                .map(|r| region(r))
                .collect::<Result<_, _>>()?;
            let g = geofabrik(cli)?;
            osm_sdk::datadir::ensure_private_dir(cache_dir)?;
            let mut store = RestStorage::new(storage, std::time::Duration::from_secs(120));
            let mut entries = Vec::new();
            let mut rows = Vec::new();
            for def in defs {
                let snap = match snapshot {
                    Some(s) => *s,
                    None => first_of_month(g.latest_snapshot(def)?),
                };
                let url = snapshot_url(def, snap)?;
                eprintln!("{}: downloading {url}", def.region);
                // One progress line per 64 MiB step and one at the end.
                let mut next_report = 0u64;
                let (path, md5) = g.download_snapshot(def, snap, cache_dir, |done, total| {
                    if done >= next_report || Some(done) == total {
                        let of = total
                            .map(|t| format!(" of {} MiB", t >> 20))
                            .unwrap_or_default();
                        eprintln!("  {} MiB{of}", done >> 20);
                        next_report = (done / (64 << 20) + 1) * (64 << 20);
                    }
                })?;
                let size = std::fs::metadata(&path)
                    .map_err(|e| SdkError::Io {
                        path: path.clone(),
                        source: e,
                    })?
                    .len();
                eprintln!(
                    "{}: MD5 {} matches Geofabrik; uploading to Storage",
                    def.region,
                    to_hex(&md5)
                );
                let cid = upload_with_retry(
                    &mut store,
                    &path,
                    &RetryPolicy::default(),
                    &mut XorShift::seeded(),
                    std::thread::sleep,
                )?;
                eprintln!("{}: stored as {cid}", def.region);
                for p in osm_sdk::datadir::enforce_cache_cap(
                    cache_dir,
                    cache_cap_gib.saturating_mul(1 << 30),
                    &path,
                )? {
                    eprintln!("cache limit: removed {}", p.display());
                }
                entries.push(format!(
                    "{}|{}|{}|{}|{:06}|{}",
                    def.region,
                    cid,
                    url,
                    to_hex(&md5),
                    snap,
                    size
                ));
                rows.push(json!({"region": def.region, "cid": cid, "source_url": url, "md5": to_hex(&md5), "snapshot": snap, "size_bytes": size}));
            }
            let res = chain_register(chain_bin, rpc, program, host, &entries)?;
            let ok = res["ok"].as_bool() == Some(true);
            if cli.json {
                println!("{}", json!({"ok": ok, "hosted": rows, "chain": res}));
            } else {
                println!(
                    "{}",
                    if ok {
                        format!("registered: {}", res["outcome"])
                    } else {
                        format!("NOT registered: {}", res["error"])
                    }
                );
            }
            if !ok {
                // Upload succeeded but registration did not: never report success (H33).
                std::process::exit(4);
            }
        }
        Cmd::Import {
            file,
            region: id,
            snapshot,
            storage,
            rpc,
            program,
            host,
            chain_bin,
        } => {
            let def = region(id)?;
            let url = snapshot_url(def, *snapshot)?;
            let want = geofabrik(cli)?.snapshot_md5(def, *snapshot)?;
            // Fails gracefully with S130 (both MD5s and the snapshot named) or S132 (not a PBF).
            verify_import(file, &want, &format!("{}-{snapshot:06}", def.region))?;
            let size = std::fs::metadata(file)
                .map_err(|e| SdkError::Io {
                    path: file.clone(),
                    source: e,
                })?
                .len();
            let mut store = RestStorage::new(storage, std::time::Duration::from_secs(120));
            let cid = upload_with_retry(
                &mut store,
                file,
                &RetryPolicy::default(),
                &mut XorShift::seeded(),
                std::thread::sleep,
            )?;
            let entry = format!(
                "{}|{}|{}|{}|{:06}|{}",
                def.region,
                cid,
                url,
                to_hex(&want),
                snapshot,
                size
            );
            let res = chain_register(chain_bin, rpc, program, host, &[entry])?;
            let ok = res["ok"].as_bool() == Some(true);
            if cli.json {
                println!(
                    "{}",
                    json!({"ok": ok, "region": def.region, "cid": cid, "md5": to_hex(&want), "chain": res})
                );
            } else {
                println!(
                    "{}: MD5 matches Geofabrik for {url}; stored as {cid}; {}",
                    def.region,
                    if ok {
                        format!("registered ({})", res["outcome"])
                    } else {
                        format!("NOT registered: {}", res["error"])
                    }
                );
            }
            if !ok {
                std::process::exit(4);
            }
        }
        Cmd::BulkRegister {
            manifest,
            skip,
            rpc,
            program,
            host,
            chain_bin,
        } => {
            let raw = std::fs::read(manifest).map_err(|e| SdkError::Io {
                path: manifest.clone(),
                source: e,
            })?;
            if raw.len() > 1 << 20 {
                return Err(SdkError::InvalidInput {
                    field: "manifest",
                    why: "larger than 1 MiB".into(),
                });
            }
            let v: serde_json::Value =
                serde_json::from_slice(&raw).map_err(|e| SdkError::InvalidInput {
                    field: "manifest",
                    why: e.to_string(),
                })?;
            let items = v["entries"]
                .as_array()
                .ok_or_else(|| SdkError::InvalidInput {
                    field: "manifest",
                    why: "missing \"entries\" array".into(),
                })?;
            let mut lines = Vec::new();
            for (i, e) in items.iter().enumerate() {
                let s = |k: &str| {
                    e[k].as_str()
                        .map(str::to_string)
                        .ok_or_else(|| SdkError::InvalidInput {
                            field: "manifest",
                            why: format!("entry {i}: {k} missing"),
                        })
                };
                let n = |k: &str| {
                    e[k].as_u64().ok_or_else(|| SdkError::InvalidInput {
                        field: "manifest",
                        why: format!("entry {i}: {k} missing"),
                    })
                };
                let r = s("region")?;
                if skip.contains(&r) {
                    continue;
                }
                region(&r)?;
                lines.push(format!(
                    "{}|{}|{}|{}|{:06}|{}",
                    r,
                    s("cid")?,
                    s("source_url")?,
                    s("md5")?,
                    n("snapshot")?,
                    n("size_bytes")?
                ));
            }
            let mut results = Vec::new();
            let mut failed = false;
            for chunk in lines.chunks(osm_core::entry::MAX_BATCH) {
                let res = chain_register(chain_bin, rpc, program, host, chunk)?;
                failed |= res["ok"].as_bool() != Some(true);
                if !cli.json {
                    println!(
                        "{} entries: {}",
                        chunk.len(),
                        if res["ok"].as_bool() == Some(true) {
                            res["outcome"].to_string()
                        } else {
                            res["error"].to_string()
                        }
                    );
                }
                results.push(res);
            }
            if cli.json {
                println!(
                    "{}",
                    json!({"ok": !failed, "entries": lines.len(), "batches": results})
                );
            }
            if failed {
                std::process::exit(4);
            }
        }
        Cmd::Unhost {
            region: id,
            cid,
            rpc,
            program,
            host,
            chain_bin,
        } => {
            let def = region(id)?;
            osm_core::entry::validate_cid(cid).map_err(|_| SdkError::InvalidInput {
                field: "--cid",
                why: format!("{cid:?} is not a CID"),
            })?;
            let out = Command::new(chain_bin)
                .args([
                    "unhost",
                    "--rpc",
                    rpc,
                    "--program",
                    program,
                    "--host",
                    host,
                    "--region",
                    def.region,
                    "--cid",
                    cid,
                ])
                .output()
                .map_err(|e| SdkError::Io {
                    path: chain_bin.clone(),
                    source: e,
                })?;
            let stdout = String::from_utf8_lossy(&out.stdout);
            let last = stdout
                .lines()
                .rev()
                .find(|l| l.trim_start().starts_with('{'))
                .unwrap_or("{}");
            let res: serde_json::Value = serde_json::from_str(last).unwrap_or_default();
            let ok = res["ok"].as_bool() == Some(true);
            if cli.json {
                println!("{res}");
            } else {
                println!(
                    "{}",
                    if ok {
                        format!("unhosted: {}", res["outcome"])
                    } else {
                        format!("NOT unhosted: {}", res["error"])
                    }
                );
            }
            if !ok {
                std::process::exit(4);
            }
        }
        Cmd::VerifyCoverage {
            storage,
            rpc,
            program,
            dir,
            newest_only,
        } => {
            osm_sdk::datadir::ensure_private_dir(dir)?;
            let reg = Registry {
                source: RpcRegistry::new(rpc, program)?,
                program: program.clone(),
            };
            let mut store = RestStorage::new(storage, std::time::Duration::from_secs(600));
            let g = geofabrik(cli)?;
            let r = osm_sdk::coverage::verify_coverage(&reg, &mut store, &g, dir, *newest_only);
            let status = |s: &osm_sdk::coverage::EntryStatus| -> String {
                use osm_sdk::coverage::EntryStatus::*;
                match s {
                    Verified => "verified".into(),
                    PublishedMd5Differs { published } => {
                        format!("bytes match the registry but Geofabrik publishes {published}")
                    }
                    BytesMismatch { got } => {
                        format!("bytes hash to {got}, not the registered checksum")
                    }
                    SnapshotGone => "Geofabrik no longer serves this snapshot's .md5".into(),
                    NotRetrievable(why) => format!("not retrievable: {why}"),
                }
            };
            if cli.json {
                let rows: Vec<_> = r.entries.iter().map(|e| json!({"region": e.region, "cid": e.cid, "source_url": e.source_url, "status": status(&e.status), "verified": e.status == osm_sdk::coverage::EntryStatus::Verified})).collect();
                println!(
                    "{}",
                    json!({"pass": r.passes(), "verified_regions": r.verified_regions.len(), "countries": r.countries.len(),
                    "min_entries": osm_sdk::coverage::MIN_ENTRIES, "min_countries": osm_sdk::coverage::MIN_COUNTRIES, "entries": rows, "errors": r.errors})
                );
            } else {
                for e in &r.entries {
                    println!("{:<34} {} {}", e.region, e.cid, status(&e.status));
                }
                for e in &r.errors {
                    println!("registry error: {e}");
                }
                println!(
                    "verified regions: {} (need {}), countries: {} (need {}) -> {}",
                    r.verified_regions.len(),
                    osm_sdk::coverage::MIN_ENTRIES,
                    r.countries.len(),
                    osm_sdk::coverage::MIN_COUNTRIES,
                    if r.passes() { "PASS" } else { "FAIL" }
                );
            }
            if !r.passes() {
                std::process::exit(10);
            }
        }
        Cmd::Download {
            region: id,
            dir,
            storage,
            rpc,
            program,
        } => {
            let def = region(id)?;
            osm_sdk::datadir::ensure_private_dir(dir)?;
            let reg = Registry {
                source: RpcRegistry::new(rpc, program)?,
                program: program.clone(),
            };
            let mut store = RestStorage::new(storage, std::time::Duration::from_secs(120));
            let g = (!cli.no_geofabrik).then(|| make_geofabrik(cli));
            let f = osm_sdk::fetch::fetch_region(&reg, &mut store, g.as_ref(), def.region, dir)?;
            let (kind, from) = match &f.source {
                osm_sdk::fetch::Source::Storage { cid } => ("storage", cid.clone()),
                osm_sdk::fetch::Source::Geofabrik { url } => ("geofabrik", url.clone()),
            };
            if cli.json {
                let skipped: Vec<_> = f
                    .skipped
                    .iter()
                    .map(|(c, why)| json!({"cid": c, "why": why}))
                    .collect();
                println!(
                    "{}",
                    json!({"ok": true, "region": def.region, "path": f.path, "source": kind, "from": from, "md5": to_hex(&f.md5), "skipped": skipped})
                );
            } else {
                for (c, why) in &f.skipped {
                    eprintln!("warning: skipped {c}: {why}");
                }
                println!(
                    "{} -> {} (from {kind} {from}, MD5 {} verified)",
                    def.region,
                    f.path.display(),
                    to_hex(&f.md5)
                );
            }
        }
        Cmd::UpdateCheck {
            regions,
            rpc,
            program,
        } => {
            let reg = Registry {
                source: RpcRegistry::new(rpc, program)?,
                program: program.clone(),
            };
            let g = geofabrik(cli)?;
            let names: Vec<String> = if regions.is_empty() {
                let (s, errors) = osm_sdk::status::hosted_status(&reg);
                for e in &errors {
                    eprintln!("warning: {e}");
                }
                s.into_iter()
                    .filter(|x| x.version.is_some())
                    .map(|x| x.region.to_string())
                    .collect()
            } else {
                regions.clone()
            };
            let refs: Vec<&str> = names.iter().map(String::as_str).collect();
            let u = osm_sdk::status::check_updates(&reg, &g, &refs)?;
            if cli.json {
                let rows: Vec<_> = u.iter().map(|x| json!({"region": x.region, "registered": x.registered, "available": x.available, "update_available": x.update_available()})).collect();
                println!("{}", json!({"regions": rows}));
            } else if u.is_empty() {
                println!("no hosted regions to check");
            } else {
                for x in &u {
                    let reg_s = x
                        .registered
                        .map(|v| format!("{v:06}"))
                        .unwrap_or_else(|| "none".into());
                    let note = if x.update_available() {
                        "update available"
                    } else {
                        "up to date"
                    };
                    println!(
                        "{:<34} registered {reg_s}  Geofabrik {:06}  {note}",
                        x.region, x.available
                    );
                }
            }
        }
        Cmd::Lookup {
            region: by_region,
            parent,
            cid,
            rpc,
            program,
        } => {
            let reg = Registry {
                source: RpcRegistry::new(rpc, program)?,
                program: program.clone(),
            };
            let (entries, errors) = match (by_region, parent, cid) {
                (Some(r), _, _) => {
                    let def = region(r)?;
                    let acc = reg.region(def.region)?;
                    let v = acc
                        .map(|a| {
                            a.history
                                .into_iter()
                                .map(|entry| RegisteredEntry {
                                    region: def.region,
                                    entry,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    (v, Vec::new())
                }
                (_, Some(p), _) => reg.by_parent(p),
                (_, _, Some(c)) => reg.by_cid(c),
                _ => {
                    return Err(SdkError::InvalidInput {
                        field: "lookup",
                        why: "give --region, --parent or --cid".into(),
                    })
                }
            };
            for e in &errors {
                eprintln!("warning: {e}");
            }
            if cli.json {
                let rows: Vec<_> = entries.iter().map(entry_json).collect();
                let errs: Vec<_> = errors.iter().map(|e| e.to_string()).collect();
                println!("{}", json!({"entries": rows, "errors": errs}));
            } else if entries.is_empty() {
                println!("no registry entries");
            } else {
                for r in &entries {
                    let e = &r.entry;
                    println!(
                        "{} {} {:06} md5={} size={} hosts={} by={}",
                        r.region,
                        e.cid,
                        e.version,
                        to_hex(&e.checksum),
                        e.size_bytes,
                        e.hosts.len(),
                        e.registered_by_b58()
                    );
                }
            }
        }
        Cmd::VerifyFile {
            file,
            region: id,
            snapshot,
        } => {
            let def = region(id)?;
            let url = snapshot_url(def, *snapshot)?;
            let want = geofabrik(cli)?.snapshot_md5(def, *snapshot)?;
            verify_import(file, &want, &format!("{}-{snapshot:06}", def.region))?;
            if cli.json {
                println!(
                    "{}",
                    json!({"ok": true, "region": def.region, "source_url": url, "md5": to_hex(&want)})
                );
            } else {
                println!(
                    "ok: {} matches Geofabrik's MD5 {} for {url}",
                    file.display(),
                    to_hex(&want)
                );
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if cli.json {
                println!(
                    "{}",
                    json!({"ok": false, "code": e.code(), "error": e.to_string()})
                );
            } else {
                eprintln!("error: {e}");
            }
            ExitCode::from(exit_for(&e))
        }
    }
}
