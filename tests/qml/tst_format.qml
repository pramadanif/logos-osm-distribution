// H36: the OSM Maps formatters in the Qt that Basecamp is built with (CI runs this with the
// qtdeclarative of logos-module-builder's nixpkgs; see .github/workflows/ci.yml).
import QtQuick
import QtTest
import "../../basecamp/osm-app/src/qml/format.js" as Format

TestCase {
    name: "Format"

    function test_size() {
        compare(Format.size(0), "0 B")
        compare(Format.size(1023), "1023 B")
        compare(Format.size(1536), "1.5 KiB")
        compare(Format.size(50702303), "48.4 MiB")
        compare(Format.size(32212254720), "30.0 GiB")
        compare(Format.size(5 * 1099511627776), "5.0 TiB")
        compare(Format.size(undefined), "")
        compare(Format.size(NaN), "")
    }

    function test_grouped_never_doubles_separators() {
        compare(Format.grouped(5000000), "5,000,000")
        compare(Format.grouped(38400000), "38,400,000")
        compare(Format.grouped(999), "999")
        compare(Format.grouped(1000), "1,000")
        compare(Format.grouped(0), "0")
        compare(Format.grouped(-1234567), "-1,234,567")
        verify(Format.grouped(123456789012).indexOf(",,") < 0)
    }

    function test_snapshot() {
        compare(Format.snapshot(261101), "2026-11-01")
        compare(Format.snapshot(260105), "2026-01-05")
        compare(Format.snapshot(10101), "2001-01-01")
        compare(Format.snapshot(0), "")
        compare(Format.snapshot(null), "")
    }

    function test_time_is_utc_and_locale_free() {
        compare(Format.time(1795406603344), "2026-11-23 04:03 UTC")
        compare(Format.time(0), "")
    }

    function test_host() {
        compare(Format.host("https://testnet.lez.logos.co"), "testnet.lez.logos.co")
        compare(Format.host("http://127.0.0.1:8080/api/storage/v1"), "127.0.0.1:8080")
        compare(Format.host(""), "")
    }
}
