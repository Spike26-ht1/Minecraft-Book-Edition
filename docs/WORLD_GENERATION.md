# Geração de mundo — Book Edition

## Estado desta etapa

O crate `book-world` agora contém um gerador procedural determinístico com seed:

- `TerrainGenerator::new(seed)` cria as configurações de geração.
- `generate_section(chunk_x, section_y, chunk_z)` gera uma seção vertical 16×16×16.
- A mesma seed, versão do algoritmo e coordenadas geram os mesmos blocos.
- Coordenadas negativas usam divisão euclidiana para manter a continuidade entre chunks.
- A versão atual do algoritmo é `TERRAIN_GENERATOR_VERSION = 1`.
- O terreno possui variação de altura, quatro classificações iniciais de bioma (planícies, floresta, deserto e montanhas), camadas de superfície/subsolo e bedrock na altura zero.

## Limites importantes

Esta é a primeira base do gerador, não uma reprodução do gerador oficial Bedrock. Os biomas ainda não possuem árvores, vegetação, rios, oceanos, cavernas, minérios, estruturas, clima ou distribuição completa de blocos. O gerador ainda não está conectado à tela de criação de mundo nem ao streaming/renderizador de chunks.

A seed e a versão do gerador precisam ser salvas junto dos dados do mundo. Ao mudar o algoritmo no futuro, devemos criar uma nova versão em vez de alterar silenciosamente mundos existentes.

## Próximas etapas

1. Testar continuidade de altura nas bordas entre chunks.
2. Criar o contêiner de mundo com seed, versão, spawn e configurações.
3. Implementar salvamento/carregamento versionado.
4. Gerar chunks em tarefas de fundo com limite de trabalho por frame.
5. Conectar o resultado ao mesher e ao Book64.
6. Adicionar árvores, água, cavernas, minérios e estruturas em etapas isoladas, cada uma com testes de seed.
7. Criar a interface de novo mundo com seed digitada ou aleatória.

## Testes automatizados incluídos

- Mesma seed e coordenadas reproduzem os mesmos blocos.
- Seeds diferentes produzem diferenças em uma área de amostragem.
- A seção inferior contém bedrock.
- As alturas permanecem dentro dos limites definidos.
