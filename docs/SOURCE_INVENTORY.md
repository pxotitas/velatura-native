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
- Dock: pinned + running apps, menu contextual, autohide, magnificação, orientação e posição configuráveis; DockItem/DockMenu auditados em UI_SURFACE_AUDIT.md.
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


## Auditoria backend com leitura do runtime (15/11/2026)

A primeira fatia da auditoria funcional foi registrada em [BACKEND_AUDIT.md](BACKEND_AUDIT.md). Ela lê os arquivos ativos, não apenas o catálogo nominal.

### Grafo ativo confirmado

- `shell.qml` importa `bar/`, `capture/`, `desktop/`, `dock/`, `lock/`, `services/`, `settings/` e `theme/`; instancia as superfícies por `Quickshell.screens` e injeta `Commands`.
- `qmldir` registra os singletons e componentes do shell Linux. A declaração no `qmldir` não significa que todos iniciam imediatamente.
- O entry point toca explicitamente `ThemeService`, `MonitorService`, `PolkitService`, `SunsetService`, `IdleService` e `SessionService` no `Component.onCompleted`. O comentário do código diz que outros singletons são criados no primeiro uso e cacheados.
- `NiriService` e `MonitorService` participam diretamente da resolução de output/foco no entry point. Settings e outros serviços entram por dependências e consumidores.
- `islet/` e `macos-qckshell/` não são importados pelo `shell.qml` Linux nem registrados como componentes do módulo principal no `qmldir` lido nesta rodada; são projetos/experimentos separados, não serviços do runtime principal confirmado.
- Arquivos `.new`, `.bak` e `.before-*` não são referências primárias do grafo atual. Só devem ser reclassificados se um import/runtime ativo apontar para eles.

### Serviços e scripts com leitura integral nesta fatia

| Serviço/área | Fonte ativa lida | Script associado | Resultado |
|---|---|---|---|
| Entry point/lifecycle | `shell.qml`, `qmldir` | — | AUDITED |
| Niri/workspaces | `services/NiriService.qml` | — | AUDITED |
| Outputs/monitores | `services/MonitorService.qml` | `scripts/monitors.py` | AUDITED |
| Preferências | `services/SettingsService.qml` | — | AUDITED |
| Áudio | `services/AudioService.qml` | — | AUDITED |
| Rede/Wi-Fi | `services/NetworkService.qml` | `scripts/network.py` | AUDITED |
| Bluetooth | `services/BluetoothService.qml` | — | AUDITED |
| Bateria | `services/BatteryService.qml` | — | AUDITED |
| Brilho | `services/BrightnessService.qml` | — | AUDITED |
| Notificações | `services/NotificationService.qml` | — | AUDITED |
| Mídia/MPRIS | `services/MediaService.qml` | — | AUDITED |
| Clipboard | `services/ClipboardService.qml` | `scripts/clipboard.py` | AUDITED |
| Tray | `services/TrayService.qml` | — | AUDITED |

AUDITED nesta tabela é auditoria estática da implementação original. Nenhum desses itens foi implementado em Rust ou declarado VERIFIED.

### Evidência do ambiente Niri

- Versão consultada: Niri `26.04 (v26.04-276-g59351300)`.
- A ajuda local confirma comandos `outputs`, `workspaces`, `windows`, `event-stream`, `action` e `raw-request`.
- A consulta JSON read-only de outputs mostrou `Virtual-1` e `eDP-1` ativos. `Virtual-1` estava em x=-1920 e não pode ser tratado como output descartável.
- O helper `monitors.py` usa `niri msg --json outputs`, timeout 5 s, e não modifica a configuração do compositor.
- A lógica atual chama de primary o output focado quando válido; caso contrário, o primeiro output não desabilitado. Não há identidade primary independente do foco nesta implementação.

### Segurança e paridade descobertas

- `NetworkService.qml` envia a senha Wi-Fi como argumento para `network.py`; isso expõe a senha em argv. O backend Rust deve evitar repetir esse comportamento, preservando a operação por um canal seguro.
- `clipboard.py` exclui offers `CLIPBOARD_STATE=sensitive` e MIME `x-kde-passwordManagerHint`, limita payload a 16 MiB, mantém payload binário separado e usa lock + replace atômico.
- `MediaService.qml` só consulta posição a cada 1000 ms enquanto existe subscriber, o player toca e a faixa é seekable.
- `BrightnessService.qml` coalesce writes DDC e recupera uma falha lendo o monitor; não pode ser convertido em writes concorrentes no slider.
- `NotificationService.qml` possui o nome D-Bus do daemon, histórico de 50 entradas, ações e regra de prioridade/timeout; não é apenas um consumidor de notificações.

### Escopo ainda não auditado

Os serviços QML e os 28 scripts Python ativos foram lidos; Commands.qml/CLI e helpers/setup relevantes também foram auditados. A auditoria estática das superfícies e componentes visuais (bar/modules, island/panels, 16 control blocks, 14 settings sections, desktop/faces, dock, deck, lock, capture, tray, games e pet) está documentada em UI_SURFACE_AUDIT.md. UI_COMPONENT_CATALOG.md indexa 303 componentes ativos, seus serviços/estados, funções e handlers. Permanecem pendentes screenshots comparativos, testes de runtime/efeitos reais e resolução do símbolo ausente em Capture Window. A matriz separa AUDITED de VERIFIED.

### Auditoria de comandos/CLI (15/11/2026)

- Commands.qml foi lido integralmente e comparado com docs/IPC_API.md; targets, verbs, validações, payloads e retornos estão em [IPC_AUDIT.md](IPC_AUDIT.md).
- ~/.local/bin/velatura também foi lido integralmente: parser de subcomandos/flags, bridge qs ipc call, timeout, tratamento de erros, watch, completion, version e delegação de captura.
- Divergências confirmadas: ajuda da CLI anuncia layouts não aceitos pelo handler; brightness.step não tem entrada na CLI; capture region/window chama capture.open com um argumento embora a assinatura tenha dois; wallpaper.set é retorno texto cru; bar/widgets/shell.reload são void.
- velatura watch faz polling do status (1 s default, mínimo 250 ms), distinto dos eventos internos.
- scripts/capture.py e CaptureService.qml foram lidos integralmente; o motor de captura/OCR está AUDITED, mas ainda sem teste de efeito real. A composição raiz de CaptureOverlay/CaptureBar foi auditada estaticamente; window picking/drag/crop e runtime seguem pendentes.

### Capture e recorder (15/11/2026)

- Leitura integral: CaptureService.qml, RecorderService.qml, scripts/capture.py e scripts/record.py; evidências em BACKEND_AUDIT.md.
- Probes read-only: grim, ImageMagick e wl-copy disponíveis; satty ausente; tesseract disponível; encoder wl-screenrec ausente e wf-recorder disponível; monitor source de áudio disponível.
- Diretórios detectados: capturas em ~/Imagens e gravações em ~/Vídeos. Nenhuma captura/gravação foi iniciada.
- CaptureOverlay/CaptureBar: camada, foco, atalhos, seleção, confirmação e opções foram auditados estaticamente; `root.windowUnder()` não tem definição encontrada e bloqueia paridade do modo Window até recuperar/reproduzir a implementação original. Capture/recorder runtime ainda não estão VERIFIED.
### Launcher e índice de aplicações (15/11/2026)

- Leitura integral: LauncherService.qml, ApplicationsService.qml e scripts/applications.py; evidências em BACKEND_AUDIT.md.
- Modos por prefixo configurável: calculate (=), desk (>), windows (@), timer (!), clipboard ('), emoji (:), apps (default).
- Histórico de lançamentos usa frecency com meia-vida de 30 dias e bônus para apps fixados; ApplicationsService é compartilhado com DockService e indexa com atraso de 300 ms.
- O cálculo original usa Function() após whitelist regex; o backend nativo deve implementar parser aritmético próprio, sem runtime JavaScript.
- XDG desktop entries são lidos em ordem de precedência, com filtros Type/NoDisplay/Hidden; LauncherPanel, DockItem e DockMenu foram auditados em UI_SURFACE_AUDIT.md.
- Auditoria estática das superfícies raiz concluída para shell/bar/reserve/DynamicIsland host/Settings navigation/Desktop/Dock/Deck/Lock/Capture; contratos, tempos, máscaras e lifecycle em UI_SURFACE_AUDIT.md. Componentes filhos e screenshots comparativos permanecem pendentes.
- Segunda fatia visual: IslandState, ControlsPanel, ControlsTray, TopButtons, Block, BlockFace, BlockInspector e TopRowInspector lidos integralmente. Prioridade das camadas, grid, drag/resize, tray, inspector e confirmação destrutiva documentados em UI_SURFACE_AUDIT.md; 16 faces ainda pendentes.
- Terceira fatia visual: Module.qml, ChipFace.qml, ClockModule, VolumeModule, NetworkModule, BatteryModule e DetailFace lidos integralmente. Registry de 22 IDs, chip compartilhado, compact/detail routing e amostra de quatro módulos documentados em UI_SURFACE_AUDIT.md; outros módulos e IslandRest/Summary seguem pendentes.
### Dock backend (15/11/2026)

- DockService.qml foi lido integralmente; pinning, reordenação, agrupamento de janelas, match de WMClass/Desktop Entry, ativação/ciclo de janelas e geometria estão em BACKEND_AUDIT.md.
- A fonte de janelas é NiriService.clients; não há watcher separado. Pinned order usa debounce de 120 ms.
- O serviço não sabe quais outputs estão cobertos por fullscreen: coveredScreens é sempre vazio no original porque o modelo público Niri usado não expõe esse estado.
- Dock.qml, DockItem.qml e DockMenu.qml foram auditados estaticamente para layer-shell, input mask, autohide, magnificação, tooltip, item actions e menu lifecycle; validação runtime/screenshots pendentes.
### Desktop/Deck backend (15/11/2026)

- Leitura integral: DesktopService.qml (1512 linhas) e DeckService.qml (228 linhas); modelo de domínio e geometria documentados em BACKEND_AUDIT.md.
- Desktop widgets persistem por profile row; grid por monitor usa stride adaptativo, colisão determinística e nearest-free; posições clamped não são regravadas automaticamente.
- Rows de widgets podem viver em grid, notes deck por edge ou spectrum edge; notes só podem estar em um local; decks e spectrum preservam keys/ordem e campos próprios.
- Monitor identity persistida por descrição do monitor; quando desconectado, widget cai para effective primary; primary não persiste screen.
- Estado de edição é transitório e depende de um único focus grab; novo toplevel ou mudança de workspace encerra edição.
- Desktop.qml, Deck.qml, Widget.qml, Face registries, Inspector.qml, Picker.qml, Tray.qml, EdgeSpectrum e SpectrumBars foram auditados estaticamente; testes de interação/runtime/screenshots permanecem pendentes.
### Serviços de produtividade, sistema e sessão (15/11/2026)

- Leitura integral: NotesService.qml, TasksService.qml, TimerService.qml, VmService.qml + scripts/vms.py, StatsService.qml + scripts/stats.py, SystemService.qml + scripts/system.py, SessionService.qml, IdleService.qml, SunsetService.qml, PolkitService.qml e LockService.qml.
- Evidências de estados, persistência, parsing, polling adaptativo, subprocessos, erros e permissões em BACKEND_AUDIT.md.
- Stats é um watcher residente a cada 3 s, com disks/sensors a cada 30 s e histórico de 5 min; VMs só atualiza a cada 3 s enquanto VM roda ou painel tem subscriber; SystemService poll 4 s só com watchers.
- Sessão monitora PrepareForSleep e segura inhibitor logind até LockService confirmar secure; Idle usa ext-idle-notify com respectInhibitors; night light depende de wlsunset.
- Lock usa ext-session-lock, PAM login e captura grim por output antes de mapear a tela; senha vai direto ao PAM e é limpa. A construção shell de screenshot/cleanup é um ponto a corrigir com argv/FS APIs na versão Rust.
- FaceService, helpers face/avatar, PAM/polkit e setup foram auditados; no sistema atual estão ausentes. LockScreen/LockSurface/LockAccount/LockClock/LockMedia/LockPower/LockIsland foram auditados estaticamente; PAM/auth/runtime continuam pendentes. Nenhum desses serviços foi implementado ou marcado VERIFIED.
### Face auth, OSD e lyrics (15/11/2026)

- Leitura integral: FaceService.qml, OsdService.qml, LyricsService.qml e scripts/lyrics.py; evidências em BACKEND_AUDIT.md.
- Face unlock exige câmera IR com formato GREY, Howdy, helper/policy/PAM instalados; add/remove passam por pkexec; senha de login e trial face são conversas PAM separadas.
- Os helpers face/avatar e policies foram lidos no checkout `/home/px/impasto-niri`; não estão instalados nesta máquina, assim como howdy/PAM. Nenhum setup/instalação foi executado; integração fonte AUDITED, runtime indisponível.
- OSD centraliza volume/brightness/charging com debounce e suppression; lyrics consulta LRCLIB apenas com subscriber, usa cache de 7 dias para miss e sincroniza com MPRIS por binary search + timer de próxima linha.
### ModuleService (15/11/2026)

- ModuleService.qml (618 linhas) lido integralmente; catálogo, disponibilidade, side effects de lazy singletons, watchers, chip values/glyphs/tints, activities, glance e open panel state estão em BACKEND_AUDIT.md.
- `has()` pode instanciar singleton e iniciar request; o contrato Rust deve separar capability check puro de `ensure_started()` e subscriptions com lifecycle.
- Watchers de Claude/Codex/Weather/Markets/ScreenTime/Updates são acionados conforme módulo é mostrado; módulos com polling precisam de contagem de consumidores.
- Comentário sobre StatsService/ScreenTimeService no boot precisa ser cruzado com shell.qml e layout real antes de virar política de inicialização.
### Wallpaper/theme pipeline (15/11/2026)

- Leitura integral: ThemeService.qml, WallpaperService.qml, scripts/theme_manager.py (4317 linhas) e scripts/greeting.py (888 linhas).
- theme_manager.py gera paletas adaptativas, posters, state/symlink, temas para kitty/btop/cava/yazi/nvim/imv/GTK/Papirus/Qt/KDE/Vesktop/Spicetify/VSCodium/Zen/Nautilus, integra Matugen e possui ações de setup; inventário completo em BACKEND_AUDIT.md.
- greeting.py desenha 4 GIFs pixel-art (lava, critters, koi, invaders) com ImageMagick e troca symlink/paint/stamp atomicamente.
- WallpaperService é ligado pelo root shell; restaura outputs vazios, faz hotplug debounce e guarda ownership de awww sem editar config Niri.
- Risco legado: stop_motion usa pkill global por nome para mpvpaper; Rust deve rastrear e encerrar apenas o próprio child. restore ignora returncode final e set-wallpaper pode persistir sem awww.
- Nenhuma wallpaper foi alterada e nenhuma animação foi renderizada durante a auditoria.
## Inventário real de scripts (15/11/2026)

Leitura do diretório scripts/ confirmou 28 scripts Python ativos (`*.py`), dois candidatos históricos/experimentos (`emoji.py.new`, `theme_manager.py.new`) e `cava.conf` (configuração, não script). Os dois `.new` não são usados como fonte primária; confirmar imports/execuções antes de reclassificar.

**28/28 scripts Python ativos lidos integralmente.**

- Primeira rodada (12): applications.py, capture.py, clipboard.py, greeting.py, lyrics.py, monitors.py, network.py, record.py, stats.py, system.py, theme_manager.py e vms.py — fichas em BACKEND_AUDIT.md.
- Segunda rodada (16): account.py, camera.py, claude_usage.py, codex_usage.py, emoji.py, fonts.py, github.py, machine.py, markets.py, packages.py, privacy.py, screentime.py, servers.py, updates.py, version.py e weather.py — fichas em INTEGRATION_AUDIT.md.
- Os adapters QML correspondentes também foram lidos: Account, Camera, Claude, Codex, Emoji, Font, Github, Machine, Markets, Packages, Privacy, ScreenTime, Servers, Updates, Version e Weather.
- Helpers privilegiados face/avatar, PAM/polkit e os verbos setup system/face/camera foram lidos no checkout /home/px/impasto-niri; fonte auditada, mas não instalada nesta máquina.
- Dois candidatos históricos/experimentos permanecem fora do runtime primário: emoji.py.new e theme_manager.py.new. cava.conf é configuração, não script. Imports e referências ativas definem a fonte; arquivos .new não substituem o original.

Próximo passo: converter as fichas em providers/traits Rust e fixtures de compatibilidade; mocks não contam como implementação funcional.

### Integrações externas e contrato API (15/11/2026)

- 28/28 scripts Python ativos lidos integralmente. Os 16 scripts da segunda rodada + adapters QML estão em INTEGRATION_AUDIT.md.
- Fonte de setup e helpers face/avatar localizada em /home/px/impasto-niri; lidos setup system/face/camera, helpers, policies e PAM. Não instalados nesta máquina; não foram executados.
- Dependências e arestas atualizadas em DEPENDENCY_MAP.md e SERVICE_GRAPH.md.
- Contrato público inicial backend/frontend em APP_API_CONTRACT.md (0.1-draft, revisão de Codeína pendente; sem decisão de runtime).

### Auditoria final dos serviços QML (15/11/2026)

- Os 9 adapters finais foram lidos integralmente: CompositorService, ShortcutService, ProfileService, PickerService, ControlsService, CalendarService, CavaService, GamesService e PetService.
- Combinados com as rodadas anteriores, os 61/61 singletons QML ativos identificados pelo SERVICE_CATALOG foram inspecionados. Auditoria estática dos serviços concluída; a matriz mantém UI/gameplay/animações e testes reais como NOT AUDITED/pendentes.
- ShortcutService é intencionalmente read-only no original: compositorBindingsEditable=false, current vazio, fixed true, clash vazio, rebind só avisa para usar NiriMod opcional. Não inventar editor.
- ProfileService mantém profiles.json separado de dados pessoais e machineKeys; import/export é formato v1 e shipped profiles ficam em XDG_DATA_HOME/velatura/profiles.
- CalendarService usa dcal/Dank Calendar como fonte de verdade; CavaService inicia só com watchers/linger; GamesService e PetService mantêm records/família em JSON de estado.
- ControlsService implementa o domínio de 15 doors, 18 toggles, 16 block types, grid 6×8, nearest-free, resize e persistência centre*; composição do painel continua responsabilidade frontend.
