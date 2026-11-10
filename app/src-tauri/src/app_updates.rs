use crate::{
    jobs::JobQueue,
    media_contract::{AppError, ErrorCode},
};
use serde::Serialize;
use std::time::Duration;
use tauri::{ipc::Channel, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Default)]
struct UpdateSession {
    update: Option<Update>,
    downloaded: Option<Vec<u8>>,
}

#[derive(Default)]
pub struct AppUpdates(tauri::async_runtime::Mutex<UpdateSession>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    current_version: String,
    version: Option<String>,
    notes: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    received: u64,
    total: Option<u64>,
}

fn valid_release_url(version: &str, url: &str) -> bool {
    !version.contains('-') && ["Pacton", "LpsToolkit"].iter().any(|name| url == format!(
        "https://github.com/leosalema213/app-compressao/releases/download/v{version}/{name}_{version}_x64-setup.exe"
    ))
}

#[tauri::command]
pub async fn check_app_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppUpdates>,
) -> Result<UpdateInfo, AppError> {
    let mut session = state.0.try_lock().map_err(|_| ErrorCode::UpdateBusy)?;
    // Keep a verified download available until it is installed or the app closes.
    if let Some(update) = session
        .update
        .as_ref()
        .filter(|_| session.downloaded.is_some())
    {
        return Ok(UpdateInfo {
            current_version: update.current_version.clone(),
            version: Some(update.version.clone()),
            notes: update.body.clone(),
        });
    }
    let pending = app
        .updater_builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|_| ErrorCode::UpdateCheckFailed)?
        .check()
        .await
        .map_err(|error| match error {
            tauri_plugin_updater::Error::ReleaseNotFound => ErrorCode::UpdateChannelUnavailable,
            _ => ErrorCode::UpdateCheckFailed,
        })?;
    if pending
        .as_ref()
        .is_some_and(|update| !valid_release_url(&update.version, update.download_url.as_str()))
    {
        return Err(ErrorCode::UpdateCheckFailed.into());
    }
    let info = UpdateInfo {
        current_version: app.package_info().version.to_string(),
        version: pending.as_ref().map(|update| update.version.clone()),
        notes: pending.as_ref().and_then(|update| update.body.clone()),
    };
    session.update = pending;
    Ok(info)
}

#[tauri::command]
pub async fn download_app_update(
    state: tauri::State<'_, AppUpdates>,
    progress: Channel<UpdateProgress>,
) -> Result<(), AppError> {
    let mut session = state.0.try_lock().map_err(|_| ErrorCode::UpdateBusy)?;
    if session.downloaded.is_some() {
        return Ok(());
    }
    let update = session.update.as_mut().ok_or(ErrorCode::UpdateNotReady)?;
    update.timeout = Some(Duration::from_secs(600));
    let mut received = 0;
    let bytes = update
        .download(
            |size, total| {
                received += size as u64;
                let _ = progress.send(UpdateProgress { received, total });
            },
            || {},
        )
        .await
        .map_err(|_| ErrorCode::UpdateDownloadFailed)?;
    session.downloaded = Some(bytes);
    Ok(())
}

#[tauri::command]
pub fn install_app_update(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppUpdates>,
) -> Result<(), AppError> {
    let session = state.0.try_lock().map_err(|_| ErrorCode::UpdateBusy)?;
    let update = session.update.as_ref().ok_or(ErrorCode::UpdateNotReady)?;
    let bytes = session
        .downloaded
        .as_ref()
        .ok_or(ErrorCode::UpdateNotReady)?;
    let queue = app.state::<JobQueue>();
    let _jobs = queue.lock_for_update().ok_or(ErrorCode::UpdateJobsActive)?;
    update
        .install(bytes)
        .map_err(|_| ErrorCode::UpdateInstallFailed)?;
    // The Windows updater exits and relaunches through NSIS.
    #[cfg(not(windows))]
    app.restart();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::valid_release_url;

    #[test]
    fn accepts_pacton_and_legacy_assets_only_from_the_release_repository() {
        let root = "https://github.com/leosalema213/app-compressao/releases/download/v0.1.2";
        assert!(valid_release_url(
            "0.1.2",
            &format!("{root}/Pacton_0.1.2_x64-setup.exe")
        ));
        assert!(valid_release_url(
            "0.1.2",
            &format!("{root}/LpsToolkit_0.1.2_x64-setup.exe")
        ));
        assert!(!valid_release_url(
            "0.1.2",
            &format!("{root}/Pacton_0.1.1_x64-setup.exe")
        ));
        assert!(!valid_release_url(
            "0.1.2",
            "https://example.com/Pacton_0.1.2_x64-setup.exe"
        ));
        assert!(!valid_release_url(
            "0.1.2-preview.1",
            &format!("{root}/Pacton_0.1.2_x64-setup.exe")
        ));
    }
}
