# Registro de decisões

## D-001 — Manter o shell original intacto

**Estado:** aceito.

A instalação original em ~/.config/quickshell continua sendo a referência e deve permanecer funcional durante toda a portabilidade. O novo projeto é desenvolvido em ~/Projects/velatura-native.

Motivo: permitir comparação e rollback sem risco para o desktop atual.

## D-002 — Implementação nativa, não um wrapper

**Estado:** aceito.

O shell final não pode depender de Quickshell, QML, Qt Quick, Electron ou WebView. Rust + GTK4 é a direção tecnológica inicial.

Motivo: reduzir dependências de runtime e obter controle nativo sobre recursos, integração e consumo.

## D-003 — Niri como primeiro compositor

**Estado:** aceito.

A primeira implementação de compositor será Niri/Wayland, com abstração para outros compositores no futuro.

Motivo: é o ambiente primário de desenvolvimento e uso.

## D-004 — Visual clássico sólido como baseline

**Estado:** aceito.

O baseline visual será surfaceStyle classic, com cápsulas escuras sólidas. Glass e liquid serão preservados como modos opcionais após a versão sólida estar correta.

Motivo: o original já tem classic como default e o hardware tem orçamento limitado de GPU/CPU.

## D-005 — Não trocar o autostart antes de aprovação

**Estado:** aceito.

A aplicação nativa não será adicionada ao autostart nem substituirá o Quickshell até aprovação explícita.

Motivo: proteger a sessão diária do usuário.

## Pendências técnicas

- Relm4 ou GTK4 direto.
- Modelo assíncrono: GLib main context, Tokio ou combinação justificada.
- Contrato de estado/eventos entre backend e frontend.
- Formato de configuração e estratégia de migração.
- Backend de tray.
- Estratégia de lock screen e autenticação.
- Captura de tela e integração com portais.
- Escopo de projetos aninhados islet e macos-qckshell.

## Pontos a confirmar na fonte original

- **Capture / window picking:** `CaptureOverlay.qml` chama `root.windowUnder(mouse.x, mouse.y)`, mas não há definição desse método no arquivo nem no checkout ativo. É um candidato a bug/símbolo ausente, não uma decisão de redesign. Reproduzir em runtime ou recuperar a implementação correta antes de especificar/portar o modo Window; não inventar um comportamento substituto.


## D-006 — GDK4 observer direto para mudanças de monitor

**Estado:** aceito para a camada de integração de monitores.

O adapter de monitores usa GDK4 diretamente: a lista `GdkDisplay::monitors()` informa inclusão/remoção via `GListModel::items-changed`, e cada `GdkMonitor` informa mudança de geometria via `notify::geometry`. Os callbacks apenas solicitam refresh ao serviço Niri já existente; debounce e subprocessos continuam fora do main thread.

A integração fica atrás do feature opcional `gdk4-observer` e usa GDK4 0.10.x, compatível com o MSRV 1.85 declarado. Isso **não** decide GTK4 direto vs Relm4 para a construção da UI nem escolhe o runtime assíncrono global. O futuro `GtkApplication` deve criar e destruir o observer no main context e manter o `ApplicationRuntime` durante a vida da aplicação.

## D-007 — Bootstrap mínimo com GTK4 direto

**Estado:** aceito para o bootstrap e a primeira superfície de teste; não congela a arquitetura final de UI.

O executável inicial usa GTK4 direto para criar `GtkApplication`, manter `ApplicationRuntime` durante `activate`/`shutdown` e exibir uma janela de teste. A UI real pode ser organizada em módulos próprios sem assumir ownership do lifecycle. Relm4 continua possível mediante decisão posterior, sem duplicar a instância da aplicação ou o backend.

O comando de desenvolvimento é instalado como `~/.local/bin/velatura-native`. O comando `~/.local/bin/velatura` pertence ao shell legado e permanece intacto até uma migração deliberada.

## D-008 — TimerSlot: fontes GLib one-shot e o panic do glib 0.22

**Estado:** aceito para island.rs e qualquer futuro timer local que precise ser cancelável.

O glib 0.22 entra em panic se uma `SourceId` já disparada for removida: um one-shot destrói a própria fonte após despachar, e `SourceId::remove()` numa fonte morta é erro. O padrão `TimerSlot(Rc<RefCell<Option<SourceId>>>)` resolve isso:

- o callback do timer chama `slot.clear()` antes de qualquer lógica, parando de rastrear a fonte que se autodestrói;
- `cancel()` só remove fontes realmente pendentes;
- `Drop` do widget chama `cancel()` nos slots, nunca `remove()` direto.

Aplicado ao morph de largura/altura (`Island::animation`) e aos três timers de hover (`dwell`, `grace`, `leave_close`). Novos timers canceláveis devem usar o mesmo padrão; remover `SourceId` crus é proibido no código de UI.

## D-009 — Hover do island: estados separados "agora" e "sticky"

**Estado:** aceito; corrige desvio de paridade detectado durante o rework dos timers.

O QML original distingue `hover.hovered` (ponteiro dentro agora) de `pointerEntered` (sticky: o ponteiro já esteve sobre o island; só é resetado quando `openPanel` muda). A primeira versão Rust mesclava os dois em `pointer_inside`, o que impedia o `leave_close` (380 ms) de armar, porque o teste `pointer_inside` já era falso no momento do armamento. O port agora mantém os dois campos (`pointer_inside`, `pointer_entered`) e espelha `onOpenPanelChanged`: abrir/fechar painel cancela o `leave_close` e recarrega `pointer_entered = pointer_inside && painel não vazio`.

## D-010 — Desvios conhecidos da primeira fatia de UI (aguardando decisão/validação)

**Estado:** registrados; nenhum é rebaixamento silencioso de recurso.

1. **Input region:** a superfície da barra usa layer-shell com teclado exclusivo sob demanda; a região de entrada precisa validar no Niri se áreas transparentes capturam clique. Pendente de validação visual.
2. **Clique em repouso:** o clique no island em repouso abre o launcher. No original, o clique abre o control centre; o launcher abre por atalho. Interim documentado até o painel de controles existir; reverter ao portar ControlsPanel.
3. **Multi-monitor:** o island vivo fica no primeiro monitor (índice 0) até existir rastreio de saída focada do Niri; as demais barras são réplicas estáticas. Pendente: focused-output tracking.

## D-011 — Primeira fatia de UI arquivada em `glm-made-things/`

**Estado:** primeira fatia arquivada por decisão do usuário em 16/11/2026; re-port parcial posterior em andamento.

A primeira implementação da barra/island (`src/ui/`) foi reprovada na
validação visual: barra torta no meio, faixa preta do módulo de notificações
até a borda direita, transparência incorreta e workspaces fora do estilo do
original. Os arquivos foram movidos para `glm-made-things/` (com `README`
descrevendo conteúdo, defeitos conhecidos e o que estudar no QML original) e
`src/ui/` foi removido naquela revisão. Os commits `0eeaa45` a `fa62b17` retomaram um re-port parcial: superfície da barra, reserva, workspaces, relógio em repouso e notificações. Isso não revoga a reprovação visual da primeira fatia nem comprova paridade da implementação atual.

- A decisão arquivou os arquivos da primeira tentativa; não deve ser lida como estado atual do re-port.
- O backend (`src/services/`, sessão Niri e CLI) permanece na árvore. O estado de validação atual deve ser consultado nos checks executados, sem inferir que os testes seguem verdes por este registro histórico.
- Lições registradas para o re-port: janela GTK4 só mapeia com
  `window.present()`; o original usa superfície full-screen transparente com
  reserve em superfície separada e band que morfa (`Bar.qml`); os estilos
  reais de cápsula/fonte estão em `BarZone.qml`, `WorkspacesWidget.qml`,
  `ClockModule.qml` e `Theme.qml`, com defaults
  `barStyle: "grouped"`, `barSides: "capsule"`.
- O re-port deve ser feito comparando lado a lado com o original na sessão,
  fatia por fatia, e validando com `niri msg layers` + screenshot.


## D-012 — Reserva persistente da faixa superior (BarReserve)

**Estado:** implementado no frontend nativo; validação em runtime no Niri pendente.

A reserva do topo é uma superfície layer-shell independente por monitor, na camada `background`, ancorada em top/left/right, com altura visual de 1 px, região de input vazia e exclusive zone igual a `barMargin + barHeight` (ou zero quando `barHidden`). A superfície visível da barra continua ignorando exclusive zones.

Isso reproduz o contrato de `bar/BarReserve.qml` e `Bar.qml`: a reserva pertence à tela, não à janela visível da barra, e permanece enquanto a superfície da barra é recriada. O comportamento do Niri com essa combinação específica ainda precisa ser verificado com `niri msg layers` e uma janela tiled de teste; não alterar a configuração do compositor para validar.
