# Phase 0: bootstrap

Session 1, 2026-11-09. Started while Phase −1 waits on human gates; Phase 0 does not depend on
the blocked items. Phase −1 is still open.

| SC | Status | Evidence / reason |
|----|--------|-------------------|
| SC-0.1 repo created, dual licence, workspace compiles | NOT LOCKED | Local: LICENSE-MIT + LICENSE-APACHE in commit 6e738c9; crate metadata `MIT OR Apache-2.0`; `cargo test` exit 0. GitHub repo `pramadanif/lp-0018-osm-distribution` does not exist (`gh repo view` → "Could not resolve"); creating a public repo waits for human approval |
| SC-0.2 regions/predefined.json, 72 leaves, tests | LOCKED | `cargo test` → `tests/predefined.rs: 5 passed; 0 failed`, exit 0. Checks: 72 total, 48 countries, 24 leaves (us 8, india 6, china 6, russia 4), unique ids, no ancestor/descendant by id or Geofabrik path, Rust table equals JSON, position check rejects `planet`, `india`, wrong parent/level |
| SC-0.3 CI skeleton Linux + macOS + Windows | NOT LOCKED | `.github/workflows/ci.yml` written (fmt, clippy -D warnings, test --locked on 3 OSes). Local `cargo fmt --check` exit 0, `cargo clippy --all-targets -D warnings` exit 0. Not green until it runs on GitHub |
| VERSIONS.md single version source | written | VERSIONS.md; nothing reads it yet |

Phase 0: NOT LOCKED.
