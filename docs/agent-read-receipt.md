# Agent read receipt

Session 1, 2026-11-09. Counts below were taken with grep over docs/plan/planlp0018.md, not by eye
(commands at the end).

- [x] **Absolute sibling path:** `/Users/muhammadbaguspramadani/Documents/myproject/lp-0018-osm-distribution` (plan in `/Users/muhammadbaguspramadani/Documents/myproject/lambda-prize`)
- [x] **Plan version:** `v1.5 — agent-executable + research lock, deep audit, full vpavlin gap map` (2026-11-09). Copy in docs/plan is identical to `../lambda-prize/planlp0018.md` (`diff -q` silent)
- [x] **P-* criteria (31):** P-F1…P-F9 (9), P-U1…P-U4 (4), P-R1…P-R4 (4), P-P1 (1), P-S1…P-S8 (8), P-A1…P-A5 (5; P-A4 and P-A5 discretionary). Required: 29
- [x] **Hard gates:** 40 (H1–H40). **Win bars:** 14 (W1–W14)
- [x] **Appendix E.1 rows:** 55. Gate per row:
  1 H17 · 2 H18,H29 · 3 H29 · 4 H19 · 5 H20 · 6 H20 · 7 H6 + preflight 6 · 8 H21 · 9 H21 · 10 H21 · 11 H21,H37,H39 · 12 H37 + §2 guard 4 · 13 §2 guards 2–3 · 14 §2 write rules · 15 H37 · 16 H39 · 17 H38 · 18 H38 · 19 H28 (N/A in substance) · 20 H34,H27 · 21 H35 · 22 H35 · 23 H33,H34 · 24 H30 · 25 H30 · 26 none (N/A) · 27 W10,H17 · 28 H36 · 29 H33,H34 · 30 H31 · 31 H1 + G5 · 32 H32 · 33 H33 · 34 H31 · 35 H30 · 36 H35,H23 · 37 H35 · 38 H35 · 39 H35,H34 · 40 H36 · 41 H36 · 42 H33 · 43 H33 · 44 H34 · 45 H28,H38 · 46 H34,H17 · 47 H35,H34 + §0.4 rule 4 · 48 H35 · 49 H34 · 50 H40 · 51 H22 · 52 H28 · 53 H21 · 54 H21 · 55 H27,H37
- [x] **Preflight items §7:** 29
- [x] **Phases:** −1, 0, A, B, C, D, E, F, G, H, I, J (66 SC ids across them)
- [x] **Inviolable rules §0.4:**
  1. Original work only; no clone/fork/vendor/rebrand of rival repos; rival gap lists only.
  2. Dual licence MIT and Apache-2.0 from the first commit.
  3. Phase order: red SC stays; green SC → status file → commit → next.
  4. No fake green: missing sequencer/Storage/r0vm/network on e2e or demo exits non-zero.
  5. Testnet 0.4 is the only counting network; v0.3 is rehearsal.
  6. Adoption integrity: never write or commit code for an independent module.
  7. Escalate only funded keys, VPS, macOS hardware, own-voice video, recruiting, Circle events.
  8. Max 3 submissions, 1 review per week; draft early, ready only after preflight exit 0.
  9. Pin one commit; re-verify every CID and tx on the ready day.
  10. Honesty: no guaranteed win; #187 is ahead on code.
  11. Abort / reassess rules checked daily (quoted below).
- [x] **Abort rules §0.4.11 (quoted):** "Stop new feature work and report to the human if any is true: an LP-0018 solution PR is merged or a reviewer writes "selected to receive this award"; `gh pr view 187 -R logos-co/lambda-prize --json reviewDecision -q .reviewDecision` == `APPROVED`; #187 (or any rival) leaves draft with 0.4 coverage **and** 5 listed adoption modules (reassess odds before spending more)."
- [x] **Human gates §0.6:** testnet 0.4 funding path (Phase −1, ask now); VPS with public IP for the Storage node (Phase F); narrated video in own voice (Phase H); macOS real install proof (Phase H); mark PR ready (Phase J, after preflight 0 and A-5); authorship Q&A readiness (before ready, via docs/walkthrough.md). This prompt adds: sudo for nix, SPEL 0.4 route choice, opening the draft PR.
- [x] **On-chain guards §2, in my words:**
  1. The program writes the transaction signer into `registered_by` and `hosts`; a caller cannot name someone else.
  2. The program carries the 72-leaf table and rejects any `region` not in it, or with a `parent`/`level` that does not match the table.
  3. Storage is bounded: each region keeps a ring of the last N entries (about 8), each CID a deduplicated host list (about 16), and a test fills everything to the maximum and proves the account stays under the LEZ size limit.
  4. `timestamp` comes from the chain if LEZ exposes it; otherwise it is client-supplied, bounded, and documented as such. Every guard has a negative test and a line in security.md.
- [x] **SPEL route first tried:** `vpavlin/spel` `feat/lez-0.3-fixes` @ `e43b59be3a80a63e3c0aa5e80a4c020dbf6d65a1` for the v0.3 backend. Why: upstream SPEL `main` (c6f73ce21e, 2026-10-13) stops at LEZ 0.2.5-rc2; this port is the only rev shown to build and deploy programs on LEZ 0.3 (it pins LEZ `v0.3.0`), and it is the evaluator's own fork. The 0.4 route is a human decision (docs/status/spel.md).
- [x] **Conflicts prize vs plan:** C1 in docs/status/conflicts.md: the prize names Ireland and Malaysia without ids; index has `ireland-and-northern-ireland` and `malaysia-singapore-brunei`. Otherwise none. Upstream prize text (last change dbf33659, 2026-10-20) equals docs/plan/prize-LP-0018.md.
- [x] **Evaluator method (Appendix E, 11 steps):**
  1. Fresh clone; compare HEAD with any earlier review.
  2. Build from source; run tests plain and under ASan/UBSan; count tests vs README.
  3. Check every technical claim against exact source lines.
  4. CI check-runs on HEAD via the API.
  5. Download the catalog `.lgx`; sha256 vs index; diff package vs source.
  6. Install in real Basecamp with `lgpm --allow-unsigned install --file`, Linux x86_64 + Xvfb; use it live; screenshots.
  7. Two isolated profiles exchange data live (host on one, download on the other).
  8. Video with yt-dlp; frames sampled at chapters; subtitles checked.
  9. Separate adversarial security pass with probes against the unmodified library.
  10. Side-by-side install with other catalogs for id/data collisions.
  11. Explicit "Not tested" list.
- [x] **v1.5 method additions:** `nix build .#lgx-portable` and a build log showing only packaging built locally · Basecamp AppImage extracted, run under Xvfb with a fresh `--user-dir` · `bwrap --unshare-net` when he must not touch shared networks · GUI driven by xdotool, recorded with ffmpeg · on-chain state checked after every action (`spel inspect`) · every action logged with a state snapshot (`log.jsonl`, `timeline.txt`) · real rejection reason read from the sequencer log · two-column "README promises / observed" table · own test traffic on shared networks stated.
- [x] **Evaluator corpus read end to end:** 14 reviews in docs/evaluator/ (gitignored); digest in docs/evaluator/DIGEST.md. Three of the 14 (Backpack x3) are the evaluator's own demo reports and Freequencies is a walkthrough; they still show his evidence format. What he checks and in what order: pins commit, lists README promises, reads critical paths, runs tests plain and under ASan/UBSan, CLAIM/WHERE/VERDICT table, probes against the unmodified library, CI via API, `.lgx` vs source (`nix build .#lgx-portable`, `grep -c "^building '"`), install via Package Repositories then `lgpm` fallback, two profiles live, `spel inspect` after each action, sequencer log for rejection reasons, side-by-side catalogs, video frames, security pass, rubric PASS/PARTIAL/FAIL/UNVERIFIABLE, "Not tested", environment, test traffic. He praises candid limitations, real failure logs, regression tests per finding, package = source. 32 checks not in E.1 are added as rows E56–E87 in docs/status/e1-extra.md, with four source-label corrections. Two attributions spot-checked against the HTML (Agora M4, Logos Forum JIT quote): both correct.

Commands used for the counts:

    grep -oE '^\| \*\*P-[A-Z][0-9]+\*\*|^\| P-[A-Z][0-9]+ ' planlp0018.md
    grep -cE '^\| \*\*H[0-9]+\*\* \|' planlp0018.md                      # 40
    grep -cE '^\| \*\*W[0-9]+\*\* \|' planlp0018.md                      # 14
    sed -n '/^### E.1/,/^Praised/p' planlp0018.md | grep -cE '^\| [0-9]+ \|'   # 55
    sed -n '/^## 7. Preflight/,/^## 8/p' planlp0018.md | grep -cE '^[0-9]+\. ' # 29
