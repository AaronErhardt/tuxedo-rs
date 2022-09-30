use relm4::SharedState;
use tailor_client::TailorConnection;

use crate::app::FullProfileInfo;

pub struct TailorState {
    pub connection: TailorConnection<'static>,
    pub active_profile_name: String,
    pub profiles: Vec<FullProfileInfo>,
}

pub static KEYBOARD_PROFILES: SharedState<Vec<String>> = SharedState::new();
pub static FAN_PROFILES: SharedState<Vec<String>> = SharedState::new();

pub static TAILOR_STATE: SharedState<Option<TailorState>> = SharedState::new();

pub async fn initialize_tailor_state() -> Result<(), String> {
    let connection = TailorConnection::new().await.map_err(|e| e.to_string())?;

    let active_profile_name = connection
        .get_active_global_profile_name()
        .await
        .map_err(|e| e.to_string())?;

    let keyboard_profiles = connection
        .list_keyboard_profiles()
        .await
        .map_err(|e| e.to_string())?;

    let fan_profiles = connection
        .list_fan_profiles()
        .await
        .map_err(|e| e.to_string())?;

    let profiles = futures::future::try_join_all(
        connection
            .list_global_profiles()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|prof_name| async {
                let name = prof_name;
                connection
                    .get_global_profile(&name)
                    .await
                    .map(|data| FullProfileInfo { name, data })
            }),
    )
    .await
    .map_err(|e| e.to_string())?;

    *TAILOR_STATE.write() = Some(TailorState {
        connection,
        active_profile_name,
        profiles,
    });

    *KEYBOARD_PROFILES.write() = keyboard_profiles;
    *FAN_PROFILES.write() = fan_profiles;

    Ok(())
}
