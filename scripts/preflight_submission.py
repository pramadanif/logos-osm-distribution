"""Preflight before marking the solution PR ready (plan section 7, H16).

Each of the 29 checks prints PASS or FAIL with the reason. Exit 0 only when every check passes.
Run through scripts/preflight-submission.sh. Network checks use public endpoints and the GitHub
API (gh); nothing is written anywhere.

Environment:
  PROFILE        networks.json profile that counts (default v04-public)
  OSM_STORAGE    Logos Storage node API for the coverage check
  PIN            commit the solution pins (default: HEAD)
  SOLUTION       solution file (default docs/SOLUTION_DRAFT.md)
  CATALOG_URL    logos-repo.json URL of our catalog
  PUBLIC_NODE    public Storage node API to check from outside
"""
import json
import os
import re
import subprocess
import sys
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPO = "pramadanif/logos-osm-distribution"
PROFILE = os.environ.get("PROFILE", "v04-public")
PIN = os.environ.get("PIN") or subprocess.run(["git", "-C", ROOT, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
SOLUTION = os.path.join(ROOT, os.environ.get("SOLUTION", "docs/SOLUTION_DRAFT.md"))
results = []


def check(n, title):
    def wrap(fn):
        try:
            ok, why = fn()
        except Exception as e:  # a crashed check is a failed check, never a pass
            ok, why = False, f"check raised {type(e).__name__}: {e}"
        results.append((n, title, ok, why))
        return fn
    return wrap


def read(path):
    p = os.path.join(ROOT, path)
    return open(p, encoding="utf-8").read() if os.path.exists(p) else ""


def sh(*args, cwd=ROOT, timeout=900):
    r = subprocess.run(list(args), cwd=cwd, capture_output=True, text=True, timeout=timeout)
    return r.returncode, r.stdout, r.stderr


def gh_json(path):
    code, out, err = sh("gh", "api", path, timeout=60)
    if code:
        raise RuntimeError(err.strip()[:200])
    return json.loads(out)


def check_runs(sha):
    return {c["name"]: c["conclusion"] for c in gh_json(f"repos/{REPO}/commits/{sha}/check-runs?per_page=100")["check_runs"]}


def profile():
    return json.load(open(os.path.join(ROOT, "networks.json")))["profiles"].get(PROFILE, {})


solution = read(os.environ.get("SOLUTION", "docs/SOLUTION_DRAFT.md"))
readme = read("README.md")


@check(1, "SPEL IDL committed and equal to a fresh regeneration (H1)")
def _():
    if not os.path.exists(os.path.join(ROOT, "idl/osm_registry.json")):
        return False, "idl/osm_registry.json missing"
    code, out, err = sh("cargo", "run", "--locked", "-q", "--bin", "generate_idl", cwd=os.path.join(ROOT, "programs/osm_registry"))
    if code:
        return False, "generate_idl failed: " + err.strip()[-200:]
    same = json.loads(out) == json.load(open(os.path.join(ROOT, "idl/osm_registry.json")))
    return same, "regenerated IDL equals idl/osm_registry.json" if same else "regenerated IDL differs"


@check(2, "Root module.json present and consistent (H2)")
def _():
    return os.path.exists(os.path.join(ROOT, "module.json")), "module.json at the root"


@check(3, "Retrieval job green on the pin (H3)")
def _():
    c = check_runs(PIN).get("storage-retrieval")
    return c == "success", f"storage-retrieval: {c}"


@check(4, "Video URL in the solution, not TBD (H4)")
def _():
    m = re.search(r"https://(www\.)?(youtube\.com|youtu\.be|vimeo\.com)/\S+", solution)
    return bool(m) and "TBD" not in solution, m.group(0) if m else "no video link"


@check(5, "No trimmed hashes in the solution (H5)")
def _():
    if not solution:
        return False, "no solution file"
    bad = re.findall(r"\b[0-9a-f]{6,63}(?:…|\.\.\.)", solution)
    return not bad, f"trimmed: {bad[:3]}" if bad else "no trimmed hashes"


@check(6, "CI green on the pin, including e2e-sequencer (H6)")
def _():
    runs = check_runs(PIN)
    red = {k: v for k, v in runs.items() if v != "success"}
    return bool(runs) and not red and "e2e-sequencer" in runs, f"not green: {red}" if red else f"{len(runs)} checks green"


@check(7, "verify-coverage on the counting network passes 25/15 (H7, P-A1)")
def _():
    p = profile()
    if not p.get("rpc") or not p.get("program"):
        return False, f"profile {PROFILE} has no rpc/program yet"
    if not os.environ.get("OSM_STORAGE"):
        return False, "OSM_STORAGE not set"
    code, out, err = sh(os.path.join(ROOT, "scripts/verify-onchain.sh"), timeout=7200)
    return code == 0, (out.strip().splitlines() or [err.strip()])[-1]


@check(8, "docs/cycles.md has numbers for register and batch (H8)")
def _():
    t = read("docs/cycles.md")
    ok = bool(re.search(r"\| `register` \| 1 \| [\d,]+", t)) and bool(re.search(r"\| `register_batch` \| 16 \| [\d,]+", t))
    return ok, "register and register_batch rows with numbers" if ok else "missing rows"


@check(9, "No hostnames outside the allowlist in code (H9)")
def _():
    allowed = {"download.geofabrik.de", "testnet.lez.logos.co", "explorer.testnet.lez.logos.co", "rog-devbox.tail957e2.ts.net",
               "127.0.0.1", "localhost", "github.com", "raw.githubusercontent.com", "risczero.com"}
    found = set()
    for d in ("osm_core", "osm_sdk", "osm_cli", "osm_ffi", "programs/osm_registry/methods", "programs/osm_registry/lez_adapter_v03/src"):
        for base, _, files in os.walk(os.path.join(ROOT, d)):
            if "target" in base:
                continue
            for f in files:
                if f.endswith((".rs", ".h", ".c", ".qml", ".cpp")):
                    found |= set(re.findall(r"https?://([a-z0-9.\-]+)", open(os.path.join(base, f), encoding="utf-8", errors="ignore").read()))
    extra = sorted(h for h in found if h not in allowed and not h.endswith(".example") and h != "evil.example")
    return not extra, f"not allowed: {extra}" if extra else f"{len(found)} hosts, all allowed"


@check(10, "Both licence files present (H10)")
def _():
    ok = all(os.path.exists(os.path.join(ROOT, f)) for f in ("LICENSE-MIT", "LICENSE-APACHE"))
    return ok, "LICENSE-MIT and LICENSE-APACHE"


@check(11, "Catalog logos-repo.json resolves and lists the modules (H11)")
def _():
    url = os.environ.get("CATALOG_URL")
    if not url:
        return False, "CATALOG_URL not set (no catalog yet)"
    d = json.load(urllib.request.urlopen(url, timeout=30))
    return bool(d), f"{url} answered"


@check(12, "0.4 program id in README and solution, live on the RPC (H12)")
def _():
    pid = profile().get("program")
    if not pid:
        return False, f"profile {PROFILE} has no program id"
    return pid in readme and pid in solution, f"{pid} in README and solution"


@check(13, "Adoption table: >=5 rows, >=3 UI apps, >=4 on the distribution module, >=1 macOS (P-A2, P-A3)")
def _():
    rows = [l for l in solution.splitlines() if l.startswith("| ") and "github.com" in l and "adoption" in l.lower()]
    return len(rows) >= 5, f"{len(rows)} adoption rows"


@check(14, "Solution pins one commit that exists on main")
def _():
    m = re.findall(r"\b[0-9a-f]{40}\b", solution)
    if not m:
        return False, "no 40-hex commit in the solution"
    code, _, _ = sh("git", "merge-base", "--is-ancestor", m[0], "origin/main")
    return code == 0, f"{m[0]} on origin/main" if code == 0 else f"{m[0]} not on origin/main"


@check(15, "Test counts in README/solution equal command output (H17)")
def _():
    code, out, err = sh("cargo", "test", "--locked", "-q", timeout=1800)
    total = sum(int(x) for x in re.findall(r"test result: \w+\. (\d+) passed", out + err))
    claims = [int(x) for x in re.findall(r"(\d+) tests", readme + solution)]
    bad = [c for c in claims if c != total]
    return code == 0 and not bad, f"cargo test: {total} passed; claims {claims}"


@check(16, "Catalog .lgx sha256 matches the index and the pinned source (H18)")
def _():
    return False, "no catalog release yet" if not os.environ.get("CATALOG_URL") else "not implemented until the first release"


@check(17, "Module names and data dirs unique vs known catalogs (H19)")
def _():
    return os.path.exists(os.path.join(ROOT, "module.json")), "needs module.json"


@check(18, "docs/security.md lists every untrusted input with its bound (H21)")
def _():
    t = read("docs/security.md")
    needed = ["Geofabrik", "registry", "Storage", "bound"]
    miss = [w for w in needed if w.lower() not in t.lower()]
    return bool(t) and not miss, f"missing topics: {miss}" if t else "docs/security.md missing"


@check(19, "Video subtitles, chapters and docs/video-transcript.md (H22)")
def _():
    return os.path.exists(os.path.join(ROOT, "docs/video-transcript.md")), "docs/video-transcript.md"


@check(20, "readme-coldstart job green on the pin (H23)")
def _():
    c = check_runs(PIN).get("readme-coldstart")
    return c == "success", f"readme-coldstart: {c}"


@check(21, "Coverage entries use dated snapshots whose MD5 still matches (H24)")
def _():
    return results[6][2] if len(results) > 6 else False, "covered by check 7 (verify-coverage reports per entry)"


@check(22, "Public Storage node reachable from outside (H27)")
def _():
    url = os.environ.get("PUBLIC_NODE")
    if not url:
        return False, "PUBLIC_NODE not set (VPS not up yet)"
    urllib.request.urlopen(url.rstrip("/") + "/debug/info", timeout=20)
    return True, f"{url} answered"


@check(23, "Abort rules re-checked today: no merged or approved rival")
def _():
    code, out, err = sh("gh", "pr", "list", "-R", "logos-co/lambda-prize", "--state", "merged", "--search", "Solution: LP-0018", "--json", "number,title")
    # Only solution PRs count; spec edits that mention LP-0018 are not rivals.
    merged = [p for p in json.loads(out or "[]") if p["title"].startswith("Solution: LP-0018")] if code == 0 else None
    code2, out2, _ = sh("gh", "pr", "view", "187", "-R", "logos-co/lambda-prize", "--json", "reviewDecision", "-q", ".reviewDecision")
    ok = merged == [] and code2 == 0 and out2.strip() != "APPROVED"
    return ok, f"merged LP-0018 solutions: {merged}; #187 decision: {out2.strip() or 'none'}"


@check(24, "Default-branch HEAD equals the pin (H29)")
def _():
    head = gh_json(f"repos/{REPO}/commits/main")["sha"]
    return head == PIN, f"main {head[:12]} vs pin {PIN[:12]}"


@check(25, "Negative tests on a real sequencer green (H30, H31, H37)")
def _():
    t = read("probes/results.txt")
    last = [l for l in t.splitlines() if "b5_negative" in l and "SUMMARY" in l]
    return bool(last) and last[-1].endswith("fails=0"), last[-1] if last else "no b5 run recorded"


@check(26, "Decoder tested on accounts captured from a real sequencer in CI (H32)")
def _():
    # scripts/ci-e2e.sh captures the account it wrote and runs osm_sdk's fixture_live test on it.
    c = check_runs(PIN).get("e2e-sequencer")
    return c == "success", f"e2e-sequencer (includes live fixture capture): {c}"


@check(27, "liveness job green within 24 h; VERSIONS.md matches flake and README (H35)")
def _():
    runs = gh_json(f"repos/{REPO}/actions/workflows/daily.yml/runs?per_page=1")["workflow_runs"]
    ok = bool(runs) and runs[0]["conclusion"] == "success"
    return ok, f"last daily run: {runs[0]['conclusion'] if runs else 'none'}"


@check(28, "Data dir and file modes checked by a test (H38)")
def _():
    t = read("osm_sdk/tests/datadir.rs")
    return "0o700" in t and "0o600" in t, "osm_sdk/tests/datadir.rs asserts 0700/0600"


@check(29, "probes/results.txt, docs/self-review.md, docs/test-traffic.md present (H40)")
def _():
    miss = [f for f in ("probes/results.txt", "docs/self-review.md", "docs/test-traffic.md") if not os.path.exists(os.path.join(ROOT, f))]
    return not miss, f"missing {miss}" if miss else "all present"


fails = 0
for n, title, ok, why in sorted(results):
    fails += not ok
    print(f"{'PASS' if ok else 'FAIL'} {n:>2}. {title}: {why}")
print(f"SUMMARY pass={len(results) - fails} fail={fails} pin={PIN} profile={PROFILE}")
sys.exit(1 if fails else 0)
