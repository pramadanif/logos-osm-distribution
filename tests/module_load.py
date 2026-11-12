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
        call(f"{m}.discover", m, "discover", check=lambda d: len(d["regions"]) + len(d["missing"]) == 72)
        call(f"{m}.settings", m, "settings", check=lambda d: d["data_dir"] and m in d["data_dir"])
        # A real read from the public v0.3 testnet: kenya is not registered there, so null.
        call(f"{m}.resolve kenya", m, "resolve", "kenya", check=lambda d: d is None or "cid" in d)
finally:
    run("daemon", "stop")
print(f"SUMMARY fails={fails}")
sys.exit(1 if fails else 0)
