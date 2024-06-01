use std::fmt::Debug;

fn print_value<T: Debug>(property: &str, value: &T) {
    println!("[OK]    {property}: {value:?}");
}

fn print_info(property: &str) {
    println!("[INFO]  {property}");
}

fn print_err<T: Debug>(property: &str, value: &T) {
    println!("[ERR]   {property}: {value:?}");
}

fn print_fatal<T: Debug>(property: &str, value: &T) {
    println!("[FATAL] {property}: {value:?}");
}

fn print_result<T: Debug, E: Debug>(property: &str, value: &Result<T, E>) {
    match value {
        Ok(ok) => print_value(property, ok),
        Err(err) => print_err(property, err),
    }
}

fn main() {
    sudo::escalate_if_needed().unwrap();

    ioctl();

    tokio_uring::start(sysfs());
}

fn ioctl() {
    let io = tuxedo_ioctl::hal::IoInterface::new();
    let io = match io {
        Ok(io) => io,
        Err(err) => {
            print_fatal("Connecting to ioctl interface failed", &err);
            return;
        }
    };

    print_value("Module version", &io.module_version);

    print_result("Device interface ID", &io.device.device_interface_id_str());
    print_result("Model ID", &io.device.device_model_id_str());

    print_result(
        "Available ODM performance profiles",
        &io.device.get_available_odm_performance_profiles(),
    );
    print_result(
        "Default ODM performance profile",
        &io.device.get_default_odm_performance_profile(),
    );

    print_value("Number of fans", &io.device.get_number_fans());
    let fan_temperatures = (0..io.device.get_number_fans())
        .map(|fan| io.device.get_fan_temperature(fan))
        .collect::<Result<Vec<_>, _>>();
    let fan_speeds = (0..io.device.get_number_fans())
        .map(|fan| io.device.get_fan_speed_percent(fan))
        .collect::<Result<Vec<_>, _>>();
    print_result("Fan temperatures [°C]", &fan_temperatures);
    print_result("Fan speeds [%]", &fan_speeds);
    print_result("Fan min speed [%]", &io.device.get_fans_min_speed());

    if let Some(webcam) = &io.webcam {
        print_result("Webcam enabled", &webcam.get_webcam());
    } else {
        print_info("Webcam control is not available");
    }

    if let Some(tdp) = &io.tdp {
        let number_of_tdp_devices = tdp.get_number_tdps();
        print_result("number_of_tdp_devices", &number_of_tdp_devices);
        print_result("tdp_descriptors", &tdp.get_tdp_descriptors());

        let number_of_tdp_devices = number_of_tdp_devices.unwrap_or_default();
        let tdps = (0..number_of_tdp_devices)
            .map(|tdp_index| tdp.get_tdp(tdp_index))
            .collect::<Result<Vec<_>, _>>();
        let max_tdps = (0..number_of_tdp_devices)
            .map(|tdp_index| tdp.get_tdp_max(tdp_index))
            .collect::<Result<Vec<_>, _>>();
        let min_tdps = (0..number_of_tdp_devices)
            .map(|tdp_index| tdp.get_tdp_min(tdp_index))
            .collect::<Result<Vec<_>, _>>();

        print_result("tdps", &tdps);
        print_result("max_tdps", &max_tdps);
        print_result("min_tdps", &min_tdps);
    } else {
        print_info("TDP control is not available");
    }
}

async fn sysfs() {
    let led_collection = tuxedo_sysfs::led::Collection::new().await.unwrap();
    print_value("Number of LED devices", &led_collection.len());

    for (idx, mut controller) in led_collection.into_inner().into_iter().enumerate() {
        print_value("LED device number", &idx);
        print_value("LED device name", &controller.device_name());
        print_value("LED device function", &controller.function());
        print_value("LED mode", &controller.mode());
        print_value("LED device color", &controller.get_color().await);
    }

    let charging_profile = tuxedo_sysfs::charging::ChargingProfile::new()
        .await
        .unwrap();
    if let Some(mut charging_profile) = charging_profile {
        print_value(
            "Available charging profiles",
            &charging_profile.available_charging_profiles,
        );
        print_value(
            "Current charging profile",
            &charging_profile.get_charging_profile().await.unwrap(),
        );
    } else {
        print_info("Charging profile control is not available");
    }

    let charging_priority = tuxedo_sysfs::charging::ChargingPriority::new()
        .await
        .unwrap();
    if let Some(mut charging_priority) = charging_priority {
        print_value(
            "Available charging priorities",
            &charging_priority.available_charging_priorities,
        );
        print_value(
            "Current charging priority",
            &charging_priority.get_charging_priority().await.unwrap(),
        );
    } else {
        print_info("Charging priority control is not available");
    }

    let first_battery = tuxedo_sysfs::charging::BatteryChargeControl::new_first_battery()
        .await
        .unwrap();
    if let Some(mut first_battery) = first_battery {
        print_value("Battery name", &first_battery.name);
        print_value(
            "Battery charge type",
            &first_battery.get_charge_type().await.unwrap(),
        );
        if let Some(available_start_thresholds) = &first_battery.available_start_thresholds {
            print_value(
                "Available charge control start thresholds",
                available_start_thresholds,
            );
        } else {
            print_info("Available charge control start thresholds not available");
        }
        print_value(
            "Battery start threshold",
            &first_battery.get_start_threshold().await.unwrap(),
        );
        if let Some(available_end_thresholds) = &first_battery.available_end_thresholds {
            print_value(
                "Available charge control end thresholds",
                available_end_thresholds,
            );
        } else {
            print_info("Available charge control end thresholds not available");
        }
        print_value(
            "Battery end threshold",
            &first_battery.get_end_threshold().await.unwrap(),
        );
    } else {
        print_info("Charge control for start/end thresholds is not available");
    }
}
