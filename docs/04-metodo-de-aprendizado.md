# Método de aprendizado

## Perfil considerado

- HTML/CSS, JavaScript/TypeScript, React e Git: nível júnior/pleno.
- Terminal: experiência inicial.
- Rust: começando do zero.

Por isso, conceitos conhecidos de frontend serão tratados de forma compacta. Terminal, Rust, processos, sistema de arquivos e arquitetura desktop receberão explicações mais graduais.

## Ciclo de cada bloco

### 1. Contexto

O Codex explica:

- qual problema será resolvido;
- como ele se conecta ao que já existe;
- quais arquivos provavelmente mudarão;
- qual conceito novo será aprendido;
- como saberemos que funcionou.

### 2. Previsão

Antes de executar, o usuário é convidado a prever algo pequeno: o resultado de um comando, o tipo retornado, a transição de estado ou onde uma regra deveria ficar. Isso transforma acompanhamento passivo em prática.

### 3. Implementação pequena

- Um conceito principal por bloco.
- Diff curto e localizado.
- Nada de gerar um módulo inteiro sem percorrer as partes importantes.
- Boilerplate mecânico pode ser automatizado, mas será resumido e apontado.

### 4. Verificação

- Rodar o teste mais próximo primeiro.
- Ler falhas em conjunto quando forem didáticas.
- Rodar validação mais ampla ao fechar o bloco.
- Fazer uma validação manual quando a experiência visual ou um arquivo gerado importar.

### 5. Recapitulação

O Codex apresenta:

- o que mudou;
- por que ficou nessa camada;
- o que os testes garantem e o que não garantem;
- uma pequena pergunta de entendimento.

### 6. Pausa obrigatória

O próximo bloco só começa depois de uma resposta explícita do usuário. Se houver dúvida, explicamos por outro exemplo antes de avançar.

## Progressão de Rust

Os conceitos serão introduzidos nesta ordem aproximada:

1. `cargo`, crate, módulo, função e tipos básicos.
2. `Result`, `Option` e operador `?`.
3. Structs, enums, `match` e métodos.
4. Empréstimo e referências em exemplos concretos, sem aula abstrata longa.
5. Traits através de `ImageProcessor` e `ProcessRunner`.
6. Erros próprios e conversões.
7. Serialização com Serde e fronteira Tauri.
8. Assíncrono, processos filhos e canais.
9. Estado compartilhado, fila e cancelamento.
10. Testes, mocks simples e integração.

## Progressão de terminal

1. Diretório atual e caminhos relativos/absolutos.
2. Comando, argumento, opção e código de saída.
3. PATH e como o sistema encontra executáveis.
4. Variáveis de ambiente sem expor segredos.
5. Processos pai/filho, stdout e stderr.
6. Ferramentas do projeto: npm, cargo, rustup e Git.
7. Leitura de logs e diagnóstico sem tentar comandos aleatórios.

## Perguntas de checkpoint

Exemplos usados no fim dos blocos:

- “Qual parte roda no WebView e qual parte roda como código nativo?”
- “Por que passamos argumentos separados em vez de montar uma linha de shell?”
- “O que um `Result<T, E>` obriga quem chama a função a considerar?”
- “O que este teste protege? Qual falha ele ainda não detectaria?”
- “Se trocarmos `image-rs` por libvips, quais camadas deveriam permanecer iguais?”

Não é uma prova. A pergunta serve para detectar lacunas enquanto o contexto ainda está fresco.

## Controle pelo usuário

O usuário pode dizer a qualquer momento:

- “explique mais devagar”;
- “mostre outro exemplo”;
- “quero tentar escrever esta parte”;
- “pule o que já domino”;
- “pare depois deste teste”;
- “faça uma revisão antes de continuar”.

## Regra para o Codex

O Codex não deve confundir autonomia operacional com autorização pedagógica. Pode investigar, ler arquivos e executar verificações seguras para concluir o bloco atual, mas não inicia o próximo bloco didático sem confirmação.

