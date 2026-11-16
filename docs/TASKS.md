# Backlog e estado do projeto

**Estado geral:** auditoria estática do backend concluída (61/61 serviços QML e 28/28 scripts Python ativos); implementação Rust inclui sessão Niri integrada, snapshot cacheado/revisionado, bridge GTK sem polling, matching GDK↔Niri e `ApplicationRuntime` RAII. Ainda pendem protótipo GTK layer-shell, interface visual 1:1 e validação física ampla de runtime/paridade. Nenhuma funcionalidade marcada como VERIFIED.

## Fase 0 — Preparação

- [x] Criar repositório local em ~/Projects/velatura-native.
- [x] Mapear diretórios e arquivos de alto nível da instalação original.
- [x] Criar regras compartilhadas para os agentes.
- [x] Criar prompt de backend.
- [x] Criar prompt de frontend para Codeína.
- [x] Criar design.md inicial baseado nos tokens e componentes reais.
- [x] Criar matriz de portabilidade inicial.
- [x] Inventariar serviços QML ativos, símbolos e consumidores.
- [x] Catalogar scripts Python e primeiras dependências.
- [x] Extrair targets e verbos da API IPC original.
- [x] Criar mapa inicial de dependências do sistema.
- [x] Mapear referências entre serviços e lifecycle de startup observado.
- [x] Extrair chaves e defaults do schema JsonAdapter.
- [x] Catalogar módulos, painéis, settings e faces visuais.
- [ ] Validar documentação visual com screenshots reais de cada superfície.
- [x] Auditar estaticamente a composição raiz de shell, bar/reserve, Dynamic Island host, Settings navigation, Desktop, Dock, Deck, Lock e Capture; contratos em UI_SURFACE_AUDIT.md (15/11/2026).
- [x] Auditoria linha a linha da primeira fatia: lifecycle, Niri, monitores, Settings, áudio, rede, Bluetooth, bateria, brilho, notifications, MPRIS, clipboard e tray (15/11/2026).
- [x] Completar auditoria linha a linha dos serviços e scripts restantes (28/28 scripts Python ativos; helpers/setup privilegiados lidos).
- [x] Validar por leitura integral os comandos externos, endpoints, schemas, cache, timeouts e permissões de cada script.
- [x] Mapear eventos, sinais, timers e lifecycle dos 16 adapters QML adicionais; fichas em INTEGRATION_AUDIT.md.
- [ ] Confirmar dependências e comportamento degradado no hardware real.
- [ ] Revisar e congelar com Codeína o contrato tipado inicial de APP_API_CONTRACT.md; draft v0.1 criado, revisão pendente.

## Fase 1 — Auditoria e especificação

- [x] Aprofundar contratos/evidências estáticas de CompositorService, MonitorService, IdleService, SessionService, LockService e PolkitService; fixtures canônicas limitadas aos schemas confirmados em `docs/APP_API_MATRIX.md`.
- [x] Classificar as pendências dos seis contratos e documentar planos de runtime isolado, completude documental e bloqueios remanescentes em `docs/APP_API_REVIEW.md` (rodada 3; testes não executados).
- [x] Fechar a classificação documental simplificada do grupo anterior e especificar FontService, EmojiService, MachineService, VersionService, SunsetService e ApplicationsService em APP_API_CONTRACT/MATRIX/REVIEW (rodada 4); sem testes ou fixtures hipotéticas.
- [ ] Resolver pendências reais dos contratos da rodada 4: persistência/dados Unicode Emoji; outcomes detached e efeitos de `git fetch` em Version; locale/ordenação e concorrência de Applications; erros não publicados dos adapters. Ver classificação por categoria em APP_API_REVIEW.
- [x] Especificar estaticamente NiriService, SettingsService, AudioService, NetworkService, BluetoothService, BatteryService e BrightnessService com estados, comandos, consumidores, lifecycle, persistência/evidências e fixtures sintéticas quando o schema permite (rodada 5); sem testes de sistema.
- [ ] Fechar bloqueios da rodada 5: comportamento de JsonAdapter para Settings (corrupt/unknown keys) e formato de migração; decidir versionamento Rust fora do arquivo original; definir handle/lifetime PipeWire e transporte seguro do segredo Network; confirmar mapping UPower enum; validar lifecycle providers em runtime isolado antes de VERIFIED.
- [ ] Validar em runtime os estados de erro/recuperação dos seis serviços, transições ext-idle/PrepareForSleep, acquire/release ext-session-lock, PAM/Howdy e ciclo de prompt/registro/fallback Polkit.
- [ ] Resolver as lacunas de compatibilidade dos seis contratos antes de marcar seus schemas como fechados: resultados de subprocessos sem handlers, timeout/saída dos watchers, validade/correlação de flow Polkit, e mudanças live/hotplug de monitor.

- [x] Mapear imports de primeiro nível, `qmldir`, singletons tocados no startup e consumidores UI; superfícies/componentes estáticos indexados em UI_SURFACE_AUDIT.md.
- [x] Atualizar grafo de dependências entre serviços e anexar arestas das integrações externas; consumidores UI estáticos catalogados, runtime ainda pendente.
- [ ] Identificar serviços usados no startup versus carregados sob demanda.
- [x] Auditar LauncherService, ApplicationsService e scripts/applications.py; documentar frecency, modos e dependências.
- [x] Auditar ModuleService; documentar catálogo, capabilities, watchers e side effects de lazy loading.
- [x] Auditar os 22 módulos da barra, IslandRest/IslandSummary, OSD/Notification layers e os 16 block faces do Control Centre; contratos em UI_SURFACE_AUDIT.md.
- [x] Auditar DockService; documentar agrupamento, matching de janelas, pinning e geometria.
- [x] Auditar DesktopService/DeckService; documentar grid, colisões, widgets, notes decks e spectrum.
- [x] Auditar Widget.qml, Face registries, Inspector.qml, Picker.qml, Tray.qml e EdgeSpectrum/SpectrumBars; contratos em UI_SURFACE_AUDIT.md.
- [x] Auditar DockItem.qml e DockMenu.qml; validação layer-shell/magnificação/autohide em runtime segue na fase de verificação.
- [x] Auditar NotesService, TasksService e TimerService; documentar persistência, parser e estados.
- [x] Auditar VmService/vms.py, StatsService/stats.py e SystemService/system.py.
- [x] Auditar SessionService, IdleService, SunsetService, PolkitService e LockService.
- [x] Auditar adapters FaceService/OsdService/LyricsService e scripts/lyrics.py.
- [x] Auditar helpers face/avatar, policies polkit, PAM e setup system/face/camera no checkout; indisponibilidade runtime registrada (não instalado).
- [x] Ler integralmente os 9 serviços QML restantes (Compositor, Shortcut, Profile, Picker, Controls, Calendar, Cava, Games e Pet); total 61/61 serviços QML ativos auditados.
- [x] Atualizar PORTING_MATRIX, BACKEND_AUDIT, DEPENDENCY_MAP e SERVICE_GRAPH com as nove fichas finais.
- [x] Expandir APP_API_CONTRACT para Profiles, Calendar, Controls, Picker, Games, Pet e eventos de Cava; Shortcut permanece catálogo read-only como no original.
- [x] Ler integralmente ThemeService, WallpaperService, scripts/theme_manager.py e greeting.py; documentar providers, Matugen, awww/mpvpaper e temas externos.
- [x] Auditar os painéis da Dynamic Island, detalhes, Settings (14 seções), Launcher, Games (11 jogos), Pet, Profiles, Lock e Capture; contratos em UI_SURFACE_AUDIT.md.
- [x] Completar auditoria estática dos filhos de Dock, Desktop, Deck, Lock e Capture; o modo Capture Window segue BLOCKED por símbolo ausente; testes reais/screenshots ficam na fase 5.
- [x] Gerar UI_COMPONENT_CATALOG.md com inventário de 303 componentes QML, handlers e referências a serviços.
- [x] Ler integralmente Commands.qml e ~/.local/bin/velatura; registrar targets, payloads, erros e divergências em docs/IPC_AUDIT.md.
- [ ] Testar compatibilidade CLI em fixtures.
- [x] Auditar CaptureService/RecorderService e scripts/capture.py/record.py; executar somente probes read-only de ferramentas.
- [ ] Resolver `root.windowUnder()` no modo Window com reprodução/recuperação da implementação original antes de portar esse modo; captura/gravação/OCR e coordenadas continuam sem teste runtime.
- [x] Mapear os 28 scripts Python ativos, os 2 `.new` e `cava.conf`; 28/28 scripts lidos integralmente (ver SOURCE_INVENTORY.md).
- [x] Ler integralmente os 16 scripts Python restantes e mapear dependências/permissões por serviço em INTEGRATION_AUDIT.md.
- [x] Ler `SettingsService.qml`: schema, defaults, aliases, perfis, machine keys, FileView e migrações observadas.
- [ ] Validar round-trip, arquivo corrompido e migração em fixtures Rust.
- [ ] Definir escopo de islet e macos-qckshell.

## Fase 2 — Arquitetura e contratos

- [ ] Escolher modelo de UI (Relm4 ou GTK4 direto).
- [ ] Definir runtime assíncrono.
- [x] Definir contrato tipado inicial de snapshot/eventos/comandos/erros em APP_API_CONTRACT.md; adicionar extensão derivada das superfícies UI (v0.1-draft, revisão Codeína pendente).
- [ ] Definir abstração de compositor após fechar contrato do event stream Niri e validar protocolo real.
- [ ] Definir modelo de superfícies e multi-monitor após auditoria visual/funcional das superfícies e validação layer-shell no Niri.
- [ ] Definir persistência e migração de configurações.
- [x] Criar fixtures mock compartilhadas de snapshot e eventos para Codeína em docs/fixtures/; shapes são draft e requerem revisão.

## Fase 3 — Fundação

- [x] Criar crate Rust inicial `velatura-core` com Cargo.lock e módulo de domínio independente de GTK.
- [x] Implementar redutor puro do event stream Niri, preservando handlers, estado e sinais observáveis do NiriService.qml; sete testes iniciais cobrem fixture, handshake, foco/urgência, eventos de janela/workspace, JSON inválido e eventos desconhecidos.
- [x] Implementar supervisor do processo `niri msg --json event-stream`, incluindo stdout JSONL, stderr, exit code, parada explícita e reconexão somente para exit não-zero com atraso de 3000ms, conforme o original; testes com executável fake.
- [x] Implementar fila serial de `niri msg action` com argumentos separados, sem shell, preservando FIFO e reportando o resultado real do subprocesso sem receipt de aceite inventado.
- [x] Implementar `niri msg --json outputs`, normalização equivalente a monitors.py, primary/internal/description helpers, timeout, debounce de 120ms, coalescing, cancelamento e refresh acionado por WorkspacesChanged.
- [x] Implementar observer GDK4 opcional para `GListModel::items-changed` (hotplug) e `GdkMonitor::notify::geometry`; callbacks só solicitam refresh ao serviço Niri. Handlers são reconectados ao mudar a lista e desconectados no Drop.
- [ ] Logging e diagnóstico.
- [ ] Configuração e schema versionado.
- [x] Canal de eventos da sessão Niri: snapshots coerentes, sinais do compositor, diagnósticos, erros de protocolo, outputs e resultados de actions.
- [x] Ciclo de vida RAII da sessão Niri: inicialização coordenada, parada explícita e join dos workers.
- [x] `ApplicationRuntime` como dono do backend por vida útil: inicia sessão + observer, expõe eventos e encerra observer antes dos workers; integração ao `GtkApplication::activate/shutdown` permanece pendente até existir o app.
- [x] Adapter de sessão Niri (event stream + reducer + actions). Abstração genérica de compositor permanece pendente.
- [x] Adapter de outputs Niri com refresh inicial, WorkspacesChanged e refresh solicitado pela UI/GDK; teste físico de hotplug e geometria em multi-monitor permanece pendente.
- [x] Implementar o bootstrap GTK4 mínimo: binário, `GtkApplication`, lifecycle real, janela de teste e comando CLI de desenvolvimento; `cargo check` passou.
- [ ] Completar build/link do executável e validar a primeira abertura no Niri; a compilação foi interrompida ao atingir temperatura de CPU acima do limite operacional de 90 °C.
- [x] Criar primeira fundação visual GTK compartilhada a partir dos tokens classic, métricas, fontes e paletas auditadas; provider aplicado somente ao bootstrap temporário.
- [ ] Conectar tokens a SettingsSnapshot/ThemeService nativos e implementar seleção/persistência sem duplicar backend; bloqueios em VISUAL_THEME_AUDIT.md.
- [x] CLI de desenvolvimento com `run`, `help`, `version`, `stop` e `restart`; `ipc` permanece pendente e nunca faz fallback para `qs`. Instalador usa `~/.local/bin/velatura-native`, preservando o comando legado.
- [ ] Protótipo GTK layer-shell isolado.
- [x] Expor snapshot Niri/output atômico e revisionado por `NiriSession`/`ApplicationRuntime`; estado inicial vazio e canal de eventos de consumidor único.
- [x] Encaminhar eventos pelo `GtkEventBridge` ao main context sem polling; encerrar bridge depois do runtime no shutdown atual.
- [x] Corresponder GDK↔Niri por connector exato e fallback de geometria lógica única; atualizar inventário em hotplug/geometry e solicitar refresh coalescido.
- [ ] Simular hotplug físico, desconexão/reconexão real do Niri e alteração geométrica real em execução isolada.
- [ ] Iniciar implementação da barra GTK após disponibilizar à UI snapshot inicial e bridge de eventos Niri não bloqueante, com teardown vinculado ao `ApplicationRuntime`; `build_bootstrap_window(app, monitor_count, refresh)` deve manter a assinatura. Hoje só há refresh handle, contagem GDK e um `NiriSession` de consumo síncrono (`recv`), sem snapshot cacheado/stream GTK, matching GDK↔Niri nem ações públicas de domínio. Contrato-base e lacunas estão em `APP_API_CONTRACT.md`/`APP_API_REVIEW.md`; alinhar o bridge com backend antes de conectar UI.
- [ ] Confirmar `gtk4-layer-shell` e comportamento no Niri em configuração isolada antes de criar superfícies por output; não tratar `ApplicationWindow` como equivalente à layer surface.
- [ ] Portar a primeira fatia auditada da barra somente após o bridge: estados reais de output/foco/workspaces, geometria estável, máscara/input/foco e handover da ilha. Não desenhar dados ou ações de domínio com mocks como se fossem funcionais.

## Fase 4 — Portabilidade

- [ ] Barra e módulos.
- [ ] Dynamic Island e painéis.
- [ ] Control center e launcher.
- [ ] Serviços de sistema essenciais.
- [ ] Settings.
- [ ] Dock.
- [ ] Desktop widgets e deck.
- [ ] Lock screen.
- [ ] Capture.
- [ ] Integrações opcionais e jogos.

## Fase 5 — Validação

- [ ] Paridade visual por screenshot.
- [ ] Paridade funcional por checklist.
- [ ] Teste multi-monitor e hotplug físico (matching determinístico coberto por testes unitários; evento real pendente).
- [ ] Teste de reconexão Niri.
- [ ] Teste de ausência de serviços/dispositivos.
- [ ] Teste prolongado de estabilidade.
- [ ] Benchmark idle e interação.
- [ ] Verificar consumo térmico no MacBook.
- [ ] Revisar acessibilidade e navegação por teclado.

## Fase 6 — Entrega

- [ ] Empacotamento Arch/CachyOS.
- [ ] Documentação de instalação e configuração.
- [ ] Modo de teste isolado.
- [ ] Plano de migração com backup.
- [ ] Aprovação explícita do usuário.
- [ ] Somente após aprovação: avaliar integração com Niri/autostart.
