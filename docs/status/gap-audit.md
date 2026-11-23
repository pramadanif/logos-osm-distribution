# Gap audit against the plan (2026-11-09, refreshed 2026-11-10 and 2026-11-23)

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
| −1.4 | done | our apps and modules load in the Basecamp desktop app (macOS, pre-release 4cea4bb) and in logosctl on Linux + macOS (docs/status/phase-D.md); the `basecamp-gui` job opens the app in the released Linux AppImage on every modules run |
| −1.5 | done for v0.3; 0.4 route chosen (b) | SPEL 0.4 port is item "0.4" below |
| −1.5b | done | docs/status/geofabrik.md |
| −1.5c | human | 0.4 funding: no faucet for 0.4 in any Logos repo; ask Logos |
| −1.6 | done | docs/status/rivals.md |
| 0.1–0.3 | done | |
| A.1 | done | ADR 0001 and 0003 Accepted, proven on v0.3 and 0.4-rc |
| A.2, A.3 | done | |
| A.4 | done | module and templates built (modules CI); public data on our Storage node (F.2) |
| A.5 | done | CI `e2e-v04` green 18/18 incl. `RegisterBatch` (run 43227068962 at 5cfda57); local 15/15 (probes/results-v04.txt) |
| B.1, B.3, B.4, B.5 | done (v0.3) | |
| B.2 | done | all listed behaviours on a real sequencer (18 probes) + max-fill test |
| C.1–C.8 | done | |
| C.6b | done | README documents `LEE_WALLET_PASSWORD` |
| C.9 | done | fuzz, bounds, confirm-or-S103, data dir, cache cap |
| D.1 | done | built for 4 platforms in CI, loaded in Basecamp and logosctl (phase-D.md) |
| D.2 | done | `pramadani_osm_app` on the Logos Design System; loaded in Basecamp; `basecamp-gui` job opens it in the released AppImage |
| D.2b | done in code | names checked by tests/module_manifests.py; `tests/side_by_side.py` installs our catalog beside another LP-0018 catalog in one logosctl session (CI `side-by-side`) |
| D.3 | done | consumer app loads in Basecamp, consumer module answers in logosctl |
| D.4 | done | `module.json` + tests/module_manifests.py in CI |
| D.5 | done | catalog `pramadanif/pramadani-osm-modules`, releases up to 0.1.2; preflight 11 + 16 pass at the released pin; re-release at the final pin before submission (owner clicks "Release all modules") |
| D.6 | done | two profiles keep separate data; formatters tested on Basecamp's Qt (`qml-format`); job states tested under ASan |
| E.1 | done | first green e2e run 42755836260 at 0f9a9f2 |
| E.2 | done | storage-retrieval job green |
| E.3 | done | `.github/workflows/modules.yml` green on Linux + macOS (run 42757080576) |
| E.4 | done (rerun at submission) | `demo.sh` exists |
| E.5 | done | readme-coldstart job (first green run to confirm) |
| E.6 | done | doc-truth job + docs/claims.md |
| E.7 | done | ASan/UBSan for the FFI and for the app backend's job table (`tests/cpp`), liveness, script tests |
| F.1 | done for 0.4-rc | step-1 actor program deployed locally (lez-04-port.md); redo with the SPEL build |
| F.2 | done | public Storage node `139.190.104.17:8070` (deploy/storage-node/public-node.json), test CID fetched from outside with matching MD5 (2026-11-21) |
| F.3, F.4, F.5 | staging (vps) + 0.4 | the 30-entry set is being staged on our node into `coverage/manifest.json` (6 of 30 on 2026-11-23); registration waits for testnet 0.4 |
| G.1 | done (v0.3) | docs/cycles.md; redo on 0.4 |
| G.2 | code + human | template repo; 15-minute test with 2 devs is human |
| G.3 | done | docs/adoption.md |
| H.1 | done except BUGS filing | 10 rows "to file" upstream (outward-facing) |
| H.2 | done (draft) | docs/SOLUTION_DRAFT.md; pending items marked |
| H.3 | human | shot list in docs/video-shot-list.md; recording is human |
| H.4 | done, refresh at the final pin | docs/walkthrough.md; docs/self-review.md answers all 87 E.1 rows |
| I.1–I.3 | human (after B–F green) | |
| J.1–J.5 | 0.4 + human | |

## Hard gates

| Gate | Status | Gate | Status |
|------|--------|------|--------|
| H1 SPEL IDL | done v0.3; 0.4 open (SPEL port) | H21 adversarial pass | done; QML renders plain text only |
| H2 module.json | done (D.4) | H22 video captions | human |
| H3 independent retrieval | done: public-node job fetches from our node on a GitHub runner, plus staged coverage files | H23 README cold start | done (job) |
| H4 own-voice video | human | H24 dated snapshots | done (first-of-month, verifier reports) |
| H5 full tx hashes | at submission | H25 funding | done for v0.3 (faucet, reserve check); 0.4 human |
| H6 e2e on every push | done (run 42755836260) | H26 polite Geofabrik | done |
| H7 coverage verifier | done | H27 large files | done (2 GiB, 2.7 MB RSS) |
| H8 cycle counts | done v0.3; 0.4 later | H28 0.4 wallet password | done (env var); README line now |
| H9 no central service | done (CI blocks Geofabrik) | H29 default branch = pin | at submission (preflight 24) |
| H10 dual licence | done | H30 no capture / front-run | done v0.3; 0.4 probes code |
| H11 builder + catalog | done (E.3, D.5) | H31 signer reuse | done v0.3; 0.4 probes code |
| H12 0.4 ids in README | 0.4 | H32 real fixtures | done (live capture in e2e) |
| H13 retry + checksum errors | done | H33 rejected never success | done (job states read the module's answer) |
| H14 macOS build + use | CI builds macOS; real use human | H34 plain errors | done (code plus message in the app) |
| H15 independent modules | human | H35 out of box + resets | done (liveness, VERSIONS.md, flake pins) |
| H16 preflight | done (script); exit 0 at J | H36 Qt number formatting | done (`qml-format`) |
| H17 numbers + claims true | done (E.6) | H37 bounded untrusted data | done |
| H18 package = source | done (preflight 16) | H38 local data private/durable | done (two profiles) |
| H19 unique ids | done in code (D.2b) | H39 bounded resources | done (job table capped at 200) |
| H20 every feature in UI | done (self-review row 5) | H40 review pack | done; refresh at the final pin |

## Win bars

| W | Status | W | Status |
|---|--------|---|--------|
| W1 one verify command | done (verify-onchain.sh) | W8 port rehearsed | done for v0.3 tooling; 0.4 after port |
| W2 retrieval CI + author | CI done; author human | W9 Windows build, Android note | done: Windows x86_64 built in CI (cross), limitations.md |
| W3 >5 mirrors | human + 0.4 | W10 honest limitations | done |
| W4 write rules tested | done | W11 criteria checklist | done |
| W5 two templates in Basecamp | done (D.3) | W12 PR body | at I |
| W6 15-minute quick start | code (docs) + human test | W13 adoption table | human |
| W7 30/20 on 0.4 | vps + 0.4 | W14 video | human |

## Order of work from here (2026-11-23)

1. Code, ongoing: keep CI green (modules incl. `basecamp-gui` and `side-by-side`, public-node with
   staged coverage); finish staging the coverage set; at the final pin, re-release the catalog
   (owner click) and rerun preflight.
2. Waiting for Logos: testnet 0.4 (then port with `scripts/port-testnet.sh`, register the staged
   set from `coverage/manifest.json`, 0.4 ids in README, cycle counts on 0.4) and 0.4 funding.
3. Blocked by rules: a SPEL IDL for 0.4 needs SPEL to support LEZ 0.4 (third-party repository).
4. People: five independent modules, the 15-minute test with two developers, the video, filing the
   rows of docs/BUGS_FILED.md upstream, the PR.
