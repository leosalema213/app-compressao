export type MediaKind = "image";
export type ImageFormat = "jpeg" | "png" | "webp";

export interface SelectedMedia {
  kind: MediaKind;
  path: string;
  name: string;
  sizeBytes: number;
  format: ImageFormat;
  thumbnail?: string;
  dimensions?: [number, number];
}

const errorMessages = {
  invalid_path: "O caminho selecionado é inválido.",
  not_found: "O item não foi encontrado.",
  unreadable: "Não foi possível ler o item.",
  not_file: "O item selecionado não é um arquivo.",
  not_directory: "O item selecionado não é uma pasta.",
  empty_file: "O arquivo está vazio.",
  unsupported_extension: "Selecione uma imagem JPEG, PNG ou WebP.",
  format_mismatch: "O conteúdo do arquivo não corresponde à extensão.",
  invalid_output_name: "O nome de saída não é válido no Windows.",
  output_names_exhausted: "Não foi possível encontrar um nome de saída livre.",
  image_too_large:
    "A imagem ultrapassa o limite de tamanho para processamento.",
  invalid_image: "Não foi possível decodificar a imagem.",
  animated_image: "Imagens animadas ainda não podem ser processadas.",
  output_write_failed: "Não foi possível gravar ou validar o arquivo de saída.",
  invalid_resize_bounds:
    "Informe pelo menos um limite inteiro de 1 a 16.384 pixels.",
  invalid_jpeg_quality: "A qualidade JPEG deve estar entre 1 e 100.",
  output_folder_unavailable:
    "A pasta solicitada não corresponde à última saída gravada.",
  output_folder_open_failed:
    "Não foi possível abrir a pasta de saída no Explorador.",
  job_cancelled: "Processamento cancelado.",
  media_tool_missing: "O FFmpeg ou o FFprobe não foi encontrado no PATH.",
  media_probe_failed: "Não foi possível ler as informações deste arquivo.",
  media_stream_missing: "O arquivo não contém a faixa de mídia necessária.",
  unsupported_media_extension:
    "Este formato de vídeo ou áudio ainda não é aceito.",
  av_processing_failed: "O FFmpeg não conseguiu gerar o arquivo.",
  video_target_unavailable:
    "Para definir um tamanho-alvo, use um vídeo com duração conhecida e uma porcentagem de 1 a 100.",
  video_target_too_small:
    "O tamanho-alvo é pequeno demais para este vídeo. Aumente a porcentagem ou use um perfil.",
  process_timed_out: "O processamento excedeu o tempo limite.",
  invalid_download_url: "Informe uma URL HTTP ou HTTPS sem credenciais.",
  downloader_missing:
    "O yt-dlp não foi encontrado. Instale o yt-dlp oficial para consultar links.",
  download_failed: "Não foi possível consultar ou baixar a mídia.",
  download_output_open_failed:
    "Não foi possível abrir a pasta ou o arquivo baixado.",
  download_unsupported:
    "Use mídia gravada ou uma playlist com itens disponíveis.",
  download_quality_unavailable:
    "A qualidade selecionada não está mais disponível.",
  download_extra_unavailable:
    "O link não oferece a miniatura ou a legenda solicitada.",
  internal: "Não foi possível concluir a operação. Tente novamente.",
};

export interface AppError {
  code: keyof typeof errorMessages;
}

export type MediaInspection =
  | { status: "selected"; media: SelectedMedia }
  | { status: "rejected"; path: string; error: AppError };

export interface PlannedOutput {
  inputPath: string;
  fileName: string;
}

export interface ResizeBounds {
  width?: number;
  height?: number;
  percent?: number;
  exact?: boolean;
}

export interface ImageOptions {
  resize?: ResizeBounds;
  format?: ImageFormat;
  jpegQuality?: number;
}

export interface ImagePreview {
  inputBytes: number;
  outputBytes: number;
  width: number;
  height: number;
  original: string;
  output: string;
}

export type CompressionResult =
  | {
      status: "saved";
      inputPath: string;
      fileName: string;
      inputBytes: number;
      outputBytes: number;
      width: number;
      height: number;
    }
  | { status: "failed"; inputPath: string; error: AppError }
  | { status: "cancelled"; inputPath: string };

export interface ImageJobUpdate {
  id: number;
  status:
    | "queued"
    | "running"
    | "paused"
    | "cancelling"
    | "finished"
    | "failed"
    | "cancelled";
  completed: number;
  total: number;
  transfer?: { downloadedBytes: number; totalBytes: number | null };
}

export type StreamKind = "video" | "audio";
export type VideoPreset = "fast" | "balanced" | "smaller";

export interface AvResult {
  fileName: string;
  inputBytes: number;
  outputBytes: number;
}

export interface MediaDetails {
  name: string;
  path: string;
  sizeBytes: number;
  durationSeconds: number | null;
  videoCodec: string | null;
  audioCodec: string | null;
  width: number | null;
  height: number | null;
}

export function displaySize(bytes: number) {
  const unit = bytes >= 1024 ** 2 ? "MB" : bytes >= 1024 ? "KB" : "bytes";
  const divisor = unit === "MB" ? 1024 ** 2 : unit === "KB" ? 1024 : 1;
  return `${(bytes / divisor).toLocaleString("pt-BR", { maximumFractionDigits: 2 })} ${unit}`;
}

export function mediaErrorMessage(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    typeof error.code === "string" &&
    Object.prototype.hasOwnProperty.call(errorMessages, error.code)
  ) {
    return errorMessages[error.code as AppError["code"]];
  }

  return errorMessages.internal;
}
