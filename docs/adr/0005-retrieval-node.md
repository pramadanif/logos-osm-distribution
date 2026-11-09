# ADR 0005: Retrieval proof with an independent node (H3, W2)

Status: Accepted. VPS is a human gate (Phase F).

- A public Storage node on a VPS (spec: 2 vCPU, 2–4 GB RAM, 40 GB disk for the planned set or 80–100 GB for all 72 regions, docs/status/coverage-plan.md; public IP,
  the node's listen and discovery ports open) seeds every covered CID with `advertise=true`.
  `deploy/storage-node/setup.sh` installs a pinned `logos-storage-nim` release with a checksum and
  a systemd unit that restarts on boot; uptime is checked daily.
- CI `storage-retrieval` job starts a second, empty node, connects only through the network,
  fetches at least 3 covered CIDs with `GET /data/{cid}/network/stream`, hashes them and compares
  to the registry checksum. A missing node or network is a failure, never a skip.
- Disclosure (E71): hosting serves data from the host's node, so its network address is visible
  to peers. Stated in docs/security.md and docs/limitations.md.
