#!/usr/bin/env bash
# Builds tests/cpp/job_table_test.cpp against Qt 6 Core with ASan + UBSan and runs it.
# Needs a C++17 compiler and Qt 6 Core (pkg-config Qt6Core).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; root="$(cd "$here/../.." && pwd)"
out="$(mktemp -d)"; trap 'rm -rf "$out"' EXIT
if command -v pkg-config >/dev/null && pkg-config --exists Qt6Core; then
  qt_cflags="$(pkg-config --cflags Qt6Core)"; qt_libs="$(pkg-config --libs Qt6Core)"
elif command -v qtpaths6 >/dev/null || command -v qtpaths >/dev/null; then
  # macOS with Qt installed as frameworks (Homebrew), which ships no .pc files.
  libdir="$( (command -v qtpaths6 >/dev/null && qtpaths6 --query QT_INSTALL_LIBS) || qtpaths --query QT_INSTALL_LIBS)"
  qt_cflags="-F $libdir -I $libdir/QtCore.framework/Headers"; qt_libs="-F $libdir -framework QtCore -Wl,-rpath,$libdir"
else
  echo "Qt 6 Core not found (no pkg-config Qt6Core, no qtpaths)" >&2; exit 2
fi
# shellcheck disable=SC2086
"${CXX:-c++}" -std=c++17 -g -O1 -fno-omit-frame-pointer -fsanitize=address,undefined -fno-sanitize-recover=all \
  -I "$root/basecamp/osm-app/src" $qt_cflags \
  "$here/job_table_test.cpp" $qt_libs -o "$out/job_table_test"
"$out/job_table_test"
