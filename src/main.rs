use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const VID: u32 = 0x046d;
const REPORT_LONG: u8 = 0x11;
const LONG_LEN: usize = 20;
const SWID: u8 = 0x07;
const STATUS_FILE: &str = "status.json";
const MAX_DEVICE_NAME_BYTES: usize = 128;
const MAX_ERROR_BYTES: usize = 512;
const DPI_PRESETS: [u32; 5] = [800, 1200, 1600, 2400, 3200];

#[derive(Clone, Copy)]
struct Features {
    name: u8,
    battery: u8,
    dpi: u8,
    hits: u8,
    profiles: u8,
    report_rate: u8,
}
const DEFAULT_FEATURES: Features = Features {
    name: 0x03,
    battery: 0x06,
    dpi: 0x09,
    hits: 0x0c,
    profiles: 0x0e,
    report_rate: 0x0d,
};

struct Device {
    file: File,
    path: String,
    index: u8,
}

impl Device {
    fn open(path: String) -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(0o4000)
            .open(&path)
            .map_err(|e| format!("open {path}: {e}"))?;
        Ok(Self {
            file,
            path,
            index: 1,
        })
    }

    fn drain(&mut self) {
        let mut buf = [0u8; 64];
        for _ in 0..32 {
            match self.file.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
    }

    fn request(
        &mut self,
        feature: u8,
        function: u8,
        params: &[u8],
        tries: u8,
        timeout: Duration,
    ) -> Result<Vec<u8>, String> {
        let mut packet = [0u8; LONG_LEN];
        packet[0] = REPORT_LONG;
        packet[1] = self.index;
        packet[2] = feature;
        packet[3] = (function << 4) | SWID;
        for (i, value) in params.iter().take(LONG_LEN - 4).enumerate() {
            packet[4 + i] = *value;
        }
        let mut last_error = None;
        let mut response = [0u8; 64];
        for _ in 0..tries {
            self.drain();
            self.file
                .write_all(&packet)
                .map_err(|e| format!("write {}: {e}", self.path))?;
            let deadline = Instant::now() + timeout;
            while Instant::now() < deadline {
                match self.file.read(&mut response) {
                    Ok(n) if n >= 4 => {
                        if response[2] == 0x8f || response[2] == 0xff {
                            last_error = response.get(5).copied();
                            break;
                        }
                        if response[1] == self.index
                            && response[2] == feature
                            && response[3] >> 4 == function
                        {
                            return Ok(response[4..n].to_vec());
                        }
                    }
                    Ok(_) | Err(_) => {}
                }
                thread::sleep(Duration::from_millis(4));
            }
            thread::sleep(Duration::from_millis(15));
        }
        Err(format!(
            "no response feat=0x{feature:02x} fn={function} (last error={last_error:?})"
        ))
    }

    fn ping(&mut self, index: u8) -> bool {
        let previous = self.index;
        self.index = index;
        if self
            .request(0, 1, &[0, 0, 0x5a], 2, Duration::from_millis(150))
            .is_ok()
        {
            true
        } else {
            self.index = previous;
            false
        }
    }

    fn feature(&mut self, id: u16, fallback: u8) -> u8 {
        self.request(
            0,
            0,
            &[(id >> 8) as u8, id as u8],
            2,
            Duration::from_millis(150),
        )
        .ok()
        .and_then(|body| body.first().copied())
        .filter(|index| *index > 0)
        .unwrap_or(fallback)
    }
}

fn is_logitech_hidraw(name: &str) -> bool {
    let Ok(text) = fs::read_to_string(format!("/sys/class/hidraw/{name}/device/uevent")) else {
        return false;
    };
    text.lines()
        .find_map(|line| line.strip_prefix("HID_ID="))
        .and_then(|id| id.split(':').nth(1))
        .and_then(|vid| u32::from_str_radix(vid, 16).ok())
        == Some(VID)
}

fn has_hidpp_usage(name: &str) -> bool {
    fs::read(format!("/sys/class/hidraw/{name}/device/report_descriptor"))
        .ok()
        .is_some_and(|bytes| bytes.windows(3).any(|chunk| chunk == [0x06, 0x00, 0xff]))
}

fn find_device() -> Result<Device, String> {
    let mut names: Vec<String> = fs::read_dir("/dev")
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("hidraw") && is_logitech_hidraw(name))
        .collect();
    names.sort();
    names.sort_by_key(|name| !has_hidpp_usage(name));
    for name in names {
        let mut device = match Device::open(format!("/dev/{name}")) {
            Ok(device) => device,
            Err(_) => continue,
        };
        if [1, 2, 3, 4, 5, 6, 0xff]
            .into_iter()
            .any(|index| device.ping(index))
        {
            return Ok(device);
        }
    }
    Err("No Logitech HID++ device found.".into())
}

fn find_ready_device() -> Result<Device, String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let last_error = loop {
        match find_device() {
            Ok(device) => return Ok(device),
            Err(error) if Instant::now() >= deadline => break error,
            Err(_) => thread::sleep(Duration::from_millis(200)),
        }
    };
    Err(format!(
        "Logitech receiver did not become ready within 5000ms: {last_error}"
    ))
}
fn features(device: &mut Device) -> Features {
    Features {
        name: device.feature(0x0005, DEFAULT_FEATURES.name),
        battery: device.feature(0x1004, DEFAULT_FEATURES.battery),
        dpi: device.feature(0x2202, DEFAULT_FEATURES.dpi),
        hits: device.feature(0x1b0c, DEFAULT_FEATURES.hits),
        profiles: device.feature(0x8100, DEFAULT_FEATURES.profiles),
        report_rate: device.feature(0x8061, DEFAULT_FEATURES.report_rate),
    }
}

fn device_name(device: &mut Device, feature: u8) -> Result<String, String> {
    let length = (*device
        .request(feature, 0, &[], 3, Duration::from_millis(400))?
        .first()
        .unwrap_or(&0) as usize)
        .min(MAX_DEVICE_NAME_BYTES);
    let mut bytes = Vec::with_capacity(length);
    for offset in (0..length).step_by(16) {
        let chunk = device.request(feature, 1, &[offset as u8], 3, Duration::from_millis(400))?;
        bytes.extend_from_slice(&chunk[..chunk.len().min(length - offset)]);
    }
    Ok(String::from_utf8_lossy(&bytes)
        .split('\0')
        .next()
        .unwrap_or("")
        .trim()
        .to_owned())
}

fn battery(device: &mut Device, feature: u8) -> Result<(u8, &'static str, &'static str), String> {
    let body = device.request(feature, 1, &[0, 0, 0], 3, Duration::from_millis(400))?;
    let level = match body.get(1) {
        Some(1) => "critical",
        Some(2) => "low",
        Some(4) => "good",
        Some(8) => "full",
        _ => "unknown",
    };
    let status = match body.get(2) {
        Some(0) => "discharging",
        Some(1) => "charging",
        Some(2) => "charging_slow",
        Some(3) => "full",
        Some(4) => "error",
        _ => "unknown",
    };
    Ok((*body.first().unwrap_or(&0), level, status))
}

fn dpi(device: &mut Device, feature: u8) -> Result<(u16, u16, u16, u16, &'static str), String> {
    let body = device.request(feature, 5, &[0, 0, 0], 3, Duration::from_millis(400))?;
    let word = |index| {
        u16::from_be_bytes([
            *body.get(index).unwrap_or(&0),
            *body.get(index + 1).unwrap_or(&0),
        ])
    };
    let lod = match body.get(9) {
        Some(0) => "unsupported",
        Some(1) => "low",
        Some(2) => "medium",
        Some(3) => "high",
        _ => "unknown",
    };
    Ok((word(1), word(3), word(5), word(7), lod))
}

fn dpi_presets(device: &mut Device, feature: u8) -> Vec<u16> {
    let Ok(body) = device.request(feature, 3, &[0, 0, 0], 3, Duration::from_millis(400)) else {
        return DPI_PRESETS.map(|value| value as u16).to_vec();
    };
    let values: Vec<u16> = (0..14)
        .step_by(2)
        .map(|index| {
            u16::from_be_bytes([
                *body.get(index).unwrap_or(&0),
                *body.get(index + 1).unwrap_or(&0),
            ])
        })
        .filter(|value| *value > 0)
        .collect();
    if values.is_empty() {
        DPI_PRESETS.map(|value| value as u16).to_vec()
    } else {
        values
    }
}

fn max_dpi(device: &mut Device, feature: u8) -> u32 {
    let Ok(body) = device.request(feature, 2, &[0, 0, 2], 3, Duration::from_millis(400)) else {
        return 32000;
    };
    (0..14)
        .step_by(2)
        .map(|index| {
            u16::from_be_bytes([
                *body.get(index).unwrap_or(&0),
                *body.get(index + 1).unwrap_or(&0),
            ]) as u32
        })
        .filter(|value| *value > 1000 && *value <= 44000 && (*value % 1000 == 0 || *value == 25600))
        .max()
        .unwrap_or(32000)
        .max(32000)
}

fn report_rate(device: &mut Device, feature: u8) -> u32 {
    for mode in [1, 0] {
        if let Ok(body) = device.request(feature, 2, &[mode], 3, Duration::from_millis(400)) {
            return match body.first() {
                Some(0) => 125,
                Some(1) => 250,
                Some(2) => 500,
                Some(3) => 1000,
                Some(4) => 2000,
                Some(5) => 4000,
                Some(6) => 8000,
                _ => 1000,
            };
        }
    }
    1000
}

fn button(device: &mut Device, feature: u8, index: u8) -> Result<(u8, u8, u8), String> {
    let body = device.request(feature, 2, &[index], 3, Duration::from_millis(400))?;
    Ok((
        body.get(1).unwrap_or(&0) / 4,
        body.get(2).unwrap_or(&0) / 4,
        body.get(3).unwrap_or(&0) / 4,
    ))
}

fn set_button(
    device: &mut Device,
    feature: u8,
    index: u8,
    actuation: Option<u8>,
    rapid_trigger: Option<u8>,
    haptics: Option<u8>,
) -> Result<(), String> {
    let current = device.request(feature, 2, &[index], 3, Duration::from_millis(400))?;
    let values = [
        index,
        actuation.map_or(*current.get(1).unwrap_or(&0), |value| value * 4),
        rapid_trigger.map_or(*current.get(2).unwrap_or(&0), |value| value * 4),
        haptics.map_or(*current.get(3).unwrap_or(&0), |value| value * 4),
    ];
    device
        .request(feature, 1, &values, 3, Duration::from_millis(400))
        .map(|_| ())
}

fn set_dpi(device: &mut Device, feature: u8, target: u32) -> Result<(), String> {
    let value = target.clamp(100, 32000) / 50 * 50;
    let [high, low] = (value as u16).to_be_bytes();
    device
        .request(
            feature,
            6,
            &[0, high, low, high, low, 2],
            3,
            Duration::from_millis(400),
        )
        .map(|_| ())
}

fn set_report_rate(device: &mut Device, feature: u8, rate: u32) {
    let code = match rate {
        125 => 0,
        250 => 1,
        500 => 2,
        1000 => 3,
        2000 => 4,
        4000 => 5,
        8000 => 6,
        _ => 3,
    };
    for mode in [0, 1] {
        let _ = device.request(feature, 3, &[code, mode, 0], 3, Duration::from_millis(400));
    }
}

fn onboard_mode(device: &mut Device, feature: u8) -> Result<(u8, &'static str), String> {
    let mode = *device
        .request(feature, 2, &[], 3, Duration::from_millis(400))?
        .first()
        .ok_or_else(|| "empty onboard-profile mode response".to_owned())?;
    let label = match mode {
        1 => "onboard",
        2 => "host",
        _ => "unknown",
    };
    Ok((mode, label))
}

fn set_onboard_mode(device: &mut Device, feature: u8, mode: u8) -> Result<(), String> {
    if !matches!(mode, 1 | 2) {
        return Err(format!("invalid onboard-profile mode: {mode}"));
    }
    device
        .request(feature, 1, &[mode], 3, Duration::from_millis(400))
        .map(|_| ())
}

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

fn status(device: &mut Device, features: Features) -> String {
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

fn offline(error: &str) -> String {
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

fn emit(value: String) {
    if let Err(error) = write_status(&value) {
        eprintln!("state write failed: {error}");
    }
    println!("{value}");
}

#[derive(Default)]
struct Args {
    interval: u64,
    once: bool,
    actuation: Option<u8>,
    rapid_trigger: Option<u8>,
    haptics: Option<u8>,
    dpi: Option<u32>,
    rate: Option<u32>,
    profile_mode: Option<u8>,
    left: bool,
    right: bool,
}
fn parse_args() -> Args {
    let mut args = Args {
        interval: 15,
        ..Args::default()
    };
    let values: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < values.len() {
        let next = || {
            values
                .get(i + 1)
                .and_then(|value| value.parse::<u64>().ok())
        };
        match values[i].as_str() {
            "--interval" => {
                if let Some(value) = next() {
                    args.interval = value;
                    i += 1;
                }
            }
            "--once" => args.once = true,
            "--set-actuation" => {
                if let Some(value) = next() {
                    args.actuation = Some(value.clamp(1, 10) as u8);
                    i += 1;
                }
            }
            "--set-rt" | "--set-rapid-trigger" => {
                if let Some(value) = next() {
                    args.rapid_trigger = Some(value.clamp(1, 5) as u8);
                    i += 1;
                }
            }
            "--set-haptics" => {
                if let Some(value) = next() {
                    args.haptics = Some(value.clamp(0, 5) as u8);
                    i += 1;
                }
            }
            "--set-dpi" => {
                if let Some(value) = next().and_then(|value| u32::try_from(value).ok()) {
                    args.dpi = Some(value);
                    i += 1;
                }
            }
            "--set-rate" | "--set-report-rate" => {
                if let Some(value) = next().and_then(|value| u32::try_from(value).ok()) {
                    args.rate = Some(value);
                    i += 1;
                }
            }
            "--set-profile-mode" => {
                if let Some(mode) = values.get(i + 1).and_then(|value| match value.as_str() {
                    "onboard" => Some(1),
                    "host" => Some(2),
                    _ => None,
                }) {
                    args.profile_mode = Some(mode);
                    i += 1;
                }
            }
            "--left" => args.left = true,
            "--right" => args.right = true,
            _ => {}
        }
        i += 1;
    }
    args
}

fn main() {
    let args = parse_args();
    let mut device = match find_ready_device() {
        Ok(device) => device,
        Err(error) => {
            emit(offline(&error));
            std::process::exit(1);
        }
    };
    let features = features(&mut device);
    let write_requested = args.actuation.is_some()
        || args.rapid_trigger.is_some()
        || args.haptics.is_some()
        || args.dpi.is_some()
        || args.rate.is_some()
        || args.profile_mode.is_some();
    let result = (|| -> Result<(), String> {
        if args.actuation.is_some() || args.rapid_trigger.is_some() || args.haptics.is_some() {
            for index in if args.left {
                vec![0]
            } else if args.right {
                vec![1]
            } else {
                vec![0, 1]
            } {
                set_button(
                    &mut device,
                    features.hits,
                    index,
                    args.actuation,
                    args.rapid_trigger,
                    args.haptics,
                )?;
            }
        }
        if let Some(value) = args.dpi {
            set_dpi(&mut device, features.dpi, value)?;
        }
        if let Some(value) = args.rate {
            set_report_rate(&mut device, features.report_rate, value);
        }
        if let Some(mode) = args.profile_mode {
            set_onboard_mode(&mut device, features.profiles, mode)?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        emit(offline(&error));
        std::process::exit(1);
    }
    loop {
        emit(status(&mut device, features));
        if args.once || write_requested {
            break;
        }
        thread::sleep(Duration::from_secs(args.interval.max(1)));
    }
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
