import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// OSM Maps (LP-0018). Every action is a job in the backend, which calls the
// pramadani_osm_distribution module. Text from the chain or from Storage is shown as plain text
// only (textFormat: Text.PlainText), never as rich text.
Item {
    id: root

    readonly property var backend: logos.module("pramadani_osm_app")
    property bool ready: false

    // Region rows: {region, name, level, parent, version, hosts}.
    property var regions: []
    // Region ids ticked for bulk hosting.
    property var selected: ({})
    property var settings: ({})
    property var jobs: ({})
    // Pending job id -> function(answerObject) run once when the job finishes.
    property var handlers: ({})
    // Region ids waiting to be hosted one after another (bulk host).
    property var hostQueue: []
    property string detail: ""

    readonly property color fg: "#e6edf3"
    readonly property color muted: "#8b949e"
    readonly property color good: "#56d364"
    readonly property color warn: "#f0883e"
    readonly property color bad: "#f85149"

    Connections {
        target: logos
        function onViewModuleReadyChanged(moduleName, isReady) {
            if (moduleName === "pramadani_osm_app") {
                root.ready = isReady && root.backend !== null;
                if (root.ready)
                    root.refresh();
            }
        }
    }
    Component.onCompleted: {
        root.ready = root.backend !== null && logos.isViewModuleReady("pramadani_osm_app");
        if (root.ready)
            root.refresh();
    }

    Connections {
        target: root.backend
        ignoreUnknownSignals: true
        function onJobsChanged() { root.onJobs(); }
    }

    // ---- helpers ----------------------------------------------------------------------------

    // 1536 -> "1.5 KiB". Plain arithmetic, no regular expressions.
    function formatSize(n) {
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

    // 261101 -> "2026-11-01".
    function formatSnapshot(s) {
        if (!s)
            return "not hosted";
        var t = String(s);
        while (t.length < 6)
            t = "0" + t;
        return "20" + t.substring(0, 2) + "-" + t.substring(2, 4) + "-" + t.substring(4, 6);
    }

    function parse(text) {
        try {
            return JSON.parse(text);
        } catch (e) {
            return {"ok": false, "code": "A003", "error": "unreadable answer"};
        }
    }

    function errorText(a) {
        return (a.code ? a.code + " " : "") + (a.error || "failed");
    }

    // Start a backend job; `then` gets the parsed answer when it is no longer submitting.
    function run(method, arg, then) {
        if (!root.ready)
            return;
        logos.watch(backend.call(method, arg === undefined ? "" : arg), function (id) {
            if (then) {
                var h = root.handlers;
                h[id] = then;
                root.handlers = h;
            }
            root.onJobs();
        }, function (error) {
            root.detail = "A001 the app backend did not answer: " + error;
        });
    }

    function onJobs() {
        root.jobs = parse(backend.jobs || "{}");
        var h = root.handlers;
        for (var id in h) {
            var j = root.jobs[id];
            if (j && j.state !== "submitting") {
                var f = h[id];
                delete h[id];
                f(parse(j.answer || "{}"));
            }
        }
        root.handlers = h;
    }

    function refresh() {
        run("settings", "", function (a) {
            if (a.ok)
                root.settings = a.data.settings;
        });
        run("discover", "", function (a) {
            if (!a.ok) {
                root.detail = errorText(a);
                return;
            }
            var rows = [];
            for (var i = 0; i < a.data.regions.length; i++) {
                var r = a.data.regions[i];
                rows.push({"region": r.region, "name": r.name, "level": r.level, "parent": r.parent || "",
                           "version": 0, "hosts": 0});
            }
            root.regions = rows;
            root.loadHosted();
        });
    }

    function loadHosted() {
        run("hosted_status", "", function (a) {
            if (!a.ok) {
                root.detail = errorText(a);
                return;
            }
            var by = {};
            for (var i = 0; i < a.data.regions.length; i++)
                by[a.data.regions[i].region] = a.data.regions[i];
            var rows = [];
            for (var k = 0; k < root.regions.length; k++) {
                var r = root.regions[k];
                var s = by[r.region];
                rows.push({"region": r.region, "name": r.name, "level": r.level, "parent": r.parent,
                           "version": s && s.version ? s.version : 0, "hosts": s ? s.hosts : 0});
            }
            root.regions = rows;
            if (a.data.errors.length > 0)
                root.detail = a.data.errors.length + " region accounts could not be read; first: "
                    + a.data.errors[0].code + " " + a.data.errors[0].error;
        });
    }

    function selectedIds() {
        var out = [];
        for (var k in root.selected)
            if (root.selected[k])
                out.push(k);
        return out;
    }

    function toggle(region, on) {
        var s = root.selected;
        s[region] = on;
        root.selected = s;
        root.selectedChanged();
    }

    // Bulk host: the module downloads and uploads the regions one by one, then registers each
    // group of up to 16 (the program's batch limit) in one transaction. One region uses `host`.
    readonly property int maxBatch: 16

    function hostSelected() {
        root.hostQueue = selectedIds();
        hostNext();
    }

    function hostNext() {
        if (root.hostQueue.length === 0) {
            loadHosted();
            return;
        }
        var q = root.hostQueue;
        var group = q.splice(0, root.maxBatch);
        root.hostQueue = q;
        var req = {"host": hostField.text};
        if (snapshotField.text.length > 0)
            req.snapshot = Number(snapshotField.text);
        if (group.length === 1) {
            req.region = group[0];
            run("host", JSON.stringify(req), function (a) { hostNext(); });
        } else {
            req.regions = group;
            run("host_batch", JSON.stringify(req), function (a) { hostNext(); });
        }
    }

    function describe(method, a) {
        if (!a.ok)
            return errorText(a);
        var d = a.data;
        if (method === "resolve")
            return d === null ? "not hosted"
                : "CID " + d.cid + "\nMD5 " + d.checksum + "\nsnapshot " + formatSnapshot(d.version)
                  + "\nsize " + formatSize(d.size_bytes) + "\nhosts " + d.hosts.length + "\nsource " + d.source_url;
        if (method === "fetch")
            return "saved " + d.path + "\nMD5 " + d.md5 + " (matches the registry)\nfrom " + d.from
                   + (d.skipped.length > 0 ? "\nskipped " + d.skipped.length + " entries that did not verify" : "");
        if (method === "check_updates") {
            var lines = [];
            for (var i = 0; i < d.length; i++)
                lines.push(d[i].region + ": registered " + formatSnapshot(d[i].registered) + ", Geofabrik "
                           + formatSnapshot(d[i].available) + (d[i].update_available ? "  update available" : ""));
            return lines.join("\n");
        }
        if (method === "host_batch")
            return "confirmed on-chain in one transaction: " + d.regions.join(", ");
        if (method === "host" || method === "import_file" || method === "register_entry" || method === "unhost")
            return "confirmed on-chain" + (d.cid ? "\nCID " + d.cid : "");
        return JSON.stringify(d, null, 2);
    }

    // ---- view -------------------------------------------------------------------------------

    Rectangle {
        anchors.fill: parent
        color: "#0d1117"
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 10

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: "OSM Maps"
                color: root.fg
                font.pixelSize: 20
            }
            Item { Layout.fillWidth: true }
            Text {
                text: root.ready ? "connected" : "connecting to the backend..."
                color: root.ready ? root.good : root.warn
                font.pixelSize: 12
            }
            Button {
                text: "Refresh"
                enabled: root.ready
                onClicked: root.refresh()
            }
        }

        TabBar {
            id: tabs
            Layout.fillWidth: true
            TabButton { text: "Regions" }
            TabButton { text: "Host" }
            TabButton { text: "Import" }
            TabButton { text: "Activity (" + Object.keys(root.jobs).length + ")" }
            TabButton { text: "Settings" }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabs.currentIndex

            // Regions: hosted status, resolve, download, update check.
            ColumnLayout {
                spacing: 6
                RowLayout {
                    TextField {
                        id: filter
                        placeholderText: "filter regions"
                        Layout.preferredWidth: 220
                    }
                    Button {
                        text: "Check updates for ticked"
                        enabled: root.ready && root.selectedIds().length > 0
                        onClicked: root.run("check_updates", JSON.stringify(root.selectedIds()), function (a) {
                            root.detail = root.describe("check_updates", a);
                        })
                    }
                    Text {
                        text: root.regions.length + " regions"
                        color: root.muted
                    }
                }
                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: root.regions
                    delegate: RowLayout {
                        required property var modelData
                        visible: filter.text.length === 0 || modelData.region.indexOf(filter.text) >= 0
                                 || modelData.name.toLowerCase().indexOf(filter.text.toLowerCase()) >= 0
                        height: visible ? implicitHeight : 0
                        width: ListView.view.width
                        CheckBox {
                            checked: root.selected[modelData.region] === true
                            onToggled: root.toggle(modelData.region, checked)
                        }
                        Text {
                            text: modelData.name + "  (" + modelData.region + ", " + modelData.level + ")"
                            textFormat: Text.PlainText
                            color: root.fg
                            Layout.preferredWidth: 360
                            elide: Text.ElideRight
                        }
                        Text {
                            text: root.formatSnapshot(modelData.version)
                                  + (modelData.hosts > 0 ? ", " + modelData.hosts + " hosts" : "")
                            color: modelData.version ? root.good : root.muted
                            Layout.preferredWidth: 180
                        }
                        Button {
                            text: "Details"
                            enabled: root.ready
                            onClicked: root.run("resolve", modelData.region, function (a) {
                                root.detail = modelData.region + "\n" + root.describe("resolve", a);
                            })
                        }
                        Button {
                            text: "Download"
                            enabled: root.ready && modelData.version > 0
                            onClicked: root.run("fetch", modelData.region, function (a) {
                                root.detail = modelData.region + "\n" + root.describe("fetch", a);
                            })
                        }
                    }
                }
            }

            // Host: one or many regions from Geofabrik's dated snapshot.
            ColumnLayout {
                spacing: 8
                Text {
                    text: "Downloads the dated Geofabrik snapshot, checks its MD5, uploads it to your Storage node, "
                          + "then registers it signed by your wallet. Each region is confirmed by reading the "
                          + "registry back."
                    color: root.muted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
                RowLayout {
                    Text { text: "Host account"; color: root.fg }
                    TextField {
                        id: hostField
                        text: root.settings.host || ""
                        placeholderText: "Public/... (your LEZ account)"
                        Layout.preferredWidth: 420
                    }
                }
                RowLayout {
                    Text { text: "Snapshot (YYMMDD, optional)"; color: root.fg }
                    TextField {
                        id: snapshotField
                        placeholderText: "first of this month"
                        validator: IntValidator { bottom: 0; top: 999999 }
                        Layout.preferredWidth: 160
                    }
                }
                Text {
                    text: "Ticked on the Regions tab: " + (root.selectedIds().length === 0 ? "none"
                          : root.selectedIds().join(", "))
                    textFormat: Text.PlainText
                    color: root.fg
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
                RowLayout {
                    Button {
                        text: root.hostQueue.length > 0 ? "Hosting... (" + root.hostQueue.length + " left)"
                              : "Host ticked regions"
                        enabled: root.ready && hostField.text.length > 0 && root.selectedIds().length > 0
                                 && root.hostQueue.length === 0
                        onClicked: root.hostSelected()
                    }
                    Button {
                        text: "Stop after current"
                        enabled: root.hostQueue.length > 0
                        onClicked: root.hostQueue = []
                    }
                }
                Item { Layout.fillHeight: true }
            }

            // Import a local file (e.g. one downloaded from another host).
            ColumnLayout {
                spacing: 8
                Text {
                    text: "Checks a local .osm.pbf against Geofabrik's published MD5 for the snapshot, uploads it "
                          + "to your Storage node (same file, same CID) and registers you as a host."
                    color: root.muted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
                TextField { id: importPath; placeholderText: "/path/to/region-261101.osm.pbf"; Layout.fillWidth: true }
                TextField { id: importRegion; placeholderText: "region id, e.g. kenya"; Layout.preferredWidth: 260 }
                TextField {
                    id: importSnapshot
                    placeholderText: "snapshot YYMMDD"
                    validator: IntValidator { bottom: 0; top: 999999 }
                    Layout.preferredWidth: 160
                }
                Button {
                    text: "Import and register"
                    enabled: root.ready && hostField.text.length > 0 && importPath.text.length > 0
                             && importRegion.text.length > 0 && importSnapshot.text.length === 6
                    onClicked: root.run("import_file", JSON.stringify({
                        "host": hostField.text, "path": importPath.text, "region": importRegion.text,
                        "snapshot": Number(importSnapshot.text)}), function (a) {
                        root.detail = root.describe("import_file", a);
                        root.loadHosted();
                    })
                }
                Text {
                    text: hostField.text.length === 0 ? "Set the host account on the Host tab first." : ""
                    color: root.warn
                }
                Item { Layout.fillHeight: true }
            }

            // Activity: every job and its state.
            ColumnLayout {
                RowLayout {
                    Button {
                        text: "Clear finished"
                        enabled: root.ready
                        onClicked: backend.clearFinished()
                    }
                }
                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    model: Object.keys(root.jobs)
                    delegate: ColumnLayout {
                        required property var modelData
                        readonly property var job: root.jobs[modelData] || ({})
                        width: ListView.view.width
                        Text {
                            text: job.method + "  " + (job.arg || "") + "  " + job.state
                            textFormat: Text.PlainText
                            color: job.state === "confirmed" ? root.good : job.state === "failed" ? root.bad : root.warn
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }
                        Text {
                            text: job.state === "submitting" ? "" : root.describe(job.method, root.parse(job.answer || "{}"))
                            textFormat: Text.PlainText
                            color: root.muted
                            wrapMode: Text.WrapAnywhere
                            Layout.fillWidth: true
                        }
                    }
                }
            }

            // Settings: network, registry program, Storage node, Geofabrik use, host account.
            ColumnLayout {
                spacing: 8
                GridLayout {
                    columns: 2
                    Text { text: "Sequencer RPC"; color: root.fg }
                    TextField { id: setRpc; text: root.settings.rpc || ""; Layout.preferredWidth: 420 }
                    Text { text: "Registry program"; color: root.fg }
                    TextField { id: setProgram; text: root.settings.program || ""; Layout.preferredWidth: 420 }
                    Text { text: "Storage API"; color: root.fg }
                    TextField { id: setStorage; text: root.settings.storage || ""; Layout.preferredWidth: 420 }
                    Text { text: "Host account"; color: root.fg }
                    TextField { id: setHost; text: root.settings.host || ""; Layout.preferredWidth: 420 }
                    Text { text: "Use Geofabrik"; color: root.fg }
                    CheckBox { id: setGeofabrik; checked: root.settings.geofabrik !== false }
                }
                Button {
                    text: "Save"
                    enabled: root.ready
                    onClicked: root.run("configure", JSON.stringify({
                        "rpc": setRpc.text, "program": setProgram.text, "storage": setStorage.text,
                        "host": setHost.text, "geofabrik": setGeofabrik.checked}), function (a) {
                        if (a.ok) {
                            root.settings = a.data;
                            root.refresh();
                        } else {
                            root.detail = root.errorText(a);
                        }
                    })
                }
                Text {
                    text: "Without Geofabrik the app still lists, downloads and verifies hosted regions; hosting "
                          + "a new snapshot needs it."
                    color: root.muted
                    wrapMode: Text.WordWrap
                    Layout.fillWidth: true
                }
                Item { Layout.fillHeight: true }
            }
        }

        // Last result, always plain text.
        ScrollView {
            Layout.fillWidth: true
            Layout.preferredHeight: 120
            visible: root.detail.length > 0
            TextArea {
                text: root.detail
                textFormat: TextEdit.PlainText
                readOnly: true
                wrapMode: TextEdit.WrapAnywhere
                color: root.fg
            }
        }
    }
}
