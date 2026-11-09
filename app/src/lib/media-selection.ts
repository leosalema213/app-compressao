import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AvResult,
  CompressionResult,
  ImageFormat,
  ImageJobUpdate,
  ImageOptions,
  ImagePreview,
  MediaDetails,
  MediaInspection,
  PlannedOutput,
  StreamKind,
  VideoPreset,
} from "./media-contract";

export function chooseImages() {
  return open({
    title: "Selecionar imagens",
    multiple: true,
    filters: [{ name: "Imagens", extensions: ["jpg", "jpeg", "png", "webp"] }],
  });
}

export function inspectImages(paths: string[]) {
  return invoke<MediaInspection[]>("inspect_images", { paths });
}

export function chooseOutputFolder() {
  return open({ title: "Selecionar pasta de saída", directory: true });
}

export function validateOutputFolder(path: string) {
  return invoke<string>("validate_output_folder", { path });
}

export function openImageOutputFolder(folder: string) {
  return invoke<void>("open_image_output_folder", { folder });
}

export function previewImageOutputs(
  paths: string[],
  folder: string,
  format?: ImageFormat,
) {
  return invoke<PlannedOutput[]>("preview_image_outputs", {
    paths,
    folder,
    ...(format && { format }),
  });
}

export function compressImages(
  paths: string[],
  folder: string,
  options: ImageOptions = {},
  onProgress?: (update: ImageJobUpdate) => void,
) {
  const progress = onProgress && new Channel<ImageJobUpdate>(onProgress);
  return invoke<CompressionResult[]>("compress_images", {
    paths,
    folder,
    ...options,
    ...(progress && { progress }),
  });
}

export function previewImageConversion(path: string, options: ImageOptions) {
  return invoke<ImagePreview>("preview_image_conversion", { path, ...options });
}

export function cancelImageJob(id: number) {
  return invoke<boolean>("cancel_image_job", { id });
}

export function chooseAvFile(kind: StreamKind) {
  const video = ["mp4", "mov", "mkv", "webm", "avi"];
  const audio = ["mp3", "m4a", "wav", "flac", "ogg", "opus", "aac"];
  return open({
    title: kind === "video" ? "Selecionar vídeo" : "Selecionar áudio ou vídeo",
    filters: [
      {
        name: "Arquivos de mídia",
        extensions: kind === "video" ? video : [...audio, ...video],
      },
    ],
  });
}

export function inspectAvFile(path: string, kind: StreamKind) {
  return invoke<MediaDetails>("inspect_av_file", { path, kind });
}

export function transcodeAv(
  path: string,
  folder: string,
  kind: StreamKind,
  preset: VideoPreset,
  onProgress: (update: ImageJobUpdate) => void,
  targetPercent?: number,
) {
  return invoke<AvResult>("transcode_av", {
    path,
    folder,
    kind,
    preset,
    ...(targetPercent !== undefined && { targetPercent }),
    progress: new Channel<ImageJobUpdate>(onProgress),
  });
}

export async function listenForImageDrops(onDrop: (paths: string[]) => void) {
  if (!isTauri()) return;
  return getCurrentWebview().onDragDropEvent(({ payload }) => {
    if (payload.type === "drop") onDrop(payload.paths);
  });
}
