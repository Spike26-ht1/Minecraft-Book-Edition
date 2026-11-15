# Build de testes Android

## Build atual
- Aplicativo: Minecraft Book Edition
- Versão: 0.1.0-dev.2
- Tipo: APK debug (não assinado para distribuição)
- Módulo: `:launcher:app`
- SDK alvo: Android 35
- SDK mínimo: Android 8.0 (API 26)
- Java: 17
- Android Gradle Plugin: 8.9.2
- Gradle: 8.11.1

## Gerar o APK pelo GitHub Actions
1. Abra a aba **Actions** do repositório.
2. Selecione o workflow **Android debug build**.
3. Abra a execução mais recente da branch `launcher/foundation`.
4. Confirme que o job terminou com sucesso.
5. Baixe o artefato `Minecraft-Book-Edition-0.1.0-dev.2-debug`.

O arquivo gerado é `app-debug.apk`. O artefato expira após 14 dias.

## Gerar localmente
Abra a raiz do projeto no Android Studio, com Java 17 e Android SDK 35 instalados. Depois execute:

```bash
gradle --no-daemon :launcher:app:testDebugUnitTest :launcher:app:assembleDebug
```

Saída do APK:

```
launcher/app/build/outputs/apk/debug/app-debug.apk
```

## Limites desta build
Esta versão valida a estrutura do launcher, a navegação, as preferências locais, o armazenamento privado e os logs. Ela **não executa Minecraft Bedrock**. O botão Jogar está intencionalmente desativado até a integração do cliente existir.

A instalação em um aparelho Android real ainda deve ser verificada após o APK ser gerado.
