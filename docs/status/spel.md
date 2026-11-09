# SPEL route (H1, SC-−1.5)

Checked 2026-11-09.

- `logos-co/spel` `main` @ `c6f73ce21e` (2026-10-13) targets LEZ 0.2.5-rc2. No branch or PR for
  LEZ 0.3 or 0.4 on upstream. Open PRs #299, #298, #294, #293, #285, #283, #279 are fixes, none a
  LEZ bump.
- `vpavlin/spel` `feat/lez-0.3-fixes` @ `e43b59be3a80a63e3c0aa5e80a4c020dbf6d65a1` (2026-10-30)
  builds against LEZ tag `v0.3.0`. The evaluator used it for Backpack's program on the public
  v0.3 testnet (plan H1 v1.5 note; to be confirmed from the review digest).

## Route

1. v0.3 backend (alpha for module authors): use `vpavlin/spel` @ `e43b59be`, pinned by commit,
   disclosed in README and solution. Reason: it is the only SPEL rev known to produce IDL and
   working programs on LEZ 0.3, and it is the evaluator's own port, so its output is the format
   he will inspect.
2. v0.4 backend: **HUMAN DECISION NEEDED.** Options: (a) wait for an upstream SPEL release for
   LEZ 0.4; (b) port `e43b59be` to LEZ `v0.4.0-rc1` ourselves and offer it upstream; (c) pinned
   third-party fork if one appears. Recommendation: (b), started after Phase B works on v0.3,
   because SPEL-for-0.4 is a shared blocker for every competitor (plan G5).

Never label hand-written JSON as SPEL output.
