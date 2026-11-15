# Android media

Este diretório do repositório documenta a estrutura de mídia usada pelo aplicativo.

## Pasta externa criada no aparelho

Na inicialização, o Book Edition tenta preparar o diretório retornado por Android `getExternalMediaDirs()`. Em armazenamento compartilhado principal, o caminho esperado é:

`/storage/emulated/0/Android/media/dev.bookedition.launcher/book-edition/`

Subpastas:
- `resource-packs/` — packs importados e atlas gerado.
- `worlds/` — mundos do Book Edition (estrutura reservada).
- `exports/` — exportações.
- `logs/` — registros exportáveis.
- `cache/` — arquivos temporários.

O caminho real é mostrado em Launcher → Ajustes. A disponibilidade depende do volume externo do aparelho; o app mantém armazenamento interno privado como fallback.

No Android 11 ou superior, o acesso visual pode ser feito pelo gerenciador de arquivos em `Android/media`. O app não solicita acesso amplo ao armazenamento.
