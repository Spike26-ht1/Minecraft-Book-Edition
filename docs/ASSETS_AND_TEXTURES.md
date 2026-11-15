# Assets e texturas

## Referência Bedrock Samples

O repositório [Mojang/bedrock-samples](https://github.com/Mojang/bedrock-samples) orienta a estrutura de Resource Packs Bedrock. O Book Edition implementa um leitor próprio e não incorpora assets Mojang no APK.

## Importador implementado

A partir de dev.6, o menu de configurações do jogo oferece **Importar Resource Pack (.mcpack / .zip)**.

O importador atual:
- Abre arquivos escolhidos pelo usuário via Android Storage Access Framework.
- Valida a presença de manifest.json e textures/terrain_texture.json.
- Lê texture_data, incluindo caminhos diretos, arrays e objetos com path/textures.
- Resolve imagens PNG referenciadas, gera terrain-atlas.png e atlas-index.json no armazenamento privado.
- Registra nomes, contagem de texturas resolvidas/ausentes e dimensões para o painel F3.
- Impõe limites de tamanho, quantidade de entradas, dimensão individual e tamanho do atlas para reduzir risco de arquivos malformados.

Limitações conhecidas:
- Esta etapa gera e indexa o atlas no armazenamento privado; ainda não envia a textura à GPU nem troca os UVs do mesh. A cena continua renderizando com cores por vértice.
- O leitor ainda não processa flipbook_textures.json, materiais, animações, tintas de bioma, geometria de blocos, render controllers ou todos os esquemas alternativos de terrain_texture.
- Um atlas que ultrapasse 2048×4096 é rejeitado; suporte a múltiplas páginas ainda será implementado.
- Não há promessa de compatibilidade com todos os packs ou versões Bedrock nesta etapa.

## Distribuição e licenças

O conteúdo de Mojang/bedrock-samples está sujeito aos termos indicados pelo próprio repositório e pelo Minecraft EULA; não é tratado como conteúdo MIT/Apache. O projeto não copia nem redistribui PNGs, sons, modelos ou outros assets proprietários. Os packs selecionados pelo usuário permanecem localmente no dispositivo.

Referências:
- [Bedrock Samples — LICENSE.md](https://github.com/Mojang/bedrock-samples/blob/main/LICENSE.md)
- [Minecraft EULA](https://www.minecraft.net/en-us/eula)
