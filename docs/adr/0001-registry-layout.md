# ADR 0001: Registry layout, one account per region

Status: Proposed. Becomes Accepted when SC-B.4 and SC-B.5 pass on a real sequencer.
Changes plan §2 row "Registry layout" (one account per program instance).

## Context

- LEZ `v0.3.2` caps account data at 100 KiB (`lee/state_machine/core/src/account/data.rs:7`,
  `DATA_MAX_LENGTH = ByteSize::kib(100)`). The 0.4 per-account bound is UNKNOWN
  (docs/status/lez-model-notes.md).
- Plan §2 guards want a per-region history ring of about 8 and up to 16 hosts per CID. In one
  account that is about 420 KiB for 72 regions; only ring 3 / hosts 6 fits under 100 KiB.
- SPEL (`vpavlin/spel` @ `e43b59be`) derives PDAs from constants and instruction arguments:
  `#[account(init, pda = [literal("config"), arg("user_id")])]`
  (`tests/e2e/fixture_program/src/lib.rs:47`), and takes variable account lists
  (`targets: Vec<AccountMeta>`, same file line 161).

## Decision

- One program-derived account per region: seed = `[literal("osm-region"), arg("region")]`. The
  program checks `region` against the compiled 72-leaf table (osm_core) before touching the
  account, so at most 72 region accounts can ever exist.
- No global registry account holds entries. Discovery reads the 72 addresses, which any client
  can compute offline from `regions/predefined.json` and the program id.
- Bounds per region account: history ring 8 entries, 16 hosts per entry. About 6 KB worst case,
  well under 100 KiB. A test fills one account to the maximum and asserts the encoded size.
- `register_batch` takes `Vec<AccountMeta>` of region accounts plus one entry per account, so a
  bulk host is one transaction (P-F7). The batch size is bounded by the transaction size limit
  (`max_block_size` 1,048,576 bytes minus overhead); the real maximum is measured in Phase G.

## Consequences

- Query by region = one account read. Query by parent or by CID = read up to 72 accounts. The
  SDK batches these reads; it never needs an off-chain index.
- W3 (more than 5 hosts on one CID) fits on-chain.
- Assumption to prove in SC-B.5: a `mut` PDA account that is still default can be written by the
  program on first use (SPEL v0.3: "writing data to a default-owned account *was* the claim",
  `spel-framework-core/src/spel_output.rs:23`). If SPEL insists on `init`, see ADR 0003 fallback.
