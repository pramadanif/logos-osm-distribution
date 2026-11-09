# Claims and their proofs (H17)

Every absolute statement in this repository's documentation ("never", "every", "no … anywhere")
is listed with the command that proves it. `tests/doc_truth.sh` runs each `proof:` line from the
repository root; a non-zero exit fails CI (job `doc-truth`). Policy statements, which a command
cannot prove, are marked as policy.

| # | Claim | Stated in | Proof |
|---|-------|-----------|-------|
| 1 | `--no-geofabrik` never contacts Geofabrik | README | `osm_cli` builds the Geofabrik client only when the flag is off; commands that need it refuse (test) |
| 2 | Every CLI command takes `--json` | README | `--json` is a clap `global` flag |
| 3 | `registered_by` and hosts come from the signer, never from arguments | security.md, ADR 0002 | no instruction in the SPEL IDL has such an argument |
| 4 | The registry decoder never panics on malformed bytes | security.md | mutation test (200k cases) and fuzz target in CI |
| 5 | Geofabrik region names are never used as file paths | security.md | `name` is only read into `DiscoveredRegion.name`; path code uses `RegionDef` fields |
| 6 | This project never stores a wallet password or logs keys | security.md | no Rust source of ours mentions a password or a private key |
| 7 | `osm-chain` is started with an argument array, never through a shell | security.md | no `Command::new` of a shell anywhere in our Rust code |
| 8 | Input is rejected, never trimmed or normalised | error-codes.md | tests reject padded and upper-case MD5 and look-alike hosts |
| 9 | Every outbound host our code contacts is listed in ADR 0006 | ADR 0006 | preflight check 9 greps code for hostnames (allowlist) |
| 10 | The consumer SDK needs no wallet and never writes to the chain | sdk.md | `osm_sdk` depends on no LEZ crate and builds no transaction |
| 11 | Nothing in a region account is deleted except by ring rotation or a host removing itself | ADR 0002 | the program's only removals are `history.remove(0)` after a push and `hosts.retain` in unhost |
| 12 | Existing error codes never change meaning | error-codes.md | policy (new codes are appended) |
| 13 | v0.3 evidence never counts for the prize | limitations.md | policy (plan rule 5) |

## Proof commands

proof: cargo test --locked -q -p osm_cli --test cli no_geofabrik_refuses_commands_that_need_it 2>&1 | grep -q "1 passed" && grep -q 'then(|| make_geofabrik(cli))' osm_cli/src/main.rs
proof: python3 -I -c 'import re,sys; sys.exit(0 if re.search(r"#\[arg\(long, global = true\)\]\s*json: bool", open("osm_cli/src/main.rs").read()) else 1)'
proof: python3 -I -c 'import json,sys; d=json.load(open("idl/osm_registry.json")); bad=[a["name"] for i in d["instructions"] for a in i.get("args",[]) if a["name"] in ("registered_by","hosts","host","signer")]; sys.exit(1 if bad else 0)'
proof: cargo test --locked -q -p osm_sdk --test decoder_mutation 2>&1 | grep -q "1 passed" && test -f fuzz/fuzz_targets/decode_region.rs
proof: ! grep -nE '(join|Path::new|PathBuf::from)\([^)]*\.name' osm_sdk/src/*.rs osm_cli/src/main.rs osm_ffi/src/lib.rs
proof: ! grep -rniE 'password|private_key|secret_key' osm_core/src osm_sdk/src osm_cli/src osm_ffi/src programs/osm_registry/lez_adapter_v03/src
proof: ! grep -rnE 'Command::new\("(sh|bash|zsh|cmd|powershell)' osm_core osm_sdk osm_cli osm_ffi programs/osm_registry/lez_adapter_v03/src
proof: cargo test --locked -q -p osm_core --test entry rejects_bad_checksums_without_normalising 2>&1 | grep -q "1 passed"
proof: grep -q "download.geofabrik.de" docs/adr/0006-external-services.md && grep -q "check(9" scripts/preflight_submission.py
proof: ! grep -E '^(lee|lee_core|nssa|nssa_core|wallet|spel)' osm_sdk/Cargo.toml && ! grep -rnE 'Transaction|send_transaction' osm_sdk/src
proof: test "$(grep -cE 'history\.remove\(|hosts\.retain\(' programs/osm_registry/methods/guest/src/bin/osm_registry.rs)" = 2
