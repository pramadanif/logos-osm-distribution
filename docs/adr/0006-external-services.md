# ADR 0006: External services and network calls (P-R4, H9)

Status: Accepted. Every outbound call our code makes is listed here; the preflight greps code
for hostnames and fails on any not in this table.

| Call | Host | When | Can be disabled | Without it |
|------|------|------|-----------------|------------|
| Sequencer RPC | user setting (`networks.json` profile) | register, read registry | no (core) | nothing works, error says so |
| Storage node | user setting, default local node | host, download | no (core) | download falls back to Geofabrik only if not hosted |
| Geofabrik index `index-v1-nogeom.json` | download.geofabrik.de | discover | yes (`geofabrik = off`) | discover lists the 72 bundled regions plus registry state |
| Geofabrik PBF + `.md5` | download.geofabrik.de | host, import check, unhosted fallback | yes | hosted regions still download by CID |
| lez-faucet claimd | from `networks.json` | `scripts/fund-testnet.sh` only | yes (script is optional) | fund another way |

No analytics, telemetry or crash reporting. The offline test runs discover and download with
download.geofabrik.de blocked and must pass for hosted regions (SC-C.6).
