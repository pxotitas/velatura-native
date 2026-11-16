# Mapa de dependências e integrações de sistema

Este mapa separa dependências de framework das integrações de domínio. A presença no código não prova que uma ferramenta esteja instalada nem que o recurso esteja habilitado.

## Framework/runtime original

| Dependência | Áreas | Substituição/estratégia nativa |
|---|---|---|
| Quickshell / QtQuick / QtQml | lifecycle, janelas, bindings, timers, processos, singleton e layer surfaces | GTK4/GLib, Wayland protocols, layer-shell e Rust. |
| QtQuick scene graph / shaders QSB | Spectrum e efeitos QML | GTK snapshot/Cairo/OpenGL somente onde necessário; preservar fallback. |
| Quickshell.Io | Process, FileView, JsonAdapter e IPC | std::process, GIO, serde, escrita atômica e async GLib/Tokio justificado. |
| Quickshell.Services.Pipewire | Audio/privacy/capture | libpipewire e APIs de sessão. |
| Quickshell.Services.UPower | Battery | D-Bus UPower via zbus. |
| Quickshell.Bluetooth | Bluetooth | D-Bus BlueZ via zbus. |
| Quickshell.Services.Mpris | Media | MPRIS D-Bus. |
| Quickshell.Services.Notifications | Notifications | org.freedesktop.Notifications D-Bus. |
| Quickshell.Services.SystemTray | Tray | StatusNotifierItem / DBusMenu. |
| Quickshell.Services.Polkit | Auth | PolicyKit agent API; PAM apenas no fluxo específico de lock. |
| Quickshell.Services.Pam | Lock/face | PAM; revisão de segurança separada. |
| Quickshell.Wayland | lock/idle/layer windows | Wayland protocols e layer-shell; validar no Niri. |

## Integrações por domínio

| Domínio | Fonte atual | Comportamento a preservar |
|---|---|---|
| Niri | niri msg --json event-stream, niri msg action, niri msg outputs | workspaces, janelas, foco, urgência, layouts, outputs, actions e reconnect. |
| Monitores | Quickshell screens + scripts/monitors.py + Niri | hotplug, geometria, primary/internal, nome e descrição. |
| Áudio | PipeWire | default sink/source, nodes, streams, volume, mute, seleção e captura. |
| Rede | NetworkManager | rádio, scan, connect/disconnect/forget, wired, strength e state. |
| Bluetooth | BlueZ | adapter, power, discovery, devices, pairing e conexão. |
| Energia | UPower/systemd/logind | bateria, autonomia, suspend, lid policy e inhibitors. |
| Brilho | sysfs/DDC/CI | interno/externo, detecção, debounce, leitura e escrita. |
| Notificações | freedesktop notifications | actions, urgency, timeout, history e do-not-disturb. |
| Tray | StatusNotifierItem/DBusMenu | watcher, lifecycle, icons, menu tree e activation. |
| Mídia | MPRIS | players, metadata, art, position, seek e capabilities. |
| Clipboard | Wayland + wl-paste/wl-copy | MIME, imagens, history, search, copy e wipe-on-lock. |
| Captura/gravação | Wayland screencopy/portal + tools | seleção, output, formato, anotação, áudio, cancelamento e permissões. |
| Tema/wallpaper | awww/matugen | scan, animated, transition, restore e paleta dinâmica. |
| Night light | wlsunset | enable, temperatura, lifecycle e availability. |
| Calendar | dcal | calendars, events, range, create/open event. |
| Pacotes | pacman/checkupdates/AUR | search, installed, upgrade, install/remove, confirmação e privilégio. |
| VMs | vms.py, Unix sockets e processos | catalog, list, create, start/pause/resume/stop/kill e recursos. |
| Métricas | procfs/sysfs/sensors/stats.py | CPU, RAM, swap, discos, rede, temperatura, uptime e histórico. |
| Serviços web | scripts Python | cache, refresh, rate-limit, offline, parsing e privacidade. |

## Segurança e permissões

- Criar matriz de cada operação que exige autenticação/privilégio (pkexec, polkit, PAM, pacman, helper de conta).
- Nunca concatenar entrada de usuário em shell.
- Daemon ausente deve gerar estado Unavailable com motivo; UI não pode fingir sucesso.
- Timeout, cancelamento, reconexão e stderr precisam de tratamento explícito.
- Nunca armazenar senhas, tokens ou clipboard sem política clara.
- Operações destrutivas exigem confirmação equivalente à original.

## Validação restante após auditoria estática

- Executar probes read-only de tools/status de capture, record e camera em ambiente controlado; nenhuma instalação ou mutação foi feita nesta rodada.
- Testar cada integração disponível e cada estado ausente/permissão negada; helpers de face/avatar e howdy estão ausentes no sistema atual.
- Validar endpoints, rate limits e formatos em fixtures offline; não depender da rede para testes determinísticos.
- Implementar e testar APIs nativas em Rust; a auditoria não equivale a runtime parity.
- Fechar política de processos opcionais e lifecycle com benchmark/observabilidade antes de declarar desempenho.


## Auditoria das integrações adicionais (15/11/2026)

A leitura integral dos 16 scripts da segunda rodada e dos adapters QML está em INTEGRATION_AUDIT.md. Somados aos 12 da primeira rodada, os 28 scripts ativos estão auditados. Dependências confirmadas:

| Serviço | Fontes externas | Lifecycle / condição |
|---|---|---|
| Account | AccountsService D-Bus, busctl legado, ImageMagick, pkexec avatar helper | leitura no startup; nome debounce 800 ms; helper privilegiado opcional |
| Face | v4l2-ctl GREY, howdy-next, pkexec helper, polkit, PAM | probe/refresh; enrol/remove interativos; requer IR |
| Camera | V4L2, v4l2loopback, OpenCV, MediaPipe, NumPy, Pillow, 3 modelos | server só quando habilitado; webcam só com reader |
| Claude | ~/.claude JSONL, OAuth credentials, Anthropic HTTPS | transcripts 120 s e limits 600 s somente com watchers; segredo nunca em argv |
| Codex | ~/.codex session JSONL | leitura local sob demanda/poll com watchers; sem rede |
| Emoji | unicode-emoji, CLDR annotations, iso-codes | lazy no modo emoji; recents XDG state |
| Fonts | fontconfig fc-list | lazy em settings; timeout 10 s |
| Machine | os-release, uname, passwd, pacman local DB, niri --version | consulta estática sob demanda |
| GitHub | github.com contributions HTML | 30 min com watchers; username opcional; último dado preservado |
| Markets | CoinGecko API, Frankfurter API | 5 min com watchers; debounce 800 ms; último dado preservado em falha parcial |
| Weather | wttr.in HTTPS, IP geolocation quando place vazio | 15 min com watchers; last-good; timeout 12 s |
| Packages | pacman, yay/paru, AUR RPC, kitty | leitura/search; install/remove/-Syu só em terminal interativo |
| Updates | checkupdates, pacman -Qu, pacman -Qm, AUR RPC, vercmp | 30 min com watchers; AUR offline não invalida repo |
| Version | git fetch, SSH BatchMode, checkout setup, kitty | check 30 min; update/restore files em terminal/ação detached |
| Privacy | PipeWire nodes, /dev/video*, inotifywait, /proc | construído quando activity visible; eventos de open/close |
| ScreenTime | Niri focus, LockService, IdleService, NotificationService, Steam manifests, XDG state | recorder de sessão; relatório/minuto só com subscribers |
| Servers | ss, /proc, signals | ss a cada 3 s só com painel aberto; stop revalida processo |
| Wallpaper setup | awww, mpvpaper, ffmpeg, Matugen | ownership explícito; nunca encerrar processos alheios |

A presença de comando ou pacote não prova disponibilidade. Nesta máquina, probes read-only confirmaram v4l2-ctl, pkexec, fc-list, ss, pacman, checkupdates, vercmp, yay, git, curl, kitty, magick e wl-copy; ausentes: howdy, inotifywait e paru. Helpers/policies/PAM face/avatar não estão instalados, embora a fonte exista no checkout /home/px/impasto-niri.

### Regras de portabilidade derivadas

- HTTP nativo com timeout, user-agent e parse tipado; não usar curl com OAuth token em argv.
- zbus para AccountsService, NetworkManager, BlueZ, UPower, PipeWire e serviços D-Bus em vez de parsing de CLI sempre que aplicável.
- Leitura de /proc deve limitar UID e dados estritamente necessários; não guardar títulos ou conteúdo privado.
- Para stop server, validar pid + starttime + executable + cwd e preferir pidfd; nunca matar grupo do próprio shell.
- V4L2 e MediaPipe precisam de capability probe, stream lazy, backpressure, métricas e benchmark antes de ativar automaticamente.
- Ações pacman/AUR permanecem visíveis e interativas; nenhum sudo/pkexec oculto.

## Serviços QML complementares (15/11/2026)

| Serviço | Dependência / persistência | Comportamento |
|---|---|---|
| CompositorService | niri --version | probe read-only no startup; não lê/escreve KDL |
| ShortcutService | catálogo estático + SettingsService.keys; NiriMod opcional | bindings não editáveis no original; não inventar editor |
| ProfileService | SettingsService, ThemeService, WallpaperService, Palettes; state/profiles.json; XDG_DATA_HOME/velatura/profiles | import/export JSON v1, profiles shipped, active settings stashed |
| ControlsService | Settings/Theme + Network/Bluetooth/Audio/System/Notification/Sunset/Capture/Picker/Recorder/Clipboard | domínio de doors/toggles/blocks; persistência centre*; debounce 120 ms |
| PickerService | hyprpicker, wl-copy, Theme settle, OsdService | probe uma vez, cancel, hex #RRGGBB, clipboard |
| CalendarService | dcal IPC/Dank Calendar | inicia daemon hidden se ping falha; calendário externo é fonte de verdade |
| CavaService | cava + scripts/cava.conf | 64 bands stdout; só com subscribers/linger; game mode para |
| GamesService | games.json XDG state | 11 jogos; best scores/plays; debounce 120 ms |
| PetService | pet.json XDG state + LockService/SettingsService/Theme | XP, cooldown, rest, mood, eggs; clock/timer condicionado a placement |

Com os nove serviços acima, os 61 singletons QML ativos do catálogo foram lidos. Isso fecha a auditoria estática dos adapters; não fecha a auditoria de superfícies visuais nem testes reais de runtime.
