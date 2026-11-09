# Coverage plan (SC-F.3)

Sizes are `Content-Length` of each region's `-latest.osm.pbf` (HEAD only, no download),
measured 2026-11-09. They change a little every day.

- All 72 regions: 60.4 GB.
- Candidate W7 set below: 30 entries, 26 countries, 10.5 GB.
- Storage node overhead measured on v0.5.3: an 8 MiB upload used 8.2 MiB of data dir (about 1.0x).
- VPS disk: 40 GB is enough for this set with room to re-host the next monthly snapshot
  alongside the old one; 80–100 GB if all 72 regions are ever hosted.

| Region | Size (MB) | Why |
|--------|-----------|-----|
| `us/pennsylvania` | 350 | decomposed depth |
| `us/georgia` | 358 | decomposed depth |
| `india/north-eastern-zone` | 110 | decomposed depth |
| `india/western-zone` | 222 | decomposed depth |
| `china/henan` | 52 | decomposed depth |
| `china/shandong` | 72 | decomposed depth |
| `russia/siberian-fed-district` | 590 | decomposed depth |
| `russia/northwestern-fed-district` | 657 | decomposed depth |
| `indonesia` | 1,741 | SE Asia (W7) |
| `malaysia-singapore-brunei` | 253 | SE Asia (W7) |
| `thailand` | 330 | SE Asia (W7) |
| `vietnam` | 332 | SE Asia (W7) |
| `philippines` | 613 | SE Asia (W7) |
| `ethiopia` | 140 | small country |
| `pakistan` | 158 | small country |
| `bulgaria` | 177 | small country |
| `egypt` | 179 | small country |
| `iran` | 230 | small country |
| `morocco` | 244 | small country |
| `peru` | 257 | small country |
| `south-korea` | 291 | small country |
| `hungary` | 329 | small country |
| `romania` | 333 | small country |
| `colombia` | 334 | small country |
| `greece` | 343 | small country |
| `chile` | 351 | small country |
| `belarus` | 352 | small country |
| `kenya` | 352 | small country |
| `bangladesh` | 356 | small country |
| `ireland-and-northern-ireland` | 418 | small country |

Full size table for all 72:

| Region | Size (MB) |
|--------|-----------|
| `china/henan` | 52 |
| `china/shandong` | 72 |
| `china/jiangsu` | 80 |
| `china/zhejiang` | 91 |
| `china/sichuan` | 108 |
| `india/north-eastern-zone` | 110 |
| `ethiopia` | 140 |
| `pakistan` | 158 |
| `china/guangdong` | 175 |
| `bulgaria` | 177 |
| `egypt` | 179 |
| `india/western-zone` | 222 |
| `india/northern-zone` | 225 |
| `iran` | 230 |
| `morocco` | 244 |
| `india/eastern-zone` | 248 |
| `malaysia-singapore-brunei` | 253 |
| `peru` | 257 |
| `south-korea` | 291 |
| `hungary` | 329 |
| `thailand` | 330 |
| `vietnam` | 332 |
| `romania` | 333 |
| `colombia` | 334 |
| `greece` | 343 |
| `us/pennsylvania` | 350 |
| `chile` | 351 |
| `belarus` | 352 |
| `kenya` | 352 |
| `india/central-zone` | 352 |
| `bangladesh` | 356 |
| `us/georgia` | 358 |
| `us/illinois` | 363 |
| `us/washington` | 366 |
| `ireland-and-northern-ireland` | 418 |
| `south-africa` | 424 |
| `portugal` | 428 |
| `argentina` | 435 |
| `denmark` | 498 |
| `us/new-york` | 499 |
| `switzerland` | 550 |
| `india/southern-zone` | 559 |
| `russia/siberian-fed-district` | 590 |
| `philippines` | 613 |
| `mexico` | 648 |
| `turkey` | 653 |
| `russia/northwestern-fed-district` | 657 |
| `us/florida` | 661 |
| `belgium` | 699 |
| `nigeria` | 715 |
| `us/texas` | 727 |
| `finland` | 776 |
| `russia/volga-fed-district` | 778 |
| `austria` | 815 |
| `ukraine` | 884 |
| `russia/central-fed-district` | 885 |
| `czech-republic` | 952 |
| `sweden` | 954 |
| `australia` | 974 |
| `us/california` | 1,335 |
| `norway` | 1,379 |
| `netherlands` | 1,410 |
| `spain` | 1,500 |
| `indonesia` | 1,741 |
| `brazil` | 2,104 |
| `poland` | 2,117 |
| `great-britain` | 2,199 |
| `italy` | 2,251 |
| `japan` | 2,573 |
| `germany` | 4,878 |
| `france` | 5,112 |
| `canada` | 6,510 |
