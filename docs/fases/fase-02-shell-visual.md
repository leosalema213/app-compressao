# Fase 02 — Shell visual e navegação

## Resultado

O aplicativo terá identidade visual inicial, navegação estável e páginas vazias úteis para todos os módulos. Ainda não haverá processamento real.

## Conceitos novos

- Shell de aplicação versus conteúdo de feature.
- Rotas e estado de navegação.
- Design tokens e temas.
- Acessibilidade por teclado e semântica.
- Testes de comportamento de componentes.

## Bloco 02.1 — Mapa e wireframe

- Revisar Home, Imagens, Vídeo, Áudio, Downloader e Configurações.
- Confirmar sidebar, topbar e comportamento em janela estreita.
- Definir conteúdo mínimo de cada estado vazio.
- Escolher direção visual, cores e base do shadcn/ui com o usuário.
- Criar wireframe simples antes do acabamento.

Checkpoint: diferenciar componente de layout de componente específico de uma feature.

## Bloco 02.2 — Tokens e componentes-base

- Definir cores semânticas, tipografia, espaços, bordas e foco.
- Implementar AppShell, Sidebar, Topbar, PageHeader e EmptyState.
- Reutilizar componentes do shadcn/ui sem criar abstrações prematuras.
- Validar contraste e foco visível.

Testes: renderização, nomes acessíveis e estado ativo de navegação.

## Bloco 02.3 — Rotas e páginas

- Implementar as rotas decididas.
- Criar uma página fina por módulo.
- Tratar rota desconhecida.
- Preservar estado que deva sobreviver à navegação e limpar o que for local.

Testes: cada item leva à página correta; voltar/avançar não quebra o shell.

## Bloco 02.4 — Tema

- Implementar claro, escuro e preferência do sistema.
- Persistência definitiva fica para a Fase 08; nesta fase basta um adaptador provisório isolado.
- Verificar componentes principais nos dois temas.

## Critérios de aceite

- [ ] Todas as páginas principais são alcançáveis.
- [ ] Navegação ativa é visível e semântica.
- [ ] Teclado percorre controles em ordem compreensível.
- [ ] Tema claro/escuro não produz texto ilegível.
- [ ] Layout permanece utilizável no tamanho mínimo de janela acordado.
- [ ] Testes de navegação e shell passam.
- [ ] O usuário aprovou avançar.

## Não fazer nesta fase

- Não criar formulários completos de mídia.
- Não adicionar animações ou biblioteca de estado sem necessidade.
- Não polir todas as telas antes da primeira fatia vertical.

