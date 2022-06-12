use tailor_api::keyboard::ColorProfile;
use zbus::fdo;

const KEYBOARD_PATH: &str = "/etc/tux_tailor/keyboard_colors";
const KEYBOARD_ACTIVE_PROFILE: &str = "/etc/tux_tailor/keyboard_colors/active_profile";

pub(crate) fn init_keyboard_directory() {
    std::fs::create_dir_all(KEYBOARD_PATH)
        .expect("Unable to create Tux Tailor configuration directory");
}

fn keyboard_path(name: &str) -> String {
    format!("{KEYBOARD_PATH}/{name}.json")
}

pub(crate) async fn write_active_profile_file(name: &str) -> Result<(), fdo::Error> {
    tokio::fs::write(KEYBOARD_ACTIVE_PROFILE, name)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub(crate) async fn read_active_profile_file() -> Result<String, fdo::Error> {
    tokio::fs::read_to_string(KEYBOARD_ACTIVE_PROFILE)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub(crate) async fn write_keyboard_file(name: &str, data: &[u8]) -> Result<(), fdo::Error> {
    tokio::fs::write(keyboard_path(name), data)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub(crate) async fn read_keyboard_file(name: &str) -> Result<Vec<u8>, fdo::Error> {
    tokio::fs::read(keyboard_path(name))
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub(crate) async fn load_keyboard_colors(name: &str) -> Result<ColorProfile, fdo::Error> {
    // Read file.
    let data = read_keyboard_file(name).await?;

    // Make sure the file has valid data.
    serde_json::from_slice(&data).map_err(|err| fdo::Error::InvalidFileContent(err.to_string()))
}

pub(crate) async fn get_color_profiles() -> fdo::Result<Vec<String>> {
    let mut dir_entries = tokio::fs::read_dir(KEYBOARD_PATH)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))?;

    let mut entries = Vec::new();
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        if entry.file_type().await.unwrap().is_file() {
            let file_name = entry
                .file_name()
                .into_string()
                .expect("Filename isn't valid UTF8");
            if file_name.contains(".json") {
                entries.push(file_name.replace(".json", ""))
            }
        }
    }

    Ok(entries)
}
