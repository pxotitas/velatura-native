# Arquitetura proposta

**Status:** proposta inicial; deve ser validada após auditoria aprofundada do original.

## Direção

Aplicação nativa em Rust, UI GTK4 e integração Wayland/Niri. Separar domínio, adaptadores de sistema, estado e apresentação.

Não criar microserviços/processos separados sem necessidade. Começar como aplicação modular única; extrair serviços em processos independentes apenas se existir requisito concreto de isolamento, ciclo de vida ou integração.

## Camadas

### Core / domínio
- ciclo de vida e inicialização;
- estado global mínimo;
- configuração e migrações;
- eventos internos;
- agendamento de tarefas;
- logging e diagnóstico;
- APIs tipadas consumidas pela UI.

### System adapters
- compositor / Niri IPC;
- monitores e outputs;
- áudio / PipeWire ou PulseAudio;
- rede / NetworkManager;
- Bluetooth / BlueZ;
- bateria / UPower ou sysfs quando adequado;
- brilho;
- notificações / freedesktop notifications;
- tray / StatusNotifierItem;
- MPRIS / players;
- polkit;
- portal de captura;
- wallpaper;
- sessão / logind;
- aplicações / desktop entries;
- clipboard;
- câmera / PipeWire;
- temperatura, CPU, memória, disco e rede.

Cada adaptador deve declarar fonte de dados, permissões, disponibilidade, eventos, reconexão e comportamento degradado.

### Presentation
- GTK4 widgets e janelas;
- superfícies Wayland/layer-shell;
- componentes visuais reutilizáveis;
- estado de apresentação separado do domínio;
- carregamento sob demanda;
- animações curtas e com custo controlado.

### Persistence
- configuração XDG em TOML ou JSON versionado;
- estado de usuário em XDG_STATE_HOME;
- dados de usuário em XDG_DATA_HOME;
- cache em XDG_CACHE_HOME;
- nunca guardar segredos em arquivos de configuração sem proteção.

## Estrutura de diretórios sugerida

    velatura-native/
    ├── Cargo.toml
    ├── README.md
    ├── AGENTS.md
    ├── design.md
    ├── docs/
    ├── prompts/
    ├── src/
    │   ├── main.rs
    │   ├── app/
    │   ├── core/
    │   ├── config/
    │   ├── domain/
    │   ├── compositor/
    │   ├── system/
    │   ├── services/
    │   ├── ipc/
    │   └── ui/
    ├── tests/
    ├── resources/
    └── packaging/

A estrutura é convenção inicial, não ordem para criar diretórios vazios. Criar módulos apenas quando houver implementação real.

## Contrato de eventos

Preferir eventos tipados, por exemplo:
- WorkspaceChanged
- FocusedOutputChanged
- OutputAdded / OutputRemoved
- AudioStateChanged
- BatteryStateChanged
- NotificationReceived
- MediaMetadataChanged
- ConfigChanged
- ServiceAvailabilityChanged

Os nomes são ilustrativos. O esquema final deve refletir as APIs reais.

## Compositor e superfícies

Criar trait/abstração de compositor para identificar compositor e versão, conectar/reconectar ao IPC, acompanhar outputs/workspaces/janelas, receber eventos sem polling agressivo e expor capacidades disponíveis.

A primeira implementação é Niri. Não assumir que todos os compositores implementam as mesmas funções.

Para layer-shell, validar suporte do Niri ao protocolo utilizado, anchors, exclusive zone, keyboard interactivity, input region, hotplug de monitor, fullscreen e reconexão de output.

A SettingsWindow do original é uma janela normal xdg-toplevel, não uma layer surface. Preservar essa distinção.

## Persistência e migração

O original persiste preferências em $XDG_STATE_HOME/quickshell/settings.json (fallback ~/.local/state/quickshell/settings.json). A nova aplicação deve inventariar o schema completo, preservar preferências quando viável, versionar schema, validar e recuperar de JSON/TOML inválido sem apagar o arquivo, nunca modificar a configuração original e importar settings apenas com consentimento e backup.

## Performance

- Eventos primeiro; polling somente quando a API não oferece eventos.
- Um watcher por recurso, compartilhado entre consumidores.
- Debounce/throttle para sensores e estatísticas.
- Intervalos de atualização diferentes por dado e visibilidade.
- Atualização de UI somente quando valor relevante muda.
- Widgets e painéis caros devem pausar quando ocultos.
- Não exigir compositor blur ou sombras GPU; superfícies sólidas são o caminho padrão.
- Registrar métricas de idle e atividade com ambiente e método reproduzíveis.

## Testabilidade

Cada adapter deve permitir provider fake. Testar estado inicial, atualizações/eventos, desconexão/reconexão, dispositivo ausente, erro de permissão, valores extremos, múltiplos monitores e mudanças rápidas de estado.

## Não decidido

- Relm4 vs GTK4 direto.
- Tokio vs GLib main context para cada integração.
- IPC interno (canal tipado) vs D-Bus/Unix socket externo.
- formato final de configuração.
- estratégia de tray.
- backend de wallpaper e captura.
- política de animações por acessibilidade/performance.

Registrar decisão e justificativa em DECISIONS.md antes de tornar uma escolha difícil de reverter.


## Estado após auditoria do backend (15/11/2026)

A primeira auditoria baseada na leitura integral de arquivos ativos está em [BACKEND_AUDIT.md](BACKEND_AUDIT.md); o mapa de arquivos/runtime foi atualizado em [SOURCE_INVENTORY.md](SOURCE_INVENTORY.md), e os estados parciais estão em [PORTING_MATRIX.md](PORTING_MATRIX.md).

Fatos que condicionam a arquitetura:

- O original usa o Niri event stream como fonte de verdade para workspaces/janelas/foco e serializa ações via `niri msg action`; o monitor inventory vem de `niri msg --json outputs`.
- O original mantém superfícies por output e exatamente uma ilha live. O "primary" efetivo atual é foco com fallback para primeiro output ativo, não uma propriedade independente do Niri.
- O ambiente atual contém `Virtual-1`; o adapter de outputs deve tratá-lo como output normal e nunca alterar a configuração do compositor.
- O serviço de settings tem schema persistido extenso, separa machine keys de profile keys, migra valores legados e coalesce writes. A implementação Rust precisa preservar chaves desconhecidas e fazer import explícito/backup.
- Áudio, Bluetooth, bateria, notifications, MPRIS e tray são originalmente integrações reativas D-Bus/PipeWire; rede, brilho externo e clipboard têm subprocessos auxiliares específicos documentados na auditoria.
- Foi identificado um problema de segurança herdado: a senha Wi-Fi aparece como argumento do subprocesso original. A API Rust deve preservar a ação sem preservar a exposição.

**Contrato com Codeína:** `Commands.qml`, CLI, serviços e superfícies QML têm auditoria estática documentada em IPC_AUDIT.md, BACKEND_AUDIT.md, INTEGRATION_AUDIT.md e UI_SURFACE_AUDIT.md. A revisão técnica está em [APP_API_REVIEW.md](APP_API_REVIEW.md) e concluiu que APP_API_CONTRACT.md não está pronto para servir de base completa à portabilidade 1:1. Mocks só devem ser sincronizados com tipos finais após fechar os bloqueadores.

**Decisões ainda abertas:** GTK4 direto vs Relm4; GLib main context vs Tokio por serviço; estratégia layer-shell no Niri; backend nativo de tray/capture/clipboard. Nenhuma dessas decisões foi tomada nesta rodada.

## Contrato tipado inicial Backend ↔ Codeína (15/11/2026)

O draft está em [APP_API_CONTRACT.md](APP_API_CONTRACT.md), com parecer em [APP_API_REVIEW.md](APP_API_REVIEW.md). Ele propõe:
- AppHandle com snapshot cacheado, connect() para snapshot/event stream (handshake, lacunas e atomicidade ainda sem especificação suficiente) e dispatch assíncrono;
- AppSnapshot revisionado, ServiceStatus por integração, SettingsSnapshot preservando chaves desconhecidas;
- AppEvent incremental e tipado, CommandReceipt/CommandFinished separados do aceite inicial;
- AppCommand por domínio, incluindo sistema, Niri, mídia, captura, account/face/camera, produtividade, pacotes, VMs, servidores, wallpaper e adaptadores de IPC legado;
- subscriptions com ownership/RAII, lifecycle lazy e sem watcher duplicado;
- estado multi-monitor com Virtual-1 como output normal e no máximo uma ilha live;
- fixtures sintéticas para snapshot inicial e eventos incrementais em docs/fixtures/ (não funcionais, apenas para composição frontend).

**Importante:** contrato v0.1-draft, não pronto e não congelado. Além da revisão de Codeína, faltam contratos de estados/comandos de serviços e superfícies, lifecycle/recovery, compatibilidade e migração; Capture Window segue bloqueado por `root.windowUnder()` sem definição. A decisão concreta de EventStream/GLib/Tokio, Relm4 vs GTK4 direto e transporte permanece aberta; este documento não toma essas decisões. Atualizações devem ser acompanhadas de fixtures. O primeiro código Rust foi iniciado em 16/11/2026; o estado atual dessa fatia está registrado abaixo.

## Auditoria visual e funcional estática (15/11/2026)

A auditoria das superfícies raiz e componentes visuais está em [UI_SURFACE_AUDIT.md](UI_SURFACE_AUDIT.md); o índice mecânico de 303 componentes ativos, com serviços, funções, signals e handlers, está em [UI_COMPONENT_CATALOG.md](UI_COMPONENT_CATALOG.md). Foram mapeados os 22 módulos, 26 IDs de painel da ilha, 16 block faces, 14 seções de Settings, 11 jogos, faces Modern/Analogue/Sticker, Dock, Deck, Lock e Capture.

Esta etapa é de leitura estática, não de paridade verificada. Screenshots, input real, hotplug, layer-shell, lock/PAM, webcam, capture/OCR e benchmarks ficam para a fase de validação. O modo Capture Window permanece bloqueado até resolver a chamada sem definição root.windowUnder().

## Primeira fatia de implementação — Niri (16/11/2026)

O repositório agora contém o crate Rust `velatura-core` (biblioteca, edition 2024), com `serde_json` como dependência inicial e sem runtime assíncrono escolhido. A primeira unidade implementada é `src/niri.rs`: um redutor puro de linhas JSONL para estado Niri, separado de processo, UI e efeitos no sistema.

A implementação foi comparada com a fonte ativa `/home/px/.config/quickshell/services/NiriService.qml` e cobre os dez handlers observados: WorkspacesChanged, WindowsChanged, WindowOpenedOrChanged, WindowClosed, WindowFocusChanged, WorkspaceActivated, WindowUrgencyChanged, WorkspaceUrgencyChanged, WorkspaceActiveWindowChanged e WindowLayoutsChanged. Eventos válidos desconhecidos continuam sendo encaminhados sem projeção; handshake, JSON inválido, connected e persistência de lastError seguem as particularidades do adapter original.

A suíte inicial usa fixtures sintéticas e testes unitários, sem alterar a sessão Niri. Em 16/11/2026, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` e `cargo test` passaram; o probe read-only `niri msg --json outputs` respondeu com o output atual. Um probe curto do event stream confirmou o handshake/snapshot inicial e eventos adicionais do Niri 26.04; nenhum título ou estado real de janela foi gravado em fixtures.

**Limite atual:** isto ainda não é um adapter Niri funcional de ponta a ponta. Faltam supervisor do processo JSONL, tratamento de stderr/exit, reconnect de 3000ms apenas em exit não-zero, fila serial de actions, consulta/refresh de outputs e integração com o ciclo de vida da aplicação. Não foi escolhido Tokio, Relm4, GTK direto nem um contrato final de eventos. O contrato Backend ↔ Codeína continua draft.


## Segunda fatia de implementação — Niri process e actions (16/11/2026)

Foram adicionados dois módulos sem dependência de GTK ou runtime assíncrono:

- `src/niri_process.rs`: inicia `niri msg --json event-stream`, lê stdout linha a linha e stderr em paralelo, publica saída/diagnósticos/exit, permite parada explícita e aplica a política observada na fonte: reconectar após 3000 ms apenas para exit não-zero. Exit zero encerra o supervisor sem restart; falha ao criar o processo é reportada como SpawnFailed e não recebe uma política de retry inventada.
- `src/niri_actions.rs`: worker FIFO que executa um único `niri msg action` por vez. Cada argumento permanece um argv separado; não há shell, interpolação ou paralelismo. O resultado contém exit code, stdout/stderr e erro de spawn reais, sem receipt de aceite fictício.

A implementação usa std::process, threads e canais da biblioteca padrão. Isso é uma escolha local e reversível para esta fatia, não uma decisão global de runtime para os demais serviços. Os executáveis podem ser injetados nos construtores para testes isolados.

Testes usam executáveis fake temporários e não invocam ações reais do compositor. Em 16/11/2026, 13 testes passaram, incluindo forwarding stdout/stderr, exit limpo sem reconnect, reconnect após exit não-zero com atraso mínimo de 3000 ms medido, parada de child, FIFO de actions e propagação de exit não-zero. Clippy com warnings negados e diff check também foram executados.

**Limites:** ainda falta consumir as mensagens do supervisor no reducer e no event bus, ligar o lifecycle ao app, implementar outputs/monitor refresh e testar reconexão não-zero com uma instância real do Niri. Os módulos não foram ligados automaticamente à sessão nem executaram ações reais.
