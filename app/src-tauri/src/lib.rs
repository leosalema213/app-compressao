mod app_updates;
mod av_processing;
mod downloader;
mod image_processing;
mod jobs;
mod media_contract;
mod media_inspection;
mod media_probe;
mod media_tools;
mod output_plan;
mod process_runner;
mod settings_backup;

#[derive(serde::Serialize)]
struct AppDiagnostic {
    version: String,
}

#[tauri::command]
fn get_app_diagnostic(app: tauri::AppHandle) -> AppDiagnostic {
    AppDiagnostic {
        version: app.package_info().version.to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(image_processing::LastImageOutputFolder::default())
        .manage(jobs::JobQueue::default())
        .manage(downloader::DownloadCache::default())
        .manage(downloader::DownloadOutputs::default())
        .manage(app_updates::AppUpdates::default())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_app_diagnostic,
            media_inspection::inspect_images,
            media_inspection::validate_output_folder,
            output_plan::preview_image_outputs,
            image_processing::compress_images,
            image_processing::preview_image_conversion,
            image_processing::open_image_output_folder,
            jobs::cancel_image_job,
            jobs::pause_download_job,
            media_tools::diagnose_media_tools,
            media_probe::inspect_av_file,
            av_processing::transcode_av,
            downloader::inspect_download,
            downloader::download_media,
            downloader::open_download_output,
            app_updates::check_app_update,
            app_updates::download_app_update,
            app_updates::install_app_update,
            settings_backup::export_settings_backup,
            settings_backup::import_settings_backup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::AppDiagnostic;

    #[test]
    fn diagnostic_serializes_version_for_frontend() {
        let diagnostic = AppDiagnostic {
            version: "0.1.0".to_owned(),
        };

        assert_eq!(
            serde_json::to_value(diagnostic).unwrap(),
            serde_json::json!({ "version": "0.1.0" })
        );
    }
}
