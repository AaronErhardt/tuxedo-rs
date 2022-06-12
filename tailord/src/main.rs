pub mod keyboard;
mod suspend;

use std::{future::pending, time::Duration};

use tailor_api::keyboard::ColorProfile;
use tokio::sync::{broadcast, mpsc};
use zbus::{dbus_interface, fdo, Connection};

use crate::keyboard::runtime::KeyboardRuntime;

struct Tailor {
    color_sender: mpsc::Sender<ColorProfile>,
}

#[dbus_interface(name = "com.tux.Tailor")]
impl Tailor {
    async fn add_color_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        let value: ColorProfile =
            serde_json::from_str(value).map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;

        // Deserialization should work after data has been validated.
        let data = serde_json::to_vec_pretty(&value).unwrap();

        keyboard::dbus::write_keyboard_file(name, &data).await
    }

    async fn get_color_profile(&self, name: &str) -> fdo::Result<String> {
        let colors = keyboard::dbus::load_keyboard_colors(name).await?;

        // Serialization should work after data has been validated.
        Ok(serde_json::to_string(&colors).unwrap())
    }

    async fn activate_color_profile(&self, name: &str) -> fdo::Result<()> {
        let colors = keyboard::dbus::load_keyboard_colors(name).await?;
        keyboard::dbus::write_active_profile_file(name).await?;
        self.color_sender.send(colors).await.unwrap();
        Ok(())
    }

    async fn list_color_profiles(&self) -> fdo::Result<Vec<String>> {
        keyboard::dbus::get_color_profiles().await
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

    keyboard::dbus::init_keyboard_directory();

    tokio_uring::start(async {
        start_dbus().await;
    });
}

async fn start_dbus() {
    let (suspend_sender, suspend_receiver) = broadcast::channel(1);
    let (color_sender, color_receiver) = mpsc::channel(1);

    let tailor = Tailor { color_sender };

    let connection = Connection::system().await.unwrap();

    // setup the server
    connection
        .object_server()
        .at("/com/tux/Tailor", tailor)
        .await
        .unwrap();

    connection.request_name("com.tux.Tailor").await.unwrap();

    let mut keyboard_rt = KeyboardRuntime::new(suspend_receiver, color_receiver).await;

    tokio::select! {
        res = suspend::wait_for_suspend(suspend_sender) => {
            res.unwrap();
        }
        _ = keyboard_rt.run() => {
        }
    }

    pending().await
}
