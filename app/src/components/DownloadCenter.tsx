import { useEffect, useRef, useState } from "react";
import {
  Check,
  Download,
  File,
  FolderOpen,
  Pause,
  Play,
  X,
} from "lucide-react";
import {
  controlDownload,
  downloadSummary,
  openDownload,
  showDownloads,
  useDownloads,
} from "@/lib/downloads";
import { displaySize } from "@/lib/media-contract";

export function DownloadCenter() {
  const { entries, open, notice } = useDownloads();
  const [visibleNotice, setVisibleNotice] = useState(0);
  const container = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const active = entries.filter(
    (entry) => !entry.result && !entry.error,
  ).length;
  const announced = entries.find((entry) => entry.id === visibleNotice);

  useEffect(() => {
    if (!notice) return;
    const start = window.setTimeout(() => setVisibleNotice(notice), 0);
    const end = window.setTimeout(() => setVisibleNotice(0), 5500);
    return () => {
      window.clearTimeout(start);
      window.clearTimeout(end);
    };
  }, [notice]);

  useEffect(() => {
    if (!open) return;
    function outside(event: PointerEvent) {
      if (
        event.target instanceof Node &&
        !container.current?.contains(event.target)
      )
        showDownloads(false);
    }
    document.addEventListener("pointerdown", outside);
    return () => document.removeEventListener("pointerdown", outside);
  }, [open]);

  return (
    <div
      className="download-center"
      ref={container}
      onKeyDown={(event) => {
        if (open && event.key === "Escape") {
          showDownloads(false);
          trigger.current?.focus();
        }
      }}
    >
      <button
        type="button"
        ref={trigger}
        className="theme-trigger download-trigger"
        aria-label="Downloads"
        aria-expanded={open}
        onClick={() => showDownloads(!open)}
      >
        <Download size={16} aria-hidden="true" />
        {active > 0 && <span className="download-count">{active}</span>}
      </button>
      {open && (
        <section
          className="theme-popup downloads-popup"
          aria-label="Lista de downloads"
        >
          <div className="media-actions">
            <strong>Downloads</strong>
            <button
              type="button"
              className="download-icon"
              aria-label="Fechar downloads"
              onClick={() => {
                showDownloads(false);
                trigger.current?.focus();
              }}
            >
              <X size={15} aria-hidden="true" />
            </button>
          </div>
          {entries.length === 0 && <p>Nenhum download nesta sessão.</p>}
          <ul>
            {entries.map((entry) => {
              const ongoing = !entry.result && !entry.error;
              const status = entry.job?.status;
              const paused = status === "paused";
              const transfer = entry.job?.transfer;
              const saved =
                entry.result &&
                ("fileName" in entry.result || entry.result.files.length > 0);
              return (
                <li key={entry.id} className="download-entry">
                  <div className="download-entry-main">
                    <div className="download-entry-cover">
                      {entry.cover ? (
                        <img
                          src={entry.cover}
                          alt=""
                          referrerPolicy="no-referrer"
                          onError={(event) => {
                            event.currentTarget.hidden = true;
                          }}
                        />
                      ) : (
                        <File size={16} aria-hidden="true" />
                      )}
                    </div>
                    <div className="download-entry-text">
                      <strong title={entry.title}>{entry.title}</strong>
                      <span>
                        {ongoing
                          ? paused
                            ? "Pausado"
                            : status === "queued"
                              ? "Na fila"
                              : status === "cancelling"
                                ? "Cancelando…"
                                : entry.job?.completed
                                  ? `${entry.job.completed}%`
                                  : "Iniciando…"
                          : downloadSummary(entry)}
                      </span>
                    </div>
                    {ongoing && (status === "running" || paused) && (
                      <button
                        type="button"
                        className="download-icon"
                        aria-label={paused ? "Retomar" : "Pausar"}
                        title={paused ? "Retomar" : "Pausar"}
                        disabled={entry.controlPending}
                        onClick={() =>
                          void controlDownload(
                            entry,
                            paused ? "resume" : "pause",
                          )
                        }
                      >
                        {paused ? (
                          <Play size={15} aria-hidden="true" />
                        ) : (
                          <Pause size={15} aria-hidden="true" />
                        )}
                      </button>
                    )}
                    {ongoing && entry.job && status !== "cancelling" && (
                      <button
                        type="button"
                        className="download-icon"
                        aria-label="Cancelar download"
                        title="Cancelar download"
                        disabled={entry.controlPending}
                        onClick={() => void controlDownload(entry, "cancel")}
                      >
                        <X size={15} aria-hidden="true" />
                      </button>
                    )}
                    {saved && (
                      <button
                        type="button"
                        className="download-icon"
                        aria-label="Abrir pasta"
                        title="Abrir pasta"
                        onClick={() => void openDownload(entry, false)}
                      >
                        <FolderOpen size={16} aria-hidden="true" />
                      </button>
                    )}
                  </div>
                  {ongoing && (
                    <>
                      <progress
                        aria-label={`Progresso de ${entry.title}`}
                        max={100}
                        value={entry.job?.completed || (paused ? 0 : undefined)}
                      />
                      {transfer && (
                        <small>
                          {displaySize(transfer.downloadedBytes)}
                          {transfer.totalBytes
                            ? ` de ${displaySize(transfer.totalBytes)}`
                            : ""}{" "}
                          · transferência atual
                        </small>
                      )}
                    </>
                  )}
                  {entry.actionError && <p role="alert">{entry.actionError}</p>}
                </li>
              );
            })}
          </ul>
        </section>
      )}
      {announced && (
        <div className="surface download-toast" role="status">
          {announced.result &&
          downloadSummary(announced) === "Download concluído" ? (
            <Check size={18} aria-hidden="true" />
          ) : (
            <Download size={18} aria-hidden="true" />
          )}
          <div>
            <strong>{downloadSummary(announced)}</strong>
            <p>{announced.title}</p>
          </div>
          <button
            type="button"
            className="download-icon"
            aria-label="Fechar aviso"
            onClick={() => setVisibleNotice(0)}
          >
            <X size={15} aria-hidden="true" />
          </button>
        </div>
      )}
    </div>
  );
}
