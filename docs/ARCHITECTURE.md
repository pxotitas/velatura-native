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
