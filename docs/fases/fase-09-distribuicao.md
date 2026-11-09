# Fase 09 — Distribuição Windows

## Resultado

Um instalador x64 reproduzível poderá ser compartilhado com amigos e funcionar em Windows limpo, com dependências, licenças e limitações documentadas.

## Conceitos novos

- Build debug versus release.
- Target/arquitetura.
- Sidecar e recursos empacotados.
- Instalador, assinatura e reputação do Windows.
- Licenças de dependências e reprodutibilidade.

## Bloco 09.1 — Matriz de distribuição

- Confirmar Windows 10/11 x64.
- Comparar NSIS e MSI em um protótipo; NSIS é candidato inicial.
- Listar tudo que a máquina limpa precisa.
- Definir dados que permanecem após desinstalação e opção de remoção.

## Bloco 09.2 — Sidecars

- Decidir, para cada ferramenta, incluir ou exigir instalação externa.
- Fixar versão, origem, licença e hash.
- Nomear binários conforme target do Tauri.
- Incluir DLLs/recursos necessários e restringir capacidades.
- Testar diagnóstico usando apenas arquivos empacotados.

## Bloco 09.3 — Conformidade

- Determinar licença efetiva do build FFmpeg usado.
- Reunir licenças do FFmpeg, yt-dlp, runtime JS e demais componentes.
- Incluir atribuições e oferta/caminho de fonte quando exigidos.
- Criar tela/arquivo “Sobre e licenças”.
- Registrar que revisão jurídica será necessária antes de eventual comercialização.

## Bloco 09.4 — Build release

- Rodar todos os gates estáticos e testes pertinentes.
- Construir frontend e Rust em release.
- Gerar instalador com versão consistente.
- Calcular hash do artefato.
- Produzir notas de versão e limitações conhecidas.

## Bloco 09.5 — Máquina limpa

- Testar em VM ou usuário/máquina sem Node, Rust, FFmpeg e yt-dlp no PATH.
- Instalar, abrir e executar smoke tests de imagem, vídeo/áudio e Downloader autorizado.
- Verificar caminhos com espaços/Unicode e permissões comuns.
- Desinstalar e verificar arquivos/dados restantes conforme política.
- Corrigir e repetir o teste desde snapshot limpo quando houver mudança no pacote.

## Bloco 09.6 — Entrega

- Entregar instalador, hash, notas e licenças.
- Documentar atualização manual e como coletar diagnóstico.
- O bloco de atualização solicitado em 2026-11-09 usa o plugin oficial Tauri e segue a decisão em `docs/decisoes/0001-atualizacoes-e-backup.md`.
- Gere os artefatos assinados com `scripts/gerar-release.ps1`. Publique instalador, assinatura, checksum e manifesto juntos para ativar o canal estável.
- Registrar backlog para macOS/Linux somente após estabilidade do Windows.

## Critérios de aceite

- [ ] Instalador funciona em Windows x64 limpo.
- [ ] Aplicativo não depende de ferramentas de desenvolvimento instaladas.
- [ ] Sidecars corretos são usados, não versões acidentais do PATH.
- [ ] Licenças e origens estão incluídas.
- [ ] Smoke tests dos módulos passam após instalação.
- [ ] Desinstalação segue política documentada.
- [ ] Hash e notas de versão foram gerados.
- [ ] O usuário aprovou o MVP distribuível.

