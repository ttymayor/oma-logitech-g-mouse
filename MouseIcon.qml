import QtQuick
import qs.Commons

Item {
    id: root

    property real iconSize: 14
    property color color: Color.foreground
    property bool connected: true

    implicitWidth: iconSize
    implicitHeight: iconSize

    // Mouse body outline
    Rectangle {
        anchors.centerIn: parent
        width: root.iconSize * 0.72
        height: root.iconSize * 0.95
        radius: width / 2
        color: "transparent"
        border.width: Style.space(1)
        border.color: root.color
        opacity: root.connected ? 1.0 : 0.4

        // Scroll wheel / sensor indicator
        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            y: parent.height * 0.2
            width: Math.max(2, parent.width * 0.22)
            height: parent.height * 0.28
            radius: width / 2
            color: root.color
            opacity: root.connected ? 1.0 : 0.4
        }
    }
}

