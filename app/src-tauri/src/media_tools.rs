use crate::{
    media_contract::{AppError, ErrorCode},
    process_runner::run_process,
};
use serde::Serialize;
use std::{env, ffi::OsString, path::PathBuf, time::Duration};

#[derive(Serialize)]
pub struct ToolReport {
    name: &'static str,
    status: &'static str,
    version: Option<String>,
}

pub(crate) fn locate(name: &str) -> Option<PathBuf> {
    let executable = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };
    env::split_paths(&env::var_os("PATH")?)
        .filter(|path| path.is_absolute())
        .map(|path| path.join(&executable))
        .find(|path| path.is_file())
}

fn version_from_output(tool: &str, bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    if matches!(tool, "yt-dlp" | "node" | "deno") {
        return text.lines().find_map(|line| match tool {
            "yt-dlp"
                if line.len() >= 10
                    && line
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || byte == b'.') =>
            {
                Some(line.to_owned())
            }
            "node" if line.starts_with('v') => Some(line.to_owned()),
            "deno" => line
                .strip_prefix("deno ")
                .and_then(|value| value.split_whitespace().next())
                .map(str::to_owned),
            _ => None,
        });
    }
    let prefix = format!("{tool} version ");
    text.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|line| line.split_whitespace().next())
        .map(str::to_owned)
}

fn diagnose(tool: &'static str) -> ToolReport {
    let Some(program) = locate(tool) else {
        return ToolReport {
            name: tool,
            status: "missing",
            version: None,
        };
    };
    let version = run_process(
        &program,
        &[OsString::from(if matches!(tool, "ffmpeg" | "ffprobe") {
            "-version"
        } else {
            "--version"
        })],
        Duration::from_secs(5),
        || false,
    )
    .ok()
    .filter(|output| output.status.success() && !output.truncated)
    .and_then(|output| {
        version_from_output(tool, &output.stdout)
            .or_else(|| version_from_output(tool, &output.stderr))
    });
    let status = if version.is_some() {
        "available"
    } else {
        "unavailable"
    };
    ToolReport {
        name: tool,
        status,
        version,
    }
}

#[tauri::command]
pub async fn diagnose_media_tools() -> Result<Vec<ToolReport>, AppError> {
    tauri::async_runtime::spawn_blocking(|| {
        ["ffmpeg", "ffprobe", "yt-dlp", "node", "deno"]
            .into_iter()
            .map(diagnose)
            .collect()
    })
    .await
    .map_err(|_| ErrorCode::Internal.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_the_expected_version_header() {
        assert_eq!(
            version_from_output("ffmpeg", b"ffmpeg version 9.0.1 build\n"),
            Some("9.0.1".to_owned())
        );
        assert_eq!(
            version_from_output("ffprobe", b"ffmpeg version 9.0.1 build\n"),
            None
        );
        assert_eq!(version_from_output("ffprobe", b"ffprobe version \n"), None);
    }
}
