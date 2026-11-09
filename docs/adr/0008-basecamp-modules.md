# ADR 0008: Basecamp modules, names and wiring

Status: Accepted for names and structure; modules not built yet.

## Context

- `logos-module-builder` (rev 5f74cdb3) builds Rust-first core modules: the module API is a Rust
  trait, the builder derives the IPC contract (`.lidl`) and the Qt plugin from it
  (`templates/rust-with-external-lib`). It copies only the crate directory into the build and
  compiles it with `buildRustPackage` with `allowBuiltinFetchGit = true`
  (`lib/mkLogosModule.nix`), so our crate can depend on `osm_sdk` through a git dependency
  pinned to a commit of this repository.
- Signing in Basecamp belongs to `lez_core` (LEZ wallet module, 0.5.1 on LEZ v0.3.1), which offers
  `send_generic_public_transaction` (docs/status/basecamp-modules.md).
- Rival catalogs already use `osm`, `osm-app`, `osm_app`, `osm-consumer-app`,
  `osm_consumer_example` (their `index.json` on 2026-11-09). A shared name makes Basecamp offer to
  "upgrade" one app into another (vpavlin's Agora review, H19).

## Decision

| Module | Name | Type | Role |
|--------|------|------|------|
| Distribution module | `pramadani_osm_distribution` | core (Rust) | the full API: frozen consumer API (`discover`, `resolve`, `fetch`, `checkUpdates`, `hostedStatus`) plus `host`, `importFile`, `bulkRegister`, `unhost`, settings; writes go through `lez_core`, every write is confirmed by read-back |
| SDK module | `pramadani_osm_sdk` | core (Rust) | the consumer subset only (no wallet, no writes), published separately as the prize asks |
| App | `pramadani_osm_app` | ui (QML) | the distribution app; calls only `pramadani_osm_distribution` |
| Template 1 | `pramadani_osm_consumer_app` | ui (QML) | `examples/distribution-consumer-app`, depends on the distribution module (the path 4 of 5 adopters take) |
| Template 2 | `pramadani_osm_consumer_module` | core (Rust) | `examples/consumer-module`, depends on the SDK module only |

Data folders use the module name (`<basecamp data>/pramadani_osm_distribution/…`), never a
shared name. A test in CI keeps the names unique against the rival list above (preflight 17).

## Consequences

- The distribution module pins a commit of this repository for `osm_sdk`; releasing a module
  means bumping that pin first, so the catalog package always matches a commit on `main` (H18, H29).
- No module of ours holds keys; the user's wallet stays in `lez_core`.
