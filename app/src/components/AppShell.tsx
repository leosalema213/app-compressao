import { useEffect, useState, type ReactNode } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { ChevronRight, HardDrive } from "lucide-react";
import { getAppDiagnostic } from "@/diagnostic";
import { version } from "../../package.json";
import { pages, type Page } from "@/navigation";
import { AppearanceControls } from "./AppearanceControls";
import { DownloadCenter } from "./DownloadCenter";
import { AppUpdatePrompt } from "./AppUpdates";
import { dragWindow, WindowControls } from "./WindowControls";

export function AppShell({
  page,
  children,
}: {
  page: Page | undefined;
  children: ReactNode;
}) {
  const [appVersion, setAppVersion] = useState(version);
  useEffect(() => {
    if (!isTauri()) return;
    let active = true;
    void getAppDiagnostic()
      .then(({ version }) => {
        if (active) setAppVersion(version);
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, []);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <a className="brand" href="#/" aria-label="Pacton — Início">
          <svg
            className="brand-wordmark"
            viewBox="0 0 230 60"
            role="img"
            aria-label="Pacton"
          >
            <image href="/branding/pacton-symbol.svg" width="60" height="60" />
            <text
              x="72"
              y="43"
              fill="currentColor"
              fontSize="42"
              fontWeight="650"
              letterSpacing="-2"
              textLength="152"
              lengthAdjust="spacingAndGlyphs"
            >
              pacton
            </text>
          </svg>
          <small className="app-version">v{appVersion}</small>
        </a>
        <nav aria-label="Navegação principal">
          <p className="sidebar-label">Área de trabalho</p>
          <ul>
            {pages.map(({ path, label, icon: Icon }) => (
              <li key={path}>
                <a
                  href={path}
                  aria-current={page?.path === path ? "page" : undefined}
                >
                  <Icon aria-hidden="true" />
                  {label}
                </a>
              </li>
            ))}
          </ul>
        </nav>
        <p className="local-note">
          <HardDrive aria-hidden="true" /> Aplicativo local
        </p>
      </aside>
      <div className="workspace">
        <header className="topbar" onMouseDown={dragWindow}>
          <div className="breadcrumb" aria-label="Localização">
            <span>Meu espaço</span>
            <ChevronRight size={13} aria-hidden="true" />
            <strong>{page?.label ?? "Página desconhecida"}</strong>
          </div>
          <div className="topbar-controls">
            <DownloadCenter />
            <AppearanceControls />
            <WindowControls />
          </div>
        </header>
        <main className="workspace-content">{children}</main>
        <AppUpdatePrompt />
      </div>
    </div>
  );
}

export function PageHeader({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <header className="page-heading">
      <h1>{title}</h1>
      <p>{description}</p>
    </header>
  );
}

export function EmptyState({ page }: { page: Page }) {
  const Icon = page.icon;
  return (
    <section className="empty-state surface" aria-label={page.label}>
      <span className="tile-icon">
        <Icon aria-hidden="true" />
      </span>
      <h2>
        {page.label === "Downloader"
          ? "Nenhum download iniciado"
          : "Nenhum arquivo selecionado"}
      </h2>
      <p>
        {page.label === "Downloader"
          ? "Os downloads aparecerão aqui quando o módulo estiver disponível."
          : "Os arquivos adicionados a este módulo aparecerão aqui."}
      </p>
    </section>
  );
}
