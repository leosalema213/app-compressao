import { useEffect, useState } from "react";
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

export function PreferencesPanel() {
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
    <section className="surface" aria-labelledby="preferences-title">
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
    </section>
  );
}
