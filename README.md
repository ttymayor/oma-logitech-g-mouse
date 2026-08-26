# Logitech G PRO X2 SUPERSTRIKE — Omarchy Shell Plugin

**English** | [繁體中文](README.zh-tw.md)

An [Omarchy Quattro](https://omarchyplugins.com/develop.html) bar widget and control panel plugin for the **Logitech G PRO X2 SUPERSTRIKE** analog gaming mouse.

Communicates directly with the Linux `/dev/hidraw` device interface via the Logitech **HID++ 2.0** protocol, providing complete telemetry and hardware customization without requiring Logitech G HUB.

---

## ✨ Features

### 1. Status Bar Widget
- **Live Battery & Status**: Shows status like `99% 󰁹`, dynamic charging indicator (`󰂅`), and disconnected status (`Off 󰍽`).
- **Right-Click Percentage Toggle**: Right-click the widget icon directly on the status bar to switch between compact icon mode (`󰁹`) and percentage mode (`99% 󰁹`). The preference is automatically persisted to `~/.config/omarchy/shell.json`.
- **Pure Monochromatic Theme**: Adheres strictly to the system `foreground` palette. The widget does not flicker or shift to accent colors when the panel is opened.

### 2. 2-Tab Control Panel
Left-click opens the floating card (`KeyboardPanel`), organized into two clean, self-contained tabs with zero vertical clipping:

- 🏷️ **Tab 1: `Sensor (DPI / Polling)`**
  - **Quick DPI Presets**: 5 buttons stretching across 100% of the width (`800` / `1200` / `1600` / `2400` / `3200`), click to apply.
  - **Continuous DPI Slider**: Smooth, continuous slider adjusting from `100` up to `32,000 DPI` in 50 DPI steps, with reference notches at `100`, `16,000`, and `32,000`.
  - **Report Rate (Polling Rate)**: 7 full-width buttons (`125` / `250` / `500` / `1K` / `2K` / `4K` / `8K Hz`) backed by native HID++ feature `0x8061`.
- 🏷️ **Tab 2: `Buttons (Analog HITS)`**
  - **Independent Left and Right Button Tuning**, each equipped with 3 native monochrome sliders:
    1. **Actuation Point**: Levels `1`–`10` (downward trigger travel: 0.1 mm ~ 1.0 mm).
    2. **Rapid Trigger**: Levels `1`–`5` (upward reset travel: 0.1 mm ~ 0.5 mm).
    3. **Click Haptics**: Levels `1`–`6` (tactile feedback vibration strength).
  - **Aligned Tick Numbers**: A numerical ruler directly under each slider tracks each notch (`1..10`, `1..5`, `1..6`), with the active value bolded and highlighted.
  - **Zero-Rebound Technology**: Uses optimistic UI updates so the slider knob stays smoothly at the user's release point without snapping back or stuttering.

---

## 🚀 Installation & Setup

### 1. Deploy to the Omarchy Plugins Directory
Copy or link the plugin files to `~/.config/omarchy/plugins/`:

```bash
mkdir -p ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike
cp -a * ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike/
```

### 2. Validate, Enable, and Restart Shell
```bash
# Validate plugin structure and manifest
omarchy plugin validate ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike

# Rescan shell plugin discovery
omarchy-shell shell rescanPlugins

# Enable and position on the right section of the bar
omarchy plugin enable tantuyu.g-pro-x2-superstrike --section right

# Restart Omarchy shell to load QML components
omarchy restart shell
```

---

## 🛠️ Modifying & Applying Changes

When you modify plugin source files, apply them based on the file type:

- **Modifying QML / UI (`Panel.qml`, `BarWidget.qml`, `Model.js`)**:
  Omarchy disables QML automatic file watching for system stability, so changes **require a shell restart**:
  ```bash
  cp -a . ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike/ && omarchy restart shell
  ```
- **Modifying TypeScript Backend (`superstrike-daemon.ts`)**:
  **No shell restart required!** The script is invoked dynamically via Bun upon clicks or polling events; changes take effect immediately on the next action.
- **One-Liner Sync & Reload Command**:
  ```bash
  omarchy plugin validate . && cp -a . ~/.config/omarchy/plugins/tantuyu.g-pro-x2-superstrike/ && omarchy restart shell
  ```

---

## 💻 Standalone CLI Usage

The core driver `superstrike-daemon.ts` can also be run independently from the command line:

```bash
# Print live telemetry in JSON format
bun run superstrike-daemon.ts --once

# Set DPI (supports 100 to 32,000 DPI)
bun run superstrike-daemon.ts --set-dpi 1600

# Set Report Rate (125 / 250 / 500 / 1000 / 2000 / 4000 / 8000 Hz)
bun run superstrike-daemon.ts --set-rate 4000

# Set Actuation Point (Levels 1–10)
bun run superstrike-daemon.ts --set-actuation 4 --left
bun run superstrike-daemon.ts --set-actuation 6 --right

# Set Rapid Trigger (Levels 1–5)
bun run superstrike-daemon.ts --set-rt 2 --left

# Set Click Haptics (Levels 1–6)
bun run superstrike-daemon.ts --set-haptics 5 --left
```

---

## 🔬 HID++ 2.0 Protocol Reference

This plugin reverse-engineers and implements the following hardware features:

| Capability | Feature ID | Index | Description |
| :--- | :--- | :--- | :--- |
| **Unified Battery** | `0x1004` | `0x06` | Charge percentage, battery level, charging/discharging state |
| **Extended DPI** | `0x2202` | `0x09` | Sensor parameters (supports up to 32K DPI) and LOD status |
| **Onboard Profiles** | `0x8100` | `0x0e` | Profile switching, Host Mode unlock for custom DPI |
| **HITS Analog Switches** | `0x1B0C` | `0x0c` | Actuation (1–10), Rapid Trigger (1–5), Click Haptics (1–6) |
| **Extended Report Rate** | `0x8061` | `0x0d` | Polling rate control from 125 Hz up to 8000 Hz |

---

## 🔒 Permissions & Udev Rules

To access `/dev/hidraw*` without root privileges, install the following udev rule:

```bash
sudo tee /etc/udev/rules.d/42-logitech-superstrike.rules << 'EOF'
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="046d", MODE="0666"
EOF

sudo udevadm control --reload && sudo udevadm trigger
```

---

## 📄 License

MIT License.  
Logitech, G PRO, and SUPERSTRIKE are trademarks of Logitech. This project is an independent third-party open-source plugin and is not affiliated with Logitech.
