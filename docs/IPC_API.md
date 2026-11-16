# API IPC pública do Velatura original

Fonte: Commands.qml ativo. Cada IpcHandler é chamado por qs ipc call <target> <verb> <args> e pelo wrapper velatura. Respostas JSON por linha: sucesso { "ok": true, ... }; falha { "error": "..." }. Argumentos entram como strings.

> A versão nativa deve preservar os targets, verbos, defaults, validação, semântica e respostas usados por automações. Não remover nem renomear comandos sem decisão explícita.

## status
- get()

## timer
- start(duration: string, label: string)
- pause()
- resume()
- cancel()
- get()

## note
- add(text: string, title: string, tint: string)
- list()
- get(key: string)
- append(key: string, text: string)
- archive(key: string)
- remove(key: string)

## task
- add(text: string, due: string, state: string, notes: string)
- list(state: string)
- move(key: string, state: string)
- done(key: string)
- due(key: string, when: string)
- remove(key: string)

## panel
- open(name: string)
- close()
- list()

## machine
- list()
- start(name: string)
- stop(name: string)
- kill(name: string)
- pause(name: string)
- resume(name: string)
- open(name: string)

## media
- get()
- play()
- pause()
- toggle()
- next()
- prev()

## capture
- open(shape: string, destination: string)

## settings
- toggle()

## launcher
- mode(name: string)

## picker
- pick()

## recorder
- toggle()

## game
- set(word: string)

## layout
- set(name: string)

## focus
- set(word: string)

## nightlight
- set(word: string)

## bar
- toggle()

## widgets
- toggle()

## volume
- set(percent: string)
- mute(word: string)
- get()

## brightness
- set(percent: string)
- step(delta: string)
- get()

## theme
- list()
- set(id: string)

## wallpaper
- set(path: string)
- list()
- random()

## osd
- say(text: string, icon: string)

## lock
- lock()

## shell
- reload()

## Compatibilidade exigida

- Preservar todos os targets e verbos listados.
- Preservar defaults, validação e mensagens de erro relevantes para scripts existentes.
- Preservar toggle/on/off, argumentos vazios e datas relativas.
- Distinguir ação aceita, ação concluída e falha assíncrona.
- Estabilizar schema JSON e nunca expor segredos.
- Testar CLI sem abrir a interface gráfica.



## Auditoria do código (15/11/2026)

A leitura integral de `Commands.qml` e do wrapper `~/.local/bin/velatura` está em [IPC_AUDIT.md](IPC_AUDIT.md). O catálogo acima enumera targets/verbs, mas o audit registra validação, payloads, respostas não JSON, comandos locais e divergências reais.

Pontos que precisam permanecer explícitos no contrato nativo:

- Nem todo handler retorna JSON: `wallpaper.set` devolve caminho/uso literal; `bar.toggle`, `widgets.toggle` e `shell.reload` retornam void.
- `focus.set` controla DND, não foco do compositor.
- `layout.set` não troca layout no Niri: aceita somente scrolling-column nativo.
- `brightness.step` existe no IPC, mas está ausente na CLI.
- A CLI lista layouts que o handler rejeita; `capture region/window` chama `capture.open` com menos argumentos do que a assinatura declara.
- `velatura watch` é polling do status com intervalo configurável (mínimo 250 ms), não o event stream Niri.
- `capture screen/text`, `version` e completion têm implementação local no wrapper, além dos handlers QML.

O backend Rust deve preservar o contrato externo usado por scripts, e qualquer correção de divergência precisa de teste de compatibilidade e registro em DECISIONS.md.
