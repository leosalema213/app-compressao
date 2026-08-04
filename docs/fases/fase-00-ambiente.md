# Fase 00 — Ambiente e linha de base

## Resultado

Ao final, saberemos exatamente quais ferramentas existem no Windows, quais faltam e por que cada uma é necessária. O workspace terá controle de versão e um diagnóstico reproduzível, mas ainda não terá código do aplicativo.

## Conceitos novos

- PowerShell e diretório atual.
- Executável, argumento, opção e código de saída.
- PATH.
- Toolchain Rust/MSVC.
- Diferença entre dependência de desenvolvimento e dependência incluída no aplicativo.

## Bloco 00.1 — Inventário somente leitura

- Explicar cada comando antes de executá-lo.
- Verificar sistema/arquitetura e versões de `git`, `node`, `npm`, `rustup`, `rustc` e `cargo`.
- Verificar o target padrão do Rust.
- Verificar WebView2 e Microsoft C++ Build Tools.
- Verificar se FFmpeg, FFprobe e yt-dlp já existem, apenas como informação.
- Registrar resultados em `docs/ambiente-local.md`; não registrar dados sensíveis.

Teste/validação: cada ferramenta encontrada responde com versão e exit code 0; ausências ficam marcadas, não “corrigidas” ainda.

Checkpoint: explicar com suas palavras o que é PATH e por que abrir outro terminal às vezes é necessário após uma instalação.

## Bloco 00.2 — Instalações necessárias

- Comparar o inventário com os pré-requisitos oficiais do Tauri.
- Propor uma instalação por vez.
- Obter confirmação antes de instalar ou alterar a máquina.
- Preferir Node LTS, Rust stable MSVC, WebView2 e workload “Desktop development with C++”.
- Reiniciar o terminal e repetir somente as verificações afetadas.

Teste/validação: `rustc -V`, `cargo -V`, `node -v`, `npm -v` e compilador/linker necessários disponíveis.

Checkpoint: distinguir Node/npm de Rust/Cargo e dizer qual lado do aplicativo cada conjunto constrói.

## Bloco 00.3 — Git e diagnóstico

- Confirmar nome/e-mail do Git sem expor informações além do necessário.
- Inicializar repositório na raiz se o usuário aprovar.
- Criar `.gitignore` inicial para Node, Rust, builds, logs, temporários e configurações locais.
- Criar script PowerShell somente leitura para repetir o diagnóstico.
- Revisar o primeiro diff e, se solicitado, preparar o primeiro commit.

Teste/validação: repositório reconhecido, script retorna resultado compreensível e nenhum arquivo pessoal é incluído por engano.

Checkpoint: explicar a diferença entre working tree, arquivo rastreado e commit.

## Critérios de aceite

- [ ] Arquitetura Windows confirmada.
- [ ] Node LTS e npm funcionam.
- [ ] Rust stable com target MSVC funciona.
- [ ] Cargo funciona.
- [ ] WebView2 está disponível.
- [ ] C++ Build Tools necessários estão disponíveis.
- [ ] Git funciona e o repositório está inicializado.
- [ ] Diagnóstico local pode ser repetido.
- [ ] O usuário aprovou avançar.

## Não fazer nesta fase

- Não criar o projeto Tauri.
- Não instalar FFmpeg/yt-dlp sem necessidade imediata.
- Não configurar CI, instalador ou múltiplas plataformas.

