# Development

This guide is for contributors. End-user installation and troubleshooting are in [README.md](README.md).

## Prerequisites

- Rust stable toolchain.
- Omarchy Quattro for QML/plugin validation.
- A supported Logitech HID++ mouse is required for hardware write verification.

## Project layout

| Path                            | Purpose                                                     |
| ------------------------------- | ----------------------------------------------------------- |
| `src/main.rs`                   | Native HID++ controller.                                    |
| `bin/logitech-g-daemon`         | x86_64 Linux executable shipped with the plugin.            |
| `Service.qml`                   | Starts the controller and applies its JSON status.          |
| `Model.js`                      | Converts controller JSON into QML state.                    |
| `manifest.json`                 | Marketplace and plugin metadata; canonical release version. |
| `.github/workflows/release.yml` | Creates GitHub version tags and releases.                   |

## Native controller

The controller communicates directly with `/dev/hidraw` using HID++ 2.0. It dynamically resolves feature indices after connection and waits up to five seconds for a receiver to become ready.

Build and update the executable shipped in the plugin:

```bash
cargo fmt
cargo build --locked --release
install -Dm755 target/release/logitech-g-daemon bin/logitech-g-daemon
```

The bundled executable targets x86_64 GNU/Linux. Build it on the target architecture when supporting another platform.

### CLI contract

```bash
./bin/logitech-g-daemon --once
./bin/logitech-g-daemon --set-dpi 1600
./bin/logitech-g-daemon --set-rate 4000
./bin/logitech-g-daemon --set-actuation 4 --left
./bin/logitech-g-daemon --set-rt 2 --left
./bin/logitech-g-daemon --set-haptics 5 --left
```

Each invocation prints one JSON status object and updates `/tmp/omarchy-logitech-g-mouse.json`. `Service.qml` depends on this output contract.

### HID++ features

| Capability           | Feature ID | Fallback index |
| -------------------- | ---------- | -------------- |
| Device name          | `0x0005`   | `0x03`         |
| Unified battery      | `0x1004`   | `0x06`         |
| Extended DPI         | `0x2202`   | `0x09`         |
| HITS analog switches | `0x1B0C`   | `0x0C`         |
| Onboard profiles     | `0x8100`   | `0x0E`         |
| Extended report rate | `0x8061`   | `0x0D`         |

Fallback indices support hardware that temporarily rejects feature discovery; discovered indices remain authoritative.

## Local development

After QML or bundled-binary changes, copy the plugin and restart the shell:

```bash
omarchy plugin validate .
cp -a . ~/.config/omarchy/plugins/tantuyu.logitech-g-mouse/
omarchy restart shell
```

Verify the native controller before testing the QML surface:

```bash
cargo build --locked --release
./bin/logitech-g-daemon --once
```

Use a no-op write matching the current DPI to exercise a hardware write without changing the configured value:

```bash
./bin/logitech-g-daemon --set-dpi 1200
```

## Releases

`manifest.json` is the canonical plugin version. On a push to `main`, `.github/workflows/release.yml` creates a GitHub Release and `v<version>` tag when that semantic version does not already have a tag. It publishes release notes only; it does not attach a binary.

To release:

1. Update `manifest.json` to the new semantic version.
2. Build and update `bin/logitech-g-daemon` if the native controller changed.
3. Run the verification commands above.
4. Commit and push to `main`.

Existing release tags are never overwritten.
