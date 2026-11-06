import { invoke } from "@tauri-apps/api/core";

export interface AppDiagnostic {
  version: string;
}

export function getAppDiagnostic(): Promise<AppDiagnostic> {
  return invoke<AppDiagnostic>("get_app_diagnostic");
}

export interface ToolReport {
  name: "ffmpeg" | "ffprobe" | "yt-dlp" | "node" | "deno";
  status: "available" | "missing" | "unavailable";
  version: string | null;
}

export function diagnoseMediaTools() {
  return invoke<ToolReport[]>("diagnose_media_tools");
}
