# Fase 07 — Downloader

## Resultado

O usuário poderá consultar uma URL e iniciar downloads suportados pelo yt-dlp com opções de qualidade, áudio, playlist, miniatura, legendas, pasta, progresso e cancelamento.

## Conceitos novos

- Metadados remotos e disponibilidade variável.
- Streams separados de vídeo/áudio e pós-processamento.
- Dependência externa sujeita a mudanças de sites.
- Processo e subprocessos.
- Uso responsável e limites do produto.

## Bloco 07.1 — Escopo responsável e diagnóstico

- Exibir aviso de que o usuário deve ter direito de baixar o conteúdo.
- Não implementar DRM, paywall, cookies ou autenticação no MVP.
- Verificar yt-dlp, FFmpeg/FFprobe e runtime JavaScript suportado.
- Registrar versões e explicar dependências ausentes.
- Avaliar Deno como runtime auxiliar recomendado sem incluí-lo ainda no instalador.

## Bloco 07.2 — Consulta de URL

- Validar sintaxe e esquemas permitidos.
- Passar URL como argumento literal, sem shell.
- Consultar JSON do yt-dlp sem baixar.
- Modelar título, duração, thumbnail, playlist e formatos disponíveis.
- Limitar tamanho de saída e tratar resposta inesperada.
- Não persistir URL automaticamente antes da decisão da Fase 08.

## Bloco 07.3 — Seleção de qualidade

- Converter formatos técnicos em opções compreensíveis.
- Opções iniciais propostas: melhor, 1080p, 720p, 480p e somente áudio, quando disponíveis.
- Mostrar quando vídeo e áudio serão mesclados.
- Manter seleção técnica avançada fora do MVP.
- Testar mapeamento com JSONs gravados, sem depender do site.

## Bloco 07.4 — Download individual

- Gerar template de nome seguro.
- Integrar pasta de saída e colisão.
- Usar saída estruturada/progress template do yt-dlp.
- Integrar fila, progresso, pós-processamento e cancelamento.
- Marcar arquivos parciais e limpar apenas o que pertence ao job.

## Bloco 07.5 — Áudio, miniatura e legendas

- Implementar somente áudio usando pós-processamento aprovado.
- Baixar miniatura opcional.
- Listar/selecionar legendas disponíveis; não prometer tradução automática.
- Validar resultados e apresentar avisos quando um extra não existir.

## Bloco 07.6 — Playlists

- Detectar playlist e mostrar quantidade/título quando disponíveis.
- Solicitar confirmação explícita com destino e estimativa.
- Permitir item único quando a URL também representa playlist.
- Representar progresso geral e por item.
- Cancelar sem iniciar itens restantes.

## Bloco 07.7 — Testes resilientes

- Parsers e argumentos usam fixtures JSON.
- Integração automatizada prefere servidor/fixture controlado.
- Um teste manual real e autorizado verifica compatibilidade atual.
- Falha externa recebe categoria própria e instrução de diagnóstico/atualização.

## Critérios de aceite

O fluxo atual consulta metadados, mostra resoluções MP4 detectadas e oferece MP4, MP3, miniatura PNG e legenda WebVTT quando disponíveis. O diagnóstico mostra yt-dlp, FFmpeg, FFprobe, Node e Deno. O executável de teste é o yt-dlp oficial 2026.10.26, com SHA-256 `f5389d3698ecde4ef06f4e6b1ce0294fe9ac1396228c8990b8d3416d6d177a7d`. Está em `app/src-tauri/target/validacao-local/download-tools`. Para a execução de desenvolvimento, acrescente a pasta ao PATH da sessão que inicia o Tauri. O empacotamento fica para a distribuição.

Playlists têm limite de 20 entradas e exigem a confirmação da quantidade e do destino. Cada item é processado na mesma fila. Cancelar preserva os arquivos já publicados e encerra o processamento restante. A validação com fonte externa autorizada e o teste de cancelamento específico de playlist permanecem pendentes.

- [x] URL inválida não inicia processo.
- [x] Consulta não baixa conteúdo.
- [ ] Qualidades disponíveis são apresentadas sem inventar opções.
- [x] Download individual, áudio e extras acordados funcionam em amostras controladas.
- [ ] Playlist exige confirmação e pode ser cancelada.
- [x] Nenhum comando usa shell concatenado.
- [x] Dependências e versões aparecem no diagnóstico.
- [x] Limites de uso estão visíveis.
- [ ] O usuário aprovou avançar.

