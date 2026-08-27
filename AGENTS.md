# AGENTS.md

This project is an [Omarchy Quattro](https://omarchy.org/) (Omarchy 4.0+) plugin that controls a **Logitech G** gaming mouse directly over the Linux `/dev/hidraw` interface using the **HID++ 2.0** protocol. The backend is written in **Rust** and the user interface in **QML**.

## Goal

Provide live telemetry and hardware customization for a Logitech G mouse without requiring Logitech G HUB.

Features include:

- Read and write G mouse sensitivity (DPI), report rate.
- Read and write left and right button analog hits settings.
- Report battery / connection status to the Omarchy shell.

## How it fits together

- `src/main.rs` is a native daemon that talks to the mouse over HID++, exposing a CLI and writing JSON state to `/tmp`.
- `Service.qml` starts that daemon and watches its JSON status file to keep the UI in sync.
- `Model.js` and the QML views render that state as a bar widget and control panel.

## Project layout

| Path                            | Purpose                                                     |
| ------------------------------- | ----------------------------------------------------------- |
| `src/main.rs`                   | Native HID++ controller / daemon.                           |
| `bin/logitech-g-daemon`         | x86_64 Linux executable shipped with the plugin.            |
| `Cargo.toml`, `Cargo.lock`      | Rust build config and locked dependency versions.           |
| `Service.qml`                   | Starts the controller and applies its JSON status.          |
| `Model.js`                      | Converts controller JSON into QML state.                    |
| `manifest.json`                 | Marketplace and plugin metadata; canonical release version. |
| `.github/workflows/release.yml` | Creates GitHub version tags and releases.                   |

## Building

The shipped `bin/logitech-g-daemon` is built from `src/main.rs`. Rebuild and install it with:

```bash
cargo build --release && install -Dm755 target/release/logitech-g-daemon bin/logitech-g-daemon
```

`target/` is build output and is gitignored; keep the built binary in `bin/` in sync when changing the driver.
