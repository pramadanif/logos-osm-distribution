# Template: OSM consumer app

A Basecamp app (QML view + C++ backend) that asks the `pramadani_osm_distribution` module for a
region's registered snapshot and downloads the verified file. No wallet, no Geofabrik.

    nix build .#lgx-portable

To start your own app: copy this folder, change `name` in `metadata.json`, rename the `.rep`,
backend and icon files to match, and build your idea in `src/`. For the LP-0018 adoption count
the module must be your own idea in your own repository (see `docs/adoption.md`).
