# Auditoria das integrações externas restantes

**Data:** 15/11/2026. **Fontes primárias:** leitura direta dos 16 scripts Python ativos restantes em `/home/px/.config/quickshell/scripts/` e dos respectivos adapters `services/*.qml`. Os scripts originais permanecem read-only.

Esta ficha registra comportamento observado, não implementação Rust nem verificação em runtime. `AUDITED` não significa que ferramentas, endpoints ou privilégios foram testados com mutações.

## 1. Conta e identidade — account.py / AccountService

- **Comandos:** `get`; `name <text>`; `picture <path>`; `picture --clear`. Saída JSON contém `user`, `name`, `fullName`, `avatar`, `accounts`, `shared` ou `error`.
- **Leitura:** UID via `pwd`; nome GECOS (primeiro campo) ou login; avatar primeiro existente entre `/var/lib/velatura/faces/{user}.face.icon`, `~/.face`, `~/.face.icon`, `/var/lib/AccountsService/icons/{user}`; `AccountsService.FindUserByName` via `busctl` (5 s).
- **Escrita de nome:** `AccountsService.User.SetRealName` via busctl, timeout 10 s; sem AccountsService, QML guarda fallback em `SettingsService.userName`.
- **Escrita de imagem:** requer `/usr/lib/velatura/avatar` executável + policy; ImageMagick normaliza primeira frame para PNG 256×256 e envia bytes stdin para `pkexec HELPER set`; limpar usa `pkexec HELPER clear`; timeout 30/60 s. Se helper não estiver instalado, QML guarda caminho em settings.
- **Lifecycle:** AccountService lê no startup; nome é debounce 800 ms e espera writer anterior; sucesso limpa override local, falha preserva fallback e expõe `failedKind`/`failure`.
- **Riscos/bloqueios:** helper e policy de avatar não encontrados no sistema nem nos diretórios pesquisados; sem código-fonte do helper, não portar operação privilegiada. `busctl` é CLI sobre D-Bus, pode ser substituído por zbus. Nunca expor imagem privada em log.

## 2. Camera effects — camera.py / CameraService

- **CLI:** `status`, `configure JSON`, `snap`, `serve`, `run FD` (run é child interno). Status verifica loopback chamado `velatura camera`, webcam colorida, módulos Python `cv2`, `mediapipe`, `numpy` e três modelos MediaPipe (`face_landmarker.task`, `gesture_recognizer.task`, `selfie_segmenter.tflite`).
- **Dispositivos:** enumera `/sys/class/video4linux`, lê nomes e formatos via V4L2 ioctl; ignora loopback e câmera GREY-only (IR); seleciona primeira câmera colorida. Loopback deve aceitar saída YU12/I420 1280×720 a 30 FPS.
- **Servidor:** cria diretório em `$XDG_RUNTIME_DIR/velatura-camera`; PID file; reivindica loopback, subscreve evento V4L2 client-usage; fica idle sem webcam; só cria child `run` quando há reader; encerra child quando stream fecha; descobre nomes dos readers em `/proc`; reinicia displaced server validando `camera.py` em cmdline.
- **Processamento:** OpenCV + MediaPipe FaceLandmarker/GestureRecognizer/ImageSegmenter; efeitos `natural`, `palette`, `mono`, `warm`, `blur`, `stage`, `shades`, `crown`, `sparkle`; gestos `thumbsup`, `thumbsdown`, `heart`, `peace`, `horns`, `ok`, `middle`; reações configuráveis, 3 frames para confirmar e cooldown 2.5 s. Processa frames, pose 3D, máscaras, sprites emoji e escreve YUV I420 no fd do loopback.
- **Fotos:** `snap` sinaliza arquivo no runtime, espera até 2 s pela próxima frame espelhada, salva em `$VELATURA_CAMERA` ou XDG camera/pictures/Camera ou `~/Camera`, e usa CaptureService para copiar.
- **Lifecycle QML:** `wanted = ready && SettingsService.cameraEnabled`; servidor auto gerido enquanto wanted; probe ao completar e ao abrir painel; retry 3 s, para após mais de 4 falhas curtas; config coalescida em 120 ms; painel 820×517; expõe state/readers/fps/faces/error/others e OSD ao fotografar.
- **Dependências:** v4l2loopback (kernel module/device), OpenCV, MediaPipe tasks, NumPy, Pillow, ImageMagick font NotoColorEmoji, três modelos baixados em XDG_DATA_HOME, permissões de vídeo, capture.py/wl-copy. Não instalar nem carregar módulo durante auditoria.
- **Riscos/paridade:** workload de 720p30 + MediaPipe é caro para i5-5350U; precisa benchmark e fallback/estado de erro sem retirar efeitos. Watcher de leitores tem scan `/proc` periódico 5 s e poll V4L2 1.5 s. Config path em runtime e PID lifecycle devem ser robustos a crash e PID reuse. Modelos/artefatos devem ser baixados sob demanda com hash/versão, não presumir instalados.
## 3. Claude Code e Codex usage — claude_usage.py / codex_usage.py

### Claude
- Lê `~/.claude/projects/**/*.jsonl`; cache incremental por arquivo com offset, mtime e buckets horários em `$XDG_STATE_HOME/quickshell/claude-usage.json` (schema v1). Se arquivo truncou/rotacionou, recomeça. Soma input/output/cache-creation tokens e mensagens; exclui cache-read tokens. Calcula janela corrente de 5 h, últimos 7 dias e picos históricos.
- `limits` lê OAuth access token de `CLAUDE_CONFIG_DIR/.credentials.json` ou `~/.claude/.credentials.json`, plano de `~/.claude.json`, consulta Anthropic `/api/oauth/usage`; fallback faz request mínimo `/v1/messages` e lê headers unified 5h/7d. Timeout 12 s, cache de limites 300 s; quando fallback, mantém limites por modelo ainda válidos do cache.
- ClaudeService inicia consulta ao construir singleton; transcript refresh a cada 120 s apenas com watchers; limites a cada 600 s com watchers; módulo depende de `available`. Sem credenciais/API, contagens locais podem funcionar, percentuais ficam ocultos; falha não apaga último dado local.
- **Risco de segredo:** o script entrega `authorization: Bearer <token>` como argumento de `curl`; o token pode ser visível em `/proc/<pid>/cmdline` durante a requisição. Rust deve usar cliente HTTP nativo e header em memória, nunca argv/logs; não persistir nem renovar token.

### Codex
- Procura `CODEX_HOME/sessions/rollout-*.jsonl` (default `~/.codex/sessions`), ordena por mtime e examina no máximo 8 logs, lendo até 256 KiB finais de cada; busca último `payload.rate_limits` e timestamp. Sem rede e sem cache.
- Normaliza `primary`/`secondary`: `used_percent` 0..100, `window_minutes`, `resets_at` ou `resets_in_seconds`, plano e observed timestamp. Ignora porcentagens inválidas; janela nomeada Month/Week/days/hours.
- CodexService consulta ao construir e ao subscribe; timer 120 s apenas enquanto watchers > 0. Dados são tão atuais quanto o último turno Codex; janela após reset desaparece até novo log.
- **Dependências/privacidade:** leitura somente local dos logs; logs podem conter outros dados, mas parser só deve abrir e interpretar `rate_limits`, nunca copiar conteúdo completo ou logar linhas.

## 4. Emoji, fontes e informações da máquina

- **emoji.py / EmojiService:** lê `/usr/share/unicode/emoji/emoji-test.txt`, CLDR `common/annotations/<lang>.xml` e `iso-codes/json/iso_3166-1.json`; nomes localizados, fallback inglês/Unicode, palavras-chave com normalização sem acentos, grupos, flags e variantes de tons de pele. Ignora componentes e entradas não fully-qualified; ZWJ names compostos por partes. QML carrega lazy na primeira entrada do modo emoji, recarrega após mudança de idioma, guarda 24 recentes em `emoji-recent.json`; tom é setting global; copia via wl-copy. Ausência de pacote gera lista vazia, não crash.
- **fonts.py / FontService:** `fc-list : family` e `fc-list :spacing=100 family`, timeout 10 s; primeira alias é canonical, deduplica/sort case-insensitive; `sans` é total menos mono. Lazy somente ao abrir settings. Sem fontconfig retorna arrays vazios.
- **machine.py / MachineService:** facts sob demanda (não polling): PRETTY_NAME em `/etc/os-release` ou `/usr/lib/os-release`, `os.uname`, `XDG_CURRENT_DESKTOP` + `niri --version` quando Niri, shell de passwd, contagem de diretórios `/var/lib/pacman/local`. Campos ausentes são null; sem pacman, packages null.
- **Persistência/risco:** emoji recent em XDG state; nenhuma persistência para fonts/machine. Não enumerar nem expor arquivos de transcript; comandos externos são fc-list/niri version; restante pode ser implementado com APIs Rust/FS.

## 5. GitHub, markets e weather

- **github.py / GithubService:** scrape HTML público `https://github.com/users/{user}/contributions` com curl timeout 12 s; regex lê dia/data/nível, tooltip count e total heading; retorna semanas de 7 dias, total, streak, today, busiest. Username vazio ou perfil sem calendário vira `available:false, reason:user`; outros erros indisponíveis. Sem token. Service consulta ao construir, subscribe atualiza se ausente/mais de 15 min, poll 30 min enquanto watchers; falha mantém último grid; username novo atualiza imediatamente.
- **markets.py / MarketsService:** argumentos são pares BASE/QUOTE ou BASE→USD. FIAT da lista suportada usa Frankfurter `/v1/{start}..?from=...&to=...` (45 dias de referência, guarda 30, variação contra último dia útil); cripto usa CoinGecko `/api/v3/coins/markets` com sparkline 7d e change 24h, 1 request por quote, amostra cada 3 horas. Sem API key; timeout 12 s; desconhecidos separados; falha parcial preserva último valor por par. Service parseia lista de settings, debounce 800 ms, polling 5 min com watchers, mantém até 4 pares secundários.
- **weather.py / WeatherService:** `wttr.in/{place}?format=j1`, place codificado URL; vazio permite geolocalização aproximada por IP. Current conditions + min/max do dia + blocos 3h de hoje e amanhã; glyph por weatherCode. Timeout 12 s; place desconhecido é distinguido de erro geral. Service busca após settings chegarem, ao mudar place e ao subscribe se stale/mais de 10 min; polling 15 min com watchers; mantém último reading em falha e marca `placeUnknown`.
- **Observação de defeito legado:** `weather.py` define `daytime = current['observation_time'] and True`, portanto sempre truthy quando a string existe e glyph atual tende a usar variante diurna. O comentário diz que day/night deveria derivar dos blocos horários; registrar correção como decisão separada para não alterar comportamento silenciosamente.
- **Riscos:** curl é processo externo; substituir por HTTP client com timeout, user-agent e parse robusto. GitHub markup/Frankfurter/wttr/CoinGecko podem mudar ou rate-limit; `available`, `complete`, `unknown`, last-good e timestamp precisam permanecer explícitos.
## 6. Pacotes, atualizações e versão do shell

- **packages.py / PackagesService:** `installed` executa `pacman -Qi` + `pacman -Qqm`; parseia multiline, descrição, tamanho, motivo explícito e origem AUR; detecta `yay`/`paru`. `search TERM` exige 2+ caracteres, consulta pacman `-Ss` com regex escapada e AUR RPC `/search/{term}?by=name`; se amplo, usa `/suggest` + `/info`; resultados exatos/prefixos/substrings, repos antes de AUR por tier, popularidade, máximo 160. Threads para repo/AUR; timeout subprocess 60 s, AUR 8 s.
- **Ações reais do QML:** instalar AUR só com helper; instalar repo via helper ou sudo pacman -S; remover `sudo pacman -Rns`; upgrade sempre `-Syu` (nunca partial upgrade). Executa em kitty com output e prompts visíveis, bloqueia outra ação enquanto terminal está aberto; ao sair recarrega instalados, Updates e pesquisa. Não fazer install/remove/update em background. O `holdScript` é shell de terminal com argv separado, não deve virar concatenação de input.
- **updates.py / UpdatesService:** tenta `checkupdates --nocolor` (2 = sem updates); fallback `pacman -Qu` (1 sem stderr = vazio); lista pacotes repo. AUR foreign via `pacman -Qm`, RPC `/info` em batches de 150, compara com `vercmp`; AUR offline não invalida resultado repo. Retorna tool, aur, count, packages (15) e updates completos. Service consulta ao construir, polling 30 min com watchers.
- **version.py / VersionService:** recebe repo/version/tree conhecidos; `git -C` com env LANG=C, `GIT_TERMINAL_PROMPT=0`, SSH BatchMode, OPTIONAL_LOCKS=0; check faz `git fetch --quiet --tags` timeout 25 s; status compara branch/upstream, ahead/behind, base instalada e tree copiada, lista 12 commits e target git describe. Fetch muda refs locais, mas não faz checkout/merge. `setup update` é aberto em kitty/terminal para prompt/reload; `setup files list` e `setup files <action> paths...` listam/revertem alterações no HOME. Manifest/version são observados via FileView; check cache 30 min e stale quando versão muda.
- **Dependências:** pacman, checkupdates, vercmp, yay/paru, AUR RPC, git, SSH, kitty, checkout Velatura + `setup` executável e manifest `$XDG_STATE_HOME/velatura`. Helpers ausentes devem esconder/indisponibilizar ações com motivo, nunca fingir sucesso.

## 7. Privacidade de câmera, screen time e servidores

- **privacy.py / PrivacyService:** lista `/dev/video*`; snapshot inicial lê `/proc/*/fd` e `/proc/<pid>/{cmdline,comm}` para holders do usuário atual, exclui `camera.py`, normaliza `pipewire`/`wireplumber` para broker. Depois usa `inotifywait -m -e open,close` com settle 250 ms para agrupar probes antes de reler /proc; filho recebe `PR_SET_PDEATHSIG(SIGTERM)`. QML combina holders com PipeWire nodes: mic listeners de AudioService, video viewers e portal `Video/Source`; sinaliza screen sharing e nomeia app. Construído somente quando `ModuleService.activities` permite Privacy; não polling.
- **Limites privacy:** processo root (ex. howdy na lock screen) não é visível em `/proc` do usuário; devices surgidos após startup não entram na lista inotify atual; se não houver `/dev/video*` o script sai e não vê hotplug futuro. Rust deve acompanhar udev/hotplug e documentar visibilidade por UID sem elevar privilégios.
- **screentime.py / ScreenTimeService:** `record` consome stdin (`focus CLASS`, `away`, `back`, `unlock`, `notify APP`), armazena somente classe da janela, nunca título. Um TSV diário em `$XDG_STATE_HOME/quickshell/screentime`; `flock` no append; fecha/split à meia-noite; checkpoint 300 s; tick 30 s e GAP 75 s detecta suspensão/crash; limpa arquivos com mais de 400 dias. `summary` e `report day|week|month DATE` calculam spans, sessões unidas até 10 s, classes, categorias, unlocks, notifications, heatmap, charts, longest session, baseline e período anterior; Steam app IDs resolvidos por manifest nas bibliotecas Steam nativas/Flatpak.
- **ScreenTimeService lifecycle:** recorder inicia ao construir singleton, recebe foco de Niri, lock/unlock, idle-screen e notifications; restart após exit em 30 s. `away = locked || screenWatch.enabled && screenWatch.isIdle`; screen off por hardware/manual sem Idle lock continua contando. Panel watchers consultam report, minuto a minuto enquanto período contém hoje; summary followers consultam a cada minuto. Category mapping tem override `vesktop.desktop`, Steam→games e categorias freedesktop. `watchers` e `followers` são contagens distintas.
- **servers.py / ServersService:** `ss -Hltnp` timeout 5 s, só TCP LISTEN; usa PID/port/host; aceita processos do UID atual cujo cwd esteja dentro de HOME e ancestral de projeto contenha marker (`package.json`, `Cargo.toml`, `.git`, etc.). Lê `/proc/{pid}/{cwd,cmdline,exe,stat,comm}`, boot time e start ticks; retorna project/path/runner/kind/started/port/network (non-loopback). Poll 3 s somente com painel subscribed; open localhost URL; copy via wl-copy.
- **Stop server:** valida novamente PID+porta na lista, envia SIGINT ao process group quando não é session leader e não é o grupo do shell; espera 3 s e SIGTERM, espera mais 2 s. Restringe ao processo do usuário e project marker. **Risco:** PID/porta podem mudar entre validação e kill; Rust deve revalidar starttime/executable/cwd imediatamente e preferir pidfd, sem sinalizar grupo do próprio shell.
## 8. Privileged face/avatar helpers — fonte encontrada no checkout

**Fonte adicional:** `/home/px/impasto-niri/setup`, `system/usr/lib/velatura/{face,avatar}`, policies polkit e `system/etc/pam.d/velatura-face`. O caminho gravado em `VersionService` confirma esse checkout como origem da instalação. A auditoria anterior não tinha localizado esse repositório; a fonte foi encontrada e lida agora. O conteúdo original segue read-only.

### `system/usr/lib/velatura/face`
- Executa como root via pkexec; exige `PKEXEC_UID`, resolve usuário com `id -nu`, define `LANG=C`/`LC_ALL=C`.
- `list`: `howdy list --plain -U <user>`, stderr suprimido, mantém linhas CSV `^[0-9]+,`, sempre exit 0 para lista vazia/erro de howdy.
- `add [label]`: remove `/`, `\\` e caracteres de controle, limita label a 24 chars, executa `howdy add --plain -y -U <user> [-- label]` em foreground.
- `remove <id>`: aceita apenas dígitos e executa `howdy remove -y -U <user> <id>`.
- Policy separa `org.velatura.face.list` (allow_active yes, sem senha) de add/remove (auth_self_keep); helper restringe operação aos modelos do usuário que chamou pkexec.

### PAM face e setup
- `system/etc/pam.d/velatura-face`: `-auth sufficient pam_howdy.so` seguido de `auth required pam_deny.so`. É um PAM service separado, não inserido em `login`; falha facial não consome tentativa de senha e senha não espera pela câmera.
- `setup system` instala árvore `system/` no root e cria `/var/lib/velatura/faces` mode 755; não executado nesta auditoria.
- `setup face` exige v4l2-ctl e câmera que exponha GREY; prefere `/dev/v4l/by-path` estável; exige PAM já instalado; instala `howdy-next` por yay/paru quando ausente, baixa modelos, configura device_path + detection_notice e opcionalmente pergunta para enrolar. Não executado.
- LockService mantém face scanning separado da senha: acorda por ação explícita em tela ativa, 3 misses no máximo, rest 1.5 s (idle 5 s quando Howdy não começou a olhar), face match segura 650 ms antes de liberar. PAM face é só iniciado se locked, secure, awake, enrolled/listed e não leaving/matched/active.

### `system/usr/lib/velatura/avatar`
- Também root via pkexec/PKEXEC_UID; só escreve a imagem do usuário chamador em `/var/lib/velatura/faces/<user>.face.icon`.
- `set` lê PNG por stdin (não aceita caminho root-side), limita 2 MiB, valida assinatura PNG, grava staging com mktemp, chmod 644 e rename atômico; `clear` remove só o avatar desse usuário.
- Policy avatar set/clear permite usuário ativo sem prompt (AccountsService-equivalent); `setup system` cria pasta pública para SDDM/lock lerem.
- AccountService usa ImageMagick como usuário para primeira frame, auto-orient, 256×256 crop/extent, strip e PNG; só bytes PNG seguem ao helper. Se helper/policy ausentes, QML usa avatar path local em settings.

### Estado real desta máquina e portabilidade
- Os helpers e policies fonte existem no checkout `/home/px/impasto-niri`, mas **não estão instalados**: `/usr/lib/velatura/{face,avatar}`, policies em `/usr/share/polkit-1/actions`, `/etc/pam.d/velatura-face` e `/usr/lib/security/pam_howdy.so` ausentes; `howdy` também ausente.
- `v4l2-ctl`, `pkexec`, `magick` existem; `inotifywait` e `howdy` não existem. Não foi carregado módulo, instalado pacote, aberto prompt polkit/PAM nem tocado em camera/face.
- Rust deve portar helper com privilégio mínimo e validar `PKEXEC_UID`, usuário, id numérico, label, tamanho/assinatura PNG, owner/mode, staging no mesmo filesystem e operações atômicas. Não aceitar path arbitrário privilegiado, não ler arquivos como root fornecidos pelo usuário e não armazenar senha.
- O helper Bash é parte do contrato operacional do setup, mas a versão final pode ser um pequeno binário Rust root-owned somente após auditoria de packaging/polkit e testes isolados. Não substituir PAM nem alterar login global.

### Probes read-only da máquina — integrações adicionais

- camera.py status retornou ready=false, missing=[loopback, cv2, mediapipe, models], camera=/dev/video0, loopback=null. O único device é /dev/video0 “Apple Facetime HD”, com formatos YUYV/YVYU; não há loopback Velatura. O módulo v4l2loopback existe no disco (modinfo), mas não está carregado; nenhum modprobe foi executado.
- Python: numpy e PIL disponíveis; cv2 e mediapipe ausentes. Os três modelos MediaPipe não existem em XDG_DATA_HOME/velatura/camera. A câmera física foi apenas enumerada; nenhum frame foi capturado.
- dcal, hyprpicker, cava e niri estão no PATH. Nenhum picker foi iniciado, nenhum daemon dcal foi iniciado e cava não foi executado.
- inotifywait e howdy ausentes; privacy.py não consegue iniciar o watcher de device e FaceService não fica ready. v4l2-ctl mostra apenas YUYV/YVYU no device atual, sem evidência de formato GREY/IR.
- Esses probes são evidência de disponibilidade local, não teste de integração. Nenhum pacote instalado, módulo carregado, configuração alterada ou serviço original interrompido.

### Detalhes exatos de setup camera/face (fonte lida)

- setup camera exige v4l2-ctl e uma câmera colorida; instala v4l2loopback-dkms, headers do kernel, python-numpy e python-opencv; se import mediapipe falhar, instala python-mediapipe-bin via yay/paru.
- Escreve /etc/modprobe.d/velatura-camera.conf com video_nr=10, card_label=velatura camera, exclusive_caps=1 e /etc/modules-load.d/velatura-camera.conf com v4l2loopback; tenta modprobe agora se o módulo não estiver carregado, senão informa que o device aparecerá após reboot.
- Baixa três assets MediaPipe do Google Storage para XDG_DATA_HOME/velatura/camera, usando .part + rename, mas não valida checksum/hash/signature. Rust deve adicionar verificação de integridade e version pin sem mudar o conteúdo funcional dos modelos.
- setup face seleciona primeiro device GREY e prefere /dev/v4l/by-path estável; exige PAM velatura-face já instalado por setup system; instala howdy-next via AUR helper, executa howdy download-models, configura device_path e detection_notice=true e só faz enrollment se o usuário aceitar interativamente.
- Nenhum desses verbos foi executado. O caminho de instalação altera config de modprobe, modules-load, pacotes e dados do usuário/root; exige autorização explícita em uma etapa futura.
