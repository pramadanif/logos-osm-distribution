#!/usr/bin/env python3
"""Headless load test of the core modules (SC--1.4): a fresh logosctl session installs every
.lgx in DROP, loads pramadani_osm_sdk and pramadani_osm_distribution, and calls methods that
need no wallet. Each answer must be the module's JSON envelope with "ok": true, and the
answers must carry real data (72 regions, API version 1, an answer from the public v0.3
testnet). Exit 0 only when every call passes.

    python3 tests/module_load.py <logosctl> <drop dir>
"""

import json
import subprocess
import sys
import tempfile

ctl, drop = sys.argv[1], sys.argv[2]
# china/henan 261101 on the public v0.3 testnet, file on the project's public Storage node.
HENAN_CID = "zDvZRwzmA5LrY5Be279q4gpJqxk6v6XrS59rHkKakguRwwFPEMfH"
session = tempfile.mkdtemp(prefix="osm-logosctl-")
fails = 0


def run(*args, timeout=300):
    r = subprocess.run([ctl, "--config-dir", session, *args], capture_output=True, text=True, timeout=timeout)
    return r.returncode, r.stdout + r.stderr


def must(label, *args):
    global fails
    code, out = run(*args)
    if code != 0:
        print(f"FAIL {label}: exit {code}\n{out[-2000:]}")
        fails += 1
    else:
        print(f"PASS {label}")
    return out


def envelope(out):
    """The module's answer: the last line of the call output that parses as a JSON object with
    an "ok" key (logosctl may wrap it in its own JSON, or print it as a JSON string)."""
    for line in reversed(out.strip().splitlines()):
        line = line.strip()
        for cand in (line,):
            try:
                v = json.loads(cand)
            except ValueError:
                continue
            for _ in range(3):
                if isinstance(v, dict) and "ok" in v:
                    return v
                if isinstance(v, dict):
                    inner = v.get("result", v.get("value", v.get("data")))
                    v = json.loads(inner) if isinstance(inner, str) else inner
                elif isinstance(v, str):
                    v = json.loads(v)
                else:
                    break
    return None


def call(label, module, method, *args, check):
    global fails
    code, out = run("call", module, method, *args)
    env = envelope(out) if code == 0 else None
    ok = env is not None and env.get("ok") is True
    try:
        ok = ok and bool(check(env["data"]))
    except Exception as e:  # a missing field is a failed check
        ok = False
        out += f"\ncheck raised {e!r}"
    print(("PASS " if ok else "FAIL ") + label + ("" if ok else f": exit {code}\n{out[-2000:]}"))
    if not ok:
        fails += 1


must("daemon start", "daemon", "start", "--detach")
try:
    must("install drop", "install", "--dir", drop, "-y")
    for m in ("pramadani_osm_sdk", "pramadani_osm_distribution"):
        must(f"load {m}", "module", "load", m)
        call(f"{m}.version", m, "version", check=lambda d: d["api"] == 1)
        call(f"{m}.settings", m, "settings", check=lambda d: d["data_dir"] and m in d["data_dir"])
        # Without Geofabrik, so the test does not depend on download.geofabrik.de answering within
        # logosctl's 20 s call timeout; discovery then comes from the predefined set (P-R4).
        call(f"{m}.configure geofabrik off", m, "configure", '{"geofabrik": false}', check=lambda d: d["geofabrik"] is False)
        call(f"{m}.discover offline", m, "discover", check=lambda d: len(d["regions"]) + len(d["missing"]) == 72)
        # Real reads from the public v0.3 testnet (the modules' default network): kenya is not
        # registered there, china/henan is (docs/test-traffic.md, 2026-11-23).
        call(f"{m}.resolve kenya", m, "resolve", "kenya", check=lambda d: d is None or "cid" in d)
        call(f"{m}.resolve china/henan", m, "resolve", "china/henan", check=lambda d: d["cid"] == HENAN_CID)
    # Registry query by CID through the distribution module (P-F7).
    call("pramadani_osm_distribution.lookup cid", "pramadani_osm_distribution", "lookup",
         json.dumps({"cid": HENAN_CID}), check=lambda d: [e["region"] for e in d["entries"]] == ["china/henan"])
    # The embedding example: a module of its own that reaches the registry only through
    # pramadani_osm_sdk, with no distribution app involved (prize: SDK usable without it).
    must("load pramadani_osm_consumer_module", "module", "load", "pramadani_osm_consumer_module")
    call("pramadani_osm_consumer_module.mapFor china/henan", "pramadani_osm_consumer_module", "mapFor",
         "china/henan", check=lambda d: d["cid"] == HENAN_CID)
finally:
    run("daemon", "stop")
print(f"SUMMARY fails={fails}")
sys.exit(1 if fails else 0)
