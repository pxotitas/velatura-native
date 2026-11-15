# Matriz de portabilidade

Legenda: NOT AUDITED, PLANNED, IN PROGRESS, IMPLEMENTED, VERIFIED, BLOCKED.

Nenhum item começa como concluído. A auditoria estrutural inicial não equivale à portabilidade.

| Área | Referência primária | Estado | Critério mínimo |
|---|---|---|---|
| Entry point / lifecycle | shell.qml | NOT AUDITED | startup/shutdown, estado e recuperação |
| Multi-monitor | shell.qml, services/MonitorService.qml | NOT AUDITED | hotplug, outputs e ilha live |
| Top bar | bar/Bar.qml, bar/BarReserve.qml | NOT AUDITED | 4 estilos, zonas, exclusão e input |
| Dynamic Island | bar/widgets/DynamicIsland.qml, bar/island/* | NOT AUDITED | estados, morphs, painéis, OSD e notificações |
| Workspaces | bar/widgets/WorkspacesWidget.qml, services/NiriService.qml | NOT AUDITED | estado/eventos Niri e visual |
| Tray | bar/widgets/TrayWidget.qml, services/TrayService.qml | NOT AUDITED | StatusNotifierItem e menu |
| Bar modules | bar/modules/*, services/ModuleService.qml | NOT AUDITED | layout configurável e todos os módulos |
| Control center | bar/island/ControlsPanel.qml | NOT AUDITED | toggles, cards, navegação e estado |
| Launcher | bar/island/LauncherPanel.qml, services/LauncherService.qml | NOT AUDITED | busca, ranking, modos e execução |
| Appearance / themes | settings/AppearanceSection.qml, theme/* | NOT AUDITED | paletas, adaptive, matugen, tokens |
| Settings window | settings/SettingsWindow.qml, settings/SettingsPanel.qml | NOT AUDITED | páginas, persistência e edição |
| Desktop widgets | desktop/*, services/DesktopService.qml | NOT AUDITED | grid, drag, resize, inspector e faces |
| Dock | dock/*, services/DockService.qml | NOT AUDITED | pinned/running, menu, autohide, magnify |
| Deck | deck/Deck.qml, services/DeckService.qml | NOT AUDITED | notas/cartões e comportamento de borda |
| Lock screen | lock/*, services/LockService.qml | NOT AUDITED | lock seguro, autenticação e mídia |
| Capture | capture/*, services/CaptureService.qml | NOT AUDITED | seleção, overlay, salvar/cancelar |
| Notifications / OSD | services/NotificationService.qml, OsdService.qml | NOT AUDITED | freedesktop, filas, timeouts e ações |
| Audio | services/AudioService.qml | NOT AUDITED | volume, mute, sinks, sources |
| Bluetooth | services/BluetoothService.qml | NOT AUDITED | dispositivos, scan, conexão |
| Network | services/NetworkService.qml | NOT AUDITED | Wi-Fi, conexões e status |
| Brightness | services/BrightnessService.qml | NOT AUDITED | leitura/controle e dispositivos ausentes |
| Battery / power | services/BatteryService.qml, SessionService.qml | NOT AUDITED | energia, sessão, suspensão |
| Clipboard | services/ClipboardService.qml, scripts/clipboard.py | NOT AUDITED | histórico, imagens e wipe-on-lock |
| Media / lyrics | services/MediaService.qml, LyricsService.qml | NOT AUDITED | MPRIS, metadados, capa, letras |
| Stats / system | services/StatsService.qml, SystemService.qml | NOT AUDITED | CPU, RAM, GPU, sensores, disco |
| Notes / tasks / timer | services/NotesService.qml, TasksService.qml, TimerService.qml | NOT AUDITED | CRUD, persistência, datas e timer |
| Games | services/GamesService.qml, bar/island/games/* | NOT AUDITED | jogos e estado |
| Pet | services/PetService.qml, bar/island/PetPanel.qml | NOT AUDITED | família, animações e persistência |
| GitHub / Claude / Codex / markets / weather | serviços correspondentes | NOT AUDITED | APIs, credenciais, cache e falhas |
| Privacy / camera / recorder | serviços correspondentes | NOT AUDITED | indicadores, portal e dispositivos |
| VM / servers / packages / updates | serviços correspondentes | NOT AUDITED | integração e comandos seguros |
| Wallpaper / matugen / sunset | serviços correspondentes | NOT AUDITED | troca, paleta adaptativa e night light |
| IPC commands | Commands.qml | NOT AUDITED | CLI compatível ou API nova documentada |
| Scripts externos | scripts/* | NOT AUDITED | classificar/reimplementar cada função |
| Preferences migration | services/SettingsService.qml | NOT AUDITED | schema, defaults e import seguro |

## Regras

- Atualizar estado e evidências ao terminar cada área.
- Não usar IMPLEMENTED quando só existe UI.
- VERIFIED exige teste real ou automatizado documentado.
- Recursos externos podem exigir token, portal, daemon ou permissão; documentar pré-requisitos.
- Itens bloqueados devem ter motivo, impacto e caminho de resolução.
