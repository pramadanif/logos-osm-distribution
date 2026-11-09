# Phase A: design

Session 1, 2026-11-09. Started with Phase −1 and 0 still open on human gates; design work does
not depend on them. Abort check repeated at 02:55Z (docs/status/rivals.md).

| SC | Status | Evidence / reason |
|----|--------|-------------------|
| SC-A.1 ADRs | NOT LOCKED | docs/adr/0001–0007 written. 0002, 0004, 0005, 0006 Accepted; 0001, 0003, 0007 Proposed until a real sequencer run (SC-B.4/B.5) and the SPEL 0.4 decision |
| SC-A.2 docs/error-codes.md | LOCKED | file committed; `tests/entry.rs::error_codes_match_docs` asserts every program code in osm_core appears in it (cargo test exit 0) |
| SC-A.3 docs/tried-failed.md | LOCKED | file committed with 3 rows |
| SC-A.4 alpha distribution module + v03 backend + template, released | NOT LOCKED | needs LEZ/SPEL build permission, nix, the GitHub repo and a funded v0.3 account. Due S+4 = 2026-11-13 |
| SC-A.5 CI matrix v0.3 + v0.4-rc | NOT LOCKED | needs the repo and a working localnet |

Shared validation for ADR 0002 is in osm_core (`entry.rs`, 8 tests). Phase A: NOT LOCKED.
