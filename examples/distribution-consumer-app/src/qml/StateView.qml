import QtQuick
import Logos.Theme
import Logos.Controls

// Loading and empty states for the region and activity lists. Built from controls that the
// design system in the released Basecamp (logos-design-system 493a243) already has.
Item {
    id: root

    enum Kind { Loading, Empty }

    property int kind: StateView.Empty
    property string title
    property string message

    Column {
        anchors.centerIn: parent
        width: Math.min(parent.width - 2 * Theme.spacing.large, 360)
        spacing: Theme.spacing.small

        LogosSpinner {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.kind === StateView.Loading
            running: visible
        }
        LogosText {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            textFormat: Text.PlainText
            font.pixelSize: Theme.typography.subtitleText
            font.weight: Theme.typography.weightMedium
            text: root.title
        }
        LogosText {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            textFormat: Text.PlainText
            font.pixelSize: Theme.typography.secondaryText
            color: Theme.palette.textSecondary
            text: root.message
        }
    }
}
