mod backlight;
mod fan;
mod led;
mod performance;
mod profiles;

pub(crate) use backlight::BacklightProxy;
pub(crate) use fan::FanProxy;
pub(crate) use led::LedProxy;
pub(crate) use performance::PerformanceProxy;
pub(crate) use profiles::ProfilesProxy;
