use crate::{
    jobs::{Job, JobQueue, JobStatus, JobUpdate},
    media_contract::{AppError, ErrorCode},
    media_inspection::inspect_folder,
    media_probe::{probe_media, MediaDetails, StreamKind},
    media_tools::locate,
    output_plan::{next_available_name, with_output_extension},
    process_runner::{run_process_streaming, ProcessError},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ffi::OsString, io::ErrorKind, path::Path, time::Duration};
use tauri::Manager;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoPreset {
    Fast,
    Balanced,
    Smaller,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvResult {
    file_name: String,
    input_bytes: u64,
    output_bytes: u64,
}

#[tauri::command]
pub async fn transcode_av(
    path: String,
    folder: String,
    kind: StreamKind,
    preset: VideoPreset,
    target_percent: Option<u8>,
    progress: Option<tauri::ipc::JavaScriptChannelId>,
    webview: tauri::Webview,
) -> Result<AvResult, AppError> {
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
        let result = transcode(&path, &folder, kind, preset, target_percent, &job);
        let (status, completed) = match result {
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

fn transcode(
    path: &str,
    folder: &str,
    kind: StreamKind,
    preset: VideoPreset,
    target_percent: Option<u8>,
    job: &Job,
) -> Result<AvResult, ErrorCode> {
    let executable = locate("ffmpeg").ok_or(ErrorCode::MediaToolMissing)?;
    let source = probe_media(Path::new(path), kind)?;
    let folder = inspect_folder(Path::new(folder))?;
    let folder = Path::new(&folder);
    let target = match (kind, target_percent) {
        (StreamKind::Video, Some(percent)) => Some(TargetBitrate::calculate(&source, percent)?),
        _ => None,
    };
    let extension = match kind {
        StreamKind::Video => ".mp4",
        StreamKind::Audio => ".mp3",
    };
    let preferred_name = with_output_extension(&source.name, extension)?;
    let mut temporary = tempfile::Builder::new()
        .suffix(extension)
        .tempfile_in(folder)
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
    let input = Path::new(&source.path);
    let pass_log = if let Some(target) = &target {
        let logs = tempfile::tempdir_in(folder).map_err(|_| ErrorCode::OutputWriteFailed)?;
        let path = logs.path().join("video");
        let args = encoder_arguments(
            input,
            temporary.path(),
            kind,
            preset,
            Some((target, 1, &path)),
        );
        run_encoder(&executable, &args, job, source.duration_seconds, 0, 50)?;
        Some((logs, path))
    } else {
        None
    };
    let pass = target
        .as_ref()
        .zip(pass_log.as_ref())
        .map(|(target, (_, path))| (target, 2, path.as_path()));
    let args = encoder_arguments(input, temporary.path(), kind, preset, pass);
    let offset = if pass.is_some() { 50 } else { 0 };
    run_encoder(
        &executable,
        &args,
        job,
        source.duration_seconds,
        offset,
        100 - offset,
    )?;
    let saved = probe_media(temporary.path(), kind).map_err(|_| ErrorCode::OutputWriteFailed)?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
    let mut reserved = HashSet::new();
    let file_name = loop {
        if job.cancelled() {
            return Err(ErrorCode::JobCancelled);
        }
        let name = next_available_name(folder, &preferred_name, &mut reserved)?;
        match temporary.persist_noclobber(folder.join(&name)) {
            Ok(_) => break name,
            Err(error) if error.error.kind() == ErrorKind::AlreadyExists => temporary = error.file,
            Err(_) => return Err(ErrorCode::OutputWriteFailed),
        }
    };
    Ok(AvResult {
        file_name,
        input_bytes: source.size_bytes,
        output_bytes: saved.size_bytes,
    })
}

fn run_encoder(
    executable: &Path,
    args: &[OsString],
    job: &Job,
    duration: Option<f64>,
    offset: usize,
    span: usize,
) -> Result<(), ErrorCode> {
    if job.cancelled() {
        return Err(ErrorCode::JobCancelled);
    }
    let mut progress = EncoderProgress {
        pending: Vec::new(),
        last: 0,
        duration,
    };
    let output = run_process_streaming(
        executable,
        args,
        Duration::from_secs(3600),
        || job.cancelled(),
        |bytes| progress.update(bytes, |percent| job.publish(offset + percent * span / 100)),
    )
    .map_err(|error| match error {
        ProcessError::Cancelled => ErrorCode::JobCancelled,
        ProcessError::TimedOut => ErrorCode::ProcessTimedOut,
        ProcessError::Unavailable => ErrorCode::AvProcessingFailed,
    })?;
    if !output.status.success() {
        return Err(ErrorCode::AvProcessingFailed);
    }
    Ok(())
}

struct TargetBitrate {
    video: u64,
    audio: u64,
}

impl TargetBitrate {
    fn calculate(source: &MediaDetails, percent: u8) -> Result<Self, ErrorCode> {
        let duration = source
            .duration_seconds
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or(ErrorCode::VideoTargetUnavailable)?;
        if !(1..=100).contains(&percent) {
            return Err(ErrorCode::VideoTargetUnavailable);
        }
        // Reserve 5% for the container and other encoding overhead.
        let total = (source.size_bytes as f64 * 8.0 * f64::from(percent) / 100.0 * 0.95
            / duration
            / 1000.0) as u64;
        let audio = if source.audio_codec.is_some() {
            (total / 5).clamp(24, 128)
        } else {
            0
        };
        let video = total.saturating_sub(audio);
        if video < 16 {
            return Err(ErrorCode::VideoTargetTooSmall);
        }
        Ok(Self { video, audio })
    }
}

fn encoder_arguments(
    input: &Path,
    output: &Path,
    kind: StreamKind,
    preset: VideoPreset,
    pass: Option<(&TargetBitrate, u8, &Path)>,
) -> Vec<OsString> {
    let mut args: Vec<OsString> = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-y",
        "-progress",
        "pipe:1",
        "-protocol_whitelist",
        "file",
        "-format_whitelist",
        "mov,matroska,webm,avi,mp3,wav,flac,ogg,aac",
        "-i",
    ]
    .into_iter()
    .map(Into::into)
    .collect();
    args.push(input.as_os_str().to_owned());
    match kind {
        StreamKind::Video => {
            let (speed, quality) = match preset {
                VideoPreset::Fast => ("fast", "23"),
                VideoPreset::Balanced => ("medium", "23"),
                VideoPreset::Smaller => ("slow", "28"),
            };
            args.extend(
                [
                    "-map",
                    "0:V:0",
                    "-map",
                    "0:a:0?",
                    "-c:v",
                    "libx264",
                    "-preset",
                    speed,
                    "-pix_fmt",
                    "yuv420p",
                    "-vf",
                    "scale=max(2\\,trunc(iw/2)*2):max(2\\,trunc(ih/2)*2)",
                ]
                .into_iter()
                .map(OsString::from),
            );
            if let Some((target, number, path)) = pass {
                args.extend([
                    "-b:v".into(),
                    format!("{}k", target.video).into(),
                    "-pass".into(),
                    number.to_string().into(),
                    "-passlogfile".into(),
                    path.as_os_str().to_owned(),
                ]);
                if number == 1 {
                    args.extend(["-an", "-f", "null", "-"].map(OsString::from));
                    return args;
                }
            } else {
                args.extend(["-crf", quality].map(OsString::from));
            }
            args.extend(["-c:a", "aac", "-b:a"].map(OsString::from));
            args.push(
                format!(
                    "{}k",
                    pass.map_or(128, |(target, _, _)| target.audio.max(24))
                )
                .into(),
            );
            args.extend(["-movflags", "+faststart", "-f", "mp4"].map(OsString::from));
        }
        StreamKind::Audio => args.extend(
            [
                "-map",
                "0:a:0",
                "-vn",
                "-c:a",
                "libmp3lame",
                "-b:a",
                "128k",
                "-f",
                "mp3",
            ]
            .into_iter()
            .map(OsString::from),
        ),
    }
    args.push(output.as_os_str().to_owned());
    args
}

struct EncoderProgress {
    pending: Vec<u8>,
    last: usize,
    duration: Option<f64>,
}

impl EncoderProgress {
    fn update(&mut self, bytes: &[u8], mut publish: impl FnMut(usize)) {
        let Some(duration) = self.duration.filter(|value| *value > 0.0) else {
            return;
        };
        for &byte in bytes {
            if byte == b'\n' {
                let seconds = std::str::from_utf8(&self.pending)
                    .ok()
                    .and_then(|line| line.trim_end().strip_prefix("out_time_us="))
                    .and_then(|value| value.parse::<u64>().ok());
                self.pending.clear();
                if let Some(microseconds) = seconds {
                    let percent =
                        ((microseconds as f64 / 1_000_000.0 / duration) * 100.0).min(99.0) as usize;
                    if percent > self.last {
                        self.last = percent;
                        publish(percent);
                    }
                }
            } else if self.pending.len() < 256 {
                self.pending.push(byte);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_paths_as_separate_arguments_and_maps_video_without_cover_art() {
        let input = Path::new("entrada com espaços.mp4");
        let output = Path::new("saída.mp4");
        let args = encoder_arguments(input, output, StreamKind::Video, VideoPreset::Smaller, None);
        let index = args.iter().position(|arg| arg == "-i").unwrap();
        assert_eq!(args[index + 1], input.as_os_str());
        assert_eq!(args.last().unwrap(), output.as_os_str());
        assert!(args.windows(2).any(|pair| pair == ["-crf", "28"]));
        assert!(args.windows(2).any(|pair| pair == ["-map", "0:V:0"]));
    }

    #[test]
    fn target_bitrate_reserves_space_for_audio_and_container() {
        let mut source = MediaDetails {
            name: "video.mp4".into(),
            path: "video.mp4".into(),
            size_bytes: 1_000_000,
            duration_seconds: Some(10.0),
            video_codec: Some("h264".into()),
            audio_codec: Some("aac".into()),
            width: None,
            height: None,
        };
        let target = TargetBitrate::calculate(&source, 50).unwrap();
        assert_eq!((target.video, target.audio), (304, 76));
        source.audio_codec = None;
        let target = TargetBitrate::calculate(&source, 50).unwrap();
        assert_eq!((target.video, target.audio), (380, 0));
        assert!(matches!(
            TargetBitrate::calculate(&source, 1),
            Err(ErrorCode::VideoTargetTooSmall)
        ));
        source.duration_seconds = None;
        assert!(matches!(
            TargetBitrate::calculate(&source, 50),
            Err(ErrorCode::VideoTargetUnavailable)
        ));
    }

    #[test]
    fn progress_is_monotonic_and_handles_split_lines() {
        let mut progress = EncoderProgress {
            pending: Vec::new(),
            last: 0,
            duration: Some(10.0),
        };
        let mut values = Vec::new();
        progress.update(b"out_time_", |value| values.push(value));
        progress.update(
            b"us=2500000\nout_time_us=1000000\nout_time_us=10000000\n",
            |value| values.push(value),
        );
        assert_eq!(values, [25, 99]);
    }
}
