#!/usr/bin/env bash
# SC-A.5: the 0.4 actor registry end to end on fresh infrastructure. Builds a standalone LEZ
# v0.4.0-rc1 sequencer and wallet, the guest from this commit (docker) and osm-chain-v04, then
# runs scripts/localnet-v04.sh (deploy + probes/b5_negative_v04.sh). Used by CI; runnable locally.
#
#   WORK=<dir> scripts/ci-e2e-v04.sh
# Needs cargo, docker, cargo-risczero 3.0.5, git, python3, perl. Every failure exits non-zero.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
: "${WORK:?}"
mkdir -p "$WORK"; WORK="$(cd "$WORK" && pwd)"
for t in cargo docker git python3 perl; do command -v "$t" >/dev/null || { echo "missing tool: $t" >&2; exit 1; }; done
cargo risczero --version >/dev/null || { echo "missing tool: cargo-risczero" >&2; exit 1; }

LEZ_TAG=v0.4.0-rc1; LEZ_SHA=1806e873ea2ac0e1b608aa7cf2f9b6e1fb12db21   # VERSIONS.md LEZ_V04
lez="$WORK/lez-$LEZ_TAG"
if [ ! -d "$lez/.git" ]; then
  mkdir -p "$lez"
  git -C "$lez" init -q
  git -C "$lez" remote add origin https://github.com/logos-blockchain/logos-execution-zone.git
  git -C "$lez" fetch -q --depth 1 origin "refs/tags/$LEZ_TAG:refs/tags/$LEZ_TAG"
  git -C "$lez" -c advice.detachedHead=false checkout -q "$LEZ_TAG"
fi
[ "$(git -C "$lez" rev-parse HEAD)" = "$LEZ_SHA" ] || { echo "LEZ $LEZ_TAG is not $LEZ_SHA" >&2; exit 1; }

echo "::group::build LEZ 0.4-rc sequencer + wallet"
for attempt in 1 2 3; do
  if (cd "$lez" && cargo build --release --locked --features standalone -p sequencer_service && cargo build --release --locked -p wallet); then break; fi
  [ "$attempt" -lt 3 ] || { echo "LEZ build failed 3 times" >&2; exit 1; }
  echo "LEZ build attempt $attempt failed; retrying in 60 s"; sleep 60
done
echo "::endgroup::"

echo "::group::build actor registry (docker) and osm-chain-v04"
(cd "$root" && cargo risczero build --manifest-path programs/osm_registry_v04/guest/Cargo.toml)
guest="$root/programs/osm_registry_v04/guest/target/riscv32im-risc0-zkvm-elf/docker/osm_registry_v04.bin"
# The wallet crates build rapidsnark; reuse the copy the LEZ build already fetched.
snark=$(find "$lez/target/release/build" -path '*rust-rapidsnark-*/out/rapidsnark' -type d | head -1)
arch=$(uname -m); [ "$arch" = arm64 ] && arch=aarch64
if [ -n "$snark" ] && [ -d "$snark/$arch" ]; then export RAPIDSNARK_LIB_DIR="$snark/$arch"; fi
(cd "$root/programs/osm_registry_v04/client" && cargo build --release --locked)
echo "::endgroup::"

LEZ_DIR="$lez" GUEST_BIN="$guest" CHAIN="$root/programs/osm_registry_v04/client/target/release/osm-chain-v04" \
  WORK="$WORK/run" "$root/scripts/localnet-v04.sh"
