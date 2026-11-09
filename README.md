# logos-osm-distribution

OpenStreetMap extracts distributed over the Logos stack, built for
[λPrize LP-0018](https://github.com/logos-co/lambda-prize/blob/master/prizes/LP-0018.md):
an on-chain registry of map snapshots on the Logos Execution Zone (LEZ), the files themselves in
Logos Storage, a Rust SDK with a C ABI, and a command line tool.

**Status (alpha).** Runs on LEZ v0.3 (local standalone sequencer and the public v0.3 testnet).
Testnet 0.4, the network that counts for the prize, is not live yet and the 0.4 port is not done.
The Basecamp app and modules are not built yet. See [docs/limitations.md](docs/limitations.md).

## What it does

- **Discover** the 72 regions of the prize's closed, non-overlapping set, with Geofabrik's names
  and each region's hosted status from the registry.
- **Host**: download a dated Geofabrik snapshot, check its MD5, upload it to Logos Storage,
  register it on-chain. Several regions go into one batch transaction.
- **Import** a local PBF after checking it against Geofabrik's published MD5.
- **Look up** entries by region, parent or CID; **check for updates** per region.
- **Download** a region from Logos Storage by CID, verified against the registered MD5; Geofabrik
  is used only when nobody hosts the region.
- **Verify coverage** the way the prize evaluator does.

## Deployments

| Network | Program id | ImageID |
|---------|------------|---------|
| LEZ v0.3 public testnet (`https://testnet.lez.logos.co`), rehearsal only | `5VZeCZKSzkQyYb2nPm4ShDAKpkwncvbQh58XSg4cCAcS` | `f329edb220bdaade9d69250bb36969b95b672b4ea9d44c7a0092557762305e85` |
| Logos testnet 0.4 | not deployed yet | |

Each region lives in its own account derived from the program id and the region id
(`["osm-region", region]`); there is no global registry account ([ADR 0001](docs/adr/0001-registry-layout.md)).
`networks.json` holds the profiles the tools read.

## Build and test

Needs a Rust toolchain (rustup; `rust-toolchain.toml` pins the version) and a C compiler.

    cargo test --locked

Building the on-chain program and the write path additionally needs Docker, `cargo-risczero`
3.0.5 and the RISC Zero Rust toolchain (`rzup install rust 1.97.0`); see
[scripts/ci-e2e.sh](scripts/ci-e2e.sh), which builds and runs everything from scratch.

## Command line

Build the CLI: `cargo build --release -p osm_cli` (binary `target/release/osm`). Read-only commands
need no wallet:

    osm regions                                   # the 72 predefined regions
    osm discover --rpc <rpc> --program <id>       # names, levels, hosted status
    osm snapshot indonesia                        # today's dated snapshot and Geofabrik's MD5
    osm lookup --region china/henan --rpc <rpc> --program <id>
    osm lookup --parent india   --rpc <rpc> --program <id>
    osm lookup --cid <cid>      --rpc <rpc> --program <id>
    osm update-check --rpc <rpc> --program <id>
    osm download --region china/henan --dir ./maps --storage <storage-api> --rpc <rpc> --program <id>
    osm verify-coverage --dir ./tmp --storage <storage-api> --rpc <rpc> --program <id>

Writing needs a funded LEZ wallet (`LEE_WALLET_HOME_DIR`) and the `osm-chain` binary
(`cargo build --release -p lez_adapter_v03 --bin osm-chain` in `programs/osm_registry`):

    osm host --region kenya --region egypt --cache-dir ./cache --storage <storage-api> \
             --rpc <rpc> --program <id> --host <account> --chain-bin <osm-chain>
    osm import ./kenya-261101.osm.pbf --region kenya --snapshot 261101 ...
    osm bulk-register --manifest coverage/manifest.json [--skip <region>] ...
    osm unhost --region kenya --cid <cid> ...

Every command takes `--json`; exit codes and messages are in [docs/error-codes.md](docs/error-codes.md).
`--no-geofabrik` never contacts Geofabrik. A write is reported only after the change is visible
on-chain; an included-but-not-applied transaction is an error (S103).

For the v0.3 testnet: `--rpc https://testnet.lez.logos.co --program 5VZeCZKSzkQyYb2nPm4ShDAKpkwncvbQh58XSg4cCAcS`.
Funds for fees come from the [lez-faucet](https://github.com/logos-co/lez-faucet) app (0.1 LEZ per
claim); each transaction needs 38,400,000 lepta available for the fee reserve. The wallet writes
`storage.json` world-readable; run `chmod 600 "$LEE_WALLET_HOME_DIR/storage.json"`.

## Verify

    OSM_STORAGE=<storage-api> scripts/verify-onchain.sh

checks that the deployed program carries the recorded ImageID, then fetches every hosted CID
from Logos Storage, hashes it and compares it with Geofabrik's published MD5 for that snapshot.
It exits 0 only at 25 regions and 15 countries (the prize floor).

## Regions

The closed set from the prize: 48 countries at country level and 24 subregions of the US, India,
China and Russia, in [regions/predefined.json](regions/predefined.json) (`osm regions` prints it).
Ireland and Malaysia map to Geofabrik's `ireland-and-northern-ireland` and
`malaysia-singapore-brunei`, the only extracts that contain them without overlapping another
region in the set.

## For module developers

The SDK and a worked example (region → CID + metadata → verified file) are in
[docs/sdk.md](docs/sdk.md). Other languages use the C ABI in
[osm_ffi/include/osm_ffi.h](osm_ffi/include/osm_ffi.h).

## Design and evidence

- Decisions: [docs/adr/](docs/adr/README.md) · Security: [docs/security.md](docs/security.md)
- Cycle counts: [docs/cycles.md](docs/cycles.md) · Probes: [probes/](probes/README.md)
- Upstream problems found: [docs/BUGS_FILED.md](docs/BUGS_FILED.md) · Test traffic: [docs/test-traffic.md](docs/test-traffic.md)
- Pinned versions: [VERSIONS.md](VERSIONS.md)

## Licence

Dual licensed under MIT ([LICENSE-MIT](LICENSE-MIT)) or Apache-2.0
([LICENSE-APACHE](LICENSE-APACHE)), at your option.
