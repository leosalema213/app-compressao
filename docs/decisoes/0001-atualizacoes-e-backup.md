# Atualizações e backup

Em 2026-11-09, o usuário solicitou configurações com abas, backup e atualização integrada seguindo o Organon.

O aplicativo verifica o canal estável do GitHub ao abrir e permite verificar novamente nas configurações. A pessoa escolhe quando baixar e aplicar. A instalação aguarda o encerramento dos trabalhos ativos. O plugin oficial do Tauri verifica a assinatura do instalador antes da instalação. O manifesto `latest.json`, o instalador e a assinatura devem acompanhar cada release final.

A chave privada fica no perfil local do responsável pela publicação, fora do projeto. O build recebe o caminho por variável de ambiente. A chave pública fica na configuração do app. Perder a chave privada impede publicar atualizações compatíveis com as instalações existentes.

O backup JSON versionado contém tema, cor e pasta padrão. Os arquivos de mídia ficam nas pastas escolhidas pelo usuário. A restauração valida o formato e avisa se a pasta salva não existe no computador atual.

A versão 0.1.0 precisa receber uma instalação manual da nova versão para passar a usar o atualizador.
