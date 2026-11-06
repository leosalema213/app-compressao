use crate::{
    media_contract::{AppError, ErrorCode, ImageFormat, SelectedMedia},
    media_inspection::{inspect_folder, inspect_image},
};
use serde::Serialize;
use std::{collections::HashSet, fs, io, path::Path};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedOutput {
    input_path: String,
    file_name: String,
}

#[tauri::command]
pub async fn preview_image_outputs(
    paths: Vec<String>,
    folder: String,
    format: Option<ImageFormat>,
) -> Result<Vec<PlannedOutput>, AppError> {
    tauri::async_runtime::spawn_blocking(move || plan_outputs(&paths, Path::new(&folder), format))
        .await
        .map_err(|_| ErrorCode::Internal)?
        .map_err(Into::into)
}

fn plan_outputs(
    paths: &[String],
    folder: &Path,
    format: Option<ImageFormat>,
) -> Result<Vec<PlannedOutput>, ErrorCode> {
    let resolved_folder = inspect_folder(folder)?;
    let folder = Path::new(&resolved_folder);
    let mut planned_names = HashSet::new();
    paths
        .iter()
        .map(|path| {
            let source = inspect_image(Path::new(path))?;
            let name = output_name(&source, format.unwrap_or(source.format))?;
            let file_name = next_available_name(folder, &name, &mut planned_names)?;
            Ok(PlannedOutput {
                input_path: source.path,
                file_name,
            })
        })
        .collect()
}

pub(crate) fn output_name(
    source: &SelectedMedia,
    format: ImageFormat,
) -> Result<String, ErrorCode> {
    if !valid_windows_name(&source.name) {
        return Err(ErrorCode::InvalidOutputName);
    }
    if format == source.format {
        return Ok(source.name.clone());
    }
    let (stem, _) = source
        .name
        .rsplit_once('.')
        .ok_or(ErrorCode::InvalidOutputName)?;
    fit_name(
        stem,
        match format {
            ImageFormat::Jpeg => ".jpg",
            ImageFormat::Png => ".png",
            ImageFormat::Webp => ".webp",
        },
    )
}

pub(crate) fn next_available_name(
    folder: &Path,
    original: &str,
    planned_names: &mut HashSet<String>,
) -> Result<String, ErrorCode> {
    let (stem, extension) = original
        .rsplit_once('.')
        .ok_or(ErrorCode::InvalidOutputName)?;
    if !valid_windows_name(original) {
        return Err(ErrorCode::InvalidOutputName);
    }

    for number in 1..=10_000 {
        let candidate = if number == 1 {
            original.to_owned()
        } else {
            let suffix = format!("-{number}.{extension}");
            fit_name(stem, &suffix)?
        };
        let key = candidate.to_uppercase();
        if planned_names.contains(&key) {
            continue;
        }
        // Directory entries, including links with missing targets, occupy a name.
        match fs::symlink_metadata(folder.join(&candidate)) {
            Ok(_) => continue,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                planned_names.insert(key);
                return Ok(candidate);
            }
            Err(_) => return Err(ErrorCode::Unreadable),
        }
    }
    Err(ErrorCode::OutputNamesExhausted)
}

pub(crate) fn with_output_extension(name: &str, suffix: &str) -> Result<String, ErrorCode> {
    if !valid_windows_name(name) {
        return Err(ErrorCode::InvalidOutputName);
    }
    let (stem, _) = name.rsplit_once('.').ok_or(ErrorCode::InvalidOutputName)?;
    fit_name(stem, suffix)
}

fn fit_name(stem: &str, suffix: &str) -> Result<String, ErrorCode> {
    let mut units_left = 255_usize
        .checked_sub(suffix.encode_utf16().count())
        .ok_or(ErrorCode::InvalidOutputName)?;
    let mut name: String = stem
        .chars()
        .take_while(|ch| {
            if ch.len_utf16() > units_left {
                return false;
            }
            units_left -= ch.len_utf16();
            true
        })
        .collect();
    name.push_str(suffix);
    Ok(name)
}

fn valid_windows_name(name: &str) -> bool {
    let first_part = name
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_ascii_uppercase();
    let reserved_port = first_part
        .strip_prefix("COM")
        .or_else(|| first_part.strip_prefix("LPT"))
        .is_some_and(|digit| {
            matches!(
                digit,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        });

    !name.is_empty()
        && !name.ends_with([' ', '.'])
        && name.encode_utf16().count() <= 255
        && !name
            .chars()
            .any(|ch| ch <= '\u{1f}' || "<>:\"/\\|?*".contains(ch))
        && !matches!(
            first_part.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        )
        && !reserved_port
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_collisions_using_the_converted_extension() {
        let folder = tempfile::tempdir().unwrap();
        let source = folder.path().join("foto.png");
        fs::write(&source, b"\x89PNG\r\n\x1a\n").unwrap();
        fs::write(folder.path().join("foto.jpg"), b"ocupado").unwrap();
        let plan = plan_outputs(
            &[source.to_str().unwrap().to_owned()],
            folder.path(),
            Some(ImageFormat::Jpeg),
        )
        .unwrap();
        assert_eq!(plan[0].file_name, "foto-2.jpg");
    }

    #[test]
    fn plans_suffixes_for_original_existing_entries_and_batch_duplicates() {
        let folder = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let name = "Foto com acento é.png";
        let original = folder.path().join(name);
        let same_name = other.path().join(name);
        let png_signature = b"\x89PNG\r\n\x1a\n";
        fs::write(&original, png_signature).unwrap();
        fs::write(&same_name, png_signature).unwrap();
        fs::create_dir(folder.path().join("Foto com acento é-2.png")).unwrap();
        let paths = [
            original.to_str().unwrap().to_owned(),
            same_name.to_str().unwrap().to_owned(),
        ];

        let plan = plan_outputs(&paths, folder.path(), None).unwrap();
        assert_eq!(plan[0].file_name, "Foto com acento é-3.png");
        assert_eq!(plan[1].file_name, "Foto com acento é-4.png");
        assert_eq!(fs::read(&original).unwrap(), png_signature);
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 2);
    }

    #[test]
    fn keeps_available_name_and_treats_case_variants_as_occupied_in_batch() {
        let folder = tempfile::tempdir().unwrap();
        let mut planned = HashSet::new();
        assert_eq!(
            next_available_name(folder.path(), "Foto.png", &mut planned),
            Ok("Foto.png".into())
        );
        assert_eq!(
            next_available_name(folder.path(), "foto.PNG", &mut planned),
            Ok("foto-2.PNG".into())
        );
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
    }

    #[test]
    fn rejects_reserved_and_unsafe_names() {
        for name in [
            "NUL.png",
            "Com¹.jpeg",
            "LPT9.extra.webp",
            "aux .png",
            "../foto.png",
            "foto.png.",
        ] {
            assert!(!valid_windows_name(name), "{name}");
        }
        assert!(valid_windows_name("Foto com acento é.png"));
        assert!(valid_windows_name("LPT10.png"));
    }

    #[test]
    fn fits_suffix_without_splitting_unicode_characters() {
        let folder = tempfile::tempdir().unwrap();
        let mut planned = HashSet::new();
        let name = format!("{}.png", "🟢".repeat(125));
        assert_eq!(
            next_available_name(folder.path(), &name, &mut planned),
            Ok(name.clone())
        );
        let renamed = next_available_name(folder.path(), &name, &mut planned).unwrap();
        assert_eq!(renamed, format!("{}-2.png", "🟢".repeat(124)));
        assert!(renamed.encode_utf16().count() <= 255);
    }
}
