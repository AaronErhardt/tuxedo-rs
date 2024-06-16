use std::sync::Arc;

use tokio::sync::mpsc;
use tuxedo_ioctl::hal::{traits::HardwareDevice, IoctlResult};

#[derive(Debug)]
pub struct PerformanceProfile(String);

impl std::fmt::Display for PerformanceProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl PerformanceProfile {
    pub fn new(value: impl ToString) -> Self {
        Self(value.to_string())
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct PerformanceProfileRuntimeHandle {
    pub profile_sender: mpsc::Sender<String>,
    /// Device i/o interface.
    io: Arc<dyn HardwareDevice>,
    /// Current profile.
    performance_profile: String,
}

impl PerformanceProfileRuntimeHandle {
    pub fn get_availables_performance_profiles(&self) -> IoctlResult<Vec<String>> {
        self.io.get_available_odm_performance_profiles()
    }
    pub fn set_active_performance_profile(&mut self, name: &str) {
        self.performance_profile = name.to_string();
    }
    pub fn get_active_performance_profile(&self) -> &str {
        &self.performance_profile
    }
}

#[allow(unused)]
pub struct PerformanceProfileRuntime {
    profile_receiver: mpsc::Receiver<String>,
    /// Device i/o interface.
    io: Arc<dyn HardwareDevice>,
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
        (
            PerformanceProfileRuntimeHandle {
                profile_sender,
                io: io.clone(),
                performance_profile,
            },
            PerformanceProfileRuntime {
                profile_receiver,
                io,
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
                tracing::warn!(
                    "Stopping runtime, the performance profile channel sender has probably dropped"
                );
                break;
            }
        }
    }
}
