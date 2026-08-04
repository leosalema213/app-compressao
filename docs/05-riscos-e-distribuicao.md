# Riscos, dependências e distribuição

## Registro inicial de riscos

| Risco | Impacto | Tratamento planejado |
|---|---|---|
| `image-rs` não atingir qualidade/desempenho | Médio/alto | Interface substituível, corpus e benchmark antes da aprovação |
| FFmpeg variar por build/codecs | Alto | Diagnóstico de versão/encoders, presets compatíveis e binário fixado na distribuição |
| yt-dlp quebrar com mudanças de sites | Alto | Isolar adaptador, mostrar versão, atualização manual e erros claros |
| yt-dlp exigir runtime JS adicional | Médio/alto | Preflight e avaliação de `deno.exe` como sidecar no Downloader |
| Command injection por nomes/URLs | Alto | Sem shell textual; argumentos separados, validação e testes maliciosos |
| Cancelamento deixar arquivos parciais | Alto | Sufixo temporário, rastreamento de processo e limpeza testada |
| Processos consumirem CPU/memória demais | Médio | Concorrência inicial 1, limites e feedback de fila |
| Histórico expor caminhos ou URLs | Médio | Persistência mínima e ação para limpar histórico |
| Instalador funcionar apenas na máquina de desenvolvimento | Alto | Teste em VM/usuário limpo antes de compartilhar |
| Plano ficar desatualizado | Médio | Atualizar `progresso.md` e decisões a cada bloco |

## FFmpeg

FFmpeg é majoritariamente LGPL, mas recursos opcionais podem tornar um build GPL. Além disso, codecs podem envolver patentes conforme a jurisdição. Para uso pessoal o risco prático é menor, mas compartilhar um instalador já constitui distribuição e exige cuidado.

Antes de incluir FFmpeg no instalador:

- identificar origem, versão e configuração do build;
- verificar a licença efetiva do binário e bibliotecas incluídas;
- preservar arquivos de licença e avisos;
- disponibilizar ou apontar o código-fonte correspondente quando exigido;
- documentar o uso do FFmpeg no aplicativo;
- não usar build marcado como `nonfree` para distribuição;
- revisar a escolha de codecs se o projeto se tornar comercial.

Este plano é orientação técnica, não aconselhamento jurídico.

## yt-dlp e sites de mídia

- O aplicativo deve ser usado apenas para conteúdo que o usuário tenha direito de baixar.
- Termos dos sites podem restringir downloads mesmo quando tecnicamente possíveis.
- O MVP não trabalha com DRM, paywalls, autenticação/cookies ou evasão de controles.
- Playlists exibem quantidade estimada antes de iniciar para evitar downloads acidentais.
- Mudanças externas podem quebrar extratores; isso será comunicado como dependência externa, não mascarado como erro genérico.
- As licenças do executável e de suas dependências serão incluídas quando houver distribuição.

## Atualizações de ferramentas

Durante o desenvolvimento, atualizar dependências é uma ação consciente e testada. Na distribuição:

- versões dos sidecars ficam fixadas;
- hashes verificam integridade dos arquivos;
- a aplicação informa versões no diagnóstico;
- atualização manual é preferida no MVP;
- autoatualização de yt-dlp/FFmpeg não será habilitada silenciosamente.

## Segredos e privacidade

- O projeto não precisa de chaves de API.
- Cookies do navegador ficam fora do MVP.
- Logs não devem incluir conteúdo de arquivos, credenciais ou linha de comando com dados sensíveis.
- URLs e caminhos completos só entram no histórico quando necessário e podem ser apagados.
- Relatórios de erro permanecem locais até o usuário decidir compartilhá-los.

## Distribuição Windows

O candidato inicial é instalador NSIS por ser familiar para usuários do Windows; MSI será comparado na Fase 09. A entrega só será considerada válida depois de instalação em um ambiente que não possua Node, Rust, Cargo, FFmpeg ou yt-dlp previamente disponíveis.

O pacote deve responder claramente:

- quais binários estão incluídos;
- quais versões e licenças se aplicam;
- onde ficam configurações, histórico e logs;
- como desinstalar e remover dados locais;
- quais formatos/serviços são suportados e quais são limitações conhecidas.

