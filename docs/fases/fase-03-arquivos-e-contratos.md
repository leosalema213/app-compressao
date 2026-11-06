# Fase 03 — Arquivos e contratos

## Resultado

A interface selecionará e exibirá arquivos reais, enquanto o Rust validará metadados e caminhos sem modificar o conteúdo.

## Conceitos novos

- DTO na fronteira IPC.
- `struct`, `enum`, Serde, `Result` e erro estruturado em Rust.
- Path/PathBuf e diferenças de caminho no Windows.
- Validação por conteúdo versus extensão.
- Permissões/capabilities do Tauri.

## Bloco 03.1 — Linguagem comum entre frontend e backend

- Definir `MediaKind`, `SelectedMedia`, `AppError` e operações iniciais.
- Modelar os tipos Rust e TypeScript de forma pequena.
- Criar fixtures JSON de contrato para detectar divergência.
- Definir códigos de erro estáveis e mensagens separadas.

Testes: serialização Rust e leitura TypeScript da mesma fixture.

Checkpoint: explicar por que não retornamos uma mensagem de erro solta para todos os casos.

O contrato inicial atende Imagens. `inspect_images` recebe caminhos absolutos e devolve uma entrada aceita ou rejeitada por caminho. O resultado inclui nome, tamanho, caminho resolvido e formato. Rejeições usam `AppError.code` e o frontend escolhe a mensagem em português. O JSON compartilhado em `app/tests/fixtures/image-inspection.json` é usado nos testes Rust e React.

## Bloco 03.2 — Seleção de arquivos e pasta

- Adicionar plugins oficiais necessários com permissões mínimas.
- Abrir diálogo de um ou vários arquivos.
- Abrir diálogo de pasta de saída.
- Cancelamento do diálogo não é erro.
- Frontend encaminha caminhos escolhidos ao comando Rust de inspeção.

Testes: wrappers frontend com mocks; Rust com caminhos temporários.

## Bloco 03.3 — Inspeção e validação

- Ler tamanho, nome e formato detectável.
- Rejeitar diretório quando arquivo é esperado e vice-versa.
- Tratar arquivo ausente, inacessível, vazio e extensão enganosa.
- Não carregar mídia grande inteira apenas para montar a lista.
- Normalizar duplicatas sem apagar intenção do usuário.

Em Windows, o fluxo inicial aceita caminhos absolutos com letra de unidade e rejeita caminhos UNC diretos, namespaces de dispositivo e fluxos alternativos. A inspeção verifica a extensão e lê até 12 bytes para reconhecer JPEG, PNG ou WebP. A decodificação completa fica para a Fase 04. A seleção de pasta verifica se o diretório existe, sem testar gravação. O aplicativo usa `dialog:allow-open` na janela principal.

## Bloco 03.4 — Drag and drop e lista

- Integrar drag and drop do Tauri.
- Reutilizar o mesmo pipeline da seleção por diálogo.
- Mostrar estado válido, aviso e erro por item.
- Permitir remover itens e limpar a seleção.

## Bloco 03.5 — Saída e colisão

- Apresentar exemplos e confirmar política padrão.
- Implementar gerador determinístico de nomes.
- Opções: renomear automaticamente, ignorar, ou substituir mediante confirmação.
- Proteger contra entrada e saída iguais.
- Testar espaços, acentos, nomes reservados e arquivo já existente.

Política confirmada: manter o nome quando estiver livre. Se o original, um item da pasta ou outra saída do lote ocupar esse nome, testar `foto-2.png`, `foto-3.png` e assim por diante. O botão **Pré-visualizar nomes** consulta o Rust e mostra a previsão sem criar arquivos. A Fase 04 deve repetir a checagem na gravação e publicar o resultado com uma operação que falhe se o nome tiver sido ocupado desde a previsão.

## Critérios de aceite

- [ ] Seleção única, múltipla e drag and drop reutilizam o mesmo fluxo.
- [x] Pasta de saída pode ser escolhida e validada.
- [x] Nenhum arquivo de entrada é alterado.
- [x] Extensão enganosa e arquivo inválido geram erro compreensível.
- [x] Nomes Unicode e espaços funcionam.
- [x] Colisão nunca sobrescreve silenciosamente. Duas gravações concorrentes da Fase 04 receberam nomes distintos e preservaram os arquivos existentes.
- [x] Permissões Tauri são mínimas e revisadas.
- [ ] O usuário aprovou avançar.

