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

## Bloco 04.4 — Conversão e compressão

- Mapear explicitamente formato de saída para encoder.
- Definir comportamento ao converter transparência para JPEG.
- Implementar qualidade JPEG/WebP e estratégia PNG.
- Detectar formato final por conteúdo nos testes, não apenas pela extensão.
- Mostrar tamanho original/final e evitar prometer redução para todo arquivo.

## Bloco 04.5 — Interface e lote

- Construir formulário orientado à operação.
- Validar antes de habilitar “Processar”.
- Mostrar resultado individual e resumo do lote.
- Permitir abrir a pasta de saída usando capacidade restrita.
- Testar interação principal com backend simulado.

## Bloco 04.6 — Gate `image-rs` versus libvips

- Rodar o corpus descrito em `docs/03-testes-e-qualidade.md` em release mode.
- Registrar tempo, tamanho, dimensões, transparência, orientação e inspeção visual.
- Produzir amostras equivalentes com libvips/Sharp como referência.
- Revisar os resultados com o usuário.
- Registrar uma decisão: manter `image-rs`, complementar casos específicos ou implementar libvips.
- Não iniciar vídeo antes dessa decisão.

## Critérios de aceite

- [ ] JPEG, PNG e WebP aprovados nas combinações definidas.
- [ ] Redimensionamento respeita contrato.
- [ ] Transparência e orientação seguem regras documentadas.
- [ ] Saída inválida nunca recebe nome final.
- [ ] Falha em um item não esconde o resultado dos demais.
- [ ] Original permanece intacto em sucesso, falha e cancelamento.
- [ ] Testes unitários e de integração passam.
- [ ] Comparativo foi registrado e a decisão do motor foi confirmada.
- [ ] O usuário aprovou avançar.

