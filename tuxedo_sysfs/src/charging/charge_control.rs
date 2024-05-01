use std::io;

use crate::sysfs_util::{
    r_file, read_int_list, read_path_to_int_list, read_path_to_string, read_to_string,
};

use super::BatteryChargeControl;

const SYSFS_POWER_SUPPLY_PATH: &str = "/sys/class/power_supply";
const TYPE: &str = "/type";
const CHARGE_TYPE: &str = "/charge_type";
const START_THRESHOLD: &str = "/charge_control_start_threshold";
const END_THRESHOLD: &str = "/charge_control_end_threshold";
const AVAILABLE_START_THRESHOLDS: &str = "/charge_control_start_available_thresholds";
const AVAILABLE_END_THRESHOLDS: &str = "/charge_control_end_available_thresholds";

impl BatteryChargeControl {
    pub async fn new(
        name: String,
        available_start_thresholds: Option<Vec<u32>>,
        available_end_thresholds: Option<Vec<u32>>,
        start_threshold_file: tokio_uring::fs::File,
        end_threshold_file: tokio_uring::fs::File,
        charge_type_file: tokio_uring::fs::File,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            name,
            available_start_thresholds,
            available_end_thresholds,
            start_threshold_file,
            end_threshold_file,
            charge_type_file,
        })
    }

    pub async fn new_first_battery() -> Result<Option<Self>, io::Error> {
        let mut dirs = tokio::fs::read_dir(SYSFS_POWER_SUPPLY_PATH).await?;
        while let Some(dir) = dirs.next_entry().await? {
            let path = dir.path();

            let file_name = path
                .file_name()
                .expect("the sysfs path must have a last segment");

            let type_path = path.join(TYPE);
            if let Ok(typ) = read_path_to_string(type_path).await {
                // not a battery, uninteresting
                if typ.trim() != "Battery" {
                    continue;
                }
            } else {
                tracing::warn!("Type file can't be read: {:?}", file_name);
                continue;
            }

            let start_threshold_file =
                if let Ok(start_threshold_file) = r_file(path.join(START_THRESHOLD)).await {
                    start_threshold_file
                } else {
                    // thresholds not supported
                    continue;
                };
            let end_threshold_file =
                if let Ok(end_threshold_file) = r_file(path.join(END_THRESHOLD)).await {
                    end_threshold_file
                } else {
                    // thresholds not supported
                    continue;
                };
            let charge_type_file =
                if let Ok(charge_type_file) = r_file(path.join(CHARGE_TYPE)).await {
                    charge_type_file
                } else {
                    // thresholds not supported
                    continue;
                };

            let available_start_thresholds_file = path.join(AVAILABLE_START_THRESHOLDS);
            let available_start_thresholds = read_path_to_int_list(available_start_thresholds_file)
                .await
                .ok();

            let available_end_thresholds_file = path.join(AVAILABLE_END_THRESHOLDS);
            let available_end_thresholds = read_path_to_int_list(available_end_thresholds_file)
                .await
                .ok();

            let name = file_name.to_string_lossy().into_owned();

            // TODO: if there is more than 1 battery in the system,
            // there is no guarantee which one is returned
            return Ok(Some(
                BatteryChargeControl::new(
                    name,
                    available_start_thresholds,
                    available_end_thresholds,
                    start_threshold_file,
                    end_threshold_file,
                    charge_type_file,
                )
                .await?,
            ));
        }

        Ok(None)
    }

    pub async fn get_start_threshold(&mut self) -> Result<u32, io::Error> {
        Ok(*read_int_list(&mut self.start_threshold_file)
            .await?
            .first()
            .expect("start threshold file returns a value on read"))
    }

    pub async fn get_end_threshold(&mut self) -> Result<u32, io::Error> {
        Ok(*read_int_list(&mut self.end_threshold_file)
            .await?
            .first()
            .expect("end threshold file returns a value on read"))
    }

    pub async fn get_charge_type(&mut self) -> Result<String, io::Error> {
        Ok(read_to_string(&mut self.charge_type_file)
            .await?
            .trim()
            .to_owned())
    }
}
