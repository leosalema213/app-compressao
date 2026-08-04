# Plano executável do MediaToolkit

Este diretório transforma o planejamento original em um processo que pode ser executado, testado, interrompido e retomado com segurança.

## Ordem de leitura

1. [`00-produto-e-escopo.md`](00-produto-e-escopo.md)
2. [`01-arquitetura.md`](01-arquitetura.md)
3. [`02-roadmap.md`](02-roadmap.md)
4. [`03-testes-e-qualidade.md`](03-testes-e-qualidade.md)
5. [`04-metodo-de-aprendizado.md`](04-metodo-de-aprendizado.md)
6. [`05-riscos-e-distribuicao.md`](05-riscos-e-distribuicao.md)
7. [`progresso.md`](progresso.md)
8. O arquivo da fase atual em [`fases/`](fases/)

## Regra de execução

Uma fase não é uma única tarefa. Cada fase contém blocos pequenos. Em cada bloco seguimos:

1. Explicação do conceito e do resultado esperado.
2. Confirmação de escopo quando houver escolha relevante.
3. Mudança pequena e revisável.
4. Teste automatizado sempre que viável.
5. Validação manual curta.
6. Revisão do diff e atualização de `progresso.md`.
7. Pergunta de entendimento e pausa obrigatória.

## Fases

| Fase | Resultado principal | Estado |
|---|---|---|
| 00 | Ambiente verificado e repositório preparado | Próxima |
| 01 | Aplicativo Tauri mínimo compilando e testado | Pendente |
| 02 | Shell visual e navegação acessível | Pendente |
| 03 | Contratos Rust/TypeScript e seleção segura de arquivos | Pendente |
| 04 | Primeiro fluxo completo de imagens | Pendente |
| 05 | Fila, progresso e cancelamento reutilizáveis | Pendente |
| 06 | Processamento de vídeo e áudio com FFmpeg | Pendente |
| 07 | Downloader com yt-dlp | Pendente |
| 08 | Histórico, configurações, notificações e acabamento | Pendente |
| 09 | Instalador Windows e pacote compartilhável | Pendente |

## Referências oficiais

- [Pré-requisitos do Tauri 2](https://v2.tauri.app/start/prerequisites/)
- [Criação de projeto Tauri 2](https://v2.tauri.app/start/create-project/)
- [Comandos, eventos e canais do Tauri](https://v2.tauri.app/develop/calling-rust/)
- [Executáveis auxiliares no Tauri](https://v2.tauri.app/develop/sidecar/)
- [Testes no Tauri](https://v2.tauri.app/develop/tests/)
- [Tailwind CSS com Vite](https://tailwindcss.com/docs/installation/using-vite)
- [shadcn/ui com Vite](https://ui.shadcn.com/docs/installation/vite)
- [image-rs](https://github.com/image-rs/image)
- [Sharp/libvips](https://sharp.pixelplumbing.com/)
- [FFmpeg](https://ffmpeg.org/documentation.html)
- [Licenciamento do FFmpeg](https://ffmpeg.org/legal.html)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp/blob/master/README.md)

