# Fase 08 — Persistência e acabamento

## Resultado

O MediaToolkit guardará preferências e histórico de forma controlada, comunicará melhor seus estados e terá consistência suficiente para uso diário.

## Conceitos novos

- Persistência local e migração de schema.
- Estado durável versus estado transitório.
- Recuperação após encerramento inesperado.
- Notificação do sistema.
- Acessibilidade e UX de erro.

## Bloco 08.1 — Schema de configurações

- Definir valores realmente úteis: tema, pasta padrão opcional, política de colisão e notificações.
- Incluir versão de schema e defaults.
- Tratar arquivo ausente, antigo ou corrompido.
- Criar comando para restaurar padrões.
- Testar migração e recuperação.

## Bloco 08.2 — Histórico

- Definir campos mínimos: operação, status, horário, resumo e saída.
- Evitar armazenar logs brutos, cookies ou conteúdo de mídia.
- Limitar quantidade e permitir limpar.
- Repetir uma operação somente após revalidar entradas e opções.
- Marcar jobs ativos no encerramento anterior como interrompidos.

## Bloco 08.3 — Notificações

- Solicitar permissão apenas quando o usuário habilitar.
- Notificar conclusão/falha de trabalho longo quando a janela não estiver em foco.
- Evitar uma notificação por arquivo de lote.
- Testar permissão negada e opção desativada.

## Bloco 08.4 — Revisão de UX

- Padronizar loading, vazio, aviso, erro e sucesso.
- Revisar textos e ações de recuperação.
- Revisar foco, teclado, contraste, labels e regiões de progresso.
- Revisar janelas estreitas e conteúdo longo/Unicode.
- Garantir que operações perigosas exigem confirmação contextual.

## Bloco 08.5 — Desempenho e estabilidade

- Testar lotes maiores sem incluir fixtures gigantes no Git.
- Medir responsividade da interface durante processo pesado.
- Verificar vazamento de listeners e processos órfãos.
- Revisar logs e tamanho do histórico.

## Critérios de aceite

- [ ] Configurações sobrevivem ao reinício e migram com segurança.
- [ ] Dados corrompidos não impedem a abertura do app.
- [ ] Histórico pode ser repetido e apagado.
- [ ] Jobs interrompidos são representados corretamente.
- [ ] Notificações respeitam preferência/permissão.
- [ ] Navegação e formulários principais passam revisão de acessibilidade.
- [ ] Não há processo órfão nos cenários testados.
- [ ] O usuário aprovou avançar.

