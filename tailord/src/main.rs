mod dbus;
mod fancontrol;
pub mod keyboard;
mod profiles;
mod suspend;
pub mod util;

use std::future::pending;

use dbus::{FanInterface, KeyboardInterface, ProfileInterface};
use fancontrol::FanRuntime;
use futures::StreamExt;
use profiles::Profile;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::sync::{broadcast, mpsc};
use zbus::ConnectionBuilder;

use crate::keyboard::runtime::KeyboardRuntime;

const DBUS_NAME: &str = "com.tux.Tailor";
const DBUS_PATH: &str = "/com/tux/Tailor";

fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "trace");
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
    let (suspend_sender, suspend_receiver) = broadcast::channel(1);
    let (shutdown_sender, mut shutdown_receiver) = broadcast::channel(1);

    let (keyboard_sender, keyboard_receiver) = mpsc::channel(1);
    let (fan_sender, fan_receiver) = mpsc::channel(1);

    let (color_sender, color_receiver) = mpsc::channel(1);
    let (fan_speed_sender, fan_speed_receiver) = mpsc::channel(1);

    let signals = Signals::new([SIGTERM, SIGINT, SIGQUIT]).unwrap();
    tracing::debug!("Starting signal handler background loop");
    tokio_uring::spawn(handle_signals(signals, shutdown_sender));

    let keyboard_interface = KeyboardInterface {
        color_sender,
        keyboard_sender: keyboard_sender.clone(),
    };

    let fan_interface = FanInterface {
        fan_speed_sender,
        fan_sender: fan_sender.clone(),
    };

    let profile_interface = ProfileInterface {
        keyboard_sender,
        fan_sender,
    };

    tracing::debug!("Connecting to DBUS as {DBUS_NAME}");
    let _ = ConnectionBuilder::system()
        .unwrap()
        .name(DBUS_NAME)
        .unwrap()
        .serve_at(DBUS_PATH, keyboard_interface)
        .unwrap()
        .serve_at(DBUS_PATH, fan_interface)
        .unwrap()
        .serve_at(DBUS_PATH, profile_interface)
        .unwrap()
        .build()
        .await
        .unwrap();

    let Profile { fan, keyboard } = Profile::load();

    let keyboard_rt = KeyboardRuntime::new(keyboard, suspend_receiver).await;
    let fan_rt = FanRuntime::new(fan, suspend_sender.subscribe());

    tracing::debug!("Starting wait_for_suspend() background loop");
    tokio_uring::spawn(suspend::wait_for_suspend(suspend_sender));
    tracing::debug!("Starting keyboard background loop");
    tokio_uring::spawn(keyboard_rt.run(keyboard_receiver, color_receiver));
    tracing::debug!("Starting fans background loop");
    // tokio_uring::spawn(fan_rt.run(fan_receiver, fan_speed_receiver));

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

#[tracing::instrument(skip(signals))]
async fn handle_signals(mut signals: Signals, shutdown_sender: broadcast::Sender<()>) {
    tracing::debug!("Waiting for signals");
    while let Some(signal) = signals.next().await {
        tracing::debug!("Caught signal {signal}");
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                // It's ok to panic here if a send error occurs.
                // The application is terminated anyway and
                // an error at this point can't be recovered.
                tracing::info!("Received a shutdown signal");
                shutdown_sender.send(()).unwrap();
            }
            _ => unreachable!(),
        }
    }
}
