import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Template view: region id in, registered snapshot and verified download out.
Item {
    id: root

    readonly property var backend: logos.module("pramadani_osm_consumer_app")
    property bool ready: false

    Connections {
        target: logos
        function onViewModuleReadyChanged(moduleName, isReady) {
            if (moduleName === "pramadani_osm_consumer_app")
                root.ready = isReady && root.backend !== null;
        }
    }
    Component.onCompleted: root.ready = root.backend !== null && logos.isViewModuleReady("pramadani_osm_consumer_app")

    Rectangle {
        anchors.fill: parent
        color: "#0d1117"
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 10

        Text {
            text: "OSM consumer (template)"
            color: "#e6edf3"
            font.pixelSize: 18
        }
        RowLayout {
            TextField {
                id: region
                placeholderText: "region id, e.g. kenya"
                Layout.preferredWidth: 240
            }
            Button {
                text: "Regions"
                enabled: root.ready && !root.backend.busy
                onClicked: root.backend.regions()
            }
            Button {
                text: "Resolve"
                enabled: root.ready && !root.backend.busy && region.text.length > 0
                onClicked: root.backend.resolve(region.text)
            }
            Button {
                text: "Download"
                enabled: root.ready && !root.backend.busy && region.text.length > 0
                onClicked: root.backend.download(region.text)
            }
            Text {
                text: !root.ready ? "connecting..." : root.backend.busy ? "working..." : ""
                color: "#f0883e"
            }
        }
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            TextArea {
                text: root.ready ? root.backend.answer : ""
                textFormat: TextEdit.PlainText
                readOnly: true
                wrapMode: TextEdit.WrapAnywhere
                color: "#e6edf3"
            }
        }
    }
}
