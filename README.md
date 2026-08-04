# MediaToolkit

Aplicativo desktop local para processamento de imagens, vídeo e áudio e para download de mídias, construído com Tauri 2, React, TypeScript e Rust.

O projeto está inicialmente na fase de planejamento. O documento original permanece em `MediaToolkit_-_Planejamento.docx`; o plano executável e versionável está em [`docs/README.md`](docs/README.md).

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

