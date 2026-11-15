# Velatura Native

Reconstrução nativa do Velatura para Linux, preservando identidade visual, comportamento e funcionalidades do shell original.

**Estado:** documentação e planejamento. Nenhuma implementação nativa foi iniciada.
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
