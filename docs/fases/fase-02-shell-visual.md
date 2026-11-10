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

### Mapa e esboço aprovados para implementação

| Tela | Rota | Conteúdo inicial |
|---|---|---|
| Início | `#/` | Atalhos dos quatro módulos e indicação de funcionamento local |
| Imagens | `#/imagens` | Área vazia para seleção de imagens |
| Vídeo | `#/video` | Área vazia para arquivos de vídeo |
| Áudio | `#/audio` | Área vazia para conversão e extração |
| Downloader | `#/downloader` | Área vazia para downloads autorizados |
| Configurações | `#/configuracoes` | Preferências visuais e diagnóstico Rust |

Em janela larga, a barra lateral de 200 px lista as telas e o cabeçalho de 48 px identifica a seção e concentra os controles de aparência. Em janela estreita, a navegação passa para cima do conteúdo e o cabeçalho permite quebra de linha. A referência é a densidade padrão do Organon, com Segoe UI, títulos de 21 px, controles compactos e cinzas neutros no escuro. As páginas usam componentes e tokens compartilhados.

As rotas por hash usam links nativos e o histórico da janela, com uma tela de retorno ao Início para endereços desconhecidos. A preferência do sistema e a troca de cor no cabeçalho usam armazenamento local provisório. A menor largura prevista para o shell é 360 px de área útil.

## Bloco 02.2 — Tokens e componentes-base

Na revisão Pacton, o cabeçalho incorpora minimizar, maximizar/restaurar e fechar na mesma barra de Tema e Downloads. A janela usa decoração personalizada e região de arraste. A referência segue o Organon, com sidebar de 200 px, marca com versão, navegação de 12 px e pesos 500/600, cabeçalho de 48 px e breadcrumb com seta de 13 px.

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

- [x] Todas as páginas principais são alcançáveis.
- [x] Navegação ativa é visível e semântica.
- [x] Teclado percorre controles em ordem compreensível.
- [x] Tema claro/escuro não produz texto ilegível.
- [x] Layout permanece utilizável no tamanho mínimo de janela acordado.
- [x] Testes de navegação e shell passam.
- [x] O usuário aprovou continuar a sequência sem pausas didáticas.

## Não fazer nesta fase

- Não criar formulários completos de mídia.
- Não adicionar animações ou biblioteca de estado sem necessidade.
- Não polir todas as telas antes da primeira fatia vertical.

