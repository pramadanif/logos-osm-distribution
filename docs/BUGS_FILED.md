# Upstream problems found while building

Each row: what, where, how we worked around it, and the upstream issue once filed. "To file" means
not yet reported; the human files or approves filing (outward-facing).

| # | Component | Problem | Workaround in this repo | Upstream |
|---|-----------|---------|-------------------------|----------|
| 1 | SPEL `spel-client-gen --target rust+ffi` (vpavlin/spel @ e43b59be) | Generated `osm_registry_client.rs` derives `Serialize, Deserialize` but not `BorshSerialize` on the instruction enum, so `Message::try_new_with_fees` does not compile; the FFI file next to it has the derive | `programs/osm_registry/lez_adapter_v03/build.rs` copies the file and adds the derive | to file |
| 2 | same | Generated `fetch_<account>` reads `account.data` as bytes; since LEZ v0.3 account data is per-program shards, so it does not compile | same build script drops the function; reads use `osm_sdk::registry` | to file |
| 3 | SPEL `spel` CLI (LEZ v0.3.0 wallet lib) vs LEZ v0.3.1 wallet | Config formats differ (`sequencers: [...]` vs `sequencer: {...}`) | one config file carrying both fields (docs/status/phase-B.md) | to file |
| 4 | SPEL `#[lez_program]` | A `Vec<AccountMeta>` parameter takes the remaining accounts; declared before another account it silently mis-maps and the tx is included without effect. Not documented in the README | signer first, vector last | to file |
| 5 | SPEL `spel` CLI | Reports "Transaction confirmed — included in a block" for a transaction whose execution failed (state unchanged) | our adapter confirms by reading the account back (H33) | to file |
| 6 | logos-storage-nim v0.5.3 | `GET /data/{cid}/exists` returns `{"<cid>": bool}`, openapi.yaml documents `{"has": bool}`; `--api-port` missing from `--help` | client accepts both shapes | to file |
| 7 | LEZ v0.3.1 debug config | Sequencer RPC binds `0.0.0.0:3040` by default with no auth | we pass `--listen-address 127.0.0.1` | to file |
| 8 | LEZ v0.3.1 `wallet` | Creates `storage.json` (account keys) with mode 0644, readable by other local users | we `chmod 600` the file after setup; documented for users in README (to do) | to file |
| 9 | LEZ v0.4.0-rc1 `wallet` | Prompts for the storage password on /dev/tty; scripts fail with "Device not configured" unless `LEE_WALLET_PASSWORD` is set (the variable is not mentioned in `--help`) | our scripts set `LEE_WALLET_PASSWORD` from a 0600 file | to file |
| 10 | LEZ v0.4.0-rc1 `wallet program-loader deploy` | `--fee-payer` is not used for segment uploads: "payer <segment account> holds 0 but the fee reserve is 16693696" | fund header and segment accounts first | to file |
