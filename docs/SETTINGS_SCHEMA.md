# Schema de configurações do Velatura original

Fonte: componente Store (JsonAdapter) em services/SettingsService.qml. O valor à direita é o default declarado pelo código, não necessariamente o valor atualmente salvo no perfil do usuário.

Chaves declaradas encontradas: 108.

> Não remover chaves por parecerem antigas. Algumas opções Hyprland são explicitamente mantidas para migração/perfis; distinguir legacy-unused de funcionalidade Niri atual. A migração precisa preservar chaves desconhecidas e nunca editar o arquivo original.

| Chave | Tipo QML | Default no código |
|---|---|---|
| clockFormat | string | "HH:mm" |
| clockShowsDate | bool | false |
| clockShowsSeconds | bool | false |
| barHeight | int | 32 |
| barMargin | int | 16 |
| islandAttached | bool | false |
| barSideMargin | int | 18 |
| barEverywhere | bool | true |
| barHidden | bool | false |
| screenCorners | bool | false |
| gameMode | bool | false |
| windowShadow | bool | false |
| barShadow | bool | false |
| dockShadow | bool | false |
| widgetShadow | bool | false |
| windowGlass | bool | false |
| surfaceStyle | string | "classic" |
| desktopGround | string | "classic" |
| barSides | string | "capsule" |
| wallpaperTransition | string | "wipe" |
| wallpaperDir | string | "" |
| matugenEnabled | bool | true |
| greeting | string | "random" |
| barStyle | string | "grouped" |
| barLeft | var | null |
| barRight | var | null |
| islandSummary | bool | true |
| lyrics | bool | true |
| islandActivities | var | null |
| islandActivitiesSeen | var | null |
| chipShape | string | "icon" |
| chipFigure | string | "on" |
| launcherResults | int | 9 |
| launcherOrder | string | "recent" |
| launcherFits | bool | false |
| emojiTone | int | 0 |
| cameraEnabled | bool | true |
| cameraEffect | string | "natural" |
| cameraReactions | var | ["thumbsup", "heart", "peace", "thumbsdown", "horns", "ok"] |
| clipboardHistory | bool | true |
| clipboardKeep | int | 200 |
| clipboardImages | bool | true |
| clipboardWipeOnLock | bool | false |
| lockBlur | int | 32 |
| lockClock | string | "stacked" |
| lockMusic | string | "lyrics" |
| lockMusicGround | string | "cover" |
| userName | string | "" |
| userAvatar | string | "" |
| doNotDisturb | bool | false |
| recorderAudio | bool | false |
| captureShape | string | "region" |
| captureKind | string | "photo" |
| notesHandwriting | bool | true |
| deckOnEmpty | bool | false |
| spectrumOnEmpty | bool | false |
| workspaceCount | int | 5 |
| workspaceMax | int | 10 |
| workspaceMode | string | "shared" |
| workspaceStyle | string | "dots" |
| motionScale | int | 100 |
| motionCurve | string | "OutCubic" |
| animationPreset | string | "macos" |
| fontFamily | string | "Inter, Cantarell, SF Pro Text, sans-serif" |
| fontMono | string | "JetBrainsMono Nerd Font Mono, monospace" |
| notificationTimeout | int | 5000 |
| compositor | var | ({}) |
| keyboard | var | ({}) |
| displays | var | ({}) |
| keys | var | ({}) |
| launcherPrefixes | var | ({}) |
| desktopWidgets | var | [] |
| desktopTheme | string | "modern" |
| desktopOpacity | int | 100 |
| desktopHidden | bool | false |
| centreButtons | var | null |
| centreTop | var | null |
| centreColumns | int | 0 |
| centreRows | int | 0 |
| centreBlocks | var | null |
| centreToggles | var | null |
| dockEnabled | bool | true |
| dockPinned | var | [] |
| dockEdge | string | "bottom" |
| dockAlignment | string | "center" |
| dockIconSize | int | 44 |
| dockMagnify | bool | true |
| dockOpacity | int | 100 |
| dockGround | string | "classic" |
| terminalGround | string | "glass" |
| dockRunning | bool | true |
| dockEverywhere | bool | true |
| dockAutohide | bool | false |
| dockLauncher | bool | true |
| weatherPlace | string | "" |
| githubUser | string | "" |
| markets | string | "BTC, ETH, EUR/USD" |
| petStyle | string | "creature" |
| idleLock | int | 0 |
| idleScreen | int | 0 |
| idleSuspend | int | 0 |
| nightLight | bool | false |
| nightTemperature | int | 4000 |
| lidPolicy | string | "off" |
| cursorColor | string | "palette" |
| cursorSize | int | 24 |
| shakeToFind | bool | true |
| language | string | "en" |

## Persistência observada

- Diretório: $XDG_STATE_HOME/quickshell; fallback ~/.local/state/quickshell.
- Arquivo principal: settings.json.
- SettingsService usa JsonAdapter e FileView; mudanças são persistidas automaticamente pelo adapter.
- O serviço mantém uma instância de defaults/pristine para comparação e reset.
- Existe machineKeyboard/arquivo separado para integração de teclado; auditar antes de decidir migração.
- Perfis exportam/importam settings e wallpaper/paleta associados via ProfileService.

## Política de migração 1:1

1. Mapear todas as chaves e tipos, inclusive listas/objetos com schema interno.
2. Preservar exatamente defaults e semântica.
3. Preservar valores desconhecidos, não truncar documentos.
4. Validar e recuperar de JSON inválido sem sobrescrever o original.
5. Criar backup antes de importar.
6. Não gravar no diretório quickshell durante testes.
7. Testar round-trip export/import e reset de cada grupo.

