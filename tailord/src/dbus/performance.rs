use zbus::{dbus_interface, fdo};

use crate::performance::PerformanceProfileRuntimeHandle;

pub struct PerformanceInterface {
    pub handler: Option<PerformanceProfileRuntimeHandle>,
}

impl PerformanceInterface {
    fn handler(&self) -> fdo::Result<&PerformanceProfileRuntimeHandle> {
        self.handler.as_ref().ok_or(fdo::Error::Failed(
            "No performance profile handler available".to_string(),
        ))
    }

    fn handler_mut(&mut self) -> fdo::Result<&mut PerformanceProfileRuntimeHandle> {
        self.handler.as_mut().ok_or(fdo::Error::Failed(
            "No performance profile handler available".to_string(),
        ))
    }
}

#[dbus_interface(name = "com.tux.Tailor.Performance")]
impl PerformanceInterface {
    async fn set_profile(&mut self, name: &str) -> fdo::Result<()> {
        self.handler()?
            .profile_sender
            .send(name.to_string())
            .await
            .map_err(|err| {
                fdo::Error::IOError(format!("unable to set performance profile {name}: {err}"))
            })?;
        self.handler_mut()?.set_active_performance_profile(name);
        Ok(())
    }

    async fn get_profile(&self) -> fdo::Result<String> {
        Ok(self.handler()?.get_active_performance_profile().to_string())
    }

    async fn list_profiles(&self) -> fdo::Result<Vec<String>> {
        Ok(self
            .handler()?
            .get_availables_performance_profiles()
            .map_err(|err| {
                fdo::Error::IOError(format!(
                    "unable to list available performance profiles: {err}"
                ))
            })?)
    }
}
