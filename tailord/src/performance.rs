use std::sync::Arc;

use tokio::sync::mpsc;
use tuxedo_ioctl::hal::traits::HardwareDevice;

#[derive(Debug)]
pub struct PerformanceProfile(String);

impl ToString for PerformanceProfile {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl PerformanceProfile {
    pub fn new(value: impl ToString) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone)]
pub struct PerformanceProfileRuntimeHandle {
    pub profile_sender: mpsc::Sender<String>,
    pub performance_profile: String,
    pub available_performance_profiles: Vec<String>,
}

#[allow(unused)]
pub struct PerformanceProfileRuntime {
    profile_receiver: mpsc::Receiver<String>,
    /// Device i/o interface.
    io: Arc<dyn HardwareDevice>,
    /// Default profile.
    default_performance_profile: String,
}

impl PerformanceProfileRuntime {
    // initialize global instance at startup
    #[tracing::instrument(skip(io))]
    pub fn new(
        io: Arc<dyn HardwareDevice>,
        performance_profile: Option<PerformanceProfile>,
        default_performance_profile: String,
    ) -> (PerformanceProfileRuntimeHandle, PerformanceProfileRuntime) {
        let (profile_sender, profile_receiver) = mpsc::channel(1);
        let performance_profile = match performance_profile {
            Some(profile) => profile.to_string(),
            None => default_performance_profile.to_string(),
        };
        io.set_odm_performance_profile(&performance_profile)
            .unwrap();
        let available_performance_profiles = io.get_available_odm_performance_profiles().unwrap();
        (
            PerformanceProfileRuntimeHandle {
                profile_sender,
                performance_profile,
                available_performance_profiles,
            },
            PerformanceProfileRuntime {
                profile_receiver,
                io,
                default_performance_profile,
            },
        )
    }

    #[tracing::instrument(skip(self))]
    pub async fn run(mut self) {
        loop {
            if let Some(profile) = self.profile_receiver.recv().await {
                tracing::info!("Loading performance profile {profile}");
                self.io.set_odm_performance_profile(&profile).unwrap();
            } else {
                tracing::warn!("Received empty performance profile, stopping runtime");
                break;
            }
        }
    }
}
