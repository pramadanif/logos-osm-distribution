# LEZ model notes for the registry ADR (read from source, 2026-11-09)

Read only; nothing here has been built or run yet.

## Account data limit

- LEZ `v0.3.2`: `lee/state_machine/core/src/account/data.rs:7`
  `pub const DATA_MAX_LENGTH: ByteSize = ByteSize::kib(100);` and writes beyond it fail with
  "data length exceeds maximum allowed length".
- LEZ `v0.4.0-rc1`: account data is `AccountData` holding one `ActorState(Vec<u8>)` per program
  (`lee/state_machine/core/src/account.rs:131`, `account/actor_state.rs:24`). No equivalent
  constant found by grep. Transactions are bounded by `max_block_size` 1,048,576 bytes minus block
  overhead (`lez/sequencer/actors/gossip/src/actor/validation.rs:47`; all network configs).
  **UNKNOWN:** the per-account bound on 0.4 and how storage gas prices large state. Must be
  measured on a 0.4-rc localnet before the ADR is final.

## Consequence for plan §2 (needs an ADR, not a silent change)

Plan §2 locks "one registry account per program instance" with a per-region ring of about 8 and
about 16 hosts. Rough size per region entry (CID 59 B, dated source URL up to 96 B, MD5 16 B,
version, timestamp, `registered_by` 32 B, framing) is about 222 B, hosts 32 B each:

| ring | hosts | 72 regions |
|------|-------|------------|
| 8 | 16 | 419.6 KiB |
| 8 | 8 | 275.6 KiB |
| 4 | 8 | 140.1 KiB |
| 4 | 4 | 104.1 KiB |
| 3 | 6 | 92.7 KiB |

Only very small bounds fit one 100 KiB account on v0.3, and those make W3 (more than 5 hosts on
one CID) nearly impossible. Options for the ADR:

1. One account per region (72 PDA-style accounts derived from the region id), each well under
   the limit even at ring 8 / hosts 16 (about 6 KB). Query by parent or CID then reads up to 72
   accounts instead of one. Front-running init per region must be covered by H30.
2. One index account plus per-region accounts.
3. Keep one account with ring 3 / hosts 6 and accept the W3 cost.

Recommendation going into Phase A: option 1. Decide in `docs/adr/0001-registry-layout.md` once
the SPEL v0.3 port shows how it derives program-owned accounts.

## Timestamps (§2 guard 4)

LEZ `v0.4.0-rc1` lets a program's response set a validity window on block and timestamp:
`timestamp_window(...)` / `block_window(...)` in `lee/state_machine/core/src/program/mod.rs:476-505`,
checked by `is_valid_at(block_id, timestamp)` (line 692). So the program can accept a
client-supplied `timestamp` and declare the transaction valid only within a small window around
it, which binds the stored value to the block time. To verify on a real sequencer (SC-B.5). Not
yet checked whether v0.3 has the same API.

## 0.4 programming model

`Account { nonce, data: AccountData }` with per-program `ActorState` replaces the v0.3 data layout
(`feat!: transactional actors with side-effects`, #935). The vpavlin SPEL port targets LEZ
`v0.3.0`, so a 0.4 port is real work, not a version bump.
