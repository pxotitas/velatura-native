# Velatura Native

Reconstrução nativa do Velatura para Linux, preservando identidade visual, comportamento e funcionalidades do shell original.

**Estado:** backend Niri implementado: reducer, supervisor/reconnect, fila de actions, outputs, sessão integrada, observer GDK4 e `ApplicationRuntime`. Bootstrap mínimo GTK4/`GtkApplication` disponível com lifecycle real e superfície temporária de teste; UI final, layer-shell e validação visual 1:1 continuam pendentes.
**Fonte de referência:** /home/px/.config/quickshell (instalação original; manter intacta).

## Objetivo

Portar o Velatura de Quickshell/QML para uma aplicação Linux nativa, preferencialmente Rust + GTK4, sem Quickshell, QML, Electron ou WebView em tempo de execução.

Não é um redesign nem um shell inspirado no Velatura. A aplicação nova deve ser uma implementação equivalente do produto existente.

## Documentos principais

- AGENTS.md — regras obrigatórias para todos os agentes.
- design.md — especificação visual, layout, superfícies, componentes e movimento.
- prompts/PROMPT_BACKEND.md — instruções para ChatGPT/backend/integrações.
- prompts/PROMPT_FRONTEND_CODEINA.md — instruções para Codeína, responsável pela UI.
- docs/ARCHITECTURE.md — arquitetura proposta e contratos entre camadas.
- docs/SOURCE_INVENTORY.md — mapa do projeto original e caminhos de referência.
- docs/PORTING_MATRIX.md — matriz de paridade funcional.
- docs/PARITY_CONTRACT.md — contrato obrigatório de paridade 1:1; define o que significa reproduzir o Velatura integralmente.
- docs/SERVICE_CATALOG.md — inventário dos serviços QML, APIs internas e consumidores.
- docs/SCRIPT_CATALOG.md — scripts auxiliares, imports e ferramentas externas.
- docs/DEPENDENCY_MAP.md — dependências e integrações de sistema.
- docs/IPC_API.md — comandos públicos do Velatura original.
- docs/SERVICE_GRAPH.md — acoplamento entre serviços e lifecycle observado.
- docs/SETTINGS_SCHEMA.md — todas as chaves e defaults de configuração.
- docs/FEATURE_SURFACE_MAP.md — inventário de módulos, painéis, settings e faces de desktop.
- docs/DECISIONS.md — decisões técnicas e pendências.
- docs/TASKS.md — backlog e estado real do trabalho.

## Ambiente primário

- CachyOS Linux (base Arch), Niri / Wayland.
- MacBook Air 2017, Intel Core i5-5350U (2C/4T), 8 GB RAM, Intel HD Graphics 6000.
- GTK 4.24.1 e Rust 1.99.1 detectados durante a preparação deste repositório.

O alvo é limitado em CPU e memória. Preferir eventos, operações assíncronas, carregamento sob demanda e poucas atualizações de UI.

## Fonte original

A instalação original está em /home/px/.config/quickshell. Ela contém aproximadamente 658 arquivos e ocupa cerca de 6,6 MB na data da auditoria inicial.

**Não mover, renomear, editar, apagar ou substituir essa instalação.** O novo projeto vive neste repositório separado. A referência original é somente leitura durante a portabilidade.

## Começar

1. Leia integralmente AGENTS.md.
2. Leia design.md e docs/ARCHITECTURE.md.
3. Leia o prompt correspondente ao seu papel.
4. Inspecione os arquivos originais citados antes de implementar cada recurso.
5. Atualize docs/PORTING_MATRIX.md e docs/TASKS.md conforme o trabalho realmente avança.

Não iniciar autostart nem substituir o shell atual até haver paridade suficiente, testes e autorização explícita.


## Build e validação do backend

O core permanece independente de GTK por padrão:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

O observer GDK4 e o dono do lifecycle da aplicação são opcionais:

```bash
cargo test --features gdk4-observer
cargo clippy --all-targets --all-features -- -D warnings
```

O feature `gdk4-observer` requer GTK4/GDK4 de desenvolvimento instalados. O binário GTK4 é habilitado pelo feature `gtk-app`; `src/main.rs` cria o `GtkApplication`, mantém um único `ApplicationRuntime` entre `activate` e `shutdown`, e monta a superfície temporária em `src/ui/bootstrap.rs`.


## Executável GTK4 e CLI de desenvolvimento

O bootstrap mínimo é construído com:

```bash
./scripts/install-dev-cli.sh
```

O instalador cria `~/.local/bin/velatura-native`, sem substituir o comando legado `~/.local/bin/velatura`. Antes do primeiro build, esse comando usa Cargo como launcher e compila com um job; depois do build, o instalador pode apontar diretamente para o binário. Neste ambiente `~/.local/bin` já está no `PATH`. Comandos disponíveis no bootstrap:

```bash
velatura-native run
velatura-native help
velatura-native version
velatura-native stop
velatura-native restart
```

`ipc` ainda não está implementado e retorna erro explícito; ele não encaminha comandos ao Quickshell antigo. O CLI legado continua preservado até uma migração deliberada.

A janela atual é uma superfície de teste GTK4: confirma inicialização, backend Niri, enumeração GDK de monitores e solicitação manual de refresh. Não representa a UI final nem usa layer-shell.
