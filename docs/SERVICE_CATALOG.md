# Catálogo técnico dos serviços Velatura

Fonte: arquivos ativos em services/*.qml lidos diretamente em 15/11/2026. Backups, .new e .before-* foram excluídos do inventário de runtime.

> Este é um inventário estático rastreável, não uma certificação de que cada integração foi testada em execução. Antes de portar um serviço, ler o QML inteiro, os scripts chamados e os consumidores.

Serviços ativos identificados: 61. Arquivos QML candidatos a consumidor examinados: 385.

## Como ler

- Responsabilidade: resumo do domínio inferido do código e dos consumidores.
- Imports: módulos QML e APIs Qt/Quickshell.
- Funções/propriedades: símbolos encontrados estaticamente; não são ainda o contrato Rust.
- Mecanismos: processos, timers, adaptadores ou APIs nativas.
- Comandos: literais em Process.command; comandos dinâmicos e scripts podem não aparecer.
- Consumidores: arquivos QML ativos que referenciam o singleton.

## Compositor, displays e superfícies

### CompositorService
Detecção de versão e configuração do compositor Niri.

- Arquivo: services/CompositorService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process
- Funções: load
- Propriedades: configPath, version, lastError, loaded, versionReader
- Comandos estáticos: "niri", "--version"
- Consumidores QML (3): settings/InputSection.qml, settings/WindowsSection.qml, theme/Theme.qml

### MonitorService
Inventário de outputs, monitor primário/interno e refresh/hotplug.

- Arquivo: services/MonitorService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: load, monitorFor, descriptionFor, isPrimary, onEventReceived, onScreensChanged, onGeometryChanged
- Propriedades: script, monitors, loaded, refreshPending, lastError, reader, primaryName, effectivePrimaryName, primaryScreen, internalName, refreshDebounce, workspaceEvents, screenEvents
- Comandos estáticos: "python3", root.script, "get"
- Consumidores QML (6): shell.qml, desktop/Desktop.qml, services/DesktopService.qml, services/CaptureService.qml, settings/MonitorsSection.qml, settings/WidgetsSection.qml

### NiriService
Niri IPC/event-stream, workspaces, janelas, foco, ações e fila de comandos.

- Arquivo: services/NiriService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: activeOn, numberOn, isOccupiedOn, isOccupied, isVisibleOn, focus, parseEvent, updateWorkspaceFocus, updateFocusedWindow, workspaceById, windowsOn, clientsOn, runAction, runActions, startNextAction, focusWorkspace, focusWorkspaceOn, focusWindow, closeWindow, toggleFloating
- Propriedades: workspaces, windows, focusedWorkspaceId, activeWorkspaceId, focusedWindow, connected, lastError, actionQueue, clients, focusedAddress, occupiedIds, occupiedWorkspaceIds, focusedWorkspace, focusedOutput, activeId, maximum, span, actionRunner, reconnect, eventStream
- Comandos estáticos: "niri", "msg", "--json", "event-stream"
- Consumidores QML (18): shell.qml, dock/DockMenu.qml, services/DesktopService.qml, services/DockService.qml, services/LauncherService.qml, services/MonitorService.qml, services/BrightnessService.qml, services/CaptureService.qml, services/DeckService.qml, services/IdleService.qml, services/RecorderService.qml, services/ScreenTimeService.qml, services/SessionService.qml, settings/BarSection.qml, bar/island/OverviewPanel.qml, bar/island/IslandRest.qml (lista parcial; buscar pelo singleton para obter todos)

### IdleService
Tempos de inatividade para bloquear, apagar tela e suspender.

- Arquivo: services/IdleService.qml
- Imports: QtQuick, Quickshell, Quickshell.Wayland
- Mecanismos: não detectados
- Funções: screen
- Propriedades: lockAfter, screenAfter, suspendAfter, lockWatch, screenWatch, sleepWatch
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (3): shell.qml, services/SettingsService.qml, services/ScreenTimeService.qml

### SessionService
Ações de sessão, suspensão, inibição de sleep e sinais logind.

- Arquivo: services/SessionService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer, D-Bus
- Funções: onSecureChanged, sleeping, run, exec
- Propriedades: actions, runner, suspendWhenLocked, suspendGiveUp, lockWatch, sleepPending, sleepHold, sleepWatch
- Comandos estáticos: "systemd-inhibit", "--what=sleep", "--mode=delay", "--who=velatura", "--why=Locking the screen first", "cat"; "dbus-monitor", "--system", "type='signal',interface='org.freedesktop.login1.Manager',member='PrepareForSleep'"
- Consumidores QML (7): shell.qml, lock/LockPower.qml, services/ControlsService.qml, services/IdleService.qml, bar/island/SessionPanel.qml, bar/island/controls/ButtonsBlock.qml, bar/island/controls/TopButtons.qml

### LockService
Ciclo de bloqueio, autenticação PAM, captura opcional e limpeza de imagem temporária.

- Arquivo: services/LockService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, Quickshell.Services.Pam, "../theme"
- Mecanismos: Process, Timer, PAM
- Funções: rouse, rest, shotPath, shotSource, lock, attemptCapture, beginCapture, captureScript, forget, begin, submit, wake, scan, release
- Propriedades: locked, secure, authenticating, message, failed, leaving, awake, typed, awakeFor, drowse, shotDirectory, shotSerial, shotReady, shellQuiet, pendingCapture, settleFrame, quietGiveUp, capture, eraser, pam, facePam, faceReady, faceScanning, faceMatched, faceRest, faceIdle, faceTries, faceMisses, faceQuietUntil, faceHold, faceCheck, face, faceHoldTimer, leave
- Comandos estáticos: "sh", "-c", `rm -f '${root.shotDirectory}'/lock-*.jpg`; "sh", "-c", "test -f /etc/pam.d/velatura-face && test -f /usr/lib/security/pam_howdy.so"
- Consumidores QML (11): shell.qml, Commands.qml, bar/Bar.qml, lock/LockAccount.qml, lock/LockIsland.qml, lock/LockScreen.qml, lock/LockSurface.qml, services/IdleService.qml, services/PetService.qml, services/ScreenTimeService.qml, services/SessionService.qml

### PolkitService
Agente de autenticação PolicyKit e fluxo de senha/solicitação.

- Arquivo: services/PolkitService.qml
- Imports: QtQuick, Quickshell, Quickshell.Services.Polkit
- Mecanismos: Timer, Polkit, execDetached
- Funções: onAuthenticationFailed, submit, cancel
- Propriedades: agent, flow, asking, registered, failed, fallback, tries, tried, watch
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (2): shell.qml, bar/island/AuthPanel.qml

### ShortcutService
Catálogo de atalhos, conflitos e edição/visualização de bindings.

- Arquivo: services/ShortcutService.qml
- Imports: QtQuick, Quickshell
- Mecanismos: não detectados
- Funções: current, fixed, clash, rebind, sheetFind
- Propriedades: catalogue, keys, table, index, compositorBindingsEditable, configPath, sheet, sheetCount, sheetWidth, sheetHeight, sheetFieldHeight, sheetRowHeight, sheetHeadingHeight
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (4): services/SettingsService.qml, settings/KeysSection.qml, bar/island/KeysPanel.qml, bar/widgets/DynamicIsland.qml

## Preferências, aparência e personalização

### SettingsService
Schema completo de preferências, defaults, layouts, atalhos e persistência JSON.

- Arquivo: services/SettingsService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: FileView, JsonAdapter, Timer
- Funções: launcherPrefix, setLauncherPrefix, barItems, barZone, onBar, tidy, setBarZone, welcomeActivities, beside, setBeside, set, shapeOf, accepts, copy, complete, snapshot, adopt, reset, machineLayout
- Propriedades: stateDirectory, clockFormat, clockShowsDate, clockShowsSeconds, barHeight, barMargin, islandAttached, barSideMargin, barStyle, barEverywhere, barHidden, screenCorners, gameMode, barLeft, barRight, islandSummary, lyrics, islandActivities, chipShape, chipFigure, desktopWidgets, desktopTheme, desktopOpacity, desktopHidden, centreButtons, centreTop, centreBlocks, centreColumns, centreRows, centreToggles, dockEnabled, dockPinned, dockEdge, dockAlignment, dockIconSize, dockMagnify, dockOpacity, dockGround, terminalGround, dockRunning, dockEverywhere, dockAutohide, dockLauncher, weatherPlace, githubUser, markets, petStyle, launcherResults, launcherOrder, launcherFits, emojiTone, cameraEnabled, cameraEffect, cameraReactions, clipboardHistory, clipboardKeep, clipboardImages, clipboardWipeOnLock, lockBlur, lockClock, lockMusic, lockMusicGround, userName, userAvatar, doNotDisturb, recorderAudio, captureShape, captureKind, notesHandwriting, deckOnEmpty, spectrumOnEmpty, workspaceCount, workspaceMax, workspaceMode, workspaceStyle, motionScale, motionCurve, animationPreset, windowShadow, barShadow, dockShadow, widgetShadow, windowGlass, surfaceStyle, desktopGround, barSides, wallpaperTransition, wallpaperDir, matugenEnabled, greeting, fontFamily, fontMono, notificationTimeout, compositor, keyboard, displays, keys, launcherPrefixes, cursorColor, cursorSize, shakeToFind, nightLight, nightTemperature, lidPolicy, idleLock, idleScreen, idleSuspend, language, launcherPrefixDefaults, clockFormats, barStyles, notched, barDefaults, besideDefaults, besideChoices, chipShapes, workspaceStyles, chipFigures, arrived, fresh, machineKeys, pristine, storedKeys, profileKeys, machineKeyboard, file, saver, islandActivitiesSeen
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (78): shell.qml, Commands.qml, ScreenCorners.qml, bar/Bar.qml, bar/BarReserve.qml, capture/CaptureBar.qml, components/BarPreview.qml, components/GreetingPreview.qml, components/PetFace.qml, desktop/Desktop.qml, desktop/Inspector.qml, dock/Dock.qml, dock/DockItem.qml, lock/LockClock.qml, lock/LockIsland.qml, lock/LockSurface.qml (lista parcial; buscar pelo singleton para obter todos)

### ThemeService
Temas, Matugen, paletas adaptativas, terminal, fonte e saudação.

- Arquivo: services/ThemeService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, FileView
- Funções: pushTerminalFont, pushTerminalGround, pushTerminalSettings, onArrivedChanged, pushGreeting, pushTerminal, parseJson, reloadAdaptiveColors, setTheme
- Propriedades: script, activeId, dynamicColors, matugenPaletteFile, adaptiveTheme, availableThemes, adaptiveSwatches, stateProcess, extractProcess, persistProcess, terminalProcess, terminalFont, fontAgain, fontProcess, terminalGround, groundAgain, groundProcess, settingsArrival, greetings, greetingScript, greeting, greetingPending, greetingProcess, greetingStamp, greetingStampView
- Comandos estáticos: root.script, "get-state"; root.script, "extract-colors"
- Consumidores QML (10): shell.qml, Commands.qml, components/GreetingPreview.qml, services/SettingsService.qml, services/WallpaperService.qml, services/ProfileService.qml, settings/AppearanceSection.qml, theme/Palettes.qml, bar/island/AppearancePanel.qml, bar/island/controls/AppearanceCard.qml

### ProfileService
Perfis de configuração, exportação/importação, duplicação e wallpaper/paleta associados.

- Arquivo: services/ProfileService.qml
- Imports: QtQuick, Qt.labs.folderlistmodel, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, FileView, JsonAdapter, Timer
- Funções: entry, settingsOf, wallpaperOf, paletteOf, cleanName, uniqueName, newId, create, duplicate, rename, remove, stash, switchTo, showWallpaper, tilde, resolveWallpaper, exportTo, importFrom, receive, parse, fromDocument, offerShipped, onArrivedChanged, normalise, arrive, save
- Propriedades: profiles, active, arrived, format, nameLength, current, wallpaperCheck, wanted, home, exporter, name, importer, shippedDirectory, shipped, offered, shippedFiles, shippedReaders, counted, shippedRead, placeholder, settingsArrive, saver, file
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (2): services/SettingsService.qml, settings/ProfilesPart.qml

### FontService
Descoberta de fontes instaladas e metadados para o seletor.

- Arquivo: services/FontService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process
- Funções: load
- Propriedades: sans, mono, loaded, query
- Comandos estáticos: Quickshell.shellPath("scripts/fonts.py")
- Consumidores QML (1): settings/AppearanceSection.qml

### WallpaperService
Biblioteca de wallpapers, awww, transições, wallpapers animados e Matugen.

- Arquivo: services/WallpaperService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: isMotion, onScreensChanged, onWallpaperDirChanged, onMatugenEnabledChanged, parseJson, scan, transitionType, apply
- Propriedades: script, awwwOwned, awwwGuard, awwwDaemon, wallpapers, animated, currentWallpaper, currentMotion, scanning, chosen, motionExtensions, scriptEnvironment, scanProcess, animatedProcess, currentProcess, asked, applyProcess, restoreProcess, restoreRetry, tries, hotplug, arriving, folder, relist, matugenProcess, matugenSetting, transitions
- Comandos estáticos: root.script, "awww-managed"; "awww-daemon"; root.script, "list-wallpapers"; root.script, "list-animated"; root.script, "get-state"; root.script, "restore"; root.script, "sync-matugen"
- Consumidores QML (10): shell.qml, Commands.qml, components/WallpaperTransitionPreview.qml, desktop/Desktop.qml, services/SettingsService.qml, services/ProfileService.qml, settings/AppearanceSection.qml, settings/SessionSection.qml, bar/island/AppearancePanel.qml, bar/island/controls/AppearanceCard.qml

### SunsetService
Night light/temperatura de cor via wlsunset.

- Arquivo: services/SunsetService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer
- Funções: setTemperature, toggle
- Propriedades: warmest, coolest, icon, temperature, available, restarting, on, detail, presets, panelWidth, panelHeight, probe, filter, retune
- Comandos estáticos: "sh", "-c", "command -v wlsunset"; "wlsunset", "-T", String(root.temperature), "-t", String(root.temperature)
- Consumidores QML (6): shell.qml, Commands.qml, services/ControlsService.qml, services/SettingsService.qml, bar/island/NightLightDetail.qml, bar/widgets/DynamicIsland.qml

### PickerService
Seletor de cor via hyprpicker e retorno de cor.

- Arquivo: services/PickerService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer, execDetached
- Funções: pick, stop, took
- Propriedades: icon, available, checked, pendingAfter, settle, picking, probe, picker, launch
- Comandos estáticos: "sh", "-c", "command -v hyprpicker"; "hyprpicker", "-f", "hex"
- Consumidores QML (2): shell.qml, services/ControlsService.qml

### AccountService
Conta do usuário, nome, avatar e alteração/limpeza de avatar via helper privilegiado.

- Arquivo: services/AccountService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: read, settle, write, setName, setPicture, clearPicture
- Propriedades: script, user, systemName, fullName, systemAvatar, accounts, shared, busy, failure, failedKind, name, avatar, initials, reader, revision, writer, pendingName, namePause
- Comandos estáticos: root.script, "get"
- Consumidores QML (7): components/Avatar.qml, lock/LockAccount.qml, services/SettingsService.qml, settings/SessionSection.qml, bar/island/AuthPanel.qml, bar/island/controls/ImpastoBlock.qml, bar/island/controls/VelaturaBlock.qml

## Barra, ilha e interação

### ModuleService
Catálogo de módulos da barra, visibilidade, atividades, valores, figuras e ativação de painéis.

- Arquivo: services/ModuleService.qml
- Imports: QtQml, QtQuick, Quickshell, "../theme"
- Mecanismos: não detectados
- Funções: entry, isButton, placeable, watch, togglePanel, shapeOf, figureOf, glyphOf, valueOf, figureLimit, tintOf, runs, shows, close, openSize, activate, requestPanel, has
- Propriedades: catalogue, buttonIds, buttons, shownPanel, ringed, today, runners, running, activities, overflow, overflowWidth, clockCore, activitySide, activityInset, privacyNameLimit, privacyName, nameMetrics, restWidth, summaryWeather, summaryWidth, glanceTimeBase, glanceTime, timeMetrics, summaryHeight, glanceRows, glanceRank, glanceFace, glanceStrip, glanceMargin, glanceFaceHeight, glanceStripHeight, openId, openHost, openGone
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (44): shell.qml, bar/Bar.qml, deck/Deck.qml, desktop/Desktop.qml, desktop/Inspector.qml, desktop/Widget.qml, lock/LockIsland.qml, services/DesktopService.qml, services/SettingsService.qml, services/GamesService.qml, services/OsdService.qml, services/PetService.qml, services/PrivacyService.qml, services/TrayService.qml, settings/IslandSection.qml, settings/LayoutEditor.qml (lista parcial; buscar pelo singleton para obter todos)

### ControlsService
Modelo/estado do control center, catálogo de toggles, blocos e layout editável.

- Arquivo: services/ControlsService.qml
- Imports: QtQml, QtQuick, Quickshell, "../theme"
- Mecanismos: JsonAdapter, Timer
- Funções: stored, door, topEntry, sideOf, toggleTop, nudgeTop, flipTop, placeTop, doorKeysOf, doorsOf, moved, activate, tileOf, toggleKeysOf, tilesOf, tileRowsOf, listRowsOf, listKeysOf, toggleIn, moveIn, entry, sizesFor, offers, parse, sizeOf, label, offsetX, offsetY, pixels, geometry, normalise, read, onCentreBlocksChanged, syncKeys, entryOf, countOf, placed, overlaps, onBoard, free, nearestFree, firstFree, cellX, cellY, write, update, newKey, add, remove, place, setSize, cycleSize, sizeNearest, restore, resize, overTray, edit
- Propriedades: doors, defaultButtons, defaultColumns, defaultRows, buttons, topCatalogue, topSides, topLeft, topRight, hasTop, topShown, topSpare, key, icon, label, detail, active, available, expandable, panel, closes, action, airborne, toggleCatalogue, shadows, count, defaultToggles, toggleKeys, catalogue, defaultBlocks, columns, rows, boardWidth, boardHeight, rowHeight, rowGap, panelWidth, panelHeight, tallestPanel, blocks, saver, keys, handleInset, editing, dragging, selected, landing, board, tray
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (18): bar/Bar.qml, components/TopRowEditor.qml, services/LauncherService.qml, services/ModuleService.qml, services/SettingsService.qml, settings/ControlsSection.qml, theme/Theme.qml, bar/island/ControlsPanel.qml, bar/widgets/DynamicIsland.qml, bar/island/controls/BlockInspector.qml, bar/island/controls/Block.qml, bar/island/controls/BlockFace.qml, bar/island/controls/ButtonsBlock.qml, bar/island/controls/ControlsTray.qml, bar/island/controls/GridSize.qml, bar/island/controls/TogglesBlock.qml (lista parcial; buscar pelo singleton para obter todos)

### OsdService
OSD transitório para volume, mute, brilho e bateria, com debounce/supressão.

- Arquivo: services/OsdService.qml
- Imports: QtQuick, Quickshell
- Mecanismos: Timer
- Funções: onVolumeChanged, onMutedChanged, onAdjusted, onChargingChanged, suppressAudio
- Propriedades: armed, debounce, audioSuppressed, suppression, armTimer, suppressionTimer, volumeTimer, brightnessDisplay, brightnessTimer, audio, brightness, battery
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (14): Commands.qml, services/ControlsService.qml, services/PickerService.qml, services/SunsetService.qml, services/WallpaperService.qml, services/CameraService.qml, services/CaptureService.qml, services/GamesService.qml, services/PetService.qml, services/ProfileService.qml, services/RecorderService.qml, services/SystemService.qml, bar/island/IslandState.qml, bar/island/ServersPanel.qml

### NotificationService
Servidor de notificações, fila, histórico, ações, timeout e Não Perturbe.

- Arquivo: services/NotificationService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, Quickshell.Services.Notifications, "../theme"
- Mecanismos: FileView, Timer, Notifications
- Funções: expireLater, onClosed, timeoutFor, record, iconOf, standIn, openOf, defaultAction, buttonsOf, invoke, open, inlineButtons, keep, present, dismiss, close, clearHistory, remove, save, toggleDoNotDisturb
- Propriedades: defaultTimeout, current, history, historyLimit, active, critical, server, lifetime, closing, toastPadding, toastRow, toastPicture, stackedRow, toastWidth, textGap, buttonGap, buttonPad, buttonHeight, inlineLetters, toastHeight, retainer, store, restored, doNotDisturb
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (10): Commands.qml, services/ControlsService.qml, services/ModuleService.qml, services/ScreenTimeService.qml, settings/IslandSection.qml, bar/island/IslandState.qml, bar/island/NotificationLayer.qml, bar/modules/NotificationsModule.qml, bar/widgets/DynamicIsland.qml, bar/island/controls/NotificationList.qml

### TrayService
StatusNotifier/System Tray, itens, menus, navegação e ativação.

- Arquivo: services/TrayService.qml
- Imports: QtQuick, Quickshell, Quickshell.Services.SystemTray, "../theme"
- Mecanismos: SystemTray
- Funções: find, lost, open, enter, back, release, activate
- Propriedades: items, item, stack, revision, top, first, second, opener, entries, gone, menuWidth, rowHeight, separatorHeight, rowSpacing, headerHeight, maxHeight, listHeight, menuHeight
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (5): settings/LayoutEditor.qml, bar/island/TrayMenuPanel.qml, bar/widgets/DynamicIsland.qml, bar/widgets/BarZone.qml, bar/widgets/TrayWidget.qml

### LauncherService
Busca universal, modos/prefixos, ranking, histórico, apps, janelas, clipboard, emoji, timer e calculadora.

- Arquivo: services/LauncherService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: FileView, JsonAdapter, execDetached
- Funções: modeFor, termFor, listHeight, heightFor, scoreOf, record, weightOf, refresh, search, rank, calculate, desk, offer, timer, clipboard, emoji, windows, spell, evaluate, activate, copy, launchDesktop
- Propriedades: maxResults, modes, query, results, fieldHeight, rowHeight, rowSpacing, gap, stripHeight, showsStrip, emojiCell, emojiColumns, chromeHeight, rows, panelWidth, panelHeight, arithmetic, applications, order, halfLife, launches, historyFile, favourites
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (6): shell.qml, services/ApplicationsService.qml, settings/KeptApplications.qml, settings/LauncherSection.qml, bar/island/LauncherPanel.qml, bar/widgets/DynamicIsland.qml

### ApplicationsService
Índice de aplicações .desktop, atualização do catálogo e lançamento de apps.

- Arquivo: services/ApplicationsService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer, execDetached
- Funções: refresh, launchDesktop
- Propriedades: applications, indexer
- Comandos estáticos: Quickshell.shellPath("scripts/applications.py")
- Consumidores QML (2): services/DockService.qml, services/LauncherService.qml

## Dispositivos e conectividade

### AudioService
Saídas/entradas PipeWire, volume, mute, seleção de dispositivos e streams.

- Arquivo: services/AudioService.qml
- Imports: QtQuick, Quickshell, Quickshell.Services.Pipewire, "../theme"
- Mecanismos: PipeWire
- Funções: outputName, choose, chooseInput, streamName, streamIcon, setStreamVolume, toggleStreamMute, setSourceVolume, setVolume, toggleMute, toggleSourceMute
- Propriedades: sink, ready, source, sourceReady, sourceMuted, sourceIcon, volume, muted, icon, sourceVolume, outputs, streams, inputs, inStreams, captures, outputPrefix, cardRows, panelWidth, panelHeight, microphoneHeight
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (14): Commands.qml, services/ControlsService.qml, services/ModuleService.qml, services/OsdService.qml, services/PrivacyService.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/VolumeFace.qml, desktop/faces/sticker/VolumeFace.qml, bar/island/MicrophoneDetail.qml, bar/island/SoundDetail.qml, bar/modules/VolumeModule.qml, bar/widgets/DynamicIsland.qml, bar/island/controls/BlockFace.qml

### BatteryService
Estado UPower, carga, autonomia, potência, capacidade e saúde da bateria.

- Arquivo: services/BatteryService.qml
- Imports: QtQuick, Quickshell, Quickshell.Services.UPower, "../theme"
- Mecanismos: UPower
- Funções: nenhuma detectada
- Propriedades: device, available, percent, state, charging, full, low, levelIcons, tint, secondsToEmpty, secondsToFull, estimate, watts, energy, energyCapacity, healthKnown, health, stateWord, icon
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (9): lock/LockSurface.qml, services/ModuleService.qml, services/OsdService.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/BatteryFace.qml, desktop/faces/sticker/BatteryFace.qml, bar/modules/BatteryModule.qml, bar/widgets/BatteryWidget.qml

### BluetoothService
Adaptador BlueZ, dispositivos, descoberta, conexão e desconexão.

- Arquivo: services/BluetoothService.qml
- Imports: QtQuick, Quickshell, Quickshell.Bluetooth
- Mecanismos: Bluetooth API
- Funções: isNamed, deviceIcon, connectDevice, setDiscovering, toggle
- Propriedades: adapter, available, enabled, allDevices, connectedDevices, summary, icon, devices, unnamedDevices, unnamedCount, discovering
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (8): services/ControlsService.qml, services/ModuleService.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/BluetoothFace.qml, desktop/faces/sticker/BluetoothFace.qml, bar/island/BluetoothDetail.qml, bar/modules/BluetoothModule.qml

### BrightnessService
Brilho interno/backlight e monitores externos via DDC/CI quando disponível.

- Arquivo: services/BrightnessService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, FileView, Timer
- Funções: iconFor, setPercent, step, refresh, isPanel, detect, parse, onScreensChanged, flush, read
- Propriedades: displays, dimmable, current, available, percent, icon, several, panelWidth, panelHeight, device, raw, maximum, backlightPercent, discovery, maxView, levelView, buses, external, detector, redetect, hotplug, variants, name, panel, title, bus, backlight, level, ceiling, answering, wanted, written, writer, reader
- Comandos estáticos: "sh", "-c", "ls -1 /sys/class/backlight/ 2>/dev/null | head -1"; "sh", "-c", "command -v ddcutil >/dev/null && exec ddcutil detect --brief"
- Consumidores QML (11): Commands.qml, services/ModuleService.qml, services/OsdService.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/BrightnessFace.qml, desktop/faces/sticker/BrightnessFace.qml, bar/island/BrightnessDetail.qml, bar/modules/BrightnessModule.qml, bar/widgets/DynamicIsland.qml, bar/island/controls/BlockFace.qml

### NetworkService
NetworkManager, Wi-Fi, rede cabeada, scan, conexão, desconexão e esquecer rede.

- Arquivo: services/NetworkService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, Quickshell.Networking
- Mecanismos: Process, Timer
- Funções: refresh, toggleWifi, strengthIcon, scan, connect, disconnect, forget, setWifi
- Propriedades: devices, wifiDevice, wiredDevice, wifiEnabled, wifiConnected, wiredConnected, online, connectionName, strength, stateLine, icon, radioOn, radioQuery, radioToggle, settleTimer, networks, scanning, busySsid, lastError, pendingAction, script, listProcess, connectProcess
- Comandos estáticos: "nmcli", "-t", "-f", "WIFI", "radio", "all"
- Consumidores QML (9): services/ControlsService.qml, services/ModuleService.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/NetworkFace.qml, desktop/faces/sticker/NetworkFace.qml, bar/island/NetworkDetail.qml, bar/modules/NetworkModule.qml, bar/island/controls/TogglesBlock.qml

### SystemService
Estado e toggles agregados de Wi-Fi, Bluetooth e perfil de energia.

- Arquivo: services/SystemService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: ingest, refresh, subscribe, release, toggle
- Propriedades: script, pollInterval, watchers, wifi, bluetooth, powerProfile, ready, performanceMode, statusProcess, actionProcess, pollTimer
- Comandos estáticos: root.script, "status"
- Consumidores QML (2): services/ControlsService.qml, bar/island/controls/TogglesBlock.qml

### PrivacyService
Indicadores de uso de microfone/câmera/tela por processos e nós PipeWire.

- Arquivo: services/PrivacyService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, Quickshell.Services.Pipewire
- Mecanismos: Process, PipeWire
- Funções: prop, nameOf
- Propriedades: tracker, nodes, listeners, viewers, sharing, cameraHolders, camera, microphone, cameraOn, screen, active, who
- Comandos estáticos: Quickshell.shellPath("scripts/privacy.py")
- Consumidores QML (3): services/ModuleService.qml, bar/island/IslandSummary.qml, bar/island/IslandRest.qml

### CameraService
Estado da câmera, detecção facial, servidor/efeitos/reação e captura de amostras.

- Arquivo: services/CameraService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer
- Funções: reacting, setReaction, check, exited, tend, hear, snap, configure
- Propriedades: script, panelWidth, panelHeight, ready, checked, missing, state, readers, fps, faces, error, others, enabled, serving, effects, reactions, effect, samples, sample, sampleMask, sampleEyes, sampleForehead, checker, wanted, server, started, failures, waiting, retry, snapping, snapper, configuration, writer, reconfigure, configurer
- Comandos estáticos: root.script, "status"; root.script, "serve"; root.script, "snap"
- Consumidores QML (3): services/SettingsService.qml, bar/island/CameraPanel.qml, bar/widgets/DynamicIsland.qml

## Conteúdo, produtividade e mídia

### CalendarService
Calendários e eventos por meio do daemon dcal, incluindo criação/abertura.

- Arquivo: services/CalendarService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: dateKey, eventStartKey, eventEndKey, eventsOn, hasEventsOn, ensure, load, refreshMonth, openCalendar, newEvent, openEvent
- Propriedades: available, starting, calendars, events, rangeFrom, rangeTo, loadedMonth, pendingDate, pendingOpen, writableCalendars, ping, daemonStart, pingRetry, calendarsReader, eventsReader, openProcess, newEventProcess, eventProcess
- Comandos estáticos: "dcal", "ipc", "ping"; "dcal", "run", "-d", "--hidden"; "dcal", "ipc", "calendars.list"; "dcal", "ipc", "events.list", `from=${root.rangeFrom}`, `to=${root.rangeTo}`, "limit=250"; "dcal", "ipc", "ui.show", "view=month"; "dcal", "ipc", "ui.newEvent", `start=${root.pendingDate}`
- Consumidores QML (2): bar/modules/CalendarModule.qml, bar/island/controls/CalendarCard.qml

### ClipboardService
Histórico de clipboard, normalização, busca, copiar, apagar e wipe.

- Arquivo: services/ClipboardService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, FileView
- Funções: normalise, tend, run, copy, forget, wipe, search, weigh, since, describe, title
- Propriedades: script, entries, count, store, wanted, watcher, started, launched, action
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (4): shell.qml, services/ControlsService.qml, services/LauncherService.qml, bar/island/LauncherPanel.qml

### EmojiService
Catálogo, grupos, tons, recentes, busca e cópia de emoji.

- Arquivo: services/EmojiService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, FileView, JsonAdapter, execDetached
- Funções: load, stepGroup, stepTone, glyphOf, remember, fold, search, copy
- Propriedades: emoji, language, fresh, indexProcess, groups, group, tone, toneMarks, keepRecent, recent, recentFile, groupWords
- Comandos estáticos: Quickshell.shellPath("scripts/emoji.py"), root.language
- Consumidores QML (2): services/LauncherService.qml, bar/island/LauncherPanel.qml

### MediaService
Players MPRIS, metadados, capa, posição, seek e comandos de reprodução.

- Arquivo: services/MediaService.qml
- Imports: QtQuick, Quickshell, Quickshell.Services.Mpris, "."
- Mecanismos: Timer, MPRIS
- Funções: subscribe, release, toggle, next, seek, previous
- Propriedades: players, active, available, playing, title, artist, album, artUrl, identity, canNext, canPrevious, canToggle, canSeek, length, position, seekable, progress, watchers, positionTimer
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (17): Commands.qml, lock/LockMedia.qml, lock/LockSurface.qml, services/LyricsService.qml, services/ModuleService.qml, settings/SessionSection.qml, desktop/faces/Larges.qml, desktop/faces/LyricsFace.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/MediaFace.qml, desktop/faces/sticker/MediaFace.qml, bar/island/IslandSummary.qml, bar/island/IslandRest.qml, bar/island/LyricsDetail.qml, bar/modules/MediaModule.qml (lista parcial; buscar pelo singleton para obter todos)

### LyricsService
Letras sincronizadas com posição da faixa MPRIS e busca/cache.

- Arquivo: services/LyricsService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "."
- Mecanismos: Process, Timer
- Funções: onPositionChanged, onPlayingChanged, place, subscribe, release, fetch, take
- Propriedades: watchers, wanted, track, readFor, lines, synced, instrumental, loading, available, current, currentText, nextText, status, anchorPosition, anchorTime, ticker, lead, again, asked, patience, retry
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (7): lock/LockMedia.qml, services/ModuleService.qml, desktop/faces/LyricsFace.qml, bar/island/IslandSummary.qml, bar/island/DetailFace.qml, bar/island/LyricsDetail.qml, bar/modules/MediaModule.qml

### NotesService
Notas persistentes, cores, arquivo/arquivamento, edição e abertura.

- Arquivo: services/NotesService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: FileView, JsonAdapter, Timer
- Funções: tintColor, paperOf, normalise, entry, noteFor, titleOf, isEmpty, newKey, write, add, create, update, setTint, archive, remove, firstLine, display, ageOf, open, leave
- Propriedades: ready, tints, notes, live, archived, count, newest, panelWidth, panelHeight, opened, direct, saver, file
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (15): Commands.qml, components/Sticky.qml, deck/Deck.qml, desktop/Desktop.qml, desktop/Inspector.qml, services/DesktopService.qml, services/ModuleService.qml, services/DeckService.qml, services/TasksService.qml, desktop/faces/NoteFace.qml, bar/island/BoardPanel.qml, bar/island/NotesPanel.qml, bar/modules/NotesModule.qml, bar/widgets/DynamicIsland.qml, bar/island/controls/BlockFace.qml

### TasksService
Tarefas persistentes, estados, datas, ordenação, vencimento e operações.

- Arquivo: services/TasksService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: FileView, JsonAdapter, Timer
- Funções: stateEntry, stateAfter, dayKey, dateOf, shifted, normalise, entry, byRank, inState, countIn, on, countOn, pendingOn, isOverdue, dueLabel, monthIndex, parseDue, split, newKey, write, lastRank, add, create, isEmpty, update, setState, place, setDue, toggle, remove, open, leave
- Propriedades: ready, states, clock, todayKey, tasks, count, pending, dated, overdue, upcoming, next, queue, summary, dayNames, monthNames, panelWidth, panelHeight, opened, direct, saver, file
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (20): Commands.qml, components/DayPicker.qml, components/TaskRow.qml, services/ModuleService.qml, services/NotesService.qml, desktop/faces/DayTasks.qml, desktop/faces/Larges.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/CalendarFace.qml, desktop/faces/analogue/Clipboard.qml, desktop/faces/analogue/TasksFace.qml, desktop/faces/sticker/CalendarFace.qml, desktop/faces/sticker/TasksFace.qml, bar/island/BoardPanel.qml, bar/modules/CalendarModule.qml (lista parcial; buscar pelo singleton para obter todos)

### TimerService
Timer com parse de duração, pause/resume/restart/cancel e notificação.

- Arquivo: services/TimerService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer
- Funções: parse, start, pause, resume, toggle, restart, cancel, complete
- Propriedades: running, paused, label, endsAt, duration, remaining, progress, seconds, tint, display, ticker, notifier
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (11): Commands.qml, services/LauncherService.qml, services/ModuleService.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/TimerFace.qml, desktop/faces/sticker/TimerFace.qml, bar/island/IslandSummary.qml, bar/island/IslandRest.qml, bar/modules/TimerModule.qml, bar/widgets/TimerWidget.qml

### ScreenTimeService
Registro de tempo por janela/aplicação, categorias, histórico e relatórios.

- Arquivo: services/ScreenTimeService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer
- Funções: send, onFocusedWindowChanged, onUnlocked, onReceived, subscribe, release, refresh, key, today, dateOf, step, show, categoryOf, appOf, digest, rankOf, follow, unfollow, readSummary, short, duration, clock
- Propriedades: script, away, recorder, restart, focus, lock, notifications, view, date, report, loaded, empty, current, watchers, stale, reader, tick, canGoBack, canGoForward, categoryOverrides, categoryNames, shades, shown, lookup, apps, categories, ranked, stacks, summary, todayDigest, todayTotal, todayBaseline, todayLead, todayParts, todayNote, todayCompared, todayFraction, followers, summaryReader, summaryTick
- Comandos estáticos: root.script, "record"; root.script, "summary"
- Consumidores QML (10): components/UsageBars.qml, services/ModuleService.qml, desktop/faces/Bands.qml, desktop/faces/Larges.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/ScreenTimeFace.qml, desktop/faces/sticker/ScreenTimeFace.qml, bar/island/ScreenTimePanel.qml, bar/modules/ScreenTimeModule.qml

## Desktop e superfícies secundárias

### DesktopService
Widgets de desktop, famílias, temas, grade, posições, decks e espectros.

- Arquivo: services/DesktopService.qml
- Imports: QtQml, QtQuick, Quickshell, "../theme"
- Mecanismos: JsonAdapter, Timer, execDetached
- Funções: family, familyOf, sizeFor, themeOf, setTheme, familiesFor, offers, drawnFamily, conform, onDesktopThemeChanged, nameOf, screenField, widgetsOn, decksOn, setBoard, boardOn, gridFor, wholeGrid, edgeAt, gridOn, strideOn, columnsOn, rowsOn, offsetX, offsetY, span, cellX, cellY, normalise, isEdge, isDeck, isSpectrum, onDesktopWidgetsChanged, keysOn, deckKeysOn, spectrumKeysOn, onEffectivePrimaryNameChanged, onMonitorsChanged, syncKeys, sortByScreen, sameOrNew, entryOf, countOf, placed, geometry, spotOf, clamped, clashes, overlaps, onBoard, free, nearestFree, firstFree, write, update, newKey, add, remove, removeMatching, deckOn, deckNotes, alongOf, setDeckAlong, placementOf, withoutNote, removeNote, placeNote, noteToGrid, noteToEdge, addDeck, setDeckEdge, setTakesNew, noteAdded, toggleDeckNote, spectrumBox, spectrumColourName, spectrumColour, spectrumOf, spectrumAwayOn, setSpectrum, spectrumOn, spectrumTakes, freeSpectrumEdge, addSpectrum, setSpectrumEdge, spectrumToEdge, spectrumToGrid, place, setFamily, cycleFamily, familyNearest, bare, opacityOf, setOpacity, inkFor, setPicture, urlOf, pictureOf, openPicture, openMenu, closeMenu, galleryAtOn, setGalleryAt, overTray, publish, edit, onWindowOpened, onActiveWorkspaceIdChanged
- Propriedades: families, themes, faces, insets, boards, hidden, grids, widgets, decks, spectra, squares, saver, shown, shownDecks, keys, deckKeys, spectrumKeys, rehomes, offerable, spots, edges, spectrumLooks, spectrumFills, spectrumRanges, handleInset, pictureTypes, picking, editing, typing, dragging, selected, landing, inHand, menu, trayBox, galleryAt, gallerySize, editingScreen, galleryScreen, surfaces, windows, compositor
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (65): shell.qml, bar/Bar.qml, deck/Deck.qml, desktop/Desktop.qml, desktop/Inspector.qml, desktop/Widget.qml, desktop/EdgeSpectrum.qml, desktop/Face.qml, desktop/Picker.qml, desktop/SpectrumBars.qml, desktop/ThemeSwatch.qml, desktop/Tray.qml, dock/Dock.qml, services/ControlsService.qml, services/ModuleService.qml, services/SettingsService.qml (lista parcial; buscar pelo singleton para obter todos)

### DockService
Apps fixados/em execução, ordenação, drag, ativação, fechamento e autohide.

- Arquivo: services/DockService.qml
- Imports: QtQml, QtQuick, Quickshell, "../theme"
- Mecanismos: JsonAdapter, Timer
- Funções: shownOn, onDockPinnedChanged, write, isPinned, pin, unpin, togglePin, reorder, entryOf, binaryOf, idFor, groupFor, describe, offsetOf, box, shifted, slotAt, beginDrag, endDrag, activate, launch, closeAll
- Propriedades: enabled, edge, alignment, showsRunning, autohide, hasLauncher, lead, launcherIndex, vertical, everywhere, zone, pinned, saver, applications, classIndex, groups, coveredScreens, items, count, pinnedCount, divides, launcherOffset, length, retreat, hiddenX, hiddenY, dragging, dragFrom, dropAt
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (7): dock/Dock.qml, dock/DockItem.qml, dock/DockMenu.qml, services/DesktopService.qml, services/LauncherService.qml, services/ScreenTimeService.qml, settings/KeptApplications.qml

### DeckService
Notas/cartões nas bordas da tela, geometria, reveal, drag e animação.

- Arquivo: services/DeckService.qml
- Imports: QtQuick, Quickshell, "../theme"
- Mecanismos: não detectados
- Funções: decksOn, awayOn, vertical, stripLength, leadOf, runOf, startOf, alongAt, tabAt, tabBox, stripBox, gripBox, peekBox, edgeAt, indexAt, publish
- Propriedades: decks, count, onEmptyOnly, sliver, tabDepth, tabLength, tabGap, peekWidth, peekHeight, margin, reach, grip, revealed, peeked, dragging, sliding, held, receiving, receivingScreen, surfaces, windows
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (6): deck/Deck.qml, desktop/Desktop.qml, desktop/Widget.qml, desktop/EdgeSpectrum.qml, desktop/Tray.qml, services/DesktopService.qml

### CaptureService
Ferramentas de captura, seleção de forma/destino, screenshot e clipboard.

- Arquivo: services/CaptureService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer, execDetached
- Funções: can, setShape, setKind, offers, open, cancel, fire, box, crop, took, said
- Propriedades: script, tools, ready, directory, photo, photoWidth, photoHeight, active, busy, settle, screenName, screen, ratio, shapes, kinds, destinations, shape, kind, to, probe, launch, grabber, shooter, marks
- Comandos estáticos: root.script, "tools"; root.script, "grab"
- Consumidores QML (7): shell.qml, Commands.qml, bar/Bar.qml, capture/CaptureBar.qml, capture/CaptureOverlay.qml, services/ControlsService.qml, services/ModuleService.qml

### RecorderService
Gravação de tela/áudio, status, duração, destino e controle.

- Arquivo: services/RecorderService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: startAt, stop, toggle, measure, ingest
- Propriedades: script, tools, tool, available, canAudio, recording, startedAt, seconds, shape, sound, size, path, name, shapeNames, subject, display, clock, probe, ask, action
- Comandos estáticos: root.script, "tools"; root.script, "status"
- Consumidores QML (5): shell.qml, Commands.qml, capture/CaptureBar.qml, services/ControlsService.qml, bar/island/IslandRest.qml

## Monitoramento e integrações externas

### StatsService
CPU, cores, memória, swap, disco, rede, temperatura e históricos.

- Arquivo: services/StatsService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: push, ingest, bytes, rate, duration
- Propriedades: pollInterval, historyLength, ready, cpu, cores, cpuModel, load, memoryUsed, memoryTotal, swapUsed, swapTotal, disks, networkDown, networkUp, temperature, uptime, cpuHistory, memoryHistory, downHistory, upHistory, temperatureHistory, memoryFraction, window, thermalWord, swapFraction, sampler, revive, byteUnits
- Comandos estáticos: Quickshell.shellPath("scripts/stats.py"), "watch", `${root.pollInterval / 1000}`
- Consumidores QML (11): services/ModuleService.qml, settings/SystemSection.qml, desktop/faces/Larges.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/StatsFace.qml, desktop/faces/sticker/StatsFace.qml, bar/island/StatsPanel.qml, bar/modules/StatsModule.qml, bar/island/controls/ImpastoBlock.qml, bar/island/controls/VelaturaBlock.qml

### CavaService
Visualizador de espectro de áudio CAVA com assinantes e ciclo de vida compartilhado.

- Arquivo: services/CavaService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: parse, subscribe, release
- Propriedades: bandCount, bands, peakFall, peaks, barCount, values, watchers, active, linger, level, process
- Comandos estáticos: "cava", "-p", Quickshell.shellPath("scripts/cava.conf")
- Consumidores QML (4): components/Spectrum.qml, desktop/SpectrumBars.qml, bar/island/IslandRest.qml, bar/modules/MediaModule.qml

### ClaudeService
Uso/limites da conta Claude, cache, métricas de sessão e janela semanal.

- Arquivo: services/ClaudeService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer
- Funções: messages, compact, percent, subscribe, release, refresh, askLimits
- Propriedades: pollInterval, watchers, available, blockStart, blockEnd, blockTokens, blockMessages, weekTokens, weekMessages, peakBlockTokens, peakWeekTokens, sessionMeasured, weeklyMeasured, measured, sessionFraction, weeklyFraction, clock, blockEnds, remaining, elapsed, resetsIn, tint, gauge, rows, poller, limitsInterval, limitsKnown, sessionUsed, weekUsed, sessionResets, weekResets, claim, plan, models, limited, limitsPoller, limitsQuery, query
- Comandos estáticos: Quickshell.shellPath("scripts/claude_usage.py"), "limits"; Quickshell.shellPath("scripts/claude_usage.py")
- Consumidores QML (8): services/ModuleService.qml, services/CodexService.qml, desktop/faces/Larges.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/ClaudeFace.qml, desktop/faces/sticker/ClaudeFace.qml, bar/modules/ClaudeModule.qml

### CodexService
Uso/limites Codex, observação de sessões e métricas.

- Arquivo: services/CodexService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer
- Funções: current, windowLine, subscribe, release, refresh
- Propriedades: pollInterval, watchers, available, plan, observed, limits, clock, live, gauge, measured, fullest, fullestName, figure, tint, age, poller, query
- Comandos estáticos: Quickshell.shellPath("scripts/codex_usage.py")
- Consumidores QML (7): services/ModuleService.qml, desktop/faces/Larges.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/CodexFace.qml, desktop/faces/sticker/CodexFace.qml, bar/modules/CodexModule.qml

### GithubService
Estatísticas de contribuições GitHub e atualização sob demanda/assinantes.

- Arquivo: services/GithubService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: grouped, subscribe, release, refresh, onGithubUserChanged
- Propriedades: pollInterval, watchers, available, user, total, streak, today, busiest, weeks, userUnknown, readAt, clock, age, totalLabel, cardGrid, poller, query
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (6): services/ModuleService.qml, settings/ModulesSection.qml, desktop/faces/GithubFace.qml, desktop/faces/analogue/GithubFace.qml, desktop/faces/sticker/GithubFace.qml, bar/modules/GithubModule.qml

### MarketsService
Cotações e variações de ativos configurados, cache e atualização.

- Arquivo: services/MarketsService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: Process, Timer
- Funções: grouped, digits, price, short, change, rising, tint, shape, glyph, subscribe, release, refresh, onMarketsChanged
- Propriedades: pollInterval, watchers, available, items, unknown, readAt, lead, others, chartHeight, cardHeight, pairs, clock, age, signs, again, asked, poller, settle, query
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (9): services/ModuleService.qml, settings/ModulesSection.qml, desktop/faces/Bands.qml, desktop/faces/Larges.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/MarketsFace.qml, desktop/faces/sticker/MarketsFace.qml, bar/modules/MarketsModule.qml

### WeatherService
Previsão/condições, localização configurada, cache e atualização.

- Arquivo: services/WeatherService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: subscribe, release, refresh, onWeatherPlaceChanged, onArrivedChanged, settle, take
- Propriedades: pollInterval, watchers, available, place, region, temperature, feelsLike, description, glyph, humidity, wind, low, high, hourly, readAt, readFor, wanted, stale, clock, age, ahead, again, asked, askedAt, poller, placeUnknown, query, unsettled
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (11): services/ModuleService.qml, settings/ModulesSection.qml, desktop/faces/Bands.qml, desktop/faces/Larges.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/WeatherFace.qml, desktop/faces/sticker/WeatherFace.qml, bar/island/IslandSummary.qml, bar/modules/WeatherModule.qml, bar/island/controls/WeatherCard.qml

### UpdatesService
Checagem de atualizações pacman/AUR e contagem de pacotes.

- Arquivo: services/UpdatesService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: subscribe, release, refresh
- Propriedades: pollInterval, watchers, available, checking, count, packages, updates, aur, tool, checkedAt, clock, age, poller, query
- Comandos estáticos: Quickshell.shellPath("scripts/updates.py")
- Consumidores QML (8): services/ModuleService.qml, services/PackagesService.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/UpdatesFace.qml, desktop/faces/sticker/UpdatesFace.qml, bar/island/PackagesPanel.qml, bar/modules/UpdatesModule.qml

### PackagesService
Pesquisa/instalação/remoção/atualização de pacotes e integração pacman/AUR.

- Arquivo: services/PackagesService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: step, load, search, matches, install, remove, upgrade, installable, open
- Propriedades: panelWidth, panelHeight, fieldHeight, rowHeight, footerHeight, view, query, installedFilter, views, loaded, loading, installed, helper, mineCount, aurCount, installedProcess, found, foundTerm, foundRepos, foundAur, foundNote, searching, term, debounce, searchProcess, descriptions, rows, busy, holdScript, terminal
- Comandos estáticos: Quickshell.shellPath("scripts/packages.py"), "installed"
- Consumidores QML (5): services/FaceService.qml, services/VersionService.qml, bar/island/PackagesPanel.qml, bar/modules/UpdatesModule.qml, bar/widgets/DynamicIsland.qml

### VersionService
Versão do shell/repositório Git, status de árvore, commits e atualização.

- Arquivo: services/VersionService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, FileView, Timer, execDetached
- Funções: check, checkStale, update, listFiles, changeFiles
- Propriedades: freshness, file, lines, version, branch, repo, tree, answered, available, checking, offline, behind, ahead, upstream, checkoutBranch, target, commits, checkedAt, checkedFor, query, deleted, edited, changing, manifest, lister, listWanted, settle
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (2): services/FaceService.qml, settings/SystemSection.qml

### MachineService
Informações da máquina/sistema operacional, kernel, host e compositor.

- Arquivo: services/MachineService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process
- Funções: refresh
- Propriedades: script, ready, os, kernel, host, wm, shell, packages, reader
- Comandos estáticos: root.script, "get"
- Consumidores QML (2): bar/island/controls/ImpastoBlock.qml, bar/island/controls/VelaturaBlock.qml

### ServersService
Inventário de servidores/processos gerenciados, uptime, URLs, abrir/copiar/parar.

- Arquivo: services/ServersService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer, execDetached
- Funções: subscribe, release, refresh, uptime, url, open, copy, stop
- Propriedades: script, servers, loaded, exposed, error, panelWidth, rowHeight, panelHeight, watchers, lister, tick, stopping, stopper
- Comandos estáticos: root.script, "list"
- Consumidores QML (2): bar/island/ServersPanel.qml, bar/widgets/DynamicIsland.qml

### VmService
Máquinas virtuais, catálogo, ciclo de vida, recursos e estado.

- Arquivo: services/VmService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: Process, Timer
- Funções: subscribe, release, refresh, uptime, find, act, onRunningChanged, start, open, pause, resume, stop, kill, remove, snapshot, setCores, setMemory, loadCatalog, create, cancelCreate, mark
- Propriedades: script, machines, tools, hostCores, hostMemory, loaded, ready, running, coresInUse, memoryInUse, error, watchers, lister, tick, actor, busy, done, catalog, catalogLoading, catalogReader, fetching, fetcher, marks
- Comandos estáticos: root.script, "list"; root.script, "catalog"
- Consumidores QML (3): Commands.qml, bar/island/MachinesPanel.qml, bar/island/IslandRest.qml

### GamesService
Catálogo de minijogos e persistência de recordes/partidas.

- Arquivo: services/GamesService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, "../theme"
- Mecanismos: FileView, JsonAdapter, Timer
- Funções: entry, tintOf, panelSize, open, leave, recordOf, played, bestOf, playsOf, record
- Propriedades: ready, catalogue, tints, shelfWidth, shelfHeight, stripHeight, stripGap, playing, lastOpened, bests, totalPlays, lastPlayed, ranked, saver, file
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (9): services/ModuleService.qml, desktop/faces/Squares.qml, desktop/faces/analogue/GamesFace.qml, desktop/faces/sticker/GamesFace.qml, bar/island/GamesPanel.qml, bar/modules/GamesModule.qml, bar/widgets/DynamicIsland.qml, bar/island/controls/GamesBlock.qml, bar/island/games/GameBoard.qml

### PetService
Família de pets, experiência, níveis, alimentação, brincadeira, humor e recompensas.

- Arquivo: services/PetService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io
- Mecanismos: FileView, JsonAdapter, Timer
- Funções: subscribe, release, speciesOf, recordAt, replace, newborn, earned, thresholdFor, reward, agoOf, feed, play, renameAt, rename, bringOut, moodAt, titleOf, adopt
- Propriedades: ready, watchers, species, styles, family, active, activeIndex, pet, name, level, xp, speciesInfo, hatched, milestones, totalLevel, complete, nextEggAt, levelsToNextEgg, eggProgress, threshold, progress, feedCooldown, playCooldown, clock, fedAgo, playedAgo, fedAwake, playedAwake, canFeed, canPlay, mood, moodLine, companyEvery, companyXp, company, saver, file
- Comandos estáticos: nenhum literal em Process.command
- Consumidores QML (14): components/PetFace.qml, components/PetFamily.qml, components/PetShelf.qml, services/ModuleService.qml, services/SettingsService.qml, settings/ModulesSection.qml, desktop/faces/Squares.qml, desktop/faces/Wides.qml, desktop/faces/analogue/CreatureFace.qml, desktop/faces/sticker/CreatureFace.qml, bar/island/PetPanel.qml, bar/modules/PetModule.qml, bar/widgets/DynamicIsland.qml, bar/island/controls/PetBlock.qml

### FaceService
Integração de autenticação facial/PAM, instalação, detecção e tentativa.

- Arquivo: services/FaceService.qml
- Imports: QtQuick, Quickshell, Quickshell.Io, Quickshell.Services.Pam
- Mecanismos: Process, PAM
- Funções: refresh, install, parse, list, settle, reason, add, remove, tryFace
- Propriedades: camera, installed, wired, known, ready, probe, installer, faces, listed, heard, lister, busy, outcome, complaint, changer, trial, tryout
- Comandos estáticos: "sh", "-c", "camera=0; for d in /dev/video*; do [ -c \"$d\"; "pkexec", "/usr/lib/velatura/face", "list"
- Consumidores QML (2): services/LockService.qml, settings/SessionSection.qml

