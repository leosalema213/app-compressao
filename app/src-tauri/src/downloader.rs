use crate::{
    jobs::{DownloadTransfer, Job, JobQueue, JobStatus, JobUpdate},
    media_contract::{AppError, ErrorCode, ImageFormat},
    media_inspection::{inspect_folder, inspect_image},
    media_probe::{probe_media, StreamKind},
    media_tools::locate,
    output_plan::{next_available_name, with_output_extension},
    process_runner::{run_process_streaming, run_process_with_limit, ProcessError},
};
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    ffi::OsString,
    io::ErrorKind,
    ops::Range,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;

type CachedDownload = (tauri::Url, Instant, Arc<DownloadInfo>);

#[derive(Default)]
pub struct DownloadCache(Mutex<Option<CachedDownload>>);

impl DownloadCache {
    fn lookup(
        &self,
        url: &tauri::Url,
        cancelled: impl Fn() -> bool,
    ) -> Result<Arc<DownloadInfo>, ErrorCode> {
        if let Some((_, _, info)) = self
            .0
            .lock()
            .unwrap()
            .as_ref()
            .filter(|(key, time, _)| key == url && time.elapsed() < Duration::from_secs(180))
        {
            return Ok(Arc::clone(info));
        }
        let info = Arc::new(inspect(url, cancelled)?);
        *self.0.lock().unwrap() = Some((url.clone(), Instant::now(), Arc::clone(&info)));
        Ok(info)
    }
}

#[derive(Default)]
pub struct DownloadOutputs(Mutex<HashMap<u64, (PathBuf, Option<PathBuf>)>>);

#[tauri::command]
pub fn open_download_output(
    app: tauri::AppHandle,
    outputs: tauri::State<'_, DownloadOutputs>,
    id: u64,
    file: bool,
) -> Result<(), AppError> {
    let path = outputs
        .0
        .lock()
        .unwrap()
        .get(&id)
        .and_then(|(folder, item)| {
            if file {
                item.clone()
            } else {
                Some(folder.clone())
            }
        })
        .ok_or(ErrorCode::DownloadOutputOpenFailed)?;
    app.opener()
        .open_path(path.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|_| ErrorCode::DownloadOutputOpenFailed.into())
}

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
    #[serde(default, skip_deserializing)]
    cover_url: Option<String>,
    #[serde(default, skip_serializing)]
    entries: Vec<Option<PlaylistEntry>>,
    #[serde(default, skip_serializing)]
    subtitles: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing)]
    thumbnail: Option<String>,
    #[serde(default, skip_serializing)]
    thumbnails: Vec<Thumbnail>,
    #[serde(default, skip_serializing)]
    formats: Vec<DownloadFormat>,
    #[serde(rename = "_type", skip_serializing)]
    entry_type: Option<String>,
    #[serde(skip_serializing)]
    is_live: Option<bool>,
}

#[derive(Deserialize)]
struct PlaylistEntry {
    title: Option<String>,
    webpage_url: Option<String>,
    url: Option<String>,
    thumbnail: Option<String>,
    #[serde(default)]
    thumbnails: Vec<Thumbnail>,
    #[serde(default)]
    formats: Vec<PlaylistFormat>,
}

#[derive(Deserialize)]
struct Thumbnail {
    url: Option<String>,
}

#[derive(Deserialize)]
struct PlaylistFormat {
    url: Option<String>,
}

#[derive(Default)]
struct DownloadItem<'a> {
    title: &'a str,
    heights: &'a [u32],
    languages: &'a [String],
    has_thumbnail: bool,
}

impl<'a> From<&'a DownloadInfo> for DownloadItem<'a> {
    fn from(info: &'a DownloadInfo) -> Self {
        Self {
            title: &info.title,
            heights: &info.heights,
            languages: &info.subtitle_languages,
            has_thumbnail: info.has_thumbnail,
        }
    }
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
    name_prefix: Option<String>,
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

fn cover_url(thumbnail: Option<&str>, thumbnails: &[Thumbnail]) -> Option<String> {
    thumbnail
        .into_iter()
        .chain(
            thumbnails
                .iter()
                .rev()
                .filter_map(|item| item.url.as_deref()),
        )
        .find_map(|url| validated_url(url).ok().map(|url| url.to_string()))
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
            "--",
        ]
        .into_iter()
        .map(OsString::from),
    );
    args.push(url.as_str().into());
    let output = run_process_with_limit(
        &executable,
        &args,
        Duration::from_secs(120),
        cancelled,
        16 * 1024 * 1024,
    )
    .map_err(download_error)?;
    if !output.status.success() || output.truncated {
        return Err(ErrorCode::DownloadFailed);
    }
    let mut info: DownloadInfo =
        serde_json::from_slice(&output.stdout).map_err(|_| ErrorCode::DownloadFailed)?;
    if info.is_live == Some(true) {
        return Err(ErrorCode::DownloadUnsupported);
    }
    info.cover_url = cover_url(info.thumbnail.as_deref(), &info.thumbnails);
    if matches!(info.entry_type.as_deref(), Some("playlist" | "multi_video")) {
        let count = info.entries.len();
        if count == 0 {
            return Err(ErrorCode::DownloadUnsupported);
        }
        info.playlist_size = Some(count);
        if info.cover_url.is_none() {
            info.cover_url = info
                .entries
                .iter()
                .flatten()
                .find_map(|entry| cover_url(entry.thumbnail.as_deref(), &entry.thumbnails));
        }
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
pub async fn inspect_download(
    url: String,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, AppError> {
    let url = validated_url(&url)?;
    tauri::async_runtime::spawn_blocking(move || {
        let info = app.state::<DownloadCache>().lookup(&url, || false)?;
        serde_json::to_value(info.as_ref()).map_err(|_| ErrorCode::Internal)
    })
    .await
    .map_err(|_| ErrorCode::Internal)?
    .map_err(Into::into)
}

#[tauri::command]
pub async fn download_media(
    url: String,
    folder: String,
    kind: DownloadKind,
    options: Option<DownloadOptions>,
    progress: Option<tauri::ipc::JavaScriptChannelId>,
    webview: tauri::Webview,
) -> Result<DownloadReport, AppError> {
    let url = validated_url(&url)?;
    let mut options = options.unwrap_or_default();
    options.name_prefix = options
        .name_prefix
        .as_deref()
        .map(str::trim)
        .filter(|prefix| !prefix.is_empty())
        .map(str::to_owned);
    if let Some(prefix) = &options.name_prefix {
        if prefix.chars().count() > 90 {
            return Err(ErrorCode::InvalidOutputName.into());
        }
        with_output_extension(&format!("{prefix}.tmp"), ".tmp")?;
    }
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
        let result = (|| {
            let folder = PathBuf::from(inspect_folder(Path::new(&folder))?);
            let info = app
                .state::<DownloadCache>()
                .lookup(&url, || job.cancelled())?;
            let report = download(&url, &folder, kind, &options, &job, &info)?;
            let saved = match &report {
                DownloadReport::Single(item) => Some(Some(folder.join(&item.file_name))),
                DownloadReport::Playlist { files, .. } if !files.is_empty() => Some(None),
                _ => None,
            };
            if let Some(file) = saved {
                app.state::<DownloadOutputs>()
                    .0
                    .lock()
                    .unwrap()
                    .insert(job.id(), (folder, file));
            }
            Ok(report)
        })();
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
    folder: &Path,
    kind: DownloadKind,
    options: &DownloadOptions,
    job: &Job,
    info: &DownloadInfo,
) -> Result<DownloadReport, ErrorCode> {
    let Some(count) = info.playlist_size else {
        return download_one(url, folder, kind, options, info.into(), job, 0..99)
            .map(DownloadReport::Single);
    };
    if !matches!(kind, DownloadKind::Video | DownloadKind::Audio) {
        return Err(ErrorCode::DownloadUnsupported);
    }
    let mut files = Vec::new();
    let mut failed = 0;
    for (index, entry) in info.entries.iter().enumerate() {
        if wait_for_download(job).is_err() {
            break;
        }
        let range = (index * 99 / count)..((index + 1) * 99 / count);
        let end = range.end;
        let result = (|| {
            let entry = entry.as_ref().ok_or(ErrorCode::DownloadFailed)?;
            let link = [entry.webpage_url.as_deref(), entry.url.as_deref()]
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
            download_one(
                &link,
                folder,
                kind,
                &DownloadOptions {
                    name_prefix: options
                        .name_prefix
                        .as_ref()
                        .map(|prefix| format!("{prefix} {}", index + 1)),
                    ..DownloadOptions::default()
                },
                DownloadItem {
                    title: entry.title.as_deref().unwrap_or(&info.title),
                    ..DownloadItem::default()
                },
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
    info: DownloadItem<'_>,
    job: &Job,
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
            "--continue",
            "--match-filter",
            "!is_live",
            "--newline",
            "--progress",
            "--progress-delta",
            "0.25",
            "--progress-template",
            "download:LPSTK:%(progress._percent_str)s|%(progress.downloaded_bytes)s|%(progress.total_bytes)s|%(progress.total_bytes_estimate)s",
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
                .filter(|language| info.languages.iter().any(|item| item == language))
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
    loop {
        wait_for_download(job)?;
        let interrupted = Cell::new(false);
        let mut pending = Vec::new();
        let output = run_process_streaming(
            &executable,
            &args,
            Duration::from_secs(3600),
            || {
                let paused = job.paused();
                if paused {
                    interrupted.set(true);
                }
                paused || job.cancelled()
            },
            |bytes| {
                for &byte in bytes {
                    if byte == b'\n' {
                        if let Some((value, transfer)) = parse_transfer(&pending) {
                            job.publish_transfer(
                                range.start
                                    + ((range.end - range.start) as f64 * value.clamp(0.0, 100.0)
                                        / 100.0) as usize,
                                transfer,
                            );
                        }
                        pending.clear();
                    } else if pending.len() < 256 {
                        pending.push(byte);
                    }
                }
            },
        );
        match output {
            Ok(output) if output.status.success() => break,
            Ok(_) => return Err(ErrorCode::DownloadFailed),
            Err(ProcessError::Cancelled) if interrupted.get() && !job.cancelled() => continue,
            Err(error) => return Err(download_error(error)),
        }
    }
    wait_for_download(job)?;
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
    let preferred = download_file_name(info.title, options.name_prefix.as_deref(), extension)?;
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

fn wait_for_download(job: &Job) -> Result<(), ErrorCode> {
    if job.paused() {
        job.publish(0);
        while job.paused() {
            std::thread::sleep(Duration::from_millis(100));
        }
        job.publish(0);
    }
    if job.cancelled() {
        return Err(ErrorCode::JobCancelled);
    }
    Ok(())
}

fn parse_transfer(line: &[u8]) -> Option<(f64, Option<DownloadTransfer>)> {
    let mut fields = std::str::from_utf8(line)
        .ok()?
        .trim_end()
        .strip_prefix("LPSTK:")?
        .split('|');
    let percent: f64 = fields.next()?.trim().trim_end_matches('%').parse().ok()?;
    let downloaded = fields.next().and_then(|value| value.parse().ok());
    let total = fields.next().and_then(|value| value.parse().ok());
    let estimated = fields.next().and_then(|value| value.parse().ok());
    percent.is_finite().then_some((
        percent,
        downloaded.map(|downloaded_bytes| DownloadTransfer {
            downloaded_bytes,
            total_bytes: total.or(estimated),
        }),
    ))
}

fn download_file_name(
    title: &str,
    prefix: Option<&str>,
    extension: &str,
) -> Result<String, ErrorCode> {
    if let Some(prefix) = prefix {
        return with_output_extension(&format!("{prefix}.tmp"), extension);
    }
    let title: String = title
        .chars()
        .filter(|ch| !ch.is_control() && !"<>:\"/\\|?*".contains(*ch))
        .take(90)
        .collect();
    let title = title.trim_matches([' ', '.']);
    Ok(if title.is_empty() {
        format!("download{extension}")
    } else {
        with_output_extension(&format!("{title}.tmp"), extension)
            .unwrap_or_else(|_| format!("download{extension}"))
    })
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
    fn names_downloads_with_the_requested_prefix_and_extension() {
        assert_eq!(
            download_file_name("Título", Some("Jazz Wall 23"), ".mp3").unwrap(),
            "Jazz Wall 23.mp3"
        );
        assert_eq!(
            download_file_name("Título", None, ".mp4").unwrap(),
            "Título.mp4"
        );
        assert_eq!(
            download_file_name("Título", Some("a/b"), ".mp3"),
            Err(ErrorCode::InvalidOutputName)
        );
    }

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
