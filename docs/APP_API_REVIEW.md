# Revisão técnica do contrato Backend ↔ Frontend

**Data:** 15/11/2026<br>
**Escopo:** revisão documental de `APP_API_CONTRACT.md` contra BACKEND_AUDIT, INTEGRATION_AUDIT, DEPENDENCY_MAP, SERVICE_GRAPH, IPC_AUDIT, UI_SURFACE_AUDIT, UI_COMPONENT_CATALOG, PORTING_MATRIX e fontes originais citadas.<br>
**Resultado:** contrato **NÃO PRONTO** como base de implementação 1:1. Serve como inventário preliminar e guia de revisão, não como API congelável.

Nenhum Rust foi implementado nesta revisão. A instalação original e o Niri não foram alterados. O código em `/home/px/.config/quickshell` foi consultado em modo somente leitura.

## Parecer executivo

Os 61 singletons QML ativos aparecem nominalmente em `ServiceId`, portanto existe cobertura nominal dos serviços. Isso não significa contrato completo: vários serviços não têm estado/comandos/eventos especificados; muitos comandos aparecem apenas em prosa, com payloads incompletos; e `DomainSnapshot` não cobre vários estados que as superfícies consomem. A UI catalogada contém 303 componentes e múltiplas superfícies; a API só modela uma parte delas. `LegacyIpc` pode ser um adaptador de compatibilidade, mas não substitui uma API de domínio tipada e não resolve comportamentos de UI.

Além das lacunas, há contradições que impedem implementação segura sem decisão: atomicidade de `connect()` com lazy loading, expiração/ordenação de eventos, resultados de comandos e efeitos aceitos, política de privacidade dos títulos, persistência/migração desconhecida, representação de grants de arquivo e comportamentos legados divergentes. O draft deve continuar como draft.

## Problemas encontrados

Gravidade: **Bloqueador** impede iniciar a implementação do contrato afetado; **Alta** pode causar perda de semântica/paridade; **Média** requer decisão antes de congelar API ou fixtures.

| ID | Gravidade | Problema / evidência | Ação necessária |
|---|---|---|---|
| API-01 | Bloqueador | `CaptureOverlay.qml` chama `root.windowUnder(mouse.x, mouse.y)` sem definição no arquivo nem em busca global nos dois checkouts disponíveis: `/home/px/.config/quickshell` e cópia-fonte `/home/px/impasto-niri/home/.config/quickshell` (as duas cópias têm a mesma chamada sem definição). `CaptureService.qml` documenta que Niri IPC não fornece retângulos globais, declara `shapes: ["region", "screen"]` e rebaixa setting legado `window` para `region`. `scripts/capture.py` não seleciona janelas. Não foi localizada implementação/helper alternativo nesses checkouts. | Manter `Capture Window` BLOCKED. A lacuna de origem está delimitada: falta a definição de `windowUnder` e o mecanismo de retângulos/seleção, e não há implementação em CaptureService/capture.py. Reproduzir o caminho original em runtime para confirmar se é defeito inoperante ou localizar outra fonte/helper antes de especificar compatibilidade. Se confirmado inoperante, registrar decisão; não inventar picker ou retângulos. |
| API-02 | Bloqueador | Serviço coberto por ID não equivale a contrato: `DomainSnapshot` omite explicitamente estado de Compositor/Monitor, Idle, Session, Lock, Polkit, Font, Emoji, Machine, Version, Sunset/NightLight e Applications. O enum também não tem comandos dedicados para vários desses domínios. | Especificar snapshots, eventos, comandos, ciclo de vida, ausência de dependência e erros a partir dos respectivos adapters/consumidores antes da implementação. |
| API-03 | Bloqueador | O contrato afirma que todos os subestados têm shape tipado, mas os tipos concretos dos domínios não estão definidos. A própria seção 12 diz que campos necessários às faces “devem” ser preenchidos antes da implementação. Isso deixa UI e backend sem payload validável. | Completar schema de cada domínio e matriz superfície → provider → campo/evento/comando, cobrindo catálogo UI e Settings, estados vazios/erro/loading/disabled. |
| API-04 | Bloqueador | `connect()` promete snapshot + stream “sem janela de perda” e “fronteira atômica”, embora os serviços possam iniciar assincronamente e `snapshot()` seja apenas cache. Não há cursor/replay, sequência por serviço, overflow, reconexão, `SnapshotReady` ou tratamento de evento perdido especificados. | Definir handshake com revision/cursor, semântica inicial (incluindo Starting e carregamento), replay ou resnapshot obrigatório, detecção de gap, ordenação global/por domínio e política de backpressure/overflow. Não declarar atomicidade até existir mecanismo verificável. |
| API-05 | Alta | Lifecycle incompleto: `AppHandle` não declara `start`, `shutdown`, reload, encerramento ordenado, flush de persistência, reconexão ou sobrevivência de processos detached. Auditorias documentam startup eager/lazy misto, recorder que sobrevive reload, timer/session/lock, Cava linger e daemon de calendário/wallpaper. | Criar sequência explícita de startup/stop e ownership por serviço; documentar flush, cancelamento, child processes, detached recording, subscriptions obrigatórias e recuperação por exit 0/non-zero/timeout. Preservar divergências observadas até decisão separada. |
| API-06 | Alta | Semântica de comando contraditória: `dispatch` diz que receipt significa aceito, porém `CommandReceipt.state` já pode ser `Completed`; `CommandFinished` usa `Result<CommandResult, SafeError>`, mas `CommandOutcome` só cobre sucesso/cancelamento/parcial e não descreve falha estruturada; cancelamento pode não emitir terminal event; não há correlação de comandos do mesmo domínio nem idempotência. IPC legado frequentemente responde `asked` antes do efeito. | Definir estados terminais, erro final, cancelamento, idempotência, concorrência, timeout e significado de “efeito confirmado” por comando. Preservar externamente pedidos aceitos/void/texto cru do IPC onde aplicável. |
| API-07 | Alta | Cobertura de comandos é parcial e heterogênea. `ApplicationService` não tem comando próprio para lançar; `Font/Emoji/Machine/Version/Sunset/Idle/Session/Lock/Polkit/Osd/Module/Compositor` não têm contratos completos. Commands.qml tem targets/verbs e retorno legacy não representados individualmente; `LegacyIpc` opaco não especifica allowlist, coerção de argumentos, código de saída e formatos excepcionais. | Criar catálogo comando-a-comando ligado a `Commands.qml`, wrapper CLI e consumidores QML; marcar operações sem equivalente tipado. `LegacyIpc` deve referenciar esquema estruturado verificável e preservar stdout/JSON/void, timeout e exit codes. |
| API-08 | Alta | Campos de superfície incompletos/ambíguos: Niri `WindowSnapshot` inclui título necessário à Overview/Launcher, mas princípio de segurança proibia títulos; tray DBusMenu, notificações e auth precisam de IDs/actions com validade; launcher selection está como índice instável; Dock/Deck/Bar/Controls contêm apenas shapes parciais; capture não expõe `photo`, geometria, escala/transform, crop/selection nem destino; Lock precisa de autenticação e estados temporizados; ActivitySnapshot usa strings livres. | Especificar tipos com identidade estável e validade de ação, dados necessários a cada componente e limites de retenção. Remover strings/índices genéricos onde a auditoria mostra enums ou IDs. Não omitir dados visuais para contornar uma regra global de privacidade. |
| API-09 | Alta | Persistência não está especificada o suficiente para preservar formatos. Settings tem 108 chaves, migra `frosted→glass`, ground, `barShadow→dockShadow`, `barFullWidth/island→bezel`, preserva machine keys; unknown keys/types inválidos têm comportamento próprio. Paths reais são `XDG_STATE_HOME/quickshell`, Games/Pet usam JSON e ScreenTime TSV; ProfileService tem JSON v1/shipped profiles e exclui dados locais. O contrato menciona backup/consentimento, mas não define importação/migração, rollback, conflito de escrita externa ou formato de compatibilidade. | Documentar formatos, versões, defaults, ordem e idempotência de migrações, backup/restore, chaves unknown e tipo inválido, escrita atômica, reload externo/conflito e estratégia de diretório novo sem alterar a fonte original. Dar decisão explícita para migrar de `quickshell` para `velatura` e para referências a arquivos. |
| API-10 | Alta | O contrato propõe `FileGrant` opaco, mas não define capacidades suportadas (ler/escrever/listar), persistência após sessão, revogação, URI/path de wallpaper/avatar/capture, import/export e tratamento de arquivos removidos. `WidgetOverrides.picture: Option<FileGrant>` tampouco pode representar uma referência persistida durável. | Definir modelo de grant separado da referência persistente validada, integração com portal/file chooser e import/export; não persistir grant efêmero como caminho durable. |
| API-11 | Alta | Disponibilidade confunde domínio não iniciado, domínio sem entidade/dado, integração indisponível e erro. Contrato usa `Option` com comentário “não iniciado ou não existe no ambiente”; serviço pode estar `Starting` enquanto substate é None. Alguns serviços preservam last-good ao falhar, outros limpam/resetam. | Definir `LoadState`/freshness separadamente da disponibilidade do serviço e políticas por domínio: last-good, stale, vazio, erro, ausência de hardware/daemon/token/permissão. Incluir timestamps de atualização e origem quando visíveis no original. |
| API-12 | Alta | Os defaults/lifecycle de subscription não refletem todos os consumidores. Settings carrega/destrói página; ModuleService tem subscription própria; ScreenTime distingue `watchers` e `followers`; Pet XP depende de placement/lock/mood; serviços de lock/session/clipboard policy/recorder têm trabalho fora da superfície. | Declarar interesse por capacidade/consumer, consumidores obrigatórios do runtime e contagens separadas observadas. Não fazer de `ServiceId` um único contador genérico quando o original diferencia tipos de follower. |
| API-13 | Alta | Há divergências originais que não podem ser “normalizadas” no contrato: Niri event stream exit 0 não reconecta; erro só loga ação; CLI anuncia layouts que IPC rejeita; `brightness.step` não tem comando CLI; `capture window` falha; `capture region/window` omite destination; wallpapers retornam string cru; `bar/widgets/shell.reload` são void; `weather.py` tem bug daytime. | Fazer tabela compatibilidade: comportamento efetivo, divergência/bug e decisão de manter/corrigir. Não corrigir sem decisão documentada em DECISIONS.md e fixture para comportamento público. |
| API-14 | Média | `SettingsSnapshot.values` usa `BTreeMap<String, SettingValue>` embora exista catálogo de schema/tipos, e descrição de tipos desconhecidos diz “aditivo” mas não documenta extensão de enum, versões de eventos nem desserialização forward-compatible. `schema_version` de settings e snapshot podem ser confundidos. | Definir separadamente versão de protocolo, schema de settings, schema de dados e revision de estado; documentar unknown enum/field e serialização compatível. |
| API-15 | Média | `OutputSnapshot` não contém metadados auditados por `monitors.py` (mode, refresh, transform, VRR, make/model/serial) que Settings/Monitor UI e superfícies podem precisar. E `effective_primary` repete regra postulada sem campos carregados completos. | Fazer matriz de campos contra MonitorService/Settings Monitor UI; somente após confirmar consumidores decidir se algum dado pode ser omitido. |
| API-16 | Média | Fixtures exercitam um recorte feliz simples, não cobrem handshake inicial, gap/reconnect, persistência/migração, input inválido, permission/missing dependency, comandos long-running, cancelamento, last-good/stale, multi-monitor handover, ações efêmeras vencidas, `windowUnder` bloqueado, tray menu, captura cancelada ou reload com gravação ativa. A fixture inclui títulos de exemplo mas não documenta sua classificação de privacidade. | Adicionar fixtures/casos descritos na seção “Fixtures necessários”; manter sintéticos, sem copiar estado real ou segredos. |

## Cobertura de serviços, superfícies e ciclo de vida

### Serviços

`ServiceId` contém nominalmente todos os 61 nomes listados no inventário de singletons ativos: cobertura nominal **61/61**, condicionada a uma checagem automática de unicidade/plenitude na próxima revisão do catálogo. Ainda faltam, em grau variável, tipos públicos de estado, comandos, eventos e lifecycle. Os grupos claramente sem contrato próprio são: CompositorService, FontService, EmojiService, MachineService, VersionService, SunsetService, IdleService, SessionService, LockService, PolkitService e ApplicationsService. Serviços com estados não representados integralmente incluem Monitor, Niri, Module, Osd, Profile, Settings, Wallpaper, Capture, Recorder, Controls, Calendar/dcal, tray, camera/face e usage APIs.

O contrato precisa incluir um apêndice de rastreabilidade com, para cada um dos 61: fonte QML/scripts, superfícies/consumidores, bootstrap (boot/lazy/event-driven), dependências e permissões, estado inicial e completo, eventos, comandos/efeitos, persistência, falhas/retry/shutdown, campos sensíveis e fixtures. Serviço listado em `services` sem isso não está “representado” em sentido suficiente à portabilidade.

### Superfícies

UI_SURFACE_AUDIT e UI_COMPONENT_CATALOG contêm 303 componentes ativos, além das superfícies de shell. Há dados candidate para Bar, Launcher, Desktop, Dock e Deck, mas ainda faltam tipos completos e comandos validáveis. A enumeração atual não demonstra cobertura de componentes: contrato de um host não significa cobertura de painéis filhos, Settings tabs, widgets, faces, menus e estados transitórios. Exigir matriz superfície/componente → campos do snapshot → eventos → comandos → estados vazios/loading/error/disabled → persistência.

Além das omissões já listadas, os exemplos de UI exigem atenção específica para: estado de foco/seleção e ações de Overview; identidade e geometria de janela/workspace; quatro estilos e 22 módulos duplicáveis da barra; 26 rotas de painel e detalhes; actions e validade de notification/tray menu; paginação/edição/collision de Controls; seleção do Launcher; grid/face/catalog de Desktop; edges/ordenação/transferência de Deck; Dock grouping/autohide/magnification; Lock/PAM/face; Capture/Recorder; games e animações/ações de Pet; subscriptions de Spectrum; Settings tabs/tabs internas e flows de arquivo.

### Inicialização, atualização, parada e erro

BACKEND_AUDIT documenta um grafo misto: alguns serviços tocados no `Component.onCompleted`, vários serviços construídos por referência/consumidor, outros lazy, e serviços com timer/follower condicionado. A proposta de `connect(ServiceId...)` não modela dependências compartilhadas, subtipos de subscriber, efeito de erro ao iniciar, retry/backoff por domínio nem o estado após última lease. Ainda precisa declarar:

1. Estado inicial e ordering antes/depois do primeiro snapshot.
2. Serviço com dependência indisponível e dependentes que continuam utilizáveis.
3. Mudanças externas de settings/arquivos e conflito com gravação local.
4. Queda de subprocesso/D-Bus/IPC com exit 0, exit != 0, timeout e payload inválido.
5. Retry imediato/periódico/backoff e manutenção de último valor bom.
6. Drop de uma ou várias leases, trabalho obrigatório de background, close de panel e shutdown do app.
7. Processo externo que sobrevive ao shell (Recorder), propriedade/recuperação segura e arquivo runtime stale.
8. Flush/reload durante transição e ordenação de updates de múltiplas fontes.

## Persistência e formatos originais a preservar

Antes de definir migração, preencher uma tabela por store contendo caminho XDG, formato/schema, keys/fields, dados de máquina/usuário, defaults, versões antigas, unknown fields, lock/concurrency, writer debounce/atomicidade, reload externo, backups e política de recuperação de corrupto. Fontes já confirmadas incluem `quickshell/settings.json`, `state/profiles.json`, perfis exportados JSON v1, `games.json`, `pet.json`, `emoji-recent.json`, cache local de Claude usage, TSV diário de ScreenTime, e `XDG_RUNTIME_DIR/velatura-recording.json`. A lista não é substituta da auditoria de todos os stores.

Não converter caminho de arquivos locais em grant UI efêmero na persistência; não descartar keys que o schema Rust desconhece. Migração deve ser executada isoladamente com backup verificável, round-trip e rollback, mantendo o original read-only. A decisão sobre diretório de estado de destino ainda está aberta.

## Fixtures necessários antes de congelar

Manter `app_snapshot.mock.json` e `app_events.mock.jsonl` como exemplo parcial, adicionando arquivos/casos nomeados para:

- `startup_lazy_services`: snapshot de serviços `Starting`/`NotConfigured`/`Unavailable`, depois estado inicial carregado e ordem de revisões.
- `stream_gap_reconnect`: revisão perdida, overflow, reconexão, resnapshot/cursor e evento duplicado.
- `command_lifecycle`: invalid/immediate rejection, queued/running/progress, accepted-but-not-completed legacy operation, success, partial failure, failure, cancel e timeout.
- `service_failures`: daemon/device/token/portal/permission ausente, malformed response, retryable/nonretryable e last-good stale.
- `multi_output_handover`: focus output muda enquanto painel aberto, output live removido, fallback e output virtual.
- `settings_migration`: cada migração comprovada, desconhecidas, tipos inválidos, arquivo ausente/corrompido, escrita externa e machine keys.
- `file_grants`: leitura/escrita negada, grant revogado, arquivo removido e referência persistente revalidada.
- `transient_actions`: notification encerrada, DBusMenu node removido, launcher result expirado e tentativa de ação com ID antigo.
- `capture_recording`: screen/region photo/video, coordenadas/escala, destinos disponíveis, cancel, tool missing, recorder fallback e reload com take ativo. Window deve aparecer explicitamente como blocked/unsupported de acordo com decisão do original, não como fluxo funcional.
- `tray_menu_controls_desktop_deck_dock`: snapshots e ações mínimas de cada estado interativo para provar shape e identidades.
- `privacy_domains`: título transitório necessário à UI versus classe persistida por ScreenTime; clipboard sensível nunca incluído em fixture.

Fixtures não provam comportamento. Cada uma precisa apontar para uma evidência de código/decisão e teste de compatibilidade planejado.

## Requisitos que impedem início da portabilidade 1:1

1. Fechar o comportamento/defeito original de Capture Window sem inventar substituto (API-01).
2. Especificar todos os estados, eventos e comandos de 61 serviços e rastreá-los a componentes/consumidores (API-02, 03, 07).
3. Fechar handshake, revision, replay/gap, backpressure e lifecycle startup/shutdown/recovery (API-04, 05, 11, 12).
4. Fechar semântica terminal de comandos e compatibilidade exata de IPC/CLI, inclusive divergências observadas (API-06, 13).
5. Fechar campos de todos os componentes/superfícies e política de dados sensíveis por domínio (API-08).
6. Aprovar formato e processo de migração/backup/rollback para todos os stores sem escrita no original (API-09, 10, 14).
7. Criar fixtures de fronteira e compatibilidade para os casos listados; reconciliar JSON com tipos Rust pretendidos (API-16).
8. Completar rastreabilidade de outputs e estados multi-monitor, incluindo metadados realmente consumidos (API-15).

As decisões de runtime GTK/GLib, processo async/IPC e event transport também devem ser registradas em `DECISIONS.md`/`ARCHITECTURE.md` quando os contratos semânticos acima estiverem fechados. A revisão não escolhe essas tecnologias.

## Alterações documentais realizadas nesta revisão

## Fechamento incremental de contratos — rodada 1

Escopo concluído documentalmente: CompositorService, MonitorService, IdleService, SessionService, LockService e PolkitService. `APP_API_CONTRACT.md` recebeu schemas draft, estado observado, transições, comandos/efeitos e lacunas; `APP_API_MATRIX.md` rastreia serviço → consumidor → estado → atualização/evento → comando → evidência. Essa rodada reduz as lacunas de API-02/API-03/API-07, mas **não fecha nenhum desses itens globalmente**: restam outros serviços sem schema e o catálogo completo de `Commands.qml`/CLI ainda não foi transcrito.

- API-02: parcialmente atendido para 6 serviços; continuam sem contrato próprio os grupos restantes listados na auditoria.
- API-03: parcialmente atendido para esses seis serviços e consumidores encontrados; Settings Session não consome SessionService diretamente (configura conta/lock/idle), e os contratos daqueles serviços e seus campos ainda precisam de fichas próprias.
- API-07: adicionados os comandos internos Session/Lock/Polkit e o target legado `lock.lock`; a tabela completa de targets/verbs, coerções, exit codes e formatos permanece pendente.
- Não foi feita decisão de produto ou equivalência runtime. Shapes de Idle e Polkit, tratamento de prompts, eventos Rust e exposição CLI continuam draft.
- Fixtures sugeridas nesta rodada estão listadas na matriz; nenhuma foi declarada compatível nem adicionada como fixture canônica porque ainda falta fechar os tipos de lifecycle/erro e validar versões Niri.

## Fechamento incremental — rodada 2 (seis contratos aprofundados)

Revisão documental estática concluída para CompositorService, MonitorService, IdleService, SessionService, LockService e PolkitService. A especificação em `APP_API_CONTRACT.md` agora distingue comportamento literal do adapter, payload necessário à UI, comando/receipt, efeitos sem confirmação, falhas que o QML realmente registra e lacunas que dependem de runtime. `APP_API_MATRIX.md` tem linha por consumidor e evidência. Foram adicionadas fixtures apenas para o sucesso normalizado de Compositor/Monitor e catálogo literal de Session; casos de erro continuam não-canônicos.

- **Documentalmente fechado (escopo limitado):** catálogo de ações Session e diferenças entre consumidores; receipt imediato `lock.lock`; conversão/defaults/last-good/coalescing de Monitor; derivação primary/internal; regras visíveis de lock/password/face/capture; campos que AuthPanel lê e guards de Submit/Cancel; semântica observável de start/refresh/handlers existentes.
- **Ainda não fechado para 1:1:** estados iniciais reais dos IdleMonitor; outcomes e supervisão de Process (Compositor, Session e capture); protocolo/hotplug e ordenação de monitor; regras de PAM multi-prompt e erro facial; resultados/validade/concorrência de Polkit flow; desligamento e recuperação de serviços/processos.
- **Runtime obrigatório:** bins/exit code/stdout/stderr e processo ocupado; ext-idle-notify com mudanças de settings e inhibitors; logind PrepareForSleep e delay inhibitor; ext-session-lock em hotplug/crash; PAM/Howdy presentes/ausentes/erro/cancel; polkit request, refusal, cancel, flow replacement, registration e fallback.
- Fixtures: `compositor_service.loaded.json`, `monitor_service.loaded.json`, `session_actions.json`. São dados sintéticos/normalizados cuja forma está ancorada no código, não testes nem prova de runtime. Não foi criada fixture Idle/Lock/Polkit.
- API-02/03/07 seguem **parciais no grupo e bloqueadores globais**; os outros serviços seguem fora desta etapa e o catálogo integral IPC/CLI continua pendente.
- Parecer para estes seis: documentação melhora significativamente e não mascara lacunas, mas **não está pronta para implementar sem adivinhação**, portanto os seis não são contratos fechados nem estão prontos para portabilidade 1:1.

- `APP_API_CONTRACT.md`: corrigida a regra de privacidade para permitir títulos transitórios necessários à UI sem persistí-los em ScreenTime; removida promessa não sustentada de atomicidade para serviços lazy; removida cardinalidade absoluta da ilha ainda sem todos os casos de output; explicitadas omissões de domínios no snapshot.
- Este parecer registra as lacunas e o checklist de bloqueadores. Os exemplos existentes permanecem sintéticos e parciais; nenhum caso novo é declarado coberto.


## Rodada 3 — fechamento da classificação dos seis contratos

Escopo: CompositorService, MonitorService, IdleService, SessionService, LockService e PolkitService. Evidências principais: QML e scripts originais citados em `APP_API_CONTRACT.md`; metadata QML instalada de Quickshell (`quickshell-io.qmltypes`, `quickshell-wayland-idle-notify.qmltypes`, `quickshell-service-pam.qmltypes`, `quickshell-service-polkit.qmltypes`, `quickshell-wayland.qmltypes`). `pacman -Q` identificou `niri-git 26.04.r276.g5935130-1` e `quickshell 0.3.2-1.1`. Metadata confirma tipos/sinais, mas não timing de integração. Não foram executados testes no sistema.

Categorias: **1** investigação adicional do original; **2** validação em runtime; **3** limite do comportamento observável original; **4** decisão arquitetural sem evidência para escolher automaticamente.

| Pendência / categoria | Comportamento conhecido e evidência | O que falta observar ou decidir | Impacto na portabilidade 1:1 |
|---|---|---|---|
| C-01 **2** | `niri --version`; stdout define versão/loaded, stderr não vazio define lastError; Process exit/error não tem handlers (`CompositorService.qml:16-40`; Process metadata). | Confirmar resultados de binário ausente, exit/status, stdout/stderr vazios e chamadas `load()` sobrepostas. | Não converter `loaded` em confirmação de exit 0 nem inventar retry. |
| C-02 **3** | Sem polling, retry, cancelamento ou shutdown; sucesso não limpa lastError (mesma evidência). | Não há mais estado de recuperação exposto no original. Qualquer status extra deve ser tratado como extensão. | Evita inventar lifecycle que altera a superfície observável. |
| M-01 **1 — resolvida** | Gatilhos/debounce de 120ms e comentário de Niri 26.04 (`MonitorService.qml:95-128`); versão instalada confirma Niri 26.04. | Versão original atual fica ancorada; versões adicionais ficam em M-04. | Schema do adapter pode ser avaliado contra a origem identificada. |
| M-02 **2** | Refresh indireto e coalescing de leitura (`MonitorService.qml:17-58,97-128`; `monitors.py`). | Observar ordem de attach/detach/geometry/focus, bursts e telas removidas. | Corrida pode trocar primary ou deixar consumidor com screen obsoleta. |
| M-03 **2** | stdout vazio é ignorado; JSON válido troca lista; falha mantém last-good; stderr/exit deixam lastError (`MonitorService.qml:19-49`). | Confirmar callbacks reais para saída vazia, JSON inválido, falha de spawn e exit 0/!=0. | Não confundir lista vazia válida com ausência de resposta. |
| M-04 **4** | Helper segue o formato Niri 26.04 e não versiona adapter. | Decidir se alvo é só a versão original ou uma faixa de Niri. | Sem escopo aprovado não declarar compatibilidade com versões diferentes. |
| I-01 **2** | Três IdleMonitor; timeout ×60, enabled iff >0, inhibitors; metadata confirma bool/sinal `isIdle` (`IdleService.qml:16-64`; IdleNotify metadata). | Observar valor inicial, timeout alterado enquanto idle, inhibitor e reconexão. | Valores/eventos iniciais e reconfiguração não podem ser presumidos. |
| I-02 **3** | Sem estado/callback de erro; efeitos chamam Lock/Niri/Session sem receipt (`IdleService.qml:29-64`). | Código original não publica falha ou retry de protocolo/DPMS. | Não há estado terminal equivalente a expor na UI. |
| S-01 **3** | `run` roteia ações e não retorna status; Process sem handlers (`SessionService.qml:30-60,108-141`; Process metadata). | Resultado real de systemctl/Niri não é estado publicado pelo serviço. | Não emitir success/failure de operação sem nova evidência. |
| S-02 **2** | Runner único recebe novo command sem guard de ocupado (`SessionService.qml:30,138-141`). | Observar efeito de chamadas sobrepostas e Process error/exit. | Reentrância pode diferir caso seja normalizada sem evidência. |
| S-03 **3** | sleepWatch/sleepHold iniciam sem supervisão/handlers (`SessionService.qml:47-106`). | O original não expressa recuperação de watcher/inhibitor. Runtime pode revelar consequência, não uma política de serviço ausente. | Não presumir restart ou proteção após falha. |
| S-04 **2** | Suspend espera `secure`; timer de desistência 5s; PrepareForSleep controla inhibitor (`SessionService.qml:32-60,94-123`). | Observar ordem/timing com lock secure antes/depois dos 5s e sinal logind. | Ordem lock-before-suspend é requisito funcional crítico. |
| L-01 **2** | `locked` solicita WlSessionLock; `secure` vem de secureStateChanged; surface por output (`LockScreen.qml:17-46`). | Validar aquisição/release multi-output, hotplug e crash do shell. | Sem isso não há confirmação de lock seguro em todas as telas. |
| L-02 **2** | Captura termina antes de `locked`; erro de exit mantém lock solicitado e `shotReady=false`; spawn/hang pode diferir (`LockService.qml:116-196`). | Observar erro de captura versus `Process.errorOccurred`/processo sem saída. | Falha de screenshot não pode ser tratada como autorização para deixar a sessão aberta. |
| L-03 **2** | PamContext `login`; um response string; Success/Failed/MaxTries/Error no QML e metadata (`LockService.qml:209-269`; PAM metadata). | Confirmar prompts reais, demora, challenge adicional, abort e retry em PAM isolado. | Conversa mais rica pode não caber no campo/password flow atual. |
| L-04 **2** | PAM `velatura-face`; `onPamMessage` liga scanning; Completed limpa; `onError` só registra warning (`LockService.qml:271-350`). | Observar erros, ausência de mensagem, abort/release e scanning preso. | Afeta retry/feedback e prontidão da face. |
| L-05 **4** | `typed` é compartilhado entre outputs e limpo nos fluxos de submit/rest/unlock (`LockService.qml:48-73,199-240`; LockAccount). | Decidir fronteira Rust para `SubmitSecret` interno, sem exposição/persistência/log. | Uma fronteira errada expõe credencial ou duplica seu ciclo de vida. |
| P-01 **1** | Quickshell metadata 0.3.2-1.1 confirma propriedades e flags/sinais do AuthFlow; serviço expõe `flow`; UI lê subset. (`quickshell-service-polkit.qmltypes`; `PolkitService.qml:28-80`; `AuthPanel.qml`). | Shape de `Identity` e semântica do cookie não constam na metadata encontrada. Nenhuma busca adicional será feita nesta rodada. | Não serializar runtime objects nem declarar schema de identities por suposição. |
| P-02 **2** | `requested/settled` sem payload; flow atual tem cookie e `flowChanged` na metadata. | Observar validade/lifetime, troca do flow e requests concorrentes. | Submit/cancel precisa atingir apenas flow atualmente válido. |
| P-03 **2** | Flow metadata tem `isCompleted/isSuccessful/isCancelled/failed` e sinais terminais; serviço só conecta failure e fecha por `isActive=false`. | Observar ordem dos flags/sinais frente a settled em sucesso, falha e cancelamento. | `settled` não substitui resultado; typed contract deve reter os flags originais confirmados. |
| P-04 **2** | Retry de registro a cada 6s e fallback detached após 3 tentativas, sem observar seu resultado (`PolkitService.qml:41-99`). | Confirmar registro, fallback e destino do pedido em falhas controladas. | Não declarar recuperação efetiva só porque o processo fallback foi solicitado. |
| A-01 **4** | QML expõe propriedades/sinais heterogêneos; não existe protocolo Rust original. | Aprovar envelope/event granularity, receipts e ownership/shutdown sem aumentar resultados semânticos. | Sem decisão podem surgir interfaces incompatíveis entre UI/backend. |

### Completude documental

“Completo” aqui significa que o contrato descreve toda a superfície observável encontrada, inclusive ausência de retorno; não significa runtime verificado nem autorização para implementar.

| Serviço | Parecer |
|---|---|
| CompositorService | **Documentalmente completo** para version/configPath/loaded/lastError e a única operação de leitura. Falhas sem callback permanecem limite documentado; runtime pendente. |
| MonitorService | **Documentalmente completo** para o helper original e seu snapshot/refresh; runtime pendente. Suporte além de Niri 26.04 depende de M-04. |
| SessionService | **Documentalmente completo** para catálogo, comandos de pedido e ausência de resultado terminal. Integração logind/Process pendente em runtime. |
| IdleService | **Incompleto:** estado inicial e comportamento de reconfiguração do IdleMonitor ainda não confirmados. |
| LockService | **Incompleto:** segurança multi-output, captura em falha e conversa PAM/face exigem runtime. |
| PolkitService | **Incompleto:** forma de Identity não foi encontrada e ordem/lifetime do flow exige runtime. |

## Rodada 4 — contratos Font, Emoji, Machine, Version, Sunset e Applications

Fechamento documental do próximo grupo pendente usando a auditoria estática já existente em `BACKEND_AUDIT.md`, `INTEGRATION_AUDIT.md`, `UI_SURFACE_AUDIT.md` e `UI_COMPONENT_CATALOG.md`, mais leitura das fontes citadas nos contratos. Os seis contratos anteriores não foram re-investigados nesta rodada. Nenhum teste de sistema/runtime foi executado.

### Resultado por serviço

| Serviço | Classificação | Pendência real e categoria | Impacto 1:1 |
|---|---|---|---|
| FontService | **Documentalmente completo** para dados e transições observáveis | Erro de spawn/timeout e stdout vazio não são publicados (**limitação do observável original**, 3); dependência/versionamento de fontconfig varia no ambiente (**runtime**, 2) | Reproduzir lista vazia em falha do helper e `loaded` como o original; não inventar warning/status de erro na UI |
| EmojiService | **Incompleto**; schema de catálogo, filtros, tom, grupos e persistência principal rastreados | Conteúdo/versão dos arquivos Unicode/CLDR e disponibilidade de variantes (**runtime**, 2); recovery/formato de JSON recente ainda precisa ser lido nas referências já identificadas (**investigação de código**, 1); `fresh=true` pode coexistir com carga vazia (**limitação original**, 3) | Catálogo, ordem, locale, recentes e glyph base precisam coincidir; erro não pode ser traduzido em “catálogo carregado com sucesso” sem equivalência |
| MachineService | **Documentalmente completo** para a saída do helper e mapeamento QML | Fontes do host e ferramentas instaladas produzem valores diferentes (**runtime/dado local**, 2); falha do Process/parse não tem estado de erro público (**limitação**, 3) | Preservar null do script e strings vazias criadas pelo adapter; não afirmar resultado terminal adicional |
| VersionService | **Documentalmente completo** para o comportamento observável, incluindo ausência de receipts das ações detached | Efeito real de update/change files e estados offline dependem do checkout/rede (**runtime**, 2); como API Rust apresentará essas operações sem ampliar resultado semântico é uma **decisão arquitetural**, 4 | Update/list/change não podem ser apresentados como concluídos sem confirmação; `Check` altera refs locais como original |
| SunsetService | **Documentalmente completo** para estados e efeitos que o adapter publica | Disponibilidade e saída do binário instalado variam (**runtime**, 2); spawn/exit sem stdout detalhado não vira estado causal (**limitação**, 3) | `on` é running do processo; não substituir por setting `nightLight`; falha observada desativa setting e solicita OSD |
| ApplicationsService | **Documentalmente completo** para parser, precedência, consumidores e ausência de resultado de launch | Conteúdo do índice depende dos desktop entries instalados (**runtime/dado local**, 2); locale keys não são consultadas pelo parser e empate mantém ordem do `os.listdir` (**comportamento original conhecido**, 3); refresh concorrente é permitido e launch é detached sem receipt (**limitação observável**, 3) | Preservar payload literal/precedência; não acrescentar suporte locale/Exec nem status de launch por conveniência |

### Fechamento e requisitos para avançar

- **Documentalmente completos neste grupo:** FontService, MachineService, VersionService, SunsetService e ApplicationsService, somente para a superfície que o original expõe. Não estão runtime-verificados; Version e Applications documentam explicitamente ausência de receipt terminal.
- **Documentalmente incompleto:** EmojiService, pela recuperação/formato do estado persistido ainda não consolidado. Dataset instalado exige validação para reproduzir o conteúdo, mas não muda o schema documentado.
- **Fixtures:** adicionadas somente para payloads estáveis de `fonts.py`, `machine.py` e `applications.py` (`docs/fixtures/fonts.index.json`, `machine.report.json`, `applications.index.json`), todos sintéticos. Sem fixture de lifecycle/erro. Emoji, Version e Sunset ficam sem fixtures porque os casos completos ainda presumiriam estado persistido/terminal não definido.
- **Bloqueios de implementação:** (1) fechar, por fonte, recuperação e shape persistido de Emoji; (2) decidir API interna para operações detached de Version sem inventar receipt; (3) preservar exatamente stdout/erro ausentes nos serviços em que o original não publica falhas; (4) revisar arquitetura global de eventos/estados iniciais para refletir a falta de erros públicos, sem fabricar resultados.
- Esta rodada não inicia implementação Rust e não muda o status global do contrato: `APP_API_CONTRACT.md` segue draft e não está pronto como base completa de portabilidade 1:1.

### Validação em runtime pendente

Nenhum teste foi executado. Quando autorizado e disponível um ambiente isolado já preparado, validar somente: C-01; M-02/M-03; I-01; S-02/S-04; L-01–L-04; P-02–P-04. Usar stubs para chamadas perigosas e nunca operar lock, suspensão, PAM ou Polkit na sessão real. Registrar divergências sem normalizar o original. C-02, I-02, S-01/S-03 são limites observáveis, não itens que runtime possa converter em resultados originais.

### Bloqueios restantes dos seis

1. Decisões arquiteturais: faixa Niri (M-04), envelopes/receipts/ownership (A-01) e fronteira do segredo Lock (L-05).
2. Investigação de código pendente: API/shape de Identity e semântica do cookie Polkit (P-01); não será aprofundada nesta rodada.
3. Validação em runtime: C-01, M-02/M-03, I-01, S-02/S-04, L-01–L-04, P-02–P-04; nenhum teste realizado.
4. Contratos incompletos: IdleService, LockService e PolkitService. CompositorService, MonitorService e SessionService estão completos somente documentalmente.

## Rodada 5 — Niri, Settings, Audio, Network, Bluetooth, Battery e Brightness

Escopo limitado aos sete serviços solicitados. Partida das fichas estáticas já existentes em `BACKEND_AUDIT.md` / `INTEGRATION_AUDIT.md`, cruzadas com adapters e componentes originais read-only. Os grupos anteriores não foram reabertos. Não houve testes em runtime nem alteração no Quickshell/Niri.

Categorias de pendência: **1** investigável em código/documentação do original; **2** depende de runtime; **3** limite já observável no original; **4** decisão arquitetural sem evidência para escolha automática.

| Serviço / classificação | Comportamento original conhecido e evidência | O que falta confirmar/decidir | Impacto para 1:1 |
|---|---|---|---|
| NiriService — **documentalmente completo para o adapter atual** | Parser do event stream, mutações por cada evento, payload genérico forwarded, ação serial e reconnect só em exit!=0 estão descritos em `APP_API_CONTRACT.md` §12.3 e `services/NiriService.qml`. | Compatibilidade do payload com versões Niri além da instalada e se exit 0 sem reconnect persiste no ambiente (**2**); o original não publica receipt de ação nem tenta reconnect no exit 0 (**3**). | Não reconsultar por polling, não “corrigir” reconnect exit 0 nem converter falha de ação em resultado inexistente. Preserve JSON bruto de evento não projetado. |
| SettingsService — **incompleto** | Store de 108 chaves e defaults, 107 aliases, machine/profile keys, FileView, reload, coalescing, first-run e quatro migrações são citados/documentados em §12.3 e `services/SettingsService.qml`; fixture sintética de defaults extraída do Store. | `JsonAdapter` em arquivo ausente/corrompido e round-trip de chaves JSON desconhecidas (**1/2**); contrato global `SettingsSnapshot.schema_version` não tem correspondente no JSON original, decidir se permanece metadado fora do arquivo Rust (**4**); tipo coerção de `set` não é validado pelo próprio método (**1/2**). | Mudar path, perda/preservação de chaves, defaults, migrations ou inclusão de version field pode impedir leitura/migração 1:1. Fixture defaults não prova comportamento do FileView. |
| AudioService — **incompleto para comando por objeto** | Estado PipeWire reativo, filtros/ordenação dos sinks/streams/inputs/captures, percent conversion, selectors, naming/icon derivation e ausência de receipt estão em §12.3 e `services/AudioService.qml`. | Handle de node/stream e sua validade quando PipeWire remove/troca objeto (**4**); troca de default e bindings após reconnect (**2**). | Frontend precisa escolher e editar streams sem expor objeto Quickshell; stale handles não podem agir sobre outro node. |
| NetworkService — **documentalmente completo para a superfície adapter/helper** | D-Bus/NetworkManager reativo, probe de rádio nmcli, debounce 1200ms, list merge/fields, scan e resultado `connectionFinished` estão em §12.3, `NetworkService.qml` e `scripts/network.py`. Senha original é argv. | Canal seguro para credencial sem argv exposto (**4**); NetworkManager ausente, locale do nmcli e sucesso/erro de associação real (**2**); efeitos de scan/rescan são fire-and-forget no helper (**3**). | Não persistir senha nem expô-la em process listing; preservar callback de connect e ausência para disconnect/forget. |
| BluetoothService — **documentalmente completo para ações expostas** | Filtros, duas listas, ordenação, summary, toggle de conexão (não pairing), adapter e discovery no lifecycle do detalhe são documentados em §12.3 e `BluetoothService.qml`/`BluetoothDetail.qml`. | Atualização de adapter/device desaparecendo durante conexão e handle stale (**2**); BlueZ permission errors não são surfaced pelo adapter (**3**). | Não adicionar Pair/Remove/PIN agent nem transformar ausência de erro em sucesso; fechar dispositivo por endereço sem objeto runtime. |
| BatteryService — **documentalmente completo para a projeção QML** | Defaults indisponíveis, percent, thresholds, strings de estimate/state, watts, energy, health, icons e reatividade UPower estão em §12.3 e `BatteryService.qml`. | Associação dos discriminants numéricos de `UPowerDeviceState` e semantics de readiness/hot-swap no binding usado (**1/2**); não foi criado fixture com enum presumido. | Converter `state` errado pode mudar charging/full e OSD; não inferir bateria presente de percent zero. |
| BrightnessService — **documentalmente completo para a superfície adapter** | Sysfs e DDC providers, screen identity/name, focused/first fallback, clamp, step, `adjusted`, hotplug debounce, write coalescing e read-after-failed-DDC-write estão em §12.3 e `BrightnessService.qml`. | Saídas reais de hardware/permissões e hotplug/reply timing (**2**); backlight writer failure não tem estado de erro público (**3**). | Sem valor sintético de sucesso/falha além do adapter; não autoelevar privilégios. Reproduzir ausência e último nível conforme fornecedor. |

### Fechamento documental desta rodada

- **Documentalmente completos para a superfície observável:** NiriService, NetworkService, BluetoothService, BatteryService e BrightnessService. “Completo” não significa runtime verificado nem contrato global aprovado.
- **Incompletos:** SettingsService por comportamento de load/unknown keys e falta de decisão sobre schema envelope; AudioService por identidade/lifetime de objetos de áudio nos comandos.
- **Fixtures:** adicionadas apenas para payloads definidos ou cenários sem hardware: stream Niri, defaults e input parcial de Settings, estado Audio indisponível, saída literal de `network.py list`, Bluetooth e Brightness sintéticos. Sem Battery fixture até mapear estado enum. Nenhum fixture representa máquina real.
- **Bloqueios que continuam:** (1) esclarecer erro/corrupt/unknown-key roundtrip do Settings JsonAdapter antes de definir migração; (2) decidir schema_version do arquivo Rust sem o inserir no JSON original; (3) definir handle/lifetime de node PipeWire; (4) escolher transporte secreto de Connect que não exponha argv; (5) confirmar mapping UPower state enum antes de publicar discriminants; (6) preservar peculiaridade Niri reconnect apenas em exit não-zero e ausência de receipt.
- Mudanças sugeridas de tipo/evento nesta rodada são propostas e permanecem draft; nenhuma decisão arquitetural de provider foi congelada.
