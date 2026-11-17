# Velatura Native

Portabilidade nativa do [Velatura](https://github.com/pxotitas/velatura) para Linux, preservando sua identidade visual, seus comportamentos e seu conjunto de funcionalidades.

**Estado atual:** a fundação GTK4 + Rust já está integrada ao Niri e a primeira implementação real do shell está em andamento. Já existem superfícies layer-shell por monitor, reserva independente da área superior, seleção da ilha ativa conforme o output focado, cápsula central com relógio e morphing, indicadores de workspaces e camada de notificações com ações. A ilha normal (estado de repouso) está sendo finalizada; ainda faltam atividades/módulos no repouso, as demais camadas e painéis da ilha, a integração funcional de vários módulos da barra e as outras superfícies do shell. O projeto continua em desenvolvimento e ainda não deve substituir a instalação original.

**Fonte funcional e visual:** [pxotitas/velatura — implementação original em Quickshell](https://github.com/pxotitas/velatura/tree/main/home/.config/quickshell). O repositório original é a referência; consulte os arquivos-fonte diretamente pelo GitHub. Nunca edite a fonte original durante a portabilidade.

## Objetivo

Reimplementar o Velatura como uma aplicação Linux nativa, principalmente em Rust + GTK4 + Wayland layer-shell, sem Quickshell, QML/Qt Quick, Electron, WebView ou runtime JavaScript em tempo de execução.

Isto não é um redesign, uma versão lite ou um shell apenas inspirado no Velatura. O produto final deve preservar o comportamento e as funcionalidades do original, mesmo que sua implementação interna seja completamente diferente.

## Estado da implementação

### Já implementado

- Backend Niri com event stream, reducer, supervisor/reconexão, fila serial de actions e descoberta de outputs.
- Sessão Niri integrada, snapshots e lifecycle coordenado por `ApplicationRuntime`.
- Observer GDK4 para mudanças de monitores.
- Uma superfície GTK4 layer-shell da barra por monitor, com região de input delimitada.
- Superfície de reserva independente para a exclusive zone da barra.
- Ilha central com cápsula desenhada nativamente, geometria animada e troca de conteúdo por cross-fade.
- Relógio no estado normal da ilha.
- Seleção da ilha ativa pelo output focado no Niri.
- Indicadores de workspaces com estados visuais e interação para trocar de workspace.
- Chip visual do launcher (a integração completa com o painel ainda está pendente).
- Serviço nativo de notificações D-Bus e apresentação na ilha, com conteúdo, botões de ação, dismiss e expiração automática.
- Tokens visuais, CSS GTK e atualização da paleta.

### Em andamento / pendente

- Finalizar o estado normal da ilha, incluindo atividades em execução ao redor do relógio e seus comportamentos.
- Completar todas as camadas da Dynamic Island: summary/glance, OSD, painéis, detalhes de módulos e transições entre estados.
- Ligar os botões e módulos da barra aos serviços e painéis reais; implementar tray, rede, Bluetooth, áudio, bateria e demais módulos conforme o original.
- Reproduzir os estilos e layouts configuráveis da barra (grouped, spread, island e bezel), incluindo todos os detalhes de input, fillets e comportamento multi-monitor.
- Portar os painéis, Control Centre, launcher, Settings e suas páginas.
- Portar as demais superfícies: Desktop/widgets, Dock, Deck, Lock, Capture, cantos de tela e integrações relacionadas.
- Completar persistência, migração de configurações, atalhos, CLI/IPC e integração dos serviços.
- Validar paridade visual e funcional em runtime, incluindo hotplug e múltiplos monitores.

Os itens acima são uma visão resumida. A lista autoritativa de trabalho é `docs/TASKS.md`, e o estado por funcionalidade fica em `docs/PORTING_MATRIX.md`. Não marcar um recurso como concluído apenas porque sua interface foi desenhada.

## Repositórios e fontes

- **Projeto nativo:** [pxotitas/velatura-native](https://github.com/pxotitas/velatura-native).
- **Shell original:** [pxotitas/velatura](https://github.com/pxotitas/velatura).
- **Fonte Quickshell original:** [home/.config/quickshell](https://github.com/pxotitas/velatura/tree/main/home/.config/quickshell).
- **Dynamic Island original:** [bar/widgets/DynamicIsland.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/bar/widgets/DynamicIsland.qml).
- **Estado e arbitragem da ilha:** [bar/island/IslandState.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/bar/island/IslandState.qml).
- **Repouso/atividades da ilha:** [bar/island/IslandRest.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/bar/island/IslandRest.qml).
- **Barra e reserva:** [bar/Bar.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/bar/Bar.qml) · [bar/BarReserve.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/bar/BarReserve.qml).
- **Workspaces:** [bar/widgets/WorkspacesWidget.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/bar/widgets/WorkspacesWidget.qml).
- **Configurações originais:** [services/SettingsService.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/services/SettingsService.qml).
- **Tokens visuais:** [theme/Theme.qml](https://github.com/pxotitas/velatura/blob/main/home/.config/quickshell/theme/Theme.qml).

## Documentação do projeto

Leia estes documentos antes de implementar:

- [AGENTS.md](AGENTS.md) — regras obrigatórias para todos os agentes.
- [design.md](design.md) — especificação visual, layout, superfícies, componentes e movimento.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — arquitetura e contratos entre camadas.
- [docs/SOURCE_INVENTORY.md](docs/SOURCE_INVENTORY.md) — inventário da fonte original.
- [docs/PORTING_MATRIX.md](docs/PORTING_MATRIX.md) — matriz de paridade funcional.
- [docs/PARITY_CONTRACT.md](docs/PARITY_CONTRACT.md) — contrato obrigatório de paridade 1:1.
- [docs/UI_SURFACE_AUDIT.md](docs/UI_SURFACE_AUDIT.md) — auditoria das superfícies visuais.
- [docs/UI_COMPONENT_CATALOG.md](docs/UI_COMPONENT_CATALOG.md) — catálogo de componentes QML.
- [docs/SERVICE_CATALOG.md](docs/SERVICE_CATALOG.md) — serviços originais e consumidores.
- [docs/SCRIPT_CATALOG.md](docs/SCRIPT_CATALOG.md) — scripts auxiliares e ferramentas externas.
- [docs/DEPENDENCY_MAP.md](docs/DEPENDENCY_MAP.md) — dependências e integrações de sistema.
- [docs/IPC_API.md](docs/IPC_API.md) e [docs/IPC_AUDIT.md](docs/IPC_AUDIT.md) — contrato público de CLI/IPC.
- [docs/SETTINGS_SCHEMA.md](docs/SETTINGS_SCHEMA.md) — chaves e defaults de configuração.
- [docs/FEATURE_SURFACE_MAP.md](docs/FEATURE_SURFACE_MAP.md) — módulos, painéis, settings e superfícies.
- [docs/DECISIONS.md](docs/DECISIONS.md) — decisões técnicas e pendências.
- [docs/TASKS.md](docs/TASKS.md) — backlog e estado real do trabalho.

## Ambiente-alvo

- CachyOS Linux (base Arch), Niri / Wayland.
- MacBook Air 2017, Intel Core i5-5350U (2 núcleos / 4 threads), 8 GB RAM e Intel HD Graphics 6000.
- Stack GTK4 e Rust; as versões exatas instaladas variam conforme o ambiente.

O hardware é limitado em CPU, memória e temperatura. Preferir arquitetura orientada a eventos, operações assíncronas, cache e carregamento sob demanda. Otimização nunca pode remover fidelidade ou funcionalidades.

## Começar

1. Leia integralmente `AGENTS.md`.
2. Leia `design.md`, `docs/ARCHITECTURE.md` e `docs/PARITY_CONTRACT.md`.
3. Leia os documentos específicos do recurso em `docs/`.
4. Compare a implementação com a fonte original em [pxotitas/velatura](https://github.com/pxotitas/velatura).
5. Atualize `docs/PORTING_MATRIX.md`, `docs/TASKS.md` e os documentos de auditoria conforme o trabalho realmente avança.

Não habilite autostart nem substitua o shell atual até haver paridade suficiente, testes e autorização explícita.

## Build e validação

O core pode ser testado sem compilar a UI GTK:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

Com o observer GDK4:

```bash
cargo test --features gdk4-observer
cargo clippy --all-targets --all-features -- -D warnings
```

A aplicação GTK4 e o layer-shell são habilitados pelo feature `gtk-app`:

```bash
cargo run --features gtk-app -- run
```

Para instalar o comando de desenvolvimento `velatura`:

```bash
./scripts/install-dev-cli.sh
```

O executável público chama-se `velatura`. O CLI já reconhece `run`, `help`, `version`, `stop` e `restart`; os grupos de domínio/IPC estão catalogados, mas ainda precisam ser conectados aos serviços nativos. Nenhum comando deve encaminhar chamadas ao Quickshell legado.

## Segurança durante a migração

- A instalação original do Velatura deve permanecer intacta e ser tratada como somente leitura.
- Não alterar configuração do Niri, autostart, serviços do usuário ou configurações do sistema sem autorização explícita.
- Desenvolver e testar em paralelo ao shell atual, sem encerrá-lo ou substituí-lo automaticamente.
- Não executar comandos privilegiados nem instalar pacotes de sistema sem autorização.
- Não adicionar builds, caches, arquivos gerados ou segredos ao Git.

## Licença

Este repositório nativo é distribuído sob a [Apache License 2.0](LICENSE).
