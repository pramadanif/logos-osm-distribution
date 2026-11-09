#!/usr/bin/env bash
# Full lifecycle demo (SC-E.4): build LEZ v0.3.1 and the registry program from this checkout,
# start a standalone sequencer and two real Logos Storage nodes, deploy, then
# host -> query -> download (Geofabrik blocked for the download) -> verify the MD5.
# No dev-mode shortcut: RISC0_DEV_MODE must be unset or 0.
#
#   ./demo.sh                     # uses ./.demo as work dir, downloads the pinned Storage node
#   STORAGE_BIN=<logos-storage> WORK=<dir> ./demo.sh
#
# Needs: cargo, Docker, cargo-risczero 3.0.5 + `rzup install rust 1.97.0`, curl, python3, unzip.
# Ports used on 127.0.0.1: 3040 (sequencer), 8071/8072 (Storage), 8081/8082 (Storage API), 8099.
# no-required-input
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
if [ "${RISC0_DEV_MODE:-0}" != "0" ]; then echo "RISC0_DEV_MODE is set; the demo runs without shortcuts" >&2; exit 2; fi
export WORK="${WORK:-$root/.demo}"
mkdir -p "$WORK"
if [ -z "${STORAGE_BIN:-}" ]; then
  # Pinned Logos Storage release with checksum (VERSIONS.md: STORAGE).
  case "$(uname -s)-$(uname -m)" in
    Darwin-arm64) asset=logos-storage-darwin-arm64-v0.5.3.zip; sha=7f32f62a14e555399ee003f11d1e29bba9b6ff842d973f38b5b280f018c2f6ab ;;
    Linux-x86_64) asset=logos-storage-linux-amd64-v0.5.3.zip; sha=1086fae4f283cab882f133abe4d5a88cfae8d61edba9f0094c7f75b368ec9d56 ;;
    *) echo "no pinned Storage binary for $(uname -s)-$(uname -m); set STORAGE_BIN" >&2; exit 2 ;;
  esac
  zip="$WORK/$asset"
  [ -f "$zip" ] || curl -fsSL --retry 5 --retry-all-errors -o "$zip" "https://github.com/logos-storage/logos-storage-nim/releases/download/v0.5.3/$asset"
  python3 -I -c 'import hashlib,sys; h=hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest(); sys.exit(0 if h==sys.argv[2] else f"sha256 {h} != {sys.argv[2]}")' "$zip" "$sha"
  mkdir -p "$WORK/storage-bin" && unzip -o -q "$zip" -d "$WORK/storage-bin" && chmod +x "$WORK"/storage-bin/*
  STORAGE_BIN="$(ls "$WORK"/storage-bin/logos-storage-* | grep -v '\.zip$' | head -1)"
fi
export STORAGE_BIN
cleanup() { pkill -f "$WORK/seq-home" 2>/dev/null || true; pkill -f "$WORK/storage" 2>/dev/null || true; pkill -f "http.server 8099" 2>/dev/null || true; }
trap cleanup EXIT
# Linux CI can block Geofabrik through /etc/hosts; locally the download step already runs with --no-geofabrik.
BLOCK_GEOFABRIK="${BLOCK_GEOFABRIK:-0}" "$root/scripts/ci-e2e.sh"
echo "DEMO OK"
