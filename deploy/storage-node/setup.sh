#!/usr/bin/env bash
# Public Logos Storage node for LP-0018 coverage (ADR 0005, H3, H27). Ubuntu 22.04/24.04, run with
# sudo. Idempotent: re-running upgrades the binary to the pinned version and restarts the unit.
#
#   sudo deploy/storage-node/setup.sh <public-ip> [listen-port]
#
# - runs as the unprivileged user `osmstorage`, data in /var/lib/osm-storage (0700)
# - REST API on 127.0.0.1:8080 only; only the libp2p port (default 8070/tcp) is opened
# - joins the Logos Storage testnet preset; restarts on failure and on boot
# - prints the node's peer id and SPR at the end
set -euo pipefail
[ "$(id -u)" = 0 ] || { echo "run with sudo" >&2; exit 2; }
ip="${1:?usage: setup.sh <public-ip> [listen-port]}"; port="${2:-8070}"
version=v0.5.3
case "$(uname -m)" in
  x86_64)  asset=logos-storage-linux-amd64-$version.zip; sha=1086fae4f283cab882f133abe4d5a88cfae8d61edba9f0094c7f75b368ec9d56 ;;
  aarch64) asset=logos-storage-linux-arm64-$version.zip; sha=ecbc32536565096b75b2330dab4ff47a993714a1f3ce345099396968bc69796b ;;
  *) echo "unsupported architecture $(uname -m)" >&2; exit 2 ;;
esac
apt-get update -qq && apt-get install -y -qq curl unzip python3 >/dev/null
id osmstorage >/dev/null 2>&1 || useradd --system --home /var/lib/osm-storage --shell /usr/sbin/nologin osmstorage
install -d -o osmstorage -g osmstorage -m 0700 /var/lib/osm-storage /var/lib/osm-storage/data
install -d -m 0755 /opt/osm-storage
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
curl -fsSL --retry 5 --retry-all-errors -o "$tmp/$asset" "https://github.com/logos-storage/logos-storage-nim/releases/download/$version/$asset"
echo "$sha  $tmp/$asset" | sha256sum -c -
unzip -q -o "$tmp/$asset" -d "$tmp/x"
bin="$(find "$tmp/x" -maxdepth 1 -type f -name 'logos-storage-*' ! -name '*.zip' | head -1)"
install -m 0755 "$bin" /opt/osm-storage/logos-storage
cat > /etc/systemd/system/osm-storage.service <<UNIT
[Unit]
Description=Logos Storage node for LP-0018 OSM coverage
After=network-online.target
Wants=network-online.target

[Service]
User=osmstorage
Group=osmstorage
UMask=0077
ExecStart=/opt/osm-storage/logos-storage --network=logos.test --data-dir=/var/lib/osm-storage/data \
  --api-bindaddr=127.0.0.1 --api-port=8080 --listen-port=$port --nat=extip:$ip --log-level=info
Restart=always
RestartSec=5
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/osm-storage
PrivateTmp=true

[Install]
WantedBy=multi-user.target
UNIT
systemctl daemon-reload
systemctl enable --now osm-storage.service
systemctl restart osm-storage.service
if command -v ufw >/dev/null && ufw status | grep -q "Status: active"; then ufw allow "$port/tcp" >/dev/null; fi
for _ in $(seq 1 60); do curl -fsS http://127.0.0.1:8080/api/storage/v1/debug/info >/dev/null 2>&1 && break; sleep 2; done
curl -fsS http://127.0.0.1:8080/api/storage/v1/debug/info | python3 -I -c 'import json,sys; d=json.load(sys.stdin); print("peer id:", d["id"]); print("addrs:", d["addrs"]); print("spr:", d["spr"])'
echo "OK: osm-storage.service running; open TCP $port in the provider firewall if it has one."
