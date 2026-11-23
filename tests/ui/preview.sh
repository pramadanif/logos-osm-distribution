#!/usr/bin/env bash
# Screenshots of the OSM Maps view, one PNG per tab, rendered offscreen with sample data.
# Needs Qt 6 (`qml`) and a checkout of logos-co/logos-design-system at the revision the released
# Basecamp ships (its --version lists it; 493a243 for pre-release-4cea4bb-1433), so the view is
# compiled against the same controls as in Basecamp.
#   DESIGN_SYSTEM=<logos-design-system checkout> OUT=<dir> tests/ui/preview.sh
set -euo pipefail
: "${DESIGN_SYSTEM:?}" "${OUT:?}"
here="$(cd "$(dirname "$0")" && pwd)"; root="$(cd "$here/../.." && pwd)"
mkdir -p "$OUT"; OUT="$(cd "$OUT" && pwd)"
stage="$(mktemp -d)"; trap 'rm -rf "$stage"' EXIT
# The view's layout inside an .lgx: qml/Main.qml beside src/icons/.
mkdir -p "$stage/qml" "$stage/src"
cp "$root/basecamp/osm-app/src/qml/"*.qml "$root/basecamp/osm-app/src/qml/"*.js "$stage/qml/"
cp -R "$root/basecamp/osm-app/src/icons" "$stage/src/"
cp "$here/preview.qml" "$here/sample.json" "$stage/"
# The offscreen renderer cannot run LogosIcon's MultiEffect (a GPU shader), so the preview uses a
# copy of the design system whose icons are drawn as plain images. Only the preview; Basecamp
# renders the real effect.
cp -R "$DESIGN_SYSTEM/src/qml" "$stage/ds"
python3 - "$stage/ds/Logos/Controls/LogosIcon.qml" <<'PY'
import sys
p = sys.argv[1]; t = open(p).read()
t = t.replace("visible: false", "visible: true", 1).replace("MultiEffect {", "MultiEffect { visible: false;", 1)
open(p, "w").write(t)
PY
names=(regions host import activity settings)
for i in 0 1 2 3 4; do
  QML_XHR_ALLOW_FILE_READ=1 QT_QPA_PLATFORM=offscreen QSG_RHI_BACKEND=metal QT_QUICK_CONTROLS_STYLE=Basic \
    qml -I "$stage/ds" "$stage/preview.qml" -- "$i" "$stage/qml/Main.qml" "$OUT/${names[$i]}.png" 2> "$OUT/${names[$i]}.log" \
    || { echo "tab ${names[$i]} failed"; tail -20 "$OUT/${names[$i]}.log"; exit 1; }
  grep -E "Main.qml:[0-9]+" "$OUT/${names[$i]}.log" | head -10 || true
  echo "$OUT/${names[$i]}.png"
done
