# Inventário local de desenvolvimento

Atualizado em: 2026-08-04

Este arquivo registra apenas a disponibilidade técnica das ferramentas. Não contém nome de usuário, e-mail, credenciais ou caminhos pessoais.

## Sistema

| Item | Resultado |
|---|---|
| Windows | Windows 10 Pro, versão 24H2, build 26100.8737 |
| Arquitetura | x64 |
| PowerShell | 5.1.26100.8737 |
| WebView2 Runtime | 150.0.4078.105 |

## Ferramentas de desenvolvimento

| Ferramenta | Situação | Observação |
|---|---|---|
| Git | Disponível: 2.50.1.windows.1 | Pronto para a Fase 00.3 |
| Node.js | Disponível: v24.12.0 | Pronto para o frontend |
| npm | Disponível: 11.6.2 por `npm.cmd` | `npm.ps1` é bloqueado pela política de execução atual; usar `npm.cmd` neste PowerShell evita alterar a política |
| rustup | Disponível | Toolchain padrão: `stable-x86_64-pc-windows-msvc` |
| rustc | Disponível: 1.97.1 | Compilação de verificação concluída |
| Cargo | Disponível: 1.97.1 | Instalado com Rust/rustup |
| Microsoft C++ Build Tools | Disponível | Build Tools 2022 com MSVC 14.44.35207; confirmado por `vswhere` |

## Ferramentas de mídia

| Ferramenta | Situação | Quando será necessária |
|---|---|---|
| FFmpeg | Ausente do PATH | Fase 06 |
| FFprobe | Ausente do PATH | Fase 06 |
| yt-dlp | Ausente do PATH | Fase 07 |

## Conclusão do bloco 00.1

- O computador atende ao alvo Windows x64 e já possui WebView2, Git e Node/npm.
- Os pré-requisitos de compilação do Tauri estão presentes: Node/npm, WebView2, C++ Build Tools e Rust stable MSVC.
- Um novo terminal deve ser aberto antes de usar `rustup`, `rustc` e `cargo` pelo nome, pois o PATH do terminal anterior não é atualizado automaticamente.
- FFmpeg, FFprobe e yt-dlp não serão instalados agora, pois não serão usados antes das Fases 06 e 07.

## Próxima ação proposta

Bloco 00.3: inicializar Git, criar `.gitignore` e um diagnóstico somente leitura reproduzível, mediante confirmação.
