use crate::{
    jobs::{JobQueue, JobStatus, JobUpdate},
    media_contract::{AppError, ErrorCode, ImageFormat},
    media_inspection::{inspect_folder, inspect_image},
    media_probe::{probe_media, StreamKind},
    media_tools::locate,
    output_plan::{next_available_name, with_output_extension},
    process_runner::{run_process, run_process_streaming, ProcessError},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    io::ErrorKind,
    ops::Range,
    path::Path,
    time::Duration,
};
use tauri::Manager;

#[derive(Deserialize, Serialize)]
pub struct DownloadInfo {
    title: String,
    duration: Option<f64>,
    #[serde(default, skip_deserializing)]
    heights: Vec<u32>,
    #[serde(default, skip_deserializing)]
    subtitle_languages: Vec<String>,
    #[serde(default, skip_deserializing)]
    has_thumbnail: bool,
    #[serde(default, skip_deserializing)]
    playlist_size: Option<usize>,
    #[serde(default, skip_serializing)]
    entries: Vec<Option<PlaylistEntry>>,
    #[serde(skip_serializing)]
    playlist_count: Option<usize>,
    #[serde(default, skip_serializing)]
    subtitles: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing)]
    thumbnail: Option<String>,
    #[serde(default, skip_serializing)]
    formats: Vec<DownloadFormat>,
    #[serde(rename = "_type", skip_serializing)]
    entry_type: Option<String>,
    #[serde(skip_serializing)]
    is_live: Option<bool>,
}

#[derive(Deserialize)]
struct PlaylistEntry {
    webpage_url: Option<String>,
    url: Option<String>,
    #[serde(default)]
    formats: Vec<PlaylistFormat>,
}

#[derive(Deserialize)]
struct PlaylistFormat {
    url: Option<String>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadKind {
    Video,
    Audio,
    Thumbnail,
    Subtitles,
}

#[derive(Deserialize)]
struct DownloadFormat {
    ext: Option<String>,
    height: Option<u32>,
    vcodec: Option<String>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadQuality {
    Best,
    P1080,
    P720,
    P480,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadOptions {
    quality: Option<DownloadQuality>,
    language: Option<String>,
    confirmed_count: Option<usize>,
}

impl DownloadQuality {
    fn height(self) -> Option<u32> {
        match self {
            Self::Best => None,
            Self::P1080 => Some(1080),
            Self::P720 => Some(720),
            Self::P480 => Some(480),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    file_name: String,
    bytes: u64,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum DownloadReport {
    Single(DownloadResult),
    Playlist {
        files: Vec<DownloadResult>,
        failed: usize,
        cancelled: usize,
    },
}

fn validated_url(input: &str) -> Result<tauri::Url, ErrorCode> {
    let url = tauri::Url::parse(input.trim()).map_err(|_| ErrorCode::InvalidDownloadUrl)?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(ErrorCode::InvalidDownloadUrl);
    }
    Ok(url)
}

fn common_args() -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        "--ignore-config",
        "--no-plugin-dirs",
        "--no-remote-components",
        "--no-playlist",
        "--socket-timeout",
        "12",
        "--retries",
        "1",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    if let Some(node) = locate("node") {
        let mut runtime = OsString::from("node:");
        runtime.push(node);
        args.extend([
            OsString::from("--no-js-runtimes"),
            "--js-runtimes".into(),
            runtime,
        ]);
    }
    args
}

fn inspect(url: &tauri::Url, cancelled: impl Fn() -> bool) -> Result<DownloadInfo, ErrorCode> {
    let executable = locate("yt-dlp").ok_or(ErrorCode::DownloaderMissing)?;
    let mut args = common_args();
    args.extend(
        [
            "--dump-single-json",
            "--skip-download",
            "--flat-playlist",
            "--playlist-end",
            "21",
            "--",
        ]
        .into_iter()
        .map(OsString::from),
    );
    args.push(url.as_str().into());
    let output = run_process(&executable, &args, Duration::from_secs(45), cancelled)
        .map_err(download_error)?;
    if !output.status.success() || output.truncated {
        return Err(ErrorCode::DownloadFailed);
    }
    let mut info: DownloadInfo =
        serde_json::from_slice(&output.stdout).map_err(|_| ErrorCode::DownloadFailed)?;
    if info.is_live == Some(true) {
        return Err(ErrorCode::DownloadUnsupported);
    }
    if matches!(info.entry_type.as_deref(), Some("playlist" | "multi_video")) {
        let count = info.entries.len();
        if count == 0 || count > 20 || info.playlist_count.is_some_and(|total| total > 20) {
            return Err(ErrorCode::DownloadUnsupported);
        }
        info.playlist_size = Some(count);
        return Ok(info);
    }
    info.heights = [1080, 720, 480]
        .into_iter()
        .filter(|height| {
            info.formats.iter().any(|format| {
                format.ext.as_deref() == Some("mp4")
                    && format.height == Some(*height)
                    && format.vcodec.as_deref() != Some("none")
            })
        })
        .collect();
    info.has_thumbnail = info.thumbnail.as_ref().is_some_and(|url| !url.is_empty());
    info.subtitle_languages = info
        .subtitles
        .keys()
        .filter(|language| {
            !language.is_empty()
                && language.len() <= 20
                && language
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
        .cloned()
        .collect();
    info.subtitle_languages.sort_unstable();
    Ok(info)
}

#[tauri::command]
pub async fn inspect_download(url: String) -> Result<DownloadInfo, AppError> {
    let url = validated_url(&url)?;
    tauri::async_runtime::spawn_blocking(move || inspect(&url, || false))
        .await
        .map_err(|_| ErrorCode::Internal)?
        .map_err(Into::into)
}

#[tauri::command]
pub async fn download_media(
    url: String,
    folder: String,
    kind: DownloadKind,
    authorized: bool,
    options: Option<DownloadOptions>,
    progress: Option<tauri::ipc::JavaScriptChannelId>,
    webview: tauri::Webview,
) -> Result<DownloadReport, AppError> {
    if !authorized {
        return Err(ErrorCode::DownloadNotAuthorized.into());
    }
    let url = validated_url(&url)?;
    let options = options.unwrap_or_default();
    let app = webview.app_handle().clone();
    let queue = app.state::<JobQueue>();
    let job = queue.register(
        100,
        progress.map(|id| id.channel_on::<_, JobUpdate>(webview)),
    );
    let execution = queue.execution.clone().lock_owned().await;
    if !job.start() {
        job.finish(JobStatus::Cancelled, 0);
        return Err(ErrorCode::JobCancelled.into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let _execution = execution;
        let result = download(&url, &folder, kind, &options, &job);
        let (status, completed) = match &result {
            Ok(DownloadReport::Playlist { cancelled, .. }) if *cancelled > 0 => {
                (JobStatus::Cancelled, 0)
            }
            Ok(_) => (JobStatus::Finished, 100),
            Err(ErrorCode::JobCancelled) => (JobStatus::Cancelled, 0),
            Err(_) => (JobStatus::Failed, 0),
        };
        job.finish(status, completed);
        result
    })
    .await
    .map_err(|_| ErrorCode::Internal)?
    .map_err(Into::into)
}

fn download(
    url: &tauri::Url,
    folder: &str,
    kind: DownloadKind,
    options: &DownloadOptions,
    job: &crate::jobs::Job,
) -> Result<DownloadReport, ErrorCode> {
    let info = inspect(url, || job.cancelled())?;
    let folder = inspect_folder(Path::new(folder))?;
    let folder = Path::new(&folder);
    let Some(count) = info.playlist_size else {
        if options.confirmed_count.is_some() {
            return Err(ErrorCode::DownloadPlaylistConfirmation);
        }
        return download_one(url, folder, kind, options, &info, job, 0..99)
            .map(DownloadReport::Single);
    };
    if options.confirmed_count != Some(count) {
        return Err(ErrorCode::DownloadPlaylistConfirmation);
    }
    if !matches!(kind, DownloadKind::Video | DownloadKind::Audio) {
        return Err(ErrorCode::DownloadUnsupported);
    }
    let mut files = Vec::new();
    let mut failed = 0;
    for (index, entry) in info.entries.into_iter().enumerate() {
        if job.cancelled() {
            break;
        }
        let range = (index * 99 / count)..((index + 1) * 99 / count);
        let end = range.end;
        let result = (|| {
            let entry = entry.ok_or(ErrorCode::DownloadFailed)?;
            let link = [entry.url.as_deref(), entry.webpage_url.as_deref()]
                .into_iter()
                .flatten()
                .chain(
                    entry
                        .formats
                        .iter()
                        .filter_map(|format| format.url.as_deref()),
                )
                .filter_map(|candidate| validated_url(candidate).ok())
                .find(|candidate| candidate != url)
                .ok_or(ErrorCode::DownloadFailed)?;
            let item = inspect(&link, || job.cancelled())?;
            if item.playlist_size.is_some() {
                return Err(ErrorCode::DownloadUnsupported);
            }
            download_one(
                &link,
                folder,
                kind,
                &DownloadOptions::default(),
                &item,
                job,
                range,
            )
        })();
        match result {
            Ok(file) => files.push(file),
            Err(ErrorCode::JobCancelled) => break,
            Err(_) => failed += 1,
        }
        job.publish(end);
    }
    let cancelled = count - files.len() - failed;
    Ok(DownloadReport::Playlist {
        files,
        failed,
        cancelled,
    })
}

fn download_one(
    url: &tauri::Url,
    folder: &Path,
    kind: DownloadKind,
    options: &DownloadOptions,
    info: &DownloadInfo,
    job: &crate::jobs::Job,
    range: Range<usize>,
) -> Result<DownloadResult, ErrorCode> {
    let executable = locate("yt-dlp").ok_or(ErrorCode::DownloaderMissing)?;
    let ffmpeg = locate("ffmpeg").ok_or(ErrorCode::MediaToolMissing)?;
    let stage = tempfile::tempdir_in(folder).map_err(|_| ErrorCode::OutputWriteFailed)?;
    let mut args = common_args();
    args.extend(
        [
            "--playlist-items",
            "1",
            "--no-simulate",
            "--newline",
            "--progress",
            "--progress-delta",
            "0.25",
            "--progress-template",
            "download:LPSTK:%(progress._percent_str)s",
            "--ffmpeg-location",
        ]
        .into_iter()
        .map(OsString::from),
    );
    args.push(ffmpeg.into_os_string());
    args.push("--output".into());
    // Percent signs in the directory are literal; only the extension is a template field.
    args.push(
        format!(
            "{}/media.%(ext)s",
            stage.path().to_string_lossy().replace('%', "%%")
        )
        .into(),
    );
    let (extension, staged_name) = match kind {
        DownloadKind::Video => {
            let selector = match options.quality.unwrap_or(DownloadQuality::Best).height() {
                Some(height) if info.heights.contains(&height) => format!(
                    "bv*[ext=mp4][height<={height}]+ba[ext=m4a]/b[ext=mp4][height<={height}]"
                ),
                Some(_) => return Err(ErrorCode::DownloadQualityUnavailable),
                None => "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]".to_owned(),
            };
            args.extend([
                OsString::from("--format"),
                selector.into(),
                "--merge-output-format".into(),
                "mp4".into(),
            ]);
            (".mp4", "media.mp4".to_owned())
        }
        DownloadKind::Audio => {
            args.extend(
                [
                    "--extract-audio",
                    "--audio-format",
                    "mp3",
                    "--audio-quality",
                    "128K",
                ]
                .into_iter()
                .map(OsString::from),
            );
            (".mp3", "media.mp3".to_owned())
        }
        DownloadKind::Thumbnail if info.has_thumbnail => {
            args.extend(
                [
                    "--skip-download",
                    "--write-thumbnail",
                    "--convert-thumbnails",
                    "png",
                ]
                .into_iter()
                .map(OsString::from),
            );
            (".png", "media.png".to_owned())
        }
        DownloadKind::Subtitles => {
            let language = options
                .language
                .as_deref()
                .filter(|language| info.subtitle_languages.iter().any(|item| item == language))
                .ok_or(ErrorCode::DownloadExtraUnavailable)?;
            args.extend(
                [
                    "--skip-download",
                    "--write-subs",
                    "--sub-langs",
                    language,
                    "--sub-format",
                    "vtt/best",
                    "--convert-subs",
                    "vtt",
                ]
                .into_iter()
                .map(OsString::from),
            );
            (".vtt", format!("media.{language}.vtt"))
        }
        DownloadKind::Thumbnail => return Err(ErrorCode::DownloadExtraUnavailable),
    };
    args.push("--".into());
    args.push(url.as_str().into());
    let mut pending = Vec::new();
    let output = run_process_streaming(
        &executable,
        &args,
        Duration::from_secs(3600),
        || job.cancelled(),
        |bytes| {
            for &byte in bytes {
                if byte == b'\n' {
                    if let Some(value) = std::str::from_utf8(&pending)
                        .ok()
                        .and_then(|line| line.strip_prefix("LPSTK:"))
                        .and_then(|value| value.trim().trim_end_matches('%').parse::<f64>().ok())
                        .filter(|value| value.is_finite())
                    {
                        job.publish(
                            range.start
                                + ((range.end - range.start) as f64 * value.clamp(0.0, 100.0)
                                    / 100.0) as usize,
                        );
                    }
                    pending.clear();
                } else if pending.len() < 256 {
                    pending.push(byte);
                }
            }
        },
    )
    .map_err(download_error)?;
    if !output.status.success() {
        return Err(ErrorCode::DownloadFailed);
    }
    let staged = stage.path().join(staged_name);
    let bytes = match kind {
        DownloadKind::Video => probe_media(&staged, StreamKind::Video).map(|item| item.size_bytes),
        DownloadKind::Audio => probe_media(&staged, StreamKind::Audio).map(|item| item.size_bytes),
        DownloadKind::Thumbnail => {
            let item = inspect_image(&staged).map_err(|_| ErrorCode::OutputWriteFailed)?;
            crate::image_processing::decode(&staged, ImageFormat::Png).map(|_| item.size_bytes)
        }
        DownloadKind::Subtitles => validate_webvtt(&staged),
    }
    .map_err(|_| ErrorCode::OutputWriteFailed)?;
    let mut temporary =
        tempfile::TempPath::try_from_path(staged).map_err(|_| ErrorCode::OutputWriteFailed)?;
    let title: String = info
        .title
        .chars()
        .filter(|ch| !ch.is_control() && !"<>:\"/\\|?*".contains(*ch))
        .take(90)
        .collect();
    let title = title.trim_matches([' ', '.']);
    let name = with_output_extension(&format!("{title}.tmp"), extension)
        .unwrap_or_else(|_| format!("download{extension}"));
    let preferred = if title.is_empty() {
        format!("download{extension}")
    } else {
        name
    };
    let mut reserved = HashSet::new();
    let file_name = loop {
        if job.cancelled() {
            return Err(ErrorCode::JobCancelled);
        }
        let candidate = next_available_name(folder, &preferred, &mut reserved)?;
        match temporary.persist_noclobber(folder.join(&candidate)) {
            Ok(()) => break candidate,
            Err(error) if error.error.kind() == ErrorKind::AlreadyExists => temporary = error.path,
            Err(_) => return Err(ErrorCode::OutputWriteFailed),
        }
    };
    Ok(DownloadResult { file_name, bytes })
}

fn validate_webvtt(path: &Path) -> Result<u64, ErrorCode> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|_| ErrorCode::OutputWriteFailed)?;
    let size = file
        .metadata()
        .map_err(|_| ErrorCode::OutputWriteFailed)?
        .len();
    if size == 0 || size > 4 * 1024 * 1024 {
        return Err(ErrorCode::OutputWriteFailed);
    }
    let mut text = String::new();
    file.take(4 * 1024 * 1024)
        .read_to_string(&mut text)
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
    if !text
        .trim_start_matches('\u{feff}')
        .lines()
        .next()
        .is_some_and(|line| line == "WEBVTT" || line.starts_with("WEBVTT "))
    {
        return Err(ErrorCode::OutputWriteFailed);
    }
    Ok(size)
}

fn download_error(error: ProcessError) -> ErrorCode {
    match error {
        ProcessError::Cancelled => ErrorCode::JobCancelled,
        ProcessError::TimedOut => ErrorCode::ProcessTimedOut,
        ProcessError::Unavailable => ErrorCode::DownloadFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_links_and_rejects_credentials_and_local_file_urls() {
        assert!(validated_url(" https://example.com/video ").is_ok());
        for input in [
            "file:///C:/video.mp4",
            "https://name:pass@example.com/",
            "--help",
        ] {
            assert_eq!(validated_url(input), Err(ErrorCode::InvalidDownloadUrl));
        }
    }
}
