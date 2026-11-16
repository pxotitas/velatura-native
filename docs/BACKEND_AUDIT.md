# Auditoria real do backend — Velatura Native

**Data:** 15/11/2026<br>
**Fonte primária:** `/home/px/.config/quickshell` (somente leitura)<br>
**Escopo desta rodada:** entry point, registro de singletons, Niri, outputs/monitores, preferências, áudio, rede, Bluetooth, bateria, brilho, notificações, MPRIS, clipboard e tray.<br>
**Status:** auditoria estática dos arquivos citados concluída nesta fatia; efeitos reais ainda precisam de testes de paridade. Nenhum item abaixo é IMPLEMENTED ou VERIFIED.

## Regras de leitura

- Arquivos ativos foram identificados pelo entry point e pelo `qmldir`; `.new`, `.bak` e `.before-*` não foram usados como fonte primária.
- A presença de um singleton no `qmldir` registra seu tipo, mas não significa que ele inicia no boot: o comentário do `shell.qml` declara criação lazy e cache no primeiro uso.
- A instalação original não foi alterada. Nenhum comando de configuração do Niri, autostart ou processo do Quickshell foi executado.
- "AUDITED" nesta rodada significa comportamento, estado, dependências e falhas lidos no código; não significa equivalência Rust ou validação funcional.

## Entry point e ciclo de vida — AUDITED

**Fonte:** `shell.qml`, `qmldir`.

1. **Responsabilidade:** construir as superfícies por output, escolher a ilha live, conectar eventos entre serviços que não devem depender diretamente uns dos outros e expor o IPC por `Commands`.
2. **API/consumidores:** `Bar`, `BarReserve`, `Desktop`, `Dock`, `ScreenCorners`, `Deck`, `SettingsWindow`, `LockScreen`, `CaptureOverlay` e `Commands`; conecta `DesktopService`, `ModuleService`, `PolkitService`, `LockService`, `ClipboardService`, `CaptureService`, `RecorderService`, `WallpaperService` e `ThemeService`.
3. **Estado inicial:** `islandName` começa em `MonitorService.effectivePrimaryName`; `wantedIslandName` acompanha `NiriService.focusedOutput` se o output existir, senão usa o fallback efetivo; `liveScreenName` faz fallback se o output escolhido desaparecer.
4. **Eventos/frequência:** `Quickshell.screens` alimenta `Variants`; mudanças de foco pedem troca da ilha; enquanto um painel está expandido a troca é adiada; ao fechar, `settleIsland()` tenta novamente. Não há timer de polling no entry point.
5. **Dependências:** Quickshell/QtQuick, screens Wayland e os serviços declarados no `qmldir`.
6. **Permissões:** depende das permissões de cada serviço; o entry point não solicita privilégios por si.
7. **Persistência:** delegada aos serviços; não grava estado global diretamente.
8. **Falhas/reconexão:** se o output live some, resolve para `effectivePrimaryName`; conexões cruzadas mantêm dependências acíclicas. Recuperação específica é responsabilidade de cada serviço.
9. **Teste de paridade:** desconectar/reconectar output, trocar foco durante e fora de painel, remover output live, manter exatamente uma ilha live, testar surfaces por monitor e confirmar que Settings continua xdg-toplevel.
10. **Equivalência Rust:** coordenador de lifecycle + registry de outputs + roteador de eventos. A identidade da ilha deve ser estado de domínio independente da janela GTK; não destruir/recriar a ilha ao trocar monitor.

**Boot observado (grafo direto do shell.qml):** `Component.onCompleted` toca explicitamente `ThemeService.activeId`, `MonitorService.monitors`, `PolkitService.asking`, `SunsetService.available`, `IdleService.lockAfter` e `SessionService.sleepPending`. Bindings/targets diretos no root incluem `NiriService`/`SettingsService` (propriedades e screens), `DesktopService`, `ModuleService`, `PolkitService`, `LockService`, `CaptureService` e `WallpaperService` (Connections/targets). `WallpaperService` inicia guard/list/animated/get-state/restore e pode iniciar `awww-daemon` como child do Quickshell somente se não encontrar dono externo; isso não cria unit/autostart. `CaptureService` faz probe de ferramentas no startup. `DesktopService` e `LockService` entram no grafo mesmo sem as superfícies abertas. `LauncherService`, `PickerService`, `ClipboardService` e `RecorderService` aparecem em corpos de callbacks/handlers; a criação pode ocorrer só quando esses callbacks forem executados, não presumir eager load sem tracing. O comentário de `qmldir` diz que outros singletons são lazy/cacheados no primeiro uso; a ordem real ainda precisa de tracing/runtime tests. Stats e ScreenTime não aparecem como targets/refs diretos do root; podem ser criados por consumers/configuração, não se deve presumir boot.

## NiriService — AUDITED

**Fonte:** `services/NiriService.qml`.

1. **Responsabilidade:** espelhar workspaces, janelas e foco a partir do event stream; executar ações Niri serializadas.
2. **API/consumidores:** estado `workspaces`, `windows`, `focusedWorkspaceId`, `activeWorkspaceId`, `focusedWindow`, `focusedOutput`, `clients`, `occupiedIds`; helpers de workspace/janela; sinais `eventReceived`, `windowOpened`, `windowClosed`.
3. **Estado inicial:** arrays vazios, IDs/foco nulos, `connected=false`, `lastError=""`, fila de ações vazia.
4. **Eventos:** `niri msg --json event-stream` em processo contínuo, uma linha JSON por evento. Reconhece `WorkspacesChanged`, `WindowsChanged`, `WindowOpenedOrChanged`, `WindowClosed`, `WindowFocusChanged`, `WorkspaceActivated`, `WindowUrgencyChanged`, `WorkspaceUrgencyChanged`, `WorkspaceActiveWindowChanged` e `WindowLayoutsChanged`. O restante ainda é encaminhado por `eventReceived` sem transformação de domínio.
5. **Dependências:** binário `niri`; event stream e `niri msg action`. Ações enviadas como argv separado, não shell concatenado.
6. **Permissões:** IPC da sessão do usuário; sem elevação.
7. **Persistência:** nenhuma.
8. **Erros/reconexão:** JSON inválido define `lastError`; resposta `Err` marca desconectado; saída não-zero agenda reconexão em 3000 ms. A fila de ações executa uma por vez; falha só gera log, sem retry e sem retorno estruturado ao chamador. Saída zero inesperada do event stream não agenda reconnect no código atual — ponto de paridade a reproduzir ou resolver mediante decisão.
9. **Teste:** fixtures por tipo de evento, linhas inválidas, stream fechado com exit 0/non-zero, rajada de eventos, ação falha, fila serial, janela aberta/alterada/fechada, urgência e foco.
10. **Rust:** implementar parser do protocolo/event stream real, stream cancelável, reconnect com backoff, snapshot inicial coerente, fila tipada de ações e erros observáveis. Não inventar mensagens IPC.

## MonitorService e script monitors.py — AUDITED

**Fontes:** `services/MonitorService.qml`, `scripts/monitors.py`.

1. **Responsabilidade:** descobrir outputs Niri, manter metadados de monitores e resolver monitor efetivo/fallback.
2. **API/consumidores:** `monitors`, `loaded`, `lastError`, `primaryName`, `effectivePrimaryName`, `primaryScreen`, `internalName`, `monitorFor()`, `descriptionFor()`, `isPrimary()`.
3. **Estado inicial:** lista vazia, `loaded=false`; primeiro carregamento no `Component.onCompleted`.
4. **Eventos/frequência:** refresh por `WorkspacesChanged`, `Quickshell.screensChanged` e `geometryChanged`, com debounce de 120 ms. Se leitura já estiver ativa, faz uma leitura posterior pendente. Não existe polling periódico.
5. **Dependências:** `python3 scripts/monitors.py get`, `niri msg --json outputs`, timeout de 5 s. Script converte modos, taxa de atualização (mHz para Hz), geometria lógica, escala, transform, VRR e metadados make/model/serial.
6. **Permissões:** IPC read-only do Niri.
7. **Persistência:** nenhuma; perfis persistentes de display são delegados ao Kanshi/configuração Niri.
8. **Erros:** falha do processo ou JSON inesperado é exposta em `lastError`; sem retry temporizado explícito. O script não altera outputs.
9. **Teste:** outputs ausentes/desabilitados, campos nulos, modos inválidos, hotplug, geometry change, foco em output não descoberto, monitor interno e output virtual.
10. **Rust:** substituir o helper Python por adapter Niri tipado; manter a leitura read-only, debounce e dados completos de output.

**Semântica importante:** `primaryName` não representa um "primary output" independente configurado pelo compositor: escolhe o output focado quando encontrado, senão o primeiro output não desabilitado. O original não mantém três identidades independentes para focused/primary/island; o port deve separar explicitamente esses conceitos sem alterar a regra visual de fallback. `internalName` reconhece nomes iniciados por `eDP`, `LVDS` ou `DSI`.

## SettingsService e persistência — AUDITED

**Fonte:** `services/SettingsService.qml`.

1. **Responsabilidade:** schema completo de preferências, defaults, aliases públicos, layouts configuráveis, perfis e persistência.
2. **API/consumidores:** aliases para as propriedades do `Store`; `set`, `snapshot`, `complete`, `adopt`, `reset`, `barItems`, `barZone`, `setBarZone`, `beside`, `setBeside`, `launcherPrefix`, `setLauncherPrefix`, além de validação/cópia de dados.
3. **Estado inicial:** defaults declarados no `JsonAdapter Store`; `arrived=false` bloqueia writes antes da leitura; `fresh` indica arquivo ausente.
4. **Eventos/frequência:** `FileView.watchChanges=true`; mudança externa chama reload; atualização do adapter reinicia saver com intervalo 0 para coalescer escritas no fim do turno.
5. **Dependências:** `$XDG_STATE_HOME/quickshell/settings.json`, fallback `$HOME/.local/state/quickshell/settings.json`; lê `/etc/X11/xorg.conf.d/00-keyboard.conf` somente para obter layout inicial se o arquivo não existir.
6. **Permissões:** escrita no diretório de estado do usuário; leitura do arquivo X11 de configuração global.
7. **Persistência/migração:** schema tipado no QML; perfil exclui `machineKeys`; `complete()` completa valores ausentes com defaults e rejeita tipos divergentes; `adopt()` aplica um perfil; `reset()` mantém machine keys. Migra `frosted` para `glass`, herda ground vazio, copia `barShadow` para `dockShadow` ausente e converte `barFullWidth=true` + `barStyle=island` para `bezel`.
8. **Erros/concorrência:** não escreve antes de carregar; desconhecidos/tipos inválidos são rejeitados no helper de validação; JSON inválido e recuperação do FileView precisam de testes específicos. O saver coalesce múltiplas mudanças e o código recusa writes antes de `arrived`.
9. **Teste:** defaults completos, round-trip de todos os tipos, campos desconhecidos preservados na migração nativa, arquivo ausente/corrompido, edição externa, concorrência/reload, perfil parcial, reset sem machine keys e compatibilidade com chaves legadas.
10. **Rust:** schema versionado sem descartar chaves desconhecidas, escrita atômica, backup/import consentido e migração reversível. Não copiar apenas os defaults visuais; extrair todas as propriedades do Store.

**Categorias persistidas:** relógio/barra/ilha/chips, módulos, launcher, câmera, clipboard, lock, DND, capture/recorder, deck/spectrum, workspaces, motion, fontes, notificações, compositor/teclado legados, displays, keys, desktop widgets, control center, dock, weather/GitHub/markets, pet, idle, night light, cursor e idioma. Alguns campos são legado Hyprland e continuam no schema para migração, embora não sejam consumidos pelo Niri.

## AudioService — AUDITED

**Fonte:** `services/AudioService.qml`.

1. **Responsabilidade:** áudio PipeWire de saída/entrada, sinks, sources e streams de aplicações.
2. **API/consumidores:** volume/mute do sink default, source default, lista de outputs, inputs, streams de playback e capture, escolha de sink/source, volume/mute por stream.
3. **Estado inicial:** depende de `Pipewire.defaultAudioSink/defaultAudioSource` e readiness; sem sink pronto volume=0 e muted=false.
4. **Eventos:** reativo via PipeWire; `PwObjectTracker` acompanha sink/source, outputs, streams e inputs. Sem polling no serviço.
5. **Dependências:** Quickshell Pipewire API (backend real PipeWire/WirePlumber), nodes e propriedades PipeWire.
6. **Permissões:** acesso da sessão ao PipeWire.
7. **Persistência:** seleção default delegada ao provider; o serviço não grava estado próprio.
8. **Erros:** ausência de sink/source é representada por `ready=false`; não há `lastError` próprio.
9. **Teste:** device ausente, troca de default, mute/volume externo, streams surgindo/desaparecendo, capture vs monitor stream, volume por aplicação, múltiplos sinks e source.
10. **Rust:** provider PipeWire reativo com capacidades e eventos; preservar distinção entre áudio de saída, entrada, monitor de sink e apps capturando áudio.

## NetworkService e network.py — AUDITED

**Fontes:** `services/NetworkService.qml`, `scripts/network.py`.

1. **Responsabilidade:** estado de conexão, rádio Wi-Fi, redes visíveis e ações connect/disconnect/forget.
2. **API/consumidores:** devices, Wi-Fi/Ethernet, `online`, `connectionName`, `stateLine`, `strength`, `networks`, `scanning`, `busySsid`, `lastError`, `connectionFinished`, `toggleWifi`, `setWifi`, `scan`, `connect`, `disconnect`, `forget`.
3. **Estado inicial:** `radioOn=false`, lista vazia, sem ação pendente; o estado de devices vem de Quickshell Networking.
4. **Eventos/frequência:** estado de devices/conectividade reativo; consulta explícita do rádio por `nmcli -t -f WIFI radio all`; ao ligar rádio aguarda 1200 ms e faz scan. Scan não é periódico.
5. **Dependências:** NetworkManager via Quickshell Networking + `nmcli`; `network.py` usa saída terse com campos nomeados, timeout padrão 25 s e 60 s para conectar; mescla APs pelo SSID e melhor sinal.
6. **Permissões:** D-Bus/NetworkManager e permissões de rede da sessão; conexões podem exigir agente de segredos/autenticação.
7. **Persistência:** perfis de conexão pertencem ao NetworkManager; o serviço não armazena senha em settings.
8. **Erros/segurança:** mensagens do script são sanitizadas para UI; erro de conexão sem stderr recebe fallback. **Risco legado:** o QML passa senha como argumento do processo (`network.py connect SSID PASSWORD`), potencialmente visível em argv/process listing. A versão Rust não deve repetir isso; usar API D-Bus/secret agent ou canal não exposto em argv. Falta timeout explícito no processo QML, embora o script tenha timeout interno.
9. **Teste:** SSID com dois-pontos/backslash, redes repetidas por AP/banda, SSID vazio, rádio desligado/sem device, senha incorreta, rede aberta, connect/disconnect/forget, NetworkManager ausente e operação cancelada.
10. **Rust:** zbus/NetworkManager como caminho primário; manter os estados e a semântica de scan/merge; não usar shell concat.

## BluetoothService — AUDITED

**Fonte:** `services/BluetoothService.qml`.

1. **Responsabilidade:** adapter BlueZ, estado de energia, discovery e dispositivos conhecidos.
2. **API/consumidores:** `available`, `enabled`, `allDevices`, `connectedDevices`, `summary`, `devices`, `unnamedDevices`, `unnamedCount`, `discovering`, `deviceIcon`, `connectDevice`, `setDiscovering`, `toggle`.
3. **Estado inicial:** adapter default; `available=false` quando ausente.
4. **Eventos:** propriedades do adapter/devices via Quickshell Bluetooth; não há timer ou polling declarado.
5. **Dependências:** BlueZ D-Bus através do módulo Bluetooth do Quickshell.
6. **Permissões:** autorização BlueZ/PolicyKit quando exigida pelo daemon.
7. **Persistência:** pareamento é mantido por BlueZ; sem storage próprio.
8. **Erros:** adapter ausente vira "Unavailable"; ações são no-op sem adapter; erros de operação não são normalizados pelo serviço.
9. **Teste:** daemon/adapter ausente, adapter desligado, discovery, device emparelhado sem nome, dispositivo desconhecido, connect/disconnect, device removido e permissão negada.
10. **Rust:** zbus BlueZ ObjectManager, adapter e devices reativos; preservar separação de devices listados e lista recolhida de dispositivos sem nome.

## BatteryService — AUDITED

**Fonte:** `services/BatteryService.qml`.

1. **Responsabilidade:** nível, estado, autonomia e saúde da bateria.
2. **API/consumidores:** `available`, `percent`, `state`, `charging`, `full`, `low`, `secondsToEmpty`, `secondsToFull`, `estimate`, `watts`, `energy`, `energyCapacity`, `healthKnown`, `health`, `stateWord`, `icon`, `tint`.
3. **Estado inicial:** UPower display device; disponível somente se device ready e laptop battery.
4. **Eventos:** atualizações reativas UPower; sem polling.
5. **Dependências:** Quickshell UPower / D-Bus UPower.
6. **Permissões:** acesso D-Bus da sessão.
7. **Persistência:** nenhuma.
8. **Erros:** máquina sem bateria ou device não pronto = indisponível; estimate vazio quando UPower não fornece tempo; health somente se suportado.
9. **Teste:** desktop sem bateria, laptop, charging/full/discharging/unknown, tempo zero, health não suportada, valores fora do intervalo e troca de bateria.
10. **Rust:** zbus UPower com propriedades opcionais e sem assumir que toda máquina tem bateria.

## BrightnessService — AUDITED

**Fonte:** `services/BrightnessService.qml`.

1. **Responsabilidade:** brilho por monitor, painel interno via backlight sysfs e externos via DDC/CI.
2. **API/consumidores:** displays/variants, dimmable/current, percent/icon, `setPercent`, `step`, `refresh`, `adjusted`; painel de detalhes quando há mais de um display ajustável.
3. **Estado inicial:** descobre uma vez o primeiro diretório em `/sys/class/backlight`; lê max/brightness; detecta DDC apenas se houver output externo.
4. **Eventos/frequência:** FileView acompanha mudança sysfs; hotplug reinicia detecção DDC após 1500 ms; leituras DDC sob demanda/ao abrir, não polling contínuo; writes DDC são serializados e coalescem para o último valor desejado.
5. **Dependências:** sysfs, `brightnessctl`, `ddcutil detect --brief`, `ddcutil getvcp/setvcp 10`, I2C; nomes de conectores Niri mapeados a buses.
6. **Permissões:** acesso a sysfs/backlight, `/dev/i2c-*` e regras/grupos locais; não elevar privilégios automaticamente.
7. **Persistência:** nível real pertence ao dispositivo; serviço não grava preferência.
8. **Erros:** write DDC falha -> leitura de recuperação; monitor que não responde fica indisponível; falha de backlight não tem estado de erro estruturado. Comandos de discovery usam shell estático sem entrada de usuário.
9. **Teste:** ausência de backlight/DDC, vários monitores, hotplug, monitor que não responde, hardware keys, slider rápido, monitor único via ddcci e escala/percentual.
10. **Rust:** providers separados para sysfs e DDC, detectar capacidades por output, serializar I/O lento e expor status por monitor.

## NotificationService — AUDITED

**Fonte:** `services/NotificationService.qml`.

1. **Responsabilidade:** atuar como daemon `org.freedesktop.Notifications`, exibir notificação na ilha e manter histórico.
2. **API/consumidores:** `current`, `history`, `active`, `critical`, sinais `arrived`/`received`, `defaultAction`, `buttonsOf`, `invoke`, `open`, `dismiss`, `close`, `clearHistory`, `remove`, DND.
3. **Estado inicial:** current nulo, histórico restaurado de `notifications.json`; servidor declara suporte a body, markup, image, actions e persistence.
4. **Eventos/frequência:** evento de notificação D-Bus; timeout individual começa na chegada; update de summary/body reinicia o timer. Críticas não expiram automaticamente; DND suprime apresentação mas registra histórico.
5. **Dependências:** Quickshell NotificationServer, desktop entries e Theme icons.
6. **Permissões:** nome D-Bus `org.freedesktop.Notifications`; se outro daemon possuir o nome, o original espera takeover quando ele liberar.
7. **Persistência:** `$XDG_STATE_HOME/quickshell/notifications.json`; limite 50; payload de imagem temporário não persistido como qsimage.
8. **Erros:** JSON de histórico corrompido gera warning; não sobrescreve antes de restore; crítica vence notificação normal; ações podem não existir e `open()` retorna false.
9. **Teste:** outro daemon ativo, takeover, urgente/normal/low, timeout customizado e cap 15 s, crítica sem timeout, DND, transient, ações default/botões, update em place, history limit e restart.
10. **Rust:** implementar freedesktop notifications server e lifecycle de notifications; preservar action invocation, hints, urgency, markup, timeout, history e ownership do nome D-Bus.

## MediaService — AUDITED

**Fonte:** `services/MediaService.qml`.

1. **Responsabilidade:** escolher um player MPRIS representativo e oferecer controles/metadata.
2. **API/consumidores:** players filtrados, active, available/playing, title/artist/album/artUrl/identity, capabilities, length/position/progress, `subscribe`, `release`, `toggle`, `next`, `previous`, `seek`.
3. **Estado inicial:** escolhe primeiro player tocando; caso contrário primeiro player controlável com título. Player parado sem título é ignorado.
4. **Eventos/frequência:** metadata/status reativos MPRIS; posição não é push, então chama `positionChanged()` a cada 1000 ms apenas se watchers > 0, tocando e seekable.
5. **Dependências:** MPRIS D-Bus.
6. **Permissões:** sessão D-Bus.
7. **Persistência:** nenhuma.
8. **Erros:** ausência de player = unavailable; capabilities impedem operações não suportadas; streams sem length não mostram progress.
9. **Teste:** vários players, troca de player ativo, paused vs playing, player vazio, seek/capabilities, stream sem length e subscription count.
10. **Rust:** zbus MPRIS com seleção determinística e polling de posição estritamente condicionado aos consumidores.

## ClipboardService e clipboard.py — AUDITED

**Fontes:** `services/ClipboardService.qml`, `scripts/clipboard.py`.

1. **Responsabilidade:** histórico de clipboard texto/imagem, pesquisa, restore, forget e wipe.
2. **API/consumidores:** `entries`, `count`, `normalise`, `copy`, `forget`, `wipe`, `search`, formatação de bytes/tempo e título.
3. **Estado inicial:** lê `clipboard.json`; ausência resulta em lista vazia; watcher começa conforme settings.
4. **Eventos/frequência:** `wl-paste --watch` executa store em cada alteração; FileView observa JSON. Configuração keep/images/histórico reinicia watcher; proteção impede respawn rápido repetido.
5. **Dependências:** `wl-paste`, `wl-copy`, wlr data-control, Python, SHA-256, flock; dados em `$XDG_STATE_HOME/quickshell/clipboard.json` e payloads em `clipboard/`.
6. **Permissões:** acesso ao protocolo clipboard Wayland e diretório de estado.
7. **Persistência:** até 200 entradas por default; payload separado, 16 MiB máximo; MIME preservado; imagens opcionais; index substituído atomicamente; lock exclusivo para processos concorrentes.
8. **Privacidade/erros:** descarta `CLIPBOARD_STATE != data`, MIME `x-kde-passwordManagerHint`, payload vazio, texto whitespace e acima de 16 MiB. Watcher PID guard verifica cmdline antes de sinalizar processo antigo. Wipe remove histórico e limpa seleção ativa. Falhas de wl-paste/wl-copy são stderr; cópia de password manager nunca é armazenada.
9. **Teste:** texto, imagens, MIME, payload grande, clipboard sensitive, clipboard clear/nil, cópias repetidas (dedup), restore/forget/wipe, concorrência, watcher órfão e restart.
10. **Rust:** preferir API Wayland/portal compatível com data-control disponível; preservar MIME, filtros de segredo, lock/atomicidade, payload binário e wipe-on-lock. Não reduzir a texto puro.

## TrayService — AUDITED

**Fonte:** `services/TrayService.qml`.

1. **Responsabilidade:** listar StatusNotifierItems não passivos e renderizar menus no painel da ilha.
2. **API/consumidores:** items, item aberto, stack de submenus (profundidade máxima 2), entries, `open`, `enter`, `back`, `release`, `activate`, cálculo de dimensões.
3. **Estado inicial:** nenhum item/menu selecionado; itens com status Passive são filtrados.
4. **Eventos:** coleção reativa do SystemTray; se item desaparece, menu é liberado; caminho do submenu é resolvido novamente para sobreviver a reconstrução de menus.
5. **Dependências:** Quickshell SystemTray, StatusNotifierItem, DBusMenu e Theme icons.
6. **Permissões:** sessão D-Bus; sem privilégio.
7. **Persistência:** nenhuma.
8. **Erros:** item removido fecha menu; menu ausente não abre; limita profundidade em dois níveis.
9. **Teste:** item passive/active, item que só tem menu, ativação direta, submenu, menu reconstruído durante scan, separators e item removido.
10. **Rust:** StatusNotifierWatcher + DBusMenu, preservar árvore dinâmica, activation/context menu e lifecycle de item.

## Ambiente Niri observado nesta auditoria

Consulta read-only executada em 15/11/2026:

- Niri: `26.04 (v26.04-276-g59351300)`.
- `niri msg --help` confirma comandos `outputs`, `workspaces`, `windows`, `event-stream`, `action` e `raw-request`.
- `niri msg --json outputs` retornou dois outputs ativos no momento: `Virtual-1` em x=-1920, 1920×1440 lógico, e `eDP-1` em x=0, 1440×900 lógico. Isso confirma que `Virtual-1` faz parte do ambiente que a portabilidade deve preservar.
- Nenhuma configuração foi modificada e nenhuma ação Niri foi enviada.

## Pontos de atenção descobertos

1. **Focused vs primary:** o original deriva "primary" do foco com fallback; não é uma terceira identidade independente. O backend nativo deve expor explicitamente focused output, fallback/primary efetivo e ilha live, documentando a política.
2. **Reconexão Niri:** o original só agenda reconnect em exit code não-zero. Saída limpa inesperada deixa `connected=false` sem timer; a paridade e robustez nativa precisam tratar esse caso deliberadamente.
3. **Senha Wi-Fi:** o original coloca senha em argv do Python; não reproduzir esse vetor de exposição.
4. **Brightness DDC:** DDC é I/O lento, com coalescing de último valor e leitura de recuperação; não transformar slider em chamadas concorrentes.
5. **Clipboard:** segurança depende de MIME sensitive + `CLIPBOARD_STATE`, além de não guardar clipboard de gerenciador de senha. Esse filtro é parte obrigatória da paridade.
6. **Notification ownership:** takeover do nome D-Bus é comportamento do original, não apenas "receber notificações".
7. **Legacy settings:** campos Hyprland continuam no schema para compatibilidade/migração, embora Niri não os consuma; não apagar chaves desconhecidas.

## Próxima fatia de auditoria

- Ler integralmente `Commands.qml` por target e comparar com `docs/IPC_API.md`.
- Auditar serviços de sessão, polkit, idle, sunset/night light, compositor, launcher, applications, theme/wallpaper e system.
- Ler integralmente todos os scripts Python ativos, não apenas os chamados pelos serviços desta fatia.
- Auditar superfícies e todos os serviços restantes; seguir consumers/import graph para separar startup, lazy e históricos.
- Só depois fechar o contrato público Rust para Codeína; mocks devem nascer desse contrato auditado.

## CaptureService, RecorderService e scripts — AUDITED

**Fontes:** `services/CaptureService.qml`, `services/RecorderService.qml`, `scripts/capture.py`, `scripts/record.py`.

### Capture / screenshot / OCR

1. **Responsabilidade:** capturar uma imagem da tela focada, desenhar seleção sobre a imagem congelada, recortar em pixels físicos e entregar a arquivo, clipboard, editor ou OCR.
2. **API/consumidores:** `tools`, `ready`, `photo`, `photoWidth/Height`, `active`, `busy`, `screenName`, `screen`, `ratio`, `shapes`, `kinds`, `destinations`, `shape`, `kind`, `to`, `open`, `cancel`, `fire`, `box`, `crop`; sinais `opened` e `recordRequested`.
3. **Estado inicial:** probe `capture.py tools` uma vez; shape/kind persistem em Settings; destino volta para file em cada abertura. Seleciona `NiriService.focusedOutput`, mantendo a tela durante a captura.
4. **Eventos/lifecycle:** open bloqueia se active/busy, espera `after` (settle da ilha), captura, abre overlay; cancel remove foto temporária; fire oculta overlay antes de processar. Video descarta still e emite `recordRequested`; foto chama `finish`.
5. **Dependências:** `grim`; ImageMagick 7 `magick` ou 6 `convert`; `wl-copy`; `satty` para editor; `tesseract` para OCR; `xdg-user-dir`; diretório `VELATURA_CAPTURES` ou XDG Pictures/HOME. O helper usa stdin DEVNULL, argv separado e timeout de até 120 s.
6. **Permissões:** captura via protocolo disponível no compositor/Wayland; clipboard Wayland; escrita em pasta de imagens. Sem elevação.
7. **Persistência:** captura final timestamped `*_velatura.png`; diretório padrão observado pelo probe é XDG Pictures; still temporário em XDG_RUNTIME_DIR/tmp e removido em cancel/crop.
8. **Erros:** tool ausente é reportada e editor/text deixam de ser oferecidos; falha de grab/crop/save/copy/OCR gera JSON de erro/resultado e OSD; `ready` só vira true após JSON parseável. `window` não é shape ativo: `shapes` é apenas region/screen; setting legado `window` é convertido para region.
9. **Teste:** cada tool ausente, output focado, escala física/lógica, retângulo fora de bounds, cancel, whole screen, region, destinos, OCR sem texto, imagem no clipboard e arquivo salvo/copied.
10. **Rust:** providers de screencopy/portal e clipboard; manter captura da tela focada e transformação logical→physical; preservar destinos e OCR como integração opcional, sem fingir suporte a janela.

### Recorder

1. **Responsabilidade:** iniciar/parar uma gravação única, sobreviver ao reload do shell e expor duração/tamanho.
2. **API/consumidores:** `available`, `tool`, `canAudio`, `recording`, `startedAt`, `seconds`, `shape`, `sound`, `size`, `path`, `name`, `subject`, `display`, `startAt`, `stop`, `toggle`, `measure`, `ingest`.
3. **Estado inicial:** probe `record.py tools`; se encoder existe, consulta `status` para recuperar take ativo; duração deriva de startedAt (não acumulador).
4. **Eventos/frequência:** relógio 1 s enquanto grava; status/tamanho consultado quando detalhe pede (`measure`), não polling contínuo no serviço. `startAt` pode receber screen/region/window + geometry.
5. **Dependências:** prioridade `wl-screenrec` (GPU) e fallback `wf-recorder`; `pactl get-default-sink` para capturar sink monitor; `xdg-user-dir VIDEOS`; arquivo de estado `XDG_RUNTIME_DIR/velatura-recording.json`.
6. **Permissões:** captura Wayland e acesso PipeWire/PulseAudio; saída em pasta Videos; sem privilégio.
7. **Persistência:** gravação em MP4 com timestamp; PID/path/started/tool/audio/shape em arquivo runtime, permitindo take continuar após shell restart.
8. **Erros:** se encoder morre nos primeiros 250 ms, tenta próximo; sem encoder reporta indisponível. Stop envia SIGINT para finalizar container, espera até 5 s, remove state file e retorna duração/tamanho. O arquivo de estado falhar ao gravar gera stderr mas o processo pode continuar.
9. **Teste:** encoder primário/fallback, saída imediata, shell restart no meio da gravação, SIGINT finalize, audio source ausente, geometria/output, stop repetido e stale PID.
10. **Rust:** processo de gravação com lifecycle explícito e detached ownership; armazenar metadados runtime com validação PID/command identity, não assumir que shell restart encerra take.

### Probes read-only executados no ambiente atual

- `capture.py tools`: grim=true, crop=true (ImageMagick), clipboard=true (wl-copy), editor=false (satty ausente), text=true (tesseract); diretório `/home/px/Imagens`.
- `record.py tools`: encoder selecionado `wf-recorder`; `wl-screenrec=false`, `wf-recorder=true`, audio=true (monitor source via pactl); diretório `/home/px/Vídeos`.
- Foram apenas consultadas capacidades; nenhuma captura ou gravação foi iniciada.

### Divergências CLI relacionadas à captura

- `velatura capture window` encaminha `window` ao IPC, mas CaptureService só aceita `region`/`screen`; além disso, `capture.py` não implementa seleção de janela.
- `velatura capture region` encaminha só um argumento a `capture.open(shape,destination)`; o handler declara dois argumentos e o destino vazio é tratado como file. Confirmar comportamento do IPC com fixture antes de alterar.
- A API de capture QML oferece photo/video, region/screen e file/clipboard/editor/text; CLI local screen/text usa fluxo direto de grab+finish.

## LauncherService e ApplicationsService — AUDITED

**Fontes:** `services/LauncherService.qml`, `services/ApplicationsService.qml`, `scripts/applications.py`.

### Applications index

1. **Responsabilidade:** compartilhar índice de Desktop Entries entre launcher e dock sem forçar criação do LauncherService.
2. **API/consumidores:** `applications`, `refresh()`, `launchDesktop(desktopId)`; LauncherService e DockService consomem o índice.
3. **Estado inicial:** lista vazia; indexer inicia 300 ms após criação para permitir primeiro frame da dock.
4. **Eventos:** carga única atrasada e refresh explícito; não há watcher de diretórios nem atualização automática de arquivos .desktop.
5. **Dependências:** Python configparser; XDG_DATA_HOME (fallback ~/.local/share) e XDG_DATA_DIRS (fallback /usr/local/share:/usr/share); `gtk-launch` para abrir.
6. **Permissões:** leitura de diretórios de aplicações do usuário/sistema.
7. **Persistência:** não mantém cache próprio; reconstrói a lista em memória.
8. **Filtros/erros:** aceita somente Type=Application, ignora NoDisplay/Hidden, exige Name; primeira ocorrência por filename vence conforme precedência XDG; erros de diretório vão para stderr; parse inválido de arquivo é ignorado.
9. **Teste:** user override sombreia system entry, diretório ausente/sem permissão, Desktop Entry inválida, Hidden/NoDisplay, categorias, WMClass, ícone vazio e refresh.
10. **Rust:** parser Desktop Entry freedesktop com precedence XDG e sem perder campos usados pelo dock/launcher/screen time; refresh e launch separados.

### Launcher

1. **Responsabilidade:** busca e ativação de aplicações e modos contextuais do launcher.
2. **API/consumidores:** `query`, `modes`, `modeFor`, `termFor`, `results`, `search`, `rank`, `calculate`, `desk`, `timer`, `clipboard`, `emoji`, `windows`, `activate`, `copy`, `launchDesktop`, `refresh`; usa ApplicationsService, SettingsService, ControlsService, TimerService, ClipboardService, EmojiService e NiriService.
3. **Estado inicial:** query vazia; modos ordenados com prefixos configuráveis; default apps é o último modo. Prefixos default: `=` calculate, `>` desk, `@` windows, `!` timer, `'` clipboard, `:` emoji.
4. **Eventos/frequência:** busca derivada da query; history FileView observa `launcher-history.json`; não há timer de busca; geometria depende de results/maxResults/launcherFits.
5. **Dependências:** `gtk-launch`, `wl-copy`, XDG desktop entries, Niri windows/workspaces, clipboard, emoji, timer e control center.
6. **Permissões:** launch na sessão do usuário; clipboard Wayland; leitura/escrita em XDG_STATE_HOME.
7. **Persistência:** `launcher-history.json` guarda desktop ID → count/last; half-life 30 dias; favoritos vêm de dockPinned e recebem bônus +1; ordem pode ser recent ou alphabetical.
8. **Erros/limites:** cálculo só aceita regex numérica `[0-9+\-*/(). %]+`; QML usa `Function` para avaliar e rejeita não finito/exception. O Rust não pode carregar JS runtime: implementar parser aritmético próprio com mesma gramática, precedência e casos de erro. Busca de apps considera prefixo, substring de nome e substring de keywords; janelas por title/app_id; desk lista portas, modos e somente toggles one-shot `closes && available`. Launcher não executa painel/mode diretamente: UI faz essa parte.
9. **Teste:** cada prefix, prefix collision/customização, query vazia, frecency/half-life, favoritos e empate, alphabetical, calculadora (parênteses, %, divisão por zero), janelas e workspace, clipboard desligado, ação unavailable e histórico corrompido.
10. **Rust:** serviço de domínio sem UI, query state e resultados tipados por `kind`; parser matemático nativo; frecency persistente; ativação com efeitos reais e callbacks tipados para painel/mode.

### Observações de fidelidade

- Aplicações são compartilhadas com DockService e não devem ser carregadas somente quando o launcher abre.
- `launchDesktop` usa `gtk-launch` via execDetached; launcher registra frecency antes de abrir, enquanto DockService launch não conta como launcher use.
- `desk` exclui toggles com estado (tile.closes=false), launcher itself e recursos indisponíveis; lista não é truncada em maxResults, apenas painel limita altura.
- Emoji e clipboard podem devolver mais resultados que maxResults; lista/grid virtualiza conteúdo visível.

## DockService — AUDITED

**Fonte:** `services/DockService.qml` (386 linhas).

1. **Responsabilidade:** combinar apps fixados e apps com janelas abertas, casar classes de janela com desktop entries, gerir pin/unpin/reorder, ações e geometria.
2. **API/consumidores:** settings enabled/edge/alignment/running/autohide/launcher/everywhere; pinned, applications, classIndex, groups, items, count, pinnedCount, divides, geometry (`box`, `offsetOf`, `length`, retreat/hiddenX/Y), drag (`beginDrag`, `shifted`, `slotAt`, `endDrag`), actions (`activate`, `launch`, `closeAll`).
3. **Estado inicial:** pinned vem de SettingsService.dockPinned; cópia local evita ler JsonAdapter stale durante drag/reorder.
4. **Eventos/frequência:** NiriService.clients é a fonte de janelas; grupos recalculados reativamente. SettingsService changes são observadas; persistência de pin list debounce 120 ms. Não há watcher separado de clients.
5. **Dependências:** ApplicationsService desktop entries, NiriService clients/focus/close, SettingsService, Theme tokens e launcher callback no entry point.
6. **Permissões:** execução de desktop entry pela sessão; ações Niri IPC; escrita no settings do usuário.
7. **Persistência:** ordem de pinned em `dockPinned`; reorder escreve uma vez no drop. App launch não incrementa frecency do launcher.
8. **Erros/limites:** match de window class pontua StartupWMClass, desktop ID, último segmento, Exec binary e Name; unmatched windows aparecem por class. Janelas sem class são ignoradas. `coveredScreens` é string vazia porque o modelo público Niri usado não expõe fullscreen; portanto não há supressão real por fullscreen no serviço atual. Pinned entry sem desktop file mantém slot para unpin.
9. **Teste:** múltiplas janelas por app, classe ambígua (browser/webapp), StartupWMClass, app sem class, pinned app missing, hide running, every monitor, drag/drop limites, click launch/focus/cycle, closeAll e autohide.
10. **Rust:** model de dock separado da UI; indexação de classes pré-computada, agrupamento estável, pinned order persistente, actions tipadas e geometria por output. Fullscreen visibility só deve ser implementada se houver fonte Niri real, não heurística inventada.

**Limite visual pendente:** `Dock.qml`, `DockItem.qml`, `DockMenu.qml` e comportamento layer-shell/magnification/autohide ainda precisam de auditoria própria.

## DesktopService e DeckService — AUDITED (modelo de domínio)

**Fontes:** `services/DesktopService.qml` (1512 linhas), `services/DeckService.qml` (228 linhas).

### DesktopService

1. **Responsabilidade:** modelo persistido de widgets no wallpaper, grid por monitor, famílias/faces/temas, colisão e reposicionamento, decks de notas, spectrum, fotos e estado de edição.
2. **API/consumidores:** families/themes/faces, familyOf/themeOf/familiesFor/offers/drawnFamily/conform; boards/gridFor/wholeGrid/edgeAt; widgets/squares/decks/spectra, keys por screen; CRUD add/update/remove/removeMatching; notes/deck placement; spectrum settings/edge/grid; geometry, opacity, ink, picture picker; editing/typing/dragging/selected/landing/inHand/menu/surfaces/gallery state.
3. **Estado inicial:** rows vêm de SettingsService.desktopWidgets e são normalizadas; `key` ausente vira id; campos legados bare/style/ink são removidos. `boards` são publicados pelas surfaces conforme tamanho real.
4. **Eventos/frequência:** settings change atualiza lista quando saver não está ativo; writes debounce 120 ms; monitor list/effective primary refaz agrupamento por board; theme change conforma famílias. Keys por screen preservam arrays de delegates não afetados.
5. **Dependências:** SettingsService, MonitorService, Theme, DockService, DeckService, ModuleService, NotesService, NiriService, Quickshell screens e `imv` para abrir foto.
6. **Permissões:** persistência em settings; `imv` é iniciado na sessão; sem privilégios.
7. **Persistência:** cada row pode guardar key/id/col/row/family/theme/style/opacity/screen/picture/caption/note e campos de spectrum/deck. `screen` é a descrição do monitor (make/model/serial), não connector; primary não grava screen. Widget órfão de monitor desconectado cai para effective primary e pode voltar quando a descrição reaparece.
8. **Erros/limites:** grid calcula stride/columns/rows/origin por dimensão; colisões são resolvidas por ordem de maior reach à direita/baixo e busca nearest-free; posições clamped não são escritas automaticamente. Add falha com string vazia se módulo inválido/sem espaço. Uma note só pode existir em um lugar; deck vazio é removido; decks por edge left/right/bottom; spectrum no máximo um por edge e escolhe bottom/left/right. Foto aceita png/jpg/jpeg/webp/bmp. Editing é efêmero; abrir outra janela ou trocar workspace encerra modo para liberar focus grab.
9. **Teste:** múltiplos monitores/descrições, monitor removido/restaurado, grid sizes, colisões e empate nearest, board pequeno, family/theme sem face, row legacy, repeated module key, note move/merge/archive, takesNew, spectrum edge/grid, picture path invalid, editing focus grab e workspace/window change.
10. **Rust:** separar estado persistido de layout calculado; preservar dados de perfil, algoritmo determinístico de grid/collision, identities de monitor por descrição + connector atual, lifecycle de edição e invariantes de note/deck/spectrum. Não fazer UI no backend.

### DeckService

1. **Responsabilidade:** geometria e pointer state das tiras de notas nas bordas.
2. **API/consumidores:** decks resolvidos por screen; `awayOn`, `stripLength`, `runOf`, `startOf`, `alongAt`, `tabAt`, `tabBox`, `stripBox`, `gripBox`, `peekBox`, `edgeAt`, `indexAt`; estado revealed/peeked/dragging/sliding/held/receiving/receivingScreen/surfaces.
3. **Estado inicial:** derivados de DesktopService; pointer state vazio/false.
4. **Eventos:** dados de notes e DesktopService são reativos; estado de pointer vem de Deck UI; sem timer/polling.
5. **Dependências:** DesktopService, NotesService, SettingsService, NiriService, Theme.
6. **Permissões:** nenhuma adicional.
7. **Persistência:** geometria ao longo da borda é fração `along` guardada no row DesktopService; pointer state é transitório.
8. **Erros/limites:** `awayOn` só implementa deckOnEmpty; fullscreen fica a cargo do compositor/layer-shell, pois Niri IPC público não expõe fullscreen no modelo original.
9. **Teste:** count 0/1/muitos, start clamp, telas de tamanhos distintos, cantos (side precedence), indexAt, peek clamp, drag/reorder, edge changes e workspace ocupado.
10. **Rust:** cálculo puro testável, separado do adapter Niri e da superfície GTK; input state permanece no frontend, domínio persiste apenas `along`/edge/notes.

**Limite pendente:** Desktop.qml/Widget.qml/Inspector/Picker/Deck e as faces GTK/visualizações QML não foram auditadas; a matriz separa o backend de domínio das superfícies visuais.
## TimerService, NotesService e TasksService — AUDITED

**Fontes:** `services/TimerService.qml`, `services/NotesService.qml`, `services/TasksService.qml`.

### TimerService

1. **Responsabilidade:** um countdown global, iniciado por launcher/IPC e mostrado na ilha.
2. **API/consumidores:** running/paused/label/endsAt/duration/remaining/progress/seconds/tint/display; `parse`, `start`, `pause`, `resume`, `toggle`, `restart`, `cancel`, `complete`; sinal finished.
3. **Estado inicial:** sem timer; estado é apenas memória do processo.
4. **Eventos:** ticker 250 ms enquanto running e não paused; cálculo deriva de `endsAt - Date.now()`, evitando drift por carga.
5. **Dependências:** `notify-send -u critical` ao completar; Theme tokens.
6. **Permissões:** sessão do usuário e D-Bus notifications; sem privilégio.
7. **Persistência:** nenhuma; reload do shell perde o timer.
8. **Erros/limites:** parse aceita clock `m:ss`/`h:mm:ss`, unidades h/m/s, decimais e número sem unidade como minutos; `1h30` significa 1h30m, `5` significa 5m. Valores inválidos/<=0 retornam 0. complete dispara notificação crítica e sinal.
9. **Teste:** parse completo, valores fracionários, zero/negativo, pause/resume, restart, cancel, busy machine e completion.
10. **Rust:** manter relógio monotônico/absoluto sem drift, sem persistir até que o original seja explicitamente alterado; notificação crítica por freedesktop.

### NotesService

1. **Responsabilidade:** notas adesivas com title/body/tint, archive, editor atual e integração com widgets/decks.
2. **API/consumidores:** notes/live/archived/count/newest, `entry`, `noteFor`, `titleOf`, `isEmpty`, `add`, `create`, `update`, `setTint`, `archive`, `remove`, `firstLine`, `display`, `ageOf`, `open`, `leave`; sinais added.
3. **Estado inicial:** carrega `notes.json`; defaults para title/text/tint/created/edited/archived.
4. **Eventos:** lista live/archived ordenada por edited; write debounce 120 ms; text/title alteram edited, tint/archive não alteram edited.
5. **Dependências:** SettingsService stateDirectory, Theme, DesktopService placement, `NotesService.noteAdded` e Notes UI.
6. **Permissões:** leitura/escrita no state directory.
7. **Persistência:** `$XDG_STATE_HOME/quickshell/notes.json`; FileView sem watchChanges; note IDs timestamp base36 com sufixo em colisão.
8. **Erros/limites:** note vazia criada para escrever é removida ao sair sem conteúdo; archive remove nota do desktop/decks; nota referenciada em widget que foi removida/arquivada cai para newest. Tint set limitado a yellow/accent/green/blue/red; texto com `[ ]`/`[x]` ganha glyph só na apresentação.
9. **Teste:** load missing/corrupt, note blank leave, title/text/tint update, archive/unarchive, remove, ordering, deck assignment, checkbox display e restart.
10. **Rust:** persistência fiel e transações entre notes + desktop placement; não transformar Notes em Tasks nem adicionar state/due sem origem.

### TasksService

1. **Responsabilidade:** tarefas com texto, detalhes, estado, due day, rank, created e finished; alimenta board e calendário.
2. **API/consumidores:** states todo/doing/done, by state/day, overdue/upcoming/queue/summary; parseDue/split/dueLabel; add/create/update/setState/place/setDue/toggle/remove/open/leave.
3. **Estado inicial:** SystemClock com precisão de minuto; lista normalizada, estado inválido vira todo; rank default é índice original.
4. **Eventos:** clock atualiza todayKey; gravação debounce 120 ms; reordenação renumera rank da coluna; finish timestamp só enquanto state=done.
5. **Dependências:** SettingsService stateDirectory, Theme, SystemClock, UI board/calendar e IPC.
6. **Permissões:** state directory.
7. **Persistência:** `$XDG_STATE_HOME/quickshell/tasks.json`; FileView sem watchChanges; rows contêm key/text/body/state/due/rank/created/finished.
8. **Erros/limites:** parseDue aceita hoje/hoy, amanhã/mañana, next week, +N, weekdays e meses em inglês/espanhol, datas dd/mm, dd/mm/yyyy, `12 sep`, `sep 12`, ISO. Weekday escolhe próxima ocorrência incluindo hoje. `split` extrai último `@` apenas se data válida. **Possível bug legado:** ISO `yyyy-MM-dd` usa `dateOf()` que apenas constrói Date e não compara componentes; dia impossível pode normalizar para mês seguinte e ainda ser aceito. `build()` das outras formas valida dia/mês.
9. **Teste:** timezone/DST, virada de mês/ano, weekday hoje, +999, nomes espanhol/inglês, invalid date, due clear, done finished, move/reorder, blank task leave, overdue/upcoming e summary.
10. **Rust:** data type local-date sem timezone para due day, parser bilíngue compatível, rank estável e migração sem descartar detalhes.

## VmService e vms.py — AUDITED

**Fontes:** `services/VmService.qml` (246 linhas), `scripts/vms.py` (518 linhas).

1. **Responsabilidade:** gerir catálogo, criação, start/open/pause/resume/stop/kill, snapshots e configuração de VMs Quickemu/Quickget.
2. **API/consumidores:** machines/tools/host cores/memory, running/coresInUse/memoryInUse, error, watchers subscribe/release, refresh/find/uptime, actor/busy/act, catalog/loadCatalog, fetching/create/cancelCreate.
3. **Estado inicial:** `list` no startup; catálogo só na primeira abertura da aba New; fetch progress é stream JSON lines.
4. **Eventos/frequência:** refresh a cada 3 s somente enquanto alguma VM está running ou painel tem subscriber; após actor/fetcher exit faz refresh.
5. **Dependências:** quickemu, quickget, spicy, qemu-img, QEMU monitor Unix socket, `/proc`, XDG_DATA_HOME/velatura/machines e XDG_CACHE_HOME/velatura/vm-catalog.json (cache 7 dias).
6. **Permissões:** execução do usuário e escrita no diretório de dados; VM não requer sudo automático.
7. **Persistência:** cada VM é NAME.conf + NAME/; metadata Velatura adicionada ao conf; catálogo cacheado; imagens/disk e snapshots geridos por Quickemu.
8. **Erros/segurança:** nome validado `[A-Za-z0-9._-]+`; subprocessos usam argv; quickemu timeout 120 s; QEMU monitor timeout 1.5 s. PID é validado por `/proc/<pid>/cmdline` contendo qemu + nome. create captura stdout/progresso, cancel envia SIGTERM ao process group e limpa novos arquivos. snapshots exigem VM desligada e verificam qemu-img depois. delete remove diretório e conf. Pause/resume/stop confiam no socket e não verificam retorno do comando monitor com a mesma robustez do snapshot. `ready` considera quickemu+quickget; spicy/qemu-img são capacidades opcionais.
9. **Teste:** tools ausentes, cache fresco/stale, quickget failure/cancel, pid reused, monitor socket missing, paused, stop, kill, snapshot create/apply/delete, VM disk not booted, invalid name, concurrent actor/fetch.
10. **Rust:** provider Quickemu separado de UI, subprocess lifecycle e cancellation robustos, PID+socket validation, progress stream, catálogo cache e todas as operações; não tratar `asked` como conclusão.

## StatsService e stats.py — AUDITED

**Fontes:** `services/StatsService.qml` (165 linhas), `scripts/stats.py` (229 linhas).

1. **Responsabilidade:** CPU total/per-core, load, memória/swap, top 4 disks, network rates, temperatura, uptime e histórico para gráficos.
2. **API/consumidores:** cpu/cores/cpuModel/load, memoryUsed/Total, swapUsed/Total, disks, networkDown/Up, temperature, uptime, histories, ready, bytes/rate/duration.
3. **Estado inicial:** arrays vazios; `ready=false`; processo sampler inicia automaticamente no serviço.
4. **Eventos/frequência:** processo residente `stats.py watch 3`; 100 amostras (5 min) por série; CPU/network amostrados a cada tick; disks e sensors a cada 10 ticks (30 s); script reinicia após crash com 5 s.
5. **Dependências:** `/proc/stat`, `/proc/net/dev`, `/proc/meminfo`, `/proc/loadavg`, `/proc/uptime`, `/proc/cpuinfo`, `/sys/class/hwmon`, `df`.
6. **Permissões:** leitura de procfs/sysfs; sem privilégios.
7. **Persistência:** nenhuma; histórico vive em memória do shell e é alimentado continuamente.
8. **Erros/limites:** exclui loopback; memory used = MemTotal - MemAvailable; ignora tmpfs/devtmpfs/squashfs/overlay/efivarfs/ramfs; temperatura é o sensor hwmon plausível mais quente, sem identificar necessariamente só CPU; não inclui GPU. `snapshot` usa janela de 250 ms. Script captura exceções e imprime `{}` para não derrubar shell.
9. **Teste:** cores hotplug, /proc incompleto, memória/swap zero, disk mounts, sensores ausentes, interface down, counters reset, script crash e history length.
10. **Rust:** um watcher compartilhado; preservar taxas por delta de contadores, intervalos lentos separados e histórico de 5 min. Não afirmar menor CPU sem benchmark.

## SystemService e system.py — AUDITED

**Fontes:** `services/SystemService.qml` (98 linhas), `scripts/system.py` (105 linhas).

1. **Responsabilidade:** estado/toggle de Wi-Fi, Bluetooth e power profile no card de sistema.
2. **API/consumidores:** wifi/bluetooth/powerProfile/ready/performanceMode, subscribe/release/refresh/toggle.
3. **Estado inicial:** consulta status no startup; ready somente após JSON parseável; campo null significa ferramenta ausente e mantém último valor.
4. **Eventos/frequência:** polling 4 s somente enquanto watchers > 0; subscribe faz refresh imediato; toggle dispara ação e lê status novamente.
5. **Dependências:** `nmcli radio wifi`, `bluetoothctl show/power`, `powerprofilesctl get/set`; timeout 5 s por comando.
6. **Permissões:** D-Bus/NetworkManager, BlueZ e power-profiles-daemon da sessão.
7. **Persistência:** settings pertencem aos daemons; serviço não grava.
8. **Erros/duplicação:** ferramenta ausente retorna null; erro de comando também vira null; `toggle` não propaga bool de sucesso para UI, apenas estado posterior. Power profile alterna somente performance ↔ balanced, mesmo se perfil atual for power-saver. Wi-Fi/Bluetooth são duplicados de NetworkService/BluetoothService por outra API e polling.
9. **Teste:** cada daemon ausente, status null, watcher 0/1, toggle failure, perfil power-saver, daemon desconectado.
10. **Rust:** consolidar radios com os providers de rede/BlueZ e expor power profile separado; watcher só enquanto consumidores pedem status, mantendo semântica do painel.

## SessionService, IdleService, SunsetService, PolkitService e LockService — AUDITED

**Fontes:** serviços QML lidos integralmente nesta rodada; LockService 377 linhas.

### SessionService
- Ações: lock/suspend/logout/reboot/shutdown. Suspend sempre exige LockService.secure; se lock não confirmar em 5 s, cancela. Logout chama Niri `quit --skip-confirmation`; reboot/shutdown/suspend usam `systemctl` via argv.
- `dbus-monitor --system` acompanha `org.freedesktop.login1.Manager.PrepareForSleep`; `systemd-inhibit --what=sleep --mode=delay ... cat` segura sleep até lock seguro; em wake, recria o inhibitor. Sem senha/privilege escalation no serviço.

### IdleService
- Três IdleMonitor Wayland (`ext-idle-notify`): lock, screen off e suspend; minutos de Settings convertidos para segundos; timeout 0 desativa; `respectInhibitors=true` em todos.
- Screen DPMS usa Niri actions `power-off-monitors`/`power-on-monitors`; suspend encaminha SessionService, que bloqueia antes.

### SunsetService
- Probe `wlsunset`; `on` é o estado do Process; inicia `wlsunset -T <temp> -t <temp>` quando setting ligado e ferramenta existe. Temperatura muda -> reinicia após 250 ms; saída inesperada desliga setting e mostra OSD. Não agenda nascer/pôr do sol; controle é temperatura fixa.

### PolkitService
- Quickshell PolkitAgent registra um agente da sessão; LazyLoader ativo; signals requested/settled; password vai diretamente a `flow.submit` somente quando responseRequired; cancel só antes de completed. Não guarda password.
- Tenta registrar 3 vezes com intervalo 6 s; depois inicia `/usr/lib/polkit-kde-authentication-agent-1` como fallback. Testar ownership/registration real e não executar fallback duplicado.

### LockService
- Integração ext-session-lock, PAM `login` para password e PAM separado `velatura-face`/Howdy; estado locked é pedido, secure é confirmação do compositor; unlock aguarda morph antes de liberar surface.
- Antes de mapear lock, aguarda shellQuiet, 32 ms de settle; timeout de 900 ms garante lock mesmo se shell não ficar quiet. Captura JPEG 90% de cada output com grim em paralelo, `timeout 2`; caminho em `$XDG_RUNTIME_DIR/quickshell`, removido no unlock.
- Password é encaminhada diretamente a PAM, campo compartilhado e limpo em submit/rest/unlock. Face só inicia quando usuário acorda a tela; 3 misses, cooldown 1.5 s após look ou 5 s sem look; match segura 650 ms antes de liberar.
- **Risco de implementação legado:** screenshot/cleanup usa `sh -c` com string montada de paths e outputs; output name é validado, mas o path de runtime vem do ambiente. Rust deve executar grim com argv separado e remover arquivos via filesystem, sem shell.
- **Estado:** LockService e FaceService foram auditados estaticamente; helper/setup fonte está em INTEGRATION_AUDIT.md. O runtime de face não está disponível (Howdy/PAM/policy ausentes); lock/* UI continua NOT AUDITED e nenhum serviço está VERIFIED.

## FaceService, OsdService e LyricsService — AUDITED

**Fontes:** `services/FaceService.qml`, `services/OsdService.qml`, `services/LyricsService.qml`, `scripts/lyrics.py`.

### FaceService (adapter QML + helper/setup fonte auditados; runtime indisponível nesta máquina)

1. **Responsabilidade:** descobrir compatibilidade Howdy/IR camera, listar/enrolar/remover faces e testar PAM face.
2. **API/consumidores:** camera/installed/wired/known/ready; faces/listed/heard; installer; busy/outcome/complaint; trial; refresh/list/add/remove/tryFace.
3. **Estado inicial:** probe shell verifica dispositivo `/dev/video*` com formato GREY via v4l2-ctl, `howdy`, helper `/usr/lib/velatura/face`, polkit policy, PAM service e `pam_howdy.so`; só lista faces se ready.
4. **Eventos:** probe uma vez/refresh; `pkexec ... list/add/remove`; stdout CSV parseado; stderr guardado até stream finish; ao terminar ação atualiza busy/outcome e lista de novo.
5. **Dependências:** v4l2-ctl, howdy, pkexec, helper privilegiado `/usr/lib/velatura/face`, policy `/usr/share/polkit-1/actions/org.velatura.face.policy`, `/etc/pam.d/velatura-face`, `/usr/lib/security/pam_howdy.so`, kitty e `PackagesService.holdScript` para install.
6. **Permissões:** list helper por pkexec; add/remove solicitam autenticação de usuário; helper faz operações privilegiadas; tentativa de face usa PAM sem root.
7. **Persistência:** modelos/face data ficam sob Howdy/root; serviço não guarda senha nem face list.
8. **Erros/limites:** pkexec exit 126/127 é cancel/unauthorized e não gera outcome; stderr classifica no face/no face, several faces, too dark ou failed. `tryFace` exige ready, faces não vazias e PAM inativo. O probe depende de GREY (IR), não apenas qualquer câmera.
9. **Teste:** sem IR camera, webcam RGB sem GREY, howdy/helper/policy/PAM ausente, pkexec cancel, list CSV/quotes, add/remove, PAM fail/success e refresh pós-instalação.
10. **Rust:** serviço de capabilities e operação privilegiada por helper/polkit bem delimitado; nunca guardar senha; separar probe hardware, setup e PAM trial.

**Atualização 15/11/2026:** helper privilegiado, policies e setup scripts foram localizados e lidos no checkout `/home/px/impasto-niri`; detalhes em `INTEGRATION_AUDIT.md`. Eles não estão instalados nesta máquina (`howdy`, helpers, PAM e policies ausentes). O adapter QML e o contrato fonte estão AUDITED; operação real continua indisponível e não foi testada.

### OsdService

1. **Responsabilidade:** centralizar mudanças efêmeras de sistema em um único evento OSD da ilha.
2. **API/consumidores:** sinal `requested(icon,label,progress)`, `armed`, `audioSuppressed`, `brightnessDisplay`, `suppressAudio()`.
3. **Estado inicial:** não emite durante os primeiros 600 ms para evitar OSD inicial causado por bindings.
4. **Eventos/frequência:** volume/mute debounce 16 ms; brightness recebe sinal `adjusted(display)` e debounce 16 ms; charging change de bateria imediato; áudio suprimido 2 s após Bluetooth toggle. Não emite volume/brightness quando detalhe correspondente está aberto.
5. **Dependências:** AudioService, BrightnessService, BatteryService, ModuleService.
6. **Permissões:** nenhuma; só agrega eventos.
7. **Persistência:** nenhuma.
8. **Erros/limites:** progress -1 indica mensagem sem barra; battery OSD é só mudança charging state, não todo percentual.
9. **Teste:** mudanças durante startup, tecla de volume em rajada, detalhe aberto, Bluetooth troca sink, brightness em monitor não focado e charging state.
10. **Rust:** event aggregator compartilhado; preservar debounce/suppression e não duplicar OSD por integração.

### LyricsService e lyrics.py

1. **Responsabilidade:** buscar letras LRCLIB por track e sincronizar linha atual com MPRIS.
2. **API/consumidores:** watchers subscribe/release; wanted/track/readFor/lines/synced/instrumental/loading/available/current/currentText/nextText/status; anchorPosition/anchorTime; fetch/take/place.
3. **Estado inicial:** sem track/lyrics; só consulta se watchers > 0, SettingsService.lyrics ligado, MediaService disponível e título não vazio.
4. **Eventos/frequência:** MediaService position updates ancoram relógio; pause/playing reancora; binary search encontra última linha iniciada com lead 250 ms; Timer agenda exatamente próxima linha, sem polling 8 Hz. Cada subscriber LyricsService também subscribe em MediaService.
5. **Dependências:** `scripts/lyrics.py`, HTTPS `https://lrclib.net/api`, cache local XDG state.
6. **Permissões:** rede de saída HTTPS e escrita em `$XDG_STATE_HOME/quickshell/lyrics`.
7. **Persistência:** cache por SHA-1 de artist/title/album/length; respostas disponíveis cacheadas; misses por 7 dias; falhas de rede não são cacheadas.
8. **Erros/limites:** requests timeout 8 s; exact get + search por campos + free query + bare title; tolerância de duração 8 s (bare match 2 s), prioriza synced e closest length; 5xx/429/network Busy -> retry uma vez após 2 s; LyricsService retry exponencial 10 s até 120 s enquanto track ainda wanted. Títulos limpam sufixos Official/Lyric/Audio/Video etc. Timed LRC e plain lyrics/instrumental suportados.
9. **Teste:** sem rede/HTTP 404/429/5xx, rate limit, missing cache, miss TTL, timed/plain/instrumental, múltiplas timestamps por linha, pause/resume/seek, track switch durante request e watchers 0.
10. **Rust:** HTTP client com timeout/cache/TTL e retries, parser LRC, binary search e timer agendado; nunca buscar letras sem consumidor ativo/setting habilitado.

## ModuleService — AUDITED

**Fonte:** `services/ModuleService.qml` (618 linhas).

1. **Responsabilidade:** catálogo de módulos e botões da barra/ilha/desktop; disponibilidade real, glyph/value/tint, watchers e estado do painel aberto.
2. **API/consumidores:** catalogue/entry, buttonIds/buttons/isButton/placeable, watch, panelToggled/togglePanel/shownPanel, settingsRequested, shapeOf/figureOf/glyphOf/valueOf/figureLimit/tintOf, runners/runs/shows, activities/overflow, glanceRows/glanceFace/glanceStrip, openId/openHost/close/openSize, activationRequested/activate, panelRequested/requestPanel, has.
3. **Estado inicial:** catálogo fixo com módulos media/timer/claude/codex/battery/volume/brightness/network/bluetooth/notifications/weather/markets/screentime/github/stats/updates/pet/games/calendar/notes/tasks/photo/spectrum/lyrics/clock; botões têm precedência sobre módulos na barra.
4. **Eventos/frequência:** `watch(id,on)` incrementa/decrementa subscriptions de Claude/Codex/Weather/Markets/ScreenTime/Updates; `openGone` fecha detalhe se módulo deixa de estar disponível; panel/activation são sinais para bar responder.
5. **Dependências:** SettingsService, Theme, ControlsService, CaptureService, Network/Bluetooth/Audio/Brightness/Battery/Weather/Markets/Notification/Media/Timer/Stats/Claude/Codex/ScreenTime/Updates/Pet/Games/Notes/Tasks/Github/Lyrics/Privacy services.
6. **Permissões:** herdadas dos serviços que cada módulo instancia; catálogo não pede privilégios.
7. **Persistência:** layout/ordem/visibilidade e preferências ficam em SettingsService; open panel e subscriptions são runtime.
8. **Erros/limites:** `has()` é mais que uma checagem pura: acessar lazy singleton pode construí-lo e disparar primeira consulta (Claude/Weather/Markets/Pet/Games etc.). `network` permanece disponível mesmo desconectado; `stats`/`screentime` são sempre apresentados como capazes; notifications/clock/calendar/timer/workspaces/tray também. Runners condicionais são somente timer/media. Glance prioriza media > timer > privacy, depois weather se já está sendo pedido, senão clock. `activities` limita a duas; workspace pode ocupar slot mas não conta overflow.
9. **Teste:** availability de cada integração, módulo opcional ausente, watcher count balanceado, openGone, dynamic openSize, button/module ID collision, ring eligibility, values/glyphs, glance priorities, weather request gating.
10. **Rust:** catálogo tipado e capability resolver sem side effects ocultos; criação lazy explicitamente separada de `is_available`; watchers com ownership/RAII e subscriptions compartilhadas; open panel state entregue à UI por eventos.

**Discrepância de lifecycle confirmada:** ModuleService comenta que StatsService e ScreenTimeService iniciam no boot. A leitura/grep de `shell.qml` mostra apenas seis singletons tocados explicitamente (Theme, Monitor, Polkit, Sunset, Idle, Session), sem Stats/ScreenTime; `barDefaults` também não inclui stats/screentime. Portanto, o comentário não prova boot: esses serviços parecem lazy até um módulo/widget/configuração que os consuma. Confirmar o layout real do usuário e o grafo completo de consumers antes de fixar a política. A disponibilidade `true` não prova que o processo já foi iniciado.

## ThemeService, WallpaperService, theme_manager.py e greeting.py — AUDITED

**Fontes:** `services/ThemeService.qml` (273 linhas), `services/WallpaperService.qml` (269), `scripts/theme_manager.py` (4317, leitura integral em blocos) e `scripts/greeting.py` (888, leitura integral).

### WallpaperService / awww / mpvpaper

1. **Responsabilidade:** listar imagens e vídeos, criar posters, aplicar transições, restaurar wallpaper em outputs vazios, lidar com hotplug e sincronizar Matugen.
2. **API/consumidores:** wallpapers/animated/currentWallpaper/currentMotion/chosen/scanning/awwwOwned; scan/apply/transitionType; sinal applied(path).
3. **Estado inicial:** inicia `awww-managed`, list-wallpapers, list-animated, get-state e restore. `awww-daemon` é iniciado como child do Quickshell somente se não houver dono systemd/Niri/existing process e o binário existir.
4. **Eventos/frequência:** wallpaperDir debounce 500 ms; restore retry até 8 vezes, intervalo 1500 ms quando exit 3; screensChanged debounce 700 ms; scan lista imagens e vídeos; Matugen toggle dispara sync.
5. **Dependências:** `awww`/`awww-daemon`, `mpvpaper`, `ffmpeg`, `theme_manager.py`, `matugen`, XDG paths e Quickshell screens.
6. **Permissões:** sessão do usuário; leitura da config Niri e systemd units; escrita em state/cache; sem sudo.
7. **Persistência:** `state.json` guarda activeTheme/currentWallpaper/currentMotion; symlink `current-wallpaper`; posters cacheados por path+mtime+size; GIF/video posters usados em lock/overview.
8. **Erros/limites:** awww guard verifica system+user units (`cat/is-enabled/is-active`), `spawn-at-startup` em arquivos KDL recursivos e `pgrep`; não altera config. Restore retorna 3 se daemon não responde, e aplica só em outputs vazios. `restore_wallpaper` ignora returncode do `awww img` final. `set_wallpaper` pode continuar a gravar estado mesmo sem binário awww (a chamada de awww é condicional). `stop_motion()` usa `pkill -f impasto-motion` e depois `pkill -x mpvpaper`, podendo encerrar qualquer mpvpaper do usuário, não só o filho Velatura. Falha de awww ocorre depois de parar o vídeo anterior.
9. **Teste:** daemon externo vs owned, unit system/user, Niri spawn-at-startup, daemon ausente/lento, restore blank outputs, hotplug físico/Virtual-1, arquivo movido, poster ausente, vídeo sem ffmpeg, transição inválida, awww failure e outro mpvpaper ativo.
10. **Rust:** manager de wallpaper com ownership explícito de processos próprios, não usar pkill global; outputs Niri read-only; state versionado; cache poster; restore por output vazio; nunca assumir awww exclusivo.

### ThemeService

1. **Responsabilidade:** escolher e persistir paleta ativa, aplicar adaptive colors do wallpaper, atualizar Matugen background, font/ground do terminal e cenas de greeting.
2. **API/consumidores:** activeId/dynamicColors/availableThemes/adaptiveSwatches; setTheme/reloadAdaptiveColors/pushTerminal/pushTerminalFont/pushTerminalGround/pushGreeting; greeting choices e greetingStamp.
3. **Estado inicial:** `get-state` restaura activeTheme; `extract-colors` gera adaptive palette; SettingsService.arrived dispara push de font, ground e greeting; matugen palette FileView observa Dank Calendar theme JSON.
4. **Eventos:** terminal font/ground têm coalescing (fontAgain/groundAgain); greeting tem pending; terminal palette Process não tem fila equivalente. Matugen file watcher atualiza Theme.matugenBackground.
5. **Dependências:** theme_manager.py, greeting.py, SettingsService, Palettes, Theme, WallpaperService e Matugen.
6. **Permissões:** escrita state/config de usuário e processos de tema externos.
7. **Persistência:** theme_manager state.json, dynamic-colors.json, kitty fragments, greeting paint/choice/stamp e Matugen files.
8. **Erros/concorrência:** JSON inválido é logado e ignorado; themeId desconhecido loga warning mas activeId é alterado antes de validar. `terminalProcess` não coalesce múltiplas palette pushes, possível perda/ordem incorreta em troca rápida de tema. Font/ground/greeting tratam requests pendentes.
9. **Teste:** startup antes/depois de Settings arrived, adaptive sem wallpaper, theme inválido, palette push em rajada, file watcher Matugen e persistência.
10. **Rust:** theme state separado do renderer; pipeline de palette async e cancelável, latest-wins; cada provider opcional com status próprio e sem travar GTK.

### theme_manager.py — comandos e efeitos

1. **Comandos CLI identificados:** `awww-managed`, `sync-matugen`, `list-wallpapers`, `list-animated`, `get-current`, `get-state`, `extract-colors [path]`, `set-wallpaper path [transition]`, `restore`, `set-theme id`, `push-terminal-palette JSON`, `push-terminal-font family`, `push-terminal-ground style`, `apply-vscodium [JSON]`, `apply-nautilus`, `apply-vesktop`. Três ações internas (`awww-managed`, `sync-matugen`, `push-terminal-ground`) não aparecem todas no usage principal.
2. **Wallpaper discovery:** IMPASTO_WALLPAPERS env > SettingsService wallpaperDir > `$XDG_DATA_HOME/wallpapers`; imagens jpg/jpeg/png/webp; vídeos mp4/webm/mkv/mov/gif em subpasta animated, criada no list. Poster ffmpeg primeira frame, cache por stat.
3. **Wallpaper runtime:** awww transições; mpvpaper `ALL`, auto-pause/auto-mode full, loop/hwdec/no-audio; delay de 1 s após transição; state JSON + symlink + dynamic colors + Matugen. Restore lê `awww query`, repinta só outputs vazios.
4. **Palette engine:** ImageMagick resize 120×120, 24 cores únicas; escolhe accent saturado midtone ou fallback #89B4FA; constrói background/surface/hover/border/text/accent/status colors; WCAG contrast floors 4.5:1 e 7:1 para ANSI bright.
5. **Programas escritos no push:** kitty palette/ground/font (kitten remote sockets), btop theme, cava theme + SIGUSR2, yazi flavor + tmTheme, Neovim colorscheme + remote apply, imv config, GTK3/GTK4 CSS, Papirus-derived folder icon theme, qt6ct palette/config, kdeglobals merge, Vesktop CSS, Spicetify color.ini/Spotify refresh, VSCodium theme/settings, Zen userChrome/user.js. Cada app é opcional via `shutil.which`; config symlinked pode ser recusada.
6. **Ações setup-only:** apply-vscodium mescla settings gerenciadas; apply-nautilus usa gsettings e adiciona XDG folders em GTK bookmarks; apply-vesktop adiciona tema em enabledThemes. Não são chamadas em todo push.
7. **Matugen:** se habilitado, registra template Dank Calendar sem substituir template existente; grava `matugen.stamp`/PID, termina run anterior somente se `/proc/PID/comm` for matugen, inicia `matugen image ... --mode dark --prefer saturation --quiet` detached; altera Dank Calendar colorSource quando apropriado.
8. **Persistência:** `$XDG_STATE_HOME/quickshell/state.json`, dynamic-colors.json, kitty fragments, current-wallpaper symlink, Matugen stamp/PID; caches em XDG_CACHE_HOME; writes externos nos config dirs de cada app.
9. **Erros/limites:** a maioria dos writes trata OSError e apps ausentes como skip; alguns kitten/remote calls não têm timeout; `write_icon_theme` cria diretório `.building`, symlinks ~2000 icons, stamp e swap atômico, recusando diretório sem stamp próprio; `write_kde_colors` preserva outras chaves mas descarta comentários; VSCodium JSONC é editado por scanner próprio. `set-wallpaper` pode retornar sucesso sem awww; restore não checa retorno final. `pkill -x mpvpaper` é amplo.
10. **Rust:** decompor esse script monolítico em providers por aplicação e wallpaper engine; preservar cada arquivo/setting gerado e guardrails; jobs independentes e latest-wins; não escrever config externa silenciosamente sem opt-in do recurso correspondente.

### greeting.py

- Desenha 4 animações pixel-art 54×60, escala 5x via ImageMagick PAM→GIF para manter pixels nítidos: lava (90 frames), critters (48), koi (80), invaders (80); cada cena usa palette ativa.
- `theme_manager.py push-terminal-palette` chama `greeting.push()`; grava paint e lança processo detached; draw gera cenas em staging, descarta render antigo se palette mudou, swap atômico e atualiza stamp; `choose` troca symlink `greeting.gif`; `seed` usa sample palette.
- `ThemeService` escolhe lava/critters/koi/invaders/random e observa greeting.stamp; greeting não é um serviço de wallpaper e não deve ser omitido da paridade do terminal/fastfetch.
- Dependência ImageMagick; timeout 60 s por cena; falhas em stderr/log. Nenhuma imagem foi renderizada nesta auditoria.

### Questões de segurança/robustez abertas

- Não matar qualquer `mpvpaper` pelo nome; guardar PID/process identity do filho criado e só encerrar o próprio.
- Confirmar política de posse do awww sem tocar config Niri; guard atual é read-only e deve continuar assim.
- Corrigir verificação de returncode no restore e não declarar wallpaper aplicado se awww não executou.
- Separar geração de tema em providers; adicionar timeout a chamadas remotas que hoje podem bloquear; coalescer palette pushes.
- Testar escrita em configs com symlinks, JSONC inválido, KDE parser e VSCodium user settings preservando chaves/comentários conforme contrato.

## Serviços QML restantes — AUDITED (15/11/2026)

Leitura integral direta dos nove serviços que faltavam no primeiro mapa. Com eles, os 61 arquivos QML singleton ativos do SERVICE_CATALOG foram inspecionados. Isso fecha a auditoria estática do backend de serviços; não fecha a auditoria visual dos painéis/widgets nem runtime parity.

### CompositorService
- Só consulta niri --version no startup; guarda version/lastError/loaded. configPath aponta para XDG_CONFIG_HOME/niri/config.kdl, mas o serviço não lê nem modifica o arquivo.
- Rust: version provider read-only, timeout e ServiceStatus. Não duplicar a versão já exposta pelo handshake Niri sem razão.

### ShortcutService
- Catálogo estático de 32 ações de shell (launcher, controls, overview, settings, appearance, palette, stats, session, lock, pet, games, notes, board, keys, zen, widgets, game mode, packages, machines, servers, screentime, camera, clipboard, emoji, picker, capture, capture region/window/screen/edit/text e record).
- SettingsService.keys é lido, mas a implementação não resolve binding: compositorBindingsEditable=false; current() retorna vazio; fixed() sempre true; clash() vazio; rebind() apenas avisa para usar NiriMod opcional; sheet/sheetFind vazios. Não edita config.kdl.
- Importante para paridade: o original já não implementa edição de keybindings; não inventar um editor nativo nem gravar Niri KDL sem decisão. Preservar o catálogo e a indicação de não editável.

### ProfileService
- Perfis nomeados; arquivo ativo de SettingsService é a fonte do perfil ativo, lista e snapshots dos outros em state/profiles.json. Formato export/import v1; inclui settings profileKeys, wallpaper e palette, exclui machineKeys e dados de usuário (notes/tasks/clipboard/pet).
- create/duplicate/rename/remove (não remove active), stash/switchTo, cleanName 32 chars e uniqueName case-insensitive; switch chama SettingsService.adopt, aplica paleta antes do wallpaper e OSD.
- Export JSON inclui marca velatura, name, wallpaper com HOME abreviado para ~, palette e settings. Import rejeita JSON inválido, arquivo de versão futura e shape inválido; SettingsService.complete preenche defaults, chaves desconhecidas/shape errado são ignorados e contados no OSD. Wallpaper resolve primeiro caminho exato do pacote, depois mesmo filename; caminho só é aplicado se test -f.
- Perfis shipped em XDG_DATA_HOME/velatura/profiles são lidos via FolderListModel; apenas os ainda não oferecidos são acrescentados. Em fresh install o primeiro substitui placeholder Default. profiles.json usa FileView watchChanges e saver interval 0.
- Rust: versionar schema/migrações, preservar perfil ativo e isolamento de machine/user data, import/export via file chooser/portal e atomic writes. Validar wallpaper path e evitar sobrescrever perfil se SettingsService.adopt falhar. Não migrar arquivos automaticamente sem backup.

### PickerService
- Probe único de hyprpicker via shell estático command -v; picking aguarda Theme.durationIslandGone, segundo pedido cancela. Executa hyprpicker -f hex, aceita apenas #RRGGBB, envia o valor como argumento para wl-copy (sem newline) e OSD. Escape/vazio não copia.
- Rust: PATH capability probe sem shell, subprocess argv + timeout/cancel e clipboard provider. Não alterar o formato/validação de cor.

### ControlsService
- Modelo do Control Center: 15 doors e 18 toggle tiles, top row left/right customizável, sessão/shortcuts; default buttons stats/settings/pet/games/notes/board; default toggles wifi/bluetooth/power/focus/microphone/airplane.
- Toggles conectam Network/Bluetooth/System/Notifications/Audio/Sunset/Settings/Capture/Picker/Recorder/Clipboard; airplane deriva Wi-Fi+Bluetooth, suppressAudio antes de rádio Bluetooth para não gerar OSD de volume. Captura/picker/record/clear clipboard são ações one-shot e fecham painel antes.
- Grid de 6×8 default, 16 block types com footprints permitidos, defaults (toggles, volume, brightness, appearance, media, weather, calendar, notifications); top row height 28 + gap 14; tamanho calculado antes do painel existir.
- Layout persiste em SettingsService.centreBlocks/centreColumns/centreRows, debounce 120 ms; blocks aceitam duplicate block id por key estável. Colisão usa nearest-free por distância quadrática; resize mantém blocos centrados horizontalmente, realoca os cortados e descarta só quando não cabe. Resize/size cycling respeita shapes ofertados; restore volta settings a null/default.
- Toggles podem ter listas independentes por bloco; top row session actions ficam à esquerda e doors à direita; drag reorder/flip/toggle persiste em centreTop. Editing/dragging/selected/landing são transitórios.
- Rust: domínio separado da composição UI; persistir mesma geometria/ordem/duplicate keys e algoritmo nearest-free/resize. Não reduzir os tipos de bloco ou toggles.

### CalendarService
- Dank Calendar/dcal é dono dos dados. ensure faz ping; se daemon não responde, inicia dcal run -d --hidden e repinga após 650 ms. Não mata daemon. Lê calendars.list e events.list para mês selecionado (ISO range, limit 250); deduplica por start|end|summary. writableCalendars exclui readOnly/holdsTasks/hidden.
- eventsOn usa intervalo start <= dia < end, all-day usa string ISO e timed converte para data local. openCalendar chama dcal ipc ui.show view=month; newEvent chama ui.newEvent com start; openEvent usa uid e start. Falha no events list marca unavailable.
- Dependência opcional externa dcal/Dank Calendar; estado indisponível deve ser gracioso. Rust não deve criar banco de calendário próprio nem duplicar eventos.

### CavaService
- Cava subprocesso só com watchers >0 ou linger 2 s após último release, exceto gameMode; saída zera bands/peaks ao parar. 64 bandas normalizadas 0..1; 8 barras espelham grupos baixos ao centro; peaks caem 0.035 por linha; parser de 64 campos separados por ';'. OSD/player ring usa level = média dos 8 elevado a 0.55.
- Comando cava -p scripts/cava.conf; cava.conf é config, não Python. Sem polling; stream stdout. Game mode interrompe imediatamente.
- Rust: watcher compartilhado, parser robusto, process ownership, peak decay e linger idênticos; Cava UI/spectrum face é camada visual pendente.

### GamesService
- Catálogo de 11 jogos: whack, snake, flood, mini2048, lights, hextris, bots, target, space, tetris, solitaire; cada um tem icon/tint/board size/unit/lower-is-better.
- Estado runtime playing/lastOpened; record best/plays/playedAt por jogo em XDG state games.json; cria vazio no primeiro run; grava debounce 120 ms. record ignora rounds vazias (score <=0 nos jogos higher-is-better), atualiza best e plays, emite newBest; OSD só para novo recorde depois do primeiro score.
- Sem subprocessos externos; os jogos em bar/island/games/* são UI/game logic e ainda precisam auditoria própria. Não marcar arcade VERIFIED só pelo registry.

### PetService
- Família de cinco espécies dot/sprout/ember/sol/drift, quatro estilos (creature/plush/paper/pixel), apenas um pet ativo na barra; demais ficam na shelf. Pet ativo ganha XP, fome/solidão e companhia; pets na shelf dormem e pausam mood, mas cooldown continua. Sem morte, penalidade ou reset; egg por milestone de totalLevel [6,16,30,50], uma espécie faltante aleatória.
- Feed cooldown 90 min e +25 XP; play cooldown 5 min e +12 XP + bônus de 3/h até +18; threshold level 40+20*level; companhia +1 XP a cada 5 min quando pet está no bar, hatched, não asleep e LockService unlocked. Mood é derivado de fed/played elapsed menos rest; fome >6h, solidão >8h, asleep ambos >16h, beaming se meal <3h e play <2h.
- rename máximo 24 chars; bringOut troca ativo, registra restedAt e acumula fedRest/playedRest; dados em state/pet.json, debounce 120 ms. Migra flat single-pet save para family; se JSON adapter não parsear family, tenta texto raw antes de criar egg.
- Rust: clock/event-driven sem tick de UI; preservar fórmula, milestones, random species selection, rest/cooldowns e migração. Dados são pessoais, não incluir em ProfileService exports.

### Riscos e limites transversais
- Profile import/export e settings adopt precisam de round-trip, malformed JSON, versão futura, chaves removidas e wallpaper ausente em fixtures.
- Calendar depende de dcal externo; não substituir por um calendário próprio.
- ShortcutService é deliberadamente não editável; a matriz deve preservar o comportamento original e marcar NiriMod como opcional.
- ControlsService contém regras de domínio de layout; não reimplementar nearest-free ou airplane logic dentro dos widgets GTK.
- Games/Pet registry e persistência estão auditados, mas os jogos e as animações/arte de pets continuam como UI pendente.
