# Self-review (evaluator method on our own repo)

Pinned commit: `37e855da66735d24f74837bf754b37d137ce0eb2` (local `main`; no public remote yet).
Date: 2026-11-09. Reviewer: the build agent, following plan Appendix E and docs/status/e1-extra.md.

## Verdict

Early-stage repository: licences, the frozen region set, shared validation and design records.
Nothing on-chain, no SDK, no Basecamp module. What exists is correct and tested: 13 tests pass on
a fresh clone and every rule in ADR 0002 that can be checked off-chain has an accept and a reject
test. It is not usable by anyone yet, and the README is empty. 0 high, 2 medium findings.

## How I reviewed

Cloned the local repository into a temporary directory (`git clone . <tmp>/lp18-review`),
ran `cargo test --locked` and `cargo fmt --check` there, listed tracked files for secrets and the
evaluator corpus, and compared every number in docs with command output.

## README says / Observed

| README says | Observed |
|-------------|----------|
| (README is empty) | `cargo test --locked`: 13 passed, 0 failed |

## Findings

**M1. README is empty (H23).** A cold reader cannot learn what the repo is or how to build it.
Reproduce: `cat README.md` prints nothing. Phase: H owns the full README; a minimal build section
is added before the repo goes public.

**M2. CI never ran (H6, SC-0.3).** `.github/workflows/ci.yml` exists, but there is no remote.
Reproduce: `gh repo view pramadanif/lp-0018-osm-distribution` → not found. Blocked on human
approval to create the repo.

**L1. CID validation is structural.** `validate_cid` accepts any base58btc string starting with
`z` of 46–96 characters; the test CID is synthetic. Pinned to the real Storage format after the
first real upload (SC-−1.3).

## What works

- `regions/predefined.json` and `osm_core::PREDEFINED` agree, 72 leaves, no overlap (5 tests).
- Entry validation rejects out-of-set regions, wrong parent/level, undated or foreign source
  URLs (including a look-alike host and a non-ASCII digit), bad MD5 forms without normalising,
  sizes 0 and above 64 GiB including `u64::MAX`, timestamps 0, 1 and below the floor (8 tests).

## Not tested

Everything on-chain, Storage, Basecamp, macOS/Windows CI, ASan/UBSan, fuzzing, two profiles,
catalogs: none of these exist yet.

## E.1 rows

All rows 1–55 and E56–E87: open, no code yet in their area, except
- 13 (§2 guards 2–3): off-chain part done in osm_core (closed set, size cap); on-chain open.
- 15 (H37 bounds): size and timestamp bounds in osm_core with edge tests; program, SDK, FFI open.
- 19, 26, E85: N/A (no key derivation, no secret hashing, no own identity).
- 31 (SPEL): open, route recorded in docs/status/spel.md.

## Environment

macOS 26.0.1 arm64, rustc 1.95.0. No network writes.

## Test traffic

None. No transaction or upload to any public network has been made.
