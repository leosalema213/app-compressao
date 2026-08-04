# Produto e escopo

## Visão

O MediaToolkit será um aplicativo desktop local que reúne operações comuns de mídia em uma interface consistente. O usuário seleciona arquivos ou uma URL, configura uma operação, acompanha seu progresso e recebe um resultado verificável sem depender de APIs pagas ou de um servidor próprio.

## Usuário inicial

- O próprio desenvolvedor e amigos.
- Usuários de Windows 10/11 x64.
- Pessoas que preferem interface gráfica a comandos de terminal.
- Uso pessoal, sem cobrança nesta etapa.

## Resultados do MVP

### Imagens

- Selecionar um ou vários arquivos.
- Arrastar e soltar arquivos válidos.
- Comprimir JPEG, PNG e WebP.
- Converter entre JPEG, PNG e WebP quando tecnicamente válido.
- Redimensionar preservando proporção por padrão.
- Escolher pasta de saída e política de colisão.
- Exibir tamanho anterior, tamanho final e economia obtida.

### Vídeo

- Ler metadados com FFprobe.
- Converter contêiner/formato entre combinações aprovadas.
- Comprimir usando presets compreensíveis, além de uma opção avançada limitada.
- Exibir progresso, tempo decorrido, estimativa e resultado.
- Cancelar sem deixar arquivo final aparentemente válido.

### Áudio

- Converter formatos aprovados.
- Extrair áudio de um vídeo.
- Escolher qualidade/preset.
- Reutilizar progresso, cancelamento e validações do módulo de vídeo.

### Downloader

- Consultar metadados de uma URL antes do download.
- Selecionar qualidade de vídeo ou somente áudio.
- Baixar item único ou playlist com confirmação explícita.
- Baixar miniatura e legendas quando disponíveis.
- Escolher pasta de saída.
- Exibir progresso e permitir cancelamento.

### Aplicativo

- Home com atalhos e atividades recentes.
- Navegação para Imagens, Vídeo, Áudio, Downloader e Configurações.
- Fila de tarefas com estados claros.
- Histórico local sem dados desnecessariamente sensíveis.
- Tema claro/escuro.
- Configurações persistentes.
- Notificações locais opcionais.

## Fora do MVP

- Editor de imagens ou vídeo.
- Corte, montagem, junção, filtros, efeitos e marca d'água.
- IA, OCR e processamento de PDF.
- Conta, nuvem, sincronização ou servidor próprio.
- DRM, quebra de proteção ou contorno de controles de acesso.
- Atualização automática do aplicativo ou dos binários auxiliares.
- macOS e Linux como plataformas entregues.

## Decisões de produto a confirmar no início de cada módulo

Detalhes como formatos finais, presets, preservação de metadados, convenção de nomes e comportamento de sobrescrita serão apresentados com exemplos no início da fase correspondente. Eles não devem ser decididos silenciosamente pelo Codex.

## Definição geral de pronto

Uma funcionalidade só está pronta quando:

- o caminho feliz funciona;
- entradas inválidas geram mensagem compreensível;
- cancelamento e falha não deixam estado incoerente;
- existem testes automatizados proporcionais ao risco;
- a validação manual documentada passou;
- o build de desenvolvimento continua funcionando;
- não surgiu permissão do Tauri mais ampla que a necessária;
- o usuário entendeu o conceito principal e aprovou o avanço.

