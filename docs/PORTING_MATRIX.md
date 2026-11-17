# Matriz de portabilidade

Estados permitidos: NOT AUDITED, AUDITED, IN PROGRESS, IMPLEMENTED, VERIFIED, BLOCKED.

AUDITED significa fonte e comportamento lidos/documentados; não significa implementação nem teste real. Os 61/61 singletons QML ativos e 28/28 scripts Python ativos foram lidos. Evidências em [BACKEND_AUDIT.md](BACKEND_AUDIT.md), [INTEGRATION_AUDIT.md](INTEGRATION_AUDIT.md), [IPC_AUDIT.md](IPC_AUDIT.md) e [UI_SURFACE_AUDIT.md](UI_SURFACE_AUDIT.md).

| Área | Referência primária | Estado | Critério mínimo |
|---|---|---|---|
| API contracts — Compositor/Monitor/Idle/Session/Lock/Polkit/Font/Emoji/Machine/Version/Sunset/Applications/Niri/Settings/Audio/Network/Bluetooth/Battery/Brightness | APP_API_CONTRACT.md, APP_API_MATRIX.md, APP_API_REVIEW.md | IN PROGRESS | três grupos especificados estaticamente com consumidores/evidências; lacunas reais de persistência, provider handles, errors e runtime permanecem; contratos não autorizam implementação 1:1 integral |
| Visual token foundation | Theme.qml, Palettes.qml, Motion.qml, AppearanceSection.qml, SettingsService.qml | IN PROGRESS | tokens classic/default e catálogo de paletas centralizados em `src/ui/theme.rs`; settings persistidos, paleta adaptativa, glass/liquid e comparação visual permanecem pendentes; ver `VISUAL_THEME_AUDIT.md` |
| Entry point / lifecycle | shell.qml | AUDITED | startup/shutdown, estado e recuperação |
| Multi-monitor | shell.qml, services/MonitorService.qml | AUDITED | hotplug, outputs e ilha live |
| Shell surface composition | shell.qml | AUDITED | variants por output, handover live, wiring de Settings/Polkit/Lock/Capture/IPC |
| Bar reserve surface | bar/BarReserve.qml | AUDITED | exclusive zone estável, zen e ausência de input |
| Bar root + modules | bar/Bar.qml, bar/BarReserve.qml, bar/modules/*, IslandRest/Summary, OSD/Notification layers | AUDITED | quatro estilos, masks, 22 módulos, activity/glance, workspaces e tray; runtime/screenshots pendentes |
| Dynamic Island host + panels | bar/widgets/DynamicIsland.qml, bar/island/* | AUDITED | 26 host panel IDs (appearance/palette share component) + detail routes, priority, sizing, hover/dwell, keyboard focus, loading/empty/error and action flows. Primeira implementação nativa (src/ui) reprovada na validação visual e arquivada em glm-made-things/ (D-011); re-port a partir do QML original pendente |
| Island state arbitration | bar/island/IslandState.qml | AUDITED | prioridade panel/notification/OSD/summary/modules, timeout, handover e dismiss |


| Niri IPC state/events/actions + outputs | services/NiriService.qml, services/MonitorService.qml, scripts/monitors.py; src/niri.rs, src/niri_process.rs, src/niri_actions.rs, src/niri_outputs.rs, src/niri_session.rs, src/gdk_monitor_observer.rs, src/application_runtime.rs, src/ui/gtk_event_bridge.rs | IN PROGRESS | Snapshot atômico revisionado, bridge sem polling até main context, matching GDK↔Niri por connector/geometry, disconnect explicitado; permanecem testes físicos de hotplug/geometry e validação real de reconnect |
| Compositor info | services/CompositorService.qml | AUDITED | Niri version probe; config path read-only |
| Shortcut catalogue | services/ShortcutService.qml | AUDITED | 32 shell actions; binding edit is intentionally disabled in original |
| Shortcut UI | settings/KeysSection.qml, bar/island/KeysPanel.qml | AUDITED | 32-action catalogue presentation, Niri KDL path, explicit no-IPC-bindings explanation and NiriMod handoff |
| Workspaces UI | bar/widgets/WorkspacesWidget.qml | AUDITED | actual per-output Niri workspace list, focused/occupied/empty, 8 styles, hit target and focusWorkspaceOn |
| Tray backend | services/TrayService.qml | AUDITED | StatusNotifierItem, DBusMenu e lifecycle |
| Tray UI | bar/widgets/TrayWidget.qml, bar/island/TrayMenuPanel.qml | AUDITED | fold/unfold, primary/secondary/context actions, scroll, DBusMenu stack and keyboard navigation |
| Module catalogue backend | services/ModuleService.qml | AUDITED | availability, glyph/value, subscriptions, panel state e glance |
| Module registry / shared chip | bar/modules/Module.qml, ChipFace.qml | AUDITED | 22 IDs, compact/detail dispatch, shared chip metrics, ring/value reveal |
| Sample bar modules | bar/modules/{Clock,Volume,Network,Battery}Module.qml, bar/island/DetailFace.qml | AUDITED | clock precision, audio streams/outputs, network status, battery health, special detail routing |
| Bar module UI | bar/modules/*, bar/island/{IslandRest,IslandSummary,DetailFace}.qml | AUDITED | 22 module contracts, shared chip, activity/glance, lifecycle and special detail routing; visual/runtime parity pending |
| Controls domain backend | services/ControlsService.qml | AUDITED | 15 doors, 18 toggles, block catalogue, grid, collisions e persistence |
| Control center UI | bar/island/ControlsPanel.qml, controls/* | AUDITED | 16 block faces, toggles pagination, card actions, notifications, session and shared cards |
| Control center grid/editing | bar/island/ControlsPanel.qml, controls/* | AUDITED | grid, all 16 block faces, add/remove/drag/resize, inspector, tray, top row and destructive confirmation |
| Launcher backend | services/LauncherService.qml | AUDITED | prefix modes, busca, frecency, calculator parser, activation |
| Launcher UI | bar/island/LauncherPanel.qml | AUDITED | mode prefixes, keyboard selection, modifiers, emoji grid/groups/tones and clipboard deletion |
| Applications index | services/ApplicationsService.qml, scripts/applications.py | AUDITED | XDG precedence, desktop entries, filters e launch |
| Appearance / themes backend | ThemeService.qml, WallpaperService.qml, theme_manager.py, greeting.py | AUDITED | paletas, adaptive, Matugen, wallpaper, providers e greeting |
| Appearance UI | settings/AppearanceSection.qml, bar/island/AppearancePanel.qml, theme/* | AUDITED | wallpaper folder/transitions, Matugen, terminal/greeting, fonts, motion/game mode and palette routing |
| Settings UI | settings/*Section.qml, SettingsWindow/Panel, LayoutEditor, ProfilesPart | AUDITED | all 14 sections, tabs, 108-key settings surface, bar layout editor, pinned apps, profile CRUD/import/export and system update/restore flows |
| Settings schema / migration | services/SettingsService.qml | AUDITED | schema integral, defaults, perfis, migração e gravação |
| Profiles backend | services/ProfileService.qml | AUDITED | create/switch/rename/import/export, shipped profiles e isolation |
| Profiles UI | settings/ProfilesPart.qml | AUDITED | profile cards, summary, rename/duplicate/export/import, delete confirmation and active switch |
| Desktop backend | services/DesktopService.qml, services/DeckService.qml | AUDITED | persisted widget model, grid, collision, notes/decks e spectrum |
| Desktop UI | desktop/{Desktop,Widget,Inspector,Picker,Tray,EdgeSpectrum,SpectrumBars,Face}.qml, faces/* | AUDITED | widget lifecycle, drag/resize, edge conversion, inspector, photo picker, gallery, 4 modern families + analogue/sticker registries |
| Dock backend | services/DockService.qml | AUDITED | pinning, grouping, class matching, activation e geometry |
| Dock UI | dock/{Dock,DockItem,DockMenu}.qml | AUDITED | layer/mask, autohide, magnification, tooltip, launch/running/pinned item actions and contextual menu |
| Deck root surface | deck/Deck.qml | AUDITED | reveal, mask, tabs/peek, drag/reorder/transfer/drop e lifecycle por output; runtime/screenshots pendentes |
| Lock backend | services/LockService.qml | AUDITED | ext-session-lock, PAM, capture-before-lock, cleanup |
| FaceService adapter | services/FaceService.qml | AUDITED | IR/GREY probe, pkexec adapter, PAM trial e state |
| Face helper/setup | setup, system/usr/lib/velatura/face, PAM/polkit | AUDITED | helper, auth scope, model CRUD e instalação; não instalado nesta máquina |
| Lock UI | lock/* | AUDITED | ext-session-lock surfaces, account/password pill, clock styles, music/lyrics, face indicator, power and blur composition; PAM/runtime pending |
| Capture backend | services/CaptureService.qml, scripts/capture.py | AUDITED | probe de ferramentas, grab/crop/destinos/OCR e cancel |
| Capture region/screen UI | capture/{CaptureOverlay,CaptureBar}.qml | AUDITED | overlay, focus, keyboard shortcuts, region geometry, threshold, confirm/cancel and options; runtime/screenshots pending |
| Capture Window picker | capture/CaptureOverlay.qml | BLOCKED | root.windowUnder(mouse.x, mouse.y) has no definition in active source; recover/reproduce original helper before implementing 1:1 |
| Colour picker backend | services/PickerService.qml | AUDITED | hyprpicker probe, settle/cancel, hex validation e wl-copy |
| Recorder backend | services/RecorderService.qml, scripts/record.py | AUDITED | lifecycle, encoder fallback, áudio e recuperação após reload |
| Polkit agent | services/PolkitService.qml | AUDITED | agent registration, PAM flow, retry e fallback |
| Notifications / OSD | services/NotificationService.qml, OsdService.qml | AUDITED | freedesktop server, OSD aggregation, debounce e DND |
| Audio | services/AudioService.qml | AUDITED | volume, mute, sinks, sources |
| Bluetooth | services/BluetoothService.qml | AUDITED | dispositivos, scan, conexão |
| Network | services/NetworkService.qml, scripts/network.py | AUDITED | Wi-Fi, conexões, scan, senha e status |
| Brightness | services/BrightnessService.qml | AUDITED | leitura/controle e dispositivos ausentes |
| Battery | services/BatteryService.qml | AUDITED | UPower, ausência de bateria e métricas |
| Power / session | services/SessionService.qml, services/IdleService.qml | AUDITED | lock-before-suspend, logind, inhibitors e ext-idle-notify |
| Clipboard | services/ClipboardService.qml, scripts/clipboard.py | AUDITED | histórico, imagens, filtros de segredo e wipe-on-lock |
| MPRIS media | services/MediaService.qml | AUDITED | seleção de player, metadata, capabilities e controles |
| Lyrics | services/LyricsService.qml, scripts/lyrics.py | AUDITED | LRCLIB, cache, retries, LRC sync e watchers |
| Calendar backend | services/CalendarService.qml | AUDITED | Dank Calendar dcal IPC, month range, event CRUD/open e lazy daemon |
| Calendar UI | bar/modules/CalendarModule.qml, bar/island/controls/CalendarCard.qml | AUDITED | month grid, week start, event/task markers, day navigation and event open/create routing |
| Stats / system services | services/StatsService.qml, SystemService.qml, scripts/stats.py, system.py | AUDITED | CPU, RAM, disks, network rates, temperature, radios e power profile |
| Cava spectrum backend | services/CavaService.qml, scripts/cava.conf | AUDITED | 64 bands, 8 mirrored groups, peak decay, watcher/linger e game mode |
| Spectrum UI | desktop/SpectrumBars.qml, desktop/EdgeSpectrum.qml, components/Spectrum.qml, bar/modules/MediaModule.qml | AUDITED | 64 bands, 16 vec4 uniforms, peak arrays, shader render, sample mode and ref-counted Cava consumers |
| Notes / tasks / timer | services/NotesService.qml, TasksService.qml, TimerService.qml | AUDITED | CRUD, persistência, parsing de datas/duração e estados |
| Games registry backend | services/GamesService.qml | AUDITED | 11 games, score records, bests e persistence |
| Games UI/logic | bar/island/GamesPanel.qml, games/* | AUDITED | GameBoard contract, 11 games, keyboard/mouse input, timers, scoring, restart and record lifecycle |
| Pet backend | services/PetService.qml | AUDITED | family, XP, cooldown, mood, rest, milestones e migration |
| Pet UI/animation | bar/island/PetPanel.qml, components/pets/* | AUDITED | pet panel, rename/feed/play/family, XP/mood and idle/hop lifecycle; visual/runtime parity pending |
| VM backend | services/VmService.qml, scripts/vms.py | AUDITED | quickemu/quickget, lifecycle, snapshots e catálogo |
| Account / avatar | AccountService.qml, scripts/account.py, system/usr/lib/velatura/avatar | AUDITED | AccountsService, fallback, PNG normalize e pkexec helper |
| Camera effects | CameraService.qml, scripts/camera.py, setup camera | AUDITED | V4L2 loopback, MediaPipe effects/reactions, lazy stream e snapshots |
| Claude / Codex usage | ClaudeService.qml, CodexService.qml, scripts/*_usage.py | AUDITED | transcript/log parsing, OAuth limits, cache e watchers |
| Emoji / fonts / machine | EmojiService.qml, FontService.qml, MachineService.qml, scripts/{emoji,fonts,machine}.py | AUDITED | CLDR index, fc-list e machine facts lazy |
| GitHub / markets / weather | serviços correspondentes e scripts | AUDITED | scrape público, CoinGecko/Frankfurter, wttr.in e cache/watchers |
| Packages / updates / version | PackagesService.qml, UpdatesService.qml, VersionService.qml, scripts correspondentes | AUDITED | pacman/AUR, terminal interativo, checkupdates e git fetch/status |
| Privacy / screen time / servers | PrivacyService.qml, ScreenTimeService.qml, ServersService.qml, scripts correspondentes | AUDITED | inotify/procfs, gravação agregada e inspeção/stop de listeners |
| Night light backend | services/SunsetService.qml | AUDITED | wlsunset, temperatura, toggle e falha |
| IPC handlers | Commands.qml | AUDITED | targets, verbs, validação, payloads e erros |
| velatura CLI | ~/.local/bin/velatura | AUDITED | parser, IPC bridge, watch, capture delegation e completion |
| Velatura Native bootstrap / CLI | src/main.rs, src/ui/bootstrap.rs, src/ui/gtk_event_bridge.rs, scripts/install-dev-cli.sh | IN PROGRESS | GTK4 lifecycle preservado; snapshot real e eventos chegam ao main context; matching é consultável no runtime; `ipc` pendente. A janela continua diagnóstico temporário; layer-shell, barra e superfícies não fazem parte desta fatia |
| Scripts Python ativos | scripts/*.py (28 arquivos) | AUDITED | 28/28 lidos integralmente; contratos em BACKEND_AUDIT.md e INTEGRATION_AUDIT.md |
| UI component inventory | UI_COMPONENT_CATALOG.md | AUDITED | 303 QML components indexed with service/state references, functions, signals and interaction handlers |

## Regras

- Atualizar estado e evidências ao terminar cada área.
- Não usar IMPLEMENTED quando só existe UI.
- VERIFIED exige teste real ou automatizado documentado.
- Recursos externos podem exigir token, portal, daemon ou permissão; documentar pré-requisitos.
- Itens bloqueados devem ter motivo, impacto e caminho de resolução.
- Um agrupamento pode ficar AUDITED somente quando todas as referências listadas na linha foram auditadas; subitens ainda pendentes devem aparecer explicitamente na referência/critério.

## Regra de paridade 1:1

Esta matriz acompanha `docs/PARITY_CONTRACT.md`. Todos os recursos do Velatura original devem ser portados integralmente. Não reduzir escopo, remover estados, trocar interações por placeholders nem marcar UI isolada como implementação funcional. Cada linha deve evoluir de NOT AUDITED para AUDITED, IN PROGRESS, IMPLEMENTED e VERIFIED com evidências; BLOCKED exige motivo e plano.
