use serde::{de::DeserializeOwned, Serialize};
use std::ffi::OsString;
use zbus::fdo;

pub fn normalize_json_path(base_path: &str, name: &str) -> fdo::Result<String> {
    // Make sure the name doesn't contain any illegal characters.
    if name.contains('/') {
        Err(fdo::Error::InvalidArgs(format!(
            "Can't use '/' in profile names: `{name}`"
        )))
    } else if name.contains('.') {
        Err(fdo::Error::InvalidArgs(format!(
            "Can't use '.' in profile names: `{name}`"
        )))
    } else if let Err(err) = OsString::try_from(name) {
        Err(fdo::Error::InvalidArgs(format!(
            "Can't convert `{name}` to OS string: `{err}`"
        )))
    } else if base_path.is_empty() {
        Ok(format!("{name}.json"))
    } else {
        let base_path = base_path.trim().trim_end_matches('/');
        Ok(format!("{base_path}/{name}.json"))
    }
}

pub async fn write_file(base_path: &str, name: &str, data: &[u8]) -> Result<(), fdo::Error> {
    tokio::fs::write(normalize_json_path(base_path, name)?, data)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub async fn write_json<T: Serialize>(
    base_path: &str,
    name: &str,
    data: &T,
) -> Result<(), fdo::Error> {
    let data =
        serde_json::to_string_pretty(data).map_err(|err| fdo::Error::Failed(err.to_string()))?;
    write_file(base_path, name, data.as_bytes()).await
}

pub async fn read_file(base_path: &str, name: &str) -> Result<String, fdo::Error> {
    tokio::fs::read_to_string(normalize_json_path(base_path, name)?)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub async fn read_json<T: DeserializeOwned>(base_path: &str, name: &str) -> Result<T, fdo::Error> {
    let data = read_file(base_path, name).await?;
    serde_json::from_str(&data).map_err(|err| fdo::Error::Failed(err.to_string()))
}

pub async fn remove_file(base_path: &str, name: &str) -> Result<(), fdo::Error> {
    tokio::fs::remove_file(normalize_json_path(base_path, name)?)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub async fn move_file(base_path: &str, from: &str, to: &str) -> Result<(), fdo::Error> {
    tokio::fs::rename(
        normalize_json_path(base_path, from)?,
        normalize_json_path(base_path, to)?,
    )
    .await
    .map_err(|err| fdo::Error::IOError(err.to_string()))
}

pub async fn get_profiles(base_path: &str) -> fdo::Result<Vec<String>> {
    let mut dir_entries = tokio::fs::read_dir(base_path)
        .await
        .map_err(|err| fdo::Error::IOError(err.to_string()))?;

    let mut entries = Vec::new();
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
        if entry
            .file_type()
            .await
            .map(|f| f.is_file())
            .unwrap_or_default()
        {
            match entry.file_name().into_string() {
                Ok(file_name) => {
                    if file_name.contains(".json") {
                        if !file_name.contains("active_profile") {
                            entries.push(file_name.replace(".json", ""))
                        }
                    } else {
                        tracing::warn!("Unknown file type (expected JSON): `{:?}`", entry.path());
                    }
                }
                Err(_) => {
                    tracing::warn!("Couldn't convert file name to UTF8: `{:?}`", entry.path());
                }
            }
        }
    }

    Ok(entries)
}
