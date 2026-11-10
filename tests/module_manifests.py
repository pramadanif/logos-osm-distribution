#!/usr/bin/env python3
"""Root module.json against the module manifests (plan H2) and module names against rival
catalogs (H19, ADR 0008). Exit 0 only when every check passes."""

import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# Names other LP-0018 catalogs already publish (their index.json, 2026-11-09).
RIVAL_NAMES = {"osm", "osm-app", "osm_app", "osm-consumer-app", "osm_consumer_example"}
PREFIX = "pramadani_osm_"
OWN = {"pramadani_osm_distribution", "pramadani_osm_sdk"}
# Modules of other projects ours may depend on; anything else must be one of ours.
EXTERNAL = {"lez_core"}

failures = []


def fail(msg):
    failures.append(msg)


def png_size(path):
    head = path.read_bytes()[:24]
    if head[:8] != b"\x89PNG\r\n\x1a\n":
        return None
    return struct.unpack(">II", head[16:24])


index = json.loads((ROOT / "module.json").read_text())
listed = {m["name"]: m for m in index["modules"]}
if len(listed) != len(index["modules"]):
    fail("module.json lists a module twice")

found = {}
for meta_path in sorted(ROOT.glob("**/metadata.json")):
    rel = meta_path.parent.relative_to(ROOT).as_posix()
    if rel.startswith((".deps", "target", "docs")) or "/result" in rel or "logos-rust-sdk-src" in rel:
        continue
    meta = json.loads(meta_path.read_text())
    found[meta["name"]] = (rel, meta)

for name, (rel, meta) in found.items():
    if name not in listed:
        fail(f"{rel}/metadata.json ({name}) is not in module.json")
    elif listed[name]["path"] != rel:
        fail(f"module.json path for {name} is {listed[name]['path']}, manifest is in {rel}")
    if not name.startswith(PREFIX):
        fail(f"{name}: module names start with {PREFIX} (H19)")
    if name in RIVAL_NAMES or name.replace("_", "-") in RIVAL_NAMES:
        fail(f"{name}: collides with a rival catalog name")
    if not re.fullmatch(r"\d+\.\d+\.\d+", meta.get("version", "")):
        fail(f"{name}: version is not X.Y.Z")
    if meta.get("main") != f"{name}_plugin":
        fail(f"{name}: main must be {name}_plugin")
    deps = set(meta.get("dependencies", [])) | set(meta.get("optional_dependencies", []))
    for d in deps:
        if d not in EXTERNAL and d not in OWN:
            fail(f"{name}: unknown dependency {d}")
    # Every dependency must be a flake input of the same name, or nix cannot resolve it.
    flake = (ROOT / rel / "flake.nix").read_text()
    for d in deps:
        if not re.search(rf"^\s*{re.escape(d)}\.url\s*=", flake, re.M):
            fail(f"{name}: dependency {d} has no flake input {d}.url")
    if meta.get("type") == "ui_qml":
        icon = ROOT / rel / meta.get("icon", "")
        if not icon.is_file() or png_size(icon) != (256, 256):
            fail(f"{name}: ui_qml icon must be a 256x256 PNG ({meta.get('icon')})")
        if not (ROOT / rel / "src" / meta.get("view", "")).is_file():
            fail(f"{name}: view {meta.get('view')} not found under src/")
        rep = meta.get("codegen", {}).get("rep")
        if not rep or not (ROOT / rel / rep).is_file():
            fail(f"{name}: codegen.rep missing")
        cmake = (ROOT / rel / "CMakeLists.txt").read_text()
        if rep and rep not in cmake:
            fail(f"{name}: CMakeLists.txt REP_FILE is not {rep}")
    impl = meta.get("codegen", {}).get("impl_header")
    if impl and not (ROOT / rel / impl).is_file():
        fail(f"{name}: impl_header {impl} not found")

for name in listed:
    if name not in found:
        fail(f"module.json lists {name} but no metadata.json has that name")

# Data folders follow the module name, so two modules never share one (H19, H38).
for rs in ROOT.glob("basecamp/*/rust-lib/src/lib.rs"):
    name = json.loads((rs.parents[2] / "metadata.json").read_text())["name"]
    if f'.join("{name}")' not in rs.read_text():
        fail(f"{name}: data folder is not instance_persistence_path/{name}")

for f in failures:
    print("FAIL", f)
print(f"{len(found)} manifests, {len(failures)} failures")
sys.exit(1 if failures else 0)
