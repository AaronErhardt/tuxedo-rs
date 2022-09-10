mod fancontrol;
pub mod keyboard;
mod profiles;
mod suspend;

use std::{future::pending, path::Component};

use fancontrol::{profile::FanProfile, FanRuntime};
use futures::StreamExt;
use profiles::{
    util::normalize_json_path, Profile, ProfileInfo, FAN_DIR, KEYBOARD_DIR, PROFILE_DIR,
};
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tailor_api::{
    fan::FanProfilePoint,
    keyboard::{Color, ColorProfile},
};
use tokio::sync::{broadcast, mpsc};
use zbus::{dbus_interface, fdo, ConnectionBuilder};

use crate::keyboard::runtime::KeyboardRuntime;

const DBUS_INTERFACE: &str = "/com/tux/Tailor";

struct Tailor {
    keyboard_sender: mpsc::Sender<ColorProfile>,
    fan_sender: mpsc::Sender<FanProfile>,
    color_sender: mpsc::Sender<Color>,
    fan_speed_sender: mpsc::Sender<u8>,
}

#[dbus_interface(name = "com.tux.Tailor1")]
impl Tailor {
    async fn add_keyboard_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<ColorProfile>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        profiles::util::write_file(KEYBOARD_DIR, name, value.as_bytes()).await
    }

    async fn add_fan_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<Vec<FanProfilePoint>>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        profiles::util::write_file(FAN_DIR, name, value.as_bytes()).await
    }

    async fn add_global_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<ProfileInfo>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;

        profiles::util::write_file(PROFILE_DIR, name, value.as_bytes()).await
    }

    async fn get_keyboard_profile(&self, name: &str) -> fdo::Result<String> {
        profiles::util::read_file(KEYBOARD_DIR, name).await
    }

    async fn get_fan_profile(&self, name: &str) -> fdo::Result<String> {
        profiles::util::read_file(FAN_DIR, name).await
    }

    async fn get_global_profile(&self, name: &str) -> fdo::Result<String> {
        profiles::util::read_file(PROFILE_DIR, name).await
    }

    async fn list_keyboard_profiles(&self) -> fdo::Result<Vec<String>> {
        profiles::util::get_profiles(KEYBOARD_DIR).await
    }

    async fn list_fan_profiles(&self) -> fdo::Result<Vec<String>> {
        profiles::util::get_profiles(FAN_DIR).await
    }

    async fn list_global_profiles(&self) -> fdo::Result<Vec<String>> {
        profiles::util::get_profiles(PROFILE_DIR).await
    }

    async fn set_active_global_profile_name(&self, name: &str) -> fdo::Result<()> {
        std::fs::metadata(normalize_json_path(PROFILE_DIR, name)?)
            .map_err(|_| fdo::Error::FileNotFound(format!("Couldn't find profile `{name}`")))?;

        let link_path = format!("{PROFILE_DIR}active_profile.json");
        drop(std::fs::remove_file(&link_path));
        std::os::unix::fs::symlink(normalize_json_path("", name)?, &link_path)
            .map_err(|err| fdo::Error::IOError(err.to_string()))
    }

    async fn get_active_global_profile_name(&self) -> fdo::Result<String> {
        let link = std::fs::read_link(&format!("{PROFILE_DIR}active_profile.json"))
            .map_err(|err| fdo::Error::IOError(err.to_string()))?;
        let components: Vec<Component> = link.components().collect();
        if components.len() == 1 {
            if let Component::Normal(name) = components.first().unwrap() {
                if let Some(name) = name.to_str() {
                    return Ok(name.trim_end_matches(".json").to_string());
                }
            }
        }

        Err(fdo::Error::InvalidFileContent(
            "File `active_profile.json` isn't a valid link".to_string(),
        ))
    }

    async fn reload(&mut self) -> fdo::Result<()> {
        let Profile { fan, keyboard } = Profile::reload()?;
        let res1 = self
            .keyboard_sender
            .send(keyboard)
            .await
            .map_err(|e| e.to_string());
        let res2 = self.fan_sender.send(fan).await.map_err(|e| e.to_string());
        res1.and(res2)
            .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))?;
        Ok(())
    }

    async fn override_fan_speed(&mut self, speed: u8) -> fdo::Result<()> {
        self.fan_speed_sender
            .send(speed)
            .await
            .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))
    }

    async fn override_keyboard_color(&mut self, color: &str) -> fdo::Result<()> {
        let color: Color =
            serde_json::from_str(color).map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        self.color_sender
            .send(color)
            .await
            .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))
    }
}

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

async fn start_runtime() {
    let (suspend_sender, suspend_receiver) = broadcast::channel(1);
    let (shutdown_sender, mut shutdown_receiver) = broadcast::channel(1);

    let (keyboard_sender, keyboard_receiver) = mpsc::channel(1);
    let (fan_sender, fan_receiver) = mpsc::channel(1);

    let (color_sender, color_receiver) = mpsc::channel(1);
    let (fan_speed_sender, fan_speed_receiver) = mpsc::channel(1);

    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT]).unwrap();
    tokio_uring::spawn(handle_signals(signals, shutdown_sender));

    let tailor = Tailor {
        keyboard_sender,
        fan_sender,
        color_sender,
        fan_speed_sender,
    };

    let _ = ConnectionBuilder::system()
        .unwrap()
        .name("com.tux.Tailor")
        .unwrap()
        .serve_at(DBUS_INTERFACE, tailor)
        .unwrap()
        .build()
        .await
        .unwrap();

    let Profile { fan, keyboard } = Profile::load();

    let keyboard_rt = KeyboardRuntime::new(keyboard, suspend_receiver).await;
    let fan_rt = FanRuntime::new(fan, suspend_sender.subscribe());

    tokio_uring::spawn(suspend::wait_for_suspend(suspend_sender));
    tokio_uring::spawn(keyboard_rt.run(keyboard_receiver, color_receiver));
    tokio_uring::spawn(fan_rt.run(fan_receiver, fan_speed_receiver));

    tokio::select! {
        _ = pending() => {}
        _ = shutdown_receiver.recv() => {
            tracing::info!("Shutting down, bye!");
            std::process::abort()
        }
    }
}

async fn handle_signals(mut signals: Signals, shutdown_sender: broadcast::Sender<()>) {
    while let Some(signal) = signals.next().await {
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
