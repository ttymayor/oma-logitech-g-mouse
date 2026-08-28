use std::time::Duration;

use crate::device::Device;

pub(crate) const DPI_PRESETS: [u32; 5] = [800, 1200, 1600, 2400, 3200];

#[derive(Clone, Copy)]
pub(crate) struct Features {
    pub(crate) name: u8,
    pub(crate) battery: u8,
    pub(crate) dpi: u8,
    pub(crate) hits: Option<u8>,
    pub(crate) profiles: u8,
    pub(crate) report_rate: u8,
}

const DEFAULT_FEATURES: Features = Features {
    name: 0x03,
    battery: 0x06,
    dpi: 0x09,
    hits: None,
    profiles: 0x0e,
    report_rate: 0x0d,
};

pub(crate) fn features(device: &mut Device) -> Features {
    Features {
        name: device.feature(0x0005, DEFAULT_FEATURES.name),
        battery: device.feature(0x1004, DEFAULT_FEATURES.battery),
        dpi: device.feature(0x2202, DEFAULT_FEATURES.dpi),
        hits: device.feature_optional(0x1b0c),
        profiles: device.feature(0x8100, DEFAULT_FEATURES.profiles),
        report_rate: device.feature(0x8061, DEFAULT_FEATURES.report_rate),
    }
}

pub(crate) fn device_name(device: &mut Device, feature: u8) -> Result<String, String> {
    let length = (*device
        .request(feature, 0, &[], 3, Duration::from_millis(400))?
        .first()
        .unwrap_or(&0) as usize)
        .min(128);
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

pub(crate) fn battery(
    device: &mut Device,
    feature: u8,
) -> Result<(u8, &'static str, &'static str), String> {
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

pub(crate) fn dpi(
    device: &mut Device,
    feature: u8,
) -> Result<(u16, u16, u16, u16, &'static str), String> {
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

pub(crate) fn dpi_presets(device: &mut Device, feature: u8) -> Vec<u16> {
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

pub(crate) fn max_dpi(device: &mut Device, feature: u8) -> u32 {
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

pub(crate) fn report_rate(device: &mut Device, feature: u8) -> u32 {
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

pub(crate) fn button(device: &mut Device, feature: u8, index: u8) -> Result<(u8, u8, u8), String> {
    let body = device.request(feature, 2, &[index], 3, Duration::from_millis(400))?;
    Ok((
        body.get(1).unwrap_or(&0) / 4,
        body.get(2).unwrap_or(&0) / 4,
        body.get(3).unwrap_or(&0) / 4,
    ))
}

pub(crate) fn set_button(
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

pub(crate) fn set_dpi(device: &mut Device, feature: u8, target: u32) -> Result<(), String> {
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

pub(crate) fn set_report_rate(device: &mut Device, feature: u8, rate: u32) {
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

pub(crate) fn onboard_mode(device: &mut Device, feature: u8) -> Result<(u8, &'static str), String> {
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

pub(crate) fn prefer_host_profile_mode(device: &mut Device, feature: u8) -> Result<(), String> {
    if onboard_mode(device, feature)?.0 == 1 {
        set_onboard_mode(device, feature, 2)?;
    }
    Ok(())
}

pub(crate) fn set_onboard_mode(device: &mut Device, feature: u8, mode: u8) -> Result<(), String> {
    if !matches!(mode, 1 | 2) {
        return Err(format!("invalid onboard-profile mode: {mode}"));
    }
    device
        .request(feature, 1, &[mode], 3, Duration::from_millis(400))
        .map(|_| ())
}
