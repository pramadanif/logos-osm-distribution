//! `osm`: command line for the LP-0018 OSM registry. Commands that need the chain are added with
//! the LEZ adapter; everything here works without it.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use osm_core::{lookup, RegionDef, PREDEFINED};
use osm_sdk::geofabrik::{discover, Geofabrik, Source};
use osm_sdk::hash::{to_hex, verify_import};
use osm_sdk::registry::{RegisteredEntry, Registry, RpcRegistry};
use osm_sdk::snapshot::snapshot_url;
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
        "S130" | "S131" | "S132" => 7,
        "S150" | "S151" | "S152" => 9,
        "S100" | "S170" => 3,
        "S160" => 2,
        _ => 1,
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
    Ok(Geofabrik::new())
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
            let g = (!cli.no_geofabrik).then(Geofabrik::new);
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
