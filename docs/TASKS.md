# Backlog e estado do projeto

**Estado geral:** inventário estático inicial e contrato 1:1 documentados; auditoria funcional profunda ainda em andamento; implementação ainda não iniciada.

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
- [ ] Completar auditoria linha a linha dos serviços e scripts.
- [ ] Validar comandos externos e endpoints lendo cada script integralmente.
- [ ] Mapear eventos, sinais, timers e lifecycle de cada serviço.
- [ ] Confirmar dependências e comportamento degradado no hardware real.
- [ ] Fechar contratos Rust tipados por domínio com Codeína.

## Fase 1 — Auditoria e especificação

- [ ] Mapear grafo de imports ativo do shell.qml.
- [ ] Identificar serviços usados no startup versus carregados sob demanda.
- [ ] Auditar todos os serviços de sistema.
- [ ] Auditar todos os painéis e módulos.
- [ ] Auditar dock, desktop, deck, lock e capture.
- [ ] Auditar comandos de Commands.qml e compatibilidade CLI.
- [ ] Catalogar dependências externas e permissões.
- [ ] Extrair schema completo de configurações e defaults.
- [ ] Definir escopo de islet e macos-qckshell.

## Fase 2 — Arquitetura e contratos

- [ ] Escolher modelo de UI (Relm4 ou GTK4 direto).
- [ ] Definir runtime assíncrono.
- [ ] Definir AppSnapshot, AppEvent, AppCommand e erros.
- [ ] Definir abstração de compositor.
- [ ] Definir modelo de superfícies e multi-monitor.
- [ ] Definir persistência e migração de configurações.
- [ ] Criar mocks/fixtures compartilhados.

## Fase 3 — Fundação

- [ ] Cargo workspace/projeto.
- [ ] Logging e diagnóstico.
- [ ] Configuração e schema versionado.
- [ ] Event bus.
- [ ] Ciclo de vida.
- [ ] Adapter Niri.
- [ ] Adapter de monitores.
- [ ] Protótipo GTK layer-shell isolado.

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
- [ ] Teste multi-monitor e hotplug.
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
