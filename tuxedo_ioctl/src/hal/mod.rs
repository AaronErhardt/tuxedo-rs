use std::sync::Arc;

use crate::{config::open_device_file, error::IoctlError, read};

use self::{
    clevo::ClevoHardware,
    traits::{HardwareDevice, TdpDevice, WebcamDevice},
    uniwill::UniwillHardware,
};

mod clevo;
pub mod traits;
mod uniwill;

pub type IoctlResult<T> = Result<T, IoctlError>;

pub struct IoInterface {
    pub module_version: String,
    pub device: Arc<dyn HardwareDevice>,
    pub webcam: Option<Arc<dyn WebcamDevice>>,
    pub tdp: Option<Arc<dyn TdpDevice>>,
}

impl IoInterface {
    pub fn new() -> IoctlResult<Self> {
        let file = open_device_file()?;
        let module_version = read::mod_version(&file)?;

        if matches!(read::cl::hw_check(&file), Ok(1)) {
            let clevo_hardware = ClevoHardware::init(file)?;
            let interface = Arc::new(clevo_hardware);
            Ok(Self {
                module_version,
                device: interface.clone(),
                webcam: Some(interface),
                tdp: None,
            })
        } else if matches!(read::uw::hw_check(&file), Ok(1)) {
            let uniwill_hardware = UniwillHardware::init(file)?;
            let interface = Arc::new(uniwill_hardware);
            Ok(Self {
                module_version,
                device: interface.clone(),
                webcam: None,
                tdp: Some(interface),
            })
        } else {
            Err(IoctlError::DevNotAvailable)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn interface() {
        sudo::escalate_if_needed().unwrap();

        let io = IoInterface::new().unwrap();

        if let Some(webcam) = &io.webcam {
            // Check webcam
            webcam.set_webcam(false).unwrap();
            assert_eq!(webcam.get_webcam().unwrap(), false);

            webcam.set_webcam(true).unwrap();
            assert_eq!(webcam.get_webcam().unwrap(), true);
        }

        let device = &io.device;
        // Set performance profile
        device.set_odm_performance_profile("quiet").unwrap();

        // Get temperatures
        assert!(20 < device.get_fan_temperature(0).unwrap());
        assert!(matches!(
            device.get_fan_temperature(2).unwrap_err(),
            IoctlError::DevNotAvailable
        ));

        // Check fans
        device.set_fan_speed_percent(0, 100).unwrap();
        assert_eq!(device.get_fan_speed_percent(0).unwrap(), 100);

        device.set_fan_speed_percent(0, 50).unwrap();
        assert_eq!(device.get_fan_speed_percent(0).unwrap(), 50);

        device.set_fan_speed_percent(0, 10).unwrap();
        assert_eq!(device.get_fan_speed_percent(0).unwrap(), 10);

        device.set_fan_speed_percent(0, 100).unwrap();
        assert_eq!(device.get_fan_speed_percent(0).unwrap(), 100);
    }
}
