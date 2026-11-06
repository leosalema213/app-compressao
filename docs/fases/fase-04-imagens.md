# Fase 04 — Imagens

## Resultado

O usuário poderá comprimir, converter e redimensionar JPEG, PNG e WebP em lote. Esta será a primeira funcionalidade completa e também a validação de `image-rs`.

## Conceitos novos

- Trait e implementação em Rust.
- Decodificação, buffer de pixels e codificação.
- Compressão com e sem perda.
- Proporção, modos de ajuste e transparência.
- Escrita temporária/atômica.
- Benchmark orientado a decisão.

## Bloco 04.1 — Contrato do produto

- Apresentar opções com exemplos visuais.
- Confirmar formatos, intervalos de qualidade e presets.
- Confirmar modos de redimensionamento do MVP: largura/altura, preservar proporção e não ampliar por padrão.
- Confirmar política de metadados: orientação aplicada; preservação ampla de EXIF/ICC será medida e poderá exigir libvips.
- Definir resultado e progresso por arquivo.

Primeira entrega em uso: compressão em lote no formato de origem. JPEG usa qualidade 80. PNG usa compressão sem perdas com filtro adaptativo e WebP usa o encoder sem perdas. A orientação é aplicada aos pixels, a transparência é mantida em PNG e WebP e a saída não copia EXIF ou ICC nesta entrega. Imagens animadas são rejeitadas. O processamento limita arquivos codificados a 64 MiB e a decodificação a 40 megapixels e 256 MiB de pixels.

Cada item termina em sucesso com nome final e tamanhos em bytes, ou em falha com código. Uma falha não impede os demais itens. O Rust valida o temporário por decodificação e publica na pasta escolhida sem sobrescrever. Se outro processo ocupar o nome, o Rust procura o próximo sufixo. Uma saída maior também é exibida com os dois tamanhos.

## Bloco 04.2 — `ImageProcessor`

- Definir tipos de entrada/saída e trait sem depender de Tauri.
- Criar implementação falsa para testar o serviço.
- Implementar serviço de processamento em lote.
- Testar ordem, falha parcial e nomes de saída antes de decodificar imagens.

Checkpoint: explicar como a trait permite trocar `image-rs` por libvips sem mudar a tela.

## Bloco 04.3 — Redimensionamento

- Implementar adaptador `image-rs` com filtro de qualidade acordado.
- Aplicar orientação necessária antes de calcular dimensões.
- Tratar largura/altura inválidas, proporção e tentativa de ampliação.
- Escrever em temporário, decodificar para validar e promover ao destino.

Testes: dimensões exatas, proporção, transparência, imagem inválida e original intacto.

Implementado com limites opcionais de largura e altura, de 1 a 16.384 px por campo preenchido. Com os dois campos preenchidos, a imagem cabe dentro do retângulo. Dimensões menores são preservadas. Ambos vazios mantêm a compressão simples. A orientação EXIF é aplicada antes do cálculo e o filtro Lanczos3 usa alfa pré-multiplicado para evitar que cores invisíveis contaminem as bordas. O resultado exibe as dimensões efetivamente gravadas.

## Bloco 04.4 — Conversão e compressão

- Mapear explicitamente formato de saída para encoder.
- Definir comportamento ao converter transparência para JPEG.
- Implementar qualidade JPEG/WebP e estratégia PNG.
- Detectar formato final por conteúdo nos testes, não apenas pela extensão.
- Mostrar tamanho original/final e evitar prometer redução para todo arquivo.

O seletor permite manter o formato ou converter o lote para JPEG, PNG ou WebP, inclusive junto do redimensionamento. JPEG tem qualidade de 1 a 100, com padrão 80, e compõe transparência sobre branco. PNG mantém o alfa e a profundidade de origem quando suportada. O encoder WebP do `image-rs` é sem perdas para pixels de 8 bits por canal. Conversões de PNG de 16 bits para WebP reduzem a profundidade antes da codificação. A prévia de nomes e a gravação usam a extensão de destino e a mesma regra de colisão.

## Bloco 04.5 — Interface e lote

- Construir formulário orientado à operação.
- Validar antes de habilitar “Processar”.
- Mostrar resultado individual e resumo do lote.
- Permitir abrir a pasta de saída usando capacidade restrita.
- Testar interação principal com backend simulado.

O último lote mostra quantos arquivos foram gravados, quantos falharam e a diferença total de bytes entre as entradas e as saídas bem-sucedidas. O botão **Abrir pasta de saída** usa a pasta do resultado, mesmo que o usuário já tenha escolhido outro destino no formulário. O Rust só abre a pasta local do lote gravado com sucesso mais recente, revalida o caminho e chama o Opener internamente. A interface continua sem permissões genéricas do plugin.

## Bloco 04.6 — Gate `image-rs` versus libvips

- Rodar o corpus descrito em `docs/03-testes-e-qualidade.md` em release mode.
- Registrar tempo, tamanho, dimensões, transparência, orientação e inspeção visual.
- Produzir amostras equivalentes com libvips/Sharp como referência.
- Revisar os resultados com o usuário.
- Registrar uma decisão: manter `image-rs`, complementar casos específicos ou implementar libvips.
- Não iniciar vídeo antes dessa decisão.

### Resultado do comparativo no Windows x64

Corpus sintético gerado por `scripts/benchmark-images.cjs` e executado com `image-rs 0.25.10` em release, Sharp 0.35.5 e libvips 8.18.7. Limites de 1600 × 1200 e JPEG na qualidade nominal 80. O tempo Rust inclui validação e gravação do temporário, enquanto Sharp mede a produção do buffer.

| Caso | Dimensões finais | image-rs, bytes / ms | Sharp, bytes / ms |
|---|---:|---:|---:|
| JPEG com orientação | 480 × 720 | 92.462 / 82 | 76.460 / 25 |
| JPEG detalhado de 12 MP | 1600 × 1200 | 656.252 / 741 | 406.096 / 252 |
| PNG transparente | 640 × 480 | 92.256 / 249 | 72.790 / 149 |
| PNG opaco | 640 × 480 | 51.822 / 117 | 43.406 / 77 |
| WebP ímpar | 401 × 307 | 83.122 / 78 | 23.966 / 141 |

As cinco saídas decodificaram com formato, dimensões e transparência esperados. PNG e WebP tiveram igualdade de pixels visíveis com a referência. Os JPEGs ficaram menores que as entradas. Na inspeção lado a lado, a orientação e as cores se mantiveram e o JPEG sintético detalhado mostrou diferenças na textura fina entre os métodos de redimensionamento. O pico amostrado do processo Rust no corpus foi 127,6 MiB. O arquivo corrompido e a extensão enganosa foram rejeitados e as entradas permaneceram intactas.

Decisão de implementação: manter `image-rs` no MVP. O WebP já otimizado cresceu de 23.966 para 83.122 bytes nesse caso. A interface informa o aumento. O resultado sintético orienta esta entrega e fica disponível para revisão com o usuário.

Para reproduzir, instalar `sharp@0.35.5` em um prefixo temporário, executar `node scripts/benchmark-images.cjs prepare <corpus> <módulo-sharp>`, definir `LPSTOOLKIT_IMAGE_BENCH` para a pasta absoluta do corpus, rodar `cargo test --manifest-path app/src-tauri/Cargo.toml --release --lib benchmark_image_corpus -- --ignored` e executar o script com `compare` e os mesmos caminhos.

## Critérios de aceite

- [x] JPEG, PNG e WebP aprovados nas combinações definidas.
- [x] Redimensionamento respeita contrato.
- [x] Transparência e orientação seguem regras documentadas.
- [x] Saída inválida nunca recebe nome final.
- [x] Falha em um item não esconde o resultado dos demais.
- [x] Original permanece intacto em sucesso, falha e cancelamento.
- [x] Testes unitários e de integração passam.
- [ ] Comparativo foi registrado e a decisão do motor foi confirmada.
- [ ] O usuário aprovou avançar.

