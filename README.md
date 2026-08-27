# Logitech G Mouse — Omarchy Plugin (oma-logitech-g-mouse)

**English** | [繁體中文](README.zh-tw.md)

An [Omarchy Quattro](https://omarchy.org/) bar widget and control panel plugin for **Logitech G Gaming Mice** (including G PRO X2 SUPERSTRIKE, G PRO X SUPERLIGHT 1/2, G502 Series, etc.).

Communicates directly with the Linux `/dev/hidraw` device interface via the Logitech **HID++ 2.0** protocol, providing complete telemetry and hardware customization without requiring Logitech G HUB.

## Install & Uninstall

### Install

#### Method A: Git Install (via Omarchy CLI)

```bash
omarchy plugin add https://github.com/ttymayor/oma-logitech-g-mouse.git --enable
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

## Features

### Status Bar Widget

- **Battery and status**: Shows `99% 󰁹`, a charging icon (`󰂅`), or a disconnected state (`Off 󰍽`).
- **Right-click percentage toggle**: Switches between icon-only (`󰁹`) and percentage (`99% 󰁹`) modes. The setting is persisted in `~/.config/omarchy/shell.json`.
- **Theme integration**: Uses the system `foreground` palette and does not change color while its panel is open.

### Control Panel

Left-click opens a `KeyboardPanel` with two tabs:

- **`Sensor (DPI / Polling)`**
  - **DPI presets**: Full-width buttons from the mouse's onboard preset list.
  - **DPI slider**: Adjusts from `100` up to the detected sensor maximum in 50 DPI steps.
  - **Report rate**: Selects from `125` / `250` / `500` / `1K` / `2K` / `4K` / `8K Hz` when supported by HID++ feature `0x8061`.
- **`Buttons (Analog HITS)`** _(shown only for analog-switch mice)_
  - Per-button **Actuation Point** (`1`–`10`), **Rapid Trigger** (`1`–`5`), and **Click Haptics** (`1`–`6`) sliders.
  - Numeric tick labels under each slider.
  - Optimistic state updates prevent slider rebound while hardware writes complete.

## Screenshots

### Sensor tab

![Sensor tab showing battery, DPI, and polling-rate controls](screenshot-1.png)
![Buttons tab showing analog-switch button controls](screenshot-2.png)

---

## Modifying & Applying Changes

- **Modifying QML / UI (`Panel.qml`, `BarWidget.qml`, `Model.js`)**:  
  Omarchy disables QML automatic file watching for system stability, so changes **require a shell restart**:
  ```bash
  cp -a . ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/ && omarchy restart shell
  ```
- **Modifying the native driver (`src/main.rs`)**:  
  Build and package the executable before syncing: `cargo build --release && install -Dm755 target/release/logitech-g-daemon bin/logitech-g-daemon`. Then use the QML/UI sync command above and restart the shell.
- **One-Liner Sync & Reload Command**:
  ```bash
  omarchy plugin validate . && cp -a . ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/ && omarchy restart shell
  ```

---

## Standalone CLI Usage

The bundled native driver can also be run independently from the command line:

```bash
# Print live telemetry in JSON format
./bin/logitech-g-daemon --once

# Set DPI (supports 100 to 32,000 DPI)
./bin/logitech-g-daemon --set-dpi 1600

# Set Report Rate (125 / 250 / 500 / 1000 / 2000 / 4000 / 8000 Hz)
./bin/logitech-g-daemon --set-rate 4000

# Set Actuation Point (Levels 1–10)
./bin/logitech-g-daemon --set-actuation 4 --left
./bin/logitech-g-daemon --set-actuation 6 --right

# Set Rapid Trigger (Levels 1–5)
./bin/logitech-g-daemon --set-rt 2 --left

# Set Click Haptics (Levels 1–6)
./bin/logitech-g-daemon --set-haptics 5 --left
```

---

## HID++ 2.0 Protocol Reference

| Capability               | Feature ID | Index  | Description                                                                  |
| :----------------------- | :--------- | :----- | :--------------------------------------------------------------------------- |
| **Device Name**          | `0x0005`   | `0x03` | Dynamic marketing name query (e.g. `PRO X2 SUPERSTRIKE`, `PRO X SUPERLIGHT`) |
| **Unified Battery**      | `0x1004`   | `0x06` | Charge percentage, battery level, charging/discharging state                 |
| **Extended DPI**         | `0x2202`   | `0x09` | Sensor parameters (supports up to 32K DPI) and LOD status                    |
| **Onboard Profiles**     | `0x8100`   | `0x0e` | Profile switching, Host Mode unlock for custom DPI                           |
| **HITS Analog Switches** | `0x1B0C`   | `0x0c` | Actuation (1–10), Rapid Trigger (1–5), Click Haptics (1–6)                   |
| **Extended Report Rate** | `0x8061`   | `0x0d` | Polling rate control from 125 Hz up to 8000 Hz                               |

---

## Permissions & Udev Rules

> **Note:** On Arch Linux, Omarchy, and most modern systemd-based desktop distributions, **you do NOT need to perform this step!**  
> `systemd-logind` automatically applies `uaccess` dynamic ACLs granting your logged-in desktop user direct read/write permissions (`user:username:rw-`) to `/dev/hidraw*` devices.

### When is this required?

This step is only necessary if:

1. Running `./bin/logitech-g-daemon --once` fails with an `EACCES: Permission denied` error.
2. You are using a minimal distribution without `systemd-logind`, running in a container, or running over a headless session where `uaccess` seat management is not active.

In those cases, create this udev rule to grant world read/write permissions to Logitech HID devices:

```bash
sudo tee /etc/udev/rules.d/42-logitech-mouse.rules << 'EOF'
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="046d", MODE="0666"
EOF

sudo udevadm control --reload && sudo udevadm trigger
```

---

## License

MIT License.  
Logitech, G PRO, and SUPERSTRIKE are trademarks of Logitech. This project is an independent third-party open-source plugin and is not affiliated with Logitech.
