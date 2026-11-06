# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Verificações de qualidade

Execute os comandos a partir de `app/`. Os checks do frontend foram validados com Node.js 24.

| Comando | O que verifica |
|---|---|
| `npm run format:check` | Formatação de TypeScript e da configuração do ESLint, sem escrever nos arquivos |
| `npm run lint` | Regras de análise estática do frontend |
| `npm run typecheck` | Tipos do código em `src/` |
| `npm test` | Comportamento do frontend com Vitest |
| `npm run build` | Tipos e geração do bundle de produção |
| `npm run rust:check` | Formatação Rust, Clippy e testes com o lockfile existente |

No Windows, rode a verificação Rust em um terminal com MSVC e Windows SDK x64 carregados. Se o cache do Cargo veio de outro caminho e o Tauri apontar para o endereço antigo, escolha um diretório vazio em `CARGO_TARGET_DIR` dentro de `src-tauri/target/` e reaproveite-o nas próximas execuções.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
