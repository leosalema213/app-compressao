# Instruções do projeto para o Codex

## Fonte de verdade

- Leia `docs/README.md`, `docs/progresso.md` e o arquivo da fase atual antes de alterar código.
- O planejamento versionável em `docs/` complementa o DOCX original.
- Não altere stack, escopo ou decisão arquitetural sem registrar a decisão e obter confirmação do usuário.

## Método didático obrigatório

- Trabalhe em apenas um bloco didático da fase por vez.
- Antes de editar, explique em português o objetivo, os conceitos novos e os arquivos que serão afetados.
- Faça mudanças incrementais e localizadas; preserve a estrutura existente e mantenha o diff pequeno.
- Depois de editar, explique o que mudou em linguagem adequada a um frontend júnior/pleno e iniciante em Rust/terminal.
- Execute os testes proporcionais ao bloco e mostre o resultado de forma resumida.
- Termine cada bloco com uma pergunta de verificação de entendimento.
- Não avance ao próximo bloco ou fase sem confirmação explícita do usuário.
- Quando o usuário autorizar execução contínua, trate essa autorização como permissão para seguir pelos blocos e documente os checkpoints. Agrupe os resultados e só interrompa por uma decisão, permissão ou ação que dependa dele.

## Qualidade e segurança

- Nunca monte comandos de sistema concatenando texto recebido da interface. Use executável e argumentos separados.
- O frontend não executa FFmpeg, yt-dlp ou outros binários diretamente; ele chama comandos Rust com dados tipados.
- Valide extensões, formato real, caminhos de entrada/saída e colisões antes de escrever arquivos.
- Testes devem acompanhar comportamento novo ou correção de bug sempre que viável.
- O agente executa os testes e as verificações que estiverem ao seu alcance. Solicite intervenção do usuário apenas quando uma etapa depender dele.
- Antes de concluir um bloco, rode formatação, análise estática, testes relevantes e build quando o risco justificar.
- Não declare uma fase concluída enquanto seus critérios de aceite não estiverem satisfeitos.

## Estado do trabalho

- Atualize `docs/progresso.md` ao concluir cada bloco.
- Registre decisões importantes na seção de decisões do progresso ou em um novo ADR dentro de `docs/decisoes/`.
- Não marque itens manualmente sem evidência do teste ou validação correspondente.
- Preserve arquivos e mudanças do usuário não relacionados à tarefa.

## Retenção de releases locais

- Entregue previews locais como executável sem instalação, com versão única e atalho `Testar LpsToolkit.lnk`, usando `scripts/gerar-preview.ps1`. Reserve o instalador para a versão publicada.
- Para versões finais com atualização integrada, use `scripts/gerar-release.ps1` e publique juntos o instalador, `.sig`, `.sha256` e `latest.json`. A chave privada fica fora do Git. O canal de atualização recebe apenas versões finais.
- Após gerar e verificar com sucesso uma compilação local, mantenha no máximo duas na pasta de releases: a recém-gerada e a imediatamente anterior, se houver.
- Antes de remover compilações antigas, verifique os caminhos absolutos resolvidos e limite a limpeza aos artefatos gerados na pasta local de releases. Preserve links, dados pessoais e o histórico publicado, incluindo tags e Releases remotas.

## Escopo inicial

- Windows 10/11 x64 primeiro; portabilidade é uma restrição arquitetural, não uma entrega do MVP.
- Aplicativo local, sem APIs pagas e sem servidor obrigatório.
- Não incluir edição de mídia, IA, OCR, PDF, marca d'água, cortes, junções ou filtros no MVP.
- Não implementar suporte a DRM ou mecanismos de evasão de restrições de sites.

