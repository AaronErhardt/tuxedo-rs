use zbus::{dbus_proxy, fdo};

#[dbus_proxy(
    interface = "com.tux.Tailor.Fan",
    default_service = "com.tux.Tailor",
    default_path = "/com/tux/Tailor"
)]
trait Fan {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()>;

    async fn get_profile(&self, name: &str) -> fdo::Result<String>;

    async fn list_profiles(&self) -> fdo::Result<Vec<String>>;

    async fn remove_profile(&self, name: &str) -> fdo::Result<()>;

    async fn override_speed(&self, speed: u8) -> fdo::Result<()>;
}
