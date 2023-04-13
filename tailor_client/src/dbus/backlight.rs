use zbus::{dbus_proxy, fdo};

#[dbus_proxy(
    interface = "com.tux.Tailor.Backlight",
    default_service = "com.tux.Tailor",
    default_path = "/com/tux/Tailor"
)]
trait Backlight {
    /// Get the current backlight percentage.
    async fn get_backlight_percentage(&self) -> fdo::Result<u8>;

    /// Get the raw maximum backlight.
    async fn get_max_backlight_raw(&self) -> fdo::Result<u8>;

    /// Set a new backlight percentage.
    async fn set_backlight_percentage(&self, value: u8) -> fdo::Result<()>;
}
