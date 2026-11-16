# Integração legítima com Minecraft Bedrock no Android

## Objetivo

O Book Edition é um cliente independente. A presença do Minecraft oficial pode ser detectada para orientar a integração, mas o Book Edition não copia APKs, não extrai dados privados do pacote oficial, não solicita credenciais Microsoft e não usa arquivos obtidos de forma clandestina.

## Detecção do aplicativo

A primeira camada usa Android PackageManager para procurar o pacote Android:

- Pacote esperado: `com.mojang.minecraftpe`.
- O manifest do launcher declara esse pacote em `<queries>` para respeitar a visibilidade de pacotes do Android 11+.
- `MinecraftInstallationProbe.inspect(context)` retorna se o pacote está instalado, sua versão e os fingerprints SHA-256 dos certificados de assinatura apresentados pelo Android.

**Limite de segurança/licença:** encontrar o pacote não prova que o usuário comprou o jogo. O fingerprint é exposto para diagnóstico, mas ainda não existe uma allow-list de certificados oficiais verificada independentemente; portanto, o projeto não declara que o APK é autêntico apenas com base no nome do pacote ou no fingerprint. O app não deve bloquear ou liberar funcionalidades pagas com base nessa detecção.

## Acesso aos recursos

A integração de conteúdo deve ser explícita e controlada pelo usuário:

1. O launcher detecta o Minecraft e mostra o estado como "instalado" ou "não encontrado".
2. O usuário escolhe um Resource Pack ou arquivo exportado pelo seletor de documentos Android (Storage Access Framework).
3. O Book Edition lê apenas o URI concedido pelo usuário.
4. O importador valida manifest, caminhos, tamanho e formato antes de processar imagens e JSON.
5. O pack importado é copiado para o diretório próprio do Book Edition em `Android/media/dev.bookedition.launcher/book-edition/resource-packs`, quando o Android permitir; o app mantém uma cópia privada de trabalho para processamento.

O Android isola os dados privados de cada aplicativo. O Book Edition não deve tentar ler `/data/data/com.mojang.minecraftpe` nem contornar as restrições de `Android/data`. Se um pack estiver acessível somente dentro do armazenamento privado do Minecraft, o usuário deve exportá-lo por um fluxo permitido pelo próprio Minecraft ou selecionar uma cópia que já tenha direito de usar.

A detecção da instalação é opcional: o usuário também pode importar packs próprios ou de terceiros compatíveis sem ter o Minecraft instalado. O Book Edition não redistribui conteúdo proprietário do jogo.

## Bedrock Samples 1.26.60.5 fornecido para análise

O arquivo `bedrock-samples-v1.26.60.5-full.zip` enviado para esta conversa foi inspecionado sem incorporar seus assets ao APK ou ao repositório.

Estrutura observada:
- `resource_pack/manifest.json`: `format_version: 2`, módulo do tipo `resources` e `min_engine_version: [1, 26, 60]`.
- `resource_pack/textures/terrain_texture.json`: 1.367 chaves em `texture_data`; usa comentários JSONC, caminhos diretos, arrays e objetos com `path` e metadados como `overlay_color`.
- `resource_pack/textures/item_texture.json`: 504 entradas em `texture_data`.
- O ZIP contém 8.232 arquivos de imagem PNG/JPG/TGA sob `resource_pack/textures/`.

O próprio `LICENSE.md` diz que os arquivos são copyright Mojang AB, "All rights reserved", e que estão sujeitos ao Minecraft EULA. Portanto, este ZIP é referência de desenvolvimento/teste local, não uma licença para redistribuir suas texturas. Não adicionar as imagens desse arquivo ao Git, APK ou releases públicas.

## Próximas etapas técnicas

- Exibir o estado da detecção na tela de integração, sem afirmar propriedade/licença.
- Adicionar fingerprint pinning somente após obter e verificar certificados de distribuição oficial por fonte confiável e documentada.
- Fazer importação por SAF com persistência do URI concedido.
- Expandir o parser JSONC para o conjunto de recursos Bedrock: aliases, arrays/variações, overlay colors, mip levels, padding e múltiplas páginas.
- Carregar o atlas no Book64/GLES, gerar UVs por face e validar com packs de teste autorizados.
- Implementar depois flipbooks, tintas de bioma, materiais e transparências.

## Referências

- https://github.com/Mojang/bedrock-samples
- https://github.com/Mojang/bedrock-samples/blob/main/LICENSE.md
- https://www.minecraft.net/en-us/eula
- https://developer.android.com/training/package-visibility
- https://developer.android.com/training/data-storage/shared/documents-files
