import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Check,
  Download,
  File,
  FolderOpen,
  Link2,
  ListMusic,
  LoaderCircle,
  Search,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  downloadSummary,
  openDownload,
  startDownload,
  useDownloads,
} from "@/lib/downloads";
import { mediaErrorMessage } from "@/lib/media-contract";
import { chooseOutputFolder } from "@/lib/media-selection";

interface DownloadInfo {
  title: string;
  duration: number | null;
  heights: number[];
  has_thumbnail: boolean;
  subtitle_languages: string[];
  playlist_size: number | null;
  cover_url: string | null;
}

type DownloadKind = "video" | "audio" | "thumbnail" | "subtitles";

function durationLabel(seconds: number) {
  const time = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(time / 3600);
  const minutes = Math.floor(time / 60) % 60;
  return [
    hours ? `${hours} h` : "",
    minutes || hours ? `${minutes} min` : "",
    `${time % 60} s`,
  ]
    .filter(Boolean)
    .join(" ");
}

export function DownloaderPanel() {
  const [url, setUrl] = useState("");
  const [kind, setKind] = useState<DownloadKind>("video");
  const [quality, setQuality] = useState("best");
  const [language, setLanguage] = useState("");
  const [namePrefix, setNamePrefix] = useState("");
  const [coverFailed, setCoverFailed] = useState(false);
  const [info, setInfo] = useState<DownloadInfo | null>(null);
  const [busy, setBusy] = useState<"inspect" | "folder" | null>(null);
  const [error, setError] = useState("");
  const [currentId, setCurrentId] = useState<number | null>(null);
  const { entries } = useDownloads();
  const current = entries.find(({ id }) => id === currentId);
  const saved =
    current?.result &&
    ("fileName" in current.result || current.result.files.length > 0);
  const count = info?.playlist_size ?? null;
  const prefix = namePrefix.trim();
  const invalidName = /[<>:"/\\|?*]/.test(prefix) || prefix.endsWith(".");
  const extension = {
    video: "mp4",
    audio: "mp3",
    thumbnail: "png",
    subtitles: "vtt",
  }[kind];

  async function run(action: () => Promise<void>, mode: "inspect" | "folder") {
    setBusy(mode);
    setError("");
    try {
      await action();
    } catch (failure) {
      setError(mediaErrorMessage(failure));
    } finally {
      setBusy(null);
    }
  }

  function inspect() {
    return run(async () => {
      setInfo(null);
      setCurrentId(null);
      setCoverFailed(false);
      const details = await invoke<DownloadInfo>("inspect_download", { url });
      setKind("video");
      setQuality("best");
      setLanguage(details.subtitle_languages[0] ?? "");
      setInfo(details);
    }, "inspect");
  }

  function download() {
    if (!info) return;
    return run(async () => {
      const folder = await chooseOutputFolder();
      if (!folder) return;
      setCurrentId(
        startDownload(info.title, info.cover_url, {
          url,
          folder,
          kind,
          options: {
            quality,
            language: language || undefined,
            namePrefix: prefix || undefined,
          },
        }),
      );
    }, "folder");
  }

  return (
    <section className="downloader-panel" aria-label="Downloader">
      <fieldset className="downloader-fields" disabled={Boolean(busy)}>
        <form
          className="surface download-search"
          onSubmit={(event) => {
            event.preventDefault();
            if (url.trim()) void inspect();
          }}
        >
          <label htmlFor="download-url">
            <Link2 size={16} aria-hidden="true" /> Link da mídia
          </label>
          <div className="download-search-row">
            <input
              id="download-url"
              type="url"
              value={url}
              placeholder="Cole o link do vídeo ou da playlist"
              onChange={(event) => {
                setUrl(event.target.value);
                setInfo(null);
                setCurrentId(null);
              }}
            />
            <Button
              type="submit"
              className="h-10 px-3 text-xs"
              disabled={!url.trim()}
            >
              {busy === "inspect" ? (
                <>
                  <LoaderCircle
                    size={15}
                    className="animate-spin"
                    aria-hidden="true"
                  />{" "}
                  Consultando…
                </>
              ) : (
                <>
                  <Search size={15} aria-hidden="true" /> Consultar link
                </>
              )}
            </Button>
          </div>
        </form>
        {info && (
          <div className="download-grid">
            <div className="surface download-card">
              <div className="download-cover">
                {info.cover_url && !coverFailed ? (
                  <img
                    src={info.cover_url}
                    alt=""
                    referrerPolicy="no-referrer"
                    onError={() => setCoverFailed(true)}
                  />
                ) : (
                  <ListMusic size={32} aria-hidden="true" />
                )}
              </div>
              <div className="download-meta">
                {count !== null && (
                  <span className="download-badge">
                    Playlist · {count} {count === 1 ? "item" : "itens"}
                  </span>
                )}
                <h2>{info.title}</h2>
                {count === null && info.duration !== null && (
                  <p>{durationLabel(info.duration)}</p>
                )}
              </div>
            </div>
            <div className="surface download-options">
              {current ? (
                <div className="download-completion">
                  <span className="download-completion-icon">
                    {saved ? (
                      <Check size={23} aria-hidden="true" />
                    ) : (
                      <Download size={23} aria-hidden="true" />
                    )}
                  </span>
                  <h2>{downloadSummary(current)}</h2>
                  {current.error && <p>{current.error}</p>}
                  {current.result && "files" in current.result && (
                    <p>
                      {current.result.files.length} de {count} itens baixados
                    </p>
                  )}
                  {current.actionError && (
                    <p role="alert">{current.actionError}</p>
                  )}
                  <div className="download-completion-actions">
                    {saved && (
                      <button
                        type="button"
                        className="theme-trigger"
                        onClick={() => void openDownload(current, false)}
                      >
                        <FolderOpen size={15} aria-hidden="true" /> Abrir pasta
                      </button>
                    )}
                    {current.result && "fileName" in current.result && (
                      <button
                        type="button"
                        className="theme-trigger"
                        onClick={() => void openDownload(current, true)}
                      >
                        <File size={15} aria-hidden="true" /> Abrir arquivo
                      </button>
                    )}
                    <button
                      type="button"
                      className="media-text-button"
                      onClick={() => {
                        setUrl("");
                        setInfo(null);
                        setCurrentId(null);
                        setNamePrefix("");
                        setError("");
                      }}
                    >
                      Novo download
                    </button>
                  </div>
                </div>
              ) : (
                <>
                  <h2>Opções de download</h2>
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
                  <div className="download-name resize-fields">
                    <label>
                      Nome base (opcional)
                      <input
                        value={namePrefix}
                        maxLength={90}
                        placeholder="Ex.: Jazz Wall"
                        aria-invalid={invalidName || undefined}
                        aria-describedby="download-name-hint"
                        onChange={(event) => setNamePrefix(event.target.value)}
                      />
                    </label>
                  </div>
                  <p
                    id="download-name-hint"
                    className={invalidName ? "media-error" : ""}
                  >
                    {invalidName
                      ? "Use um nome de arquivo válido no Windows."
                      : prefix
                        ? count !== null && count > 1
                          ? `${prefix} 1.${extension}, ${prefix} 2.${extension}…`
                          : `${prefix}${count === 1 ? " 1" : ""}.${extension}`
                        : "Em branco, usa o título de cada item."}
                  </p>
                  <Button
                    type="button"
                    className="mt-3 h-8 px-3 text-xs"
                    disabled={
                      invalidName || (kind === "subtitles" && !language)
                    }
                    onClick={download}
                  >
                    <Download size={14} aria-hidden="true" />
                    {info.playlist_size !== null ? "Baixar playlist" : "Baixar"}
                  </Button>
                </>
              )}
            </div>
          </div>
        )}
      </fieldset>
      {error && (
        <p className="media-error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
