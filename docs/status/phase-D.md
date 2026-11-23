# Phase D: Basecamp modules + catalog

| SC | Status | Evidence / missing |
|----|--------|--------------------|
| SC-D.1 core module (mkLogosModule) exposing the SDK | done | `pramadani_osm_distribution` (16 methods, `progress` event) and `pramadani_osm_sdk`, built in CI (`modules.yml`) for Linux amd64, macOS arm64, Linux arm64 and Windows x86_64 (cross); loaded in Basecamp (below) and in logosctl (`load` job) |
| SC-D.2 QML app | done | `pramadani_osm_app` on the Logos Design System: regions with hosted status, search, CID and parent lookup, details, download, update check (one or all), single and bulk host with a confirm dialog, import, activity with progress, settings with effective values and a connection test; plain text only, no regex (claims 8, 14). Loaded in Basecamp at v0.1.0; the 0.1.3 screens are rendered offscreen (`tests/ui/preview.sh`) and, from `32246de`, opened in the released Basecamp AppImage under Xvfb by the `basecamp-gui` job |
| SC-D.2b unique names, side-by-side with rival catalogs | written, first CI result pending | `tests/module_manifests.py` (prefix, rival names) and `tests/side_by_side.py`: our catalog and another LP-0018 catalog installed and loaded in one logosctl session (`side-by-side` job) |
| SC-D.3 both templates inside Basecamp | done | `pramadani_osm_consumer_app` loads as a Basecamp app (below); `pramadani_osm_consumer_module` loads and answers `mapFor` in logosctl (`tests/module_load.py`) |
| SC-D.4 root module.json | done | `module.json` indexes the 5 manifests; `tests/module_manifests.py` checks paths, names, versions, `main`, dependencies vs flake inputs, icons, per-module data folder; runs in CI `scripts` and `modules` |
| SC-D.5 catalog fork + release | done | `pramadanif/pramadani-osm-modules`: releases 0.1.0–0.1.2 of sdk, distribution and app, 0.1.1 of both templates; preflight 11 and 16 pass at the released pin; a re-release at the final pin is due before submission |
| SC-D.6 UI states, formatter in Basecamp Qt, two profiles | done | two profiles keep separate data (below); formatters tested with the Qt of the Basecamp build (`qml-format` job); Submitting/Confirmed/Failed job states in `job_table.h`, tested under ASan |

Toolchain check: the unmodified `rust-with-external-lib` template also built
(`rust_external_lib_plugin.dylib`), so the nix setup on this Mac works.

## Headless load (SC-−1.4, runtime part)

CI job `load` (modules run 43228643018 at `cfb770d`, Linux amd64 and macOS arm64, logosctl dev build
`66d098d3`): a fresh logosctl session installed the `.lgx` of `pramadani_osm_sdk` and
`pramadani_osm_distribution` from disk, loaded both and called `version` (API 1), `discover`
(72 regions), `settings` (per-module data folder) and `resolve kenya` (a real read from the
public v0.3 testnet, answer `null`): 12 of 12 PASS on each OS (`tests/module_load.py`). This proves the
modules load and answer at runtime; it is not the Basecamp GUI, and the app (`ui_qml`) and the
write methods (which need `lez_core` and a wallet) were not exercised.

## Catalog (SC-D.5)

Prepared locally from `logos-co/logos-modules-release-base` @ `33fb564`: `logos-repo.json` for
`pramadanif/pramadani-osm-modules`, one submodule pinned to this repository, a release workflow
per module pointing at its subfolder, `modules.txt` for "Release all", variants
`darwin-arm64,linux-amd64`. Published by the owner on 2026-11-23; releases are listed in the SC-D.5 row.

## In the Basecamp app (2026-11-23, macOS arm64)

Logos Basecamp pre-release `pre-release-4cea4bb-1433` (logos-co/logos-basecamp, unsigned macOS
build), our catalog packages v0.1.0 unpacked into two separate data folders and opened with
`LogosBasecamp --user-dir <profile> --module <profile>`:

- `pramadani_osm_app` ("OSM Maps (LP-0018)") and `pramadani_osm_consumer_app` load as Basecamp
  apps ("Successfully loaded UI module", a dock each); their backends
  (`pramadani_osm_app_backend`, `pramadani_osm_consumer_app_backend`) and
  `pramadani_osm_distribution` load as modules; `lez_core` absent is reported as an optional
  dependency and the module loads without it. No QML error from our views in the log.
- Two profiles at once: each has its own `module_data/pramadani_osm_distribution/<id>/
  pramadani_osm_distribution` folder (mode 0700) and its own module registry
  (`…_bcd12232eaf5` vs `…_d300985b7376`); nothing is shared (H38, D.6 two-profile part).
- Not yet: the screens themselves (no screenshot permission in this session), clicking through
  the flows, Qt number formatting on screen.

