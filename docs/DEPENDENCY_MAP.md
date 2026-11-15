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

## Pendências da auditoria

- Ler integralmente cada script e registrar endpoints, schemas, cache, caminhos XDG e flags.
- Capturar saídas reais dos probes tools/status de capture, record e camera.
- Verificar dependências realmente disponíveis no ambiente alvo.
- Mapear comandos dinâmicos e execDetached não capturados por literais.
- Definir quais integrações rodam no processo principal e quais são subprocessos opcionais.

