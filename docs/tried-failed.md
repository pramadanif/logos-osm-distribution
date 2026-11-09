# What we tried and what did not work

Honest log for the solution's "What did not work" section and BUGS_FILED.md. Each line: date,
what, result, evidence, upstream issue if any.

| Date | Tried | Result | Evidence |
|------|-------|--------|----------|
| 2026-11-09 | Single registry account for all 72 regions (plan §2) | Does not fit: about 420 KiB at ring 8 / hosts 16 vs the 100 KiB v0.3 limit | docs/status/lez-model-notes.md; replaced by ADR 0001 |
| 2026-11-09 | Upstream SPEL for LEZ 0.3/0.4 | None exists; `main` stops at LEZ 0.2.5-rc2 | docs/status/spel.md |
| 2026-11-09 | Geofabrik ids = prize ids | Not true for Ireland, Malaysia, and the India/China/Russia/US leaves | docs/status/conflicts.md C1, docs/status/geofabrik.md |
| 2026-11-09 | Storage `/data/{cid}/exists` per openapi.yaml (`{"has": bool}`) | Real v0.5.3 node returns `{"<cid>": bool}`; client fixed to accept both | docs/status/phase--1.md SC-−1.3 |
| 2026-11-09 | `--disc-port` for a second local Storage node | Option does not exist in v0.5.3; discovery uses the libp2p port | same |
| 2026-11-09 | Timestamp validity window in seconds | The sequencer included the `register` tx (fee charged, nonce up) but the shard stayed empty: LEZ block timestamps are milliseconds, so the tx fell outside its own window. Switched to ms; test rejects a seconds value. Also fixed `MIN_TIMESTAMP` that said 2026 but was 2025-11-01 | docs/status/phase-B.md |
| 2026-11-09 | `spel` CLI (LEZ v0.3.0 libs) with a v0.3.1 wallet config | v0.3.0 wants `sequencers: [...]`, v0.3.1 writes `sequencer: {...}`; one config with both fields works for both | same |
| 2026-11-09 | Guest build with a path dependency on `osm_core` from the program folder | Docker context is the current folder, so `../../osm_core` is missing; build from the repo root | same |
