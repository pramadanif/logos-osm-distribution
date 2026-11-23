# Release integrity and security (lambda-prize PR 189)

Source: logos-co/lambda-prize PR 189, "Release integrity and security requirements for
submissions" (weboko, opened 2026-11-13, open and not merged on 2026-11-23). It adds a README
policy that every prize shipping a Basecamp module refers to by security tier. Until it is merged
the checks it adds to the submission validator are warnings; we follow it now so the submission
does not depend on how it is merged. The reviewer pages at https://data.vpavlin.xyz/reviews/ were
checked on 2026-11-23: none newer than 2026-11-02, all identical to the copies we already read.

## Baseline requirements and where we stand (2026-11-23)

| # | Requirement | Status | Evidence / what is left |
|---|-------------|--------|-------------------------|
| 1 | Packages signed with `lgx sign`, signer DID in `trustedSigners` | configured | catalog `_release-module.yml` `signing_mode: inline` with the `LOGOS_SIGNING_KEY` Actions secret (set by the owner 2026-11-23); `logos-repo.json` `trustedSigners` holds the DID below; signing and `lgx verify` tested locally on sdk 0.1.2 ("Signature is valid"). First signed release: next "Release all modules" |
| 2 | Signer DID declared up front (solution PR and README) | to do | DID below; goes into the README install section and the solution PR with the first signed release |
| 3 | Immutable GitHub releases | done | release immutability enabled on `pramadani-osm-modules` and `logos-osm-distribution` (API `immutable-releases`: `enabled: true`, 2026-11-23); the rolling `index` release existed before. Releases 0.1.0–0.1.2 were published before this and are not immutable; only releases from 0.1.3 on are submitted |
| 4 | Freeze rule: never replace or delete a published release | done | catalog workflow `skip_if_published: true`, `force_build` ignored; changes ship as a new version |
| 5 | Release integrity manifest in the solution PR | to do | after the signed release: source commit SHA, catalog URL, package, version, release URL, `rootHash`, SHA-256, variants, DID, taken from the release sidecar JSON / `lgx manifest` |
| 6 | Canonical install instructions (catalog URL + DID + `rootHash`) | to do | README Basecamp section, after the signed release; remove the `--allow-unsigned` fallback then |
| 7 | Catalog on a GitHub organisation account | owner decision | today `pramadanif/pramadani-osm-modules` (personal account); moving means a new catalog URL, so decide before the release that is submitted |
| 8 | 2FA, protected default branch and release tags, actions pinned to SHAs, signing key only in a secret | partly | actions and reusable workflows pinned to full SHAs in both repos, checked by the ci `scripts` job (`62b97d0`, catalog `7a2dcd8`); key only in the Actions secret and in `~/.config/logos/keys/` on the owner's Mac, never in a repository. Owner: 2FA, rulesets for `main` and tags |
| 9 | Secure development basics: no secrets in the repo, lockfiles | done | `Cargo.lock` and a `flake.lock` for every module (four were missing, added in `62b97d0`, builder pinned to `5f74cdb3`); CI fails on a module without a lock |
| 10 | Safe for evaluators: no `--accept-flake-config`, no extra substituters, no piped remote scripts | done | none in the repository's flakes, scripts or README |

Security tier: not stated for LP-0018 yet (PR 189 does not touch existing prizes). Our modules hold
no keys (signing is `lez_core`'s wallet) and no personal data; what they store is described in
[docs/security.md](../security.md) ("Local data"), which serves as the Tier 1 data-handling note
if the prize ends up at Tier 1.

Signer DID:

    did:jwk:eyJjcnYiOiJFZDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6ImxkcmZVLWIzNkdyWlhSbGZUbkpDbng4X1hmN3hUY2JCQUszRFpnN2dQaFUifQ

## Order from here

1. Modules CI green at a commit; bump the catalog submodule to it.
2. Owner clicks "Release all modules": signed, immutable 0.1.3 releases (consumer app 0.1.2).
3. Check each release: Immutable badge, `lgx verify` with the DID, sha256 against the index.
4. Fill the manifest (solution draft) and the README install section; preflight 11 and 16.
5. Owner: decide on the organisation account, 2FA, rulesets.
