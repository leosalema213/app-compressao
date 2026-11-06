import { useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { readSettings } from "@/lib/settings";
import { mediaErrorMessage, type ImageJobUpdate } from "@/lib/media-contract";
import {
  cancelImageJob,
  chooseOutputFolder,
  validateOutputFolder,
} from "@/lib/media-selection";

interface DownloadInfo {
  title: string;
  duration: number | null;
  heights: number[];
  has_thumbnail: boolean;
  subtitle_languages: string[];
  playlist_size: number | null;
}

type DownloadKind = "video" | "audio" | "thumbnail" | "subtitles";

interface DownloadResult {
  fileName: string;
  bytes: number;
}

type DownloadReport =
  | DownloadResult
  | { files: DownloadResult[]; failed: number; cancelled: number };

export function DownloaderPanel() {
  const [url, setUrl] = useState("");
  const [folder, setFolder] = useState(() => readSettings().outputFolder);
  const [kind, setKind] = useState<DownloadKind>("video");
  const [quality, setQuality] = useState("best");
  const [language, setLanguage] = useState("");
  const [authorized, setAuthorized] = useState(false);
  const [info, setInfo] = useState<DownloadInfo | null>(null);
  const [result, setResult] = useState<DownloadReport | null>(null);
  const [confirmedPlaylist, setConfirmedPlaylist] = useState(false);
  const [busy, setBusy] = useState("");
  const [error, setError] = useState("");
  const [job, setJob] = useState<ImageJobUpdate | null>(null);
  const [cancelling, setCancelling] = useState(false);

  async function run(action: () => Promise<void>, message: string) {
    setBusy(message);
    setError("");
    try {
      await action();
    } catch (failure) {
      setError(mediaErrorMessage(failure));
    } finally {
      setBusy("");
    }
  }

  function inspect() {
    return run(async () => {
      setInfo(null);
      setResult(null);
      setConfirmedPlaylist(false);
      const details = await invoke<DownloadInfo>("inspect_download", { url });
      setKind("video");
      setQuality("best");
      setLanguage(details.subtitle_languages[0] ?? "");
      setInfo(details);
    }, "Consultando o link…");
  }

  function selectFolder() {
    return run(async () => {
      const selected = await chooseOutputFolder();
      if (selected) {
        setFolder(await validateOutputFolder(selected));
        setConfirmedPlaylist(false);
      }
    }, "Aguardando escolha de pasta…");
  }

  function download() {
    return run(async () => {
      setResult(null);
      setCancelling(false);
      let active = true;
      try {
        const progress = new Channel<ImageJobUpdate>((update) => {
          if (active) setJob(update);
        });
        setResult(
          await invoke<DownloadReport>("download_media", {
            url,
            folder,
            kind,
            authorized,
            options: {
              quality,
              language: language || undefined,
              confirmedCount: confirmedPlaylist
                ? (info?.playlist_size ?? undefined)
                : undefined,
            },
            progress,
          }),
        );
      } finally {
        active = false;
        setJob(null);
      }
    }, "Baixando mídia…");
  }

  async function cancel() {
    if (!job || cancelling) return;
    setCancelling(true);
    try {
      await cancelImageJob(job.id);
    } catch (failure) {
      setCancelling(false);
      setError(mediaErrorMessage(failure));
    }
  }

  return (
    <section className="surface" aria-labelledby="download-title">
      <h2 id="download-title">Mídia autorizada</h2>
      <p>Use apenas links de arquivos que você tem autorização para baixar.</p>
      <fieldset disabled={Boolean(busy)}>
        <div className="resize-fields">
          <label className="w-full">
            Link
            <input
              className="download-url"
              type="url"
              value={url}
              onChange={(event) => {
                setUrl(event.target.value);
                setInfo(null);
                setResult(null);
                setAuthorized(false);
                setConfirmedPlaylist(false);
              }}
            />
          </label>
        </div>
        <Button
          type="button"
          className="mt-3 h-8 px-3 text-xs"
          disabled={!url.trim()}
          onClick={inspect}
        >
          Consultar link
        </Button>
        {info && (
          <div className="mt-3">
            <p className="media-folder">
              <strong>{info.title}</strong>
            </p>
            {info.duration !== null && (
              <p>
                {info.duration.toLocaleString("pt-BR", {
                  maximumFractionDigits: 2,
                })}{" "}
                s
              </p>
            )}
            {info.playlist_size !== null && (
              <p>Playlist · {info.playlist_size} itens</p>
            )}
            <div className="resize-fields">
              <label>
                Saída
                <select
                  value={kind}
                  onChange={(event) =>
                    setKind(event.target.value as DownloadKind)
                  }
                >
                  <option value="video">Vídeo MP4</option>
                  <option value="audio">Áudio MP3</option>
                  {info.has_thumbnail && (
                    <option value="thumbnail">Miniatura PNG</option>
                  )}
                  {info.subtitle_languages.length > 0 && (
                    <option value="subtitles">Legenda WebVTT</option>
                  )}
                </select>
              </label>
              {kind === "video" && info.playlist_size === null && (
                <label>
                  Qualidade
                  <select
                    value={quality}
                    onChange={(event) => setQuality(event.target.value)}
                  >
                    <option value="best">Melhor disponível</option>
                    {info.heights.map((height) => (
                      <option key={height} value={`p${height}`}>
                        {height}p
                      </option>
                    ))}
                  </select>
                </label>
              )}
              {kind === "subtitles" && (
                <label>
                  Idioma
                  <select
                    value={language}
                    onChange={(event) => setLanguage(event.target.value)}
                  >
                    {info.subtitle_languages.map((item) => (
                      <option key={item} value={item}>
                        {item}
                      </option>
                    ))}
                  </select>
                </label>
              )}
            </div>
            <Button
              type="button"
              className="mt-3 h-8 px-3 text-xs"
              onClick={selectFolder}
            >
              Escolher pasta
            </Button>
            <p className="media-folder">
              {folder || "Nenhuma pasta selecionada"}
            </p>
            <label className="mt-2 flex items-center gap-2 text-xs">
              <input
                type="checkbox"
                checked={authorized}
                onChange={(event) => setAuthorized(event.target.checked)}
              />
              Tenho autorização para baixar esta mídia
            </label>
            {info.playlist_size !== null && folder && (
              <label className="mt-2 flex items-center gap-2 text-xs">
                <input
                  type="checkbox"
                  checked={confirmedPlaylist}
                  onChange={(event) =>
                    setConfirmedPlaylist(event.target.checked)
                  }
                />
                Confirmo baixar {info.playlist_size} itens para a pasta
                selecionada
              </label>
            )}
            <Button
              type="button"
              className="mt-3 h-8 px-3 text-xs"
              disabled={
                !folder ||
                !authorized ||
                (kind === "subtitles" && !language) ||
                (info.playlist_size !== null && !confirmedPlaylist)
              }
              onClick={download}
            >
              {info.playlist_size !== null ? "Baixar playlist" : "Baixar"}
            </Button>
          </div>
        )}
      </fieldset>
      {busy && (
        <p role="status">
          {cancelling && job
            ? "Cancelando…"
            : job?.status === "queued"
              ? "Na fila"
              : busy}
        </p>
      )}
      {job && (
        <div className="media-actions">
          <progress
            aria-label="Progresso do download"
            max={100}
            value={job.completed || undefined}
          />
          {(job.status === "queued" || job.status === "running") && (
            <button
              type="button"
              className="media-text-button"
              disabled={cancelling}
              onClick={cancel}
            >
              Cancelar
            </button>
          )}
        </div>
      )}
      {result && "fileName" in result && (
        <p role="status">
          {result.fileName} · {result.bytes.toLocaleString("pt-BR")} bytes
          gravados
        </p>
      )}
      {result && "files" in result && (
        <div role="status">
          <p>
            {result.files.length}{" "}
            {result.files.length === 1 ? "gravado" : "gravados"} ·{" "}
            {result.failed} {result.failed === 1 ? "falha" : "falhas"} ·{" "}
            {result.cancelled}{" "}
            {result.cancelled === 1 ? "cancelado" : "cancelados"}
          </p>
          <ul className="media-list">
            {result.files.map((file) => (
              <li key={file.fileName}>
                <span className="media-folder">{file.fileName}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {error && (
        <p className="media-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
