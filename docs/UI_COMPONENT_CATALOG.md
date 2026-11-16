# Catálogo estático de componentes QML

**Fonte:** instalação ativa em ~/.config/quickshell.
**Data:** 15/11/2026.
**Uso:** inventário mecânico para localizar componentes e consumidores antes do porte. Não substitui UI_SURFACE_AUDIT.md, leitura semântica, screenshot ou teste de runtime.

| Arquivo | Linhas | Serviços/estados referenciados | Funções | Interações |
|---|---:|---|---|---|
| bar/island/AppearancePanel.qml | 434 | ThemeService, WallpaperService | settle, onWallpapersChanged, onAnimatedChanged, onChosenChanged, apply, turn, daubsOf; signals: closed, panelRequested, clicked | onActivated, onClicked, onPressed |
| bar/island/AuthPanel.qml | 263 | AccountService, PolkitService | answer, onFailedChanged | onAccepted, onClicked, onPressed, onReleased |
| bar/island/BluetoothDetail.qml | 255 | BluetoothService | signals: back | onClicked, onToggled |
| bar/island/BoardPanel.qml | 818 | NotesService, TasksService | back, laneCards, cardUnderCursor, lowest, walkLane, walkCard, openCurrent, aim (+3); signals: closed | onActiveChanged, onCentroidChanged, onClicked, onPressed, onTapped, onTextEdited |
| bar/island/BrightnessDetail.qml | 56 | BrightnessService | signals: back | onMoved |
| bar/island/CameraPanel.qml | 476 | CameraService, SettingsService | signals: closed | onClicked, onToggled |
| bar/island/ControlsPanel.qml | 269 | ControlsService | signals: closed, panelRequested, settingsRequested | onTapped |
| bar/island/DetailChoice.qml | 62 | — | signals: picked | onClicked |
| bar/island/DetailFace.qml | 91 | LyricsService, ModuleService | signals: closed | — |
| bar/island/DetailHeading.qml | 57 | — | signals: back | onClicked |
| bar/island/DetailSection.qml | 43 | — | — | — |
| bar/island/GamesPanel.qml | 392 | GamesService | play, again, onFinished, onScoreChanged, onBeatenChanged; signals: closed | onActivated, onClicked, onPressed |
| bar/island/IslandRest.qml | 457 | CavaService, MediaService, ModuleService, NiriService, PrivacyService, RecorderService, TimerService, VmService | — | onClicked |
| bar/island/IslandState.qml | 127 | NotificationService, OsdService | onRequested, open, close | onActiveChanged, onTriggered |
| bar/island/IslandSummary.qml | 637 | LyricsService, MediaService, ModuleService, PrivacyService, SettingsService, TimerService, WeatherService | press, title, value; signals: pressed | onClicked, onPressed |
| bar/island/KeysPanel.qml | 66 | ShortcutService | signals: closed | — |
| bar/island/LauncherPanel.qml | 573 | ClipboardService, EmojiService, LauncherService | move, kindColour, run, confirm, forgetSelected, onQueryChanged; signals: closed, panelRequested, settingsRequested | onClicked, onTextEdited |
| bar/island/LevelRow.qml | 102 | — | signals: moved, markClicked | onClicked, onMoved |
| bar/island/LyricsDetail.qml | 298 | LyricsService, MediaService | clock | onClicked, onPressed |
| bar/island/MachinesPanel.qml | 950 | VmService | gigabytes, cores; signals: closed, pressed, pressed, changed | onClicked, onPressed, onTextEdited |
| bar/island/MicrophoneDetail.qml | 86 | AudioService | signals: back | onMoved |
| bar/island/NetworkDetail.qml | 330 | NetworkService | onConnectionFinished; signals: back | onAccepted, onClicked, onToggled |
| bar/island/NightLightDetail.qml | 66 | SunsetService | signals: back | onMoved |
| bar/island/NotesPanel.qml | 555 | DesktopService, NotesService | back, openCurrent, archiveCurrent, placement, toTitle, follow; signals: closed | onClicked, onPressed, onTextEdited |
| bar/island/NotificationLayer.qml | 326 | NotificationService | — | onClicked |
| bar/island/OsdLayer.qml | 78 | — | — | — |
| bar/island/OverviewPanel.qml | 238 | NiriService | selectWorkspace, activateWorkspace, activateWindow, keepSelectionValid; signals: closed | onClicked, onDoubleClicked |
| bar/island/PackagesPanel.qml | 466 | PackagesService, UpdatesService | move, act, activateSelected; signals: closed | onClicked, onTextEdited |
| bar/island/PetPanel.qml | 217 | PetService | onPlayed, onCelebrated, onBrought; signals: closed | onClicked, onTextEdited |
| bar/island/ScreenTimePanel.qml | 808 | ScreenTimeService | dayName, bucketDay, label, describe; signals: closed | onClicked, onEntered, onExited |
| bar/island/ServersPanel.qml | 255 | OsdService, ServersService | signals: closed, pressed | onClicked, onPressed |
| bar/island/SessionPanel.qml | 158 | SessionService | move, activate; signals: closed | onClicked, onTriggered |
| bar/island/SoundDetail.qml | 88 | AudioService | signals: back | onMoved |
| bar/island/StatsPanel.qml | 283 | StatsService | signals: closed | — |
| bar/island/TrayMenuPanel.qml | 225 | TrayService | usable, move, run; signals: closed | onClicked |
| bar/island/controls/AppearanceCard.qml | 125 | ThemeService, WallpaperService | signals: requested | onClicked |
| bar/island/controls/Block.qml | 336 | ControlsService | aim; signals: panelRequested, settingsRequested, dismissed | onActiveChanged, onCentroidChanged, onTapped, onTriggered, onWheel |
| bar/island/controls/BlockFace.qml | 227 | AudioService, BrightnessService, ControlsService, NotesService | signals: panelRequested, settingsRequested, dismissed | onMoved, onTapped |
| bar/island/controls/BlockInspector.qml | 339 | ControlsService | — | onClicked, onTapped |
| bar/island/controls/ButtonsBlock.qml | 135 | ControlsService, SessionService | signals: panelRequested, settingsRequested, ran | onClicked, onTriggered |
| bar/island/controls/CalendarCard.qml | 413 | CalendarService, SettingsService, TasksService | signals: panelRequested | onClicked, onTapped, onWheel |
| bar/island/controls/ClockBlock.qml | 70 | SettingsService | — | — |
| bar/island/controls/ControlsTray.qml | 220 | ControlsService | smallest, aim | onActiveChanged, onCentroidChanged, onTapped |
| bar/island/controls/GamesBlock.qml | 129 | GamesService | signals: panelRequested | onClicked |
| bar/island/controls/GridSize.qml | 156 | ControlsService | within | onTapped |
| bar/island/controls/ImpastoBlock.qml | 336 | AccountService, MachineService, StatsService | — | — |
| bar/island/controls/MediaCard.qml | 175 | MediaService | clock | onClicked |
| bar/island/controls/NotificationList.qml | 239 | ModuleService, NotificationService | — | onClicked, onTapped |
| bar/island/controls/PetBlock.qml | 156 | PetService | signals: panelRequested | onClicked |
| bar/island/controls/TasksBlock.qml | 178 | TasksService | openOn; signals: panelRequested | onClicked, onTapped |
| bar/island/controls/TogglesBlock.qml | 276 | ControlsService, NetworkService, SystemService | spin, goTo, turn; signals: panelRequested, dismissed, stepped | onActiveChanged, onClicked, onToggled, onWheel |
| bar/island/controls/TopButtons.qml | 118 | ControlsService, SessionService | signals: ran, panelRequested, settingsRequested | onClicked, onTriggered |
| bar/island/controls/TopRowInspector.qml | 232 | ControlsService | — | onTapped |
| bar/island/controls/VelaturaBlock.qml | 336 | AccountService, MachineService, StatsService | — | — |
| bar/island/controls/WeatherCard.qml | 245 | WeatherService | — | — |
| bar/island/games/Bots.qml | 398 | — | restart, spawn, miss, bash, bashLane, bashAt, step, onBashed; signals: finished, bashed | onPressed, onTriggered |
| bar/island/games/Burst.qml | 77 | — | play | — |
| bar/island/games/Flood.qml | 249 | — | restart, apart, flood; signals: finished | onClicked, onPressed |
| bar/island/games/GameBoard.qml | 89 | GamesService | — | — |
| bar/island/games/Hextris.qml | 383 | — | restart, sideUnder, key, spawn, rotate, drop, matches, resolve (+5); signals: finished | onClicked, onPressed, onTriggered |
| bar/island/games/Lights.qml | 248 | — | restart, flipped, press; signals: finished | onClicked, onPressed |
| bar/island/games/Mini2048.qml | 269 | — | restart, spawn, slide, stuck, move, fill, ink; signals: finished | onPressed, onValueChanged |
| bar/island/games/Pop.qml | 53 | — | play | — |
| bar/island/games/Snake.qml | 378 | — | restart, occupied, placeApple, turn, step, onAte; signals: finished, ate | onPressed, onTriggered |
| bar/island/games/Solitaire.qml | 507 | — | restart, slotX, turned, isRed, select, isSelected, inHand, fitsColumn (+6); signals: finished | onClicked, onPressed |
| bar/island/games/Space.qml | 539 | — | restart, breedOf, colourOf, scatter, spawnWave, invaderX, invaderY, fire (+3); signals: finished | onPressed, onReleased, onTriggered |
| bar/island/games/Target.qml | 276 | — | restart, place, smash, miss; signals: finished | onPressed |
| bar/island/games/Tetris.qml | 486 | — | restart, deal, rotated, covered, fits, spawn, move, rotate (+7); signals: finished, swept | onPressed, onTriggered |
| bar/island/games/Whack.qml | 453 | — | restart, put, spawn, miss, whack, step, onStruck; signals: finished, struck | onPressed, onTriggered |
| bar/modules/BatteryModule.qml | 87 | BatteryService, ModuleService | — | — |
| bar/modules/BluetoothModule.qml | 93 | BluetoothService | — | onToggled |
| bar/modules/BrightnessModule.qml | 112 | BrightnessService | — | onMoved |
| bar/modules/CalendarModule.qml | 383 | CalendarService, ModuleService, TasksService | keyOf | onClicked |
| bar/modules/ChipFace.qml | 170 | ModuleService | — | — |
| bar/modules/ClaudeModule.qml | 138 | ClaudeService | — | — |
| bar/modules/ClockModule.qml | 71 | SettingsService | — | — |
| bar/modules/CodexModule.qml | 100 | CodexService | — | — |
| bar/modules/GamesModule.qml | 117 | GamesService, ModuleService | — | onClicked |
| bar/modules/GithubModule.qml | 76 | GithubService | — | — |
| bar/modules/MarketsModule.qml | 140 | MarketsService | — | — |
| bar/modules/MediaModule.qml | 296 | CavaService, LyricsService, MediaService | clock | onClicked, onPressed |
| bar/modules/Module.qml | 84 | ModuleService | — | — |
| bar/modules/NetworkModule.qml | 95 | NetworkService | — | onToggled |
| bar/modules/NotesModule.qml | 157 | ModuleService, NotesService | openOn | onClicked |
| bar/modules/NotificationsModule.qml | 249 | ModuleService, NotificationService | — | onClicked, onTapped |
| bar/modules/PetModule.qml | 167 | PetService | onPlayed, onCelebrated, onBrought | onClicked |
| bar/modules/ScreenTimeModule.qml | 92 | ModuleService, ScreenTimeService | — | onClicked |
| bar/modules/StatsModule.qml | 150 | StatsService | — | — |
| bar/modules/TasksModule.qml | 111 | ModuleService, TasksService | openOn | onClicked |
| bar/modules/TimerModule.qml | 198 | TimerService | — | onClicked |
| bar/modules/UpdatesModule.qml | 118 | ModuleService, PackagesService, UpdatesService | — | onClicked |
| bar/modules/VolumeModule.qml | 157 | AudioService | — | onClicked, onMoved |
| bar/modules/WeatherModule.qml | 128 | WeatherService | — | — |
| bar/widgets/BarChip.qml | 120 | ModuleService, SettingsService | — | onClicked |
| bar/widgets/BarZone.qml | 238 | ModuleService, SettingsService, TrayService | — | — |
| bar/widgets/BatteryWidget.qml | 48 | BatteryService | — | — |
| bar/widgets/DynamicIsland.qml | 760 | AudioService, BrightnessService, CameraService, ControlsService, GamesService, IslandState, LauncherService, ModuleService, NiriService, NotesService, NotificationService, PackagesService, PetService, ServersService, SettingsService, ShortcutService, SunsetService, TasksService, TrayService | panelSizeFor, layerSizeFor, open, close, toggle, leaveAllowed, onHoveredChanged, onOpenPanelChanged (+4); signals: settingsRequested | onClicked, onTriggered |
| bar/widgets/TimerWidget.qml | 47 | TimerService | — | — |
| bar/widgets/TrayWidget.qml | 152 | ModuleService, TrayService | — | onClicked, onWheel |
| bar/widgets/WorkspacesWidget.qml | 268 | NiriService, SettingsService | glyphOf | onClicked |
| capture/CaptureBar.qml | 134 | CaptureService, RecorderService, SettingsService | — | onPressed |
| capture/CaptureOverlay.qml | 305 | CaptureService | onOpened, take | onPressed, onReleased |
| components/Avatar.qml | 61 | AccountService | — | — |
| components/BarPreview.qml | 138 | SettingsService | — | — |
| components/BezelOutline.qml | 122 | — | — | — |
| components/BezelShape.qml | 98 | — | — | — |
| components/Card.qml | 37 | — | — | — |
| components/CardChoice.qml | 76 | — | signals: picked | onClicked |
| components/CardFact.qml | 45 | — | — | — |
| components/CardHeading.qml | 27 | — | — | — |
| components/CardLevel.qml | 143 | — | signals: moved, markClicked | onClicked, onMoved, onWheel |
| components/CardLimit.qml | 91 | — | — | — |
| components/Carousel.qml | 127 | — | offsetOf, scaleOf, goTo, step, press, turn; signals: activated | onActiveChanged, onWheel |
| components/CarouselTile.qml | 47 | — | — | onClicked |
| components/ClaudeMark.qml | 83 | — | — | — |
| components/CodexMark.qml | 73 | — | — | — |
| components/ColorSwatch.qml | 47 | — | — | — |
| components/ContributionGrid.qml | 97 | — | — | — |
| components/CornerGuard.qml | 23 | — | — | — |
| components/DayDial.qml | 163 | — | angleOf | onTriggered |
| components/DayPicker.qml | 258 | TasksService | signals: picked, dismissed | onClicked, onTapped, onWheel |
| components/EditTray.qml | 632 | — | span, packGroups, pack, spotFor, clampX, clampY, scrollTo; signals: moved, resized | onActiveChanged, onCentroidChanged, onTapped, onWheel |
| components/Figure.qml | 55 | — | — | — |
| components/GlassSheen.qml | 93 | — | — | — |
| components/GreetingPreview.qml | 62 | SettingsService, ThemeService | — | onTriggered |
| components/GroupHeading.qml | 84 | — | — | onClicked |
| components/IconButton.qml | 57 | — | signals: clicked | onClicked |
| components/IslandMotionPreview.qml | 66 | — | — | onTriggered |
| components/LidPreview.qml | 94 | — | — | — |
| components/LitEdge.qml | 82 | — | — | — |
| components/ModuleCard.qml | 91 | — | — | — |
| components/MotionPreview.qml | 94 | — | — | onTriggered |
| components/NiriModButton.qml | 23 | — | — | onClicked |
| components/NotchFillet.qml | 95 | — | — | — |
| components/NotchOutline.qml | 88 | — | — | — |
| components/PaletteBoard.qml | 34 | — | — | — |
| components/PetFace.qml | 136 | PetService, SettingsService | — | — |
| components/PetFamily.qml | 207 | PetService | — | onClicked, onTextEdited |
| components/PetShelf.qml | 95 | PetService | — | onClicked |
| components/PillButton.qml | 79 | — | signals: clicked | onClicked |
| components/PopMenu.qml | 107 | — | signals: chosen | onClicked |
| components/PreviewTile.qml | 81 | — | signals: picked | onClicked |
| components/QuickTile.qml | 138 | — | signals: toggled, expanded | onClicked |
| components/RingIndicator.qml | 89 | — | — | — |
| components/ScreenCorner.qml | 47 | — | — | — |
| components/SegmentedControl.qml | 142 | — | signals: selected | onClicked |
| components/SessionRibbon.qml | 59 | — | — | — |
| components/SettingBlock.qml | 38 | — | — | — |
| components/SettingDivider.qml | 22 | — | — | — |
| components/SettingField.qml | 94 | — | signals: edited | onTextEdited |
| components/SettingGroup.qml | 56 | — | — | — |
| components/SettingLabel.qml | 60 | — | — | — |
| components/SettingRow.qml | 74 | — | — | — |
| components/SettingSlider.qml | 248 | — | commit, nudge; signals: moved | onClicked, onEditingFinished, onMoved, onWheel |
| components/SettingTiles.qml | 65 | — | — | — |
| components/SettingsHero.qml | 41 | — | — | — |
| components/SliderRow.qml | 136 | — | signals: moved, iconClicked, expandClicked | onClicked, onMoved |
| components/Sparkline.qml | 82 | — | pointAt | — |
| components/Spectrum.qml | 73 | CavaService | — | — |
| components/StatCard.qml | 100 | — | — | — |
| components/Stepper.qml | 82 | — | step; signals: changed | onClicked |
| components/Sticky.qml | 96 | NotesService | — | — |
| components/TabStrip.qml | 82 | — | signals: picked | onClicked |
| components/TaskRow.qml | 128 | TasksService | signals: opened | onTapped |
| components/ToggleSwitch.qml | 58 | — | signals: toggled | onClicked |
| components/TopRowEditor.qml | 291 | ControlsService | entryOf, aim, drop | onPressed, onReleased |
| components/TrayIcon.qml | 55 | — | — | — |
| components/UsageBar.qml | 43 | — | — | — |
| components/UsageBars.qml | 102 | ScreenTimeService | — | — |
| components/UsageRing.qml | 89 | — | — | — |
| components/WallpaperTransitionPreview.qml | 209 | WallpaperService | frame, halfPlane, circle, wave, outline, hole | — |
| components/WindowsPreview.qml | 232 | — | — | — |
| components/pets/PetCreature.qml | 426 | — | — | — |
| components/pets/PetEgg.qml | 99 | — | — | — |
| components/pets/PetPaper.qml | 355 | — | — | — |
| components/pets/PetPixel.qml | 269 | — | runsOf, inkOf | — |
| components/pets/PetPlush.qml | 286 | — | — | — |
| components/pets/PetSoftFace.qml | 205 | — | — | — |
| deck/Deck.qml | 800 | DeckService, DesktopService, ModuleService, NotesService | boxFor, startOf, syncNotes, aim, drop, openNote | onActiveChanged, onCentroidChanged, onPressed, onTapped, onTriggered |
| desktop/Desktop.qml | 529 | DeckService, DesktopService, ModuleService, MonitorService, NotesService, SettingsService, WallpaperService | about, onAfterSynchronizing, onFrameSwapped | onTapped |
| desktop/EdgeSpectrum.qml | 182 | DeckService, DesktopService | pointer, aim | onActiveChanged, onCentroidChanged, onTapped |
| desktop/Face.qml | 93 | DesktopService | — | — |
| desktop/Inspector.qml | 1196 | DesktopService, ModuleService, NotesService, SettingsService | onTypingChanged, choose; signals: chosen | onClicked, onMoved, onTapped, onTextEdited, onToggled |
| desktop/Picker.qml | 391 | DesktopService | local, parentOf, choose | onClicked, onTapped |
| desktop/SpectrumBars.qml | 133 | CavaService, DesktopService | four, listen | — |
| desktop/ThemeSwatch.qml | 53 | DesktopService | — | — |
| desktop/Tray.qml | 249 | DeckService, DesktopService | smallest, aim | onActiveChanged, onCentroidChanged, onMoved, onTapped |
| desktop/Widget.qml | 587 | DeckService, DesktopService, ModuleService | rewatch, edgeUnder, aim, onCentroidChanged | onActiveChanged, onCentroidChanged, onTapped, onTriggered, onWheel |
| desktop/faces/Bands.qml | 337 | DesktopService, MarketsService, ScreenTimeService, SettingsService, WeatherService | — | — |
| desktop/faces/DayTasks.qml | 142 | ModuleService, TasksService | signals: back | onTapped |
| desktop/faces/GithubFace.qml | 52 | DesktopService, GithubService, SettingsService | — | — |
| desktop/faces/Larges.qml | 1006 | ClaudeService, CodexService, DesktopService, MarketsService, MediaService, ModuleService, ScreenTimeService, StatsService, TasksService, WeatherService | — | onClicked, onTapped |
| desktop/faces/LyricsFace.qml | 119 | DesktopService, LyricsService, MediaService | — | — |
| desktop/faces/NoteFace.qml | 61 | DesktopService, ModuleService, NotesService | — | onTapped |
| desktop/faces/PhotoFace.qml | 118 | DesktopService | — | onTapped |
| desktop/faces/SpectrumFace.qml | 49 | DesktopService | — | — |
| desktop/faces/Squares.qml | 616 | AudioService, BatteryService, BluetoothService, BrightnessService, ClaudeService, CodexService, DesktopService, GamesService, MarketsService, MediaService, NetworkService, PetService, ScreenTimeService, SettingsService, StatsService, TasksService, TimerService, UpdatesService, WeatherService | — | onTapped |
| desktop/faces/Wides.qml | 1127 | AudioService, BatteryService, BluetoothService, BrightnessService, ClaudeService, CodexService, DesktopService, MarketsService, MediaService, ModuleService, NetworkService, PetService, ScreenTimeService, SettingsService, StatsService, TasksService, TimerService, UpdatesService, WeatherService | — | onClicked, onTapped |
| desktop/faces/WidgetFace.qml | 164 | DesktopService | — | — |
| desktop/faces/analogue/Analogue.qml | 78 | DesktopService | — | — |
| desktop/faces/analogue/Arcs.qml | 131 | DesktopService | — | — |
| desktop/faces/analogue/BatteryCell.qml | 92 | DesktopService | — | — |
| desktop/faces/analogue/BatteryFace.qml | 35 | BatteryService | — | — |
| desktop/faces/analogue/BluetoothFace.qml | 63 | BluetoothService | — | — |
| desktop/faces/analogue/BrightnessFace.qml | 42 | BrightnessService | — | — |
| desktop/faces/analogue/CalendarFace.qml | 454 | DesktopService, ModuleService, TasksService | agendaLine | onTapped |
| desktop/faces/analogue/ClaudeFace.qml | 77 | ClaudeService | — | — |
| desktop/faces/analogue/Clipboard.qml | 93 | DesktopService, ModuleService, TasksService | — | — |
| desktop/faces/analogue/ClockFace.qml | 63 | DesktopService | — | — |
| desktop/faces/analogue/CodexFace.qml | 69 | CodexService | — | — |
| desktop/faces/analogue/CreatureFace.qml | 72 | PetService | — | — |
| desktop/faces/analogue/Dial.qml | 175 | DesktopService, SettingsService | — | — |
| desktop/faces/analogue/Fader.qml | 97 | DesktopService | — | — |
| desktop/faces/analogue/GamesFace.qml | 42 | GamesService, ModuleService | — | onClicked |
| desktop/faces/analogue/Gauge.qml | 169 | DesktopService | — | — |
| desktop/faces/analogue/GithubFace.qml | 50 | DesktopService, GithubService, SettingsService | — | — |
| desktop/faces/analogue/Hourglass.qml | 131 | DesktopService | halfAt | — |
| desktop/faces/analogue/Instrument.qml | 137 | DesktopService | — | — |
| desktop/faces/analogue/Joystick.qml | 72 | DesktopService | — | — |
| desktop/faces/analogue/Knob.qml | 104 | DesktopService | — | — |
| desktop/faces/analogue/Leaf.qml | 125 | DesktopService | — | — |
| desktop/faces/analogue/MarketsFace.qml | 44 | DesktopService, MarketsService | — | — |
| desktop/faces/analogue/MediaFace.qml | 135 | MediaService | — | onClicked |
| desktop/faces/analogue/NetworkFace.qml | 32 | NetworkService | — | — |
| desktop/faces/analogue/Paper.qml | 33 | — | — | — |
| desktop/faces/analogue/Parcel.qml | 120 | DesktopService | — | — |
| desktop/faces/analogue/PhotoFace.qml | 252 | DesktopService | frame | onTapped |
| desktop/faces/analogue/Record.qml | 144 | DesktopService | — | — |
| desktop/faces/analogue/Recorder.qml | 149 | DesktopService | — | — |
| desktop/faces/analogue/ScreenTimeFace.qml | 64 | DesktopService, ScreenTimeService | — | — |
| desktop/faces/analogue/Sky.qml | 268 | DesktopService | — | — |
| desktop/faces/analogue/StatsFace.qml | 179 | DesktopService, StatsService | — | — |
| desktop/faces/analogue/TasksFace.qml | 84 | DesktopService, ModuleService, TasksService | — | onClicked |
| desktop/faces/analogue/Thermometer.qml | 92 | DesktopService | — | — |
| desktop/faces/analogue/TimerFace.qml | 70 | TimerService | — | onClicked |
| desktop/faces/analogue/UpdatesFace.qml | 64 | UpdatesService | — | — |
| desktop/faces/analogue/VolumeFace.qml | 44 | AudioService | — | — |
| desktop/faces/analogue/WeatherFace.qml | 230 | WeatherService | px, py | — |
| desktop/faces/sticker/BatteryFace.qml | 73 | BatteryService | — | — |
| desktop/faces/sticker/BluetoothFace.qml | 61 | BluetoothService | — | — |
| desktop/faces/sticker/BrightnessFace.qml | 53 | BrightnessService | — | — |
| desktop/faces/sticker/CalendarFace.qml | 315 | DesktopService, TasksService | agendaLine | onTapped |
| desktop/faces/sticker/ClaudeFace.qml | 75 | ClaudeService | — | — |
| desktop/faces/sticker/ClockFace.qml | 199 | DesktopService, SettingsService | — | — |
| desktop/faces/sticker/CodexFace.qml | 72 | CodexService | — | — |
| desktop/faces/sticker/CreatureFace.qml | 67 | PetService | — | — |
| desktop/faces/sticker/Cut.qml | 99 | — | — | — |
| desktop/faces/sticker/Dot.qml | 49 | — | signals: clicked | onTapped |
| desktop/faces/sticker/GamesFace.qml | 58 | GamesService, ModuleService | — | onClicked |
| desktop/faces/sticker/GithubFace.qml | 61 | DesktopService, GithubService, SettingsService | — | — |
| desktop/faces/sticker/MarketsFace.qml | 67 | MarketsService | — | — |
| desktop/faces/sticker/MediaFace.qml | 182 | DesktopService, MediaService | — | onClicked |
| desktop/faces/sticker/NetworkFace.qml | 47 | NetworkService | — | — |
| desktop/faces/sticker/PhotoFace.qml | 94 | DesktopService | — | — |
| desktop/faces/sticker/Ring.qml | 60 | — | — | — |
| desktop/faces/sticker/ScreenTimeFace.qml | 56 | ScreenTimeService | — | — |
| desktop/faces/sticker/Spread.qml | 97 | DesktopService | — | — |
| desktop/faces/sticker/StatsFace.qml | 104 | DesktopService, StatsService | — | — |
| desktop/faces/sticker/Sticker.qml | 78 | DesktopService | — | — |
| desktop/faces/sticker/Tag.qml | 79 | — | — | — |
| desktop/faces/sticker/TasksFace.qml | 155 | DesktopService, ModuleService, TasksService | — | onTapped |
| desktop/faces/sticker/TimerFace.qml | 91 | TimerService | — | onClicked |
| desktop/faces/sticker/Tone.qml | 30 | — | mix | — |
| desktop/faces/sticker/UpdatesFace.qml | 63 | UpdatesService | — | — |
| desktop/faces/sticker/VolumeFace.qml | 60 | AudioService | — | — |
| desktop/faces/sticker/WeatherFace.qml | 138 | WeatherService | — | — |
| dock/Dock.qml | 670 | DesktopService, DockService, SettingsService | sizeOf, slotOf, openMenu, closeMenu, alongOf, scaleOf, onHoveredIndexChanged; signals: launcherRequested | onPressed, onTriggered |
| dock/DockItem.qml | 277 | DockService, SettingsService | track; signals: menuRequested | onActiveChanged |
| dock/DockMenu.qml | 238 | DockService, NiriService | signals: closed | onClicked |
| lock/FaceRing.qml | 107 | — | — | — |
| lock/FlipCard.qml | 166 | — | — | onValueChanged |
| lock/LockAccount.qml | 323 | AccountService, LockService | claim, clear, onFailedChanged, onTypedChanged; signals: submitted | onAccepted, onClicked, onPressed |
| lock/LockClock.qml | 134 | SettingsService | — | — |
| lock/LockIsland.qml | 191 | LockService, ModuleService, SettingsService | onFaceMissed | onTriggered |
| lock/LockMedia.qml | 302 | LyricsService, MediaService | clock | onClicked |
| lock/LockPower.qml | 121 | SessionService | — | onClicked, onTriggered |
| lock/LockScreen.qml | 48 | LockService | — | — |
| lock/LockSurface.qml | 303 | BatteryService, LockService, MediaService, SettingsService | claim, onAwakeChanged; signals: submitted | onTapped |
| lock/Padlock.qml | 68 | — | — | — |
| settings/AppearanceSection.qml | 282 | FontService, SettingsService, ThemeService, WallpaperService | signals: panelRequested | onClicked, onToggled |
| settings/BarSection.qml | 203 | NiriService, SettingsService | — | onMoved, onToggled |
| settings/ControlsSection.qml | 82 | ControlsService | signals: arranging | onClicked |
| settings/DockSection.qml | 257 | SettingsService | — | onMoved, onToggled |
| settings/FontPicker.qml | 134 | — | signals: picked | onClicked |
| settings/GroundRow.qml | 47 | SettingsService | — | — |
| settings/InputSection.qml | 48 | CompositorService | — | — |
| settings/IslandSection.qml | 245 | ModuleService, NotificationService, SettingsService | — | onClicked, onMoved, onToggled |
| settings/KeptApplications.qml | 287 | DockService, LauncherService | — | onClicked |
| settings/KeysSection.qml | 45 | ShortcutService | — | — |
| settings/LauncherSection.qml | 302 | LauncherService, SettingsService | accepts | onEditingFinished, onMoved, onTextEdited, onToggled |
| settings/LayoutEditor.qml | 1159 | ModuleService, SettingsService, TrayService | pick, unpick, setLook, removePicked, listOf, shown, groupsOf, nameOf (+6) | onClicked, onPressed, onReleased |
| settings/ModulesPart.qml | 143 | SettingsService | — | — |
| settings/ModulesSection.qml | 153 | GithubService, MarketsService, PetService, SettingsService, WeatherService | — | onToggled |
| settings/MonitorsSection.qml | 67 | MonitorService | — | — |
| settings/ProfilesPart.qml | 422 | ProfileService | summary, local, rename, onRenamingChanged, exportProfile | onAccepted, onClicked, onEditingFinished |
| settings/SessionSection.qml | 715 | AccountService, AccountsService, FaceService, MediaService, SettingsService, WallpaperService | — | onAccepted, onClicked, onMoved |
| settings/SettingsPanel.qml | 596 | — | firstTab, go, show, back, forward, matches; signals: closed, panelRequested | onClicked, onTextEdited |
| settings/SettingsSection.qml | 23 | — | — | — |
| settings/SettingsWindow.qml | 66 | — | open, close, toggle; signals: panelRequested | — |
| settings/SystemSection.qml | 570 | SettingsService, StatsService, VersionService | describe, spell | onClicked |
| settings/WidgetsSection.qml | 139 | DesktopService, MonitorService, SettingsService | signals: arranging | onClicked, onMoved, onToggled |
| settings/WindowsSection.qml | 77 | CompositorService | — | — |
| settings/WorkspaceStyles.qml | 57 | SettingsService | — | — |

Total: 303 arquivos QML ativos catalogados nas famílias visuais auditadas.

## Regras de interpretação

- Uma referência textual a XService não prova que o serviço seja iniciado; conferir lifecycle e subscriptions no código.
- Um handler não prova que o input seja alcançável em runtime; layer, mask, focus e Loader também importam.
- Arquivos .new, .bak e .before-* foram excluídos da fonte primária.
- Para a semântica e os contratos por superfície, usar UI_SURFACE_AUDIT.md; para backend/serviços, BACKEND_AUDIT.md e INTEGRATION_AUDIT.md.
