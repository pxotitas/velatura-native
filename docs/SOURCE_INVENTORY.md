# Inventário da fonte original

Auditoria inicial feita em 15/11/2026 diretamente em /home/px/.config/quickshell.

## Métricas observadas

- 658 arquivos no total (inclui arquivos auxiliares, backups, versões .new, recursos e conteúdo aninhado).
- Aproximadamente 6,6 MB.
- Estrutura original mantida em ~/.config/quickshell.
- O diretório não foi copiado nem alterado por esta preparação.
- Há arquivos .bak, .new e snapshots com sufixos de data. Eles não devem ser tratados automaticamente como fonte atual. O ponto de entrada atual é shell.qml; os imports ativos e a implementação usada por ele definem o caminho primário. Backups são contexto histórico apenas.

## Mapa de diretórios

| Diretório | Conteúdo / responsabilidade |
|---|---|
| shell.qml | Entry point, instanciação de superfícies por monitor, wiring global e lifecycle |
| bar/ | Barra superior, Dynamic Island, módulos, painéis, controles e jogos |
| capture/ | Barra e overlay de captura |
| components/ | Componentes visuais reutilizáveis |
| deck/ | Deck de notas/widgets no desktop |
| desktop/ | Widgets de desktop, editor, faces, inspector e picker |
| dock/ | Dock de aplicações, menus, autohide e magnificação |
| lock/ | Lock screen e seus componentes |
| scripts/ | Integrações auxiliares Python e scripts de suporte |
| services/ | Serviços, integrações com sistema, dados, estado e persistência |
| settings/ | Páginas e editores de configuração |
| theme/ | Tokens, paletas, movimento e tradução |
| islet/ | Projeto aninhado de terceiros/integração; auditar separadamente antes de definir escopo |
| macos-qckshell/ | Variante/experimento de shell macOS; não confundir com implementação Linux principal |

## Superfícies de primeiro nível

O shell.qml instancia por monitor:
- Bar: uma barra por output, uma delas marcada como live para receber interações globais;
- BarReserve: reserva o espaço da barra por monitor;
- Desktop: camada inferior com widgets e modo de organização;
- Dock: dock por monitor;
- Deck: notas/widgets ancorados na borda;
- ScreenCorners: cantos arredondados opcionais;
- SettingsWindow: janela xdg normal, separada da Dynamic Island.

Também existem módulos para captura e lock screen que devem ser rastreados nos imports e serviços correspondentes.

## Barra e Dynamic Island

Arquivos de entrada:
- bar/Bar.qml
- bar/BarReserve.qml
- bar/widgets/DynamicIsland.qml
- bar/island/IslandState.qml
- bar/island/IslandRest.qml
- bar/island/IslandSummary.qml
- bar/modules/Module.qml
- bar/widgets/WorkspacesWidget.qml
- bar/widgets/TrayWidget.qml

Estilos de barra configuráveis em SettingsService.qml:
- grouped: lados junto da ilha, movem-se para abrir espaço;
- spread: lados nas extremidades;
- island: tudo numa cápsula;
- bezel: faixa de ponta a ponta com a ilha recortada no centro.

A barra mantém uma superfície full-screen estável e muda a região de input, evitando redimensionar a layer surface em cada frame. Há uma instância por monitor e apenas uma ilha ativa/live.

## Módulos em bar/modules

Battery, Bluetooth, Brightness, Calendar, Claude, Clock, Codex, Games, Github, Markets, Media, Network, Notes, Notifications, Pet, ScreenTime, Stats, Tasks, Timer, Updates, Volume e Weather, além de Module e ChipFace.

O layout dos módulos é configurável e deve ser extraído dos defaults atuais em SettingsService.qml e ModuleService.qml, não inferido por esta lista.

## Painéis da ilha

Appearance/palette; Auth; Bluetooth, brightness, microphone, network, night-light e sound details; Board/tasks; Controls; Games; Keys; Launcher; Lyrics; Machines/VMs; Notes; Overview/workspaces; Packages; Pet; Screen time; Servers; Session/power; Stats; Tray menu; Notifications e OSD.

O mapeamento completo dos painéis ativos está em bar/widgets/DynamicIsland.qml.

## Desktop, dock e lock

- Desktop: widgets em grade, arranjo por drag, resize, inspector, picker, faces analógicas e sticker.
- Dock: pinned + running apps, menu contextual, autohide, magnificação, orientação e posição configuráveis.
- Lock: LockScreen, LockSurface, LockIsland, LockClock, LockMedia, LockPower, LockAccount, FaceRing, FlipCard e Padlock.
- Deck: notas/cartões nas bordas.
- Capture: seleção/overlay e integração com ferramentas do sistema.

## Serviços (services/)

Serviços observados:
Account, Applications, Audio, Battery, Bluetooth, Brightness, Calendar, Camera, Capture, Cava, Claude, Clipboard, Codex, Compositor, Controls, Deck, Desktop, Dock, Emoji, Face, Font, Games, Github, Idle, Launcher, Lock, Lyrics, Machine, Markets, Media, Module, Monitor, Network, Niri, Notes, Notification, OSD, Packages, Pet, Picker, Polkit, Privacy, Profile, Recorder, ScreenTime, Servers, Session, Settings, Shortcut, Stats, Sunset, System, Tasks, Theme, Timer, Tray, Updates, Version, VM, Wallpaper e Weather.

Cada serviço precisa ser aberto e auditado para identificar comandos, dependências, dados, intervalos, estado persistente e falhas possíveis. Esta lista é inventário nominal, não prova de portabilidade concluída.

## Scripts (scripts/)

Scripts Python observados:
account, applications, camera, capture, claude_usage, clipboard, codex_usage, emoji, fonts, github, greeting, lyrics, machine, markets, monitors, network, packages, privacy, record, screentime, servers, stats, system, theme_manager, updates, version, vms e weather; além de configuração CAVA e arquivos auxiliares.

Não manter scripts Python como substitutos automáticos da implementação nativa. Determinar se cada script deve ser reimplementado em Rust, chama ferramenta externa indispensável, pode ser removido porque a função é coberta por API nativa, ou precisa continuar como integração opcional e isolada.

## Configuração e temas

- Preferências: services/SettingsService.qml.
- Tokens visuais: theme/Theme.qml.
- Paletas: theme/Palettes.qml.
- Movimento: theme/Motion.qml.
- Traduções: theme/Tr.qml.
- Configuração persistida pelo original em $XDG_STATE_HOME/quickshell/settings.json, fallback ~/.local/state/quickshell/settings.json.

Defaults visuais confirmados:
- barHeight = 32;
- barMargin = 16;
- barSideMargin = 18;
- barStyle = grouped;
- surfaceStyle = classic (superfície sólida, sem blur);
- dockIconSize = 44;
- dockMagnify = true;
- relógio em formato 24h, sem data/segundos por padrão;
- motionScale e curva precisam ser confirmados no schema completo.

## Próxima auditoria necessária

A auditoria inicial mapeia estrutura e componentes principais, mas não substitui a leitura linha a linha de cada serviço. Antes de implementar um módulo, registrar:
1. arquivo(s) fonte;
2. API(s) externas;
3. estados e eventos;
4. dados persistidos;
5. comportamento visual esperado;
6. cenários de erro;
7. teste de paridade.


## Auditoria técnica inicial (15/11/2026)

A varredura estática identificou 61 serviços QML ativos e 28 scripts Python ativos, excluindo arquivos .new, .bak e .before-*. A lista de funções, propriedades, imports, mecanismos, comandos estáticos e consumidores está em SERVICE_CATALOG.md. Scripts e ferramentas externas estão em SCRIPT_CATALOG.md; integrações por domínio em DEPENDENCY_MAP.md; API IPC em IPC_API.md.

A contagem de consumidores é baseada em referências textuais entre QML ativos; pode incluir comentários e não prova execução em runtime. A varredura não resolve imports dinâmicos, singletons lazy, comandos montados em runtime ou todos os endpoints. É um mapa de auditoria, não certificação de completude.
