# Geração de mundo — Book Edition

## Estado desta etapa

O crate `book-world` contém um gerador procedural determinístico com seed:

- `TerrainGenerator::new(seed)` cria as configurações de geração.
- O menu **JOGAR → Criar mundo** aceita uma seed numérica ou gera uma seed aleatória; a seleção é lembrada localmente e enviada ao Book64 pelo JNI.
- `generate_section(chunk_x, section_y, chunk_z)` gera uma seção vertical 16×16×16.
- A mesma seed, versão do algoritmo e coordenadas geram os mesmos blocos.
- As coordenadas do terreno são absolutas; chunks vizinhos consultam a mesma função de altura e de bioma.
- Coordenadas negativas usam divisão euclidiana.
- A versão atual do algoritmo é `TERRAIN_GENERATOR_VERSION = 1`.
- O terreno possui variação de altura, quatro classificações iniciais de bioma (planícies, floresta, deserto e montanhas), camadas de superfície/subsolo e bedrock na altura zero.
- A cena de teste dev.9 gera uma região horizontal de 5×5 chunks, cada coluna com quatro seções verticais.
- O mesher consulta os chunks vizinhos nas direções X, Y e Z e elimina faces internas em bordas entre chunks.

## Limites importantes

Esta é uma base procedural própria, não uma reprodução do gerador oficial Bedrock. Os biomas ainda não possuem árvores, vegetação, rios, oceanos, cavernas, minérios, estruturas, clima ou distribuição completa de blocos. A região 5×5 é gerada na inicialização e ainda não existe streaming, descarregamento ou atualização incremental de malha.

A câmera é livre e ainda não possui colisão, gravidade ou física de jogador. A seed selecionada é lembrada nas preferências locais, mas ainda não existe um arquivo de mundo com blocos persistidos. A seed e a versão do gerador precisam ser gravadas nos metadados de cada mundo. Ao mudar o algoritmo no futuro, devemos criar uma nova versão em vez de alterar silenciosamente mundos existentes.

## Próximas etapas

1. Introduzir um contêiner de mundo com seed, versão, spawn e configurações.
2. Implementar salvamento/carregamento versionado de blocos modificados.
3. Gerar chunks em tarefas de fundo com limite de trabalho por frame.
4. Adicionar invalidation de malha e carregamento/descarregamento por distância.
5. Adicionar árvores, vegetação, água, cavernas, minérios e estruturas em etapas isoladas, cada uma com testes de seed.
6. Implementar colisões, gravidade, spawn seguro e movimento de jogador.
7. Expandir a interface de mundo com nome, configurações e lista de mundos salvos.

## Testes automatizados incluídos

- Mesma seed e coordenadas reproduzem os mesmos blocos.
- Seeds diferentes produzem diferenças em uma área de amostragem.
- A seção inferior contém bedrock.
- As alturas permanecem dentro dos limites definidos.
- Faces internas entre dois blocos no mesmo chunk são removidas.
- Faces internas entre seções verticais vizinhas são removidas.
- Faces internas entre chunks horizontais vizinhos são removidas.
- Coordenadas negativas são transformadas em posições de mundo corretamente.
