use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    device::Device,
    hidpp::{
        DPI_PRESETS, Features, battery, button, device_name, dpi, dpi_presets, max_dpi,
        onboard_mode, report_rate,
    },
};

const STATUS_FILE: &str = "status.json";
const MAX_ERROR_BYTES: usize = 512;

fn bounded(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

fn quote(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            character if character.is_control() => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let code = character as u32;
                escaped.push_str("\\u");
                escaped.push(HEX[((code >> 12) & 0xf) as usize] as char);
                escaped.push(HEX[((code >> 8) & 0xf) as usize] as char);
                escaped.push(HEX[((code >> 4) & 0xf) as usize] as char);
                escaped.push(HEX[(code & 0xf) as usize] as char);
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

fn timestamp() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub(crate) fn status(device: &mut Device, features: Features) -> String {
    let mut errors = Vec::new();
    let name = if features.name > 0 {
        match device_name(device, features.name) {
            Ok(value) if !value.is_empty() => value,
            Ok(_) => "Logitech G Mouse".into(),
            Err(error) => {
                errors.push(format!("name: {error}"));
                "Logitech G Mouse".into()
            }
        }
    } else {
        "Logitech G Mouse".into()
    };
    let battery = if features.battery > 0 {
        match battery(device, features.battery) {
            Ok((percentage, level, state)) => format!(
                "{{\"percentage\":{percentage},\"level\":\"{level}\",\"status\":\"{state}\"}}"
            ),
            Err(error) => {
                errors.push(format!("battery: {error}"));
                "null".into()
            }
        }
    } else {
        "null".into()
    };
    let dpi_value = if features.dpi > 0 {
        match dpi(device, features.dpi) {
            Ok((x, default_x, y, default_y, lod)) => format!(
                "{{\"dpiX\":{x},\"defaultDpiX\":{default_x},\"dpiY\":{y},\"defaultDpiY\":{default_y},\"lod\":\"{lod}\"}}"
            ),
            Err(error) => {
                errors.push(format!("dpi: {error}"));
                "null".into()
            }
        }
    } else {
        "null".into()
    };
    let presets = if features.dpi > 0 {
        dpi_presets(device, features.dpi)
    } else {
        DPI_PRESETS.map(|value| value as u16).to_vec()
    };
    let max = if features.dpi > 0 {
        max_dpi(device, features.dpi)
    } else {
        32000
    };
    let rate = if features.report_rate > 0 {
        report_rate(device, features.report_rate)
    } else {
        1000
    };
    let onboard_profile_mode = if features.profiles > 0 {
        match onboard_mode(device, features.profiles) {
            Ok((code, mode)) => format!("{{\"code\":{code},\"mode\":\"{mode}\"}}"),
            Err(error) => {
                errors.push(format!("onboard profiles: {error}"));
                "null".into()
            }
        }
    } else {
        "null".into()
    };
    let hits = if features.hits > 0 {
        match (
            button(device, features.hits, 0),
            button(device, features.hits, 1),
        ) {
            (Ok(left), Ok(right)) => format!(
                "{{\"left\":{{\"actuation\":{},\"rapidTrigger\":{},\"haptics\":{}}},\"right\":{{\"actuation\":{},\"rapidTrigger\":{},\"haptics\":{}}}}}",
                left.0, left.1, left.2, right.0, right.1, right.2
            ),
            (Err(error), _) | (_, Err(error)) => {
                errors.push(format!("hits: {error}"));
                "null".into()
            }
        }
    } else {
        "null".into()
    };
    let error = if errors.is_empty() {
        "null".into()
    } else {
        quote(bounded(&errors.join("; "), MAX_ERROR_BYTES))
    };
    format!(
        "{{\"connected\":true,\"deviceName\":{},\"battery\":{battery},\"dpi\":{dpi_value},\"dpiMin\":100,\"dpiMax\":{max},\"dpiPresets\":[{}],\"reportRate\":{rate},\"onboardProfileMode\":{onboard_profile_mode},\"hasHits\":{},\"hits\":{hits},\"error\":{error},\"updatedAt\":{}}}",
        quote(&name),
        presets
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(","),
        features.hits > 0,
        timestamp()
    )
}

pub(crate) fn offline(error: &str) -> String {
    format!(
        "{{\"connected\":false,\"deviceName\":\"Logitech G Mouse\",\"battery\":null,\"dpi\":null,\"dpiMin\":100,\"dpiMax\":32000,\"dpiPresets\":[800,1200,1600,2400,3200],\"reportRate\":1000,\"onboardProfileMode\":null,\"hasHits\":false,\"hits\":null,\"error\":{},\"updatedAt\":{}}}",
        quote(bounded(error, MAX_ERROR_BYTES)),
        timestamp()
    )
}

fn status_path() -> Result<PathBuf, String> {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "XDG_RUNTIME_DIR is not set".to_owned())?;
    let directory = runtime_dir.join("omarchy-logitech-g-mouse");
    fs::create_dir_all(&directory).map_err(|error| format!("create state directory: {error}"))?;
    let metadata = fs::symlink_metadata(&directory)
        .map_err(|error| format!("inspect state directory: {error}"))?;
    if !metadata.is_dir() {
        return Err("state path is not a directory".into());
    }
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("protect state directory: {error}"))?;
    Ok(directory.join(STATUS_FILE))
}

fn write_status(value: &str) -> Result<(), String> {
    let path = status_path()?;
    let temporary = path.with_extension(format!("{}-{}.tmp", std::process::id(), timestamp()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(0o400000)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| format!("create state file: {error}"))?;
    file.write_all(value.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("write state file: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("publish state file: {error}"))
}

pub(crate) fn emit(value: String) {
    if let Err(error) = write_status(&value) {
        eprintln!("state write failed: {error}");
    }
    println!("{value}");
}

#[cfg(test)]
mod tests {
    use super::{bounded, quote};

    #[test]
    fn quotes_control_characters_as_json() {
        assert_eq!(
            quote("a\r\n\t\u{0001}\"\\b"),
            "\"a\\r\\n\\t\\u0001\\\"\\\\b\""
        );
    }

    #[test]
    fn bounds_at_utf8_boundary() {
        assert_eq!(bounded("éclair", 1), "");
        assert_eq!(bounded("éclair", 2), "é");
    }
}
