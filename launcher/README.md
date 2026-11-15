# Launcher

Aplicativo Android do Minecraft Book Edition, desenvolvido com Kotlin, Jetpack Compose e Material Design 3.

## Versão de testes atual

**0.1.0-dev.2** — primeira build com navegação e ferramentas locais básicas.

### Incluído
- Tela inicial com estado real da integração Bedrock.
- Navegação entre Início, Versões, Ajustes e Logs.
- Preferência persistente de tema claro/escuro.
- Configuração de registros detalhados.
- Criação de diretórios no armazenamento privado do aplicativo.
- Visualização e exportação manual dos logs.

### Ainda não incluído
- Cliente Bedrock ou runtime de jogo.
- Login/autenticação.
- Download/instalação de versões do Minecraft.
- Inicialização de mundos.

O botão Jogar permanece desativado para não sugerir que uma integração inexistente já funciona.

## Compilar localmente

Abra a raiz do repositório no Android Studio e aguarde a sincronização do Gradle. O projeto usa Android Gradle Plugin 8.9.2, Gradle 8.11.1 e Java 17.

Para gerar o APK debug em um ambiente com Gradle e Android SDK configurados:

```bash
gradle --no-daemon :launcher:app:assembleDebug
```

Para executar os testes unitários:

```bash
gradle --no-daemon :launcher:app:testDebugUnitTest
```

O workflow **Android debug build** executa os testes e publica o APK como artefato de build por 14 dias.
