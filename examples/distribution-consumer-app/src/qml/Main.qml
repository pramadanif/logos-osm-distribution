import QtQuick
import QtQuick.Layouts
import Logos.Theme
import Logos.Controls

// Template view: a region id in, its registered snapshot and a verified download out.
// Built on the Logos Design System (Logos.Theme / Logos.Controls), as Basecamp apps are; copy it
// and replace the body with your own idea. Text from the module is shown as plain text.
Item {
    id: root

    readonly property var backend: logos.module("pramadani_osm_consumer_app")
    property bool ready: false
    readonly property bool busy: ready && backend.busy
    readonly property var answer: {
        if (!ready || !backend.answer)
            return null;
        try {
            return JSON.parse(backend.answer);
        } catch (e) {
            return {"ok": false, "error": "unreadable answer"};
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.palette.background
    }

    Connections {
        target: logos
        function onViewModuleReadyChanged(moduleName, isReady) {
            if (moduleName === "pramadani_osm_consumer_app")
                root.ready = isReady && root.backend !== null;
        }
    }
    Component.onCompleted: root.ready = root.backend !== null && logos.isViewModuleReady("pramadani_osm_consumer_app")

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacing.xlarge
        spacing: Theme.spacing.large

        RowLayout {
            spacing: Theme.spacing.medium
            Image {
                source: "../src/icons/pramadani_osm_consumer_app.png"
                sourceSize: Qt.size(40, 40)
                Layout.preferredWidth: 40
                Layout.preferredHeight: 40
            }
            ColumnLayout {
                spacing: 2
                LogosText {
                    text: "OSM consumer (template)"
                    font.pixelSize: Theme.typography.panelTitleText
                    font.weight: Theme.typography.weightBold
                }
                LogosText {
                    text: "Look up a region in the OSM registry and download its verified file"
                    color: Theme.palette.textSecondary
                    font.pixelSize: Theme.typography.secondaryText
                }
            }
            Item { Layout.fillWidth: true }
            LogosBadge {
                text: !root.ready ? "Connecting" : root.busy ? "Working" : "Ready"
                color: !root.ready ? Theme.palette.warning : root.busy ? Theme.palette.accentOrange : Theme.palette.success
            }
        }

        LogosFrame {
            Layout.fillWidth: true
            padding: Theme.spacing.xlarge
            backgroundColor: Theme.palette.surfaceRaised
            borderColor: Theme.palette.borderHairline
            radius: Theme.spacing.radiusLarge
            RowLayout {
                width: parent.width
                spacing: Theme.spacing.small
                LogosTextField {
                    id: region
                    Layout.fillWidth: true
                    placeholderText: "Region id, e.g. china/henan or kenya"
                }
                LogosButton {
                    text: "Resolve"
                    variant: LogosButton.Variant.Primary
                    enabled: root.ready && !root.busy && region.text.length > 0
                    onClicked: root.backend.resolve(region.text)
                }
                LogosButton {
                    text: "Download"
                    enabled: root.ready && !root.busy && region.text.length > 0
                    onClicked: root.backend.download(region.text)
                }
                LogosButton {
                    text: "All regions"
                    enabled: root.ready && !root.busy
                    onClicked: root.backend.regions()
                }
            }
        }

        LogosStateView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.answer === null || root.busy
            kind: root.busy ? LogosStateView.Loading : LogosStateView.Empty
            title: root.busy ? "Asking the registry" : "Nothing asked yet"
            message: root.busy ? "Downloads of large regions can take a while." : "Type a region id and press Resolve."
        }

        LogosNotice {
            Layout.fillWidth: true
            visible: !root.busy && root.answer !== null && !root.answer.ok
            severity: LogosNotice.Error
            title: "The module answered with an error"
            message: root.answer && !root.answer.ok ? (root.answer.code ? root.answer.code + " " : "") + root.answer.error : ""
        }

        LogosFrame {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !root.busy && root.answer !== null && root.answer.ok
            padding: Theme.spacing.large
            backgroundColor: Theme.palette.surfaceRaised
            borderColor: Theme.palette.borderHairline
            radius: Theme.spacing.radiusLarge
            LogosScrollView {
                anchors.fill: parent
                LogosText {
                    width: parent.width
                    text: root.answer && root.answer.ok ? JSON.stringify(root.answer.data, null, 2) : ""
                    font.family: Theme.typography.mono
                    font.pixelSize: Theme.typography.secondaryText
                    wrapMode: Text.WrapAnywhere
                }
            }
        }
    }
}
