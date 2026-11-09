# LEZ 0.4 port: what changes (read from v0.4.0-rc1 source, 2026-11-09)

Nothing below is built yet. It sizes the 0.4 work (ADR 0007, SPEL route b).

## Programming model

- v0.3: a handler is a `#[plan]` (effects per shard, sees no data) plus `#[apply]` functions
  (old bytes in, new bytes out). Types: `PlanInput`, `ApplyInput`, `PlanOutput`, `GuestOutput`.
- v0.4.0-rc1: those types are gone. Every account under a program is an **actor**; a program is an
  entry point `lee_core::actor_entrypoint!(handler)` with `fn handler(input: &ReceiveInput, msg: M)
  -> Response`, where `M` is the borsh-decoded message (`run_actor`, program/mod.rs:736).
  `ReceiveInput { receiver: Actor, from: Option<Actor>, is_authorized, pre_state, message }`;
  `Response::set_state(bytes)` / `keep_state()`, plus `.send_sync(actor, &msg)` / `send_async`.
  Example: `test_programs/guest/src/bin/cooldown.rs`.

## Effect on the registry design

| Topic | v0.3 (built) | v0.4 |
|-------|--------------|------|
| One account per region (ADR 0001) | PDA shard written in apply | one actor per region; same idea, `pre_state` in, `set_state` out |
| Timestamp (guard 4) | client value bound by a validity window | still client-supplied; bound to chain time by a `send_sync` to the built-in clock program, `clock_core::Message::AssertTimestamp { at_least, at_most }` (as `test_programs/.../cooldown.rs` does). An earlier note here said the handler receives the block time; that was wrong: `proposed` in that example is the message |
| `registered_by` = signer (guard 1) | SPEL signer account in the plan | the receiving actor only sees `is_authorized` (signed by itself or by its owning program) and `from`. Likely pattern: the tx is authorized at the host's actor, which `send_sync`s to the region actor; the region actor records `from.account_id` after checking `from_own_program()`. To prove on an rc sequencer |
| Batch | `Vec<AccountMeta>`, one effect per account | one message per region actor from the host actor (`send_sync` per region) |
| Fees | reserve `(gas+100k)*64` | in-proof reserve with refund, dry-runs (#985): re-measure |

## Authorization in 0.4 (read from `lee/state_machine/core/src/execution_state.rs`, `authorize`)

An actor's `is_authorized` is true when its account signed the transaction, or when the sending
program delivers the message with PDA seeds that derive the actor's account from the sender
program (`AccountId::for_public_pda(&issuer, seed) == account_id`). Design for the registry:

1. The transaction sends `Register` to the host's own actor under our program; the host signs, so
   that actor is authorized and knows its account id.
2. That handler `send_sync`s the entry to the region actor with the region's PDA seed, which
   authorizes the region actor through our program.
3. The region actor accepts only messages that are authorized and `from_own_program()`, and
   records `from.account_id` as `registered_by`. A direct transaction to a region actor has no
   `from` and is refused.

## SPEL

`vpavlin/spel` @ e43b59be generates plan/apply code. For 0.4 its `#[lez_program]` expansion,
the IDL generator's account model and the client generator (`Message::try_new_with_fees`, shard
selectors) all have to move to actors and messages. This is the largest remaining code item and
the reason H1 (SPEL IDL) for 0.4 is still open. No upstream SPEL 0.4 work was visible on
2026-11-09 (docs/status/spel.md).

## Order of work when it starts

1. A plain 0.4 actor version of the registry against the v0.4.0-rc1 standalone sequencer
   (SC-−1.2 runs it), to prove the signer pattern and the timestamp source.
2. Port SPEL's macro expansion and IDL generator for that shape; regenerate the IDL.
3. `lez_adapter` `lez-v04` feature over the regenerated client; rerun probes, e2e, cycles on 0.4.

## Step 1 result (2026-11-09, local LEZ v0.4.0-rc1 standalone sequencer on 127.0.0.1:3041)

- `programs/osm_registry_v04/guest` (actor program, no SPEL) built with `cargo risczero build`,
  ImageID `d1213b2811def8ad89db14db2d34c6523ffc6b0b2723f0eca95897e720881c1a`; deployed with the 0.4
  wallet's program-loader: header `5UFsUGATkMAduN5xYG8Rpw7wKYQeE4BTzceLNkfrkEi4` (4 segments).
- `programs/osm_registry_v04/client` (`osm-chain-v04 register …`) sent tx
  `ab1e7ae3d286eb6425dd3ccfdfa904440f0c7809ae93213015f94997406e8c9f`: host actor (signed) →
  clock `AssertTimestamp` → region actor via PDA seed.
- `getAccount CMvUj7VH…` shows `actor_states[program]` holding the same `RegionAccount` layout as
  v0.3; `osm lookup --region china/henan` against 3041 decodes it, `registered_by` = the signer
  `2RHZhw9h…`; `osm download --no-geofabrik` fetched the file via Storage node B and verified it.
- So the signer design, the clock-bound timestamp and the shared data layout work on 0.4-rc.

Problems met (docs/BUGS_FILED.md #9–#10): the 0.4 wallet reads its password from the terminal
unless `LEE_WALLET_PASSWORD` is set; `program-loader deploy --fee-payer` was not used for segment
uploads (each segment account had to be funded first).

Still open for 0.4: negative probes, batch, unhost on 0.4; SPEL port and SPEL IDL (H1); the
`lez-v04` adapter with read-back confirmation; cycle counts on 0.4.
