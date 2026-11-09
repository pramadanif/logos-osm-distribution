# Probes

Adversarial checks against the unmodified program and library (H40). Each probe prints one
`PASS|FAIL <case> <detail>` line; runs are appended to `results.txt` with the network, program
header and ImageID they ran against.

| Script | What it attacks | Gates |
|--------|-----------------|-------|
| `b5_negative_v03.sh` | front-run copy of `register`, same signer 3+ times, `unhost` by another host, timestamps (1 h old, seconds instead of ms), regions outside the set (`planet`, decomposed `india`), wrong parent, undated `-latest` source, size `u64::MAX`; `register_batch` with swapped accounts, a duplicate region, a valid batch, and a single `register` reaching the same account | H30, H31, H33, H37, §2 guards 1–4, P-F7 |

| `p10_atomic_kill.sh` | kill -9 during atomic writes, 200 rounds: the file is always a complete document; stale temp files are cleaned | H38 |
| `cargo fuzz` (`fuzz/`) | arbitrary bytes into the registry decoder | H37, E87 |
| `osm_sdk/examples/transfer.rs` | 2 GiB upload + network download, peak memory | H27 |

Every case of `b5_negative_v03.sh` reads the region account back with `spel inspect`; a transaction that is "included"
but did not change state counts as rejected, never as success.
