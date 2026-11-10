# Template: OSM consumer module

A Logos core module (C++, universal interface) that reads OpenStreetMap snapshots registered on
LEZ through the read-only `pramadani_osm_sdk` module. It holds no wallet and never needs
Geofabrik.

    nix build .#lgx-portable      # package for Basecamp

Methods (`logoscore -c` or another module's `modules().pramadani_osm_consumer_module`):

| method | answer |
|--------|--------|
| `regions()` | the 72 regions |
| `mapFor(region)` | `{cid, checksum, version, size_bytes, hosts, source_url}` of the newest hosted snapshot |
| `download(region)` | `{path, md5, from, skipped}`; the file matches the registered MD5 |

To start your own module: copy this folder, change `name` in `metadata.json` (and the flake
description), and write your idea in `src/`. For the LP-0018 adoption count the module must be
your own idea in your own repository (see `docs/adoption.md`).
