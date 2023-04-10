use std::time::Duration;

use relm4::tokio::sync::OnceCell;
use relm4::{Reducer, Reducible};
use tailor_api::{Color, ColorProfile, FanProfilePoint, LedDeviceInfo, ProfileInfo};
use tailor_client::{ClientError, TailorConnection};

use crate::app::FullProfileInfo;

pub static STATE: Reducer<TailorState> = Reducer::new();
static CONNECTION: OnceCell<TailorConnection<'static>> = OnceCell::const_new();
static HARDWARE_CAPABILITIES: OnceCell<HardwareCapabilities> = OnceCell::const_new();

pub fn tailor_connection() -> Option<&'static TailorConnection<'static>> {
    CONNECTION.get()
}

pub fn hardware_capabilities() -> Option<&'static HardwareCapabilities> {
    HARDWARE_CAPABILITIES.get()
}

#[derive(Clone)]
pub struct HardwareCapabilities {
    pub num_of_fans: u8,
    pub led_devices: Vec<LedDeviceInfo>,
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
    pub led_profiles: Vec<String>,
    pub fan_profiles: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub enum TailorStateMsg {
    Load(TailorStateInner),
    SetActiveProfile(String),
    AddProfile {
        name: String,
        profile: ProfileInfo,
    },
    AddFanProfile {
        name: String,
        profile: Vec<FanProfilePoint>,
    },
    AddLedProfile {
        name: String,
        profile: ColorProfile,
    },
    CopyProfile {
        from: String,
        to: String,
    },
    CopyFanProfile {
        from: String,
        to: String,
    },
    CopyLedProfile {
        from: String,
        to: String,
    },
    RenameProfile {
        from: String,
        to: String,
    },
    RenameFanProfile {
        from: String,
        to: String,
    },
    RenameLedProfile {
        from: String,
        to: String,
    },
    DeleteProfile(String),
    DeleteFanProfile(String),
    DeleteLedProfile(String),
    OverwriteColor(Color),
    OverwriteFanSpeed {
        fan_idx: u8,
        speed: u8,
    },
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
                return false;
            }
            TailorStateMsg::AddProfile { name, profile } => {
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

                    if let Some(full_profile) = state
                        .profiles
                        .iter_mut()
                        .find(|profile| profile.name == name)
                    {
                        full_profile.data = profile;
                        return false;
                    } else {
                        state.get_mut_profiles().push(FullProfileInfo {
                            name,
                            data: profile,
                        });
                    }
                }
            }
            TailorStateMsg::AddFanProfile { name, profile } => {
                if let Some(state) = self.get_mut() {
                    {
                        let name = name.clone();
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
            TailorStateMsg::AddLedProfile { name, profile } => {
                if let Some(state) = self.get_mut() {
                    {
                        let name = name.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.add_led_profile(&name, &profile).await);
                        });
                    }
                    if state.led_profiles.iter().any(|profile| profile == &name) {
                        return false;
                    } else {
                        state.get_mut_led_profiles().push(name);
                    }
                }
            }
            TailorStateMsg::CopyProfile { from, to } => {
                if let Some(state) = self.get_mut() {
                    {
                        let to = to.clone();
                        let from = from.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.copy_global_profile(&from, &to).await);
                        });
                    }
                    let profiles = state.get_mut_profiles();
                    if let Some(info) = profiles.iter().find(|profile| profile.name == from) {
                        profiles.push(FullProfileInfo {
                            name: to,
                            data: info.data.clone(),
                        });
                    }
                }
            }
            TailorStateMsg::CopyFanProfile { from, to } => {
                if let Some(state) = self.get_mut() {
                    {
                        let to = to.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.copy_fan_profile(&from, &to).await);
                        });
                    }
                    state.get_mut_fan_profiles().push(to);
                }
            }
            TailorStateMsg::CopyLedProfile { from, to } => {
                if let Some(state) = self.get_mut() {
                    {
                        let to = to.clone();
                        let connection = state.connection.clone();
                        relm4::spawn(async move {
                            handle_result(connection.copy_led_profile(&from, &to).await);
                        });
                    }
                    state.get_mut_led_profiles().push(to);
                }
            }
            TailorStateMsg::RenameProfile { from, to } => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_profiles();
                    if let Some(profile) = profiles.iter_mut().find(|p| p.name == from) {
                        {
                            let to = to.clone();
                            relm4::spawn(async move {
                                handle_result(connection.rename_global_profile(&from, &to).await);
                            });
                        }
                        profile.name = to;
                    }
                }
            }
            TailorStateMsg::RenameFanProfile { from, to } => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_fan_profiles();
                    if let Some(profile) = profiles.iter_mut().find(|p| *p == &from) {
                        {
                            let to = to.clone();
                            relm4::spawn(async move {
                                handle_result(connection.rename_fan_profile(&from, &to).await);
                            });
                        }
                        *profile = to;
                    }
                }
            }
            TailorStateMsg::RenameLedProfile { from, to } => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_led_profiles();
                    if let Some(profile) = profiles.iter_mut().find(|p| *p == &from) {
                        {
                            let to = to.clone();
                            relm4::spawn(async move {
                                handle_result(connection.rename_led_profile(&from, &to).await);
                            });
                        }
                        *profile = to;
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
            TailorStateMsg::DeleteLedProfile(name) => {
                if let Some(state) = self.get_mut() {
                    let connection = state.connection.clone();
                    let profiles = state.get_mut_led_profiles();
                    if let Some(pos) = profiles.iter().position(|p| p == &name) {
                        {
                            let name = name.clone();
                            relm4::spawn(async move {
                                handle_result(connection.remove_led_profile(&name).await);
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
                        handle_result(connection.override_led_colors(&color).await);
                    });
                }
                return false;
            }
            TailorStateMsg::OverwriteFanSpeed { fan_idx, speed } => {
                if let Some(state) = self.get() {
                    let connection = state.connection.clone();
                    relm4::spawn(async move {
                        handle_result(connection.override_fan_speed(fan_idx, speed).await);
                    });
                }
                return false;
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
    let num_of_fans = connection
        .get_number_of_fans()
        .await
        .map_err(|err| err.to_string())?;
    let led_devices = connection
        .get_led_devices()
        .await
        .map_err(|err| err.to_string())?;

    let capabilities = HardwareCapabilities {
        num_of_fans,
        led_devices,
    };

    let active_profile_name = connection
        .get_active_global_profile_name()
        .await
        .map_err(|e| e.to_string())?;

    let led_profiles = connection
        .list_led_profiles()
        .await
        .map_err(|e| e.to_string())?;

    let fan_profiles = connection
        .list_fan_profiles()
        .await
        .map_err(|e| e.to_string())?;

    let profiles = futures::future::join_all(
        connection
            .list_global_profiles()
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(|prof_name| async {
                let name = prof_name;
                let data = connection
                    .get_global_profile(&name)
                    .await
                    .unwrap_or_default();
                FullProfileInfo { name, data }
            }),
    )
    .await;

    // Everything worked so far, we can now safely set the global variables
    HARDWARE_CAPABILITIES.set(capabilities).ok().unwrap();
    CONNECTION
        .set(connection.clone())
        .expect("App was initialized twice");

    let state = TailorStateInner {
        connection,
        active_profile_name,
        profiles,
        led_profiles,
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
