use std::time::Duration;

use relm4::tokio::sync::OnceCell;
use relm4::{Reducer, Reducible};
use tailor_api::{Color, ColorProfile, FanProfilePoint, ProfileInfo};
use tailor_client::{ClientError, TailorConnection};

use crate::app::FullProfileInfo;

pub static STATE: Reducer<TailorState> = Reducer::new();
static CONNECTION: OnceCell<TailorConnection<'static>> = OnceCell::const_new();

pub fn tailor_connection() -> Option<&'static TailorConnection<'static>> {
    CONNECTION.get()
}

pub enum TailorState {
    Uninitialized,
    Initialized(TailorStateInner),
}

impl TailorState {
    pub fn get(&self) -> Option<&TailorStateInner> {
        match self {
            Self::Initialized(state) => Some(state),
            Self::Uninitialized => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut TailorStateInner> {
        match self {
            Self::Initialized(state) => Some(state),
            Self::Uninitialized => None,
        }
    }

    pub fn unwrap(&self) -> &TailorStateInner {
        self.get().expect("Unwrapped a uninitialized tailor state")
    }

    pub fn unwrap_mut(&mut self) -> &TailorStateInner {
        self.get_mut()
            .expect("Unwrapped a uninitialized tailor state")
    }
}

#[derive(Debug)]
#[tracker::track]
pub struct TailorStateInner {
    #[no_eq]
    pub connection: TailorConnection<'static>,
    pub active_profile_name: String,
    pub profiles: Vec<FullProfileInfo>,
    pub keyboard_profiles: Vec<String>,
    pub fan_profiles: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub enum TailorStateMsg {
    Load(TailorStateInner),
    SetActiveProfile(String),
    AddProfile(String, ProfileInfo),
    AddFanProfile(String, Vec<FanProfilePoint>),
    AddKeyboardProfile(String, ColorProfile),
    CopyFanProfile(String, String),
    CopyKeyboardProfile(String, String),
    RenameProfile(String, String),
    RenameFanProfile(String, String),
    RenameKeyboardProfile(String, String),
    DeleteProfile(String),
    DeleteFanProfile(String),
    DeleteKeyboardProfile(String),
    OverwriteColor(Color),
    OverwriteFanSpeed(u8),
    Error(String),
}

impl Reducible for TailorState {
    type Input = TailorStateMsg;

    fn init() -> Self {
        Self::Uninitialized
    }

    fn reduce(&mut self, input: Self::Input) -> bool {
        match input {
            TailorStateMsg::Load(mut state) => {
                state.mark_all_changed();
                *self = Self::Initialized(state);
            }
            TailorStateMsg::SetActiveProfile(name) => {
                if let Some(state) = self.get_mut() {
                    {
                        let name = name.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.set_active_global_profile_name(&name).await);
                            handle_result(connection.reload().await);
                        });
                    }
                    *state.get_mut_active_profile_name() = name;
                }
            }
            TailorStateMsg::AddProfile(name, profile) => {
                if let Some(state) = self.get_mut() {
                    {
                        let name = name.clone();
                        let profile = profile.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.add_global_profile(&name, &profile).await);
                            if let Some(active_name) =
                                handle_result(connection.get_active_global_profile_name().await)
                            {
                                if active_name == name {
                                    handle_result(connection.reload().await);
                                }
                            }
                        });
                    }

                    if state.profiles.iter().any(|profile| profile.name == name) {
                        return false;
                    } else {
                        state.get_mut_profiles().push(FullProfileInfo {
                            name,
                            data: profile,
                        });
                    }
                }
            }
            TailorStateMsg::AddFanProfile(name, profile) => {
                if let Some(state) = self.get_mut() {
                    {
                        let name = name.clone();
                        let profile = profile.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.add_fan_profile(&name, &profile).await);
                        });
                    }
                    if state.fan_profiles.iter().any(|profile| profile == &name) {
                        return false;
                    } else {
                        state.get_mut_fan_profiles().push(name);
                    }
                }
            }
            TailorStateMsg::AddKeyboardProfile(name, profile) => {
                if let Some(state) = self.get_mut() {
                    {
                        let name = name.clone();
                        let profile = profile.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.add_keyboard_profile(&name, &profile).await);
                        });
                    }
                    if state
                        .keyboard_profiles
                        .iter()
                        .any(|profile| profile == &name)
                    {
                        return false;
                    } else {
                        state.get_mut_keyboard_profiles().push(name);
                    }
                }
            }
            TailorStateMsg::CopyFanProfile(other_name, new_name) => {
                if let Some(state) = self.get_mut() {
                    {
                        let new_name = new_name.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(
                                connection.copy_fan_profiles(&new_name, &other_name).await,
                            );
                        });
                    }
                    state.get_mut_fan_profiles().push(new_name);
                }
            }
            TailorStateMsg::CopyKeyboardProfile(other_name, new_name) => {
                if let Some(state) = self.get_mut() {
                    {
                        let new_name = new_name.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(
                                connection
                                    .copy_keyboard_profiles(&new_name, &other_name)
                                    .await,
                            );
                        });
                    }
                    state.get_mut_keyboard_profiles().push(new_name);
                }
            }
            TailorStateMsg::RenameProfile(old_name, new_name) => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_profiles();
                    if let Some(profile) = profiles.iter_mut().find(|p| p.name == old_name) {
                        {
                            let new_name = new_name.clone();
                            relm4::spawn(async move {
                                handle_result(
                                    connection.rename_global_profile(&old_name, &new_name).await,
                                );
                            });
                        }
                        profile.name = new_name;
                    }
                }
            }
            TailorStateMsg::RenameFanProfile(old_name, new_name) => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_fan_profiles();
                    if let Some(profile) = profiles.iter_mut().find(|p| *p == &old_name) {
                        {
                            let new_name = new_name.clone();
                            relm4::spawn(async move {
                                handle_result(
                                    connection.rename_fan_profile(&old_name, &new_name).await,
                                );
                            });
                        }
                        *profile = new_name;
                    }
                }
            }
            TailorStateMsg::RenameKeyboardProfile(old_name, new_name) => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_keyboard_profiles();
                    if let Some(profile) = profiles.iter_mut().find(|p| *p == &old_name) {
                        {
                            let new_name = new_name.clone();
                            relm4::spawn(async move {
                                handle_result(
                                    connection
                                        .rename_keyboard_profile(&old_name, &new_name)
                                        .await,
                                );
                            });
                        }
                        *profile = new_name;
                    }
                }
            }
            TailorStateMsg::DeleteProfile(name) => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_profiles();
                    if let Some(pos) = profiles.iter().position(|p| p.name == name) {
                        {
                            let name = name.clone();
                            relm4::spawn(async move {
                                handle_result(connection.remove_global_profile(&name).await);
                            });
                        }
                        profiles.remove(pos);
                    }
                }
            }
            TailorStateMsg::DeleteFanProfile(name) => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_fan_profiles();
                    if let Some(pos) = profiles.iter().position(|p| p == &name) {
                        {
                            let name = name.clone();
                            relm4::spawn(async move {
                                handle_result(connection.remove_fan_profile(&name).await);
                            });
                        }
                        profiles.remove(pos);
                    }
                }
            }
            TailorStateMsg::DeleteKeyboardProfile(name) => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_keyboard_profiles();
                    if let Some(pos) = profiles.iter().position(|p| p == &name) {
                        {
                            let name = name.clone();
                            relm4::spawn(async move {
                                handle_result(connection.remove_keyboard_profile(&name).await);
                            });
                        }
                        profiles.remove(pos);
                    }
                }
            }
            TailorStateMsg::OverwriteColor(color) => {
                if let Some(state) = self.get() {
                    let connection = state.connection.clone();
                    relm4::spawn(async move {
                        handle_result(connection.override_keyboard_color(&color).await);
                    });
                }
            }
            TailorStateMsg::OverwriteFanSpeed(speed) => {
                if let Some(state) = self.get() {
                    let connection = state.connection.clone();
                    relm4::spawn(async move {
                        handle_result(connection.override_fan_speed(speed).await);
                    });
                }
            }
            TailorStateMsg::Error(error) => {
                if let Some(state) = self.get_mut() {
                    state.set_error(Some(error));
                }
            }
        }
        true
    }
}

pub async fn initialize_tailor_state() -> Result<(), String> {
    let connection = TailorConnection::new().await.map_err(|e| e.to_string())?;
    CONNECTION
        .set(connection.clone())
        .expect("App was initialized twice");

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

    let state = TailorStateInner {
        connection,
        active_profile_name,
        profiles,
        keyboard_profiles,
        fan_profiles,
        tracker: 0,
        error: None,
    };

    tokio::time::sleep(Duration::from_millis(100)).await;

    STATE.emit(TailorStateMsg::Load(state));

    Ok(())
}

fn handle_result<T>(result: Result<T, ClientError>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            STATE.emit(TailorStateMsg::Error(error.to_string()));
            None
        }
    }
}
