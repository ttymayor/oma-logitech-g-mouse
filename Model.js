// Model.js — Pure JS helper for parsing superstrike-daemon JSON output.

var LEVEL_UNKNOWN = -1

function defaultButton() {
  return { actuation: 0, rapidTrigger: 0, haptics: 0 }
}

function defaultStatus() {
  return {
    ok: true,
    connected: false,
    deviceName: "Logitech G Mouse",
    batteryPercentage: LEVEL_UNKNOWN,
    batteryLevel: "unknown",
    batteryStatus: "unknown",
    dpiX: 0,
    defaultDpiX: 0,
    dpiY: 0,
    defaultDpiY: 0,
    dpiMin: 100,
    dpiMax: 32000,
    dpiPresets: [800, 1200, 1600, 2400, 3200],
    reportRate: 1000,
    lod: "unknown",
    hasHits: false,
    hitsLeft: defaultButton(),
    hitsRight: defaultButton(),
    error: ""
  }
}

function parseStatus(raw) {
  var s = defaultStatus()
  if (!raw || raw === "") return s

  var data
  try { data = JSON.parse(raw) }
  catch (e) { s.ok = false; s.error = "parse error: " + e; return s }

  s.connected = !!data.connected
  if (data.deviceName) s.deviceName = String(data.deviceName)

  if (data.battery) {
    s.batteryPercentage = typeof data.battery.percentage === "number" ? data.battery.percentage : LEVEL_UNKNOWN
    s.batteryLevel = String(data.battery.level || "unknown")
    s.batteryStatus = String(data.battery.status || "unknown")
  }

  if (data.dpi) {
    s.dpiX = data.dpi.dpiX || 0
    s.defaultDpiX = data.dpi.defaultDpiX || 0
    s.dpiY = data.dpi.dpiY || 0
    s.defaultDpiY = data.dpi.defaultDpiY || 0
    s.lod = String(data.dpi.lod || "unknown")
  }

  if (typeof data.dpiMin === "number") s.dpiMin = data.dpiMin
  if (typeof data.dpiMax === "number") s.dpiMax = data.dpiMax
  if (Array.isArray(data.dpiPresets) && data.dpiPresets.length > 0) {
    s.dpiPresets = data.dpiPresets
  }

  if (data.reportRate) {
    s.reportRate = Number(data.reportRate) || 1000
  }

  s.hasHits = !!data.hasHits

  if (data.hits) {
    if (data.hits.left) {
      s.hitsLeft = {
        actuation: data.hits.left.actuation || 0,
        rapidTrigger: data.hits.left.rapidTrigger || 0,
        haptics: data.hits.left.haptics || 0
      }
    }
    if (data.hits.right) {
      s.hitsRight = {
        actuation: data.hits.right.actuation || 0,
        rapidTrigger: data.hits.right.rapidTrigger || 0,
        haptics: data.hits.right.haptics || 0
      }
    }
  }

  if (data.error) s.error = String(data.error)
  s.ok = true
  return s
}

// Standard glyphs matching Omarchy power & input plugins
function mouseIcon() {
  return "󰍽"  // nf-md-mouse
}

function batteryIcon(percentage, status) {
  var chargingIcons = ["󰢜", "󰂆", "󰂇", "󰂈", "󰢝", "󰂉", "󰢞", "󰂊", "󰂋", "󰂅"]
  var defaultIcons  = ["󰁺", "󰁻", "󰁼", "󰁽", "󰁾", "󰁿", "󰂀", "󰂁", "󰂂", "󰁹"]

  if (percentage === LEVEL_UNKNOWN) return "󰍽"
  var idx = Math.max(0, Math.min(9, Math.floor(percentage / 10)))

  if (status === "charging" || status === "charging_slow") {
    return chargingIcons[idx]
  }
  if (status === "full") return "󰂅"
  return defaultIcons[idx]
}

function batteryText(percentage) {
  if (percentage === LEVEL_UNKNOWN) return "--"
  return String(percentage) + "%"
}

function dpiText(dpiX, dpiY) {
  if (dpiX <= 0) return "--"
  if (dpiY > 0 && dpiY !== dpiX) return String(dpiX) + "×" + String(dpiY)
  return String(dpiX)
}

function lodLabel(lod) {
  if (lod === "low") return "Low"
  if (lod === "medium") return "Medium"
  if (lod === "high") return "High"
  return "--"
}

function actuationLabel(level) {
  if (level <= 0) return "--"
  return "L" + String(level)
}

function rapidTriggerLabel(level) {
  if (level <= 0) return "--"
  return "L" + String(level)
}

function hapticsLabel(level) {
  if (level <= 0) return "--"
  return "L" + String(level)
}

function barTooltip(mouse) {
  var name = (mouse && mouse.deviceName) ? mouse.deviceName : "Logitech G Mouse"
  if (!mouse || !mouse.connected) return name + " (Disconnected)"
  var parts = [name]
  if (mouse.batteryPercentage !== LEVEL_UNKNOWN) {
    parts.push(batteryText(mouse.batteryPercentage) + " (" + mouse.batteryStatus + ")")
  }
  if (mouse.dpiX > 0) {
    parts.push(dpiText(mouse.dpiX, mouse.dpiY) + " DPI")
  }
  return parts.join(" · ")
}

if (typeof module !== "undefined") {
  module.exports = {
    LEVEL_UNKNOWN: LEVEL_UNKNOWN,
    defaultStatus: defaultStatus,
    parseStatus: parseStatus,
    mouseIcon: mouseIcon,
    batteryIcon: batteryIcon,
    batteryText: batteryText,
    dpiText: dpiText,
    barTooltip: barTooltip,
    actuationLabel: actuationLabel,
    rapidTriggerLabel: rapidTriggerLabel,
    hapticsLabel: hapticsLabel,
    lodLabel: lodLabel
  }
}
