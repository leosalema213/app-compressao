# Progresso do MediaToolkit

Atualizado em: 2026-11-05

## Estado atual

- Status geral: Fases 05–07 com fluxos funcionais em validação. Preferências locais da Fase 08 iniciadas.
- Próxima fase: [`Fase 08 — Persistência e acabamento`](fases/fase-08-acabamento.md), após registrar a validação restante do Downloader.
- Próximo bloco: 08.2 — Histórico. Verificar cancelamento de playlist e mídia externa autorizada antes de encerrar a Fase 07.
- Código do aplicativo: imagens em lote, fila e cancelamento, vídeo MP4, áudio MP3, Downloader e preferências locais.

## Decisões confirmadas

- [x] Windows 10/11 x64 é o primeiro alvo.
- [x] Uso inicial pessoal e gratuito para amigos.
- [x] Tauri 2 + React + TypeScript + Rust permanece como stack.
- [x] Tailwind CSS + shadcn/ui serão usados na interface.
- [x] `image-rs` será testado primeiro atrás de `ImageProcessor`.
- [x] A troca para libvips depende de critérios objetivos, não de preferência.
- [x] A execução segue em blocos pequenos e documentados. O usuário dispensou as pausas para checkpoints didáticos e delegou a continuidade da implementação.
- [x] O usuário tem base frontend júnior/pleno e está começando Rust/terminal.
- [x] Nome do aplicativo: LpsToolkit; identificador: `com.lpstoolkit.app`.
- [x] Organon é a referência visual e de experiência para instalação, atualização e fluxo de previews/releases. A adaptação segue o [roadmap](02-roadmap.md) e a stack Tauri.
- [x] Manter no máximo dois builds de instalador local, conforme `AGENTS.md`, e prototipar o seletor de cores no cabeçalho na Fase 02.
- [x] Se o nome de saída estiver ocupado, renomear automaticamente com sufixo numérico e preservar o arquivo existente.

## Fases

- [x] Fase 00 — Ambiente e linha de base
- [x] Fase 01 — Fundação Tauri
- [x] Fase 02 — Shell visual
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
| 2026-11-05 | Planejamento da retomada | Inspeção dos estilos, regras, instalador e atualizador do Organon e conferência com os arquivos de fase | Tarefas e referência registradas no roadmap. Checkpoint 01.1 pendente |
| 2026-11-05 | 01.2 — Primeiro comando Rust | Checkpoint 01.1 confirmado. TypeScript e mock do IPC passaram para sucesso e rejeição | Implementado. Build Vite e Cargo encontraram restrição de acesso em `Z:`. Teste Rust e validação da janela pendentes |
| 2026-11-05 | 01.2 — Verificação no Windows | Build do Vite confirmado no terminal do usuário. `cargo fmt --check` passou. O cache anterior contém caminhos de `Z:\meu\app`. Com cache separado e MSVC carregado, o linker parou em `LNK1181` por ausência de `kernel32.lib` no Windows SDK registrado | Teste Rust, Clippy e validação da janela pendentes. Retomar após disponibilizar as ferramentas e bibliotecas x64 do Windows SDK |
| 2026-11-05 | 01.2 — Validação após instalação do SDK | Windows SDK 26100 x64, teste de serialização Rust (1 aprovado), Clippy e build desktop. No WebView real, o botão retornou a versão 0.1.0 | Validação técnica concluída. Checkpoint antes do 01.3 |
| 2026-11-05 | Ambiente — Retenção do SDK | Componente 22621 removido pelo instalador. Verificação confirmou apenas o 26100 ativo e o teste Rust passou novamente | 11,3 GiB livres em `C:` e 10,3 GiB em `Z:`. Cache isolado de validação em `app/src-tauri/target/validacao-local` |
| 2026-11-05 | 01.3 — Qualidade frontend e Rust | Prettier, ESLint, typecheck, build Vite e `npm run rust:check` passaram. O Vitest detectou a versão esperada errada de propósito. Após a correção, os dois testes React passaram | Concluído. Os comandos estão documentados em `app/README.md`. Checkpoint antes do 01.4 |
| 2026-11-05 | 01.4 — Tailwind e shadcn/ui | Formatação, lint, checagem de tipos, 2 testes React e builds Vite e Rust passaram. Chrome e WebView confirmaram o estilo do botão em claro e escuro. O clique retornou a versão no desktop e exibiu a mensagem esperada no navegador | Integração validada com base New York e paleta neutra provisória. Aprovação para a Fase 02 pendente |
| 2026-11-05 | 02.1 — Mapa e esboço | Mapa de seis telas, rotas por hash e layout estreito registrados na fase. Usuário autorizou iniciar a Fase 02 | Concluído |
| 2026-11-05 | 02.2 — Tokens e componentes-base | Segoe UI, barra lateral de 200 px e foco por teclado verificados no WebView. Layout de 360 px sem rolagem horizontal | Concluído |
| 2026-11-05 | 02.3 — Rotas e páginas | Vitest verificou as seis rotas, página desconhecida e histórico. No WebView, Configurações retornou a versão do Rust | Concluído |
| 2026-11-05 | 02.4 — Tema e cores | Vitest verificou a recuperação de tema e cor ao remontar. No WebView, o contraste da navegação ativa passou em oito combinações. Formatação, lint, tipos, 6 testes React, gate Rust e builds passaram | Concluído. Preferências provisórias no armazenamento local |
| 2026-11-05 | 03.1 — Contrato inicial | Rust e React usaram o mesmo exemplo JSON para um lote com imagem aceita e item rejeitado | Concluído para Imagens. Os erros têm código estável separado do texto |
| 2026-11-05 | 03.2 — Seleção e pasta | Mocks cobriram cancelamento e escolha de pasta. No WebView, o comando de abertura do diálogo foi aceito e o Rust validou uma pasta real por IPC | Integrado. A escolha física de itens na janela nativa não fez parte do teste automatizado |
| 2026-11-05 | 03.3 — Inspeção | Testes Rust cobriram extensão enganosa, arquivo vazio, caminhos inválidos, pasta e Unicode. O WebView usou arquivos descartáveis e a saída ficou vazia | Inspeção preliminar de JPEG, PNG e WebP validada |
| 2026-11-05 | 03.4 — Arraste e lista | Evento de arraste simulado no WebView percorreu a inspeção Rust real. Testes React cobriram remoção, limpeza e duplicatas. 10 testes React, 6 Rust, lint, formatação, tipos e builds passaram | Integrado. Arraste físico no Explorador não fez parte do teste |
| 2026-11-05 | 03.5 — Saída e colisão | 11 testes React, 10 Rust, análise estática e builds passaram. O IPC real previu `Foto com acento é-3.PNG` quando o original e a saída `-2` existiam, sem alterar nenhum dos arquivos | Política e previsão implementadas. Verificar publicação sem sobrescrita na Fase 04 |
| 2026-11-05 | Fase 04 — Primeira compressão | 12 testes React, 13 Rust, formatação, lint, tipos, Clippy e builds passaram. O IPC real reduziu o PNG de teste de 16.516 para 154 bytes. Duas chamadas concorrentes criaram `-3` e `-4`, preservando os arquivos existentes e reportando a falha da imagem corrompida | Compressão no formato de origem validada. Redimensionamento, conversão e comparativo de motores pendentes |
| 2026-11-05 | 04.3 — Redimensionamento | 13 testes React, 16 Rust, análise estática e builds passaram. O IPC real gravou PNG de 64×32 em 20×10 e 24×12, preservou 64×32 com limite maior e rejeitou zero. Testes Rust verificaram EXIF, transparência e original intacto | Concluído. Limites opcionais, sem ampliação |
| 2026-11-05 | 04.4 — Conversão e compressão | 14 testes React, 18 Rust, análise estática e builds passaram. O Rust validou as nove combinações por conteúdo. O IPC real confirmou prévia e saída JPEG com sufixo, qualidade, redimensionamento combinado, WebP e originais intactos | Concluído. Qualidade JPEG ajustável e WebP de 8 bits codificado sem perdas |
| 2026-11-05 | 04.5 — Interface e lote | 14 testes React, 19 Rust, análise estática e builds passaram. O frontend manteve o destino associado ao resultado após mudar a seleção. O Tauri real recusou outra pasta e abriu no Explorador o destino do lote | Concluído. Resumo de sucessos, falhas e diferença total de tamanho |
| 2026-11-05 | 04.6 — Comparativo | Corpus sintético de sete entradas, image-rs em release, Sharp/libvips, comparação visual e pico amostrado de 127,6 MiB | Manter image-rs. A referência gerou arquivos menores neste corpus; o WebP do Rust cresceu em um caso |
| 2026-11-05 | 05 — Fila e processos | IPC real testou dois trabalhos, cancelamento na fila e durante um lote, progresso monotônico e preservação das saídas já gravadas. Teste do executor verificou stdout, stderr e timeout | Fila FIFO compartilhada, canal e cancelamento integrados. Lista global e repetição pendentes |
| 2026-11-05 | 06 — Vídeo e áudio | FFmpeg/FFprobe 9.0.1 no Tauri real. Metadados JSON, MP4 H.264/AAC com e sem faixa de áudio, WAV para MP3 e extração de MP3. Entradas com Unicode, rejeição de extensão enganosa e destino ocupado | Fluxo inicial integrado. Testar robustez e cancelamento do FFmpeg com trabalhos longos |
| 2026-11-05 | 07 — Downloader | yt-dlp 2026.10.26 oficial verificado por SHA-256 em pasta de teste. IPC real com HTTP local cobriu consulta sem saída, consentimento, MP4/MP3, miniatura PNG, legenda WebVTT, colisão e playlist de dois itens com confirmação | Qualidade por formatos detectados e playlist de até 20 itens implementadas. Teste externo autorizado e cancelamento de playlist pendentes |
| 2026-11-05 | 08.1 — Preferências | Schema v1 no armazenamento local do WebView, migração de tema e cor antigos, pasta padrão validada no Rust e restauração de padrões. 15 testes React, 26 Rust, análise estática e builds | Implementado. Histórico e notificações pendentes |
| 2026-11-06 | 09 — Preview Windows | Build x64 NSIS em português, instalação por usuário, executável abriu a janela LpsToolkit com perfil temporário. 15 testes React, lint e formatação passaram | Instalador `0.1.0-preview.1`. Ferramentas externas no PATH e atualização manual durante o preview. Instalação em Windows limpo pendente |
| 2026-11-06 | Feedback do preview — Tema e imagens | 16 testes React, 27 Rust, lint, Clippy e build passaram. No WebView2, troca de tema e cor, arraste simulado, miniatura, prévia JPEG a 50% e ampliação funcionaram. O teste Rust comparou bytes e dimensões da prévia com os da saída gravada | Instalador local `0.1.0-preview.2` com SHA-256. Retidos os previews 1 e 2 |
| 2026-11-06 | Preview local sem instalação | Script gerou o executável, conferiu a cópia e atualizou o atalho. WebView2 abriu pelo executável indicado, retornou `0.1.0-preview.3` e criou o perfil separado | Retidos os previews locais 2 e 3. Fluxo de instalação reservado para publicação |
| 2026-11-06 | Feedback do preview — Campos, escala, vídeo e Downloader | 16 testes React, 28 Rust, lint, Clippy e build passaram. No executável, alvo de 50% gerou 927.862 bytes a partir de 2.013.898 (46,1%). A consulta pública do YouTube retornou título e duração com o yt-dlp oficial e Node existente | Preview local `0.1.0-preview.4`. Atalho atualizado e retidos os previews 3 e 4 |
| 2026-11-06 | Feedback do preview — Downloader | 16 testes React, 29 Rust, lint, Clippy e build passaram. O executável mostrou capa e 23 itens, campo de link de 390 px na janela padrão e duas colunas. O lote sintético gravou dois MP3 numerados e preservou um arquivo existente | Preview local `0.1.0-preview.5`. Atalho atualizado e retidos os previews 4 e 5 |
| 2026-11-06 | Feedback do preview — Central de downloads | 16 testes React, 29 Rust, lint, Clippy e build passaram. O executável pausou uma transferência local, retomou com requisição HTTP parcial e gerou MP4 válido. Consulta pública do YouTube levou 5,8 s na primeira vez e 4 ms em cache | Preview local `0.1.0-preview.6`. Atalho atualizado e retidos os previews 5 e 6 |
| 2026-11-09 | Entrega 0.1.0 | Usuário validou o lote e autorizou subir. 16 testes React e formatação Rust passaram. Build NSIS x64 em português e abertura da janela final verificados. SHA-256 `84219319053a8c01ff94a84ba58792d439d9be9102b2c6ebba9a87990e76c9fc` | Versão final preparada para [GitHub](https://github.com/leosalema213/app-compressao/releases/tag/v0.1.0). Atualização manual e ferramentas externas instaladas separadamente |

## Decisões pendentes no momento adequado

- Gerenciador de pacotes: npm é o padrão proposto; confirmar na Fase 01.
- Formatos e presets exatos de cada módulo: confirmar no início das Fases 04, 06 e 07.
- Origem/build dos sidecars e forma de empacotamento: decidir nas Fases 06, 07 e 09.
- NSIS ou MSI: decidir na Fase 09 após protótipo de instalação.
- Mecanismo, assinatura e canal de atualização: definir em fase própria conforme a regra da Fase 09.

## Como atualizar este arquivo

- Marcar um bloco somente após testes e validação manual correspondentes.
- Acrescentar uma linha ao diário com a evidência principal.
- Manter exatamente uma “próxima fase” e um “próximo bloco”.
- Se uma decisão mudar, registrar o motivo; não apagar silenciosamente a decisão anterior.
