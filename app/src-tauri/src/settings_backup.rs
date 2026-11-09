use crate::{
    media_contract::{AppError, ErrorCode},
    media_inspection::{inspect_folder, resolve_path},
    output_plan::with_output_extension,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use tempfile::NamedTempFile;

const MAX_BACKUP_BYTES: u64 = 1024 * 1024;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Palette {
    Purple,
    Blue,
    Green,
    Pink,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedSettings {
    version: u8,
    theme: Theme,
    palette: Palette,
    output_folder: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Backup {
    application: String,
    schema_version: u8,
    settings: SavedSettings,
}

impl Backup {
    fn validate(&self) -> Result<(), ErrorCode> {
        if self.application != "LpsToolkit"
            || self.schema_version != 1
            || self.settings.version != 1
            || self.settings.output_folder.len() > 32768
        {
            return Err(ErrorCode::InvalidSettingsBackup);
        }
        Ok(())
    }
}

fn write_backup(path: &Path, settings: SavedSettings) -> Result<(), ErrorCode> {
    let backup = Backup {
        application: "LpsToolkit".into(),
        schema_version: 1,
        settings,
    };
    backup.validate()?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ErrorCode::InvalidPath)?;
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("json"))
    {
        return Err(ErrorCode::InvalidSettingsBackup);
    }
    with_output_extension(name, ".json")?;
    let folder = inspect_folder(path.parent().ok_or(ErrorCode::InvalidPath)?)?;
    let mut temporary = NamedTempFile::new_in(&folder).map_err(|_| ErrorCode::OutputWriteFailed)?;
    let bytes = serde_json::to_vec_pretty(&backup).map_err(|_| ErrorCode::InvalidSettingsBackup)?;
    temporary
        .write_all(&bytes)
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
    temporary
        .persist(Path::new(&folder).join(name))
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
    Ok(())
}

fn read_backup(path: &Path) -> Result<SavedSettings, ErrorCode> {
    let path = resolve_path(path)?;
    let file = File::open(path).map_err(|_| ErrorCode::Unreadable)?;
    let mut bytes = Vec::new();
    file.take(MAX_BACKUP_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ErrorCode::Unreadable)?;
    if bytes.len() as u64 > MAX_BACKUP_BYTES {
        return Err(ErrorCode::InvalidSettingsBackup);
    }
    let backup: Backup =
        serde_json::from_slice(&bytes).map_err(|_| ErrorCode::InvalidSettingsBackup)?;
    backup.validate()?;
    Ok(backup.settings)
}

#[tauri::command]
pub async fn export_settings_backup(path: String, settings: SavedSettings) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || write_backup(Path::new(&path), settings))
        .await
        .map_err(|_| ErrorCode::Internal)?
        .map_err(Into::into)
}

#[tauri::command]
pub async fn import_settings_backup(path: String) -> Result<SavedSettings, AppError> {
    tauri::async_runtime::spawn_blocking(move || read_backup(Path::new(&path)))
        .await
        .map_err(|_| ErrorCode::Internal)?
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_rejects_other_apps_and_future_schemas() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("backup.json");
        let original = serde_json::json!({
            "version": 1, "theme": "dark", "palette": "blue", "outputFolder": "C:\\Saída"
        });
        write_backup(&path, serde_json::from_value(original.clone()).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(read_backup(&path).unwrap()).unwrap(),
            original
        );
        let valid: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        for changes in [
            serde_json::json!({ "application": "other" }),
            serde_json::json!({ "schemaVersion": 2 }),
            serde_json::json!({ "settings": { "version": 1, "theme": "unknown", "palette": "blue", "outputFolder": "" } }),
        ] {
            let mut invalid = valid.clone();
            for (key, value) in changes.as_object().unwrap() {
                invalid[key] = value.clone();
            }
            std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
            assert!(matches!(
                read_backup(&path),
                Err(ErrorCode::InvalidSettingsBackup)
            ));
        }
    }
}
