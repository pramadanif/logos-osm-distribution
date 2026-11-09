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

- One program-derived account per region: seed = `[literal("osm-region"), arg("region")]`, with
  `region: String`. SPEL zero-pads a string seed and panics above 32 bytes
  (`spel-framework-core/src/pda.rs`, `seed_from_str`); the longest id,
  `russia/northwestern-fed-district`, is exactly 32 bytes, and a test in osm_core keeps it so. The
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
- LEZ v0.3 model (`spel-framework-core/src/spel_output.rs` header): a handler is a `#[plan]` that
  names effects per shard and sees no account data, plus `#[apply]` functions that get one
  shard's current bytes and return the replacement. "A program may write `shards[self]` at any
  address, whenever it likes, and doing so denies that address to nobody"; `#[account(init)]`
  emits nothing. So the first `register` for a region creates its shard in the apply half
  (`pre_data.is_empty()`), with no separate init. To be proven on a real sequencer (SC-B.4/B.5).
