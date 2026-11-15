# Assets e texturas

## Referência Bedrock Samples

O repositório [Mojang/bedrock-samples](https://github.com/Mojang/bedrock-samples) é a referência oficial para a estrutura de Resource Packs e exemplos de conteúdo Bedrock. A pasta `resource_pack/` contém definições, manifestos e referências de texturas.

## Regra de distribuição

O `LICENSE.md` do repositório informa que os arquivos estão sujeitos ao Minecraft End User License Agreement (EULA); não é uma licença permissiva como MIT ou Apache. Por isso, este repositório Book Edition não copia nem distribui os PNGs, sons, modelos ou outros assets proprietários da Mojang dentro do código-fonte ou do APK.

Referências:
- [Bedrock Samples — LICENSE.md](https://github.com/Mojang/bedrock-samples/blob/main/LICENSE.md)
- [Minecraft EULA](https://www.minecraft.net/en-us/eula)

## Plano técnico para texturas

1. Implementar um leitor próprio de Resource Pack, começando por `manifest.json`, `textures/terrain_texture.json` e atlas de blocos.
2. Permitir que o usuário selecione arquivos locais por meio do seletor de documentos do Android.
3. Validar dimensões, nomes e referências do atlas antes de enviar dados à GPU.
4. Manter os assets selecionados no armazenamento privado do app, sem enviá-los ao GitHub ou a servidores.
5. Criar texturas originais de teste para desenvolvimento e CI, com licença compatível e autoria documentada.

O renderer atual ainda usa cores por vértice. A integração de texturas reais depende do leitor de atlas e do novo layout de vértices com UVs; não será simulada com cores fixas nem declarada pronta antes dos testes.
