# Minecraft Book Edition — contexto do projeto

## Objetivo
Criar um projeto independente chamado Minecraft Book Edition, aberto, documentado e fácil de entender para desenvolvedores e modders. O foco inicial é uma integração funcional com Minecraft Bedrock.

## Prioridades
1. Desenvolver primeiro o launcher Android.
2. Depois avançar na integração/execução do cliente Bedrock.
3. Deixar Book64, APIs de mods e Book Loader para etapas posteriores.
4. Mods não são prioridade nesta fase.

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
- android/médias/: diretório reservado para mídias do projeto.
- docs/: documentação de arquitetura, decisões e desenvolvimento.
- tests/: testes.

## Dados privados no Android
Os dados de execução devem ficar no armazenamento privado do aplicativo, separados do código-fonte:
- launcher/settings/
- launcher/logs/
- launcher/versions/
- minecraft/assets/
- minecraft/libraries/
- minecraft/resources/
- minecraft/worlds/
- minecraft/cache/

O launcher deverá centralizar os logs do launcher e do cliente, registrar inicialização, versão, dispositivo, falhas e encerramento, e permitir exportação manual de relatórios. Não haverá upload automático de logs.

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

## Launcher — primeira entrega
A primeira base usa Kotlin + Jetpack Compose + Material 3. A tela inicial é um esqueleto de interface; o botão Jogar ainda não inicia o cliente. Próximas etapas: navegação, configurações, gestão de versões e sistema de logs.
