use tailor_api::{Color, ColorProfile, ProfileInfo};
use zbus::{dbus_interface, fdo};

use crate::{
    led::LedRuntimeHandle,
    profiles::{Profile, LED_DIR, PROFILE_DIR},
    util,
};

pub struct LedInterface {
    pub handles: Vec<LedRuntimeHandle>,
}

#[dbus_interface(name = "com.tux.Tailor.Led")]
impl LedInterface {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<ColorProfile>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        util::write_file(LED_DIR, name, value.as_bytes()).await?;

        // Reload if the led profile is part of the active global profile
        let info = Profile::get_active_profile_info()?;
        if info.leds.iter().any(|prof| prof.profile == name) {
            let info = Profile::load();
            for handle in &self.handles {
                let profile = info
                    .leds
                    .iter()
                    .find_map(|(info, profile)| {
                        if info == &handle.info {
                            Some(profile.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                handle.profile_sender.send(profile).await.unwrap();
            }
        }
        Ok(())
    }

    async fn get_profile(&self, name: &str) -> fdo::Result<String> {
        util::read_file(LED_DIR, name).await
    }

    async fn list_profiles(&self) -> fdo::Result<Vec<String>> {
        util::get_profiles(LED_DIR).await
    }

    async fn remove_profile(&self, name: &str) -> fdo::Result<()> {
        util::remove_file(LED_DIR, name).await
    }

    async fn rename_profile(&self, from: &str, to: &str) -> fdo::Result<Vec<String>> {
        if self.list_profiles().await?.contains(&to.to_string()) {
            Err(fdo::Error::InvalidArgs(format!(
                "File `{to}` already exists"
            )))
        } else {
            let profiles = util::get_profiles(PROFILE_DIR).await?;

            for profile in profiles {
                let mut data =
                    if let Ok(data) = util::read_json::<ProfileInfo>(PROFILE_DIR, &profile).await {
                        data
                    } else {
                        continue;
                    };
                let mut changed = false;

                for led in &mut data.leds {
                    if led.profile == from {
                        led.profile = to.to_owned();
                        changed = true;
                    }
                }

                if changed {
                    util::write_json(PROFILE_DIR, &profile, &data).await?;
                }
            }

            util::move_file(LED_DIR, from, to).await?;

            self.list_profiles().await
        }
    }

    async fn override_color(&mut self, color: &str) -> fdo::Result<()> {
        let color: Color =
            serde_json::from_str(color).map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        for handle in &self.handles {
            handle
                .color_sender
                .send(color.clone())
                .await
                .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))?;
        }
        Ok(())
    }
}
