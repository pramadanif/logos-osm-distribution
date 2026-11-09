# ADR 0007: LEZ 0.4 port strategy

Status: Proposed. The SPEL 0.4 route waits on a human decision (docs/status/spel.md).

- All LEZ-specific code sits in crate `lez-adapter` with features `lez-v03` and `lez-v04`
  (plan §2). SDK, modules and UI depend on the adapter trait only.
- `networks.json` profiles: `v03-public`, `v04-local`, `v04-public`. Fields: RPC URL, program id,
  network id/channel, explorer URL, faucet URL. Defaults point at the current live deployment.
- 0.4 changes that force adapter work (release notes v0.4.0-rc1): transactional actors with
  per-program `ActorState` (#935), fees with in-proof reserve and refund plus dry-runs (#985),
  argon2-encrypted wallet storage (#987), FFI unification (#993, #996).
- Coverage is replayable: `coverage/manifest.json` (region, CID, MD5, snapshot, size). Storage
  CIDs do not depend on LEZ, so after the 0.4 reset `osm bulk-register --manifest ... --network
  v04-public` re-registers everything; `scripts/port-testnet.sh` wraps rebuild, deploy, register,
  verify (W8).
- v0.3 entries never count for P-A1 (rule 5).
