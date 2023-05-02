use zbus::{dbus_interface, fdo};

use crate::backlight::BacklightRuntimeHandle;

pub struct BacklightInterface {
    pub handler: BacklightRuntimeHandle,
}

#[dbus_interface(name = "com.tux.Tailor.Backlight")]
impl BacklightInterface {
    /// Temporarily override the performance profile. Please note that this will not survive a
    /// restart as the performance profile is handled by the overall profile configuration.
    async fn set_backlight_percentage(&mut self, value: u32) -> fdo::Result<()> {
        self.handler
            .backlight_percentage_sender
            .send(value as u8)
            .await
            .map_err(|err| {
                fdo::Error::IOError(format!(
                    "Unable to set backlight percentage to {value}%: {err}"
                ))
            })?;
        self.handler.actual_backlight_percentage = value as u8;
        Ok(())
    }

    /// Read the current performance profile.
    async fn get_backlight_percentage(&self) -> fdo::Result<u8> {
        Ok(self.handler.actual_backlight_percentage)
    }

    /// Read the list of supported performance profiles.
    async fn get_max_backlight_raw(&self) -> fdo::Result<u8> {
        Ok(self.handler.max_backlight_raw)
    }
}
