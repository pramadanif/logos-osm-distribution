# ADR 0003: No init authority; front-running is harmless (H30, H31)

Status: Proposed. Accepted when the SC-B.5 negative tests pass on a real sequencer.

## Decision

- The program stores no admin, fee, treasury or curator. There is no `init_registry` that a first
  caller could capture. Region accounts are created on first `register` (ADR 0001).
- A copied `register` from the mempool appends an entry with the copier as `registered_by`, or
  adds the copier as a mirror of our CID. It cannot remove or hide our entry (ADR 0002 rule 2).
- Every instruction takes `#[account(signer)]` from SPEL; no hand-written signer checks (H31).
  Because each call is signed and carries a nonce, two calls never share a hash.

## v0.3 facts

In LEZ v0.3 there is no account claim: writing to a program's own shard at an address denies
that address to nobody (SPEL v0.3 `spel_output.rs`). Only our program can write our shard, and
only through `register`'s rules. So "front-running init" has no target on v0.3.

## Fallback if SPEL requires `init` for a fresh PDA

Keep a permissionless `init_region(region)` that only creates the empty account for a valid
region. Calling it twice fails harmlessly; it stores nothing a caller controls. `register` then
needs the account to exist, and the SDK calls `init_region` first when it reads a default account.

## Tests (SC-B.5)

p01 front-run copy of register; p02 second create of the same region account; p03 the same signer
registers three times, adds a mirror and runs a batch; out-of-set region; wrong parent/level.
