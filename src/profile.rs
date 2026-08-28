use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::PathBuf,
};

use crate::{
    cli::Args,
    device::Device,
    hidpp::{Features, button, dpi, report_rate, set_button, set_dpi, set_report_rate},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ButtonProfile {
    actuation: Option<u8>,
    rapid_trigger: Option<u8>,
    haptics: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Profile {
    dpi: Option<u32>,
    rate: Option<u32>,
    left: ButtonProfile,
    right: ButtonProfile,
}

impl Profile {
    pub(crate) fn update(&mut self, args: &Args) {
        self.dpi = args.dpi.or(self.dpi);
        self.rate = args.rate.or(self.rate);

        if args.left {
            update_button(&mut self.left, args);
        } else if args.right {
            update_button(&mut self.right, args);
        } else {
            update_button(&mut self.left, args);
            update_button(&mut self.right, args);
        }
    }

    pub(crate) fn is_empty(self) -> bool {
        self == Self::default()
    }

    pub(crate) fn capture(device: &mut Device, features: Features) -> Result<Self, String> {
        let (dpi, _, _, _, _) = dpi(device, features.dpi)?;
        let mut profile = Self {
            dpi: Some(u32::from(dpi)),
            rate: Some(report_rate(device, features.report_rate)),
            ..Self::default()
        };
        if let Some(feature) = features.hits {
            let (left_actuation, left_rapid_trigger, left_haptics) = button(device, feature, 0)?;
            let (right_actuation, right_rapid_trigger, right_haptics) = button(device, feature, 1)?;
            profile.left = ButtonProfile {
                actuation: Some(left_actuation),
                rapid_trigger: Some(left_rapid_trigger),
                haptics: Some(left_haptics),
            };
            profile.right = ButtonProfile {
                actuation: Some(right_actuation),
                rapid_trigger: Some(right_rapid_trigger),
                haptics: Some(right_haptics),
            };
        }
        Ok(profile)
    }

    pub(crate) fn apply(&self, device: &mut Device, features: Features) -> Result<(), String> {
        if let Some(dpi) = self.dpi {
            set_dpi(device, features.dpi, dpi)?;
        }
        if let Some(rate) = self.rate {
            set_report_rate(device, features.report_rate, rate);
        }
        if let Some(feature) = features.hits {
            apply_button(device, feature, 0, self.left)?;
            apply_button(device, feature, 1, self.right)?;
        }
        Ok(())
    }
}

fn update_button(button: &mut ButtonProfile, args: &Args) {
    button.actuation = args.actuation.or(button.actuation);
    button.rapid_trigger = args.rapid_trigger.or(button.rapid_trigger);
    button.haptics = args.haptics.or(button.haptics);
}

fn apply_button(
    device: &mut Device,
    feature: u8,
    index: u8,
    profile: ButtonProfile,
) -> Result<(), String> {
    if profile.actuation.is_some() || profile.rapid_trigger.is_some() || profile.haptics.is_some() {
        set_button(
            device,
            feature,
            index,
            profile.actuation,
            profile.rapid_trigger,
            profile.haptics,
        )?;
    }
    Ok(())
}

pub(crate) fn load() -> Result<Profile, String> {
    let path = profile_path()?;
    let mut text = String::new();
    match File::open(path) {
        Ok(mut file) => {
            file.read_to_string(&mut text)
                .map_err(|error| format!("read profile: {error}"))?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Profile::default()),
        Err(error) => return Err(format!("open profile: {error}")),
    }
    parse(&text)
}

pub(crate) fn save(profile: Profile) -> Result<(), String> {
    let path = profile_path()?;
    let parent = path.parent().expect("profile path has a parent");
    fs::create_dir_all(parent).map_err(|error| format!("create profile directory: {error}"))?;
    let metadata = fs::symlink_metadata(parent)
        .map_err(|error| format!("inspect profile directory: {error}"))?;
    if !metadata.is_dir() {
        return Err("profile path is not a directory".to_owned());
    }
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
        .map_err(|error| format!("secure profile directory: {error}"))?;

    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .custom_flags(0o400000)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| format!("create profile: {error}"))?;
    file.write_all(serialize(profile).as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("write profile: {error}"))?;
    fs::rename(&temporary, &path).map_err(|error| format!("install profile: {error}"))?;
    Ok(())
}

fn profile_path() -> Result<PathBuf, String> {
    let state_home = match env::var_os("XDG_STATE_HOME") {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        _ => PathBuf::from(env::var_os("HOME").ok_or("HOME is not set")?).join(".local/state"),
    };
    Ok(state_home.join("omarchy/logitech-g-mouse/profile"))
}

fn parse(text: &str) -> Result<Profile, String> {
    let mut profile = Profile::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            return Err("invalid profile format".to_owned());
        };
        let value = value
            .parse::<u32>()
            .map_err(|_| format!("invalid profile value for {key}"))?;
        match key {
            "dpi" => profile.dpi = Some(value.clamp(100, 32000) / 50 * 50),
            "rate" => profile.rate = Some(value),
            "left.actuation" => profile.left.actuation = Some(value.clamp(1, 10) as u8),
            "left.rapid_trigger" => profile.left.rapid_trigger = Some(value.clamp(1, 5) as u8),
            "left.haptics" => profile.left.haptics = Some(value.clamp(0, 5) as u8),
            "right.actuation" => profile.right.actuation = Some(value.clamp(1, 10) as u8),
            "right.rapid_trigger" => profile.right.rapid_trigger = Some(value.clamp(1, 5) as u8),
            "right.haptics" => profile.right.haptics = Some(value.clamp(0, 5) as u8),
            _ => return Err(format!("unknown profile key: {key}")),
        }
    }
    Ok(profile)
}

fn serialize(profile: Profile) -> String {
    let mut text = String::new();
    for (key, value) in [
        ("dpi", profile.dpi),
        ("rate", profile.rate),
        ("left.actuation", profile.left.actuation.map(u32::from)),
        (
            "left.rapid_trigger",
            profile.left.rapid_trigger.map(u32::from),
        ),
        ("left.haptics", profile.left.haptics.map(u32::from)),
        ("right.actuation", profile.right.actuation.map(u32::from)),
        (
            "right.rapid_trigger",
            profile.right.rapid_trigger.map(u32::from),
        ),
        ("right.haptics", profile.right.haptics.map(u32::from)),
    ] {
        if let Some(value) = value {
            text.push_str(key);
            text.push('=');
            text.push_str(&value.to_string());
            text.push('\n');
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_normalizes_profile_values() {
        let profile = parse("dpi=1337\nleft.actuation=12\nright.haptics=8\n").unwrap();
        assert_eq!(
            serialize(profile),
            "dpi=1300\nleft.actuation=10\nright.haptics=5\n"
        );
    }

    #[test]
    fn updates_only_the_requested_button() {
        let mut profile = parse("left.actuation=4\nright.actuation=6\n").unwrap();
        profile.update(&Args {
            actuation: Some(9),
            left: true,
            ..Args::default()
        });
        assert_eq!(serialize(profile), "left.actuation=9\nright.actuation=6\n");
    }
    #[test]
    fn rejects_unknown_or_malformed_profile_entries() {
        assert!(parse("mode=host\n").is_err());
        assert!(parse("dpi\n").is_err());
    }
}
