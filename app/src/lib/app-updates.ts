import { useSyncExternalStore } from "react";
import { Channel, invoke, isTauri } from "@tauri-apps/api/core";
import { getAppDiagnostic } from "@/diagnostic";
import { mediaErrorMessage } from "./media-contract";

type UpdateStatus =
  | "idle"
  | "checking"
  | "current"
  | "available"
  | "downloading"
  | "ready"
  | "installing"
  | "error";

interface UpdateInfo {
  currentVersion: string;
  version: string | null;
  notes: string | null;
}

interface UpdateState extends UpdateInfo {
  status: UpdateStatus;
  received: number;
  total: number | null;
  error: string;
  open: boolean;
}

let state: UpdateState = {
  status: "idle",
  currentVersion: "",
  version: null,
  notes: null,
  received: 0,
  total: null,
  error: "",
  open: false,
};
const listeners = new Set<() => void>();
let startupChecked = false;
const busy = () =>
  ["checking", "downloading", "installing"].includes(state.status);

function update(changes: Partial<UpdateState>) {
  state = { ...state, ...changes };
  listeners.forEach((listener) => listener());
}

export function useAppUpdates() {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    () => state,
  );
}

export function showAppUpdate(open: boolean) {
  if (state.status !== "installing") update({ open });
}

export async function checkForAppUpdates() {
  if (busy() || state.status === "ready") return;
  update({ status: "checking", error: "" });
  try {
    const info = await invoke<UpdateInfo>("check_app_update");
    update({
      ...info,
      status: info.version ? "available" : "current",
      open: Boolean(info.version),
    });
  } catch (error) {
    update({ status: "error", error: mediaErrorMessage(error) });
  }
}

export function initializeAppUpdates() {
  if (!isTauri() || startupChecked) return;
  const timer = window.setTimeout(() => {
    if (startupChecked) return;
    startupChecked = true;
    void getAppDiagnostic()
      .then(({ version }) => update({ currentVersion: version }))
      .catch(() => {});
    void checkForAppUpdates();
  }, 1800);
  return () => window.clearTimeout(timer);
}

export async function downloadAppUpdate() {
  if (busy()) return;
  update({ status: "downloading", received: 0, total: null, error: "" });
  try {
    await invoke("download_app_update", {
      progress: new Channel<{ received: number; total: number | null }>(
        (progress) => update(progress),
      ),
    });
    update({ status: "ready", open: true });
  } catch (error) {
    update({ status: "available", error: mediaErrorMessage(error) });
  }
}

export async function installAppUpdate() {
  if (state.status !== "ready") return;
  update({ status: "installing", error: "" });
  try {
    await invoke("install_app_update");
  } catch (error) {
    update({ status: "ready", error: mediaErrorMessage(error) });
  }
}
