# Extra E.1 rows (evaluator checks not covered by plan Appendix E.1)

From a full read of the 14 reviews on 2026-11-09 (local digest in docs/evaluator/DIGEST.md, not
committed). Rows E56 onwards; docs/self-review.md must answer each one like rows 1–55.

| # | Check seen in a review (source) | Applies to us as | Gate |
|---|---------------------------------|------------------|------|
| E56 | Limits in bytes documented as characters; non-ASCII dropped (forumer-review-fresh F12) | Region names, URLs, error text; units in docs = code | H36, H17 |
| E57 | QRegularExpression JIT traps in Basecamp's hardened runtime (logos-forum-review) | No QRegularExpression in our C++/QML; a grep in docs/claims.md proves it | H36, H17 |
| E58 | QML runtime warnings in the Basecamp log; bundled theme modules ignored (lez-m1 U11, dex-ux) | Zero QML warnings at start and on every screen | H34 |
| E59 | Program ImageID reproducible and equal to the deployed id (lez-m1, lns) | Release-profile guest build; ImageID in README = on-chain | H12, H32 |
| E60 | Spec vectors checked byte for byte (lns) | Geofabrik MD5 vectors, CID of fixture files | H32 |
| E61 | Unit or scale mismatch hidden by fixtures (lez-m1 F-ST1) | Bytes vs MB, lepta vs LEZ, seconds vs ms between program, SDK, UI | H32, H36 |
| E62 | Timestamp hides staleness (lez-m1 T5) | Show snapshot date and registration time separately | H34 |
| E63 | Documented parameter silently ignored (lez-m1 T4) | Every CLI flag and setting has a test that it changes behaviour | H17 |
| E64 | Silent no-ops, silent input changes (lez-m1 T9/U8, dex-ux) | Reject invalid input with a message, never trim or clamp | H34 |
| E65 | Shown fee differs from what the chain charges (lez-m1 U5/D3, dex-ux) | Fee estimate from 0.4 dry-run vs actual, test within tolerance | H36, H33 |
| E66 | Dangerous default not in the confirm dialog (lez-m1 U2, dex-ux) | Bulk host confirm shows total GB, fee, region count | H36 |
| E67 | Long operation with no progress (dex-ux) | Progress for download, MD5, upload, proof wait | H34, H27 |
| E68 | Settings hide the active value from env or default (dex-ux) | Settings show effective value and its source | H35, H20 |
| E69 | Runtime dependency not declared in the manifest (agora) | Manifest lists Storage, wallet, LEZ modules with versions | H35 |
| E70 | Portable native library: no RUNPATH, glibc floor, loaded from package dir (backpack e2e 10-27) | Check our .so in the .lgx | H18 |
| E71 | Storage hosting exposes node address; blob confirmation (forumer 0.2.3) | Disclose in security.md and limitations.md for hosts | H21, W10 |
| E72 | Limits enforced again on read, not only on write (forumer 0.2.3 security) | SDK decoder re-checks every bound from the program | H37 |
| E73 | Look-alike ids through short truncation (forumer, agora) | Show full account ids and CIDs, never short prefixes alone | H21 |
| E74 | Unbounded params read from local files (forumer) | Config and cache index values bounded on load | H37, H38 |
| E75 | Atomic replace without validating the new file (forumer) | Validate before rename in every atomic write | H38 |
| E76 | Deep link or intent reports success while nothing opens (forumer x2) | Any "open" action returns real status | H33 |
| E77 | Misleading status ("connected" while dials fail; tooltip only) (forumer 0.2.3, agora L2) | Storage/RPC status from a real probe, inline with a fix hint | H34 |
| E78 | Contrast and light theme (dex-ux, forumer fresh) | Check both themes | H34 |
| E79 | Catalog metadata as Basecamp renders it (dex-ux) | Size, name, description in the installer view | H35 |
| E80 | `curl | sh` installer with no signed path (lns) | Scripts download pinned versions with checksums | H35 |
| E81 | Debug line left in shipped Settings (lns) | No developer output in the package | H35 |
| E82 | No default-level log for key operations (lns) | Info log for host, register, download, verify | H34 |
| E83 | Time estimates checked against commit history (lns) | Never claim durations we did not measure | H17 |
| E84 | Own criteria rubric PASS/PARTIAL/FAIL/UNVERIFIABLE (agora) | Same rubric in self-review and criteria-checklist | W11, H40 |
| E85 | Same default identity on every install (agora L6) | N/A: no identity of our own; wallet is the LEZ wallet module | none |
| E86 | Placeholders from unverified references persist (forumer) | Unverified registry entry shown as unverified, never as a region | H33, H34 |
| E87 | Decoders fuzzed under sanitizers (forumer fresh) | cargo fuzz on the registry decoder with ASan | H37 |

## Corrections to E.1 source labels

- Row 7: red CI and the flaky pixel test are Agora M4 (logos-agora-review line "MEDIUM M4. CI is red on the reviewed commit"); Logos Forum CI was green, 8/8 jobs.
- Row 8: rich-text rendering came from Agora and Forumer, not Logos Forum.
- Row 12: Forumer F3 (forumer-review) and F6 (forumer-review-fresh) are the same rate-limit class.
- Rows 53–55: clean checks in forumer-review-0.2.3-security, not findings. Keep the gates anyway.
