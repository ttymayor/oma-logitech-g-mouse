mod cli;
mod device;
mod hidpp;
mod status;

use std::{process, thread, time::Duration};

use crate::{
    cli::parse_args,
    device::find_ready_device,
    hidpp::{
        features, prefer_host_profile_mode, set_button, set_dpi, set_onboard_mode, set_report_rate,
    },
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
