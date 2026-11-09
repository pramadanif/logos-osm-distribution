//! Worked example (docs/sdk.md): resolve a region to its CID and metadata, then download and
//! verify it.
//!
//!     cargo run -p osm_sdk --example resolve -- <rpc-url> <program-id> <region> <storage-url> <out-dir>
use osm_sdk::fetch::fetch_region;
use osm_sdk::hash::to_hex;
use osm_sdk::registry::{Registry, RpcRegistry};
use osm_sdk::rest_storage::RestStorage;
use std::time::Duration;

fn main() -> Result<(), osm_sdk::SdkError> {
    let a: Vec<String> = std::env::args().collect();
    let [_, rpc, program, region, storage, out] = &a[..] else {
        eprintln!("usage: resolve <rpc-url> <program-id> <region> <storage-url> <out-dir>");
        std::process::exit(2);
    };

    // 1. Point the SDK at a registry: a sequencer RPC and the registry program id.
    let registry = Registry {
        source: RpcRegistry::new(rpc, program)?,
        program: program.clone(),
    };

    // 2. Resolve the region to CID + metadata.
    let Some(r) = registry.resolve(region)? else {
        println!("{region} is not hosted yet");
        return Ok(());
    };
    println!(
        "{} -> {} (snapshot {:06}, MD5 {}, {} bytes, {} hosts)",
        r.region,
        r.cid,
        r.version,
        to_hex(&r.checksum),
        r.size_bytes,
        r.hosts
    );

    // 3. Download it by CID and verify the bytes against the registered MD5.
    //    `None` = never fall back to Geofabrik (hosted regions do not need it).
    let mut node = RestStorage::new(storage, Duration::from_secs(120));
    std::fs::create_dir_all(out).map_err(|e| osm_sdk::SdkError::InvalidInput {
        field: "out-dir",
        why: e.to_string(),
    })?;
    let f = fetch_region(&registry, &mut node, None, region, out.as_ref())?;
    println!(
        "saved {} (MD5 {} verified)",
        f.path.display(),
        to_hex(&f.md5)
    );
    Ok(())
}
