# Arquitetura proposta

## Princípios

- Frontend apresenta estado e coleta intenção; Rust controla arquivos, processos e persistência.
- Operações externas recebem argumentos separados, nunca uma string de shell concatenada.
- Cada módulo de mídia implementa contratos pequenos e testáveis.
- Dependências nativas ficam atrás de adaptadores substituíveis.
- O aplicativo deve continuar utilizável sem internet, exceto o Downloader.
- Erros internos são estruturados e convertidos para mensagens úteis na interface.

## Fluxo principal

```text
React/UI
  -> cliente Tauri tipado
    -> comando Rust
      -> caso de uso
        -> serviço/adaptador
          -> image-rs ou processo externo
      <- resultado estruturado / canal de progresso
  <- store da interface
```

Para atualizações frequentes e ordenadas de progresso, o plano usa canais do Tauri. Eventos globais ficam reservados para notificações simples que realmente tenham múltiplos consumidores.

## Estrutura-alvo do código

```text
MediaToolkit/
├── app/                        # produto executável
│   ├── src/
│   │   ├── app/                # bootstrap, rotas e providers
│   │   ├── components/
│   │   │   ├── ui/             # componentes do shadcn/ui
│   │   │   └── shared/         # componentes reutilizáveis do produto
│   │   ├── features/
│   │   │   ├── home/
│   │   │   ├── images/
│   │   │   ├── video/
│   │   │   ├── audio/
│   │   │   ├── downloader/
│   │   │   ├── jobs/
│   │   │   └── settings/
│   │   ├── lib/
│   │   │   ├── tauri/          # única porta de chamada ao backend
│   │   │   ├── validation/
│   │   │   └── formatting/
│   │   ├── test/               # setup e utilitários de teste
│   │   └── types/              # DTOs usados pela interface
│   ├── src-tauri/
│   │   ├── capabilities/       # permissões mínimas do Tauri
│   │   ├── binaries/           # sidecars por target, quando aprovados
│   │   └── src/
│   │       ├── commands/       # fronteira IPC; pouca lógica
│   │       ├── domain/         # tipos e regras sem Tauri
│   │       ├── services/       # casos de uso
│   │       ├── image/          # ImageProcessor e adaptadores
│   │       ├── process/        # execução, progresso e cancelamento
│   │       ├── jobs/           # fila e ciclo de vida
│   │       ├── storage/        # configurações e histórico
│   │       └── error.rs        # erros estruturados
│   └── tests/
│       ├── fixtures/           # mídias pequenas e licenciadas para teste
│       ├── integration/        # testes com binários reais
│       └── e2e/                # fluxos da aplicação
└── docs/
```

Essa é uma direção, não uma obrigação de criar todas as pastas no primeiro dia. Cada pasta só nasce quando tiver responsabilidade real.

## Fronteiras

### Frontend

- Não acessa diretamente o sistema de arquivos por APIs web.
- Não chama executáveis.
- Não conhece detalhes de argumentos do FFmpeg ou yt-dlp.
- Mantém estado visual transitório e apresenta o estado recebido dos jobs.

### Comandos Tauri

- Recebem DTOs pequenos e serializáveis.
- Validam entrada básica e chamam um serviço.
- Retornam DTOs ou erros estruturados.
- Não concentram regra de negócio nem comandos gigantes.

### Domínio e serviços Rust

- Modelam operações, formatos, presets, caminhos e estados.
- Validam combinações antes de iniciar trabalho custoso.
- Podem ser testados sem abrir uma janela Tauri.

### Adaptadores

- `ImageProcessor`: começa com `image-rs`; permite trocar para libvips.
- `ProcessRunner`: executa e cancela sidecars; permite um falso em testes.
- `MediaProbe`: traduz a saída do FFprobe para metadados internos.
- `Downloader`: traduz opções do produto em argumentos seguros do yt-dlp.

## Modelo de job

```text
queued -> validating -> running -> succeeded
                         |   |
                         |   -> cancelling -> cancelled
                         -> failed
```

Cada job terá identificador, tipo, entrada, saída planejada, timestamps, progresso, etapa atual, aviso e erro opcional. O primeiro MVP executará um job por vez para reduzir uso de CPU, disco e memória.

## Estratégia do motor de imagens

1. Definir comportamento público de compressão, conversão e redimensionamento.
2. Implementar `ImageProcessor` com `image-rs`.
3. Testar JPEG, PNG e WebP, incluindo transparência, orientação e imagem grande.
4. Medir tempo, pico de memória quando viável, tamanho e validade do resultado.
5. Comparar amostras com uma referência libvips/Sharp.
6. Manter `image-rs` se atingir os critérios; caso contrário, criar adaptador libvips sem alterar comandos ou UI.

## Dependências externas

O conjunto provável do Downloader no Windows será `yt-dlp.exe`, `ffmpeg.exe`, `ffprobe.exe` e um runtime JavaScript suportado, provavelmente `deno.exe`, porque versões atuais do yt-dlp o recomendam para suporte completo a alguns sites. A presença, versão e compatibilidade serão verificadas por um serviço de diagnóstico antes de habilitar as ações.

Durante o desenvolvimento, os binários podem ser instalados separadamente. O empacotamento dentro do instalador só ocorrerá na Fase 09, após auditoria de licença e tamanho.

## Persistência

- Configurações: armazenamento local simples, versionado por schema.
- Histórico: registros pequenos em JSON/Store no MVP; sem copiar mídia para banco.
- Jobs em execução: não serão retomados automaticamente após encerramento no primeiro MVP; serão marcados como interrompidos.
- Caminhos e URLs: armazenados apenas quando úteis e com opção de limpar o histórico.

## Segurança

- Capacidades do Tauri limitadas aos comandos e arquivos necessários.
- Binários externos identificados por caminho controlado pelo aplicativo.
- Argumentos formados a partir de enums e valores validados.
- Nenhuma senha, cookie de navegador ou token no MVP.
- URL do Downloader passada como argumento literal, nunca interpretada por shell.
- Arquivo temporário usa extensão distinta e só é promovido ao nome final após sucesso.
