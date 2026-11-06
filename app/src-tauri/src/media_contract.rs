use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedMedia {
    pub kind: MediaKind,
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub format: ImageFormat,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MediaInspection {
    Selected { media: SelectedMedia },
    Rejected { path: String, error: AppError },
}

#[derive(Serialize)]
pub struct AppError {
    pub code: ErrorCode,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidPath,
    NotFound,
    Unreadable,
    NotFile,
    NotDirectory,
    EmptyFile,
    UnsupportedExtension,
    FormatMismatch,
    InvalidOutputName,
    OutputNamesExhausted,
    ImageTooLarge,
    InvalidImage,
    AnimatedImage,
    OutputWriteFailed,
    InvalidResizeBounds,
    InvalidJpegQuality,
    OutputFolderUnavailable,
    OutputFolderOpenFailed,
    JobCancelled,
    MediaToolMissing,
    MediaProbeFailed,
    MediaStreamMissing,
    UnsupportedMediaExtension,
    AvProcessingFailed,
    ProcessTimedOut,
    InvalidDownloadUrl,
    DownloaderMissing,
    DownloadFailed,
    DownloadUnsupported,
    DownloadNotAuthorized,
    DownloadQualityUnavailable,
    DownloadExtraUnavailable,
    DownloadPlaylistConfirmation,
    Internal,
}

impl From<ErrorCode> for AppError {
    fn from(code: ErrorCode) -> Self {
        Self { code }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection_matches_shared_contract() {
        let response = [
            MediaInspection::Selected {
                media: SelectedMedia {
                    kind: MediaKind::Image,
                    path: r"C:\Mídia\foto.png".to_owned(),
                    name: "foto.png".to_owned(),
                    size_bytes: 128,
                    format: ImageFormat::Png,
                },
            },
            MediaInspection::Rejected {
                path: r"C:\Mídia\outra.png".to_owned(),
                error: ErrorCode::FormatMismatch.into(),
            },
        ];
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/image-inspection.json"))
                .unwrap();
        assert_eq!(serde_json::to_value(response).unwrap(), fixture);
    }
}
