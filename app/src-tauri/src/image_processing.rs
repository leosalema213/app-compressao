use crate::{
    jobs::{Job, JobQueue, JobStatus, JobUpdate},
    media_contract::{AppError, ErrorCode, ImageFormat, SelectedMedia},
    media_inspection::{inspect_folder, inspect_image},
    output_plan::{next_available_name, output_name},
};
use image::{
    codecs::{
        jpeg::{JpegDecoder, JpegEncoder},
        png::{CompressionType, FilterType, PngDecoder, PngEncoder},
        webp::{WebPDecoder, WebPEncoder},
    },
    imageops::FilterType as ResizeFilter,
    DynamicImage, ImageDecoder, ImageError, Limits,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, BufWriter, ErrorKind, Write},
    path::Path,
    sync::Mutex,
};
use tauri::Manager;
use tauri_plugin_opener::OpenerExt;
use tempfile::NamedTempFile;

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_DECODED_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PIXELS: u64 = 40_000_000;
const JPEG_QUALITY: u8 = 80;
const MAX_RESIZE_EDGE: u32 = 16_384;

#[derive(Deserialize, Clone, Copy)]
pub struct ResizeBounds {
    width: Option<u32>,
    height: Option<u32>,
}

impl ResizeBounds {
    fn validate(self) -> Result<(), ErrorCode> {
        let dimensions = [self.width, self.height];
        if dimensions.iter().all(Option::is_none)
            || dimensions
                .into_iter()
                .flatten()
                .any(|edge| !(1..=MAX_RESIZE_EDGE).contains(&edge))
        {
            return Err(ErrorCode::InvalidResizeBounds);
        }
        Ok(())
    }

    fn apply(self, image: DynamicImage) -> DynamicImage {
        let width = self.width.unwrap_or(image.width()).min(image.width());
        let height = self.height.unwrap_or(image.height()).min(image.height());
        if (width, height) == (image.width(), image.height()) {
            return image;
        }
        if !image.has_alpha() {
            return image.resize(width, height, ResizeFilter::Lanczos3);
        }

        // Resize premultiplied pixels so transparent colors do not bleed into visible edges.
        let preserve_16_bits = image.color().bytes_per_pixel() > image.color().channel_count();
        let mut rgba = image.into_rgba16();
        for pixel in rgba.pixels_mut() {
            let alpha = u32::from(pixel[3]);
            for channel in &mut pixel.0[..3] {
                *channel = ((u32::from(*channel) * alpha + 32_767) / 65_535) as u16;
            }
        }
        let mut resized = DynamicImage::ImageRgba16(rgba)
            .resize(width, height, ResizeFilter::Lanczos3)
            .into_rgba16();
        for pixel in resized.pixels_mut() {
            let alpha = u32::from(pixel[3]);
            for channel in &mut pixel.0[..3] {
                *channel = (u32::from(*channel) * 65_535 + alpha / 2)
                    .checked_div(alpha)
                    .unwrap_or(0)
                    .min(65_535) as u16;
            }
        }
        let result = DynamicImage::ImageRgba16(resized);
        if preserve_16_bits {
            result
        } else {
            DynamicImage::ImageRgba8(result.into_rgba8())
        }
    }
}

#[derive(Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum CompressionResult {
    Saved {
        input_path: String,
        file_name: String,
        input_bytes: u64,
        output_bytes: u64,
        width: u32,
        height: u32,
    },
    Failed {
        input_path: String,
        error: AppError,
    },
    Cancelled {
        input_path: String,
    },
}

#[derive(Default)]
pub struct LastImageOutputFolder(Mutex<Option<String>>);

impl LastImageOutputFolder {
    fn remember(&self, folder: String, results: &[CompressionResult]) {
        if results
            .iter()
            .any(|item| matches!(item, CompressionResult::Saved { .. }))
        {
            *self.0.lock().unwrap_or_else(|error| error.into_inner()) = Some(folder);
        }
    }

    fn validated_folder(&self, requested: &str) -> Result<String, ErrorCode> {
        let remembered = self
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        if remembered.as_deref() != Some(requested) {
            return Err(ErrorCode::OutputFolderUnavailable);
        }
        let current = inspect_folder(Path::new(requested))?;
        if current != requested {
            return Err(ErrorCode::OutputFolderUnavailable);
        }
        Ok(current)
    }
}

#[tauri::command]
pub async fn open_image_output_folder(
    app: tauri::AppHandle,
    last_output: tauri::State<'_, LastImageOutputFolder>,
    folder: String,
) -> Result<(), AppError> {
    let folder = last_output.validated_folder(&folder)?;
    app.opener()
        .open_path(folder, None::<&str>)
        .map_err(|_| ErrorCode::OutputFolderOpenFailed.into())
}

trait ImageProcessor {
    fn output_format(&self, source: ImageFormat) -> ImageFormat {
        source
    }

    fn encode(
        &self,
        source: &SelectedMedia,
        format: ImageFormat,
        output: &mut File,
    ) -> Result<(), ErrorCode>;
}

#[derive(Default)]
struct ImageRsProcessor {
    resize: Option<ResizeBounds>,
    format: Option<ImageFormat>,
    jpeg_quality: Option<u8>,
}

impl ImageProcessor for ImageRsProcessor {
    fn output_format(&self, source: ImageFormat) -> ImageFormat {
        self.format.unwrap_or(source)
    }

    fn encode(
        &self,
        source: &SelectedMedia,
        format: ImageFormat,
        output: &mut File,
    ) -> Result<(), ErrorCode> {
        let image = decode(Path::new(&source.path), source.format)?;
        let image = match self.resize {
            Some(bounds) => bounds.apply(image),
            None => image,
        };
        let mut writer = BufWriter::new(output);
        match format {
            ImageFormat::Jpeg => {
                jpeg_pixels(image).write_with_encoder(JpegEncoder::new_with_quality(
                    &mut writer,
                    self.jpeg_quality.unwrap_or(JPEG_QUALITY),
                ))
            }
            ImageFormat::Png => image.write_with_encoder(PngEncoder::new_with_quality(
                &mut writer,
                CompressionType::Best,
                FilterType::Adaptive,
            )),
            ImageFormat::Webp => {
                let pixels = if image.has_alpha() {
                    DynamicImage::ImageRgba8(image.into_rgba8())
                } else {
                    DynamicImage::ImageRgb8(image.into_rgb8())
                };
                pixels.write_with_encoder(WebPEncoder::new_lossless(&mut writer))
            }
        }
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
        writer.flush().map_err(|_| ErrorCode::OutputWriteFailed)
    }
}

fn jpeg_pixels(image: DynamicImage) -> DynamicImage {
    if !image.has_alpha() {
        return match image {
            DynamicImage::ImageLuma8(_) | DynamicImage::ImageRgb8(_) => image,
            _ => DynamicImage::ImageRgb8(image.into_rgb8()),
        };
    }
    let rgba = image.into_rgba8();
    let rgb =
        image::RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
            let [red, green, blue, alpha] = rgba.get_pixel(x, y).0;
            let alpha = u16::from(alpha);
            image::Rgb([red, green, blue].map(|channel| {
                ((u16::from(channel) * alpha + 255 * (255 - alpha) + 127) / 255) as u8
            }))
        });
    DynamicImage::ImageRgb8(rgb)
}

#[tauri::command]
pub async fn compress_images(
    paths: Vec<String>,
    folder: String,
    resize: Option<ResizeBounds>,
    format: Option<ImageFormat>,
    jpeg_quality: Option<u8>,
    progress: Option<tauri::ipc::JavaScriptChannelId>,
    webview: tauri::Webview,
) -> Result<Vec<CompressionResult>, AppError> {
    if let Some(bounds) = resize {
        bounds.validate()?;
    }
    if jpeg_quality.is_some_and(|quality| !(1..=100).contains(&quality)) {
        return Err(ErrorCode::InvalidJpegQuality.into());
    }
    let app = webview.app_handle().clone();
    let queue = app.state::<JobQueue>();
    let job = queue.register(
        paths.len(),
        progress.map(|id| id.channel_on::<_, JobUpdate>(webview)),
    );
    let execution = queue.execution.clone().lock_owned().await;
    if !job.start() {
        job.finish(JobStatus::Cancelled, 0);
        return Ok(paths
            .into_iter()
            .map(|input_path| CompressionResult::Cancelled { input_path })
            .collect());
    }
    let (_, results) = tauri::async_runtime::spawn_blocking(move || {
        let _execution = execution;
        let batch = process_batch(
            paths,
            &folder,
            &ImageRsProcessor {
                resize,
                format,
                jpeg_quality,
            },
            Some(&job),
        );
        match &batch {
            Ok((folder, results)) => {
                app.state::<LastImageOutputFolder>()
                    .remember(folder.clone(), results);
                let completed = results
                    .iter()
                    .filter(|item| !matches!(item, CompressionResult::Cancelled { .. }))
                    .count();
                let status = if completed == results.len() {
                    JobStatus::Finished
                } else {
                    JobStatus::Cancelled
                };
                job.finish(status, completed);
            }
            Err(_) => job.finish(JobStatus::Failed, 0),
        }
        batch
    })
    .await
    .map_err(|_| ErrorCode::Internal)??;
    Ok(results)
}

fn process_batch(
    paths: Vec<String>,
    folder: &str,
    processor: &impl ImageProcessor,
    job: Option<&Job>,
) -> Result<(String, Vec<CompressionResult>), ErrorCode> {
    let resolved_folder = inspect_folder(Path::new(folder))?;
    let folder = Path::new(&resolved_folder);
    let mut reserved = HashSet::new();
    let results = paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let item = match process_image(Path::new(&path), folder, processor, &mut reserved, job)
            {
                Ok(item) => item,
                Err(ErrorCode::JobCancelled) => {
                    return CompressionResult::Cancelled { input_path: path }
                }
                Err(code) => CompressionResult::Failed {
                    input_path: path,
                    error: code.into(),
                },
            };
            if let Some(job) = job {
                job.publish(index + 1);
            }
            item
        })
        .collect();
    Ok((resolved_folder, results))
}

fn process_image(
    path: &Path,
    folder: &Path,
    processor: &impl ImageProcessor,
    reserved: &mut HashSet<String>,
    job: Option<&Job>,
) -> Result<CompressionResult, ErrorCode> {
    if job.is_some_and(Job::cancelled) {
        return Err(ErrorCode::JobCancelled);
    }
    let source = inspect_image(path)?;
    let format = processor.output_format(source.format);
    let preferred_name = output_name(&source, format)?;
    let mut temporary = NamedTempFile::new_in(folder).map_err(|_| ErrorCode::OutputWriteFailed)?;
    processor.encode(&source, format, temporary.as_file_mut())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|_| ErrorCode::OutputWriteFailed)?;
    // Read back the encoded pixels before giving the file its final name.
    let (width, height) = {
        let validated =
            decode(temporary.path(), format).map_err(|_| ErrorCode::OutputWriteFailed)?;
        (validated.width(), validated.height())
    };
    let output_bytes = temporary
        .as_file()
        .metadata()
        .map_err(|_| ErrorCode::OutputWriteFailed)?
        .len();

    let file_name = loop {
        if job.is_some_and(Job::cancelled) {
            return Err(ErrorCode::JobCancelled);
        }
        let name = next_available_name(folder, &preferred_name, reserved)?;
        match temporary.persist_noclobber(folder.join(&name)) {
            Ok(_) => break name,
            Err(error) if error.error.kind() == ErrorKind::AlreadyExists => temporary = error.file,
            Err(_) => return Err(ErrorCode::OutputWriteFailed),
        }
    };

    Ok(CompressionResult::Saved {
        input_path: source.path,
        file_name,
        input_bytes: source.size_bytes,
        output_bytes,
        width,
        height,
    })
}

pub(crate) fn decode(path: &Path, format: ImageFormat) -> Result<DynamicImage, ErrorCode> {
    let file = File::open(path).map_err(|_| ErrorCode::Unreadable)?;
    if file.metadata().map_err(|_| ErrorCode::Unreadable)?.len() > MAX_FILE_BYTES {
        return Err(ErrorCode::ImageTooLarge);
    }
    let reader = BufReader::new(file);
    match format {
        ImageFormat::Jpeg => decode_pixels(JpegDecoder::new(reader).map_err(decode_error)?),
        ImageFormat::Png => {
            let mut limits = Limits::default();
            limits.max_alloc = Some(MAX_DECODED_BYTES);
            let decoder = PngDecoder::with_limits(reader, limits).map_err(decode_error)?;
            if decoder.is_apng().map_err(decode_error)? {
                return Err(ErrorCode::AnimatedImage);
            }
            decode_pixels(decoder)
        }
        ImageFormat::Webp => {
            let decoder = WebPDecoder::new(reader).map_err(decode_error)?;
            if decoder.has_animation() {
                return Err(ErrorCode::AnimatedImage);
            }
            decode_pixels(decoder)
        }
    }
}

fn decode_pixels(mut decoder: impl ImageDecoder) -> Result<DynamicImage, ErrorCode> {
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_PIXELS
        || decoder.total_bytes() > MAX_DECODED_BYTES
    {
        return Err(ErrorCode::ImageTooLarge);
    }
    let orientation = decoder.orientation().map_err(decode_error)?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(decode_error)?;
    image.apply_orientation(orientation);
    Ok(image)
}

fn decode_error(error: ImageError) -> ErrorCode {
    match error {
        ImageError::Limits(_) => ErrorCode::ImageTooLarge,
        _ => ErrorCode::InvalidImage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use std::fs;

    struct InvalidEncoder;

    #[test]
    #[ignore = "requires the synthetic benchmark corpus"]
    fn benchmark_image_corpus() {
        use std::time::Instant;

        let root = std::env::var("LPSTOOLKIT_IMAGE_BENCH").unwrap();
        let root = Path::new(&root);
        let output = root.join("rust");
        fs::create_dir(&output).unwrap();
        let mut measurements = Vec::new();
        for name in [
            "oriented.jpg",
            "detailed.jpg",
            "alpha.png",
            "opaque.png",
            "odd.webp",
            "corrupt.png",
            "misleading.jpg",
        ] {
            let path = root.join(name);
            let original = fs::read(&path).unwrap();
            let start = Instant::now();
            let (_, result) = process_batch(
                vec![path.to_str().unwrap().to_owned()],
                output.to_str().unwrap(),
                &ImageRsProcessor {
                    resize: Some(ResizeBounds {
                        width: Some(1600),
                        height: Some(1200),
                    }),
                    ..Default::default()
                },
                None,
            )
            .unwrap();
            measurements.push(serde_json::json!({
                "name": name,
                "ms": start.elapsed().as_secs_f64() * 1000.0,
                "result": result[0],
            }));
            assert_eq!(fs::read(&path).unwrap(), original);
        }
        fs::write(
            root.join("rust.json"),
            serde_json::to_vec_pretty(&measurements).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn opens_only_the_current_successful_output_directory() {
        let output = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let folder = inspect_folder(output.path()).unwrap();
        let unrelated = inspect_folder(other.path()).unwrap();
        let state = LastImageOutputFolder::default();
        assert_eq!(
            state.validated_folder(&folder),
            Err(ErrorCode::OutputFolderUnavailable)
        );

        let saved = CompressionResult::Saved {
            input_path: "foto.png".into(),
            file_name: "foto-2.png".into(),
            input_bytes: 128,
            output_bytes: 96,
            width: 4,
            height: 4,
        };
        state.remember(folder.clone(), &[saved]);
        state.remember(
            unrelated.clone(),
            &[CompressionResult::Failed {
                input_path: "corrupta.png".into(),
                error: ErrorCode::InvalidImage.into(),
            }],
        );
        assert_eq!(state.validated_folder(&folder), Ok(folder.clone()));
        assert_eq!(
            state.validated_folder(&unrelated),
            Err(ErrorCode::OutputFolderUnavailable)
        );

        output.close().unwrap();
        assert_eq!(state.validated_folder(&folder), Err(ErrorCode::NotFound));
    }

    #[test]
    fn converts_between_all_supported_formats() {
        let root = tempfile::tempdir().unwrap();
        let formats = [
            (ImageFormat::Jpeg, image::ImageFormat::Jpeg, "jpg"),
            (ImageFormat::Png, image::ImageFormat::Png, "png"),
            (ImageFormat::Webp, image::ImageFormat::WebP, "webp"),
        ];
        for (source_format, source_codec, source_extension) in formats {
            let source = root.path().join(format!("foto.{source_extension}"));
            let pixels = match source_format {
                ImageFormat::Jpeg => DynamicImage::new_rgb8(24, 16),
                ImageFormat::Png => DynamicImage::ImageRgba16(image::ImageBuffer::from_pixel(
                    24,
                    16,
                    Rgba([0, 0, 65_535, 0]),
                )),
                ImageFormat::Webp => {
                    DynamicImage::ImageRgba8(RgbaImage::from_pixel(24, 16, Rgba([0, 0, 255, 0])))
                }
            };
            pixels.save_with_format(&source, source_codec).unwrap();
            let original = fs::read(&source).unwrap();
            for (target, codec, extension) in formats {
                let output = tempfile::tempdir().unwrap();
                let (_, result) = process_batch(
                    vec![source.to_str().unwrap().into()],
                    output.path().to_str().unwrap(),
                    &ImageRsProcessor {
                        format: Some(target),
                        jpeg_quality: Some(95),
                        ..Default::default()
                    },
                    None,
                )
                .unwrap();
                let name = format!("foto.{extension}");
                assert!(
                    matches!(&result[0], CompressionResult::Saved { file_name, .. } if file_name == &name)
                );
                let bytes = fs::read(output.path().join(name)).unwrap();
                assert_eq!(image::guess_format(&bytes).unwrap(), codec);
                let saved = image::load_from_memory(&bytes).unwrap();
                assert_eq!((saved.width(), saved.height()), (24, 16));
                if source_format != ImageFormat::Jpeg {
                    if target == ImageFormat::Jpeg {
                        assert_eq!(saved.to_rgb8().get_pixel(0, 0).0, [255, 255, 255]);
                    } else {
                        assert_eq!(saved.to_rgba8().get_pixel(0, 0).0[3], 0);
                    }
                }
            }
            assert_eq!(fs::read(source).unwrap(), original);
        }
    }

    impl ImageProcessor for InvalidEncoder {
        fn encode(
            &self,
            _source: &SelectedMedia,
            _format: ImageFormat,
            output: &mut File,
        ) -> Result<(), ErrorCode> {
            output
                .write_all(b"invalid image")
                .map_err(|_| ErrorCode::OutputWriteFailed)
        }
    }

    #[test]
    fn fits_width_height_or_both_without_upscaling() {
        for (width, height, expected) in [
            (Some(48), None, (48, 32)),
            (None, Some(16), (24, 16)),
            (Some(30), Some(12), (18, 12)),
            (Some(200), Some(200), (96, 64)),
        ] {
            let bounds = ResizeBounds { width, height };
            assert_eq!(bounds.validate(), Ok(()));
            let resized = bounds.apply(DynamicImage::new_rgb8(96, 64));
            assert_eq!((resized.width(), resized.height()), expected);
        }
        for (width, height) in [(None, None), (Some(0), Some(1)), (None, Some(16_385))] {
            assert_eq!(
                ResizeBounds { width, height }.validate(),
                Err(ErrorCode::InvalidResizeBounds)
            );
        }
    }

    #[test]
    fn transparent_pixels_do_not_tint_visible_edges() {
        let pixels = RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 0]).unwrap();
        let bounds = ResizeBounds {
            width: Some(1),
            height: None,
        };
        let resized = bounds.apply(DynamicImage::ImageRgba8(pixels)).into_rgba8();
        let [red, green, blue, alpha] = resized.get_pixel(0, 0).0;
        assert_eq!([red, green, blue], [255, 0, 0]);
        assert!((120..=136).contains(&alpha));
    }

    #[test]
    fn applies_exif_orientation_before_sizing_and_reports_output_dimensions() {
        use image::ImageEncoder;

        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = input.path().join("rotated.jpg");
        let mut encoder = JpegEncoder::new_with_quality(File::create(&source).unwrap(), 90);
        // Little-endian TIFF EXIF entry 0x0112, value 6 (90 degrees clockwise).
        encoder
            .set_exif_metadata(vec![
                0x49, 0x49, 0x2a, 0, 8, 0, 0, 0, 1, 0, 0x12, 1, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0,
                0, 0,
            ])
            .unwrap();
        DynamicImage::new_rgb8(80, 40)
            .write_with_encoder(encoder)
            .unwrap();
        let original = fs::read(&source).unwrap();
        let (_, result) = process_batch(
            vec![source.to_str().unwrap().into()],
            output.path().to_str().unwrap(),
            &ImageRsProcessor {
                resize: Some(ResizeBounds {
                    width: Some(20),
                    height: None,
                }),
                ..ImageRsProcessor::default()
            },
            None,
        )
        .unwrap();
        assert!(matches!(
            result[0],
            CompressionResult::Saved {
                width: 20,
                height: 40,
                ..
            }
        ));
        let saved = image::open(output.path().join("rotated.jpg")).unwrap();
        assert_eq!((saved.width(), saved.height()), (20, 40));
        assert_eq!(fs::read(source).unwrap(), original);
    }

    #[test]
    fn never_publishes_invalid_encoder_output() {
        let input = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let source = input.path().join("foto.png");
        RgbaImage::new(8, 8).save(&source).unwrap();
        let (_, results) = process_batch(
            vec![source.to_str().unwrap().into()],
            output.path().to_str().unwrap(),
            &InvalidEncoder,
            None,
        )
        .unwrap();
        assert!(
            matches!(&results[0], CompressionResult::Failed { error, .. } if error.code == ErrorCode::OutputWriteFailed)
        );
        assert_eq!(fs::read_dir(output.path()).unwrap().count(), 0);
    }

    #[test]
    fn writes_valid_images_in_each_format_and_preserves_alpha() {
        let folder = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let image =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(32, 24, Rgba([12, 72, 90, 120])));
        for (name, format) in [
            ("foto.jpg", image::ImageFormat::Jpeg),
            ("foto.png", image::ImageFormat::Png),
            ("foto.webp", image::ImageFormat::WebP),
        ] {
            let source = folder.path().join(name);
            if format == image::ImageFormat::Jpeg {
                DynamicImage::ImageRgb8(image.to_rgb8())
                    .save_with_format(&source, format)
                    .unwrap();
            } else {
                image.save_with_format(&source, format).unwrap();
            }
            let before = fs::read(&source).unwrap();
            let (_, results) = process_batch(
                vec![source.to_str().unwrap().into()],
                output.path().to_str().unwrap(),
                &ImageRsProcessor::default(),
                None,
            )
            .unwrap();
            assert!(
                matches!(&results[0], CompressionResult::Saved { file_name, .. } if file_name == name)
            );
            let encoded = fs::read(output.path().join(name)).unwrap();
            assert_eq!(image::guess_format(&encoded).unwrap(), format);
            let decoded = image::load_from_memory(&encoded).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (32, 24));
            if format != image::ImageFormat::Jpeg {
                assert_eq!(decoded.to_rgba8().get_pixel(0, 0).0, [12, 72, 90, 120]);
            }
            assert_eq!(fs::read(source).unwrap(), before);
        }
    }

    #[test]
    fn preserves_occupied_files_and_continues_after_corrupt_image() {
        let folder = tempfile::tempdir().unwrap();
        let original = folder.path().join("foto.png");
        let occupied = folder.path().join("foto-2.png");
        let corrupt = folder.path().join("corrupta.png");
        RgbaImage::new(8, 8).save(&original).unwrap();
        let original_bytes = fs::read(&original).unwrap();
        fs::write(&occupied, b"ocupado").unwrap();
        fs::write(&corrupt, b"\x89PNG\r\n\x1a\n").unwrap();
        let (_, results) = process_batch(
            vec![
                corrupt.to_str().unwrap().into(),
                original.to_str().unwrap().into(),
            ],
            folder.path().to_str().unwrap(),
            &ImageRsProcessor::default(),
            None,
        )
        .unwrap();
        assert!(
            matches!(&results[0], CompressionResult::Failed { error, .. } if error.code == ErrorCode::InvalidImage)
        );
        assert!(
            matches!(&results[1], CompressionResult::Saved { file_name, .. } if file_name == "foto-3.png")
        );
        assert_eq!(fs::read(original).unwrap(), original_bytes);
        assert_eq!(fs::read(occupied).unwrap(), b"ocupado");
        assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 4);
    }
}
