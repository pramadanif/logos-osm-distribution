//! `osm-chain register`: confirmed registry writes on LEZ v0.3 (one entry or a batch).
//!
//!     LEE_WALLET_HOME_DIR=<wallet> osm-chain register --rpc <url> --program <id> --host <id> \
//!         --entry '<region>|<cid>|<source_url>|<md5>|<YYMMDD>|<size>' [--entry ...]
//!
//! Prints one JSON object. Exit 0 = confirmed or already registered; 2 = invalid input;
//! 3 = RPC/registry; 4 = transaction not included / rejected / wallet.
use lez_adapter_v03::{Adapter, AdapterError, NewEntry, Outcome};

fn arg(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone())
}

fn parse_entry(s: &str) -> Result<NewEntry, String> {
    let p: Vec<&str> = s.split('|').collect();
    if p.len() != 6 {
        return Err(format!("--entry needs 6 '|' separated fields, got {}", p.len()));
    }
    Ok(NewEntry {
        region: p[0].into(),
        cid: p[1].into(),
        source_url: p[2].into(),
        checksum: p[3].into(),
        version: p[4].parse().map_err(|_| format!("version {:?} is not a number", p[4]))?,
        size_bytes: p[5].parse().map_err(|_| format!("size {:?} is not a number", p[5]))?,
    })
}

fn exit_code(e: &AdapterError) -> i32 {
    match e.code() {
        "S160" => 2,
        "S100" | "S170" => 3,
        _ => 4,
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let fail = |code: i32, kind: &str, msg: String| -> ! {
        println!("{}", serde_json_lite(&[("ok", "false".into()), ("code", format!("\"{kind}\"")), ("error", format!("{msg:?}"))]));
        std::process::exit(code)
    };
    if args.get(1).map(String::as_str) != Some("register") {
        fail(2, "S160", "usage: osm-chain register --rpc <url> --program <id> --host <id> --entry '<region>|<cid>|<url>|<md5>|<YYMMDD>|<size>'...".into());
    }
    let (Some(rpc), Some(program), Some(host)) = (arg(&args, "--rpc"), arg(&args, "--program"), arg(&args, "--host")) else {
        fail(2, "S160", "--rpc, --program and --host are required".into())
    };
    let entries: Result<Vec<_>, _> = args.windows(2).filter(|w| w[0] == "--entry").map(|w| parse_entry(&w[1])).collect();
    let entries = entries.unwrap_or_else(|m| fail(2, "S160", m));
    let adapter = Adapter::from_env(&rpc, &program).await.unwrap_or_else(|e| fail(exit_code(&e), e.code(), e.to_string()));
    match adapter.register(host.trim_start_matches("Public/"), &entries).await {
        Ok(Outcome::Confirmed { tx_hash }) => println!(
            "{}",
            serde_json_lite(&[("ok", "true".into()), ("outcome", "\"confirmed\"".into()), ("tx_hash", format!("\"{tx_hash}\""))])
        ),
        Ok(Outcome::AlreadyRegistered) => println!(
            "{}",
            serde_json_lite(&[("ok", "true".into()), ("outcome", "\"already_registered\"".into())])
        ),
        Err(e) => fail(exit_code(&e), e.code(), e.to_string()),
    }
}

/// Minimal JSON object writer; values are already JSON literals.
fn serde_json_lite(kv: &[(&str, String)]) -> String {
    let body: Vec<String> = kv.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
    format!("{{{}}}", body.join(","))
}
