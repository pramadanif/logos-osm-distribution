# Geofabrik research (SC-−1.5b, H24, H26)

Checked 2026-11-09 against the live server. Commands are reproducible; outputs below are what was observed on that day.

## Dated snapshots

- `<region>-latest.osm.pbf` is an HTTP 307 redirect to the dated file of the day:
  `curl -sI https://download.geofabrik.de/asia/indonesia-latest.osm.pbf` gave
  `Location: https://download.geofabrik.de/asia/indonesia-261108.osm.pbf`.
  So the snapshot id can be read from the redirect without downloading the file.
- Dated names are `<region>-YYMMDD.osm.pbf`, each with a `.md5` next to it
  (`<md5>  <region>-YYMMDD.osm.pbf`).
- Retention seen in the directory listing of `asia/indonesia.html`:
  - daily files for roughly the last week (261103 … 261108);
  - first-of-month files for the current year (260801, 260901, 261001, …);
  - first-of-year files back to 2014 (140101 … 260101).
  Geofabrik's technical page says older files "are not usually shown but you can access them
  through the directory index" and "We delete these older files after a while".
- Decision input for the ADR: register the first-of-month snapshot (`YYMM01`) so the source URL
  and its `.md5` stay available for months, not days. Daily files would vanish within about a
  week, which breaks evaluator verification (H24). The verify script must still report entries
  whose snapshot Geofabrik no longer serves.

## Index shape (`index-v1-nogeom.json`, 554 ids on 2026-11-09)

Index ids and `parent` values do not match the prize ids one to one:

| Prize leaf | Index id | Index parent | PBF URL path |
|------------|----------|--------------|--------------|
| `us/california` | `us/california` | `north-america` | `north-america/us/california` |
| `india/central-zone` | `central-zone` | `india` | `asia/india/central-zone` |
| `china/guangdong` | `guangdong` | `china` | `asia/china/guangdong` |
| `russia/central-fed-district` | `central-fed-district` | `russia` | `russia/central-fed-district` |
| `georgia` (country) | not in set | `europe` | collides with `us/georgia` by last segment only |

Consequence: `regions/predefined.json` must store, per leaf, the prize id (our `region`), the
prize parent (`us`/`india`/`china`/`russia` or null), the level, and the Geofabrik index id. Never
derive one from the other by string slicing.

All 48 countries and 24 leaves resolve in the index. Two countries have no index id equal to the
prize name: see docs/status/conflicts.md.

## Sizes (for the coverage plan, SC-F.3)

- `indonesia-261108.osm.pbf`: 1,741,362,950 bytes (HEAD, Content-Length).
- `indonesia-260101.osm.pbf`: 1,686,629,405 bytes.
Indonesia alone is about 1.7 GB, so the SE Asia set is not small. Full table to be built in
docs/status/coverage-plan.md from HEAD requests only (no downloads).

## Politeness (H26)

- The technical page asks nothing specific beyond normal use; aegonmyy reported throttling of
  some cloud egress for large files. Rules for us: sequential downloads, resume with `Range`,
  local cache of PBF + MD5, no Geofabrik downloads in CI (fixtures only), live job manual.
