use crate::{
    media_contract::{AppError, ErrorCode},
    media_inspection::resolve_path,
    media_tools::locate,
    process_runner::run_process,
};
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, fs, path::Path, time::Duration};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamKind {
    Video,
    Audio,
}

#[derive(Deserialize)]
struct ProbeOutput {
    format: ProbeFormat,
    #[serde(default)]
    streams: Vec<ProbeStream>,
}

#[derive(Deserialize)]
struct ProbeFormat {
    format_name: String,
    duration: Option<String>,
}

#[derive(Deserialize)]
struct ProbeStream {
    codec_type: Option<String>,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    disposition: Option<Disposition>,
}

#[derive(Deserialize)]
struct Disposition {
    attached_pic: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDetails {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub duration_seconds: Option<f64>,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[tauri::command]
pub async fn inspect_av_file(path: String, kind: StreamKind) -> Result<MediaDetails, AppError> {
    tauri::async_runtime::spawn_blocking(move || probe_media(Path::new(&path), kind))
        .await
        .map_err(|_| ErrorCode::Internal)?
        .map_err(Into::into)
}

pub fn probe_media(path: &Path, kind: StreamKind) -> Result<MediaDetails, ErrorCode> {
    let expected = extension_format(path)?;
    let path = resolve_path(path)?;
    if extension_format(&path)? != expected {
        return Err(ErrorCode::FormatMismatch);
    }
    let metadata = fs::metadata(&path).map_err(|_| ErrorCode::Unreadable)?;
    if !metadata.is_file() {
        return Err(ErrorCode::NotFile);
    }
    if metadata.len() == 0 {
        return Err(ErrorCode::EmptyFile);
    }
    let executable = locate("ffprobe").ok_or(ErrorCode::MediaToolMissing)?;
    let mut args: Vec<OsString> = [
        "-v", "error", "-protocol_whitelist", "file",
        "-format_whitelist", "mov,matroska,webm,avi,mp3,wav,flac,ogg,aac",
        "-show_entries", "format=format_name,duration:stream=codec_type,codec_name,width,height:stream_disposition=attached_pic",
        "-of", "json", "-i",
    ].into_iter().map(Into::into).collect();
    args.push(path.as_os_str().to_owned());
    let output = run_process(&executable, &args, Duration::from_secs(15), || false)
        .map_err(|_| ErrorCode::MediaProbeFailed)?;
    if !output.status.success() || output.truncated {
        return Err(ErrorCode::MediaProbeFailed);
    }
    let probe: ProbeOutput =
        serde_json::from_slice(&output.stdout).map_err(|_| ErrorCode::MediaProbeFailed)?;
    if !probe
        .format
        .format_name
        .split(',')
        .any(|name| name == expected)
    {
        return Err(ErrorCode::FormatMismatch);
    }
    let video = probe.streams.iter().find(|stream| {
        stream.codec_type.as_deref() == Some("video")
            && !stream
                .disposition
                .as_ref()
                .is_some_and(|item| item.attached_pic != 0)
    });
    let audio = probe
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"));
    if matches!(kind, StreamKind::Video) && video.is_none()
        || matches!(kind, StreamKind::Audio) && audio.is_none()
    {
        return Err(ErrorCode::MediaStreamMissing);
    }
    let duration_seconds = probe
        .format
        .duration
        .and_then(|duration| duration.parse::<f64>().ok())
        .filter(|duration| duration.is_finite() && *duration >= 0.0);
    Ok(MediaDetails {
        name: path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(ErrorCode::InvalidPath)?
            .to_owned(),
        path: path.to_str().ok_or(ErrorCode::InvalidPath)?.to_owned(),
        size_bytes: metadata.len(),
        duration_seconds,
        video_codec: video.and_then(|stream| stream.codec_name.clone()),
        audio_codec: audio.and_then(|stream| stream.codec_name.clone()),
        width: video.and_then(|stream| stream.width),
        height: video.and_then(|stream| stream.height),
    })
}

fn extension_format(path: &Path) -> Result<&'static str, ErrorCode> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp4" | "mov" | "m4a") => Ok("mov"),
        Some("mkv" | "webm") => Ok("matroska"),
        Some("avi") => Ok("avi"),
        Some("mp3") => Ok("mp3"),
        Some("wav") => Ok("wav"),
        Some("flac") => Ok("flac"),
        Some("ogg" | "opus") => Ok("ogg"),
        Some("aac") => Ok("aac"),
        _ => Err(ErrorCode::UnsupportedMediaExtension),
    }
}
