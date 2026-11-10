import { useEffect, useState, type MouseEvent } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";

export function dragWindow(event: MouseEvent<HTMLElement>) {
  if (
    !isTauri() ||
    event.button !== 0 ||
    (event.target instanceof Element &&
      event.target.closest(".topbar-controls, a, button"))
  )
    return;
  const current = getCurrentWindow();
  const action =
    event.detail === 2 ? current.toggleMaximize() : current.startDragging();
  void action.catch(() => {});
}

export function WindowControls() {
  const [maximized, setMaximized] = useState(false);
  const [error, setError] = useState("");
  const native = isTauri();

  useEffect(() => {
    if (!native) return;
    const current = getCurrentWindow();
    let active = true;
    const sync = () =>
      void current
        .isMaximized()
        .then((value) => {
          if (active) setMaximized(value);
        })
        .catch(() => {});
    sync();
    const subscription = current.onResized(sync).catch(() => undefined);
    return () => {
      active = false;
      void subscription.then((unlisten) => unlisten?.());
    };
  }, [native]);

  async function run(action: () => Promise<void>) {
    setError("");
    try {
      await action();
    } catch {
      setError("Não foi possível controlar a janela. Tente novamente.");
    }
  }

  return (
    <div
      className="window-controls"
      role="group"
      aria-label="Controles da janela"
    >
      <span className="window-separator" aria-hidden="true" />
      <button
        className="window-button"
        type="button"
        aria-label="Minimizar janela"
        title="Minimizar janela"
        disabled={!native}
        onClick={() => void run(() => getCurrentWindow().minimize())}
      >
        <Minus size={16} aria-hidden="true" />
      </button>
      <button
        className="window-button"
        type="button"
        aria-label={maximized ? "Restaurar janela" : "Maximizar janela"}
        title={maximized ? "Restaurar janela" : "Maximizar janela"}
        disabled={!native}
        onClick={() => void run(() => getCurrentWindow().toggleMaximize())}
      >
        {maximized ? (
          <Copy size={13} aria-hidden="true" />
        ) : (
          <Square size={13} aria-hidden="true" />
        )}
      </button>
      <button
        className="window-button window-close"
        type="button"
        aria-label="Fechar aplicativo"
        title="Fechar aplicativo"
        disabled={!native}
        onClick={() => void run(() => getCurrentWindow().close())}
      >
        <X size={17} aria-hidden="true" />
      </button>
      {error && (
        <p role="alert" className="surface window-error">
          {error}
        </p>
      )}
    </div>
  );
}
