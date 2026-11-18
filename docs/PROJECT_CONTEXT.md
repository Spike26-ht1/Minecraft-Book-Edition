# Minecraft Book Edition — contexto do projeto

## Objetivo

Minecraft Book Edition é uma reimplementação independente do cliente Minecraft Bedrock, escrita principalmente em Rust. A meta não é apenas criar um launcher para abrir o jogo oficial. O launcher Android é o host e distribuidor do runtime próprio do Book Edition.

## Direção aprovada

- Cliente independente em Rust, com Bedrock 26.60 como alvo de compatibilidade.
- Kotlin + Jetpack Compose para o launcher Android.
- Um único APK contendo launcher e runtime próprio do jogo.
- Book64 como fronteira de renderização; OpenGL ES primeiro, Vulkan futuro.
- Android ARM64 primeiro; Windows futuro.
- Referências: Horizon para engenharia de paridade e organização de sistemas; Cinnabar para arquitetura de cliente Bedrock em Rust; LeviLauncher para organização do launcher Android e preparação de conteúdo.
- Estudar referências não significa copiar código. Licenças são registradas em [REFERENCE_STUDY.md](REFERENCE_STUDY.md).

## Arquitetura atual

- `book-core`: coordenadas e tipos compartilhados.
- `book-world`: blocos, seções, colunas carregadas, consultas de blocos e geração própria.
- `book-game`: sessão single-player, relógio fixo de 20 TPS, input, jogador, gravidade, salto e colisão básica.
- `book-meshing`: extração de geometria a partir de snapshots do mundo.
- `book64-gles`: renderer OpenGL ES que consome snapshots, sem controlar a simulação.
- `book-android`: composição JNI da sessão e renderer.
- `launcher/app`: entrada Android, interface, configurações, arquivos e logs.

A simulação deve permanecer independente da taxa de quadros. O mundo é a fonte de verdade; a malha é uma representação derivada; o renderer só apresenta snapshots.

## Estado do primeiro marco jogável

Implementado no código, aguardando validação CI e teste físico da versão correspondente:
- Mundo carregado com região inicial de 5 × 5 colunas e quatro seções verticais.
- Consulta de blocos por coordenadas globais, incluindo coordenadas negativas.
- Sessão single-player em Rust.
- Simulação fixa de 20 ticks por segundo.
- Movimento horizontal, gravidade, salto e colisão básica contra blocos sólidos.
- Book64 recebe posição/câmera de um snapshot da sessão.
- Controles Android enviam movimento e pedido de salto para a simulação.
- F3 exibe posição do jogador, estado no chão, seed, versão do gerador, colunas carregadas e tick atual.

Limites ainda existentes:
- O gerador é algoritmo próprio Book Edition v1, não é o gerador Bedrock.
- A região é limitada; não existe streaming.
- O mesh é criado na inicialização; edição do mundo ainda não invalida/reconstrói malhas.
- Não existe raycast, quebrar/colocar blocos, inventário funcional, salvamento/reabertura, entidades, áudio, protocolo, autenticação ou multiplayer.
- O importador cria atlas local, mas Book64 ainda não o envia à GPU.
- Não declarar compatibilidade Bedrock até haver leitores e testes por versão.

## Dados e segurança no Android

O runtime e os mundos usam armazenamento privado do aplicativo por padrão. O launcher registra logs localmente e permite exportação manual. Não há upload automático de logs. A detecção de `com.mojang.minecraftpe` é apenas diagnóstica; não prova licença e não dá acesso ao sandbox privado de outro app. Recursos externos só devem ser lidos por mecanismos Android autorizados, como o seletor de documentos, e não devem ser redistribuídos.

## Regras de desenvolvimento

- Trabalhar na branch `launcher/game-integration`; não alterar `main` nem fazer merge sem autorização explícita.
- Não reutilizar o código do protótipo abandonado.
- Não copiar código do Horizon ou Cinnabar.
- Revisar a licença de cada dependência.
- Não incluir assets, binários ou credenciais proprietários da Mojang/Microsoft.
- Validar com testes Rust, formatação, build Android ARM64 e teste físico.
- Não declarar build ou sistema como pronto sem evidência real.
- Cada incremento deve entregar um sistema permanente e testável do jogo, não apenas outra cena de demonstração.
