import type { ReactNode } from "react";
import { HardDrive, Layers3 } from "lucide-react";
import { pages, type Page } from "@/navigation";
import { AppearanceControls } from "./AppearanceControls";
import { DownloadCenter } from "./DownloadCenter";
import { AppUpdatePrompt } from "./AppUpdates";

export function AppShell({
  page,
  children,
}: {
  page: Page | undefined;
  children: ReactNode;
}) {
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <a className="brand" href="#/" aria-label="LpsToolkit — Início">
          <span className="brand-mark">
            <Layers3 aria-hidden="true" />
          </span>
          <span>
            LpsToolkit <small>FERRAMENTAS DE MÍDIA</small>
          </span>
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
        <header className="topbar">
          <span>
            Ferramentas <span aria-hidden="true">/</span>{" "}
            <strong>{page?.label ?? "Página desconhecida"}</strong>
          </span>
          <div className="topbar-controls">
            <DownloadCenter />
            <AppearanceControls />
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
