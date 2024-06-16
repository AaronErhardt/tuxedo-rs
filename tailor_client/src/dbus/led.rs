use zbus::{fdo, proxy};

#[proxy(
    interface = "com.tux.Tailor.Led",
    default_service = "com.tux.Tailor",
    default_path = "/com/tux/Tailor"
)]
trait Led {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()>;

    async fn get_profile(&self, name: &str) -> fdo::Result<String>;

    async fn list_profiles(&self) -> fdo::Result<Vec<String>>;

    async fn remove_profile(&self, name: &str) -> fdo::Result<()>;

    async fn rename_profile(&self, from: &str, to: &str) -> fdo::Result<Vec<String>>;

    async fn override_color(&self, color: &str) -> fdo::Result<()>;
}
