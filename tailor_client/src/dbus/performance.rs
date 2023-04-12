use zbus::{dbus_proxy, fdo};

#[dbus_proxy(
    interface = "com.tux.Tailor.Performance",
    default_service = "com.tux.Tailor",
    default_path = "/com/tux/Tailor"
)]
trait Performance {
    async fn set_performance_profile(&self, name: &str, value: &str) -> fdo::Result<()>;

    async fn get_performance_profile(&self, name: &str) -> fdo::Result<String>;

    async fn list_performance_profiles(&self) -> fdo::Result<Vec<String>>;
}
