use tailor_api::{Color, ColorPoint, ColorProfile, ColorTransition, FanProfilePoint};
use tailor_client::TailorConnection;

#[tokio::test]
async fn test_profiles() {
    let connection = TailorConnection::new().await.unwrap();
    let name = "__test_global_profile";
    let second_name = "__test_global_profile2";

    let fan_rename = "__test_fan_rename";
    let led_rename = "__test_led_rename";

    connection.get_number_of_fans().await.unwrap();
    connection.get_led_devices().await.unwrap();

    let active_name = connection.get_active_global_profile_name().await.unwrap();
    let active_profile = connection.get_global_profile(&active_name).await.unwrap();

    // Add profile
    connection
        .add_global_profile(name, &active_profile)
        .await
        .unwrap();
    // Overwrite profile
    connection
        .add_global_profile(name, &active_profile)
        .await
        .unwrap();
    // Get profile
    assert_eq!(
        connection.get_global_profile(name).await.unwrap(),
        active_profile
    );
    // List should contain name
    assert!(connection
        .list_global_profiles()
        .await
        .unwrap()
        .contains(&name.to_owned()));

    // Test rename
    connection
        .rename_fan_profile(&active_profile.fans[0], fan_rename)
        .await
        .unwrap();
    connection
        .rename_led_profile(&active_profile.leds[0].profile, led_rename)
        .await
        .unwrap();

    let profile_after_rename = connection.get_global_profile(name).await.unwrap();
    assert_eq!(profile_after_rename.fans[0], fan_rename);
    assert_eq!(profile_after_rename.leds[0].profile, led_rename);

    // Undo rename
    connection
        .rename_fan_profile(fan_rename, &active_profile.fans[0])
        .await
        .unwrap();
    connection
        .rename_led_profile(led_rename, &active_profile.leds[0].profile)
        .await
        .unwrap();

    // Make sure that renaming twice doesn't change anything
    assert_eq!(
        connection.get_global_profile(name).await.unwrap(),
        active_profile
    );

    // Rename profile
    connection
        .rename_global_profile(name, second_name)
        .await
        .unwrap();
    // List should contain new name
    assert!(connection
        .list_global_profiles()
        .await
        .unwrap()
        .contains(&second_name.to_owned()));
    // List should not contain old name
    assert!(!connection
        .list_global_profiles()
        .await
        .unwrap()
        .contains(&name.to_owned()));
    // Rename profile again (should fail)
    connection
        .rename_global_profile(name, second_name)
        .await
        .unwrap_err();
    // Remove with old name (should fail)
    connection.remove_global_profile(name).await.unwrap_err();

    // Copy profile to old name
    connection
        .copy_global_profile(second_name, name)
        .await
        .unwrap();
    // Remove with old name
    connection.remove_global_profile(name).await.unwrap();

    // Remove profile
    connection.remove_global_profile(second_name).await.unwrap();
    // Remove profile again (should fail)
    connection
        .remove_global_profile(second_name)
        .await
        .unwrap_err();
}

#[tokio::test]
async fn test_fan() {
    let connection = TailorConnection::new().await.unwrap();
    let name = "__test_fan_profile";
    let second_name = "__test_fan_profile2";

    let profile = vec![
        FanProfilePoint { temp: 30, fan: 20 },
        FanProfilePoint { temp: 70, fan: 100 },
    ];

    // Add profile
    connection.add_fan_profile(name, &profile).await.unwrap();
    // Overwrite profile
    connection.add_fan_profile(name, &profile).await.unwrap();
    // Get profile
    assert_eq!(connection.get_fan_profile(name).await.unwrap(), profile);
    // List should contain name
    assert!(connection
        .list_fan_profiles()
        .await
        .unwrap()
        .contains(&name.to_owned()));

    // Rename profile
    connection
        .rename_fan_profile(name, second_name)
        .await
        .unwrap();
    // List should contain new name
    assert!(connection
        .list_fan_profiles()
        .await
        .unwrap()
        .contains(&second_name.to_owned()));
    // List should not contain old name
    assert!(!connection
        .list_fan_profiles()
        .await
        .unwrap()
        .contains(&name.to_owned()));
    // Rename profile again (should fail)
    connection
        .rename_fan_profile(name, second_name)
        .await
        .unwrap_err();
    // Remove with old name (should fail)
    connection.remove_fan_profile(name).await.unwrap_err();

    // Copy profile to old name
    connection
        .copy_fan_profile(second_name, name)
        .await
        .unwrap();
    // Remove with old name
    connection.remove_fan_profile(name).await.unwrap();

    // Remove profile
    connection.remove_fan_profile(second_name).await.unwrap();
    // Remove profile again (should fail)
    connection
        .remove_fan_profile(second_name)
        .await
        .unwrap_err();
}

#[tokio::test]
async fn test_leds() {
    let connection = TailorConnection::new().await.unwrap();
    let name = "__test_led_profile";
    let second_name = "__test_led_profile2";

    let profile = ColorProfile::Multiple(vec![
        ColorPoint {
            color: Color { r: 0, g: 255, b: 0 },
            transition: ColorTransition::Linear,
            transition_time: 3000,
        },
        ColorPoint {
            color: Color { r: 255, g: 0, b: 0 },
            transition: ColorTransition::Linear,
            transition_time: 3000,
        },
        ColorPoint {
            color: Color { r: 0, g: 0, b: 255 },
            transition: ColorTransition::Linear,
            transition_time: 3000,
        },
    ]);

    // Add profile
    connection.add_led_profile(name, &profile).await.unwrap();
    // Overwrite profile
    connection.add_led_profile(name, &profile).await.unwrap();
    // Get profile
    assert_eq!(connection.get_led_profile(name).await.unwrap(), profile);
    // List should contain name
    assert!(connection
        .list_led_profiles()
        .await
        .unwrap()
        .contains(&name.to_owned()));

    // Rename profile
    connection
        .rename_led_profile(name, second_name)
        .await
        .unwrap();
    // List should contain new name
    assert!(connection
        .list_led_profiles()
        .await
        .unwrap()
        .contains(&second_name.to_owned()));
    // List should not contain old name
    assert!(!connection
        .list_led_profiles()
        .await
        .unwrap()
        .contains(&name.to_owned()));
    // Rename profile again (should fail)
    connection
        .rename_led_profile(name, second_name)
        .await
        .unwrap_err();
    // Remove with old name (should fail)
    connection.remove_led_profile(name).await.unwrap_err();

    // Copy profile to old name
    connection
        .copy_led_profile(second_name, name)
        .await
        .unwrap();
    // Remove with old name
    connection.remove_led_profile(name).await.unwrap();

    // Remove profile
    connection.remove_led_profile(second_name).await.unwrap();
    // Remove profile again (should fail)
    connection
        .remove_led_profile(second_name)
        .await
        .unwrap_err();
}
