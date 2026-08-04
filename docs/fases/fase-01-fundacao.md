# Fase 01 — Fundação Tauri

## Resultado

O diretório `app/` conterá uma aplicação Tauri 2 com React e TypeScript. A janela abrirá, o frontend chamará um pequeno comando Rust e os primeiros gates de qualidade estarão automatizados.

## Conceitos novos

- Processo de desenvolvimento Vite versus aplicação desktop Tauri.
- `package.json`/npm e `Cargo.toml`/Cargo.
- WebView e backend nativo.
- Comando Tauri e serialização entre TypeScript e Rust.
- Lockfile e build reproduzível.

## Bloco 01.1 — Scaffold controlado

- Confirmar nome exibido, nome de diretório e identificador reverso do aplicativo.
- Confirmar npm como gerenciador padrão proposto.
- Criar `app/` com `create-tauri-app`, React e TypeScript.
- Ler a árvore criada antes de editar.
- Instalar dependências e preservar os lockfiles.

Validação: frontend inicia; depois `npm run tauri dev` abre a janela padrão.

Checkpoint: identificar quais arquivos pertencem ao frontend e quais pertencem ao backend Rust.

## Bloco 01.2 — Primeiro comando Rust

- Ler `lib.rs`/`main.rs` existentes.
- Criar um comando mínimo de diagnóstico, por exemplo versão/saudação estruturada.
- Criar um wrapper TypeScript único para `invoke`.
- Mostrar o resultado na tela e tratar falha.
- Testar a função Rust independente quando possível e o wrapper frontend com mock.

Checkpoint: explicar o caminho React -> `invoke` -> comando Rust -> resposta JSON.

## Bloco 01.3 — Qualidade frontend e Rust

- Configurar formatação e lint sem reformatar arquivos desnecessariamente.
- Adicionar Vitest e suporte a testes React.
- Criar scripts `format:check`, `lint`, `typecheck`, `test` e `build`.
- Configurar `cargo fmt`, `cargo clippy` e `cargo test` como gate Rust.
- Criar um teste intencionalmente falho, entender a mensagem e corrigi-lo.

Checkpoint: dizer qual diferença existe entre typecheck, lint e teste.

## Bloco 01.4 — Tailwind e shadcn/ui

- Integrar Tailwind CSS usando o plugin oficial do Vite.
- Configurar o alias `@/`.
- Inicializar shadcn/ui após confirmar base e estilo inicial.
- Adicionar somente os componentes necessários para provar a instalação.
- Criar uma tela mínima “MediaToolkit” sem antecipar o design completo.

Validação: estilos aparecem no browser e no WebView Tauri; build frontend e Tauri passam.

Checkpoint: explicar por que os componentes do shadcn passam a fazer parte do nosso código.

## Critérios de aceite

- [ ] `app/` foi criado com React + TypeScript + Tauri 2.
- [ ] Janela Tauri abre sem erro.
- [ ] Primeiro comando Rust retorna dado estruturado.
- [ ] Erro do comando pode ser apresentado sem quebrar a tela.
- [ ] Tailwind e shadcn/ui funcionam.
- [ ] Gates frontend e Rust passam.
- [ ] Lockfiles estão preservados.
- [ ] O usuário aprovou avançar.

