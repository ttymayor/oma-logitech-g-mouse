# Logitech G Mouse — Omarchy Shell Plugin (oma-logitech-g-mouse)

**English** | [繁體中文](README.zh-tw.md)

An [Omarchy Quattro](https://omarchyplugins.com/develop.html) bar widget and control panel plugin for **Logitech G Gaming Mice** (including G PRO X2 SUPERSTRIKE, G PRO X SUPERLIGHT 1/2, G502 Series, etc.).

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
  - **Quick DPI Presets**: Full-width buttons dynamically adapting to the mouse's onboard presets (e.g. `800` / `1200` / `1600` / `2400` / `3200`), click to apply.
  - **Continuous DPI Slider**: Smooth, continuous slider adjusting dynamically from `100` up to your sensor's maximum (up to `32,000 DPI` on HERO 2) in 50 DPI steps.
  - **Report Rate (Polling Rate)**: 7 full-width buttons (`125` / `250` / `500` / `1K` / `2K` / `4K` / `8K Hz`) backed by native HID++ feature `0x8061`.
- 🏷️ **Tab 2: `Buttons (Analog HITS)`** *(automatically shown if mouse has analog switches)*
  - **Independent Left and Right Button Tuning**, each equipped with 3 native monochrome sliders:
    1. **Actuation Point**: Levels `1`–`10` (downward trigger travel: 0.1 mm ~ 1.0 mm).
    2. **Rapid Trigger**: Levels `1`–`5` (upward reset travel: 0.1 mm ~ 0.5 mm).
    3. **Click Haptics**: Levels `1`–`6` (tactile feedback vibration strength).
  - **Aligned Tick Numbers**: A numerical ruler directly under each slider tracks each notch (`1..10`, `1..5`, `1..6`), with the active value bolded and highlighted.
  - **Zero-Rebound Technology**: Uses optimistic UI updates so the slider knob stays smoothly at the user's release point without snapping back or stuttering.

---

## 📦 Install & Uninstall

### Install

#### Method A: Git Install (via Omarchy CLI)
```bash
omarchy plugin add https://github.com/<username>/oma-logitech-g-mouse.git --enable
omarchy restart shell
```

#### Method B: Local Manual Install (from source checkout)
```bash
# 1. Copy plugin files to the Omarchy user plugins directory
mkdir -p ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse
cp -a * ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/

# 2. Validate plugin structure
omarchy plugin validate ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse

# 3. Discover and enable the plugin on the right section of the bar
omarchy-shell shell rescanPlugins
omarchy plugin enable tantuyu.logitech-g-mouse --section right

# 4. Restart Omarchy shell to compile and load QML components
omarchy restart shell
```

---

### Uninstall

#### Method A: Remove via Omarchy CLI
```bash
# Disable the plugin from the bar and remove files
omarchy plugin disable tantuyu.logitech-g-mouse
omarchy plugin remove tantuyu.logitech-g-mouse --yes
omarchy restart shell
```

#### Method B: Manual Complete Removal
```bash
# 1. Disable the plugin
omarchy plugin disable tantuyu.logitech-g-mouse

# 2. Remove the plugin folder and temporary telemetry cache
rm -rf ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse
rm -f /tmp/omarchy-logitech-g-mouse.json

# 3. Notify the shell and restart
omarchy-shell shell rescanPlugins
omarchy restart shell
```

---

## 🛠️ Modifying & Applying Changes

- **Modifying QML / UI (`Panel.qml`, `BarWidget.qml`, `Model.js`)**:  
  Omarchy disables QML automatic file watching for system stability, so changes **require a shell restart**:
  ```bash
  cp -a . ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/ && omarchy restart shell
  ```
- **Modifying TypeScript Backend (`logitech-g-daemon.ts`)**:  
  **No shell restart required!** The script is invoked dynamically via Bun upon clicks or polling events; changes take effect immediately on the next action.
- **One-Liner Sync & Reload Command**:
  ```bash
  omarchy plugin validate . && cp -a . ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/ && omarchy restart shell
  ```

---

## 💻 Standalone CLI Usage

The core driver `logitech-g-daemon.ts` can also be run independently from the command line:

```bash
# Print live telemetry in JSON format
bun run logitech-g-daemon.ts --once

# Set DPI (supports 100 to 32,000 DPI)
bun run logitech-g-daemon.ts --set-dpi 1600

# Set Report Rate (125 / 250 / 500 / 1000 / 2000 / 4000 / 8000 Hz)
bun run logitech-g-daemon.ts --set-rate 4000

# Set Actuation Point (Levels 1–10)
bun run logitech-g-daemon.ts --set-actuation 4 --left
bun run logitech-g-daemon.ts --set-actuation 6 --right

# Set Rapid Trigger (Levels 1–5)
bun run logitech-g-daemon.ts --set-rt 2 --left

# Set Click Haptics (Levels 1–6)
bun run logitech-g-daemon.ts --set-haptics 5 --left
```

---

## 🔬 HID++ 2.0 Protocol Reference

| Capability | Feature ID | Index | Description |
| :--- | :--- | :--- | :--- |
| **Device Name** | `0x0005` | `0x03` | Dynamic marketing name query (e.g. `PRO X2 SUPERSTRIKE`, `PRO X SUPERLIGHT`) |
| **Unified Battery** | `0x1004` | `0x06` | Charge percentage, battery level, charging/discharging state |
| **Extended DPI** | `0x2202` | `0x09` | Sensor parameters (supports up to 32K DPI) and LOD status |
| **Onboard Profiles** | `0x8100` | `0x0e` | Profile switching, Host Mode unlock for custom DPI |
| **HITS Analog Switches** | `0x1B0C` | `0x0c` | Actuation (1–10), Rapid Trigger (1–5), Click Haptics (1–6) |
| **Extended Report Rate** | `0x8061` | `0x0d` | Polling rate control from 125 Hz up to 8000 Hz |

---

## 🔒 Permissions & Udev Rules

> **Note:** On Arch Linux, Omarchy, and most modern systemd-based desktop distributions, **you do NOT need to perform this step!**  
> `systemd-logind` automatically applies `uaccess` dynamic ACLs granting your logged-in desktop user direct read/write permissions (`user:username:rw-`) to `/dev/hidraw*` devices.

### When is this required?
This step is only necessary if:
1. Running `bun run logitech-g-daemon.ts --once` fails with an `EACCES: Permission denied` error.
2. You are using a minimal distribution without `systemd-logind`, running in a container, or running over a headless session where `uaccess` seat management is not active.

In those cases, create this udev rule to grant world read/write permissions to Logitech HID devices:

```bash
sudo tee /etc/udev/rules.d/42-logitech-mouse.rules << 'EOF'
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="046d", MODE="0666"
EOF

sudo udevadm control --reload && sudo udevadm trigger
```

---

## 📄 License

MIT License.  
Logitech, G PRO, and SUPERSTRIKE are trademarks of Logitech. This project is an independent third-party open-source plugin and is not affiliated with Logitech.
