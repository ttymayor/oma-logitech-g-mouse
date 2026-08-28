use std::env;

#[derive(Default)]
pub(crate) struct Args {
    pub(crate) interval: u64,
    pub(crate) once: bool,
    pub(crate) actuation: Option<u8>,
    pub(crate) rapid_trigger: Option<u8>,
    pub(crate) haptics: Option<u8>,
    pub(crate) dpi: Option<u32>,
    pub(crate) rate: Option<u32>,
    pub(crate) profile_mode: Option<u8>,
    pub(crate) left: bool,
    pub(crate) right: bool,
}

pub(crate) fn parse_args() -> Args {
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
