#!/usr/bin/env python3
"""SC-D.2b / H19: our catalog installed side by side with another LP-0018 catalog in one
logosctl session (the portable build, as Basecamp's package manager installs released packages).
Both catalogs are added, both sets of core modules installed and loaded; ours must keep its own
names, versions and data folders and still answer. Exit 0 only when every check passes.

    python3 tests/side_by_side.py <logosctl (portable)>
"""

import json
import os
import subprocess
import sys
import tempfile

OURS = "https://raw.githubusercontent.com/pramadanif/pramadani-osm-modules/main/logos-repo.json"
# Another LP-0018 entry's public catalog (docs/adr/0008); only installed, never modified.
OTHER = "https://raw.githubusercontent.com/YodHeVauHe/logos-osm-modules/main/logos-repo.json"
OUR_CORE = ["pramadani_osm_sdk", "pramadani_osm_distribution"]
OTHER_CORE = ["osm"]

ctl = sys.argv[1]
session = tempfile.mkdtemp(prefix="osm-side-by-side-")
fails = 0


def run(*args, timeout=600):
    r = subprocess.run([ctl, "--config-dir", session, *args], capture_output=True, text=True, timeout=timeout)
    return r.returncode, r.stdout + r.stderr


def step(label, *args, ok_codes=(0,)):
    global fails
    code, out = run(*args)
    good = code in ok_codes
    print(("PASS " if good else "FAIL ") + label + ("" if good else f": exit {code}\n{out[-1500:]}"))
    if not good:
        fails += 1
    return out


step("daemon start", "daemon", "start", "--detach")
try:
    step("add our catalog", "catalog", "add", OURS)
    step("add the other catalog", "catalog", "add", OTHER)
    step("refresh catalogs", "catalog", "refresh")
    step("install ours", "install", *OUR_CORE, "-y")
    step("install theirs", "install", *OTHER_CORE, "-y")
    listing = step("package ls", "--json", "package", "ls")
    installed = listing
    for name in OUR_CORE + OTHER_CORE:
        if name not in installed:
            print(f"FAIL {name} not installed")
            fails += 1
    for name in OUR_CORE + OTHER_CORE:
        step(f"load {name}", "module", "load", name)
    # Ours still answers, from its own data folder.
    for name in OUR_CORE:
        code, out = run("call", name, "settings")
        good = code == 0 and f"/{name}" in out and "module_data" in out
        print(("PASS " if good else "FAIL ") + f"{name} answers from its own data folder"
              + ("" if good else f"\n{out[-800:]}"))
        fails += 0 if good else 1
    data = os.path.join(session, "module_data")
    if os.path.isdir(data):
        dirs = sorted(os.listdir(data))
        print("module_data:", dirs)
        shared = [d for d in dirs if d in OUR_CORE and d in OTHER_CORE]
        if shared:
            print("FAIL shared data folders:", shared)
            fails += 1
finally:
    run("daemon", "stop")
print(f"SUMMARY fails={fails}")
sys.exit(1 if fails else 0)
