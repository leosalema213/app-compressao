# LpsToolkit

Aplicativo desktop local para processamento de imagens, vídeo e áudio e para download de mídias, construído com Tauri 2, React, TypeScript e Rust.

O planejamento e o estado atual do projeto estão em [`docs/README.md`](docs/README.md).

## Preview para Windows

Em [GitHub Releases](https://github.com/leosalema213/app-compressao/releases), abra a versão de preview e baixe o arquivo `_x64-setup.exe` em **Assets**. O instalador é para Windows 10/11 x64. Para atualizar durante o preview, baixe e execute o instalador da versão mais recente.

- **Imagens** — Comprima, redimensione e converta lotes de JPEG, PNG e WebP, com previsão da saída e preservação dos originais.
- **Vídeo e áudio** — Consulte metadados, gere MP4 com perfis de compressão, converta WAV em MP3 e extraia MP3 de um vídeo. Essas funções usam FFmpeg e FFprobe disponíveis no PATH.
- **Downloader** — Consulte opções e baixe MP4, MP3, miniaturas PNG e legendas WebVTT quando a fonte oferecer esses formatos. Playlists de até 20 itens exigem confirmação. Esta função também usa yt-dlp no PATH. Algumas fontes exigem um runtime JavaScript.
- **Preferências** — Troque o tema e a cor de destaque pelo cabeçalho e defina uma pasta padrão. Acompanhe o progresso e cancele tarefas em execução.

O preview usa o assistente padrão em português, por usuário. As ferramentas externas são instaladas separadamente. Use o Downloader em mídias que você tem autorização para baixar.

## Decisões já confirmadas

- Primeiro alvo: Windows 10/11 x64.
- Uso inicial: pessoal e compartilhamento gratuito com amigos.
- Frontend: React, TypeScript, Tailwind CSS e shadcn/ui.
- Desktop/backend: Tauri 2 e Rust.
- Imagens: começar com `image-rs`, atrás de uma interface substituível; migrar para libvips se os testes comparativos não atingirem os critérios definidos.
- Vídeo e áudio: FFmpeg/FFprobe executados localmente.
- Downloads: yt-dlp executado localmente, com dependências verificadas pelo aplicativo.
- Processo didático: uma etapa pequena por vez, com explicação, implementação, testes, revisão e confirmação do usuário antes de avançar.

## Como continuar

1. Abra [`docs/progresso.md`](docs/progresso.md) para ver o ponto atual.
2. Leia a fase marcada como `PRÓXIMA`.
3. Execute apenas um bloco didático por vez.
4. Rode os testes e valide os critérios de aceite.
5. Atualize o progresso e aguarde a confirmação do usuário.

