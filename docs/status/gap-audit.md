# Gap audit against the plan (2026-11-09)

Every SC of plan §5 (66, extracted from the plan text), every hard gate H1–H40 and win bar
W1–W14, with status and what blocks it. Categories:

- **done**: evidence in the repo (linked status file or test).
- **code**: can be done by the agent now (nix is installed since 2026-11-09).
- **vps**: needs the public Storage node (human gives SSH access).
- **0.4**: needs testnet 0.4 to launch (date unknown) or the 0.4 port.
- **human**: recruiting, video, PR clicks, questions to Logos.

## Phase SCs

| SC | Status | Note |
|----|--------|------|
| −1.1 | done | nix 3.24.2 |
| −1.2 | done | v0.4.0-rc1 standalone sequencer runs |
| −1.2b | done | docs/status/basecamp-modules.md |
| −1.3 | done | two local Storage nodes, CID round trip |
| −1.4 | code | Basecamp + hello mkLogosModule (now possible) |
| −1.5 | done for v0.3; 0.4 route chosen (b) | SPEL 0.4 port is item "0.4" below |
| −1.5b | done | docs/status/geofabrik.md |
| −1.5c | human | 0.4 funding: no faucet for 0.4 in any Logos repo; ask Logos |
| −1.6 | done | docs/status/rivals.md |
| 0.1–0.3 | done | |
| A.1 | done except ADR 0001/0003 "Proposed" wording | proven on v0.3 and 0.4-rc; flip to Accepted (code-free, do now) |
| A.2, A.3 | done | |
| A.4 | code + vps | alpha distribution module + template on v0.3 public; module needs Basecamp work, public data needs the VPS node |
| A.5 | code | CI e2e on v0.4-rc localnet missing (v0.3.1 job exists); LEZ tag watcher done |
| B.1, B.3, B.4, B.5 | done (v0.3) | |
| B.2 | done | all listed behaviours on a real sequencer (18 probes) + max-fill test |
| C.1–C.8 | done | |
| C.6b | done except H28 doc | 0.4 wallet password via `LEE_WALLET_PASSWORD` used by our scripts; README line needed (code-free, do now) |
| C.9 | done | fuzz, bounds, confirm-or-S103, data dir, cache cap |
| D.1 | code | core module over `osm_ffi` |
| D.2 | code | QML app; writes through `lez_core.send_generic_public_transaction` |
| D.2b | code | unique names + side-by-side install with #187 / aegonmyy catalogs |
| D.3 | code | both templates inside Basecamp |
| D.4 | code | root module.json + consistency test |
| D.5 | code + human | catalog fork (outward-facing; within the human's blanket approval) |
| D.6 | code | UI states, plain errors, Qt formatter test, two profiles |
| E.1 | code | e2e workflow written; first green run pending (GitHub 504s on upstream downloads) |
| E.2 | done | storage-retrieval job green |
| E.3 | code | nix builds of modules on Linux + macOS in CI |
| E.4 | code | **`demo.sh` missing** |
| E.5 | done | readme-coldstart job (first green run to confirm) |
| E.6 | code | **doc-truth job missing**; also **docs/claims.md missing** (absolute words need grep proof) |
| E.7 | partial | ASan/UBSan for FFI, liveness, script tests done; C++ module under ASan after D.1 |
| F.1 | done for 0.4-rc | step-1 actor program deployed locally (lez-04-port.md); redo with the SPEL build |
| F.2 | vps | |
| F.3, F.4, F.5 | vps + 0.4 | 30 entries / 20 countries; plan in docs/status/coverage-plan.md |
| G.1 | done (v0.3) | docs/cycles.md; redo on 0.4 |
| G.2 | code + human | template repo; 15-minute test with 2 devs is human |
| G.3 | code | **docs/adoption.md missing** |
| H.1 | done except BUGS filing | 10 rows "to file" upstream (outward-facing) |
| H.2 | code | **docs/SOLUTION_DRAFT.md missing** |
| H.3 | code + human | **video shot list missing**; recording is human |
| H.4 | code | self-review refresh on the current commit; **docs/walkthrough.md missing** (§0.6) |
| I.1–I.3 | human (after B–F green) | |
| J.1–J.5 | 0.4 + human | |

## Hard gates

| Gate | Status | Gate | Status |
|------|--------|------|--------|
| H1 SPEL IDL | done v0.3; 0.4 open (SPEL port) | H21 adversarial pass | done for CLI/SDK; QML plain text after D.2 |
| H2 module.json | code (D.4) | H22 video captions | human |
| H3 independent retrieval | CI done; public node vps | H23 README cold start | done (job) |
| H4 own-voice video | human | H24 dated snapshots | done (first-of-month, verifier reports) |
| H5 full tx hashes | at submission | H25 funding | done for v0.3 (faucet, reserve check); 0.4 human |
| H6 e2e on every push | code (first green run) | H26 polite Geofabrik | done |
| H7 coverage verifier | done | H27 large files | done (2 GiB, 2.7 MB RSS) |
| H8 cycle counts | done v0.3; 0.4 later | H28 0.4 wallet password | done (env var); README line now |
| H9 no central service | done (CI blocks Geofabrik) | H29 default branch = pin | at submission (preflight 24) |
| H10 dual licence | done | H30 no capture / front-run | done v0.3; 0.4 probes code |
| H11 builder + catalog | code (D, E.3) | H31 signer reuse | done v0.3; 0.4 probes code |
| H12 0.4 ids in README | 0.4 | H32 real fixtures | done (live capture in e2e) |
| H13 retry + checksum errors | done | H33 rejected never success | done CLI/SDK; UI after D.6 |
| H14 macOS build + use | CI builds macOS; real use human | H34 plain errors | done CLI; UI after D.6 |
| H15 independent modules | human | H35 out of box + resets | done partly (liveness, VERSIONS.md); flake pins after D |
| H16 preflight | done (script); exit 0 at J | H36 Qt number formatting | code (D.6) |
| H17 numbers + claims true | code (E.6, claims.md) | H37 bounded untrusted data | done |
| H18 package = source | code (D.5) | H38 local data private/durable | done CLI; two profiles after D |
| H19 unique ids | code (D.2b) | H39 bounded resources | done CLI; paged UI after D |
| H20 every feature in UI | code (D.2) | H40 review pack | code (H.4 refresh) |

## Win bars

| W | Status | W | Status |
|---|--------|---|--------|
| W1 one verify command | done (verify-onchain.sh) | W8 port rehearsed | done for v0.3 tooling; 0.4 after port |
| W2 retrieval CI + author | CI done; author human | W9 Windows build, Android note | Windows CI done; **Android note missing** (now) |
| W3 >5 mirrors | human + 0.4 | W10 honest limitations | done |
| W4 write rules tested | done | W11 criteria checklist | done |
| W5 two templates in Basecamp | code (D.3) | W12 PR body | at I |
| W6 15-minute quick start | code (docs) + human test | W13 adoption table | human |
| W7 30/20 on 0.4 | vps + 0.4 | W14 video | human |

## Order of work from here

1. Code-free fixes now: ADR 0001/0003 Accepted, Android note, H28 README line.
2. Missing files: demo.sh, docs/claims.md + doc-truth CI job, docs/adoption.md,
   docs/walkthrough.md, docs/SOLUTION_DRAFT.md, docs/video-shot-list.md, self-review refresh.
3. Basecamp (−1.4, D.1–D.6, E.3, A.4 module side).
4. 0.4: CI e2e on 0.4-rc (A.5), probes/batch/unhost on 0.4, SPEL port (H1), `lez-v04` adapter.
