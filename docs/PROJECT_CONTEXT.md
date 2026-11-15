# Minecraft Book Edition — contexto do projeto

## Objetivo
Criar um projeto independente chamado Minecraft Book Edition, aberto, documentado e fácil de entender para desenvolvedores e modders. O foco inicial é estabelecer uma base de launcher Android e, em seguida, avançar na integração/execução do Minecraft Bedrock.

## Prioridades
1. Evoluir o cliente/jogo independente e o motor Book64, mantendo o launcher como entrada única.
2. Validar cada build Android ARM64 em CI e em dispositivo físico.
3. Definir e implementar compatibilidade Bedrock por etapas, sem copiar o cliente oficial.
4. Deixar APIs de mods e Book Loader para etapas posteriores.

## Plataforma e tecnologia
- Android ARM64 é a primeira plataforma-alvo.
- Windows também está no escopo futuro.
- Launcher Android: Kotlin, Jetpack Compose e Material Design 3.
- Componentes de baixo nível do jogo: Rust.
- Plano gráfico inicial: OpenGL / OpenGL ES.
- Vulkan poderá ser adicionado futuramente como opção.
- Preparar integração Rust sem bloquear a interface Android inicial.

## Arquitetura planejada
- launcher/: aplicativo, interface, configurações, gestão de versões e logs.
- minecraft/: componentes e dados do cliente, recursos, bibliotecas e runtime.
- crates/: componentes Rust reutilizáveis.
- android/launcher/: integração Android do launcher.
- android/media/: diretório reservado para mídias do projeto.
- docs/: documentação de arquitetura, decisões e desenvolvimento.
- tests/: testes.

## Dados privados no Android
Os dados de execução ficam no armazenamento privado do aplicativo, separados do código-fonte:
- launcher/settings/
- launcher/logs/
- launcher/versions/
- minecraft/assets/
- minecraft/libraries/
- minecraft/resources/
- minecraft/worlds/
- minecraft/cache/

O launcher centraliza os logs, registra inicialização e eventos selecionados e permite exportação manual de relatórios. Não há upload automático de logs.

## Referência arquitetural: Cinnabar
O Cinnabar foi estudado como referência de organização e separação de componentes. Suas ideias podem inspirar a arquitetura, mas o Book Edition terá implementação própria. Não copiar código do Cinnabar. Verificar licenças de dependências individualmente.

## Decisão ainda pendente
Definir se o cliente Bedrock será um cliente independente implementado para compatibilidade com Bedrock ou uma integração que carrega o cliente oficial instalado pelo usuário. Essa escolha afeta a arquitetura do runtime e não deve ser presumida.

## Regras de desenvolvimento
- Não reutilizar o código antigo do projeto como base do novo produto.
- Preservar o histórico do GitHub; não substituir main sem autorização explícita.
- Não incluir nem redistribuir assets proprietários do Minecraft.
- Não declarar builds ou testes como concluídos sem execução real.
- Manter documentação clara e incremental.

## Launcher — histórico da fundação 0.1.0-dev.2
A build de testes do launcher inclui:
- Navegação entre Início, Versões, Ajustes e Logs.
- Tema claro/escuro com preferência persistente.
- Opção para registrar eventos detalhados localmente.
- Criação de diretórios privados para configurações, versões, recursos e mundos.
- Visualização e exportação manual dos logs.
- Botão Jogar desativado enquanto o cliente Bedrock não estiver integrado.

A build não inclui cliente Bedrock, autenticação, instalação de versões de jogo ou execução de mundos. O APK de depuração é produzido pelo workflow do GitHub Actions e não é uma versão de distribuição final.


## Atualização — integração launcher + jogo

- O launcher é a entrada única do aplicativo Android.
- O módulo `game/android/app` é uma Android Library chamada `:game-runtime`, incluída no projeto Gradle raiz.
- O APK do launcher incorpora a Activity do jogo e a biblioteca Rust/JNI.
- O botão Jogar do launcher inicia `dev.bookedition.game.MainActivity` por Intent explícito.
- O menu do jogo aparece antes da cena: Jogar, Configurações e Sair.
- A cena voxel de teste só é criada depois da seleção de Jogar no menu do jogo.
- A cena tem HUD mobile com câmera por toque, direcional, movimento vertical e botão de menu.
- O painel F3 inicial mede FPS/frame time, resolução, heap Android e telemetria nativa de câmera/viewport/vértices.
- O menu de teste alterna F3 e controles e permite ajustar a sensibilidade.
- Os controles atuais movimentam uma câmera livre; física, colisão, gravidade e pulo com chão ainda não existem.
- A integração de texturas Bedrock Samples depende de leitor de Resource Pack e assets fornecidos pelo usuário; os arquivos proprietários da Mojang não serão copiados para o repositório ou APK.
- Branch de integração: `launcher/game-integration`.
- As branches `launcher/foundation` e `game/clean-foundation` permanecem preservadas.
- A compilação CI deve gerar um único APK debug com launcher + jogo, ABI arm64-v8a.
- Ainda não declarar build validada até CI passar e instalação física ser testada.
