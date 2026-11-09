# Phase 0: bootstrap

Session 1, 2026-11-09. Started while Phase −1 waits on human gates; Phase 0 does not depend on
the blocked items. Phase −1 is still open.

| SC | Status | Evidence / reason |
|----|--------|-------------------|
| SC-0.1 repo created, dual licence, workspace compiles | LOCKED | Public repo https://github.com/pramadanif/logos-osm-distribution (created by the human 2026-11-09); LICENSE-MIT + LICENSE-APACHE from the first commit; crate metadata `MIT OR Apache-2.0`; fresh clone of 9bee462 `cargo test --locked`: 47 passed, 0 failed |
| SC-0.2 regions/predefined.json, 72 leaves, tests | LOCKED | `cargo test` → `tests/predefined.rs: 5 passed; 0 failed`, exit 0. Checks: 72 total, 48 countries, 24 leaves (us 8, india 6, china 6, russia 4), unique ids, no ancestor/descendant by id or Geofabrik path, Rust table equals JSON, position check rejects `planet`, `india`, wrong parent/level |
| SC-0.3 CI skeleton Linux + macOS + Windows | LOCKED | `gh api repos/pramadanif/logos-osm-distribution/commits/9bee4623b2a4ed046066e85a199576d8a3757a26/check-runs`: rust (ubuntu-latest), (macos-latest), (windows-latest) all success. Run https://github.com/pramadanif/logos-osm-distribution/actions/runs/42555186573. Earlier runs failed: newer clippy lint on stable (fixed by pinning 1.95.0) and two unix-only imports on Windows |
| VERSIONS.md single version source | written | VERSIONS.md; rust-toolchain.toml matches its `RUST` row; nothing reads it by script yet |

Phase 0: LOCKED (SC-0.1, SC-0.2, SC-0.3), 2026-11-09. Phase −1 is still open on human gates.
