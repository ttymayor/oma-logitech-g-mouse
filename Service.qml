import QtQuick
import Quickshell
import Quickshell.Io
import "Model.js" as Model

Item {
    id: root

    property var settings: ({})

    property bool connected: false
    property string deviceName: "Logitech G Mouse"
    property int batteryPercentage: Model.LEVEL_UNKNOWN
    property string batteryLevel: "unknown"
    property string batteryStatus: "unknown"
    property int dpiX: 0
    property int defaultDpiX: 0
    property int dpiY: 0
    property int defaultDpiY: 0
    property int dpiMin: 100
    property int dpiMax: 32000
    property var dpiPresets: [800, 1200, 1600, 2400, 3200]
    property int reportRate: 1000
    property string lod: "unknown"
    property bool hasHits: false
    property var hitsLeft: Model.defaultButton()
    property var hitsRight: Model.defaultButton()
    property string lastError: ""

    readonly property int pollIntervalSec: {
        var v = settings ? settings.pollInterval : undefined
        return Math.max(5, Number(v) || 15)
    }

    readonly property string statePath: "/tmp/omarchy-logitech-g-mouse.json"
    readonly property string daemonPath: Quickshell.env("HOME") + "/.config/omarchy/plugins/tantuyu.logitech-g-mouse/bin/logitech-g-daemon"

    function refresh() {
        stateFile.reload()
        if (!pollProc.running) {
            pollProc.running = true
        }
    }

    function applyStatus(status) {
        if (!status) return
        connected = (status.connected === true)
        deviceName = String(status.deviceName || "Logitech G Mouse")
        batteryPercentage = Number(status.batteryPercentage)
        batteryLevel = String(status.batteryLevel || "unknown")
        batteryStatus = String(status.batteryStatus || "unknown")
        dpiX = Number(status.dpiX || 0)
        defaultDpiX = Number(status.defaultDpiX || 0)
        dpiY = Number(status.dpiY || 0)
        defaultDpiY = Number(status.defaultDpiY || 0)
        dpiMin = Number(status.dpiMin || 100)
        dpiMax = Number(status.dpiMax || 32000)
        dpiPresets = status.dpiPresets || [800, 1200, 1600, 2400, 3200]
        reportRate = Number(status.reportRate || 1000)
        lod = String(status.lod || "unknown")
        hasHits = !!status.hasHits
        hitsLeft = status.hitsLeft || Model.defaultButton()
        hitsRight = status.hitsRight || Model.defaultButton()
        lastError = String(status.error || "")
    }

    function applyLine(raw) {
        var parsed = Model.parseStatus(raw)
        applyStatus(parsed)
    }

    // Optimistic UI updates
    function setActuation(btn, level) {
        var act = Math.max(1, Math.min(10, Math.round(level)))
        if (btn === 0) {
            hitsLeft = {
                actuation: act,
                rapidTrigger: hitsLeft.rapidTrigger,
                haptics: hitsLeft.haptics
            }
        } else {
            hitsRight = {
                actuation: act,
                rapidTrigger: hitsRight.rapidTrigger,
                haptics: hitsRight.haptics
            }
        }

        var args = [root.daemonPath, "--set-actuation", String(act)]
        if (btn === 0) args.push("--left")
        else if (btn === 1) args.push("--right")
        cmdProc.command = args
        cmdProc.running = true
    }

    function setRapidTrigger(btn, level) {
        var rt = Math.max(1, Math.min(5, Math.round(level)))
        if (btn === 0) {
            hitsLeft = {
                actuation: hitsLeft.actuation,
                rapidTrigger: rt,
                haptics: hitsLeft.haptics
            }
        } else {
            hitsRight = {
                actuation: hitsRight.actuation,
                rapidTrigger: rt,
                haptics: hitsRight.haptics
            }
        }

        var args = [root.daemonPath, "--set-rt", String(rt)]
        if (btn === 0) args.push("--left")
        else if (btn === 1) args.push("--right")
        cmdProc.command = args
        cmdProc.running = true
    }

    function setHaptics(btn, level) {
        var hap = Math.max(0, Math.min(5, Math.round(level)))
        if (btn === 0) {
            hitsLeft = {
                actuation: hitsLeft.actuation,
                rapidTrigger: hitsLeft.rapidTrigger,
                haptics: hap
            }
        } else {
            hitsRight = {
                actuation: hitsRight.actuation,
                rapidTrigger: hitsRight.rapidTrigger,
                haptics: hap
            }
        }

        var args = [root.daemonPath, "--set-haptics", String(hap)]
        if (btn === 0) args.push("--left")
        else if (btn === 1) args.push("--right")
        cmdProc.command = args
        cmdProc.running = true
    }

    function setDpi(dpi) {
        dpiX = dpi
        dpiY = dpi
        cmdProc.command = [root.daemonPath, "--set-dpi", String(dpi)]
        cmdProc.running = true
    }

    function setReportRate(rate) {
        reportRate = rate
        cmdProc.command = [root.daemonPath, "--set-rate", String(rate)]
        cmdProc.running = true
    }

    Process {
        id: cmdProc
        stdout: StdioCollector {
            id: cmdOut
            waitForEnd: true
            onStreamFinished: {
                var lines = (cmdOut.text || "").trim()
                if (lines !== "") {
                    root.applyLine(lines)
                }
            }
        }
    }

    FileView {
        id: stateFile
        path: root.statePath
        watchChanges: true
        printErrors: false
        onFileChanged: reload()
        onLoaded: root.applyLine(text())
    }

    Timer {
        id: pollTimer
        interval: root.pollIntervalSec * 1000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: root.refresh()
    }

    Process {
        id: pollProc
        command: [
            root.daemonPath,
            "--once"
        ]
        stdout: StdioCollector {
            id: pollOutput
            waitForEnd: true
            onStreamFinished: {
                var text = (pollOutput.text || "").trim()
                if (text !== "") {
                    root.applyLine(text)
                }
            }
        }
    }

    Component.onCompleted: {
        root.refresh()
    }
}
