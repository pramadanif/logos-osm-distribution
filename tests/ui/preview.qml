// Renders the OSM Maps view offscreen with a fake `logos` host and canned module answers, and
// saves one tab as a PNG. tests/ui/preview.sh drives it; nothing here talks to a network.
import QtQuick
import QtQuick.Window

Window {
    id: win
    width: 1280
    height: 860
    visible: true
    color: "#171717"

    readonly property var args: Qt.application.arguments
    readonly property int tab: Number(args[args.length - 3])
    readonly property string viewFile: args[args.length - 2]
    readonly property string out: args[args.length - 1]

    function readJson(path) {
        var x = new XMLHttpRequest();
        x.open("GET", Qt.resolvedUrl(path), false);
        x.send();
        return JSON.parse(x.responseText);
    }
    readonly property var sample: readJson("sample.json")

    // What the Basecamp host gives a view: logos.module(), readiness, and watch() on replies.
    QtObject {
        id: logos
        signal viewModuleReadyChanged(string moduleName, bool isReady)
        function module(name) { return backend; }
        function isViewModuleReady(name) { return true; }
        function watch(reply, ok, err) { ok(reply); }
    }

    // The app backend: every call finishes at once with the sample answer.
    QtObject {
        id: backend
        property string jobs: JSON.stringify(win.sample.jobs)
        property string progress: JSON.stringify({"region": "germany", "stage": "download", "done": 2254857830, "total": 4402341478})
        property int n: 0
        function call(method, arg) {
            var id = method + "-" + (++n);
            var j = JSON.parse(jobs);
            var answer = win.sample.answers[method + ":" + arg] || win.sample.answers[method]
                         || {"ok": false, "code": "S160", "error": "no sample answer"};
            j[id] = {"method": method, "arg": arg, "state": "confirmed", "answer": JSON.stringify(answer)};
            jobs = JSON.stringify(j);
            return id;
        }
        function clearFinished() {}
    }

    Loader {
        id: loader
        anchors.fill: parent
        source: win.viewFile
        // A view that does not compile ends the preview with an error instead of a hang.
        onStatusChanged: if (status === Loader.Error) Qt.exit(1)
    }

    function find(item, name) {
        if (!item)
            return null;
        if (item.objectName === name)
            return item;
        var kids = item.children || [];
        for (var i = 0; i < kids.length; i++) {
            var f = find(kids[i], name);
            if (f)
                return f;
        }
        return null;
    }

    Timer {
        interval: 800
        running: loader.status === Loader.Ready
        onTriggered: {
            var v = loader.item;
            ["china/henan", "kenya", "egypt"].forEach(function (r) { v.toggle(r, true); });
            if (win.tab === 0) {
                v.openDetails("china/henan");
                v.checkVersions();
            }
            find(v, "tabs").currentIndex = win.tab;
            shot.start();
        }
    }
    Timer {
        id: shot
        interval: 700
        onTriggered: win.contentItem.grabToImage(function (r) {
            r.saveToFile(win.out);
            Qt.quit();
        })
    }
}
