import { useEffect, useRef, useState, type ReactNode } from "react";
import { SettingsBackup } from "./SettingsBackup";
import { UpdateSettings } from "./AppUpdates";
import { Button } from "@/components/ui/button";
import { mediaErrorMessage } from "@/lib/media-contract";
import {
  chooseOutputFolder,
  validateOutputFolder,
} from "@/lib/media-selection";
import {
  defaultSettings,
  readSettings,
  saveSettings,
  subscribeToSettings,
} from "@/lib/settings";

const tabs = [
  { id: "general", label: "Geral" },
  { id: "backup", label: "Backup" },
  { id: "updates", label: "Atualizações" },
  { id: "diagnostic", label: "Diagnóstico" },
] as const;

export function PreferencesPanel({ diagnostic }: { diagnostic?: ReactNode }) {
  const [tab, setTab] = useState<(typeof tabs)[number]["id"]>("general");
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const [settings, setSettings] = useState(readSettings);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  useEffect(() => subscribeToSettings(setSettings), []);

  async function chooseFolder() {
    setBusy(true);
    setError("");
    try {
      const chosen = await chooseOutputFolder();
      if (chosen)
        saveSettings({ outputFolder: await validateOutputFolder(chosen) });
    } catch (failure) {
      setError(mediaErrorMessage(failure));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="surface settings-panel" aria-label="Preferências">
      <div
        className="image-sections settings-tabs"
        role="tablist"
        aria-label="Categorias de preferências"
      >
        {tabs.map(({ id, label }, index) => (
          <button
            type="button"
            key={id}
            ref={(element) => {
              tabRefs.current[index] = element;
            }}
            role="tab"
            id={`settings-tab-${id}`}
            aria-controls={`settings-panel-${id}`}
            aria-selected={tab === id}
            tabIndex={tab === id ? 0 : -1}
            onClick={() => setTab(id)}
            onKeyDown={(event) => {
              if (
                !["ArrowRight", "ArrowLeft", "Home", "End"].includes(event.key)
              )
                return;
              event.preventDefault();
              const next =
                event.key === "Home"
                  ? 0
                  : event.key === "End"
                    ? tabs.length - 1
                    : (index +
                        (event.key === "ArrowRight" ? 1 : -1) +
                        tabs.length) %
                      tabs.length;
              setTab(tabs[next].id);
              tabRefs.current[next]?.focus();
            }}
          >
            {label}
          </button>
        ))}
      </div>
      <div
        role="tabpanel"
        id={`settings-panel-${tab}`}
        aria-labelledby={`settings-tab-${tab}`}
        tabIndex={0}
      >
        {tab === "general" && (
          <div className="settings-section">
            <h2 id="preferences-title">Preferências</h2>
            <p className="media-folder">
              Pasta padrão: {settings.outputFolder || "nenhuma selecionada"}
            </p>
            <p>Em caso de colisão, os arquivos recebem um sufixo numérico.</p>
            <div className="mt-3 flex flex-wrap gap-2">
              <Button
                type="button"
                className="h-8 px-3 text-xs"
                disabled={busy}
                onClick={chooseFolder}
              >
                Escolher pasta padrão
              </Button>
              <Button
                type="button"
                className="h-8 px-3 text-xs"
                disabled={busy}
                onClick={() => saveSettings(defaultSettings)}
              >
                Restaurar preferências
              </Button>
            </div>
            {error && (
              <p className="media-error" role="alert">
                {error}
              </p>
            )}
          </div>
        )}
        {tab === "backup" && <SettingsBackup />}
        {tab === "updates" && <UpdateSettings />}
        {tab === "diagnostic" && diagnostic}
      </div>
    </section>
  );
}
