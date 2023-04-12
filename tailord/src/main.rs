mod dbus;
mod fancontrol;
pub mod led;
mod performance;
mod profiles;
pub mod shutdown;
mod suspend;
pub mod util;

use std::future::pending;

use dbus::{FanInterface, PerformanceInterface, ProfileInterface};
use profiles::Profile;
use tuxedo_ioctl::hal::IoInterface;
use zbus::ConnectionBuilder;

use crate::{
    dbus::LedInterface,
    fancontrol::FanRuntime,
    led::{LedRuntime, LedRuntimeData},
    performance::PerformanceProfileRuntime,
};

const DBUS_NAME: &str = "com.tux.Tailor";
const DBUS_PATH: &str = "/com/tux/Tailor";

fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .without_time()
        .init();

    tokio_uring::start(start_runtime());
}

#[tracing::instrument]
async fn start_runtime() {
    tracing::info!("Starting tailord");

    // Setup shutdown
    let mut shutdown_receiver = shutdown::setup();

    let profile = Profile::load();

    let (device, _webcam, _tdp) = match IoInterface::new() {
        Ok(interface) => {
            let IoInterface {
                device,
                webcam,
                tdp,
                module_version,
            } = interface;
            tracing::info!("Connected to Tuxedo ioctl interface with version {module_version}");
            (Some(device), webcam, tdp)
        }
        Err(err) => {
            tracing::warn!("No tuxedo ioctl interface available: {err}");
            (None, None, None)
        }
    };

    let mut fan_handles = Vec::new();
    let mut fan_runtimes = Vec::new();
    if let Some(device) = &device {
        let available_fans = device.get_number_fans();
        for fan_idx in 0..available_fans {
            let profile = profile
                .fans
                .get(fan_idx as usize)
                .cloned()
                .unwrap_or_default();
            let (handle, runtime) = FanRuntime::new(fan_idx, device.clone(), profile);

            fan_handles.push(handle);
            fan_runtimes.push(runtime);
        }
    }

    let led_devices = tuxedo_sysfs::led::Collection::new()
        .await
        .map(|c| c.into_inner())
        .unwrap_or_default();

    let mut led_handles = Vec::new();
    let mut led_runtimes = Vec::new();
    for led_device in led_devices {
        let profile = profile
            .leds
            .iter()
            .find_map(|(info, profile)| {
                if info.device_name == led_device.device_name
                    && info.function == led_device.function
                {
                    Some(profile.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let (handle, runtime) = LedRuntime::new(LedRuntimeData {
            controller: led_device,
            profile,
        });

        led_handles.push(handle);
        led_runtimes.push(runtime);
    }

    let (performance_profile_handle, performance_profile_runtime) = match device {
        Some(device) => {
            let default_performance_profile = device.get_default_odm_performance_profile().unwrap();
            let (handle, runtime) = PerformanceProfileRuntime::new(
                device,
                profile.performance_profile,
                default_performance_profile,
            );
            (Some(handle), Some(runtime))
        }
        None => (None, None),
    };

    let profile_interface = ProfileInterface {
        led_handles: led_handles.clone(),
        fan_handles: fan_handles.clone(),
        performance_profile_handle: performance_profile_handle.clone(),
    };

    let led_interface = LedInterface {
        handles: led_handles,
    };

    let fan_interface = FanInterface {
        handles: fan_handles,
    };

    let performance_profile_interface = PerformanceInterface {
        handler: performance_profile_handle,
    };

    tracing::debug!("Connecting to DBUS as {DBUS_NAME}");
    let _conn = ConnectionBuilder::system()
        .unwrap()
        .name(DBUS_NAME)
        .unwrap()
        .serve_at(DBUS_PATH, led_interface)
        .unwrap()
        .serve_at(DBUS_PATH, fan_interface)
        .unwrap()
        .serve_at(DBUS_PATH, profile_interface)
        .unwrap()
        .serve_at(DBUS_PATH, performance_profile_interface)
        .unwrap()
        .build()
        .await
        .unwrap();

    tracing::debug!("Starting suspend watcher runtime");
    tokio_uring::spawn(suspend::wait_for_suspend());

    tracing::debug!("Starting {} led runtime(s)", led_runtimes.len());
    for runtime in led_runtimes {
        tokio_uring::spawn(runtime.run());
    }

    tracing::debug!("Starting {} fans runtime(s)", fan_runtimes.len());
    for runtime in fan_runtimes {
        tokio_uring::spawn(runtime.run());
    }

    if let Some(performance_profile_runtime) = performance_profile_runtime {
        tracing::debug!("Starting performance profile runtime");
        tokio_uring::spawn(performance_profile_runtime.run());
    }

    tracing::info!("Tailord started");
    tokio::select! {
        _ = pending() => {
            tracing::debug!("Pending main thread");
        }
        _ = shutdown_receiver.recv() => {
            tracing::info!("Shutting down, bye!");
            std::process::abort()
        }
    }
}
