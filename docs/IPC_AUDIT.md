# Auditoria do IPC e da CLI — Velatura original

**Data:** 15/11/2026<br>
**Fontes:** `/home/px/.config/quickshell/Commands.qml` (607 linhas, leitura integral) e `/home/px/.local/bin/velatura` (470 linhas, leitura integral).<br>
**Status:** auditoria estática concluída; não foram executados comandos de mutação via CLI.

## Contrato geral do IpcHandler

- Acesso documentado no original: `qs ipc call <target> <verb> <args>`; o wrapper `velatura` traduz subcomandos amigáveis para essa chamada.
- Argumentos chegam ao handler como strings; argumentos opcionais podem chegar vazios.
- Helpers centrais: `ok(extra)` serializa `{"ok":true,...}`; `fail(message)` serializa `{"error":"..."}`.
- `switched(word,current)`: string vazia equivale a `toggle`; aceita `on/true/yes/1`, `off/false/no/0`, `toggle`; outros valores retornam erro.
- O retorno de sucesso em vários handlers significa "pedido aceito", não necessariamente efeito de sistema concluído. Isso é explícito para máquinas (`asked`) e implícito para várias operações async.
- Exceções ao JSON uniforme: `wallpaper.set` retorna o caminho literal ou uma string de uso; `bar.toggle`, `widgets.toggle` e `shell.reload` não retornam string (o wrapper trata stdout vazio como sucesso).
- O wrapper adiciona outra camada: timeout 15 s para IPC, normaliza erro de shell ausente, target/verb desconhecido e argumentos em excesso; saída não JSON vira `{"ok":true,"result":"..."}`.

## Targets e semântica real

| Target | Verbos | Validação e payload/efeito observado |
|---|---|---|
| `status` | `get` | Snapshot agregado: timer ou null, media ou null, contagem de notes, contagem tasks por state, nomes de VMs running, palette, wallpaper, layout, game, focus/DND, nightLight, volume/muted e brightness. |
| `timer` | `start`, `pause`, `resume`, `cancel`, `get` | `start` usa `TimerService.parse`; exige ms > 0; exemplos 25m, 1h30m, 90s, 10:00. pause/resume/cancel falham se não há timer; get retorna running/paused/label/left. |
| `note` | `add`, `list`, `get`, `append`, `archive`, `remove` | add exige texto ou título; tint aceita yellow/pink/blue/green/orange/purple e fallback yellow. list só live; get retorna erro se key ausente; append adiciona newline; archive/remove validam key. Row: key/title/text/tint/archived/edited ISO. |
| `task` | `add`, `list`, `move`, `done`, `due`, `remove` | States todo/doing/done; texto obrigatório; data aceita today/tomorrow/fri/12 oct/+3/ISO, vazio limpa; state vazio default todo; notes opcionais. Row inclui key/text/notes/state/due/overdue. |
| `panel` | `open`, `close`, `list` | Allowlist fixa: launcher, controls, overview, appearance, palette, stats, session, pet, games, notes, board, keys, packages, machines, servers, screentime, camera, wifi, bluetooth, sound, microphone, brightness, nightlight. |
| `machine` | `list`, `start`, `stop`, `kill`, `pause`, `resume`, `open` | Lista name/title/running/paused/cores/memory. Ação valida existência e busy state; sucesso é `{name,asked}`, não confirmação de conclusão. |
| `media` | `get`, `play`, `pause`, `toggle`, `next`, `prev` | get metadata/player/playing ou null; play/pause são idempotentes conforme estado; next/prev validam capabilities. |
| `capture` | `open` | destino vazio default file; shape deve existir em `CaptureService.shapes`; destino file/editor/text; chama photo e settle; retorna pedido aceito. |
| `settings` | `toggle` | exige callback da janela disponível. |
| `launcher` | `mode` | Somente clipboard ou emoji; exige callback disponível. |
| `picker` | `pick` | Exige callback disponível; picker é pedido assíncrono. |
| `recorder` | `toggle` | Encaminha a RecorderService sem validação adicional no handler. |
| `game` | `set` | Usa `switched` para SettingsService.gameMode. |
| `layout` | `set` | Aceita vazio/scrolling/scrolling-columns e responde scrolling-columns. Outros layouts falham: Niri usa native scrolling-column; extensão separada seria necessária. |
| `focus` | `set` | Na realidade alterna `SettingsService.doNotDisturb`/NotificationService DND. |
| `nightlight` | `set` | Usa `switched`; falha se SunsetService indisponível (wlsunset); alterna estado se necessário. |
| `bar` | `toggle` | Inverte `barHidden`; retorno void. |
| `widgets` | `toggle` | Inverte `desktopHidden`; retorno void. |
| `volume` | `set`, `mute`, `get` | set aceita inteiro 0–100; mute usa switched; get volume/muted. |
| `brightness` | `set`, `step`, `get` | set aceita inteiro 0–100; step aceita delta não zero entre -50 e 50; get percent atual. |
| `theme` | `list`, `set` | list current + IDs/names; set valida ID na lista disponível. |
| `wallpaper` | `set`, `list`, `random` | set vazio retorna string de uso; caso contrário chama apply e retorna path literal. list inclui wallpapers estáticos + animated. random escolhe outro somente da lista estática e falha se não houver outro. |
| `osd` | `say` | Texto obrigatório; ícone default Material; chama OsdService.requested com timeout -1. |
| `lock` | `lock` | Encaminha LockService.lock. |
| `shell` | `reload` | Chama Quickshell.reload(false); retorno void. |

## Wrapper `~/.local/bin/velatura`

1. **Responsabilidade:** CLI humana/scriptável, completion, watch, capture e version, além de encaminhar targets IPC.
2. **Parser:** tabela `COMMANDS` descreve noun/verb/positional/flags; aceita argumentos com resto (`...`), opcionais e flags; injeta string vazia para argumentos omitidos; `--` protege argumentos que começam com hífen.
3. **IPC:** chama `qs ipc call target function -- args`, timeout 15 s; detecta `qs` ausente, shell ausente, target/verb não encontrado e mensagens de argumentos; resposta não JSON vira `result`.
4. **Local commands:** `help`, `version`, `capture screen/text/region/window`, `watch`, `complete`.
5. **Capture local:** screen/text consulta workspace focado via `niri msg --json workspaces`, chama `scripts/capture.py grab` e depois `finish --to file|text`; screen retorna path/copied; text retorna OCR/copied. region/window encaminha `capture open`.
6. **Watch:** polling de `status.get` com intervalo default 1 s (mínimo 0,25 s), primeira linha status, depois eventos derivados de timer/media/machines e mudanças em notes/tasks/palette/wallpaper/layout/game/focus/nightLight/volume/muted/brightness. Não é event stream nativo.
7. **Completion:** consulta listas de notes/tasks/machines/panels/themes e sugere estados/tints/switches.
8. **Version:** lê `$XDG_STATE_HOME/velatura/version` (fallback `~/.local/state/velatura/version`).
9. **Teste:** parser de args/flags, espaços, hífen inicial, flags ausentes, payloads não JSON, shell ausente, timeout, watch mudanças, completion, capture e version.
10. **Rust:** CLI compatível deve manter os comandos e o formato de saída/exit code, mas pode corrigir divergências identificadas com registro de decisão e compatibilidade explícita.

## Divergências e defeitos encontrados

1. **Layout CLI vs IPC:** a CLI oferece `dwindle|master|scrolling|next`, mas o handler só aceita vazio, `scrolling` e `scrolling-columns`. Valores listados pela ajuda falham no handler. O port não deve implementar layouts que Niri não oferece sem integração real.
2. **Brightness step:** `Commands.qml` e `IPC_API.md` expõem `brightness.step`, mas a tabela de comandos da CLI não oferece subcomando/flag para step.
3. **Capture region/window:** wrapper encaminha `capture.open` com apenas `shape`, enquanto o handler declara `open(shape,destination)`. Confirmar comportamento real do IpcHandler para argumento omitido e corrigir mantendo compatibilidade.
4. **Capture IPC default:** handler vazio de destination vira `file`; a CLI local não oferece destino no caminho region/window.
5. **Wallpaper set:** é exceção histórica ao JSON do handler; o wrapper normaliza texto puro em campo `result`. Preservar o comportamento externo da CLI mesmo se o IPC interno nativo tiver envelope estruturado.
6. **Watch:** é polling deliberado do wrapper (mínimo 250 ms), diferente do event bus interno. A versão nativa pode usar subscriptions para detectar mudanças, mas deve manter o contrato de linhas JSON, timestamp e eventos observáveis.
7. **Capture helpers:** `scripts/capture.py` ainda precisa de auditoria integral antes de definir backend de captura/OCR.

## Próximos testes de compatibilidade

- Fixtures de cada target/verb com args válidos, vazios, inválidos e mensagens de erro.
- Comparar saída JSON e exit code do wrapper, incluindo casos de retorno void e texto cru.
- Testar scripts que usam `velatura` e `qs ipc call` sem iniciar UI.
- Cobrir divergências acima com testes de regressão e documentar decisão antes de mudar a interface pública.

### Divergências adicionais confirmadas após leitura do motor de captura

- `CaptureService.shapes` contém somente `region` e `screen`; o valor persistido legado `window` é exibido como `region`. Portanto, o comando CLI `capture window` não corresponde a uma captura de janela real e o handler rejeita o shape `window`.
- `scripts/capture.py` não implementa seleção de janela; faz screenshot da tela/output inteiro e o recorte é uma região retangular.
- `capture.open(shape,destination)` recebe dois parâmetros no QML, mas o wrapper region/window fornece só um; destino padrão do handler é `file` quando chega string vazia. Requer fixture para confirmar o comportamento do dispatcher.
- O capture UI oferece photo/video, region/screen e destinos file/clipboard/editor/text. A CLI screen/text chama diretamente grab+finish e não usa o overlay.
