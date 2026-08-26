import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Ui
import "Model.js" as Model

Panel {
    id: root
    moduleName: "tantuyu.g-pro-x2-superstrike"
    manageIpc: false

    property var anchorItem: null
    property var hostWidget: null
    property var mouse: null

    // Active tab: "sensor" or "buttons"
    property string currentTab: "sensor"

    readonly property color foreground: bar ? bar.foreground : Color.foreground
    readonly property color urgent: bar ? bar.urgent : Color.urgent
    readonly property string fontFamily: bar ? bar.fontFamily : Style.font.family

    readonly property bool mouseConnected: mouse ? mouse.connected : false
    readonly property string deviceName: mouse ? mouse.deviceName : "Logitech G Mouse"
    readonly property int batteryPercentage: mouse ? mouse.batteryPercentage : Model.LEVEL_UNKNOWN
    readonly property string batteryStatus: mouse ? mouse.batteryStatus : "unknown"
    readonly property int dpiX: mouse ? mouse.dpiX : 0
    readonly property int dpiY: mouse ? mouse.dpiY : 0
    readonly property int dpiMin: mouse ? mouse.dpiMin : 100
    readonly property int dpiMax: mouse ? mouse.dpiMax : 32000
    readonly property var dpiPresets: mouse ? mouse.dpiPresets : [800, 1200, 1600, 2400, 3200]
    readonly property int reportRate: mouse ? mouse.reportRate : 1000
    readonly property string lod: mouse ? mouse.lod : "unknown"
    readonly property bool hasHits: mouse ? mouse.hasHits : false
    readonly property var hitsLeft: mouse ? mouse.hitsLeft : Model.defaultButton()
    readonly property var hitsRight: mouse ? mouse.hitsRight : Model.defaultButton()

    function open() {
        root.controller.show()
        if (mouse) mouse.refresh()
    }

    function close() {
        root.controller.hide()
    }

    function toggle() {
        if (root.opened) root.close()
        else root.open()
    }

    function switchPanel(direction) {
        if (root.bar && typeof root.bar.switchPanelFrom === "function") {
            return root.bar.switchPanelFrom(root.hostWidget || root, direction)
        }
        return false
    }

    onOpenedChanged: if (opened) {
        if (panelFlick) panelFlick.contentY = 0
        if (mouse) mouse.refresh()
        Qt.callLater(function () { keyCatcher.forceActiveFocus() })
    }

    // Popup keyboard panel
    KeyboardPanel {
        id: panel
        anchorItem: root.anchorItem
        owner: root.hostWidget || root
        bar: root.bar
        open: root.opened
        focusTarget: keyCatcher
        contentWidth: panel.fittedContentWidth(Style.space(380))
        contentHeight: panel.fittedContentHeight(content.implicitHeight, Style.space(720))

        PanelKeyCatcher {
            id: keyCatcher
            anchors.fill: parent
            onCloseRequested: root.close()
            onTabRequested: function (direction) {
                root.switchPanel(direction);
            }

            Flickable {
                id: panelFlick
                anchors.fill: parent
                contentWidth: width
                contentHeight: content.implicitHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                flickableDirection: Flickable.VerticalFlick
                interactive: contentHeight > height
                ScrollBar.vertical: ScrollBar {
                    policy: ScrollBar.AsNeeded
                }

                Column {
                    id: content
                    width: panelFlick.width - (panelFlick.interactive ? Style.space(8) : 0)
                    spacing: Style.space(12)

                    // ── Dynamic Device Header ──
                    Row {
                        width: parent.width
                        spacing: Style.space(8)

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: "󰍽"
                            color: root.foreground
                            font.family: "monospace"
                            font.pixelSize: Style.font.heading
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: root.deviceName.toUpperCase()
                            color: root.foreground
                            font.family: root.fontFamily
                            font.pixelSize: Style.font.subtitle
                            font.bold: true
                        }
                    }

                    // ── Battery & Telemetry Banner ──
                    Rectangle {
                        width: parent.width
                        height: Style.space(50)
                        radius: 8
                        color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, root.mouseConnected ? 0.08 : 0.04)
                        border.width: 1
                        border.color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.12)

                        Row {
                            anchors.centerIn: parent
                            spacing: Style.space(14)

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: root.mouseConnected ? Model.batteryIcon(root.batteryPercentage, root.batteryStatus) : "󰍽"
                                color: root.foreground
                                opacity: root.mouseConnected ? 1.0 : 0.5
                                font.family: "monospace"
                                font.pixelSize: Style.font.display
                            }

                            Column {
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: Style.space(2)

                                Text {
                                    text: root.mouseConnected ? "Battery: " + Model.batteryText(root.batteryPercentage) + " (" + root.batteryStatus + ")" : "Disconnected"
                                    color: root.foreground
                                    font.family: root.fontFamily
                                    font.pixelSize: Style.font.body
                                    font.bold: true
                                }

                                Text {
                                    text: root.mouseConnected ? "DPI: " + Model.dpiText(root.dpiX, root.dpiY) + " · LOD: " + Model.lodLabel(root.lod) + " · " + root.reportRate + "Hz" : "Check USB connection / permissions"
                                    color: root.foreground
                                    font.family: root.fontFamily
                                    font.pixelSize: Style.font.bodySmall
                                    opacity: 0.6
                                }
                            }
                        }
                    }

                    // ── TABS: only displayed if mouse supports HITS analog switches ──
                    Row {
                        width: parent.width
                        spacing: Style.space(6)
                        visible: root.mouseConnected && root.hasHits

                        Repeater {
                            model: [
                                {
                                    id: "sensor",
                                    label: "Sensor (DPI / Polling)"
                                },
                                {
                                    id: "buttons",
                                    label: "Buttons (Analog HITS)"
                                }
                            ]
                            delegate: Rectangle {
                                width: Math.floor((content.width - Style.space(6)) / 2)
                                height: Style.space(30)
                                radius: 4
                                color: root.currentTab === modelData.id ? Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.22) : Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.06)
                                border.width: root.currentTab === modelData.id ? 1 : 0
                                border.color: root.foreground

                                Text {
                                    anchors.centerIn: parent
                                    text: modelData.label
                                    color: root.foreground
                                    font.family: root.fontFamily
                                    font.pixelSize: Style.font.bodySmall
                                    font.bold: root.currentTab === modelData.id
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.currentTab = modelData.id
                                }
                            }
                        }
                    }

                    // ═══════════════════════════════════════════════════════════
                    // ── TAB 1: SENSOR (DPI & POLLING RATE)
                    // ═══════════════════════════════════════════════════════════
                    Column {
                        width: parent.width
                        spacing: Style.space(12)
                        visible: root.mouseConnected && (!root.hasHits || root.currentTab === "sensor")

                        // Section 1: Sensitivity
                        Column {
                            width: parent.width
                            spacing: Style.space(8)

                            Text {
                                text: "SENSITIVITY (DPI)"
                                color: root.foreground
                                font.family: root.fontFamily
                                font.pixelSize: Style.font.bodySmall
                                font.bold: true
                                opacity: 0.6
                            }

                            // Dynamic Presets Buttons (filling 100% width)
                            Row {
                                width: parent.width
                                spacing: Style.space(6)
                                Repeater {
                                    model: root.dpiPresets
                                    delegate: Rectangle {
                                        width: Math.floor((parent.width - Style.space(6) * (root.dpiPresets.length - 1)) / root.dpiPresets.length)
                                        height: Style.space(28)
                                        radius: 4
                                        color: root.dpiX === modelData ? Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.22) : Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.06)
                                        border.width: root.dpiX === modelData ? 1 : 0
                                        border.color: root.foreground

                                        Text {
                                            anchors.centerIn: parent
                                            text: String(modelData)
                                            color: root.foreground
                                            font.family: root.fontFamily
                                            font.pixelSize: Style.font.bodySmall
                                            font.bold: root.dpiX === modelData
                                        }

                                        MouseArea {
                                            anchors.fill: parent
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: {
                                                dpiSlider.value = modelData;
                                                if (root.mouse)
                                                root.mouse.setDpi(modelData);
                                            }
                                        }
                                    }
                                }
                            }

                            // Continuous DPI Slider (Dynamically bounded from dpiMin to dpiMax)
                            Column {
                                width: parent.width
                                spacing: 0

                                Item {
                                    width: parent.width
                                    height: Style.space(20)

                                    Text {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Custom Sensitivity"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.caption
                                        opacity: 0.8
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: Math.round(dpiSlider.dragging ? dpiSlider.liveValue : (root.mouse ? root.mouse.dpiX : 800)) + " DPI"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }
                                }

                                PanelSlider {
                                    id: dpiSlider
                                    width: parent.width
                                    bar: root.bar
                                    minimum: root.dpiMin
                                    maximum: root.dpiMax
                                    step: 50
                                    integer: true
                                    tickCount: 0
                                    value: root.dpiX || 800
                                    onMoved: function (val) {
                                        dpiSlider.value = Math.round(val / 50) * 50;
                                    }
                                    onReleased: function (val) {
                                        var target = Math.round(val / 50) * 50;
                                        dpiSlider.value = target;
                                        if (root.mouse)
                                        root.mouse.setDpi(target);
                                    }
                                }

                                // Dynamic Min, Mid, and Max range markers
                                Item {
                                    width: parent.width
                                    height: Style.space(14)

                                    Text {
                                        anchors.left: parent.left
                                        text: String(root.dpiMin)
                                        color: root.foreground
                                        opacity: 0.35
                                        font.family: root.fontFamily
                                        font.pixelSize: 10
                                    }

                                    Text {
                                        anchors.centerIn: parent
                                        text: Math.round(root.dpiMax / 2).toLocaleString()
                                        color: root.foreground
                                        opacity: 0.25
                                        font.family: root.fontFamily
                                        font.pixelSize: 10
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        text: root.dpiMax.toLocaleString()
                                        color: root.foreground
                                        opacity: 0.35
                                        font.family: root.fontFamily
                                        font.pixelSize: 10
                                    }
                                }
                            }
                        }

                        // Divider
                        Rectangle {
                            width: parent.width
                            height: 1
                            color: root.foreground
                            opacity: 0.1
                        }

                        // Section 2: Report Rate
                        Column {
                            width: parent.width
                            spacing: Style.space(8)

                            Item {
                                width: parent.width
                                height: Style.space(20)

                                Text {
                                    anchors.left: parent.left
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: "REPORT RATE (POLLING)"
                                    color: root.foreground
                                    font.family: root.fontFamily
                                    font.pixelSize: Style.font.bodySmall
                                    font.bold: true
                                    opacity: 0.6
                                }

                                Text {
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: (root.mouse ? root.mouse.reportRate : 1000) + " Hz"
                                    color: root.foreground
                                    font.family: root.fontFamily
                                    font.pixelSize: Style.font.bodySmall
                                    font.bold: true
                                }
                            }

                            // Rate Buttons: [125, 250, 500, 1000, 2000, 4000, 8000] (7 items filling 100% width)
                            Row {
                                width: parent.width
                                spacing: Style.space(6)
                                Repeater {
                                    model: [125, 250, 500, 1000, 2000, 4000, 8000]
                                    delegate: Rectangle {
                                        width: Math.floor((parent.width - Style.space(6) * 6) / 7)
                                        height: Style.space(26)
                                        radius: 4
                                        color: (root.mouse && root.mouse.reportRate === modelData) ? Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.22) : Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.06)
                                        border.width: (root.mouse && root.mouse.reportRate === modelData) ? 1 : 0
                                        border.color: root.foreground

                                        Text {
                                            anchors.centerIn: parent
                                            text: modelData >= 1000 ? (modelData / 1000) + "K" : String(modelData)
                                            color: root.foreground
                                            font.family: root.fontFamily
                                            font.pixelSize: Style.font.caption
                                            font.bold: root.mouse && root.mouse.reportRate === modelData
                                        }

                                        MouseArea {
                                            anchors.fill: parent
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: if (root.mouse)
                                            root.mouse.setReportRate(modelData)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ═══════════════════════════════════════════════════════════
                    // ── TAB 2: BUTTONS (ONLY AVAILABLE IF MOUSE SUPPORTS HITS)
                    // ═══════════════════════════════════════════════════════════
                    Column {
                        width: parent.width
                        spacing: Style.space(14)
                        visible: root.mouseConnected && root.hasHits && root.currentTab === "buttons"

                        // ──────────────────────────────────────────
                        // ── LEFT BUTTON
                        // ──────────────────────────────────────────
                        Column {
                            width: parent.width
                            spacing: Style.space(10)

                            Rectangle {
                                width: parent.width
                                height: Style.space(24)
                                radius: 4
                                color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.06)

                                Text {
                                    anchors.left: parent.left
                                    anchors.leftMargin: Style.space(8)
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: "LEFT BUTTON"
                                    color: root.foreground
                                    font.family: root.fontFamily
                                    font.pixelSize: Style.font.caption
                                    font.bold: true
                                    opacity: 0.85
                                }
                            }

                            // 1. Actuation Point Slider
                            Column {
                                width: parent.width
                                spacing: 0

                                Item {
                                    width: parent.width
                                    height: Style.space(20)

                                    Text {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Actuation Point"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Level " + Math.round(leftActSlider.dragging ? leftActSlider.liveValue : (root.mouse ? root.mouse.hitsLeft.actuation : 1)) + " / 10"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }
                                }

                                PanelSlider {
                                    id: leftActSlider
                                    width: parent.width
                                    bar: root.bar
                                    minimum: 1
                                    maximum: 10
                                    step: 1
                                    integer: true
                                    tickCount: 10
                                    value: root.hitsLeft.actuation || 1
                                    onMoved: function (val) {
                                        leftActSlider.value = Math.round(val);
                                    }
                                    onReleased: function (val) {
                                        leftActSlider.value = Math.round(val);
                                        if (root.mouse)
                                        root.mouse.setActuation(0, Math.round(val));
                                    }
                                }

                                Item {
                                    width: parent.width
                                    height: Style.space(14)
                                    Repeater {
                                        model: 10
                                        Text {
                                            required property int index
                                            readonly property int level: index + 1
                                            readonly property int cur: Math.round(leftActSlider.dragging ? leftActSlider.liveValue : leftActSlider.value)
                                            readonly property bool isCurrent: level === cur
                                            text: String(level)
                                            color: root.foreground
                                            opacity: isCurrent ? 1.0 : 0.35
                                            font.family: root.fontFamily
                                            font.pixelSize: 10
                                            font.bold: isCurrent
                                            x: Math.max(0, Math.min(parent.width - implicitWidth, parent.width * (index / 9) - implicitWidth / 2))
                                        }
                                    }
                                }
                            }

                            // 2. Rapid Trigger Slider
                            Column {
                                width: parent.width
                                spacing: 0

                                Item {
                                    width: parent.width
                                    height: Style.space(20)

                                    Text {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Rapid Trigger"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Level " + Math.round(leftRtSlider.dragging ? leftRtSlider.liveValue : (root.mouse ? root.mouse.hitsLeft.rapidTrigger : 1)) + " / 5"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }
                                }

                                PanelSlider {
                                    id: leftRtSlider
                                    width: parent.width
                                    bar: root.bar
                                    minimum: 1
                                    maximum: 5
                                    step: 1
                                    integer: true
                                    tickCount: 5
                                    value: root.hitsLeft.rapidTrigger || 1
                                    onMoved: function (val) {
                                        leftRtSlider.value = Math.round(val);
                                    }
                                    onReleased: function (val) {
                                        leftRtSlider.value = Math.round(val);
                                        if (root.mouse)
                                        root.mouse.setRapidTrigger(0, Math.round(val));
                                    }
                                }

                                Item {
                                    width: parent.width
                                    height: Style.space(14)
                                    Repeater {
                                        model: 5
                                        Text {
                                            required property int index
                                            readonly property int level: index + 1
                                            readonly property int cur: Math.round(leftRtSlider.dragging ? leftRtSlider.liveValue : leftRtSlider.value)
                                            readonly property bool isCurrent: level === cur
                                            text: String(level)
                                            color: root.foreground
                                            opacity: isCurrent ? 1.0 : 0.35
                                            font.family: root.fontFamily
                                            font.pixelSize: 10
                                            font.bold: isCurrent
                                            x: Math.max(0, Math.min(parent.width - implicitWidth, parent.width * (index / 4) - implicitWidth / 2))
                                        }
                                    }
                                }
                            }

                            // 3. Click Haptics Slider
                            Column {
                                width: parent.width
                                spacing: 0

                                Item {
                                    width: parent.width
                                    height: Style.space(20)

                                    Text {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Click Haptics"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Level " + Math.round(leftHapSlider.dragging ? leftHapSlider.liveValue : (root.mouse ? root.mouse.hitsLeft.haptics : 1)) + " / 6"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }
                                }

                                PanelSlider {
                                    id: leftHapSlider
                                    width: parent.width
                                    bar: root.bar
                                    minimum: 1
                                    maximum: 6
                                    step: 1
                                    integer: true
                                    tickCount: 6
                                    value: root.hitsLeft.haptics || 1
                                    onMoved: function (val) {
                                        leftHapSlider.value = Math.round(val);
                                    }
                                    onReleased: function (val) {
                                        leftHapSlider.value = Math.round(val);
                                        if (root.mouse)
                                        root.mouse.setHaptics(0, Math.round(val));
                                    }
                                }

                                Item {
                                    width: parent.width
                                    height: Style.space(14)
                                    Repeater {
                                        model: 6
                                        Text {
                                            required property int index
                                            readonly property int level: index + 1
                                            readonly property int cur: Math.round(leftHapSlider.dragging ? leftHapSlider.liveValue : leftHapSlider.value)
                                            readonly property bool isCurrent: level === cur
                                            text: String(level)
                                            color: root.foreground
                                            opacity: isCurrent ? 1.0 : 0.35
                                            font.family: root.fontFamily
                                            font.pixelSize: 10
                                            font.bold: isCurrent
                                            x: Math.max(0, Math.min(parent.width - implicitWidth, parent.width * (index / 5) - implicitWidth / 2))
                                        }
                                    }
                                }
                            }
                        }

                        // Divider between buttons
                        Rectangle {
                            width: parent.width
                            height: 1
                            color: root.foreground
                            opacity: 0.1
                        }

                        // ──────────────────────────────────────────
                        // ── RIGHT BUTTON
                        // ──────────────────────────────────────────
                        Column {
                            width: parent.width
                            spacing: Style.space(10)

                            Rectangle {
                                width: parent.width
                                height: Style.space(24)
                                radius: 4
                                color: Qt.rgba(root.foreground.r, root.foreground.g, root.foreground.b, 0.06)

                                Text {
                                    anchors.left: parent.left
                                    anchors.leftMargin: Style.space(8)
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: "RIGHT BUTTON"
                                    color: root.foreground
                                    font.family: root.fontFamily
                                    font.pixelSize: Style.font.caption
                                    font.bold: true
                                    opacity: 0.85
                                }
                            }

                            // 1. Actuation Point Slider
                            Column {
                                width: parent.width
                                spacing: 0

                                Item {
                                    width: parent.width
                                    height: Style.space(20)

                                    Text {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Actuation Point"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Level " + Math.round(rightActSlider.dragging ? rightActSlider.liveValue : (root.mouse ? root.mouse.hitsRight.actuation : 1)) + " / 10"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }
                                }

                                PanelSlider {
                                    id: rightActSlider
                                    width: parent.width
                                    bar: root.bar
                                    minimum: 1
                                    maximum: 10
                                    step: 1
                                    integer: true
                                    tickCount: 10
                                    value: root.hitsRight.actuation || 1
                                    onMoved: function (val) {
                                        rightActSlider.value = Math.round(val);
                                    }
                                    onReleased: function (val) {
                                        rightActSlider.value = Math.round(val);
                                        if (root.mouse)
                                        root.mouse.setActuation(1, Math.round(val));
                                    }
                                }

                                Item {
                                    width: parent.width
                                    height: Style.space(14)
                                    Repeater {
                                        model: 10
                                        Text {
                                            required property int index
                                            readonly property int level: index + 1
                                            readonly property int cur: Math.round(rightActSlider.dragging ? rightActSlider.liveValue : rightActSlider.value)
                                            readonly property bool isCurrent: level === cur
                                            text: String(level)
                                            color: root.foreground
                                            opacity: isCurrent ? 1.0 : 0.35
                                            font.family: root.fontFamily
                                            font.pixelSize: 10
                                            font.bold: isCurrent
                                            x: Math.max(0, Math.min(parent.width - implicitWidth, parent.width * (index / 9) - implicitWidth / 2))
                                        }
                                    }
                                }
                            }

                            // 2. Rapid Trigger Slider
                            Column {
                                width: parent.width
                                spacing: 0

                                Item {
                                    width: parent.width
                                    height: Style.space(20)

                                    Text {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Rapid Trigger"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Level " + Math.round(rightRtSlider.dragging ? rightRtSlider.liveValue : (root.mouse ? root.mouse.hitsRight.rapidTrigger : 1)) + " / 5"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }
                                }

                                PanelSlider {
                                    id: rightRtSlider
                                    width: parent.width
                                    bar: root.bar
                                    minimum: 1
                                    maximum: 5
                                    step: 1
                                    integer: true
                                    tickCount: 5
                                    value: root.hitsRight.rapidTrigger || 1
                                    onMoved: function (val) {
                                        rightRtSlider.value = Math.round(val);
                                    }
                                    onReleased: function (val) {
                                        rightRtSlider.value = Math.round(val);
                                        if (root.mouse)
                                        root.mouse.setRapidTrigger(1, Math.round(val));
                                    }
                                }

                                Item {
                                    width: parent.width
                                    height: Style.space(14)
                                    Repeater {
                                        model: 5
                                        Text {
                                            required property int index
                                            readonly property int level: index + 1
                                            readonly property int cur: Math.round(rightRtSlider.dragging ? rightRtSlider.liveValue : rightRtSlider.value)
                                            readonly property bool isCurrent: level === cur
                                            text: String(level)
                                            color: root.foreground
                                            opacity: isCurrent ? 1.0 : 0.35
                                            font.family: root.fontFamily
                                            font.pixelSize: 10
                                            font.bold: isCurrent
                                            x: Math.max(0, Math.min(parent.width - implicitWidth, parent.width * (index / 4) - implicitWidth / 2))
                                        }
                                    }
                                }
                            }

                            // 3. Click Haptics Slider
                            Column {
                                width: parent.width
                                spacing: 0

                                Item {
                                    width: parent.width
                                    height: Style.space(20)

                                    Text {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Click Haptics"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }

                                    Text {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Level " + Math.round(rightHapSlider.dragging ? rightHapSlider.liveValue : (root.mouse ? root.mouse.hitsRight.haptics : 1)) + " / 6"
                                        color: root.foreground
                                        font.family: root.fontFamily
                                        font.pixelSize: Style.font.bodySmall
                                        font.bold: true
                                    }
                                }

                                PanelSlider {
                                    id: rightHapSlider
                                    width: parent.width
                                    bar: root.bar
                                    minimum: 1
                                    maximum: 6
                                    step: 1
                                    integer: true
                                    tickCount: 6
                                    value: root.hitsRight.haptics || 1
                                    onMoved: function (val) {
                                        rightHapSlider.value = Math.round(val);
                                    }
                                    onReleased: function (val) {
                                        rightHapSlider.value = Math.round(val);
                                        if (root.mouse)
                                        root.mouse.setHaptics(1, Math.round(val));
                                    }
                                }

                                Item {
                                    width: parent.width
                                    height: Style.space(14)
                                    Repeater {
                                        model: 6
                                        Text {
                                            required property int index
                                            readonly property int level: index + 1
                                            readonly property int cur: Math.round(rightHapSlider.dragging ? rightHapSlider.liveValue : rightHapSlider.value)
                                            readonly property bool isCurrent: level === cur
                                            text: String(level)
                                            color: root.foreground
                                            opacity: isCurrent ? 1.0 : 0.35
                                            font.family: root.fontFamily
                                            font.pixelSize: 10
                                            font.bold: isCurrent
                                            x: Math.max(0, Math.min(parent.width - implicitWidth, parent.width * (index / 5) - implicitWidth / 2))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

