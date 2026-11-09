# Tracking

- Plan source: ../lambda-prize/planlp0018.md (version: v1.5, 2026-11-09; copy in docs/plan/planlp0018.md, identical by `diff -q`)
- Prize source: docs/plan/prize-LP-0018.md (fetched: 2026-11-09; upstream last change dbf33659, 2026-10-20; equal to the fresh fetch)
- Public repo: https://github.com/pramadanif/logos-osm-distribution (created by the human 2026-11-09). docs/plan/planlp0018.md and the prompt stay local, never published
- Sibling layout: intentional, plan left (../lambda-prize) / solution right (this repo)
- Build start day S: 2026-11-09   (plan §5 dates are relative to S; A.4 alpha due S+4 = 2026-11-13)
- Current phase: A (design); −1 and 0 open on human gates
- Last LOCKED SC: SC-A.3 (also LOCKED: SC-−1.5b, SC-−1.6, SC-0.2, SC-A.2)
- Open H gates: H1–H40 all open (H10 dual licence done locally, not yet public)
- Self-review: docs/self-review.md @ 37e855d, 0 high, 2 medium (M1 README empty, M2 CI never ran)
- Blockers / HUMAN_BLOCKED:
  - nix not installed (needs sudo): SC-−1.1, SC-−1.4, Phase D
  - testnet 0.4 funding route unknown: SC-−1.5c (H25)
  - SPEL route for LEZ 0.4 needs a human choice: SC-−1.5 (H1)
  - Upstream builds (LEZ, SPEL, Storage) refused by the session permission classifier; need a permission rule
