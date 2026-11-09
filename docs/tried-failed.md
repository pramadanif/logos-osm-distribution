# What we tried and what did not work

Honest log for the solution's "What did not work" section and BUGS_FILED.md. Each line: date,
what, result, evidence, upstream issue if any.

| Date | Tried | Result | Evidence |
|------|-------|--------|----------|
| 2026-11-09 | Single registry account for all 72 regions (plan §2) | Does not fit: about 420 KiB at ring 8 / hosts 16 vs the 100 KiB v0.3 limit | docs/status/lez-model-notes.md; replaced by ADR 0001 |
| 2026-11-09 | Upstream SPEL for LEZ 0.3/0.4 | None exists; `main` stops at LEZ 0.2.5-rc2 | docs/status/spel.md |
| 2026-11-09 | Geofabrik ids = prize ids | Not true for Ireland, Malaysia, and the India/China/Russia/US leaves | docs/status/conflicts.md C1, docs/status/geofabrik.md |
