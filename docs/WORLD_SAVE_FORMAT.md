# Formato local de mundo — Book Edition

**Formato:** `BOOKWLD1` (version 1)  
**Gerador:** Book Edition terrain generator v1  
**Escopo:** dados locais do Book Edition; não é um arquivo de mundo Bedrock e não é compatível com Minecraft.

## Layout binário

Todos os inteiros usam little-endian.

| Campo | Tamanho | Descrição |
|---|---:|---|
| Magic | 8 bytes | ASCII `BOOKWLD1` |
| Format version | u32 | Versão do codec |
| Generator version | u32 | Versão do gerador que criou o mundo |
| Seed | u64 | Seed do mundo |
| Column count | u32 | Total de colunas armazenadas |

Cada coluna contém coordenadas X/Z (i32), quantidade de seções (u16) e os dados das seções. Cada seção contém exatamente 4096 IDs de bloco u16 na ordem interna de armazenamento de Book Edition (Y, Z, X).

## Salvamento Android

O runtime grava em `Android/media/<application-id>/book-edition/worlds` quando o diretório de mídia externo do app está disponível, usando `filesDir/book_worlds` como fallback.

A gravação usa arquivo temporário, sincronização e renomeação para reduzir o risco de arquivo parcial.

## Limites de leitura

O decoder limita o arquivo a 64 MiB, 1024 colunas e 32 seções por coluna. Rejeita magic/formato/gerador incompatíveis, arquivo truncado, dados excedentes e coordenadas de coluna repetidas.

## Limitações atuais

O formato ainda não é Bedrock LevelDB/DB, NBT ou subchunk palette. Ele salva apenas blocos das colunas carregadas, seed e versão do gerador. Posição do jogador, inventário, entidades, biomas persistidos e streaming ainda não são salvos. Um arquivo com versão de gerador desconhecida é recusado para evitar interpretar dados de modo incorreto.
