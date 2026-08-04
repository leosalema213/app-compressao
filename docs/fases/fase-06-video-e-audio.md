# Fase 06 — Vídeo e áudio

## Resultado

O MediaToolkit inspecionará, converterá e comprimirá vídeo e converterá/extrairá áudio por FFmpeg, integrado à fila e com cancelamento seguro.

## Conceitos novos

- Contêiner, codec, bitrate, resolução e frame rate.
- Compressão com CRF/qualidade versus tamanho alvo.
- FFprobe e metadados estruturados.
- Mapeamento de streams.
- Parsing de progresso de processo externo.

## Bloco 06.1 — Diagnóstico das ferramentas

- Confirmar origem dos binários usados no desenvolvimento.
- Obter versões de FFmpeg e FFprobe.
- Listar capacidades/encoders necessários de forma controlada.
- Mostrar diagnóstico na interface quando indisponível ou incompatível.
- Não empacotar os binários ainda.

## Bloco 06.2 — FFprobe

- Executar com saída JSON.
- Modelar duração, streams, dimensões, codecs e bitrate disponível.
- Tratar metadado ausente ou duração desconhecida.
- Testar parser com fixtures JSON antes da integração real.

Checkpoint: diferenciar contêiner MP4 de codec H.264, por exemplo.

## Bloco 06.3 — Presets de vídeo

- Apresentar e confirmar combinações suportadas.
- Traduzir “rápido/equilibrado/menor arquivo” para configuração explícita.
- Manter opção avançada pequena, sem expor linha de comando livre.
- Validar formato, codec, qualidade, resolução e caminho.
- Testar o vetor de argumentos exatamente.

## Bloco 06.4 — Conversão e compressão

- Executar pelo ProcessRunner.
- Usar arquivo temporário e validar saída com FFprobe.
- Interpretar progresso pela saída própria de progresso do FFmpeg.
- Integrar fila, UI, cancelamento e relatório final.
- Testar vídeo com áudio, sem áudio e entrada inválida.

## Bloco 06.5 — Áudio

- Confirmar formatos/presets iniciais.
- Converter arquivo de áudio.
- Extrair stream de vídeo com mapeamento explícito.
- Diferenciar remux/cópia de stream de recodificação quando aplicável.
- Validar saída com FFprobe.

## Bloco 06.6 — Robustez

- Nomes com espaços/Unicode.
- Duração desconhecida e progresso indeterminado.
- Disco insuficiente/sem permissão.
- Cancelamento e fechamento do aplicativo durante execução.
- Logs úteis sem expor conteúdo ou usar texto bruto como mensagem principal.

## Critérios de aceite

- [ ] Diagnóstico detecta ferramentas ausentes/incompatíveis.
- [ ] FFprobe gera metadados internos estáveis.
- [ ] Presets geram somente argumentos aprovados.
- [ ] Conversão/compressão de vídeo funciona nas fixtures acordadas.
- [ ] Conversão e extração de áudio funcionam.
- [ ] Progresso e cancelamento funcionam pela fila.
- [ ] Saída é validada antes do nome final.
- [ ] Licença/origem do build de desenvolvimento foi registrada.
- [ ] O usuário aprovou avançar.

