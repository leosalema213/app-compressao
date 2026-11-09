# Roadmap detalhado

## Por que esta ordem

O roadmap original foi preservado em conteúdo, mas reorganizado por dependência. A fila e o controle de processos entram antes dos módulos pesados, pois vídeo, áudio e downloads precisam do mesmo ciclo de progresso, cancelamento e falha. O Downloader fica depois de FFmpeg porque utiliza FFmpeg/FFprobe para mesclar e pós-processar vários downloads.

```text
Ambiente
  -> base Tauri
    -> shell visual
      -> contratos + arquivos
        -> imagens (primeira fatia vertical)
          -> fila/processos
            -> vídeo/áudio
              -> downloader
                -> acabamento
                  -> distribuição
```

## Fase 00 — Ambiente e linha de base

Resultado: ferramentas necessárias identificadas, Git inicializado e diagnóstico reproduzível.

- Inventariar Node LTS, npm, Rust stable MSVC, Cargo, Git, WebView2 e Microsoft C++ Build Tools.
- Explicar PATH, terminal, comando, argumento, diretório atual e código de saída.
- Registrar versões realmente instaladas.
- Instalar apenas o que estiver faltando, com confirmação antes de cada instalação.
- Inicializar Git e criar `.gitignore` quando aprovado.
- Criar um script somente de diagnóstico, sem alterar a máquina.
- Validar os pré-requisitos oficiais do Tauri no Windows.

Arquivo: [`fases/fase-00-ambiente.md`](fases/fase-00-ambiente.md)

## Fase 01 — Fundação Tauri

Resultado: janela desktop mínima, React/TypeScript e Rust compilando com testes básicos.

- Criar o projeto com o template oficial React + TypeScript.
- Conhecer `package.json`, `vite.config.ts`, `src/`, `Cargo.toml` e `src-tauri/`.
- Rodar primeiro o frontend e depois a janela Tauri.
- Criar o primeiro comando Rust didático e chamá-lo do React.
- Configurar scripts de format, lint, typecheck, test e build.
- Instalar Tailwind pelo plugin oficial do Vite e inicializar shadcn/ui.
- Fixar dependências pelos lockfiles; não escolher versões por memória.

Arquivo: [`fases/fase-01-fundacao.md`](fases/fase-01-fundacao.md)

## Fase 02 — Shell visual

Resultado: navegação e layout responsivo do aplicativo, ainda sem processamento real.

- Definir mapa de telas e navegação.
- Construir sidebar/topbar, área principal e estados vazios.
- Criar tokens visuais, temas claro/escuro e componentes essenciais.
- Tornar navegação utilizável por teclado e com nomes acessíveis.
- Prototipar no cabeçalho um seletor de cores inspirado no Organon e validar a troca de paleta nos temas claro e escuro.
- Testar renderização e mudança de rota.
- Evitar implementar detalhes de todas as telas antes dos fluxos reais.

Arquivo: [`fases/fase-02-shell-visual.md`](fases/fase-02-shell-visual.md)

## Fase 03 — Contratos e arquivos

Resultado: arquivos selecionados chegam ao Rust de forma validada e são apresentados como itens de trabalho.

- Modelar DTOs, erros e respostas.
- Adicionar diálogo de arquivos e pasta com permissões mínimas.
- Implementar drag and drop e múltiplos arquivos.
- Ler metadados básicos sem modificar os originais.
- Validar caminho, extensão, assinatura/formato e duplicidade.
- Definir política de saída e colisão: renomear, ignorar ou substituir com confirmação.
- Testar entradas inválidas, nomes especiais e caminhos Unicode.

Arquivo: [`fases/fase-03-arquivos-e-contratos.md`](fases/fase-03-arquivos-e-contratos.md)

## Fase 04 — Imagens

Resultado: primeira fatia vertical completa, do formulário ao arquivo final.

- Definir `ImageProcessor` antes do adaptador.
- Implementar leitura e escrita atômica.
- Entregar redimensionamento, conversão e compressão de JPEG/PNG/WebP.
- Aplicar orientação quando necessário e tratar transparência explicitamente.
- Exibir comparação de tamanho e resultado por arquivo.
- Criar corpus de testes pequeno e legalmente reutilizável.
- Executar benchmark comparativo `image-rs` versus referência libvips/Sharp.
- Decidir formalmente manter `image-rs` ou criar adaptador libvips.

Arquivo: [`fases/fase-04-imagens.md`](fases/fase-04-imagens.md)

## Fase 05 — Fila e processos

Resultado: infraestrutura reutilizável para trabalhos demorados.

- Implementar estados de job e transições válidas.
- Fila FIFO com concorrência inicial igual a um.
- Canal de progresso ordenado entre Rust e React.
- Cancelamento cooperativo para Rust e encerramento de processos filhos.
- Diretórios/arquivos temporários e limpeza após erro.
- UI de fila com progresso, etapa, cancelamento, erro e repetição.
- Testar corrida entre conclusão e cancelamento.

Arquivo: [`fases/fase-05-fila-e-processos.md`](fases/fase-05-fila-e-processos.md)

## Fase 06 — Vídeo e áudio

Resultado: FFprobe inspeciona e FFmpeg processa arquivos com progresso confiável.

- Criar diagnóstico de presença e versão dos binários.
- Interpretar JSON do FFprobe.
- Traduzir presets do produto para argumentos aprovados.
- Implementar conversão/compressão de vídeo.
- Implementar conversão e extração de áudio.
- Interpretar progresso do FFmpeg sem depender de texto localizado.
- Cancelar, limpar temporários e preservar o original.
- Testar com fixtures curtas para manter a suíte rápida.

Arquivo: [`fases/fase-06-video-e-audio.md`](fases/fase-06-video-e-audio.md)

## Fase 07 — Downloader

Resultado: consulta e download controlado com yt-dlp.

- Executar diagnóstico de yt-dlp, FFmpeg e runtime JavaScript.
- Consultar metadados sem iniciar download.
- Modelar formatos/qualidades em opções compreensíveis.
- Implementar vídeo, somente áudio, miniatura e legendas.
- Implementar playlist com quantidade, capa quando disponível e nome base opcional.
- Interpretar progresso por saída estruturada.
- Cancelar árvore de processos e limpar arquivos parciais quando seguro.
- Incluir aviso de uso responsável e não implementar DRM/cookies no MVP.

Arquivo: [`fases/fase-07-downloader.md`](fases/fase-07-downloader.md)

## Fase 08 — Persistência e acabamento

Resultado: aplicativo coerente para uso diário.

- Persistir configurações com versão de schema.
- Registrar histórico, permitir repetir e limpar.
- Marcar jobs interrompidos por encerramento inesperado.
- Adicionar notificações locais opcionais.
- Finalizar temas, estados vazios, mensagens de erro e acessibilidade.
- Testar primeira execução, dados antigos/corrompidos e reset.
- Revisar desempenho percebido e consumo em filas maiores.

Arquivo: [`fases/fase-08-acabamento.md`](fases/fase-08-acabamento.md)

## Fase 09 — Distribuição Windows

Resultado: instalador reproduzível e pacote testado em máquina limpa.

- Escolher NSIS ou MSI com base no teste real; NSIS é o candidato inicial.
- Definir se os sidecars serão incluídos ou instalados separadamente.
- Fixar versões e hashes dos binários externos.
- Incluir avisos, licenças e fontes exigidas.
- Gerar SBOM/lista de dependências quando viável.
- Validar a retenção de no máximo duas compilações locais após cada build de instalador bem-sucedido, conforme o `AGENTS.md`.
- Testar instalação, primeira execução, desinstalação e caminhos sem ferramentas de desenvolvimento.
- Documentar limitações, formatos e procedimento de atualização manual.
- Assinatura de código e atualização automática permanecem posteriores ao MVP.

Arquivo: [`fases/fase-09-distribuicao.md`](fases/fase-09-distribuicao.md)

## Fila de trabalho para retomar

| Ordem | Tarefa | Evidência esperada |
|---|---|---|
| 01.2 | Ligar um diagnóstico Rust tipado ao React por `invoke` | Janela apresenta a resposta e trata falha da chamada |
| 01.3–01.4 | Preparar os gates de qualidade, Tailwind e shadcn/ui | Checagens passam e os estilos funcionam no WebView |
| 02.1–02.4 | Adaptar a identidade visual do Organon e montar a navegação do LpsToolkit | Comparação visual em tema claro e escuro, janela estreita e teste por teclado |
| 03–04 | Validar entradas e saídas e entregar o primeiro fluxo de imagens | Arquivo gerado a partir de uma amostra, com original preservado e colisões tratadas |
| 05–07 | Entregar fila, vídeo, áudio e Downloader nos blocos de cada fase | Trabalhos mostram progresso, aceitam cancelamento e limpam arquivos parciais |
| 08 | Persistir configurações e histórico e finalizar os fluxos | Preferências e histórico recuperados após reiniciar |
| 09 | Prototipar o instalador Windows e preparar preview e publicação | Instalação em Windows limpo, artefato verificado e preview aprovado pelo usuário |
| Fase própria de atualização | Definir e validar o mecanismo Tauri para reproduzir a experiência de atualização do Organon | Testes de detecção, integridade, consentimento, reinício e preservação de dados |

Cada linha resume uma entrega. A execução e as pausas continuam nos blocos dos arquivos de fase.

## Referência visual e de distribuição

A referência é o Organon em `Documentos/ChatGPT/Pessoal/estudo-desktop`. Na Fase 02, usar `src/density.css`, `src/styles.css`, `src/theme.css`, `src/typography.css` e `src/palettes.css` como fonte dos tokens. A densidade padrão tem sidebar de 200 px, topbar de 48 px, respiro de página de 24 px na horizontal e 18 px na vertical, título de 21 px em Segoe UI e controles de 32 px com fonte de 12 px e ícones de 14 px. Usar as superfícies cinza-neutras do escuro e os acentos suaves de sucesso, erro e progresso. Consultar o Financeiro, `Modal.jsx` e `PopupMenu.jsx` para compor filtros, diálogos e menus. Adaptar os componentes ao React/TypeScript e shadcn/ui do LpsToolkit.

- **Instalação inicial.** Usar o README e `build/installer.nsh` do Organon como referência do assistente em português. Apresentar o app, permitir escolher a pasta, mostrar “Confirmar instalação” e terminar com “Instalado com sucesso” e a opção “Abrir o LpsToolkit agora” desmarcada. Testar os atalhos e a preservação de dados. NSIS é o primeiro candidato do protótipo.
- **Atualização.** Usar o README e `electron/updates.cjs` como referência de comportamento. Verificar ao abrir e manualmente nas preferências. Oferecer “Depois” e “Baixar atualização”, mostrar progresso e pedir “Atualizar e reiniciar”. Validar origem e integridade do pacote, preservar dados e pasta instalada e reabrir o app após aplicar a atualização em segundo plano. O mecanismo, a assinatura e o canal próprios do LpsToolkit serão decididos na fase de atualização.
- **Releases.** Seguir o `AGENTS.md` do Organon para fechar o lote. Após a validação técnica e o encerramento dos ajustes, entregar um executável local sem instalação em `release/MAJOR.MINOR.PATCH-preview.N`, com número único e atalho. Depois da validação explícita do usuário e da autorização para subir, promover a versão estável, gerar instalador e SHA-256, fazer commit e push, publicar a Release com notas e verificar a distribuição. O canal estável recebe apenas versões finais. Na numeração estável, depois de PATCH 9 sobe-se MINOR, e MAJOR muda por solicitação explícita.
- **Validação do lote.** Executar as verificações focadas exigidas pelo bloco. Para validações amplas e smoke tests manuais no terminal do usuário, entregar um comando PowerShell copiável com caminho absoluto. Registrar a evidência antes de marcar uma tarefa como concluída.

## Marcos demonstráveis

- M1 — Fase 01: janela Tauri e primeira chamada Rust.
- M2 — Fase 04: primeira mídia processada ponta a ponta.
- M3 — Fase 06: processamento longo com progresso e cancelamento.
- M4 — Fase 07: download completo e pós-processado.
- M5 — Fase 09: instalação em outra máquina Windows.

