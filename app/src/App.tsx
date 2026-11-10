import { useState, useSyncExternalStore } from "react";
import { AppShell, EmptyState, PageHeader } from "@/components/AppShell";
import { ImageSelection } from "@/components/ImageSelection";
import { DownloaderPanel } from "@/components/DownloaderPanel";
import { MediaDetailsPanel } from "@/components/MediaDetailsPanel";
import { PreferencesPanel } from "@/components/PreferencesPanel";
import { Button } from "@/components/ui/button";
import { getRoute, pages, subscribeToRoute } from "@/navigation";
import {
  diagnoseMediaTools,
  getAppDiagnostic,
  type AppDiagnostic,
  type ToolReport,
} from "./diagnostic";

function Diagnostic() {
  const [diagnostic, setDiagnostic] = useState<AppDiagnostic | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [tools, setTools] = useState<ToolReport[]>([]);
  const [checkingTools, setCheckingTools] = useState(false);
  const [toolsError, setToolsError] = useState("");

  async function checkDiagnostic() {
    setLoading(true);
    setDiagnostic(null);
    setError("");

    try {
      setDiagnostic(await getAppDiagnostic());
    } catch {
      setError(
        "Não foi possível consultar a versão. Abra o Pacton pelo aplicativo desktop e tente novamente.",
      );
    } finally {
      setLoading(false);
    }
  }

  async function checkTools() {
    setCheckingTools(true);
    setToolsError("");
    setTools([]);
    try {
      setTools(await diagnoseMediaTools());
    } catch {
      setToolsError("Não foi possível verificar as ferramentas de mídia.");
    } finally {
      setCheckingTools(false);
    }
  }

  return (
    <section className="surface diagnostic" aria-labelledby="diagnostic-title">
      <h2 id="diagnostic-title">Diagnóstico</h2>
      <p>Confira a versão instalada do Pacton.</p>
      <Button
        type="button"
        className="h-8 px-3 text-xs"
        onClick={checkDiagnostic}
        disabled={loading}
      >
        Consultar diagnóstico
      </Button>
      <div role="status">
        {loading && <p>Consultando diagnóstico…</p>}
        {diagnostic && <p>Versão do aplicativo: {diagnostic.version}</p>}
      </div>
      {error && <p role="alert">{error}</p>}
      <Button
        type="button"
        className="h-8 px-3 text-xs"
        disabled={checkingTools}
        onClick={checkTools}
      >
        Verificar ferramentas de mídia
      </Button>
      <div aria-live="polite">
        {checkingTools && <p>Verificando ferramentas de mídia…</p>}
        {tools.map((tool) => (
          <p key={tool.name}>
            {tool.name} ·{" "}
            {tool.version ??
              (tool.status === "missing"
                ? "Ausente no PATH"
                : "Não respondeu corretamente")}
          </p>
        ))}
      </div>
      {toolsError && <p role="alert">{toolsError}</p>}
    </section>
  );
}

function App() {
  const route = useSyncExternalStore(subscribeToRoute, getRoute);
  const page = pages.find(({ path }) => path === route);

  return (
    <AppShell page={page}>
      <PageHeader
        title={page?.label ?? "Página não encontrada"}
        description={
          page?.description ??
          "Esse endereço não corresponde a uma tela do Pacton."
        }
      />
      {!page ? (
        <a className="return-home" href="#/">
          Voltar ao início
        </a>
      ) : page.path === "#/" ? (
        <section className="tool-grid" aria-label="Ferramentas de mídia">
          {pages
            .slice(1, -1)
            .map(({ path, label, icon: Icon, description }) => (
              <a className="surface tool-card" href={path} key={path}>
                <span className="tile-icon">
                  <Icon aria-hidden="true" />
                </span>
                <div>
                  <h2>{label}</h2>
                  <p>{description}</p>
                </div>
              </a>
            ))}
        </section>
      ) : page.path === "#/configuracoes" ? (
        <PreferencesPanel diagnostic={<Diagnostic />} />
      ) : page.path === "#/imagens" ? (
        <ImageSelection />
      ) : page.path === "#/video" || page.path === "#/audio" ? (
        <MediaDetailsPanel
          key={page.path}
          kind={page.path === "#/video" ? "video" : "audio"}
        />
      ) : page.path === "#/downloader" ? (
        <DownloaderPanel />
      ) : (
        <EmptyState page={page} />
      )}
    </AppShell>
  );
}

export default App;
