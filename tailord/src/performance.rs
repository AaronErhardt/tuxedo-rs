use std::{sync::Arc, time::Duration};

use tokio::sync::{broadcast, mpsc};
use tuxedo_ioctl::hal::traits::HardwareDevice;

use crate::suspend::get_suspend_receiver;

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
}

pub struct PerformanceProfileRuntime {
    profile_receiver: mpsc::Receiver<String>,
    /// Device i/o interface.
    io: Arc<dyn HardwareDevice>,
    /// Default profile
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
        (
            PerformanceProfileRuntimeHandle {
                profile_sender,
                performance_profile,
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
            tokio::select! {
                new_profile = self.profile_receiver.recv() => {
                    if let Some(profile) = new_profile {
                        tracing::info!("Loading performance profile {profile}");
                        self.io.set_odm_performance_profile(&profile).unwrap();
                    } else {
                        break;
                    }
                },
                _ = tokio::time::sleep(Duration::from_millis(1000)) => {},
            }
        }
    }
}
