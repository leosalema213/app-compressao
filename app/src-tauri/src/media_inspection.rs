use crate::media_contract::{
    AppError, ErrorCode, ImageFormat, MediaInspection, MediaKind, SelectedMedia,
};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

#[tauri::command]
pub async fn inspect_images(paths: Vec<String>) -> Result<Vec<MediaInspection>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        paths
            .into_iter()
            .map(|path| match inspect_image(Path::new(&path)) {
                Ok(media) => MediaInspection::Selected { media },
                Err(code) => MediaInspection::Rejected {
                    path,
                    error: code.into(),
                },
            })
            .collect()
    })
    .await
    .map_err(|_| ErrorCode::Internal.into())
}

#[tauri::command]
pub async fn validate_output_folder(path: String) -> Result<String, AppError> {
    tauri::async_runtime::spawn_blocking(move || inspect_folder(Path::new(&path)))
        .await
        .map_err(|_| ErrorCode::Internal)?
        .map_err(Into::into)
}

pub(crate) fn inspect_image(selected_path: &Path) -> Result<SelectedMedia, ErrorCode> {
    let path = resolve_path(selected_path)?;
    if !fs::metadata(&path).map_err(io_error)?.is_file() {
        return Err(ErrorCode::NotFile);
    }

    let expected = image_format(selected_path).ok_or(ErrorCode::UnsupportedExtension)?;
    if image_format(&path) != Some(expected) {
        return Err(ErrorCode::FormatMismatch);
    }

    let file = File::open(&path).map_err(io_error)?;
    let metadata = file.metadata().map_err(io_error)?;
    if !metadata.is_file() {
        return Err(ErrorCode::NotFile);
    }
    let size_bytes = metadata.len();
    if size_bytes == 0 {
        return Err(ErrorCode::EmptyFile);
    }

    let mut header = Vec::with_capacity(12);
    file.take(12).read_to_end(&mut header).map_err(io_error)?;
    let matches = match expected {
        ImageFormat::Jpeg => header.starts_with(b"\xff\xd8\xff"),
        ImageFormat::Png => header.starts_with(b"\x89PNG\r\n\x1a\n"),
        ImageFormat::Webp => header.starts_with(b"RIFF") && header.get(8..12) == Some(b"WEBP"),
    };
    if !matches {
        return Err(ErrorCode::FormatMismatch);
    }

    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ErrorCode::InvalidPath)?;
    Ok(SelectedMedia {
        kind: MediaKind::Image,
        name: name.to_owned(),
        path: path_text(&path)?,
        size_bytes,
        format: expected,
    })
}

fn image_format(path: &Path) -> Option<ImageFormat> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "webp" => Some(ImageFormat::Webp),
        _ => None,
    }
}

pub(crate) fn inspect_folder(path: &Path) -> Result<String, ErrorCode> {
    let path = resolve_path(path)?;
    if !fs::metadata(&path).map_err(io_error)?.is_dir() {
        return Err(ErrorCode::NotDirectory);
    }
    path_text(&path)
}

pub(crate) fn resolve_path(path: &Path) -> Result<PathBuf, ErrorCode> {
    if !is_local_path(path) {
        return Err(ErrorCode::InvalidPath);
    }
    let resolved = dunce::canonicalize(path).map_err(io_error)?;
    if !is_local_path(&resolved) {
        return Err(ErrorCode::InvalidPath);
    }
    Ok(resolved)
}

#[cfg(windows)]
fn is_local_path(path: &Path) -> bool {
    use std::path::{Component, Prefix};

    if !path.is_absolute() {
        return false;
    }
    let mut parts = path.components();
    let local_drive = matches!(
        parts.next(),
        Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
    );
    local_drive
        && parts.all(|part| match part {
            Component::Normal(name) => !name.to_string_lossy().contains(':'),
            _ => true,
        })
}

#[cfg(not(windows))]
fn is_local_path(path: &Path) -> bool {
    path.is_absolute()
}

fn path_text(path: &Path) -> Result<String, ErrorCode> {
    path.to_str()
        .map(str::to_owned)
        .ok_or(ErrorCode::InvalidPath)
}

fn io_error(error: io::Error) -> ErrorCode {
    match error.kind() {
        io::ErrorKind::NotFound => ErrorCode::NotFound,
        _ => ErrorCode::Unreadable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_image_signatures_and_preserves_input() {
        let root = tempfile::tempdir().unwrap();
        for (name, data, format) in [
            (
                "Foto com acento é.PNG",
                b"\x89PNG\r\n\x1a\n".as_slice(),
                ImageFormat::Png,
            ),
            ("foto.jpeg", b"\xff\xd8\xff".as_slice(), ImageFormat::Jpeg),
            (
                "foto.webp",
                b"RIFF\x00\x00\x00\x00WEBP".as_slice(),
                ImageFormat::Webp,
            ),
        ] {
            let path = root.path().join(name);
            fs::write(&path, data).unwrap();
            let selected = inspect_image(&path).unwrap();
            assert_eq!(selected.name, name);
            assert_eq!(selected.size_bytes, data.len() as u64);
            assert!(selected.format == format);
            assert_eq!(fs::read(path).unwrap(), data);
        }
    }

    #[test]
    fn rejects_empty_unsupported_and_mislabeled_files() {
        let root = tempfile::tempdir().unwrap();
        for (name, data, expected) in [
            ("vazio.png", b"".as_slice(), ErrorCode::EmptyFile),
            (
                "outra.txt",
                b"\x89PNG\r\n\x1a\n".as_slice(),
                ErrorCode::UnsupportedExtension,
            ),
            (
                "falsa.jpg",
                b"\x89PNG\r\n\x1a\n".as_slice(),
                ErrorCode::FormatMismatch,
            ),
        ] {
            let path = root.path().join(name);
            fs::write(&path, data).unwrap();
            assert_eq!(inspect_image(&path).err(), Some(expected));
        }
    }

    #[test]
    fn validates_file_and_folder_paths() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("inexistente.png");
        assert_eq!(inspect_image(&path).err(), Some(ErrorCode::NotFound));
        assert_eq!(inspect_image(root.path()).err(), Some(ErrorCode::NotFile));
        assert_eq!(
            inspect_image(Path::new("relativo.png")).err(),
            Some(ErrorCode::InvalidPath)
        );
        assert!(inspect_folder(root.path()).is_ok());
        fs::write(&path, []).unwrap();
        assert_eq!(inspect_folder(&path), Err(ErrorCode::NotDirectory));
    }

    #[cfg(windows)]
    #[test]
    fn rejects_device_paths_network_shares_and_alternate_streams() {
        for path in [
            r"\\.\PhysicalDrive0",
            r"\\servidor\pasta\imagem.png",
            r"\\?\GLOBALROOT\Device\HarddiskVolume1\imagem.png",
            r"C:\imagem.png:fluxo.png",
        ] {
            assert!(!is_local_path(Path::new(path)), "{path}");
        }
    }
}
