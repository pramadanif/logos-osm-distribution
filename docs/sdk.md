# OSM registry SDK

`osm_sdk` lets a Logos module find OpenStreetMap extracts that people host on Logos Storage,
resolve a region to its content id (CID) and metadata, download it, and check it. It reads the
on-chain registry over the sequencer's JSON-RPC and talks to a Storage node over its REST API.

Status: alpha, LEZ v0.3 only. The Basecamp module wrapping this API is not built yet; see
[limitations.md](limitations.md).

## Worked example: region → CID + metadata → verified file

This is `osm_sdk/examples/resolve.rs`, which CI compiles on every push.

```rust
use osm_sdk::fetch::fetch_region;
use osm_sdk::hash::to_hex;
use osm_sdk::registry::{Registry, RpcRegistry};
use osm_sdk::rest_storage::RestStorage;
use std::time::Duration;

// 1. A registry is a sequencer RPC URL plus the registry program id.
let registry = Registry { source: RpcRegistry::new(rpc, program)?, program: program.into() };

// 2. Resolve a region to the CID and metadata of its newest hosted entry.
if let Some(r) = registry.resolve("china/henan")? {
    println!("{} -> {} snapshot {:06} md5 {} {} bytes, {} hosts",
             r.region, r.cid, r.version, to_hex(&r.checksum), r.size_bytes, r.hosts);
}

// 3. Download by CID and verify the bytes against the registered MD5.
//    `None` means: never fall back to Geofabrik.
let mut node = RestStorage::new(storage_url, Duration::from_secs(120));
let f = fetch_region(&registry, &mut node, None, "china/henan", out_dir)?;
println!("saved {} (MD5 {} verified)", f.path.display(), to_hex(&f.md5));
```

Run it against a sequencer and a Storage node:

    cargo run -p osm_sdk --example resolve -- <rpc-url> <program-id> china/henan <storage-api-url> <out-dir>

Output on the local v0.3.1 test network (2026-11-09):

    china/henan -> zDvZRwzmA5LrY5Be279q4gpJqxk6v6XrS59rHkKakguRwwFPEMfH (snapshot 261101, MD5 5f3c041353b4886c9984cf6e7724fd10, 50702303 bytes, 1 hosts)
    saved …/china_henan-261101.osm.pbf (MD5 5f3c041353b4886c9984cf6e7724fd10 verified)

`5f3c0413…` is Geofabrik's published MD5 for `henan-261101.osm.pbf`.

## API

The consumer API below is what other modules should depend on. It is the subset the prize allows
for consumers: it needs no wallet and never writes to the chain.

| Call | What it returns | Network |
|------|-----------------|---------|
| `discovery::discover_offline()` | the 72 predefined regions with parent and level | none |
| `geofabrik::discover(Some(&Geofabrik::new()))` | same, with Geofabrik's names; falls back to offline and says why | Geofabrik (optional) |
| `Registry::resolve(region)` | newest hosted entry: CID, MD5, snapshot, source URL, size, hosts, timestamp | sequencer RPC |
| `Registry::region / by_parent / by_cid / all` | registry entries, sorted by timestamp | sequencer RPC |
| `fetch::fetch_region(..)` | a verified local file, and which entries were skipped and why | Storage; Geofabrik only if unhosted and allowed |
| `status::hosted_status(&registry)` | hosted snapshot and host count for all 72 regions | sequencer RPC |
| `status::check_updates(&registry, &geofabrik, regions)` | registered vs current Geofabrik snapshot per region | sequencer RPC, Geofabrik |

The full API used by the distribution app adds hosting: `geofabrik::Geofabrik::download_snapshot`
(dated snapshot, resumable, MD5-checked), `storage::upload_with_retry` (exponential back-off with
jitter), and confirmed registration through `lez_adapter_v03` (`osm-chain register`).

## Trust rules the SDK applies for you

- Registry data is written through the program but is still treated as untrusted. Every account
  is size-checked and decoded with bounds; region, parent and level must match the predefined set.
- A download counts only if its MD5 equals the registered checksum. Newer entries that do not
  verify are skipped and their files deleted, so a junk entry cannot hide a good one.
- Hosted regions never need Geofabrik. Geofabrik is used for discovery names, update checks and
  as a fallback for unhosted regions, and can be turned off (`None`).
- Region names from Geofabrik are untrusted text: control and bidi characters are removed and the
  length is capped; render them as plain text.

## Errors

Every error has a stable code (`SdkError::code()`), listed with its user message in
[error-codes.md](error-codes.md). For example `S140` region not hosted, `S141` hosted but nothing
verified, `S100` sequencer unreachable, `S110`/`S111` Storage unreachable or upload retries used up.

## Using it from another crate

Until a crates.io release exists, depend on the git repository at a pinned commit:

```toml
osm_sdk = { git = "https://github.com/pramadanif/logos-osm-distribution", rev = "<commit>" }
```
