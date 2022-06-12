use tailor_api::keyboard::ColorProfile;
use zbus::{dbus_proxy, fdo, Connection};

#[dbus_proxy(
    interface = "com.tux.Tailor",
    default_service = "com.tux.Tailor",
    default_path = "/com/tux/Tailor"
)]
trait Tailor {
    async fn add_color_profile(&self, name: &str, value: &str) -> fdo::Result<()>;

    async fn get_color_profile(&self, name: &str) -> fdo::Result<String>;

    async fn activate_color_profile(&self, name: &str) -> fdo::Result<()>;

    async fn list_color_profiles(&self) -> fdo::Result<Vec<String>>;
}

pub struct TailorConnection<'a> {
    proxy: TailorProxy<'a>,
}

impl<'a> TailorConnection<'a> {
    pub async fn new() -> Result<TailorConnection<'a>, zbus::Error> {
        let connection = Connection::system().await?;
        let proxy = TailorProxy::new(&connection).await?;
        Ok(Self { proxy })
    }

    pub async fn add_color_profile(&self, name: &str, colors: &ColorProfile) -> fdo::Result<()> {
        let value = serde_json::to_string(colors).unwrap();
        self.proxy.add_color_profile(name, &value).await
    }

    pub async fn get_color_profile(&self, name: &str) -> fdo::Result<ColorProfile> {
        self.proxy
            .get_color_profile(name)
            .await
            .map(|string| serde_json::from_str(&string).unwrap())
    }

    pub async fn activate_color_profile(&self, name: &str) -> fdo::Result<()> {
        self.proxy.activate_color_profile(name).await
    }

    pub async fn list_color_profiles(&self) -> fdo::Result<Vec<String>> {
        self.proxy.list_color_profiles().await
    }
}

#[cfg(test)]
mod test {
    use tailor_api::keyboard::{Color, ColorPoint, ColorProfile, ColorTransition};

    use crate::TailorConnection;

    #[tokio::test]
    async fn test_connection() {
        let connection = TailorConnection::new().await.unwrap();
        let profile = ColorProfile::Multiple(vec![
            ColorPoint {
                color: Color { r: 0, g: 255, b: 0 },
                transition: ColorTransition::Linear,
                transition_time: 3000,
            },
            ColorPoint {
                color: Color { r: 255, g: 0, b: 0 },
                transition: ColorTransition::Linear,
                transition_time: 3000,
            },
            ColorPoint {
                color: Color { r: 0, g: 0, b: 255 },
                transition: ColorTransition::Linear,
                transition_time: 3000,
            },
        ]);

        connection
            .add_color_profile("test", &profile)
            .await
            .unwrap();
        assert_eq!(connection.get_color_profile("test").await.unwrap(), profile);
        connection
            .list_color_profiles()
            .await
            .unwrap()
            .iter()
            .find(|s| s.as_str() == "test")
            .unwrap();
        connection.activate_color_profile("test").await.unwrap();
    }
}
