# Progresso do MediaToolkit

Atualizado em: 2026-08-04

## Estado atual

- Status geral: Fase 01 em andamento.
- Próxima fase: [`Fase 01 — Fundação Tauri`](fases/fase-01-fundacao.md).
- Próximo bloco: checkpoint 01.1 — confirmar entendimento antes do primeiro comando Rust próprio.
- Código do aplicativo: template Tauri React + TypeScript criado em `app/`.

## Decisões confirmadas

- [x] Windows 10/11 x64 é o primeiro alvo.
- [x] Uso inicial pessoal e gratuito para amigos.
- [x] Tauri 2 + React + TypeScript + Rust permanece como stack.
- [x] Tailwind CSS + shadcn/ui serão usados na interface.
- [x] `image-rs` será testado primeiro atrás de `ImageProcessor`.
- [x] A troca para libvips depende de critérios objetivos, não de preferência.
- [x] O ensino será gradual, com pausa obrigatória entre blocos importantes.
- [x] O usuário tem base frontend júnior/pleno e está começando Rust/terminal.
- [x] Nome do aplicativo: LpsToolkit; identificador: `com.lpstoolkit.app`.

## Fases

- [x] Fase 00 — Ambiente e linha de base
- [ ] Fase 01 — Fundação Tauri
- [ ] Fase 02 — Shell visual
- [ ] Fase 03 — Contratos e arquivos
- [ ] Fase 04 — Imagens
- [ ] Fase 05 — Fila e processos
- [ ] Fase 06 — Vídeo e áudio
- [ ] Fase 07 — Downloader
- [ ] Fase 08 — Persistência e acabamento
- [ ] Fase 09 — Distribuição Windows

## Diário de execução

| Data | Bloco | Evidência | Resultado |
|---|---|---|---|
| 2026-08-04 | Planejamento | Documentos em `docs/` e regras em `AGENTS.md` | Concluído |
| 2026-08-04 | 00.1 — Inventário | `docs/ambiente-local.md` | Concluído: Git, Node/npm e WebView2 presentes; Rust ausente |
| 2026-08-04 | 00.2a — C++ Build Tools | `vswhere` detectou MSVC 14.44.35207 | Concluído |
| 2026-08-04 | 00.2b — Rust MSVC | `rustc 1.97.1` compilou um executável temporário | Concluído |
| 2026-08-04 | 00.3 — Git e diagnóstico | `git status --ignored` e `scripts/verificar-ambiente.ps1` | Concluído: repositório `main`, PDF pessoal ignorado e diagnóstico validado |
| 2026-08-04 | Fase 00 | Checkpoints de ambiente, Rust, terminal e Git respondidos | Concluída com autorização do usuário |
| 2026-08-04 | 01.1 — Scaffold | `npm run build` e inicialização de `lps-toolkit.exe` | Concluído: template React/TypeScript/Tauri, npm e lockfile criados |

## Decisões pendentes no momento adequado

- Gerenciador de pacotes: npm é o padrão proposto; confirmar na Fase 01.
- Base do shadcn/ui e identidade visual: confirmar na Fase 02.
- Formatos e presets exatos de cada módulo: confirmar no início das Fases 04, 06 e 07.
- Política padrão para nomes e arquivos existentes: confirmar na Fase 03.
- Origem/build dos sidecars e forma de empacotamento: decidir nas Fases 06, 07 e 09.
- NSIS ou MSI: decidir na Fase 09 após protótipo de instalação.

## Como atualizar este arquivo

- Marcar um bloco somente após testes e validação manual correspondentes.
- Acrescentar uma linha ao diário com a evidência principal.
- Manter exatamente uma “próxima fase” e um “próximo bloco”.
- Se uma decisão mudar, registrar o motivo; não apagar silenciosamente a decisão anterior.
