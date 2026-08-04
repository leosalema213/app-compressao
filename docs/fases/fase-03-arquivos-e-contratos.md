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

## Critérios de aceite

- [ ] Seleção única, múltipla e drag and drop reutilizam o mesmo fluxo.
- [ ] Pasta de saída pode ser escolhida e validada.
- [ ] Nenhum arquivo de entrada é alterado.
- [ ] Extensão enganosa e arquivo inválido geram erro compreensível.
- [ ] Nomes Unicode e espaços funcionam.
- [ ] Colisão nunca sobrescreve silenciosamente.
- [ ] Permissões Tauri são mínimas e revisadas.
- [ ] O usuário aprovou avançar.

