import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Download, Upload } from "lucide-react";
import { Button } from "./ui/button";
import { mediaErrorMessage } from "@/lib/media-contract";
import { validateOutputFolder } from "@/lib/media-selection";
import { readSettings, saveSettings, type Settings } from "@/lib/settings";

export function SettingsBackup() {
  const [busy, setBusy] = useState<"export" | "import" | "restore" | null>(
    null,
  );
  const [pending, setPending] = useState<Settings | null>(null);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const filters = [{ name: "Backup do Pacton", extensions: ["json"] }];

  async function run(action: () => Promise<void>, mode: typeof busy) {
    setBusy(mode);
    setError("");
    setMessage("");
    try {
      await action();
    } catch (failure) {
      setError(mediaErrorMessage(failure));
    } finally {
      setBusy(null);
    }
  }

  function exportBackup() {
    return run(async () => {
      const path = await save({
        title: "Exportar backup",
        defaultPath: "Pacton-backup.json",
        filters,
      });
      if (!path) return;
      await invoke("export_settings_backup", {
        path,
        settings: readSettings(),
      });
      setMessage("Backup exportado com sucesso.");
    }, "export");
  }

  function importBackup() {
    return run(async () => {
      const path = await open({
        title: "Importar backup",
        multiple: false,
        filters,
      });
      if (!path) return;
      setPending(await invoke<Settings>("import_settings_backup", { path }));
    }, "import");
  }

  function restore() {
    if (!pending) return;
    const settings = pending;
    return run(async () => {
      let outputFolder = settings.outputFolder;
      let unavailable = false;
      if (outputFolder) {
        try {
          outputFolder = await validateOutputFolder(outputFolder);
        } catch {
          outputFolder = "";
          unavailable = true;
        }
      }
      saveSettings({
        theme: settings.theme,
        palette: settings.palette,
        outputFolder,
      });
      setPending(null);
      setMessage(
        unavailable
          ? "Preferências restauradas. A pasta salva não está disponível neste computador. Escolha uma pasta padrão na aba Geral."
          : "Preferências restauradas com sucesso.",
      );
    }, "restore");
  }

  return (
    <div className="settings-section">
      <h2>Seus dados pessoais</h2>
      <p>
        Exporte tema, cor principal e pasta padrão para restaurar suas
        preferências ou levá-las a outro computador.
      </p>
      <p>Os arquivos de mídia continuam nas pastas em que você os salvou.</p>
      <div className="settings-actions">
        <Button
          type="button"
          className="h-8 px-3 text-xs"
          disabled={Boolean(busy)}
          onClick={() => void exportBackup()}
        >
          <Download size={14} aria-hidden="true" />{" "}
          {busy === "export" ? "Exportando…" : "Exportar backup"}
        </Button>
        <button
          type="button"
          className="theme-trigger"
          disabled={Boolean(busy)}
          onClick={() => void importBackup()}
        >
          <Upload size={14} aria-hidden="true" />{" "}
          {busy === "import" ? "Importando…" : "Importar backup"}
        </button>
      </div>
      {pending && (
        <div className="backup-confirmation">
          <h3>Restaurar este backup?</h3>
          <p>
            As preferências atuais serão substituídas pelas do arquivo
            selecionado.
          </p>
          <div className="settings-actions">
            <Button
              type="button"
              className="h-8 px-3 text-xs"
              disabled={Boolean(busy)}
              onClick={() => void restore()}
            >
              {busy === "restore" ? "Restaurando…" : "Restaurar preferências"}
            </Button>
            <button
              type="button"
              className="media-text-button"
              disabled={Boolean(busy)}
              onClick={() => setPending(null)}
            >
              Cancelar
            </button>
          </div>
        </div>
      )}
      {message && <p role="status">{message}</p>}
      {error && (
        <p role="alert" className="media-error">
          {error}
        </p>
      )}
    </div>
  );
}
