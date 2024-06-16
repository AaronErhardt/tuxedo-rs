use zbus::{fdo, proxy};

#[proxy(
    interface = "com.tux.Tailor.Performance",
    default_service = "com.tux.Tailor",
    default_path = "/com/tux/Tailor"
)]
trait Performance {
    /// Temporarily override the performance profile. Please note that this will not survive a
    /// restart as the performance profile is handled by the overall profile configuration.
    async fn set_profile(&self, name: &str, value: &str) -> fdo::Result<()>;

    /// Read the current performance profile.
    async fn get_profile(&self, name: &str) -> fdo::Result<String>;

    /// Read the list of supported performance profiles.
    async fn list_profiles(&self) -> fdo::Result<Vec<String>>;
}
