# Self-review (evaluator method on our own repo)

Reviewed commit: `d868dca` (https://github.com/pramadanif/logos-osm-distribution, `main`) and the
documentation commit that adds this file. Date: 2026-11-23. Reviewer: the build agent, following
plan Appendix E, every row of Appendix E.1 (1–55) and docs/status/e1-extra.md (E56–E87).
Previous reviews: 2026-11-12 at `b9053ae`, 2026-11-09 at `9bee462`.

## Verdict

Everything the prize asks that does not need testnet 0.4 or other people is built and shown
working, most of it in CI on every push: the registry on LEZ v0.3 (public testnet, with
`china/henan` registered and its file on our public Storage node) and on a local 0.4-rc
sequencer; SDK, C ABI and CLI; the Basecamp app, the SDK module and two templates, built with
logos-module-builder for Linux, macOS (and, from this commit, Windows and Linux arm64 in CI),
loaded in the Basecamp desktop app and in logosctl, and released through our catalog. Still open
and outside code: testnet 0.4 (not launched), its 25-region coverage, a SPEL IDL for 0.4 (SPEL has
no 0.4 support), five independent modules, the video.

## How I reviewed

Fresh clone and `cargo test --locked`; tracked files listed for wallets, keys, passwords and
private plans; CI runs read through the API; README and docs compared with code and evidence;
the app rendered offscreen with the design system (`tests/ui/preview.sh`) and loaded in Basecamp
(docs/status/phase-D.md); every E.1 row answered below with PASS, PARTIAL, FAIL, UNVERIFIABLE or
N/A and its evidence.

## Findings still open

**M3 → PARTIAL. The new app screens are rendered offscreen, not yet seen in Basecamp.** The app
loaded in Basecamp at v0.1.0 (old screens). The design-system screens are verified by offscreen
renders and the headless load test; a look in Basecamp at 0.1.3 is pending (needs a desktop).

**M4 (open). Coverage is only on the v0.3 rehearsal network.** One region (`china/henan`) is
registered on the public v0.3 testnet; the 15/25 coverage must be on testnet 0.4.

**L3 (open). No SPEL IDL for 0.4.** Needs SPEL support for LEZ 0.4.

**L5 (fixed).** Long operations report progress: the distribution module emits `progress`
events and the app shows a progress bar on the running job (E67).

## E.1 rows

| # | Row (short) | Verdict | Evidence |
|---|-------------|---------|----------|
| 1 | Numbers and absolute claims true | PASS | `tests/doc_truth.sh` in CI: every `proof:` in docs/claims.md, cycle tables vs raw JSON, test counts |
| 2 | Catalog package = source | PASS | preflight 16: index sha256 = downloaded .lgx, manifest version = source, catalog submodule = pin; re-release at the final pin |
| 3 | Default branch = pin | PASS | preflight 24 |
| 4 | Module id collision, shared data | PASS | `tests/module_manifests.py` (prefix, rival names), preflight 17; per-module data folder |
| 5 | Every feature in the UI | PASS | app: discover, details, download, update check (one or all), host, bulk host, import, query by parent and CID, settings, status |
| 6 | Storage endpoint and RPC settable | PASS | Settings tab; `configure` |
| 7 | CI green at the pin | PASS | ci, e2e (v0.3 + 0.4-rc), modules, public-node, daily |
| 8 | Remote strings as rich text | PASS | LogosText/LogosCopyButton default to PlainText; no StyledText in our QML |
| 9 | Markdown/link spoofing | N/A | no Markdown rendered, no links opened by the app |
| 10 | Alias spoofing | PASS | accounts shown as full ids, compared by id |
| 11 | Permissionless input costs work | PASS | bounded decoder (fuzzed), closed region set, 8-entry ring, 16 hosts |
| 12 | Sender timestamps bypass limits | PASS | ±10 min window bound to block time (v0.3 validity window, 0.4 clock check); probes p06a/b |
| 13 | Flood fills storage | PASS | closed set of 72 accounts, ring of 8 entries (ADR 0001/0002) |
| 14 | Forged entry hides a good one | PASS | append-only; `fetch` skips entries whose bytes do not hash to the MD5 |
| 15 | Unbounded indices, overflow | PASS | size, timestamp, history bounds with edge tests (`osm_core/tests/entry.rs`) |
| 16 | Unbounded log or table | PASS | cache cap 20 GiB, 200 finished jobs kept (`job_table.h`, tested), 72-row list |
| 17 | File modes, fsync | PASS | 0700/0600 + fsync + folder fsync (`osm_sdk/src/datadir.rs`, test `datadir.rs`) |
| 18 | Second account sees first's data | PASS | two Basecamp profiles at once keep separate data (phase-D.md) |
| 19 | Key reuse on restore | N/A | no keys of our own; signing is the LEZ wallet's |
| 20 | Heavy work on the UI thread | PASS | every app call is async through the backend; MD5 and transfers run in the module |
| 21 | In-app install fails | UNVERIFIABLE | catalog installs through `lgpd`/package manager not exercised by us; .lgx verified by preflight 16 |
| 22 | Debug screens shipped | PASS | no debug screen or developer output in the package |
| 23 | "Synced" without a peer; clipped at 900 px | PARTIAL | status badges come from real probes (E77); rendered at 1280 px only |
| 24 | Front-running register | PASS | a copy adds the copier as a host only (probe p01, v0.3 and 0.4) |
| 25 | First caller configures deploy | PASS | no init instruction (ADR 0003) |
| 26 | Global salt | N/A | no hashing of secrets |
| 27 | Spec understates a limit | PASS | docs/limitations.md written as found |
| 28 | QML regex bug | PASS | no regex in QML/C++ (claim 14 in docs/claims.md); arithmetic formatter |
| 29 | Stale panel after a poll | PASS | panels replace their content on each answer; jobs show their own state |
| 30 | Signer usable once | PASS | same host registers many times (probes p03a–c) |
| 31 | SPEL for the target LEZ | PARTIAL | SPEL IDL for v0.3 (CI regenerates and diffs); none for 0.4 |
| 32 | Fake fixtures | PASS | Geofabrik index and registry accounts captured live (e2e live capture, `fixture_live`) |
| 33 | Rejected tx shown as success | PASS | every write confirmed by read-back, else S103 "may still land" |
| 34 | Unsigned calls | PASS | every write signed by the host |
| 35 | Bootstrap prerequisite | PASS | no init, no prerequisite account |
| 36 | Docs drift from pins | PASS | readme-coldstart job, VERSIONS.md, daily lez-tags |
| 37 | Deployment gone after reset | PASS | daily liveness checks the ImageID; redeployed after the layout change |
| 38 | Empty default config | PASS | defaults point at the v0.3 deployment |
| 39 | Silent sync failure | PASS | `status` probes shown as badges and in Settings |
| 40 | Raw units | PASS | sizes as KiB/MiB/GiB, lepta written with separators |
| 41 | Alarming preview | PASS | confirm dialog only after a selection, with real counts |
| 42 | No confirmed state, double submit | PASS | Submitting/Confirmed/Failed; host button disabled while a run is going |
| 43 | Created item invisible | PASS | hosted status reloaded from the chain after each run |
| 44 | Raw codes, wrong cause | PASS | code plus message (docs/error-codes.md), no duplicated codes |
| 45 | Unlock without password | PASS | no password stored; 0.4 wallet password only in env |
| 46 | Several names, unstyled widgets | PASS | "OSM Maps (LP-0018)" everywhere; Logos Design System controls |
| 47 | Script exits 0 on failure | PASS | `tests/scripts.sh` strictness test |
| 48 | Unsigned packages | PASS | README says the catalog's packages are unsigned |
| 49 | Empty board after restart | PASS | StateView (spinner) while regions load |
| 50 | Untested list | PASS | "Not tested" below |
| 51 | Video, subtitles | FAIL | not recorded yet (human) |
| 52 | Unencrypted keys | PASS | no wallet in the repo; 0.4 wallet encrypted |
| 53 | Shell with remote strings | PASS | claim 6 (no shell `Command`); argument arrays only |
| 54 | SQL | N/A | no database |
| 55 | Decompression bomb | PASS | PBF streamed and hashed, never decompressed |
| E56 | Bytes vs characters | PASS | region ids ≤ 32 bytes, names bounded in bytes (`MAX_NAME_BYTES`) |
| E57 | QRegularExpression in Basecamp | PASS | claim 14 |
| E58 | QML warnings in the Basecamp log | PARTIAL | none from our views at v0.1.0 in Basecamp; offscreen renders of 0.1.3 log none |
| E59 | ImageID reproducible = deployed | PASS | `verify-onchain.sh` header check (v0.3 public, 0.4-rc in e2e-v04) |
| E60 | Spec vectors byte for byte | PASS | Geofabrik MD5 files and a real CID in tests |
| E61 | Unit mismatch | PASS | ms vs s probe (p06b), bytes everywhere |
| E62 | Timestamp hides staleness | PASS | app shows snapshot date and registration time separately |
| E63 | Parameter silently ignored | PARTIAL | key flags tested (`--no-geofabrik`, `--json`, `--skip`); not every flag |
| E64 | Silent input changes | PASS | rejected, never trimmed (claim 11) |
| E65 | Shown fee ≠ charged | PARTIAL | the dialog states the reserve, not a fee estimate; measured fees in docs/cycles.md |
| E66 | Confirm hides the cost | PARTIAL | dialog: regions, transactions, fee reserve, cache cap; not the total GB (known only after download) |
| E67 | No progress | PASS | the module emits `progress` events (download bytes, verify, upload, register); the app shows a progress bar on the running job |
| E68 | Settings hide the effective value | PASS | Settings show each effective value and whether it is the default or set by the user (`settings().sources`) |
| E69 | Undeclared runtime dependency | PASS | manifests declare lez_core (optional) and the SDK; Storage is reached over its REST API (ADR 0004) |
| E70 | Portable library RUNPATH | UNVERIFIABLE | built by logos-module-builder's portable bundler; loads in Basecamp on macOS, not inspected |
| E71 | Hosting exposes node address | PASS | security.md and limitations.md |
| E72 | Limits re-checked on read | PASS | SDK decoder re-checks every program bound |
| E73 | Look-alike short ids | PASS | full ids with copy buttons |
| E74 | Unbounded local values | PASS | settings keys allow-listed, parsed as JSON; cache bounded |
| E75 | Replace without validating | PASS | `atomic_write` validates before rename |
| E76 | Deep link reports false success | N/A | no deep links |
| E77 | Misleading connection status | PASS | `status` probes (sequencer block id, Storage /space) shown with the error when they fail |
| E78 | Contrast and light theme | UNVERIFIABLE | only Theme tokens are used, so a light theme recolours the app; not looked at |
| E79 | Catalog metadata in the installer | PARTIAL | name, description, icon, version set; the installer view not seen |
| E80 | curl \| sh installers | PASS | pinned versions with sha256 (Storage, rzup via crates.io) |
| E81 | Debug line in Settings | PASS | none |
| E82 | No log for key operations | PASS | module logs downloads, uploads, writes and confirmations |
| E83 | Unmeasured durations | PASS | no duration claims |
| E84 | PASS/PARTIAL/FAIL rubric | PASS | this table |
| E85 | Same default identity | N/A | no identity of our own |
| E86 | Unverified shown as verified | PASS | details say the entry is as registered and that download checks the MD5 |
| E87 | Decoder fuzzed under sanitizers | PASS | `cargo fuzz` (ASan) on `decode_region`, daily |

## Not tested

The app's 0.1.3 screens inside Basecamp (offscreen renders only); the package-manager install
flow; a light theme; Windows and Linux arm64 at runtime (built in CI only); Android; testnet 0.4;
proving time; any host other than ours on a public network.

## Environment

macOS 26.0.1 arm64, rustc 1.95.0 (workspace), 1.98.1 (0.4 client), Qt 6.11 (previews). LEZ
v0.3.1 `1a003bfb`, v0.4.0-rc1 `1806e873`; Logos Storage v0.5.3; logos-module-builder `5f74cdb3`;
Logos Basecamp pre-release `4cea4bb`.

## Test traffic

docs/test-traffic.md: one faucet claim, two program deploys and one registration on the public
v0.3 testnet; our public Storage node; read-only Geofabrik requests.
