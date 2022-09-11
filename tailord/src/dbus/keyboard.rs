use tailor_api::keyboard::{Color, ColorProfile};
use tokio::sync::mpsc;
use zbus::{dbus_interface, fdo};

use crate::{profiles::KEYBOARD_DIR, util};

pub struct KeyboardInterface {
    pub color_sender: mpsc::Sender<Color>,
}

#[dbus_interface(name = "com.tux.Tailor.Keyboard")]
impl KeyboardInterface {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<ColorProfile>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        util::write_file(KEYBOARD_DIR, name, value.as_bytes()).await
    }

    async fn get_profile(&self, name: &str) -> fdo::Result<String> {
        util::read_file(KEYBOARD_DIR, name).await
    }

    async fn list_profiles(&self) -> fdo::Result<Vec<String>> {
        util::get_profiles(KEYBOARD_DIR).await
    }

    async fn remove_profile(&self, name: &str) -> fdo::Result<()> {
        util::remove_file(KEYBOARD_DIR, name).await
    }

    async fn override_color(&mut self, color: &str) -> fdo::Result<()> {
        let color: Color =
            serde_json::from_str(color).map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        self.color_sender
            .send(color)
            .await
            .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))
    }
}