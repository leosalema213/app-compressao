# Estratégia de testes e qualidade

## Pirâmide de testes

### Testes unitários rápidos

Executados constantemente.

- TypeScript: validações, formatadores, reducers/stores e mapeamento de DTOs com Vitest.
- React: componentes e hooks pelo comportamento observado, sem testar implementação interna.
- Rust: regras de domínio, serialização, nomes de saída, presets, transições de job e parsing.

### Testes de integração

Executados quando o módulo correspondente muda.

- Rust com diretórios temporários e fixtures reais pequenas.
- Adaptador `image-rs` produzindo arquivos que podem ser decodificados novamente.
- FFprobe/FFmpeg com mídia de poucos segundos.
- yt-dlp somente contra fixture/servidor controlado sempre que possível; testes reais de sites ficam manuais para evitar instabilidade e tráfego.
- Processo falso para simular progresso, erro, travamento e cancelamento.

### Testes ponta a ponta

Poucos fluxos críticos usando a abordagem oficial de WebDriver do Tauri quando a base estiver estável.

- Abrir aplicação e navegar.
- Selecionar uma fixture e concluir processamento de imagem.
- Iniciar e cancelar um job longo.
- Alterar configuração, reiniciar e confirmar persistência.
- Verificar mensagem quando um sidecar não está disponível.

## Comandos-alvo

Os nomes exatos serão criados na Fase 01 e registrados no `package.json`.

```text
npm run format:check
npm run lint
npm run typecheck
npm run test
npm run test:e2e
npm run build
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Não executaremos sempre a suíte mais lenta. Cada bloco informa o conjunto mínimo e o gate de fase executa a bateria completa pertinente.

## Matriz por camada

| Mudança | Mínimo obrigatório |
|---|---|
| Utilitário TypeScript | typecheck + teste unitário afetado |
| Componente React | typecheck + teste do componente + inspeção visual |
| Regra Rust | fmt + clippy direcionado + cargo test relevante |
| Comando Tauri | testes Rust + teste de contrato/frontend |
| Processamento de imagem | unitário + integração com fixture + inspeção do resultado |
| Argumentos FFmpeg/yt-dlp | teste de construção + integração controlada + caso de erro |
| Fila/cancelamento | unitário de estados + integração concorrente |
| Permissões/configuração Tauri | build Tauri + teste manual da função afetada |
| Empacotamento | build release + instalação em ambiente limpo |

## Fixtures

O repositório manterá arquivos pequenos, próprios ou com licença compatível, acompanhados de origem/licença quando necessário.

### Imagens mínimas

- JPEG paisagem com EXIF de orientação.
- JPEG grande com muito detalhe.
- PNG transparente.
- PNG sem transparência.
- WebP com dimensões ímpares.
- Arquivo corrompido e arquivo com extensão enganosa.

### Áudio/vídeo mínimos

- MP4 curto com H.264/AAC ou combinação aprovada.
- Vídeo sem áudio.
- Áudio WAV curto.
- Arquivo inválido.
- Nome com espaços, acentos e caracteres Unicode.

Fixtures grandes não entram no Git. Um script reproduzível poderá gerá-las localmente.

## Teste comparativo do motor de imagens

O `image-rs` será aprovado para o MVP se, no corpus acordado:

- 100% das saídas puderem ser decodificadas e tiverem o formato real esperado;
- dimensões e proporção seguirem o contrato;
- transparência for preservada ou convertida conforme regra explícita;
- orientação visual estiver correta;
- não houver regressão visual óbvia na comparação lado a lado;
- compressão com perda reduzir tamanho nas fixtures escolhidas sem ultrapassar o limite visual acordado;
- tempo e memória forem aceitáveis para uso pessoal em uma imagem grande;
- erros não corromperem nem substituírem o original.

Não será prometida equivalência abstrata com Sharp. A decisão será baseada no nosso escopo e corpus. Se falhar em requisito importante, a interface `ImageProcessor` receberá um adaptador libvips.

## Qualidade de saída

- Usar arquivo temporário na mesma unidade quando a promoção atômica for necessária.
- Validar o resultado antes de renomeá-lo para o destino final.
- Nunca apagar o original automaticamente.
- Em sobrescrita aprovada, preservar uma estratégia recuperável até o novo arquivo ser válido.
- Registrar comando lógico, versão da ferramenta, duração e erro; não registrar conteúdo sensível.

## Gate de fase

Para encerrar qualquer fase:

1. Todos os critérios de aceite estão marcados com evidência.
2. Análise estática não possui erro novo.
3. Testes relevantes passam em execução limpa.
4. Fluxo manual principal foi demonstrado.
5. Documentação e progresso refletem o código real.
6. O usuário respondeu ao checkpoint de entendimento e autorizou a próxima fase.

