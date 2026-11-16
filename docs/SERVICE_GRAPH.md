# Grafo de dependências entre serviços

Arestas inferidas por referências textuais a singletons dentro de outros arquivos de serviço ativos. Isso mostra acoplamento potencial; não prova que todas as referências são avaliadas em startup.

## AccountService
SettingsService

## ApplicationsService
LauncherService

## BrightnessService
NiriService

## CameraService
OsdService, SettingsService

## CaptureService
MonitorService, NiriService, OsdService, SettingsService

## CavaService
SettingsService

## ClipboardService
SettingsService

## CodexService
ClaudeService

## ControlsService
AudioService, BluetoothService, CaptureService, ClipboardService, DesktopService, NetworkService, NotificationService, OsdService, PickerService, RecorderService, SessionService, SettingsService, SunsetService, SystemService

## DeckService
DesktopService, NiriService, NotesService, SettingsService

## DesktopService
DeckService, DockService, ModuleService, MonitorService, NiriService, NotesService, SettingsService

## DockService
ApplicationsService, NiriService, SettingsService

## EmojiService
SettingsService

## FaceService
PackagesService, VersionService

## GamesService
ModuleService, OsdService, SettingsService

## GithubService
SettingsService

## IdleService
LockService, NiriService, SessionService, SettingsService

## LauncherService
ApplicationsService, ClipboardService, ControlsService, DockService, EmojiService, NiriService, SettingsService, TimerService

## LockService
FaceService

## LyricsService
MediaService, SettingsService

## MarketsService
SettingsService

## ModuleService
AudioService, BatteryService, BluetoothService, BrightnessService, CaptureService, ClaudeService, CodexService, ControlsService, DesktopService, GamesService, GithubService, LyricsService, MarketsService, MediaService, NetworkService, NotesService, NotificationService, PetService, PrivacyService, ScreenTimeService, SettingsService, StatsService, TasksService, TimerService, UpdatesService, WeatherService

## MonitorService
NiriService

## NiriService
SettingsService

## NotesService
DesktopService, SettingsService, TasksService

## NotificationService
SettingsService

## OsdService
AudioService, BatteryService, BrightnessService, ModuleService

## PackagesService
UpdatesService

## PetService
LockService, ModuleService, OsdService, SettingsService

## PickerService
OsdService

## PrivacyService
AudioService, ModuleService

## ProfileService
OsdService, SettingsService, ThemeService, WallpaperService

## RecorderService
NiriService, OsdService, SettingsService

## ScreenTimeService
DockService, IdleService, LockService, NiriService, NotificationService, SettingsService

## SessionService
LockService, NiriService

## SettingsService
AccountService, CameraService, ControlsService, DesktopService, IdleService, ModuleService, PetService, ProfileService, ShortcutService, SunsetService, ThemeService, WallpaperService

## ShortcutService
SettingsService

## SunsetService
OsdService, SettingsService

## SystemService
OsdService

## TasksService
NotesService, SettingsService

## ThemeService
SettingsService

## TrayService
ModuleService

## VersionService
PackagesService

## WallpaperService
OsdService, SettingsService, ThemeService

## WeatherService
SettingsService

## Serviços centrais com muitos consumidores

A contagem abaixo considera arquivos QML ativos que mencionam o singleton; pode contar comentário e não mede frequência de execução.

- SettingsService: 78 arquivos consumidores
- DesktopService: 65 arquivos consumidores
- ModuleService: 44 arquivos consumidores
- TasksService: 20 arquivos consumidores
- NiriService: 18 arquivos consumidores
- ControlsService: 18 arquivos consumidores
- MediaService: 17 arquivos consumidores
- NotesService: 15 arquivos consumidores
- PetService: 14 arquivos consumidores
- OsdService: 14 arquivos consumidores
- AudioService: 14 arquivos consumidores
- WeatherService: 11 arquivos consumidores
- TimerService: 11 arquivos consumidores
- StatsService: 11 arquivos consumidores
- LockService: 11 arquivos consumidores
- BrightnessService: 11 arquivos consumidores
- WallpaperService: 10 arquivos consumidores
- ThemeService: 10 arquivos consumidores
- ScreenTimeService: 10 arquivos consumidores
- NotificationService: 10 arquivos consumidores
- NetworkService: 9 arquivos consumidores
- MarketsService: 9 arquivos consumidores
- GamesService: 9 arquivos consumidores
- BatteryService: 9 arquivos consumidores
- UpdatesService: 8 arquivos consumidores
- ClaudeService: 8 arquivos consumidores
- BluetoothService: 8 arquivos consumidores
- SessionService: 7 arquivos consumidores
- LyricsService: 7 arquivos consumidores
- DockService: 7 arquivos consumidores
- CodexService: 7 arquivos consumidores
- CaptureService: 7 arquivos consumidores
- AccountService: 7 arquivos consumidores
- SunsetService: 6 arquivos consumidores
- MonitorService: 6 arquivos consumidores
- LauncherService: 6 arquivos consumidores
- GithubService: 6 arquivos consumidores
- DeckService: 6 arquivos consumidores
- TrayService: 5 arquivos consumidores
- RecorderService: 5 arquivos consumidores
- PackagesService: 5 arquivos consumidores
- ShortcutService: 4 arquivos consumidores
- ClipboardService: 4 arquivos consumidores
- CavaService: 4 arquivos consumidores
- VmService: 3 arquivos consumidores
- PrivacyService: 3 arquivos consumidores
- IdleService: 3 arquivos consumidores
- CompositorService: 3 arquivos consumidores
- CameraService: 3 arquivos consumidores
- VersionService: 2 arquivos consumidores
- SystemService: 2 arquivos consumidores
- ServersService: 2 arquivos consumidores
- ProfileService: 2 arquivos consumidores
- PolkitService: 2 arquivos consumidores
- PickerService: 2 arquivos consumidores
- MachineService: 2 arquivos consumidores
- FaceService: 2 arquivos consumidores
- EmojiService: 2 arquivos consumidores
- CalendarService: 2 arquivos consumidores
- ApplicationsService: 2 arquivos consumidores
- FontService: 1 arquivos consumidores

## Ciclo de vida do startup observado em shell.qml

No Component.onCompleted o shell toca explicitamente estas propriedades para instanciar/ativar serviços críticos:
- ThemeService.activeId
- MonitorService.monitors
- PolkitService.asking
- SunsetService.available
- IdleService.lockAfter
- SessionService.sleepPending

O comentário do entry point afirma que os demais singletons são criados no primeiro uso e ficam em cache durante a vida do processo. A implementação nativa deve preservar o custo de startup e o lazy loading, mas não deve deixar serviço essencial indisponível quando uma superfície depende dele.

## Topologia de superfícies observada

- Uma BarReserve por output.
- Uma Bar por output; exatamente uma recebe live=true.
- Uma Desktop por output.
- Uma Dock por output, com visibilidade decidida por preferências e monitor live.
- ScreenCorners por output quando habilitado.
- Um Deck por output.
- SettingsWindow é janela normal, não layer surface.
- Capture/Lock têm superfícies e lifecycle próprios; rastrear imports e triggers no shell.qml completo.


## Arestas de integração externa auditadas (15/11/2026)

- AccountService → account.py → AccountsService D-Bus/busctl; avatar helper + polkit opcional.
- FaceService → howdy-next + helper face + polkit + PAM velatura-face + v4l2 GREY.
- CameraService → camera.py → V4L2/v4l2loopback + OpenCV/MediaPipe/NumPy/Pillow + CaptureService para copiar foto.
- ClaudeService → claude_usage.py → ~/.claude transcripts/cache + Anthropic OAuth usage endpoint; CodexService → codex_usage.py → ~/.codex session logs (local only).
- EmojiService → emoji.py → Unicode emoji-test, CLDR annotations, iso-codes; FontService → fonts.py → fontconfig; MachineService → machine.py → os-release/proc/passwd/pacman.
- GithubService → github.py → public GitHub contribution HTML; MarketsService → markets.py → CoinGecko/Frankfurter; WeatherService → weather.py → wttr.in.
- PackagesService → packages.py → pacman + yay/paru + AUR RPC + kitty; UpdatesService → updates.py → checkupdates/pacman/AUR/vercmp; VersionService → version.py → git fetch + checkout setup.
- PrivacyService → privacy.py → /dev/video*, inotifywait e /proc; ScreenTimeService → screentime.py → Niri focus + Lock/Idle/Notifications + XDG state + Steam manifests; ServersService → servers.py → ss + /proc + signals.
- Theme/Wallpaper → theme_manager.py/greeting.py → awww/mpvpaper/ffmpeg/Matugen/ImageMagick e configs de apps; ownership de processos precisa ser estrito.

As arestas acima descrevem dependências, não autorizam inicialização em startup. Consulte INTEGRATION_AUDIT.md para intervalos, payloads, falhas e condições de subscription.

## Arestas QML complementares auditadas (15/11/2026)

- CompositorService → niri --version (somente leitura); ShortcutService → catálogo de 31 shell actions + SettingsService.keys; NiriMod opcional, edição desativada no original.
- ProfileService → SettingsService.snapshot/complete/adopt + ThemeService + WallpaperService + Palettes + state/profiles.json + XDG_DATA_HOME/velatura/profiles.
- ControlsService → Session/Network/Bluetooth/System/Notification/Audio/Sunset/Settings/Capture/Picker/Recorder/Clipboard e SettingsService.centre*; UI de control center consome o modelo.
- PickerService → hyprpicker + wl-copy + OsdService; aguarda morph/settle antes de congelar a tela.
- CalendarService → dcal IPC; pode iniciar Dank Calendar em modo hidden; dados permanecem no serviço externo.
- CavaService → cava -p scripts/cava.conf; subscriptions dos chips/painéis e gameMode controlam o processo.
- GamesService → XDG state games.json; game panels são consumidores, scores são domínio persistente.
- PetService → XDG state pet.json + LockService + SettingsService.onBar("pet"); UI da família e estilos consome apenas o pet ativo.

Os 61 serviços ativos estão mapeados estaticamente; ainda faltam as fichas visuais dos consumers e a validação runtime.

## Consumers visuais identificados por referência estática (não auditados)

- ProfilesPart.qml consome ProfileService: cards, resumo de settings/palette, rename/duplicate/create/switch/remove e dialogs import/export.
- KeysSection.qml e KeysPanel.qml consomem ShortcutService.catalogue/configPath; o binding sheet permanece vazio por design do backend atual.
- CalendarModule.qml consome refreshMonth/eventsOn/hasEventsOn/newEvent/openCalendar/openEvent.
- SpectrumBars.qml, components/Spectrum.qml, MediaModule.qml e IslandRest.qml assinam CavaService; desktop faces usam SpectrumBars.
- ControlsPanel.qml, ControlsSection.qml, TopRowEditor.qml, TopRowInspector.qml, TogglesBlock.qml consomem ControlsService.
- GamesModule.qml, GamesPanel.qml, GameBoard.qml, Game faces e desktop faces consomem GamesService.
- PetModule.qml, PetPanel.qml, PetBlock.qml, Pet faces, ModulesSection.qml e desktop faces consomem PetService.
- shell.qml e ControlsService chamam PickerService; OSD é emitido pelo serviço.
- CompositorService não tem consumer visual próprio; NiriService consome a integração de compositor.

Este mapa é resultado de rg de referências e não substitui auditoria visual. Cada caminho acima permanece NOT AUDITED na matriz quando é superfície UI.
