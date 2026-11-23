import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Logos.Theme
import Logos.Controls
import Logos.Icons

// OSM Maps (LP-0018), built on the Logos Design System so it looks like the rest of Basecamp.
//
// Every action is a job in the backend, which calls the pramadani_osm_distribution module; the
// view only renders job states (submitting / confirmed / failed). Text that comes from the chain,
// from Storage or from Geofabrik is shown as plain text only: LogosText and LogosCopyableText
// default to Text.PlainText, and nothing here opts into StyledText.
Item {
    id: root

    readonly property var backend: logos.module("pramadani_osm_app")
    property bool ready: false

    // Region rows: {region, name, level, parent, version, hosts}.
    property var regions: []
    property bool loadingRegions: false
    // Region ids ticked for hosting.
    property var selected: ({})
    property int selectedCount: 0
    property var settings: ({})
    property var jobs: ({})
    // Pending job id -> function(answerObject), run once when the job finishes.
    property var handlers: ({})
    // Region ids still to be hosted (bulk host).
    property var hostQueue: []
    property string problem: ""

    // The region open in the detail panel, and what the registry says about it.
    property string openRegion: ""
    property var openEntry: null
    property bool openLoading: false
    property string openResult: ""
    property bool openResultOk: true

    // 0 all, 1 hosted, 2 not hosted.
    property int filter: 0
    // A decomposed country to narrow the list to ("" for all), and what the registry holds under it.
    property string parentFilter: ""
    property string parentInfo: ""
    property string searchInfo: ""
    // Geofabrik's current snapshot per region and whether the registered one is older, from one
    // update check over all regions; empty until the user asks.
    property var available: ({})
    property bool checkingVersions: false
    // What the configured sequencer and Storage node answered (E77): real probes, not assumptions.
    property var services: ({})
    property bool probing: false

    Rectangle {
        anchors.fill: parent
        color: Theme.palette.background
    }

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
            return "";
        var t = String(s);
        while (t.length < 6)
            t = "0" + t;
        return "20" + t.substring(0, 2) + "-" + t.substring(2, 4) + "-" + t.substring(4, 6);
    }

    // "https://testnet.lez.logos.co" -> "testnet.lez.logos.co".
    function hostOf(url) {
        var u = String(url || "");
        var i = u.indexOf("://");
        if (i >= 0)
            u = u.substring(i + 3);
        var j = u.indexOf("/");
        return j >= 0 ? u.substring(0, j) : u;
    }

    // Unix ms -> "2026-11-23 10:23 UTC", built from the date's parts (no locale formatting).
    function formatTime(ms) {
        if (!ms)
            return "";
        var d = new Date(Number(ms));
        var two = function (n) { return n < 10 ? "0" + n : String(n); };
        return d.getUTCFullYear() + "-" + two(d.getUTCMonth() + 1) + "-" + two(d.getUTCDate()) + " "
            + two(d.getUTCHours()) + ":" + two(d.getUTCMinutes()) + " UTC";
    }

    function probe() {
        root.probing = true;
        run("status", "", function (a) {
            root.probing = false;
            if (a.ok)
                root.services = a.data;
            else
                root.problem = errorText(a);
        });
    }

    function parse(text) {
        try {
            return JSON.parse(text);
        } catch (e) {
            return {"ok": false, "code": "A003", "error": "unreadable answer"};
        }
    }

    function errorText(a) {
        var e = a.error || "failed";
        return a.code && e.indexOf(a.code) !== 0 ? a.code + " " + e : e;
    }

    // Start a backend job; `then` gets the parsed answer once it is no longer submitting.
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
            root.problem = "A001 the app backend did not answer: " + error;
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
        root.probe();
        run("settings", "", function (a) {
            if (a.ok)
                root.settings = a.data.settings;
        });
        root.loadingRegions = true;
        run("discover", "", function (a) {
            if (!a.ok) {
                root.loadingRegions = false;
                root.problem = errorText(a);
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
            root.loadingRegions = false;
            if (!a.ok) {
                root.problem = errorText(a);
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
                root.problem = a.data.errors.length + " region accounts could not be read; first: "
                    + a.data.errors[0].code + " " + a.data.errors[0].error;
        });
    }

    function hostedCount() {
        var n = 0;
        for (var i = 0; i < regions.length; i++)
            if (regions[i].version)
                n++;
        return n;
    }

    function hostTotal() {
        var n = 0;
        for (var i = 0; i < regions.length; i++)
            n += regions[i].hosts;
        return n;
    }

    function visibleRegions() {
        var q = search.text.toLowerCase();
        var out = [];
        for (var i = 0; i < regions.length; i++) {
            var r = regions[i];
            if (filter === 1 && !r.version)
                continue;
            if (filter === 2 && r.version)
                continue;
            if (parentFilter.length > 0 && r.parent !== parentFilter)
                continue;
            if (q.length > 0 && r.region.indexOf(q) < 0 && r.name.toLowerCase().indexOf(q) < 0)
                continue;
            out.push(r);
        }
        return out;
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
        root.selectedCount = selectedIds().length;
    }

    function clearSelection() {
        root.selected = ({});
        root.selectedCount = 0;
    }

    function openDetails(region) {
        root.openRegion = region;
        root.openEntry = null;
        root.openResult = "";
        root.openLoading = true;
        run("resolve", region, function (a) {
            if (root.openRegion !== region)
                return;
            root.openLoading = false;
            if (a.ok)
                root.openEntry = a.data;
            else
                root.problem = errorText(a);
        });
    }

    // Registry query by parent (P-F7): how many entries and hosted regions sit under a country.
    function setParent(p) {
        root.parentFilter = p;
        root.parentInfo = "";
        if (p.length === 0)
            return;
        run("lookup", JSON.stringify({"parent": p}), function (a) {
            if (root.parentFilter !== p)
                return;
            if (!a.ok) {
                root.problem = errorText(a);
                return;
            }
            var hosted = {};
            for (var i = 0; i < a.data.entries.length; i++)
                hosted[a.data.entries[i].region] = true;
            root.parentInfo = a.data.entries.length + (a.data.entries.length === 1 ? " entry" : " entries")
                + " registered under " + p + " across " + Object.keys(hosted).length + " of its subregions";
        });
    }

    // Version info for every region (P-F1, P-F6): Geofabrik's current snapshot beside the
    // registered one, one HEAD request per region in the module.
    function checkVersions() {
        var ids = [];
        for (var i = 0; i < regions.length; i++)
            ids.push(regions[i].region);
        root.checkingVersions = true;
        run("check_updates", JSON.stringify(ids), function (a) {
            root.checkingVersions = false;
            if (!a.ok) {
                root.problem = errorText(a);
                return;
            }
            var m = {};
            for (var k = 0; k < a.data.length; k++)
                // The SDK calls a region with no entry "update available"; here an update means
                // a hosted region whose registered snapshot is older than Geofabrik's.
                m[a.data[k].region] = {"version": a.data[k].available,
                                       "update": a.data[k].registered !== null && a.data[k].registered !== undefined && a.data[k].update_available};
            root.available = m;
        });
    }

    function updatesCount() {
        var n = 0;
        for (var k in root.available)
            if (root.available[k].update)
                n++;
        return n;
    }

    function looksLikeCid(s) {
        return s.length >= 46 && (s.indexOf("zDv") === 0 || s.indexOf("Qm") === 0);
    }

    // Registry query by CID (P-F7): which region holds this file.
    function findCid(cid) {
        root.searchInfo = "";
        run("lookup", JSON.stringify({"cid": cid}), function (a) {
            if (!a.ok) {
                root.problem = errorText(a);
                return;
            }
            if (a.data.entries.length === 0) {
                root.searchInfo = "No registered entry has this CID.";
                return;
            }
            var e = a.data.entries[a.data.entries.length - 1];
            root.searchInfo = "Found in " + regionName(e.region) + " (" + e.region + "), snapshot " + formatSnapshot(e.version) + ".";
            search.text = "";
            root.openDetails(e.region);
        });
    }

    function regionName(id) {
        for (var i = 0; i < regions.length; i++)
            if (regions[i].region === id)
                return regions[i].name;
        return id;
    }

    // Bulk host: the module downloads and uploads the regions one by one, then registers each
    // group of up to 16 (the program's batch limit) in one transaction. One region uses `host`.
    readonly property int maxBatch: 16

    function hostSelected() {
        root.hostQueue = selectedIds();
        tabs.currentIndex = 3;
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

    readonly property var actionNames: ({
        "settings": "Read settings", "configure": "Save settings", "discover": "List regions",
        "hosted_status": "Read hosted status", "resolve": "Look up region", "fetch": "Download",
        "check_updates": "Check for updates", "host": "Host region", "host_batch": "Host regions",
        "import_file": "Import file", "register_entry": "Register", "unhost": "Stop hosting",
        "register_batch": "Register batch", "lookup": "Search registry", "status": "Test connection",
        "version": "Version"
    })

    function describe(method, a) {
        if (!a.ok)
            return errorText(a);
        var d = a.data;
        if (method === "resolve")
            return d === null ? "Not hosted yet" : "Snapshot " + formatSnapshot(d.version) + " · " + formatSize(d.size_bytes)
                  + " · " + d.hosts.length + (d.hosts.length === 1 ? " host" : " hosts");
        if (method === "fetch")
            return "Saved to " + d.path + "\nMD5 " + d.md5 + " matches the registry · from " + d.from
                   + (d.skipped.length > 0 ? "\nSkipped " + d.skipped.length + " entries that did not verify" : "");
        if (method === "check_updates") {
            var lines = [];
            for (var i = 0; i < d.length; i++)
                lines.push(d[i].region + ": registered " + (formatSnapshot(d[i].registered) || "never") + ", Geofabrik "
                           + formatSnapshot(d[i].available) + (d[i].update_available ? " — update available" : " — up to date"));
            return lines.join("\n");
        }
        if (method === "host_batch")
            return "Confirmed on-chain in one transaction: " + d.regions.join(", ");
        if (method === "host" || method === "import_file" || method === "register_entry" || method === "unhost")
            return "Confirmed on-chain" + (d.cid ? " · CID " + d.cid : "");
        if (method === "discover")
            return d.regions.length + " regions (" + d.source + ")";
        if (method === "hosted_status") {
            var hosted = 0;
            for (var k = 0; k < d.regions.length; k++)
                if (d.regions[k].version)
                    hosted++;
            return hosted + " of " + d.regions.length + " regions hosted";
        }
        if (method === "settings" || method === "configure")
            return "Done";
        return JSON.stringify(d);
    }

    readonly property var routine: ["settings", "discover", "hosted_status", "resolve", "version", "lookup", "status"]

    function jobList() {
        var ids = Object.keys(root.jobs);
        var out = [];
        for (var i = ids.length - 1; i >= 0; i--) {
            var j = root.jobs[ids[i]];
            if (routine.indexOf(j.method) >= 0 && j.state !== "failed")
                continue;
            out.push({"id": ids[i], "method": j.method, "arg": j.arg, "state": j.state, "answer": j.answer || ""});
        }
        return out;
    }

    // What a job acted on: the region(s) in its request, or its plain argument.
    function jobSubject(j) {
        var a = String(j.arg || "");
        if (a.charAt(0) !== "{" && a.charAt(0) !== "[")
            return a;
        try {
            var v = JSON.parse(a);
            if (Array.isArray(v))
                return v.join(", ");
            if (v.regions)
                return v.regions.join(", ");
            return v.region || "";
        } catch (e) {
            return "";
        }
    }

    function runningCount() {
        var n = 0;
        for (var k in root.jobs)
            if (root.jobs[k].state === "submitting")
                n++;
        return n;
    }

    // ---- reusable pieces --------------------------------------------------------------------

    // A label above a value, for the detail panel. `copy` adds a copy button for ids and hashes.
    component Field: ColumnLayout {
        id: field
        property string label: ""
        property string value: ""
        property bool copy: false
        property bool mono: false
        spacing: Theme.spacing.tiny
        Layout.fillWidth: true
        LogosText {
            visible: field.label.length > 0
            text: field.label
            color: Theme.palette.textSecondary
            font.pixelSize: Theme.typography.secondaryText
        }
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacing.small
            LogosText {
                Layout.fillWidth: true
                text: field.value
                wrapMode: Text.WrapAnywhere
                font.family: field.mono ? Theme.typography.mono : Theme.typography.publicSans
            }
            LogosCopyButton {
                visible: field.copy
                Layout.alignment: Qt.AlignTop
                value: field.value
            }
        }
    }

    // A form card, at most 720 px wide, as the design system's forms are.
    component Card: LogosFrame {
        id: card
        default property alias content: body.data
        property string title: ""
        property string subtitle: ""
        Layout.fillWidth: true
        Layout.maximumWidth: 720
        Layout.alignment: Qt.AlignHCenter | Qt.AlignTop
        padding: Theme.spacing.xlarge
        backgroundColor: Theme.palette.surfaceRaised
        borderColor: Theme.palette.borderHairline
        radius: Theme.spacing.radiusLarge
        ColumnLayout {
            id: body
            width: card.availableWidth
            spacing: Theme.spacing.medium
            LogosText {
                text: card.title
                font.pixelSize: Theme.typography.subtitleText
                font.weight: Theme.typography.weightMedium
            }
            LogosText {
                visible: text.length > 0
                Layout.fillWidth: true
                Layout.bottomMargin: Theme.spacing.small
                text: card.subtitle
                color: Theme.palette.textSecondary
                wrapMode: Text.WordWrap
            }
        }
    }

    // ---- view -------------------------------------------------------------------------------

    // What a host run will do, before it starts (E66): regions, transactions, fee reserve, disk.
    LogosDialog {
        id: confirmHost
        anchors.centerIn: parent
        preferredWidth: 480
        title: "Host " + root.selectedCount + (root.selectedCount === 1 ? " region?" : " regions?")
        message: "Each region is downloaded from Geofabrik, checked against its MD5 and uploaded to "
                 + (root.settings.storage || "your Storage node") + ". Then "
                 + Math.ceil(root.selectedCount / root.maxBatch) + (Math.ceil(root.selectedCount / root.maxBatch) === 1 ? " transaction" : " transactions")
                 + " signed by " + hostField.text + " register" + (Math.ceil(root.selectedCount / root.maxBatch) === 1 ? "s" : "")
                 + " them. Each transaction needs 38,400,000 lepta available for the fee reserve; the fee charged is far lower."
                 + " Country files can be several GB each; the cache keeps at most 20 GiB."
        leftActions: [ LogosButton { text: "Cancel"; onClicked: confirmHost.close() } ]
        rightActions: [ LogosButton {
            text: "Host"
            variant: LogosButton.Variant.Primary
            onClicked: { confirmHost.close(); root.hostSelected(); }
        } ]
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacing.xlarge
        spacing: Theme.spacing.large

        // Header: app identity, network, connection, refresh.
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacing.medium
            Image {
                source: "../src/icons/pramadani_osm_app.png"
                sourceSize: Qt.size(40, 40)
                Layout.preferredWidth: 40
                Layout.preferredHeight: 40
            }
            ColumnLayout {
                spacing: 2
                LogosText {
                    text: "OSM Maps"
                    font.pixelSize: Theme.typography.panelTitleText
                    font.weight: Theme.typography.weightBold
                }
                LogosText {
                    text: "OpenStreetMap snapshots registered on Logos, stored in Logos Storage"
                    color: Theme.palette.textSecondary
                    font.pixelSize: Theme.typography.secondaryText
                }
            }
            Item { Layout.fillWidth: true }
            LogosBadge {
                visible: root.services.rpc !== undefined
                text: root.services.rpc && root.services.rpc.ok ? root.hostOf(root.services.rpc.url) + " · block " + root.services.rpc.block
                      : "Sequencer unreachable"
                color: root.services.rpc && root.services.rpc.ok ? Theme.palette.success : Theme.palette.error
                ToolTip.visible: rpcHover.hovered
                ToolTip.text: root.services.rpc ? (root.services.rpc.ok ? root.services.rpc.url : root.services.rpc.error) : ""
                HoverHandler { id: rpcHover }
            }
            LogosBadge {
                visible: root.services.storage !== undefined
                text: root.services.storage && root.services.storage.ok
                      ? "Storage · " + root.formatSize(root.services.storage.quota_max_bytes - root.services.storage.quota_used_bytes) + " free"
                      : "Storage unreachable"
                color: root.services.storage && root.services.storage.ok ? Theme.palette.success : Theme.palette.warning
                ToolTip.visible: storageHover.hovered
                ToolTip.text: root.services.storage ? (root.services.storage.ok ? root.services.storage.url : root.services.storage.error + " — reading still works; hosting needs your node") : ""
                HoverHandler { id: storageHover }
            }
            LogosBadge {
                text: !root.ready ? "Connecting" : root.runningCount() > 0 ? root.runningCount() + " running" : "Ready"
                color: !root.ready ? Theme.palette.warning : root.runningCount() > 0 ? Theme.palette.accentOrange : Theme.palette.success
            }
            LogosIconButton {
                iconSource: LogosIcons.refresh
                enabled: root.ready
                onClicked: root.refresh()
                ToolTip.visible: hovered
                ToolTip.text: "Refresh"
            }
        }

        // At a glance.
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacing.medium
            LogosStatCard {
                Layout.fillWidth: true
                label: "Regions"
                value: root.regions.length > 0 ? String(root.regions.length) : "—"
                caption: "48 countries · 24 subregions"
            }
            LogosStatCard {
                Layout.fillWidth: true
                label: "Hosted"
                value: root.loadingRegions ? "…" : String(root.hostedCount())
                caption: "regions on Logos Storage"
                severity: root.hostedCount() > 0 ? LogosStatCard.Success : LogosStatCard.None
            }
            LogosStatCard {
                Layout.fillWidth: true
                label: "Hosts"
                value: root.loadingRegions ? "…" : String(root.hostTotal())
                caption: "copies registered"
            }
            LogosStatCard {
                Layout.fillWidth: true
                label: "Selected"
                value: String(root.selectedCount)
                caption: root.selectedCount > 0 ? "ready to host" : "tick regions to host"
            }
        }

        LogosNotice {
            Layout.fillWidth: true
            visible: root.problem.length > 0
            severity: LogosNotice.Error
            title: "Something went wrong"
            message: root.problem
            closable: true
            onDismissed: root.problem = ""
        }

        LogosTabBar {
            id: tabs
            objectName: "tabs"
            Layout.fillWidth: true
            LogosTabButton { text: "Regions" }
            LogosTabButton { text: root.selectedCount > 0 ? "Host (" + root.selectedCount + ")" : "Host" }
            LogosTabButton { text: "Import" }
            LogosTabButton { text: root.runningCount() > 0 ? "Activity (" + root.runningCount() + ")" : "Activity" }
            LogosTabButton { text: "Settings" }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: tabs.currentIndex

            // ── Regions ──────────────────────────────────────────────────────────────────
            RowLayout {
                spacing: Theme.spacing.large

                ColumnLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: Theme.spacing.medium

                    LogosSearchBar {
                        id: search
                        Layout.fillWidth: true
                        placeholderText: "Search regions, or paste a CID and press Enter"
                        onSubmitted: function (text) {
                            if (root.looksLikeCid(text.trim()))
                                root.findCid(text.trim());
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.spacing.small
                        LogosComboBox {
                            Layout.preferredWidth: 150
                            model: ["All countries", "us", "india", "china", "russia"]
                            onActivated: function (i) { root.setParent(i === 0 ? "" : model[i]); }
                        }
                        Repeater {
                            model: ["All", "Hosted", "Not hosted"]
                            LogosButton {
                                required property int index
                                required property string modelData
                                text: modelData
                                compact: true
                                variant: root.filter === index ? LogosButton.Variant.Primary : LogosButton.Variant.Secondary
                                onClicked: root.filter = index
                            }
                        }
                        Item { Layout.fillWidth: true }
                        LogosButton {
                            text: root.checkingVersions ? "Checking…" : "Geofabrik versions"
                            compact: true
                            enabled: root.ready && !root.checkingVersions && root.regions.length > 0
                            leadingIcon.source: LogosIcons.refresh
                            onClicked: root.checkVersions()
                        }
                    }

                    LogosNotice {
                        Layout.fillWidth: true
                        visible: Object.keys(root.available).length > 0 && root.updatesCount() > 0
                        severity: LogosNotice.Warning
                        message: root.updatesCount() + (root.updatesCount() === 1 ? " hosted region has" : " hosted regions have")
                                 + " a newer snapshot on Geofabrik. Host it again to register the new version."
                    }

                    LogosNotice {
                        Layout.fillWidth: true
                        visible: root.parentInfo.length > 0 || root.searchInfo.length > 0
                        severity: LogosNotice.Info
                        message: root.searchInfo.length > 0 ? root.searchInfo : root.parentInfo
                        closable: true
                        onDismissed: { root.searchInfo = ""; root.parentInfo = ""; }
                    }

                    LogosFrame {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        backgroundColor: Theme.palette.surfaceRaised
                        borderColor: Theme.palette.borderHairline
                        radius: Theme.spacing.radiusLarge
                        clip: true

                        LogosStateView {
                            anchors.fill: parent
                            visible: root.loadingRegions && root.regions.length === 0
                            kind: LogosStateView.Loading
                            title: "Loading regions"
                            message: "Reading the predefined set and the registry."
                        }
                        LogosStateView {
                            anchors.fill: parent
                            visible: !root.loadingRegions && regionList.count === 0
                            kind: LogosStateView.Empty
                            title: root.regions.length === 0 ? "No regions yet" : "No region matches"
                            message: root.regions.length === 0 ? "Refresh to read the registry." : "Try another search or filter."
                        }

                        LogosListView {
                            id: regionList
                            anchors.fill: parent
                            anchors.margins: Theme.spacing.tiny
                            clip: true
                            model: root.visibleRegions()
                            delegate: Rectangle {
                                id: row
                                required property var modelData
                                width: ListView.view.width
                                height: 60
                                radius: Theme.spacing.radiusSmall
                                color: root.openRegion === modelData.region ? Theme.palette.primarySoft
                                       : hover.hovered ? Theme.palette.surfaceInteractiveHover : "transparent"

                                HoverHandler { id: hover }
                                TapHandler { onTapped: root.openDetails(row.modelData.region) }

                                RowLayout {
                                    anchors.fill: parent
                                    anchors.leftMargin: Theme.spacing.small
                                    anchors.rightMargin: Theme.spacing.medium
                                    spacing: Theme.spacing.medium
                                    LogosCheckbox {
                                        checked: root.selected[row.modelData.region] === true
                                        onToggled: root.toggle(row.modelData.region, checked)
                                    }
                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        LogosText {
                                            Layout.fillWidth: true
                                            text: row.modelData.name
                                            font.weight: Theme.typography.weightMedium
                                            elide: Text.ElideRight
                                        }
                                        LogosText {
                                            Layout.fillWidth: true
                                            text: row.modelData.region + (row.modelData.level === "subregion" ? " · subregion" : " · country")
                                            color: Theme.palette.textSecondary
                                            font.pixelSize: Theme.typography.secondaryText
                                            elide: Text.ElideRight
                                        }
                                    }
                                    LogosText {
                                        visible: root.available[row.modelData.region] !== undefined
                                        text: root.available[row.modelData.region] ? "Geofabrik " + root.formatSnapshot(root.available[row.modelData.region].version) : ""
                                        ToolTip.visible: versionHover.hovered
                                        ToolTip.text: "Geofabrik's current first-of-month snapshot"
                                        HoverHandler { id: versionHover }
                                        color: Theme.palette.textSecondary
                                        font.pixelSize: Theme.typography.secondaryText
                                    }
                                    LogosBadge {
                                        visible: root.available[row.modelData.region] !== undefined && root.available[row.modelData.region].update === true
                                        text: "Update"
                                        color: Theme.palette.warning
                                    }
                                    LogosText {
                                        visible: row.modelData.hosts > 0
                                        text: row.modelData.hosts + (row.modelData.hosts === 1 ? " host" : " hosts")
                                        color: Theme.palette.textSecondary
                                        font.pixelSize: Theme.typography.secondaryText
                                    }
                                    LogosBadge {
                                        text: row.modelData.version ? root.formatSnapshot(row.modelData.version) : "Not hosted"
                                        color: row.modelData.version ? Theme.palette.success : Theme.palette.textTertiary
                                    }
                                }
                            }
                        }
                    }
                }

                // Detail of the open region.
                LogosFrame {
                    visible: root.openRegion.length > 0
                    Layout.preferredWidth: 372
                    Layout.fillHeight: true
                    backgroundColor: Theme.palette.surfaceRaised
                    borderColor: Theme.palette.borderHairline
                    radius: Theme.spacing.radiusLarge

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Theme.spacing.large
                        spacing: Theme.spacing.medium

                        RowLayout {
                            Layout.fillWidth: true
                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2
                                LogosText {
                                    Layout.fillWidth: true
                                    text: root.regionName(root.openRegion)
                                    font.pixelSize: Theme.typography.subtitleText
                                    font.weight: Theme.typography.weightMedium
                                    elide: Text.ElideRight
                                }
                                LogosText {
                                    text: root.openRegion
                                    color: Theme.palette.textSecondary
                                    font.pixelSize: Theme.typography.secondaryText
                                }
                            }
                            LogosIconButton {
                                iconSource: LogosIcons.close
                                onClicked: root.openRegion = ""
                            }
                        }

                        LogosSpinner {
                            visible: root.openLoading
                            Layout.alignment: Qt.AlignHCenter
                        }

                        LogosNotice {
                            Layout.fillWidth: true
                            visible: !root.openLoading && root.openEntry === null
                            severity: LogosNotice.Info
                            title: "Not hosted yet"
                            message: "Nobody has registered this region. Tick it and host it from Geofabrik's dated snapshot."
                        }

                        ColumnLayout {
                            visible: !root.openLoading && root.openEntry !== null
                            Layout.fillWidth: true
                            spacing: Theme.spacing.medium
                            RowLayout {
                                spacing: Theme.spacing.small
                                LogosBadge {
                                    text: root.openEntry ? "Snapshot " + root.formatSnapshot(root.openEntry.version) : ""
                                    color: Theme.palette.success
                                }
                                LogosBadge {
                                    text: root.openEntry ? root.formatSize(root.openEntry.size_bytes) : ""
                                    color: Theme.palette.textSecondary
                                }
                            }
                            LogosText {
                                Layout.fillWidth: true
                                text: "As registered on-chain. Download checks the file against this MD5 before keeping it."
                                color: Theme.palette.textSecondary
                                font.pixelSize: Theme.typography.secondaryText
                                wrapMode: Text.WordWrap
                            }
                            Field { label: "CID (Logos Storage)"; value: root.openEntry ? root.openEntry.cid : ""; copy: true }
                            Field { label: "MD5 (from Geofabrik)"; value: root.openEntry ? root.openEntry.checksum : ""; copy: true }
                            Field { label: "Registered"; value: root.openEntry ? root.formatTime(root.openEntry.timestamp_ms) : "" }
                            Repeater {
                                model: root.openEntry ? root.openEntry.hosts : []
                                Field {
                                    required property string modelData
                                    required property int index
                                    label: index === 0 ? (root.openEntry.hosts.length === 1 ? "Host" : "Hosts (" + root.openEntry.hosts.length + ")") : ""
                                    value: modelData
                                    copy: true
                                }
                            }
                            Field { label: "Source"; value: root.openEntry ? root.openEntry.source_url : "" }
                        }

                        LogosNotice {
                            Layout.fillWidth: true
                            visible: root.openResult.length > 0
                            severity: root.openResultOk ? LogosNotice.Success : LogosNotice.Error
                            message: root.openResult
                            closable: true
                            onDismissed: root.openResult = ""
                        }

                        Item { Layout.fillHeight: true }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacing.small
                            LogosButton {
                                Layout.fillWidth: true
                                text: "Download"
                                variant: LogosButton.Variant.Primary
                                enabled: root.ready && root.openEntry !== null
                                leadingIcon.source: LogosIcons.install
                                onClicked: {
                                    var region = root.openRegion;
                                    root.openResult = "";
                                    root.run("fetch", region, function (a) {
                                        root.openResultOk = a.ok;
                                        root.openResult = root.describe("fetch", a);
                                    });
                                }
                            }
                            LogosButton {
                                Layout.fillWidth: true
                                text: "Check update"
                                enabled: root.ready
                                onClicked: root.run("check_updates", JSON.stringify([root.openRegion]), function (a) {
                                    root.openResultOk = a.ok;
                                    root.openResult = root.describe("check_updates", a);
                                })
                            }
                        }
                    }
                }
            }

            // ── Host ─────────────────────────────────────────────────────────────────────
            Flickable {
                contentHeight: hostCol.implicitHeight
                clip: true
                ColumnLayout {
                    id: hostCol
                    width: parent.width
                    spacing: Theme.spacing.large

                    Card {
                        title: "Host regions"
                        subtitle: "Downloads Geofabrik's dated snapshot of each ticked region, checks its MD5, uploads it to your Storage node and registers it signed by your wallet. Up to 16 regions share one transaction; each is confirmed by reading the registry back."

                        LogosText { text: "Host account"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                        LogosTextField {
                            id: hostField
                            Layout.fillWidth: true
                            text: root.settings.host || ""
                            placeholderText: "Public/… your LEZ account"
                        }
                        LogosText { text: "Snapshot (optional)"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                        LogosTextField {
                            id: snapshotField
                            Layout.preferredWidth: 200
                            placeholderText: "YYMMDD · first of this month"
                            validator: IntValidator { bottom: 0; top: 999999 }
                        }

                        LogosText {
                            text: root.selectedCount === 0 ? "No region ticked yet — tick them on the Regions tab."
                                  : root.selectedCount + (root.selectedCount === 1 ? " region ticked" : " regions ticked")
                            color: root.selectedCount === 0 ? Theme.palette.textSecondary : Theme.palette.text
                        }
                        Flow {
                            Layout.fillWidth: true
                            spacing: Theme.spacing.small
                            Repeater {
                                model: root.selectedCount > 0 ? root.selectedIds() : []
                                LogosBadge {
                                    required property string modelData
                                    text: root.regionName(modelData)
                                    color: Theme.palette.accentOrange
                                }
                            }
                        }

                        RowLayout {
                            spacing: Theme.spacing.small
                            LogosButton {
                                text: root.hostQueue.length > 0 ? "Hosting… " + root.hostQueue.length + " left"
                                      : root.selectedCount > 1 ? "Host " + root.selectedCount + " regions" : "Host region"
                                variant: LogosButton.Variant.Primary
                                enabled: root.ready && hostField.text.length > 0 && root.selectedCount > 0 && root.hostQueue.length === 0
                                onClicked: confirmHost.open()
                            }
                            LogosButton {
                                text: "Stop after current"
                                visible: root.hostQueue.length > 0
                                onClicked: root.hostQueue = []
                            }
                            LogosButton {
                                text: "Clear selection"
                                visible: root.selectedCount > 0 && root.hostQueue.length === 0
                                onClicked: root.clearSelection()
                            }
                        }
                    }
                }
            }

            // ── Import ───────────────────────────────────────────────────────────────────
            Flickable {
                contentHeight: importCol.implicitHeight
                clip: true
                ColumnLayout {
                    id: importCol
                    width: parent.width
                    spacing: Theme.spacing.large

                    Card {
                        title: "Import a local file"
                        subtitle: "Checks a .osm.pbf you already have against Geofabrik's published MD5 for its snapshot, uploads it to your Storage node — the same file gives the same CID — and registers you as a host."

                        LogosText { text: "File"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                        LogosTextField { id: importPath; Layout.fillWidth: true; placeholderText: "/path/to/region-261101.osm.pbf" }
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Theme.spacing.medium
                            ColumnLayout {
                                Layout.fillWidth: true
                                LogosText { text: "Region"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                                LogosTextField { id: importRegion; Layout.fillWidth: true; placeholderText: "e.g. kenya" }
                            }
                            ColumnLayout {
                                LogosText { text: "Snapshot"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                                LogosTextField {
                                    id: importSnapshot
                                    Layout.preferredWidth: 160
                                    placeholderText: "YYMMDD"
                                    validator: IntValidator { bottom: 0; top: 999999 }
                                }
                            }
                        }
                        LogosNotice {
                            Layout.fillWidth: true
                            visible: hostField.text.length === 0
                            severity: LogosNotice.Warning
                            message: "Set your host account on the Host tab first."
                        }
                        LogosButton {
                            text: "Import and register"
                            variant: LogosButton.Variant.Primary
                            enabled: root.ready && hostField.text.length > 0 && importPath.text.length > 0
                                     && importRegion.text.length > 0 && importSnapshot.text.length === 6
                            onClicked: {
                                tabs.currentIndex = 3;
                                root.run("import_file", JSON.stringify({
                                    "host": hostField.text, "path": importPath.text, "region": importRegion.text,
                                    "snapshot": Number(importSnapshot.text)}), function (a) { root.loadHosted(); });
                            }
                        }
                    }
                }
            }

            // ── Activity ─────────────────────────────────────────────────────────────────
            ColumnLayout {
                spacing: Theme.spacing.medium

                RowLayout {
                    Layout.fillWidth: true
                    LogosText {
                        Layout.fillWidth: true
                        text: "Every action and its result. A write shows Confirmed only after the registry was read back."
                        color: Theme.palette.textSecondary
                        wrapMode: Text.WordWrap
                    }
                    LogosButton {
                        text: "Clear finished"
                        compact: true
                        enabled: root.ready && Object.keys(root.jobs).length > root.runningCount()
                        onClicked: backend.clearFinished()
                    }
                }

                LogosStateView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: activityList.count === 0
                    kind: LogosStateView.Empty
                    title: "No activity yet"
                    message: "Downloads, hosting and imports appear here with their result."
                }

                LogosListView {
                    id: activityList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    visible: count > 0
                    clip: true
                    spacing: Theme.spacing.small
                    model: root.jobList()
                    delegate: LogosFrame {
                        id: jobRow
                        required property var modelData
                        width: ListView.view.width
                        backgroundColor: Theme.palette.surfaceRaised
                        borderColor: Theme.palette.borderHairline
                        radius: Theme.spacing.radiusMedium
                        RowLayout {
                            width: jobRow.availableWidth
                            spacing: Theme.spacing.medium
                            ColumnLayout {
                                id: jobCol
                                Layout.fillWidth: true
                                spacing: Theme.spacing.tiny
                                LogosText {
                                    Layout.fillWidth: true
                                    text: (root.actionNames[jobRow.modelData.method] || jobRow.modelData.method)
                                          + (root.jobSubject(jobRow.modelData) ? " · " + root.jobSubject(jobRow.modelData) : "")
                                    font.weight: Theme.typography.weightMedium
                                    elide: Text.ElideRight
                                }
                                LogosText {
                                    Layout.fillWidth: true
                                    visible: jobRow.modelData.state !== "submitting"
                                    text: root.describe(jobRow.modelData.method, root.parse(jobRow.modelData.answer || "{}"))
                                    color: jobRow.modelData.state === "failed" ? Theme.palette.error : Theme.palette.textSecondary
                                    font.pixelSize: Theme.typography.secondaryText
                                    wrapMode: Text.WrapAnywhere
                                }
                            }
                            LogosSpinner {
                                visible: jobRow.modelData.state === "submitting"
                                Layout.preferredWidth: 16
                                Layout.preferredHeight: 16
                            }
                            LogosBadge {
                                text: jobRow.modelData.state === "submitting" ? "Submitting"
                                      : jobRow.modelData.state === "confirmed" ? "Confirmed" : "Failed"
                                color: jobRow.modelData.state === "submitting" ? Theme.palette.warning
                                       : jobRow.modelData.state === "confirmed" ? Theme.palette.success : Theme.palette.error
                            }
                        }
                    }
                }
            }

            // ── Settings ─────────────────────────────────────────────────────────────────
            Flickable {
                contentHeight: settingsCol.implicitHeight
                clip: true
                ColumnLayout {
                    id: settingsCol
                    width: parent.width
                    spacing: Theme.spacing.large

                    Card {
                        title: "Network and storage"
                        subtitle: "Where the registry lives and which Logos Storage node your files go to. Saved per Basecamp profile."

                        LogosText { text: "Sequencer RPC"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                        LogosTextField { id: setRpc; Layout.fillWidth: true; text: root.settings.rpc || "" }
                        LogosText { text: "Registry program"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                        LogosTextField { id: setProgram; Layout.fillWidth: true; text: root.settings.program || "" }
                        LogosText { text: "Storage API"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                        LogosTextField { id: setStorage; Layout.fillWidth: true; text: root.settings.storage || "" }
                        LogosText { text: "Host account"; color: Theme.palette.textSecondary; font.pixelSize: Theme.typography.secondaryText }
                        LogosTextField { id: setHost; Layout.fillWidth: true; text: root.settings.host || ""; placeholderText: "Public/…" }
                        LogosSwitch {
                            id: setGeofabrik
                            text: "Use Geofabrik"
                            checked: root.settings.geofabrik !== false
                        }
                        LogosText {
                            Layout.fillWidth: true
                            text: "Without Geofabrik the app still lists, downloads and verifies hosted regions; hosting a new snapshot needs it."
                            color: Theme.palette.textSecondary
                            font.pixelSize: Theme.typography.secondaryText
                            wrapMode: Text.WordWrap
                        }
                        LogosNotice {
                            Layout.fillWidth: true
                            visible: root.services.rpc !== undefined
                            severity: root.services.rpc && root.services.rpc.ok && root.services.storage && root.services.storage.ok
                                      ? LogosNotice.Success : LogosNotice.Warning
                            title: "Connection"
                            message: !root.services.rpc ? "" :
                                     (root.services.rpc.ok ? "Sequencer answered, block " + root.services.rpc.block + "." : root.services.rpc.error)
                                     + "\n" + (root.services.storage.ok
                                        ? "Storage node answered: " + root.formatSize(root.services.storage.quota_used_bytes) + " used of "
                                          + root.formatSize(root.services.storage.quota_max_bytes) + "."
                                        : root.services.storage.error + " Reading hosted regions still works; hosting needs your node.")
                        }
                        LogosButton {
                            text: root.probing ? "Testing…" : "Test connection"
                            enabled: root.ready && !root.probing
                            onClicked: root.probe()
                        }
                        LogosButton {
                            text: "Save"
                            variant: LogosButton.Variant.Primary
                            enabled: root.ready
                            onClicked: root.run("configure", JSON.stringify({
                                "rpc": setRpc.text, "program": setProgram.text, "storage": setStorage.text,
                                "host": setHost.text, "geofabrik": setGeofabrik.checked}), function (a) {
                                if (a.ok) {
                                    root.settings = a.data;
                                    root.refresh();
                                } else {
                                    root.problem = root.errorText(a);
                                }
                            })
                        }
                    }
                }
            }
        }
    }
}
