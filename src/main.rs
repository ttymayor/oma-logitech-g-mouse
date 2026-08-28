mod cli;
mod device;
mod hidpp;
mod profile;

mod status;

use std::{process, thread, time::Duration};

use crate::{
    cli::parse_args,
    device::find_ready_device,
    hidpp::{features, prefer_host_profile_mode, set_onboard_mode},
    profile::{load as load_profile, save as save_profile},
    status::{emit, offline, status},
};

fn main() {
    let args = parse_args();
    let mut device = match find_ready_device() {
        Ok(device) => device,
        Err(error) => {
            emit(offline(&error));
            process::exit(1);
        }
    };
    let features = features(&mut device);
    if args.profile_mode.is_none() && features.profiles > 0 {
        if let Err(error) = prefer_host_profile_mode(&mut device, features.profiles) {
            eprintln!("default profile source failed: {error}");
        }
    }

    let settings_requested = args.actuation.is_some()
        || args.rapid_trigger.is_some()
        || args.haptics.is_some()
        || args.dpi.is_some()
        || args.rate.is_some();
    let write_requested = settings_requested || args.profile_mode.is_some();
    let result = (|| -> Result<(), String> {
        let mut profile = load_profile()?;
        if settings_requested {
            profile.update(&args);
            save_profile(profile)?;
        } else if profile.is_empty() {
            profile = profile::Profile::capture(&mut device, features)?;
            save_profile(profile)?;
        }
        if args.profile_mode.is_none() {
            profile.apply(&mut device, features)?;
        }
        if let Some(mode) = args.profile_mode {
            set_onboard_mode(&mut device, features.profiles, mode)?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        emit(offline(&error));
        process::exit(1);
    }
    loop {
        emit(status(&mut device, features));
        if args.once || write_requested {
            break;
        }
        thread::sleep(Duration::from_secs(args.interval.max(1)));
    }
}
