import QtQuick
import Quickshell
import qs.Commons
import qs.Ui
import "Model.js" as Model

BarWidget {
    id: root
    moduleName: "tantuyu.g-pro-x2-superstrike"

    readonly property bool opened: panelLoader.item ? panelLoader.item.opened === true : false
    readonly property bool popoutSwitchClosing: panelLoader.item ? panelLoader.item.popoutSwitchClosing === true : false

    readonly property bool showPercentage: setting("showPercentage", true) === true

    function togglePercentage() {
        root.settings = Object.assign({}, root.settings, {
            showPercentage: !root.showPercentage
        });
        if (root.bar && root.bar.shell && typeof root.bar.shell.updateEntryInline === "function") {
            root.bar.shell.updateEntryInline(root.moduleName, root.settings);
        }
    }

    function open() {
        if (panelLoader.item) panelLoader.item.open()
        mouse.refresh()
    }

    function close() {
        if (panelLoader.item) panelLoader.item.close()
    }

    function toggle() {
        if (panelLoader.item) panelLoader.item.toggle()
        mouse.refresh()
    }

    function closeForPopoutSwitch() {
        if (panelLoader.item) panelLoader.item.closeForPopoutSwitch()
    }

    function injectPanel() {
        if (!panelLoader.item) return
        panelLoader.item.bar = root.bar
        panelLoader.item.anchorItem = button
        panelLoader.item.hostWidget = root
        panelLoader.item.mouse = mouse
    }

    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight

    onBarChanged: injectPanel()

    Service {
        id: mouse
        settings: root.settings
    }

    Loader {
        id: panelLoader
        active: true
        source: Qt.resolvedUrl("Panel.qml")
        visible: false
        onLoaded: {
            root.injectPanel();
            Qt.callLater(root.injectPanel);
        }
    }

    BarIconButton {
        id: button
        anchors.fill: parent
        bar: root.bar
        active: false
        useActiveColor: false
        slotSize: Style.bar.iconSlot * (root.showPercentage ? 2.2 : 1.0)
        tooltipText: Model.barTooltip(mouse)
        text: {
            if (!mouse.connected) {
                return root.showPercentage ? "Off 󰍽" : "󰍽";
            }
            var icon = Model.batteryIcon(mouse.batteryPercentage, mouse.batteryStatus);
            if (!root.showPercentage) {
                return icon;
            }
            return Model.batteryText(mouse.batteryPercentage) + " " + icon;
        }
        onPressed: function (buttonCode) {
            if (buttonCode === Qt.LeftButton) {
                root.toggle();
            } else if (buttonCode === Qt.RightButton) {
                root.togglePercentage();
            }
        }
    }
}

