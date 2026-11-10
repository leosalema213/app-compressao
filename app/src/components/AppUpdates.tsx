import { useEffect, useRef } from "react";
import { Download, LoaderCircle, RefreshCw } from "lucide-react";
import { Button } from "./ui/button";
import {
  checkForAppUpdates,
  downloadAppUpdate,
  initializeAppUpdates,
  installAppUpdate,
  showAppUpdate,
  useAppUpdates,
} from "@/lib/app-updates";
import { displaySize } from "@/lib/media-contract";

export function UpdateSettings() {
  const state = useAppUpdates();
  const busy = ["checking", "downloading", "installing"].includes(state.status);
  const messages = {
    idle: "Novas versões são verificadas ao abrir o aplicativo.",
    checking: "Verificando atualizações…",
    current: "Você está na versão mais recente.",
    available: `Pacton ${state.version} está disponível.`,
    downloading: "Baixando atualização…",
    ready: "A atualização está pronta para instalar.",
    installing: "Aplicando atualização… O aplicativo será reaberto.",
    error: "Tente verificar novamente.",
  };
  return (
    <div className="settings-section">
      <h2>Versão e atualizações</h2>
      <p>
        <strong>Pacton {state.currentVersion || "—"}</strong>
      </p>
      <p role="status">{messages[state.status]}</p>
      {state.error && (
        <p className="media-error" role="alert">
          {state.error}
        </p>
      )}
      <div className="settings-actions">
        {["available", "ready", "downloading", "installing"].includes(
          state.status,
        ) ? (
          <Button
            type="button"
            className="h-8 px-3 text-xs"
            onClick={() => showAppUpdate(true)}
          >
            Ver atualização
          </Button>
        ) : (
          <Button
            type="button"
            className="h-8 px-3 text-xs"
            disabled={busy}
            onClick={() => void checkForAppUpdates()}
          >
            <RefreshCw
              size={14}
              aria-hidden="true"
              className={busy ? "animate-spin" : ""}
            />
            {busy ? "Verificando…" : "Verificar atualização"}
          </Button>
        )}
      </div>
      <p>
        O aplicativo avisa quando houver uma nova versão. Você escolhe quando
        baixar e atualizar.
      </p>
    </div>
  );
}

export function AppUpdatePrompt() {
  const state = useAppUpdates();
  const dialog = useRef<HTMLDialogElement>(null);
  const busy = ["downloading", "installing"].includes(state.status);
  const ready = state.status === "ready";
  const percent = state.total
    ? Math.min(100, Math.floor((state.received / state.total) * 100))
    : null;

  useEffect(initializeAppUpdates, []);
  useEffect(() => {
    const element = dialog.current;
    if (state.open && !element?.open) element?.showModal();
    if (!state.open && element?.open) element.close();
  }, [state.open]);

  return (
    <dialog
      className="app-update-dialog"
      ref={dialog}
      aria-labelledby="app-update-title"
      onCancel={(event) => {
        if (state.status === "installing") event.preventDefault();
        else showAppUpdate(false);
      }}
    >
      <span className="download-completion-icon">
        <Download size={22} aria-hidden="true" />
      </span>
      <h2 id="app-update-title">
        {state.status === "installing"
          ? "Atualizando o Pacton"
          : ready
            ? "Pronto para atualizar"
            : "Nova versão disponível"}
      </h2>
      <p>
        Pacton {state.currentVersion} → {state.version}
      </p>
      <p>
        {state.status === "installing"
          ? "Aplicando a atualização. O aplicativo será reaberto ao terminar."
          : ready
            ? "Seus dados serão mantidos. O aplicativo será fechado e reaberto durante a atualização."
            : busy
              ? "Você pode continuar usando o aplicativo enquanto a atualização é baixada."
              : "Deseja baixar esta atualização?"}
      </p>
      {state.notes && !busy && (
        <details>
          <summary>Novidades desta versão</summary>
          <p className="update-notes">{state.notes}</p>
        </details>
      )}
      {busy && (
        <div className="update-progress">
          <progress
            aria-label="Progresso da atualização"
            max={100}
            value={
              state.status === "installing" ? undefined : (percent ?? undefined)
            }
          />
          <span>
            {state.status === "installing"
              ? "Reiniciando…"
              : `${displaySize(state.received)}${state.total ? ` de ${displaySize(state.total)} · ${percent}%` : ""}`}
          </span>
        </div>
      )}
      {state.error && (
        <p className="media-error" role="alert">
          {state.error}
        </p>
      )}
      <div className="settings-actions">
        {state.status !== "installing" && (
          <button
            type="button"
            className="theme-trigger"
            onClick={() => showAppUpdate(false)}
          >
            {busy ? "Continuar usando" : "Depois"}
          </button>
        )}
        {!busy && (
          <Button
            type="button"
            className="h-8 px-3 text-xs"
            onClick={() =>
              void (ready ? installAppUpdate() : downloadAppUpdate())
            }
          >
            {ready ? (
              <RefreshCw size={14} aria-hidden="true" />
            ) : (
              <Download size={14} aria-hidden="true" />
            )}
            {ready ? "Atualizar e reiniciar" : "Baixar atualização"}
          </Button>
        )}
        {state.status === "installing" && (
          <LoaderCircle size={16} className="animate-spin" aria-hidden="true" />
        )}
      </div>
    </dialog>
  );
}
