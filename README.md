# logos-osm-distribution

OpenStreetMap data distribution for the Logos stack, built for
[λPrize LP-0018](https://github.com/logos-co/lambda-prize/blob/master/prizes/LP-0018.md): an
on-chain region registry on the Logos Execution Zone (LEZ), map files in Logos Storage, a Rust
SDK, a CLI and Logos Basecamp modules.

**Status: early development. Nothing is deployed and there is no release yet.** What exists
today is listed below; everything else is planned, not built.

## What works now

- `regions/predefined.json`: the 72 regions of the prize's closed, non-overlapping set
  (48 countries, 24 subregions of `us`, `india`, `china`, `russia`), with the matching Geofabrik
  path for each. Two names needed an interpretation; see
  [docs/limitations.md](docs/limitations.md).
- `osm_core`: the region table and entry validation shared by the future registry program and
  the SDK (dated Geofabrik source URLs, MD5 format, size and timestamp bounds).
- `osm_sdk` (offline parts): region discovery from the Geofabrik index restricted to the set,
  offline discovery, dated snapshot handling, streaming MD5 import check, upload retry with
  exponential back-off and jitter, private data folder with atomic writes.

## Build and test

Needs a Rust toolchain (stable).

    cargo test --locked

## Design

- Decisions: [docs/adr/](docs/adr/README.md)
- Error codes: [docs/error-codes.md](docs/error-codes.md)
- Limitations: [docs/limitations.md](docs/limitations.md)
- Pinned upstream versions: [VERSIONS.md](VERSIONS.md)

## Licence

Dual licensed under MIT ([LICENSE-MIT](LICENSE-MIT)) or Apache-2.0
([LICENSE-APACHE](LICENSE-APACHE)), at your option.
