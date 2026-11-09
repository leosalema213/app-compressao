import { useSyncExternalStore } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { mediaErrorMessage, type ImageJobUpdate } from "./media-contract";

export interface DownloadResult {
  fileName: string;
  bytes: number;
}

type DownloadReport =
  | DownloadResult
  | { files: DownloadResult[]; failed: number; cancelled: number };

export interface DownloadEntry {
  id: number;
  title: string;
  cover: string | null;
  job?: ImageJobUpdate;
  result?: DownloadReport;
  error?: string;
  actionError?: string;
  controlPending?: boolean;
}

let nextId = 0;
let state = { entries: [] as DownloadEntry[], open: false, notice: 0 };
const listeners = new Set<() => void>();

function update(next: Partial<typeof state>) {
  state = { ...state, ...next };
  listeners.forEach((listener) => listener());
}

function patch(
  id: number,
  changes: Partial<DownloadEntry>,
  notice = state.notice,
) {
  update({
    entries: state.entries.map((entry) =>
      entry.id === id ? { ...entry, ...changes } : entry,
    ),
    notice,
  });
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function useDownloads() {
  return useSyncExternalStore(subscribe, () => state);
}

export function showDownloads(open: boolean) {
  update({ open });
}

export function startDownload(
  title: string,
  cover: string | null,
  request: {
    url: string;
    folder: string;
    kind: string;
    options: Record<string, string | undefined>;
  },
) {
  const id = ++nextId;
  update({ open: true, entries: [{ id, title, cover }, ...state.entries] });
  const progress = new Channel<ImageJobUpdate>((job) => {
    const previous = state.entries.find((entry) => entry.id === id)?.job;
    patch(id, {
      job: { ...job, transfer: job.transfer ?? previous?.transfer },
    });
  });
  void invoke<DownloadReport>("download_media", { ...request, progress })
    .then((result) => patch(id, { result }, id))
    .catch((error) => patch(id, { error: mediaErrorMessage(error) }, id));
  return id;
}

export async function controlDownload(
  entry: DownloadEntry,
  action: "pause" | "resume" | "cancel",
) {
  if (!entry.job || entry.controlPending) return;
  const { id } = entry.job;
  patch(entry.id, { controlPending: true, actionError: undefined });
  try {
    const accepted =
      action === "cancel"
        ? await invoke<boolean>("cancel_image_job", { id })
        : await invoke<boolean>("pause_download_job", {
            id,
            paused: action === "pause",
          });
    const current = state.entries.find((item) => item.id === entry.id);
    if (
      accepted &&
      current?.job &&
      !current.result &&
      !current.error &&
      ["running", "paused", "queued"].includes(current.job.status)
    ) {
      patch(entry.id, {
        job: {
          ...current.job,
          status: { pause: "paused", resume: "running", cancel: "cancelling" }[
            action
          ] as ImageJobUpdate["status"],
        },
      });
    }
  } catch (error) {
    patch(entry.id, { actionError: mediaErrorMessage(error) });
  } finally {
    patch(entry.id, { controlPending: false });
  }
}

export async function openDownload(entry: DownloadEntry, file: boolean) {
  if (!entry.job) return;
  patch(entry.id, { actionError: undefined });
  try {
    await invoke("open_download_output", { id: entry.job.id, file });
  } catch (error) {
    patch(entry.id, { actionError: mediaErrorMessage(error) });
  }
}

export function downloadSummary(entry: DownloadEntry) {
  if (entry.error)
    return entry.job?.status === "cancelled"
      ? "Download cancelado"
      : "Download interrompido";
  if (!entry.result) return "Download em andamento";
  if ("fileName" in entry.result) return "Download concluído";
  if (entry.result.cancelled) return "Playlist cancelada";
  if (entry.result.failed)
    return entry.result.files.length
      ? "Playlist concluída com falhas"
      : "Nenhum item foi baixado";
  return "Download concluído";
}
