use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    thread,
    time::{Duration, Instant},
};

const VID: u32 = 0x046d;
const REPORT_LONG: u8 = 0x11;
const LONG_LEN: usize = 20;
const SWID: u8 = 0x07;

pub(crate) struct Device {
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

    pub(crate) fn request(
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

    pub(crate) fn feature_optional(&mut self, id: u16) -> Option<u8> {
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
    }

    pub(crate) fn feature(&mut self, id: u16, fallback: u8) -> u8 {
        self.feature_optional(id).unwrap_or(fallback)
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

pub(crate) fn find_ready_device() -> Result<Device, String> {
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
