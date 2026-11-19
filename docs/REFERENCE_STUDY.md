# Estudo de referências e decisões de arquitetura

Data da auditoria: 2026-11-18.

Este documento registra o que foi observado nos projetos de referência e, principalmente, o que o Book Edition decidiu implementar por conta própria. Referências arquiteturais não autorizam copiar código, assets ou dados proprietários.

## Fontes consultadas

### Horizon — reimplementação de Java Edition em Rust

- Repositório: https://github.com/horizon-rust/horizon
- Arquitetura: https://github.com/horizon-rust/horizon/blob/main/docs/architecture.md
- Cliente nativo: https://github.com/horizon-rust/horizon/blob/main/docs/native-client.md
- Processo de paridade: https://github.com/horizon-rust/horizon/blob/main/docs/parity-workflow.md
- Geração de mundo: https://github.com/horizon-rust/horizon/blob/main/docs/world-generation.md

Observações:

- A arquitetura separa gameplay lógico, código reutilizável do cliente, entrada, apresentação/renderização e mundo.
- O single-player usa uma simulação de servidor integrada; a apresentação do cliente consome o estado do jogo.
- A simulação de movimento e gameplay é executada em 20 ticks por segundo, independentemente da taxa de quadros.
- O documento de arquitetura mapeia áreas de gameplay para arquivos e testes específicos.
- A paridade é tratada por comportamento: cada método/funcionalidade relevante tem um estado de implementação e harnesses que comparam resultados com o jogo de referência.
- Salvamento, carregamento, geração, streaming, colisões, inventário e rede são sistemas separados; não são etapas de uma única demonstração gráfica.

Licença verificada no HEAD consultado: o README atual informa GNU LGPL versão 3 ou posterior para o trabalho novo; o código anterior à mudança permanece disponível em MIT em `LICENSES/MIT.txt`. Não copiar código do Horizon para o Book Edition.

### Cinnabar — reimplementação de Bedrock em Rust

- Repositório: https://github.com/cinnabar-mc/cinnabar
- README: https://github.com/cinnabar-mc/cinnabar/blob/dev/README.md
- Android experimental: https://github.com/cinnabar-mc/cinnabar/blob/dev/docs/android.md
- Workspace: https://github.com/cinnabar-mc/cinnabar/blob/dev/Cargo.toml

Observações:

- O cliente Rust é separado do núcleo de autenticação e transporte Go; a comunicação entre eles usa um socket local.
- O cliente Rust possui responsabilidades explícitas para mundo, simulação, sessão, protocolo, inventário, interação, apresentação, renderização, pipeline de chunks e compilação de recursos.
- O código do mundo mantém armazenamento paletizado e dados de subchunks separados da malha renderizada.
- O pipeline de chunks tem limites de trabalho e publicação; geração, decodificação, luz, meshing e apresentação não devem bloquear indiscriminadamente o frame.
- O repositório possui testes de aceitação e observações de comportamento do runtime, além de testes unitários.
- O Android é descrito como experimental, para ARM64 e Vulkan. O APK prepara recursos localmente e não distribui os arquivos de jogo.

Licença verificada no repositório principal consultado: PolyForm Noncommercial 1.0.0. Dependências e subprojetos podem ter licenças próprias; qualquer futura dependência precisa de revisão individual. O Book Edition não copiará nem vinculará código do Cinnabar.

### LeviLauncher Android — launcher para Bedrock oficial

- Repositório: https://github.com/LiteLDev/LeviLauncher-Android
- README: https://github.com/LiteLDev/LeviLauncher-Android/blob/main/README.md

Observações:

- O launcher separa preparação do runtime, gerenciamento de versões, sessão de lançamento, mundos, pacotes e autenticação da interface principal.
- O fluxo de lançamento valida a versão selecionada, prepara arquivos e dependências, registra etapas e só então entrega o controle ao runtime.
- O gerenciamento de mundos é um subsistema independente, com abertura, mudanças pendentes, revisão e salvamento.
- O projeto Android usa Kotlin/Java e código nativo auxiliar.

Licença do repositório: Apache-2.0. Vamos usá-lo como referência de organização Android e gestão de conteúdo, não como base para executar o cliente oficial dentro do Book Edition.

## Decisões do Book Edition

### 1. O produto é um cliente independente

Book Edition será uma reimplementação própria e progressiva do cliente Bedrock em Rust. O APK oficial, suas bibliotecas nativas e o runtime oficial não serão carregados como motor do Book Edition. O launcher pode detectar uma instalação legítima e oferecer importação de arquivos que o usuário possa acessar e esteja autorizado a usar, mas isso não transforma o cliente oficial no runtime do projeto.

### 2. Fronteiras de responsabilidade

- `book-core`: tipos fundamentais, coordenadas e contratos estáveis.
- `book-world`: armazenamento de blocos, colunas/seções, geração própria e futuras leituras/escritas de mundo.
- `book-game`: sessão de jogo, relógio fixo, estado do jogador, física e regras de simulação.
- `book-meshing`: extração de geometria a partir de snapshots do mundo.
- `book64-gles`: apresentação OpenGL ES e recursos de GPU; não decide física, input ou regras de jogo.
- `book-android`: composição do runtime e fronteira JNI.
- Launcher Kotlin: navegação, arquivos, configurações, logs e preparação da aplicação, sem implementar regras do jogo.

Próximos limites a criar conforme houver implementação real: `book-save`, `book-assets`, `book-streaming`, `book-protocol` e uma camada de cliente/apresentação.

### 3. Relógio de simulação e renderização

A simulação single-player usa um passo fixo de 20 Hz. O renderizador pode apresentar a 60/90/120 Hz sem alterar a quantidade de ticks de gameplay. O tempo acumulado por frame é limitado para evitar que uma pausa longa gere uma sequência ilimitada de ticks atrasados.

### 4. Mundo, simulação e malha são dados diferentes

O mundo mantém blocos e seções. A simulação lê e modifica esse estado. O mesher cria um snapshot de geometria. Book64 envia esse snapshot para a GPU. Alterações de blocos futuramente invalidarão e reconstruirão apenas as malhas afetadas; o renderer não será a fonte de verdade do mundo.

### 5. Compatibilidade deve ser demonstrada

A seed e a geração atuais são algoritmos próprios do Book Edition, não equivalentes às seeds Bedrock. A compatibilidade Bedrock 26.60 será implementada por formatos e comportamentos versionados, com fixtures legítimos e testes diferenciais. Não marcar compatibilidade só porque o projeto desenha chunks.

### 6. Assets e licenças

O projeto não incorpora assets proprietários do Minecraft no repositório ou APK público. O importador trabalhará com arquivos selecionados pelo usuário e acessíveis pelas APIs normais do Android. Cada dependência externa terá sua licença revisada antes de ser adicionada.

## Primeiro marco de jogo após a auditoria

A dev.10 inaugura a separação de sessão e simulação: mundo carregado, jogador com movimento horizontal, gravidade, salto e colisão básica, atualização fixa de 20 TPS e renderer que consome um snapshot. Isso ainda não é paridade Bedrock e ainda não inclui salvamento, interação com blocos, streaming ou atlas na GPU.

A próxima entrega deverá acrescentar interação real e persistência de mundo, não aumentar a cena de demonstração sem necessidade.


## Atualização de estratégia e auditoria de worldgen — 2026-11-19

A análise do código atual do Horizon confirma que `crates/horizon-world/src/worldgen/` é uma implementação orientada aos formatos, registries, parâmetros de noise e ordem de geração da Java Edition 26.3. O próprio módulo descreve que a geração completa do terreno ainda está pendente. O crate depende de `horizon-core`, `horizon-assets`, `horizon-threads` e bibliotecas adicionais. Portanto, não é um gerador Bedrock que possa ser conectado diretamente ao Book World.

Decisão: reutilizar métodos de engenharia — etapas de geração isoladas, workers, dados versionados, testes diferenciais e armazenamento independente —, mas implementar separadamente os algoritmos e dados específicos de Bedrock 26.60. Qualquer reutilização de código LGPL exige revisão da licença aplicável, preservação de avisos e cumprimento de obrigações; esta entrega não copia código Horizon ou Cinnabar.

A auditoria do Cinnabar `dev` mostra cliente Rust separado de núcleo Go de transporte/autenticação; o README informa PolyForm Noncommercial 1.0.0 e descreve BDS como backend para mundos locais. O projeto permanece referência arquitetural e não dependência incorporada.

### dev.12 — primeiro ciclo de interação e persistência local

- Adicionado formato local `BOOKWLD1`, versionado e limitado, que salva os blocos das colunas carregadas, seed e versão do gerador.
- O runtime Android reabre o mundo salvo pela seed selecionada e grava em arquivo temporário com sincronização e renomeação.
- Adicionado raycast de primeira pessoa, mineração e colocação inicial de bloco, com atualização da malha no thread OpenGL.
- A colocação ainda usa Dirt fixa e não representa inventário/hotbar. Posição do jogador e entidades não são persistidas.
- O formato `BOOKWLD1` é exclusivo do Book Edition e não é formato de mundo Bedrock.
