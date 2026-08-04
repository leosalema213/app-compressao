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
- Implementar playlist com quantidade e confirmação visíveis.
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
- Testar instalação, primeira execução, desinstalação e caminhos sem ferramentas de desenvolvimento.
- Documentar limitações, formatos e procedimento de atualização manual.
- Assinatura de código e atualização automática permanecem posteriores ao MVP.

Arquivo: [`fases/fase-09-distribuicao.md`](fases/fase-09-distribuicao.md)

## Marcos demonstráveis

- M1 — Fase 01: janela Tauri e primeira chamada Rust.
- M2 — Fase 04: primeira mídia processada ponta a ponta.
- M3 — Fase 06: processamento longo com progresso e cancelamento.
- M4 — Fase 07: download completo e pós-processado.
- M5 — Fase 09: instalação em outra máquina Windows.

