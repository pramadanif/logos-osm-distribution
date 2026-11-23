.pragma library
// Number and date formatting for the OSM Maps view (H36). Plain arithmetic and string slicing,
// no regular expressions: Qt's regex JIT is unavailable in Basecamp's hardened runtime, and a
// look-ahead thousands separator doubled commas in Qt 6 QML. One locale-free format everywhere.
// Tested by tests/qml/tst_format.qml with the Qt of the Basecamp build.

// 1536 -> "1.5 KiB".
function size(n) {
    if (n === undefined || n === null || isNaN(n))
        return "";
    var units = ["B", "KiB", "MiB", "GiB", "TiB"];
    var i = 0;
    var v = Number(n);
    while (v >= 1024 && i < units.length - 1) {
        v = v / 1024;
        i++;
    }
    return (i === 0 ? String(v) : v.toFixed(1)) + " " + units[i];
}

// 38400000 -> "38,400,000".
function grouped(n) {
    var s = String(Math.floor(Math.abs(Number(n))));
    var out = "";
    for (var i = 0; i < s.length; i++) {
        if (i > 0 && (s.length - i) % 3 === 0)
            out += ",";
        out += s.charAt(i);
    }
    return (Number(n) < 0 ? "-" : "") + out;
}

// 261101 -> "2026-11-01"; nothing for a missing snapshot.
function snapshot(s) {
    if (!s)
        return "";
    var t = String(s);
    while (t.length < 6)
        t = "0" + t;
    return "20" + t.substring(0, 2) + "-" + t.substring(2, 4) + "-" + t.substring(4, 6);
}

// Unix ms -> "2026-11-23 10:23 UTC", from the date's parts.
function time(ms) {
    if (!ms)
        return "";
    var d = new Date(Number(ms));
    var two = function (n) { return n < 10 ? "0" + n : String(n); };
    return d.getUTCFullYear() + "-" + two(d.getUTCMonth() + 1) + "-" + two(d.getUTCDate()) + " "
        + two(d.getUTCHours()) + ":" + two(d.getUTCMinutes()) + " UTC";
}

// "https://testnet.lez.logos.co/rpc" -> "testnet.lez.logos.co".
function host(url) {
    var u = String(url || "");
    var i = u.indexOf("://");
    if (i >= 0)
        u = u.substring(i + 3);
    var j = u.indexOf("/");
    return j >= 0 ? u.substring(0, j) : u;
}
