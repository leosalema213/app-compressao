# Fase 05 — Fila, progresso e processos

## Resultado

O aplicativo terá uma infraestrutura única para enfileirar, acompanhar e cancelar operações demoradas. Imagens poderão adotá-la e vídeo, áudio e Downloader nascerão sobre ela.

## Conceitos novos

- Máquina de estados.
- Concorrência e condição de corrida.
- Async no Rust.
- Canal de progresso Tauri.
- Processo pai/filho, stdout, stderr e exit code.
- Cancelamento cooperativo e forçado.

## Bloco 05.1 — Estado de job

- Implementar `JobId`, `JobKind`, `JobStatus`, `Progress` e resultado.
- Definir transições válidas e rejeitar transições impossíveis.
- Usar progresso indeterminado quando a ferramenta não fornece total confiável.
- Testar toda transição, incluindo repetição de cancelamento.

Checkpoint: desenhar o caminho de um job concluído e de um cancelado.

## Bloco 05.2 — Fila FIFO

- Implementar concorrência igual a um.
- Separar enfileirar de executar.
- Iniciar o próximo job após sucesso, falha ou cancelamento limpo.
- Impedir que falha em um job trave a fila.
- Deixar extensão para concorrência futura sem implementá-la agora.

## Bloco 05.3 — Canal de progresso

- Definir mensagem de progresso pequena e ordenada.
- Enviar etapa, percentual opcional, texto curto e métricas disponíveis.
- Consumir no frontend com cleanup correto.
- Limitar frequência para não sobrecarregar a UI.
- Testar atualização fora de ordem no reducer mesmo que o canal preserve ordem.

## Bloco 05.4 — `ProcessRunner`

- Definir contrato para executar binário e argumentos separados.
- Capturar stdout/stderr sem travar buffers.
- Implementar timeout de diagnóstico.
- Criar runner falso com scripts de progresso/erro/cancelamento.
- Preparar identificação de árvore de processos no Windows.

## Bloco 05.5 — Cancelamento e temporários

- Cancelamento idempotente.
- Encerrar processo e descendentes quando aplicável.
- Marcar arquivo parcial e limpar somente os artefatos conhecidos do job.
- Tratar corrida “concluiu enquanto cancelava”.
- Nunca remover caminho derivado de entrada não validada.

## Bloco 05.6 — Interface da fila

- Lista compacta com estados e ações válidas.
- Progresso determinado/indeterminado.
- Erro expansível, repetir e remover concluídos.
- Confirmação antes de cancelar lote quando houver múltiplos itens.

## Critérios de aceite

- [ ] Transições inválidas são impossíveis ou retornam erro.
- [ ] Fila continua após falha.
- [ ] Progresso não regride visualmente.
- [ ] Cancelamento repetido não causa pânico nem apaga arquivo incorreto.
- [ ] ProcessRunner nunca usa concatenação de shell.
- [ ] Testes de corrida principais passam repetidamente.
- [ ] UI representa todos os estados.
- [ ] O usuário aprovou avançar.

