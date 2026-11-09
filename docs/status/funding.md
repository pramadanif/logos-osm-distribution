# Testnet funding routes (H25, SC-−1.5c)

Searched 2026-11-09 across the `logos-co`, `logos-blockchain` and `logos-storage` GitHub
organisations (repo search and code search for "faucet", issue search for faucet and testnet 0.4).

## What exists

| Source | What it gives | Network | Useful for fees? |
|--------|---------------|---------|------------------|
| `logos-co/lez-faucet` 0.5.0 (2026-10-26), Basecamp app | legacy: Piñata claim 150 LEZ; v0.3: hosted `claimd` pays 100,000,000 lepta = 0.1 LEZ, 1 claim per account per day | `testnet-legacy.lez.logos.co` (v0.2.4), `testnet.lez.logos.co` (v0.3.1) | yes, v0.3 only. `claimd` root `https://rog-devbox.tail957e2.ts.net:8443` (docs/testnet.md). No 0.4 row |
| `logos-blockchain/lez-programs` `token_mint_authority` | permissionless mint of a test **token**, once per 24 h per recipient | legacy v0.2.4 (DEPLOYMENTS.md) | no: custom tokens, not the native fee token |
| `logos-blockchain/logos-blockchain` `deployment/faucet` | HTTP `POST /faucet/{pk}` with a cooldown, behind nginx `/faucet-backend/` | Logos L1 (Bedrock) node deployments | indirect only: L1 funds, then a bridge deposit into the zone (route b in the plan) |
| `lez/testnet_initial_state` in the LEZ repo | published signing keys of two funded genesis accounts | any network started from that genesis | yes, but shared and drainable; LEZ issue #938 (open) says this is the only documented route on public testnets |

## What does not exist yet

- No faucet, `claimd` row or funding statement for testnet 0.4 in any Logos repository.
- logos-co/ecosystem#251 (2026-11-03): 0.4 adds a testnet reset; all λ-Prizes target 0.4; nothing
  on funding. LEZ v0.4.0-rc1 changes fees (in-proof reserve and refund, dry-runs, #985), so the
  per-transaction cost has to be re-measured.

## Plan

1. v0.3 alpha (SC-A.4): claim from `lez-faucet` / `claimd` for our deployer account. 0.1 LEZ per
   day; one `register` on 0.3 cost about 1.66M lepta in #187's measurement, so one claim covers
   tens of registrations. Program deploy cost still to be measured.
2. Localnet (v0.3.1 and v0.4-rc): the genesis accounts in the local config, no public funds.
3. Testnet 0.4: ask Logos (human gate). Fallbacks: genesis keys of the 0.4 testnet if published
   (read at runtime, never copied), or an L1 faucet plus bridge deposit.
