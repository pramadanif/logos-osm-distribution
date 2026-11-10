//! `osm-chain-v04`: confirmed registry writes on LEZ 0.4 (actor registry, no SPEL yet), with the
//! same command line and answers as the v0.3 `osm-chain`, so `osm host|bulk-register|import|unhost
//! --chain-bin osm-chain-v04` works unchanged.
//!
//!     LEE_WALLET_HOME_DIR=<wallet> LEE_WALLET_PASSWORD=… osm-chain-v04 register --rpc <url> \
//!         --program <id> --host <id> --entry '<region>|<cid>|<source_url>|<md5>|<YYMMDD>|<size>' [--entry ...]
//!     LEE_WALLET_HOME_DIR=<wallet> osm-chain-v04 unhost --rpc <url> --program <id> --host <id> \
//!         --region <region> --cid <cid>
//!     osm-chain-v04 region-account <program> <region>
//!     osm-chain-v04 state --rpc <url> --program <id> --region <region>
//!
//! For the negative probes only (probes/b5_negative_v04.sh), two commands send a message without
//! the client's checks and without waiting, so the program's own checks are what is tested:
//!
//!     osm-chain-v04 probe-register --rpc … --program … --host … --region … --parent … --level … \
//!         --cid … --source-url … --md5 … --version … --size … --timestamp <ms>
//!     osm-chain-v04 probe-direct-apply --rpc … --program … --host … --region … --cid …
//!
//! `probe-direct-apply` sends `Apply` straight to the region actor, skipping the host actor: the
//! program must refuse it (a region actor accepts messages only from this program's actors).
//!
//! One `--entry` sends `Register`, several send one `RegisterBatch`. The transaction goes to the
//! host's own actor under the program (the host signs); the program forwards each entry to its
//! region actor. Every actor the transaction touches is declared. After sending, the region
//! accounts are read back until the change is visible (H33).
//!
//! Prints one JSON object. Exit 0 = confirmed or already registered; 2 = invalid input;
//! 3 = RPC/registry; 4 = transaction not included / rejected / wallet.
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use borsh::BorshSerialize;
use lee_core::account::AccountId;
use lee_core::program::PdaSeed;
use osm_sdk::registry::{RegionAccount, Registry, RpcRegistry};
use sha2::{Digest, Sha256};
use wallet::{AccountIdentity, AccountMention};

/// Wire format of the program's `Msg` (variant order matters; internal variants are never sent
/// by a transaction but keep their place).
#[derive(BorshSerialize)]
enum Msg {
    Register { region: String, parent: String, level: u8, cid: String, source_url: String, checksum: String, version: u32, size_bytes: u64, timestamp: u64 },
    Unhost { region: String, cid: String },
    Apply { region: String, parent: String, level: u8, entry: osm_sdk::registry::Entry },
    #[allow(dead_code)]
    Remove,
    RegisterBatch { entries: Vec<NewEntry>, timestamp: u64 },
}

#[derive(BorshSerialize, Clone)]
struct NewEntry {
    region: String,
    parent: String,
    level: u8,
    cid: String,
    source_url: String,
    checksum: String,
    version: u32,
    size_bytes: u64,
}

/// How long to wait for the change to show up in the region accounts.
const CONFIRM_TIMEOUT: Duration = Duration::from_secs(120);

struct Failure {
    exit: i32,
    code: &'static str,
    message: String,
}

fn invalid(m: impl Into<String>) -> Failure {
    Failure { exit: 2, code: "S160", message: m.into() }
}

fn account(b58: &str) -> Result<AccountId, Failure> {
    let b: [u8; 32] = bs58::decode(b58.trim_start_matches("Public/"))
        .into_vec()
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| invalid(format!("{b58:?} is not a base58 account id")))?;
    Ok(AccountId::new(b))
}

fn region_seed(region: &str) -> PdaSeed {
    let mut both = [0u8; 64];
    both[..10].copy_from_slice(b"osm-region");
    both[32..32 + region.len()].copy_from_slice(region.as_bytes());
    PdaSeed::new(Sha256::digest(both).into())
}

fn mention(identity: AccountIdentity, program: AccountId, authorizes: bool) -> AccountMention {
    AccountMention { identity, program_account_id: program, authorizes, address_openings: BTreeSet::new() }
}

fn arg(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone())
}

/// `<region>|<cid>|<source_url>|<md5>|<YYMMDD>|<size>`, checked with the program's own rules.
fn parse_entry(s: &str, now_ms: u64) -> Result<NewEntry, Failure> {
    let p: Vec<&str> = s.split('|').collect();
    if p.len() != 6 {
        return Err(invalid(format!("--entry needs 6 '|' separated fields, got {}", p.len())));
    }
    let def = osm_core::lookup(p[0]).ok_or_else(|| invalid(format!("{:?} is not in the predefined set", p[0])))?;
    let version: u32 = p[4].parse().map_err(|_| invalid(format!("version {:?} is not a number", p[4])))?;
    let size_bytes: u64 = p[5].parse().map_err(|_| invalid(format!("size {:?} is not a number", p[5])))?;
    osm_core::entry::validate_entry(&osm_core::entry::EntryInput {
        region: def.region,
        parent: def.parent,
        level: def.level,
        cid: p[1],
        source_url: p[2],
        checksum_hex: p[3],
        version,
        size_bytes,
        timestamp_ms: now_ms,
    })
    .map_err(|e| invalid(format!("{} {e:?}", e.code())))?;
    Ok(NewEntry {
        region: def.region.into(),
        parent: def.parent.unwrap_or("").into(),
        level: u8::from(def.level == osm_core::Level::Subregion),
        cid: p[1].into(),
        source_url: p[2].into(),
        checksum: p[3].into(),
        version,
        size_bytes,
    })
}

fn registry(rpc: &str, program: &str) -> Result<Registry<RpcRegistry>, Failure> {
    let source = RpcRegistry::new(rpc, program).map_err(|e| Failure { exit: 3, code: e.code(), message: e.to_string() })?;
    Ok(Registry { source, program: program.to_string() })
}

fn read(reg: &Registry<RpcRegistry>, region: &str) -> Result<Option<RegionAccount>, Failure> {
    reg.region(region).map_err(|e| Failure { exit: 3, code: e.code(), message: e.to_string() })
}

fn hosts(acc: &Option<RegionAccount>, cid: &str, host: &[u8; 32]) -> bool {
    acc.as_ref().is_some_and(|a| a.history.iter().any(|e| e.cid == cid && e.hosts.contains(host)))
}

/// Polls until `done(region)` holds for every region, or fails with S103.
fn confirm(reg: &Registry<RpcRegistry>, regions: &[String], tx: &str, done: impl Fn(&str, &Option<RegionAccount>) -> bool) -> Result<(), Failure> {
    let start = Instant::now();
    loop {
        let mut all = true;
        for r in regions {
            if !done(r, &read(reg, r)?) {
                all = false;
                break;
            }
        }
        if all {
            return Ok(());
        }
        if start.elapsed() > CONFIRM_TIMEOUT {
            return Err(Failure {
                exit: 4,
                code: "S103",
                message: format!("TxNotIncluded: sent {tx} but the registry shows no change after {} s", CONFIRM_TIMEOUT.as_secs()),
            });
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

async fn send(accounts: Vec<AccountMention>, msg: &Msg) -> Result<String, Failure> {
    let wallet_err = |m: String| Failure { exit: 4, code: "S104", message: m };
    let w = wallet::WalletCore::from_env().map_err(|e| wallet_err(format!("wallet: {e:?}")))?;
    let bytes = borsh::to_vec(msg).expect("message serializes");
    let tx = w.send_pub_tx(accounts, 0, bytes).await.map_err(|e| wallet_err(format!("transaction rejected: {e:?}")))?;
    Ok(tx.as_ref().iter().map(|x| format!("{x:02x}")).collect())
}

fn clock_mention() -> AccountMention {
    mention(AccountIdentity::PublicNoSign(clock_core::CLOCK_01_PROGRAM_ACCOUNT_ID), clock_core::clock_account_id(), false)
}

async fn run(args: &[String]) -> Result<String, Failure> {
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    if cmd == "region-account" {
        let (Some(p), Some(r)) = (args.get(2), args.get(3)) else { return Err(invalid("region-account <program> <region>")) };
        let program = account(p)?;
        let id = AccountId::for_public_pda(&program, &region_seed(r));
        return Ok(format!("{{\"ok\":true,\"account\":\"{}\"}}", bs58::encode(id.value()).into_string()));
    }
    if cmd == "state" {
        let (Some(rpc), Some(program), Some(region)) = (arg(args, "--rpc"), arg(args, "--program"), arg(args, "--region")) else {
            return Err(invalid("state --rpc <url> --program <id> --region <region>"));
        };
        let reg = registry(&rpc, &program)?;
        if osm_core::lookup(&region).is_none() {
            // Outside the set the SDK refuses to look; read the would-be PDA directly so the
            // probes can show that nothing was ever written there.
            use osm_sdk::registry::RegistrySource;
            let id = AccountId::for_public_pda(&account(&program)?, &region_seed(&region));
            let raw = reg.source.fetch(&bs58::encode(id.value()).into_string()).map_err(|e| Failure { exit: 3, code: e.code(), message: e.to_string() })?;
            return Ok(match raw {
                Some(b) if !b.is_empty() => format!("{{\"unexpected_bytes\":{}}}", b.len()),
                _ => "{}".into(),
            });
        }
        let acc = read(&reg, &region)?;
        let v = match acc {
            None => serde_json::json!({}),
            Some(a) => serde_json::json!({
                "region": a.region, "parent": a.parent, "level": a.level,
                "history": a.history.iter().map(|e| serde_json::json!({
                    "cid": e.cid, "source_url": e.source_url, "version": e.version, "size_bytes": e.size_bytes,
                    "hosted": e.hosted, "timestamp": e.timestamp,
                    "registered_by": e.registered_by_b58(), "hosts": e.hosts_b58(),
                })).collect::<Vec<_>>(),
            }),
        };
        return Ok(v.to_string());
    }
    if cmd == "probe-register" || cmd == "probe-direct-apply" {
        return probe(cmd, args).await;
    }
    if cmd != "register" && cmd != "unhost" {
        return Err(invalid("usage: osm-chain-v04 register --rpc <url> --program <id> --host <id> --entry '<region>|<cid>|<url>|<md5>|<YYMMDD>|<size>'... | unhost --rpc <url> --program <id> --host <id> --region <r> --cid <cid> | region-account <program> <region>"));
    }
    let (Some(rpc), Some(program_b58), Some(host_b58)) = (arg(args, "--rpc"), arg(args, "--program"), arg(args, "--host")) else {
        return Err(invalid("--rpc, --program and --host are required"));
    };
    let program = account(&program_b58)?;
    let host = account(&host_b58)?;
    let host_bytes = *host.value();
    let reg = registry(&rpc, &program_b58)?;
    let region_mention = |r: &str| mention(AccountIdentity::PublicNoSign(AccountId::for_public_pda(&program, &region_seed(r))), program, false);

    if cmd == "unhost" {
        let (Some(region), Some(cid)) = (arg(args, "--region"), arg(args, "--cid")) else { return Err(invalid("--region and --cid are required")) };
        osm_core::lookup(&region).ok_or_else(|| invalid(format!("{region:?} is not in the predefined set")))?;
        osm_core::entry::validate_cid(&cid).map_err(|_| invalid(format!("{cid:?} is not a CID")))?;
        if !hosts(&read(&reg, &region)?, &cid, &host_bytes) {
            return Ok("{\"ok\":true,\"outcome\":\"not_hosted\"}".into());
        }
        let accounts = vec![mention(AccountIdentity::Public(host), program, true), region_mention(&region)];
        let tx = send(accounts, &Msg::Unhost { region: region.clone(), cid: cid.clone() }).await?;
        confirm(&reg, &[region], &tx, |_, acc| !hosts(acc, &cid, &host_bytes))?;
        return Ok(format!("{{\"ok\":true,\"outcome\":\"confirmed\",\"tx_hash\":\"{tx}\"}}"));
    }

    let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0);
    let entries: Vec<NewEntry> = args.windows(2).filter(|w| w[0] == "--entry").map(|w| parse_entry(&w[1], now_ms)).collect::<Result<_, _>>()?;
    let names: Vec<&str> = entries.iter().map(|e| e.region.as_str()).collect();
    osm_core::entry::validate_batch_shape(&names, &[]).map_err(|e| invalid(format!("{} {e:?}", e.code())))?;
    // Skip what this host already hosts, as the v0.3 adapter does.
    let mut todo = Vec::new();
    for e in entries {
        if !hosts(&read(&reg, &e.region)?, &e.cid, &host_bytes) {
            todo.push(e);
        }
    }
    if todo.is_empty() {
        return Ok("{\"ok\":true,\"outcome\":\"already_registered\"}".into());
    }
    let mut accounts = vec![mention(AccountIdentity::Public(host), program, true)];
    accounts.extend(todo.iter().map(|e| region_mention(&e.region)));
    accounts.push(clock_mention());
    let msg = if todo.len() == 1 {
        let e = todo[0].clone();
        Msg::Register { region: e.region, parent: e.parent, level: e.level, cid: e.cid, source_url: e.source_url, checksum: e.checksum, version: e.version, size_bytes: e.size_bytes, timestamp: now_ms }
    } else {
        Msg::RegisterBatch { entries: todo.clone(), timestamp: now_ms }
    };
    let tx = send(accounts, &msg).await?;
    let regions: Vec<String> = todo.iter().map(|e| e.region.clone()).collect();
    confirm(&reg, &regions, &tx, |r, acc| todo.iter().filter(|e| e.region == r).all(|e| hosts(acc, &e.cid, &host_bytes)))?;
    Ok(format!("{{\"ok\":true,\"outcome\":\"confirmed\",\"tx_hash\":\"{tx}\",\"entries\":{}}}", regions.len()))
}

/// Sends one unchecked message and prints its transaction hash; the caller reads the state.
async fn probe(cmd: &str, args: &[String]) -> Result<String, Failure> {
    let need = |k: &str| arg(args, k).ok_or_else(|| invalid(format!("{k} is required")));
    let program = account(&need("--program")?)?;
    let host = account(&need("--host")?)?;
    let region = need("--region")?;
    let num = |k: &str| -> Result<u64, Failure> { need(k)?.parse().map_err(|_| invalid(format!("{k} is not a number"))) };
    let region_id = AccountId::for_public_pda(&program, &region_seed(&region));
    let tx = if cmd == "probe-register" {
        let msg = Msg::Register {
            region: region.clone(),
            parent: need("--parent").unwrap_or_default(),
            level: num("--level")? as u8,
            cid: need("--cid")?,
            source_url: need("--source-url")?,
            checksum: need("--md5")?,
            version: num("--version")? as u32,
            size_bytes: num("--size")?,
            timestamp: num("--timestamp")?,
        };
        let accounts = vec![
            mention(AccountIdentity::Public(host), program, true),
            mention(AccountIdentity::PublicNoSign(region_id), program, false),
            clock_mention(),
        ];
        send(accounts, &msg).await?
    } else {
        // A forged entry naming the signer, sent to the region actor directly.
        let h = *host.value();
        let entry = osm_sdk::registry::Entry {
            cid: need("--cid")?,
            source_url: format!("https://download.geofabrik.de/{region}-261101.osm.pbf"),
            checksum: [0; 16],
            version: 261101,
            size_bytes: 1,
            hosted: true,
            timestamp: 0,
            registered_by: h,
            hosts: vec![h],
        };
        // The region actor receives; the host signs, so the message is authorized but has no
        // sender actor of this program.
        let accounts = vec![
            mention(AccountIdentity::PublicNoSign(region_id), program, false),
            mention(AccountIdentity::Public(host), program, true),
        ];
        send(accounts, &Msg::Apply { region: region.clone(), parent: String::new(), level: 0, entry }).await?
    };
    Ok(format!("{{\"ok\":true,\"outcome\":\"sent\",\"tx_hash\":\"{tx}\"}}"))
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    match run(&args).await {
        Ok(json) => println!("{json}"),
        Err(f) => {
            println!("{{\"ok\":false,\"code\":\"{}\",\"error\":{:?}}}", f.code, f.message);
            std::process::exit(f.exit);
        }
    }
}
