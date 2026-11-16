# Auditoria visual e funcional das superfícies — Velatura

**Data:** 15/11/2026<br>
**Fonte primária:** instalação ativa em `/home/px/.config/quickshell`<br>
**Escopo desta rodada:** composição e lifecycle das superfícies raiz; roteamento de painéis; interação de alto nível, foco, input masks, camadas e transições.<br>
**Estado:** auditoria estática dos arquivos raiz concluída; auditoria dos componentes filhos, screenshots comparativos e testes de runtime continuam pendentes.

> Esta ficha não declara paridade visual verificada. A leitura do QML documenta o comportamento pretendido e implementado no original; não substitui comparação por screenshot nem teste real de input/compositor.

## 1. Mapa de composição do shell

`shell.qml` é o orquestrador. Para cada `Quickshell.screens` ele cria:

- `BarReserve`: uma faixa transparente de 1 px, `WlrLayer.Background`, sem input, responsável por manter a exclusive zone da barra estável.
- `Bar`: uma instância por output; exatamente uma recebe `live=true`.
- `Desktop`: uma superfície de desktop por output.
- `Dock`: uma instância por output, com visibilidade condicionada ao modo de dock e ao output live.
- `Deck`: uma instância por output para notas nas bordas.
- `ScreenCorners`: somente quando a preferência correspondente está ativa.

Fora das variantes por monitor, o shell instancia:

- `SettingsWindow`: janela normal xdg-shell, aberta independentemente da ilha.
- `LockScreen`: integração ext-session-lock.
- `CaptureOverlay`: overlay persistente como objeto QML, ativado somente durante captura.
- `Commands`: ponte dos comandos IPC/CLI para a ilha, settings, launcher e picker.

### Handover da Dynamic Island

- O output focado pelo Niri é a primeira escolha.
- Se o nome focado não corresponder a um output conectado, usa `MonitorService.effectivePrimaryName`.
- `claimIsland()` também recebe a reivindicação da barra sob o ponteiro, cobrindo pointer warp ou compositor que não siga o ponteiro.
- A ilha não troca de output enquanto está expandida; ao fechar/assentar, `settleIsland()` resolve o output desejado.
- A instância visual existe em todas as barras; a propriedade `live` decide qual pode abrir painéis e capturar teclado. O handover é uma troca de estado, não destruição/recriação da janela.

### Ligações cruzadas no shell

- Pedido de Settings pela ilha ou pelo Desktop abre `SettingsWindow`; no caso do Desktop, a ilha é fechada antes.
- Pedido de launcher pelo Dock abre o modo correspondente da ilha.
- Pedido Polkit abre `auth`; quando o pedido termina, a ilha fecha se ainda estiver nesse painel.
- Lock pode limpar o clipboard conforme preferência.
- `LockService.shellQuiet` espera a ilha estar fechada e assentada antes da captura do desktop para o lock.
- `CaptureService.recordRequested` delega para `RecorderService.startAt()`.
- Wallpaper aplicado solicita recálculo da paleta adaptativa.
- O shell preserva `CaptureOverlay` instanciado para evitar flash de frame preto ao criar uma layer surface durante captura.

## 2. Barra e reserva

### `bar/BarReserve.qml` — AUDITED (raiz)

- Uma faixa por output, altura de 1 px, transparente, sem região de input.
- Camada `Background`.
- `exclusiveZone = 0` quando a barra está escondida; caso contrário usa `Theme.barReserve`.
- A reserva é separada da janela da barra para que alternar estilos/visibilidade não faça tiled windows saltarem.

### `bar/Bar.qml` — AUDITED (estrutura raiz; filhos pendentes)

- Uma `PanelWindow` full-screen por output, namespace `impasto-bar`.
- Não reserva espaço diretamente; a reserva é responsabilidade de `BarReserve`.
- `live` identifica o output que recebe input/ações. `painted` é `live || barEverywhere`.
- Quatro estilos estruturais: `grouped`, `spread`, `island` e `bezel`. `grouped`/bare e o modo unified alteram como os lados se relacionam com a cápsula central.
- O painel pode ficar maior que o espaço entre os lados no estilo spread; nesse caso `crowded` faz os lados cederem espaço.
- A barra desativa input em condições como desktop em edição, superfície não pintada ou estado zen/away.
- A região de input é derivada da geometria real da ilha, banda e módulos; não é simplesmente um retângulo de clique permanente.
- Só a instância live toma foco de teclado quando a ilha expande.
- Auth pode elevar a barra para `Overlay`; normalmente fica em `Top`.
- Ações de módulos são encaminhadas por `ModuleService` e `IslandState`; clicar no relógio/centro abre Controls.
- A geometria de banda, notch/fillets, glass, sombras e lados é calculada em conjunto com o tamanho animado da ilha.

**Ainda pendente nesta área:** auditoria individual dos 24 arquivos de módulos, widgets da barra, Workspaces, Tray, OSD/notification rendering e screenshots dos quatro estilos.

## 3. Dynamic Island — host e roteamento AUDITED; conteúdo dos painéis pendente

Arquivo raiz: `bar/widgets/DynamicIsland.qml` (760 linhas).

### Camadas

O host escolhe uma das camadas:

- `modules`: ilha em repouso e conteúdo normal da barra.
- `summary`: resumo acionado por hover.
- `osd`: OSD transitório.
- `notification`: toast/notificação.
- `panel`: painel expandido.

O conteúdo é carregado por `Loader`; o tamanho é calculado antes da carga para que a cápsula chegue à geometria-alvo. A forma anima enquanto o conteúdo faz cross-fade.

### Resumo e hover

- Resumo só é permitido se `islandSummary` estiver ativo, a ilha for live, não estiver aterrissando de outro painel, nenhum módulo estiver aberto e a camada for modules/summary.
- Hover precisa permanecer por **80 ms** antes de abrir o resumo.
- Ao sair, há uma graça de **260 ms** antes de fechar o resumo.
- Reentrar cancela o fechamento.
- Um clique mantém o resumo bloqueado até o ponteiro sair, evitando que o resumo reapareça imediatamente depois de dispensar um painel.
- Se o módulo estiver ocupado, o timer de dwell é suspenso.

### Fechamento ao sair do painel

- Um painel aberto por teclado não fecha antes de o ponteiro realmente entrar nele.
- Depois que o ponteiro entrou, sair inicia um timer de **380 ms**.
- Pressionar/arrastar um botão interrompe o fechamento.
- Auth, Notes, Board e Camera não fecham ao sair.
- Games não fecha enquanto um jogo estiver em andamento.
- Controls não fecha enquanto o modo de edição estiver ativo.
- Escape fecha qualquer painel pelo FocusScope pai.
- Clique com botão direito na margem do Controls alterna edição do Control Center.

### Roteamento de painéis

O host mapeia explicitamente: controls, appearance, palette, launcher, wifi, sound, microphone, nightlight, brightness, bluetooth, stats, overview, session, auth, machines, servers, screentime, camera, pet, games, notes, board, keys, packages, module e tray.

- Appearance e Palette compartilham a mesma instância/componente, alternando a página por nome.
- Wi-Fi e Bluetooth são painéis próprios, não subpáginas do Controls.
- Detalhes de áudio, microfone, brilho, night light, Bluetooth e rede retornam ao Controls por ação de Back.
- Painéis que solicitam Settings fecham a ilha antes de emitir o pedido para o shell.
- Detalhes de módulos são construídos com o `ModuleService.openId` atual.

### Animação e forma

- A primeira apresentação não anima de zero; `animated` só é ligado após o primeiro frame.
- Width/height usam `Theme.durationMorph`; radius e cantos usam `Theme.durationMedium`; cor usa `Theme.durationFast`.
- Notched mode altera padding e cantos superiores; Notes aberto vira papel de ponta a ponta, sem rim/padding/glass.
- Em estilo unified/hosted, o fundo da ilha fica transparente quando o glass da banda já fornece o material, evitando dupla composição.

**Ainda pendente:** auditoria linha a linha de cada painel filho, todos os controles internos, empty/error/loading states e comparação visual.

## 4. Settings Window — raiz e navegação AUDITED; páginas pendentes

Arquivos: `settings/SettingsWindow.qml` e `settings/SettingsPanel.qml`.

- Janela flutuante xdg-shell normal, título `Settings`, tamanho implícito **980×680**. Não é layer surface; Niri a gerencia como janela comum.
- O conteúdo é criado ao abrir e destruído ao fechar por um Loader. Isso faz as páginas adquirirem e liberarem subscriptions junto ao lifecycle da janela.
- A janela permanece independente da ilha para que as alterações de aparência sejam vistas ao vivo.
- Painel tem sidebar fixa de **206 px**, marca/palette board, busca, lista de seções e área de página rolável.
- Navegação usa histórico `trail + cursor`, com Back/Forward; cada seção lembra a última tab visitada.
- A busca combina labels/blurb traduzidos e keywords em inglês; Enter abre o primeiro resultado.
- Escape limpa a busca primeiro; com campo vazio o evento não é consumido pelo campo.
- Trocar seção/tab volta o scroll para o topo; o conteúdo de página é trocado por Loader com fade rápido.
- O botão de ajuda abre o guia externo específico da seção/tab.
- As páginas são componentes independentes; Widgets pode fechar Settings e entrar em modo de organização no desktop; Controls fecha Settings e abre o painel Controls; Appearance pode solicitar painéis Palette/Wallpaper.

### Catálogo de seções raiz

1. Bar
2. Island
3. Control Centre
4. Launcher
5. Dock
6. Desktop
7. Modules
8. Appearance
9. Windows
10. Displays
11. Input
12. Keys
13. Session
14. System

Tabs, subpáginas e controles internos ainda precisam de fichas próprias. A presença de uma seção no catálogo não prova que seus fluxos estejam auditados.

## 5. Desktop — superfície raiz AUDITED; widgets/inspector/faces pendentes

Arquivo: `desktop/Desktop.qml` (529 linhas).

- Uma `PanelWindow` full-screen por output, namespace `impasto-desktop`, sem exclusive zone e com `ExclusionMode.Ignore`.
- Em repouso fica em `WlrLayer.Bottom`, abaixo das janelas; ao organizar widgets sobe para `Top`.
- Apenas o output que iniciou a organização recebe keyboard focus `OnDemand`; perder ativação encerra o modo.
- A superfície cobre a tela para que drag não seja cancelado ao passar pela faixa da barra/dock.
- A geometria útil do grid é descontada dos insets de barra e dock.
- Clique direito no fundo abre menu contextual; clique esquerdo fecha menu e, em edição, limpa seleção.
- Escape encerra edição.
- Durante edição há grade, indicador de célula de destino, galeria/tray, Inspector e Picker carregados sob demanda.
- O card segue o ponteiro entre boards; enquanto algo está em mão ou Deck está arrastando, a tela não reivindica o card para evitar transferir um drag em andamento.
- Backdrop de wallpaper é uma segunda superfície, sem input, e só aparece depois que a nova layer do desktop foi sincronizada e desenhada. Isso evita cobrir os widgets com uma imagem atrasada.
- Menu contextual é uma surface separada no Top; não eleva o desktop inteiro sobre janelas normais.

**Ainda pendente:** Widget.qml, Inspector.qml, Picker.qml, Tray.qml, faces e variações analógica/sticker; drag/resize real, menus de cada widget e screenshots.

## 6. Dock — superfície raiz AUDITED; item/menu pendentes

Arquivo: `dock/Dock.qml` (670 linhas).

- Uma superfície full-screen por output, ancorada à borda inferior ou lateral selecionada; namespace `impasto-dock`.
- `exclusiveZone = 0`; o dock não retelha janelas. O DesktopService considera a zona do dock nos insets.
- A lista combina launcher, apps fixados e apps em execução; todas as telas exibem a mesma shelf, salvo modo que restringe ao output live.
- Input mask é calculada pela geometria da cápsula e cresce para o alcance dos ícones ampliados; clique fora da shelf continua chegando à janela de trás.
- Autohide usa um único progresso de 0–1 para mover cápsula e região de input; o gatilho de borda é um sliver. Ao sair, recolhe após **350 ms**.
- Magnificação usa curva de cosseno centrada no ponteiro; os ícones crescem e afastam vizinhos sem aumentar a cápsula. Desativa durante drag, menu ou organização do desktop.
- Tooltip mantém o último índice durante fade-out, é posicionado no lado interno e some durante drag/menu.
- Menu é identificado por key, não índice, porque a lista de janelas pode ser reconstruída quando apps abrem/fecham.
- Launcher é item separado: sem pinning/drag/indicador de janela; emite pedido ao shell para abrir o launcher na ilha.
- O menu ocupa a mesma superfície e expande sua input mask para a tela inteira, evitando problemas de ordem entre layer surfaces.

**Ainda pendente:** DockItem.qml, DockMenu.qml, ações de pin/unpin/close/launch, matching visual de ícones e validação de bordas/orientações.

## 7. Deck — superfície raiz AUDITED

Arquivo: `deck/Deck.qml` (800 linhas).

- Uma superfície full-screen por output, namespace `velatura-deck`, sem exclusive zone e sem keyboard focus.
- Em repouso usa `Top`; durante organização sobe para `Overlay`, para que o Desktop elevado não roube os presses das tabs.
- Apresenta strips finos nas bordas; hover revela tabs, e a tab sob o ponteiro mostra uma prévia da nota.
- Reveal anima por `Theme.durationMedium`; retração ocorre após **320 ms**.
- Clique abre a nota na ilha; menu de contexto permite abrir/organizar o deck.
- Durante edição, grip arrasta o deck ao longo da borda; tabs podem ser movidas/reordenadas, transferidas para outra borda, arrastadas para o grid para virar widget ou para o tray para remoção.
- Input mask contém somente tabs/strips e peek; ao segurar/arrastar/sliding ou abrir menu expande para tela inteira. Enquanto Desktop arrasta widget, Deck fica deaf para não roubar o ponteiro.
- Estado de press/drag/sliding é limpo ao trocar o modo e também no destruction dos grips, prevenindo máscara presa.
- `ScriptModel` e chaves estáveis evitam reconstruir tabs/grips em cada atualização de array durante drag.

**Ainda pendente:** screenshots, testes reais de drag/drop em todos os edges e verificação dos menus/inspector integrados.

## 8. Lock screen — raiz e composição AUDITED; componentes auxiliares pendentes

Arquivos: `lock/LockScreen.qml` e `lock/LockSurface.qml`.

- `WlSessionLock` usa ext-session-lock; cada output recebe uma `WlSessionLockSurface`. A sessão só conta como segura depois que todas as superfícies necessárias estão presentes.
- `LockService.secure` acompanha o secure state do protocolo.
- Cada superfície mostra screenshot do próprio output, desfocada enquanto o lock está ativo; o blur relaxa quando o lock está saindo.
- O estado awake faz clock subir, account field e power controls aparecerem. Clique/tecla acorda; movimento real do ponteiro acima do limiar também acorda e pode disparar tentativa de face.
- Ao voltar a repouso, o campo de senha é limpo.
- Se media está disponível e a preferência permite, o relógio muda de escala/posição e aparece player; opcionalmente a capa fornece o wash desfocado de fundo.
- Indicador de bateria aparece somente se BatteryService estiver disponível.
- Account mantém foco para que a primeira tecla possa ser a primeira letra da senha; submit sobe ao LockService/PAM.
- LockPower só fica visível no estado awake.

**Ainda pendente:** LockAccount, LockClock, LockMedia, LockPower, LockIsland e componentes de face; estados de erro/autenticação e comparação visual. Testes de senha devem permanecer isolados e não podem ser simulados com credenciais reais nesta auditoria.

## 9. Capture — overlay raiz lido; fluxo detalhado ainda pendente

Arquivos: `capture/CaptureOverlay.qml` e `capture/CaptureBar.qml`.

- Overlay é uma `PanelWindow` na camada `Overlay`, namespace `velatura-capture`, keyboard focus `Exclusive`, com `ExclusionMode.Ignore`.
- Trabalha sobre screenshot estático capturado pelo backend, não sobre uma cena congelada; o crop usa os mesmos pixels.
- Modos de shape: region, window e screen. Teclas 1/2/3 alternam os modos; Escape cancela; Enter/Return/Space confirma.
- Region desenha seleção retangular, quatro áreas de wash fora da seleção, outline e label de dimensões. Screen mostra outline de tela inteira.
- Window tenta atualizar a seleção pelo método `root.windowUnder(mouse.x, mouse.y)`, mas não há declaração desse método em CaptureOverlay.qml nem outra ocorrência/definição no checkout ativo (busca global por `windowUnder`). **Candidato a bug do original / símbolo ausente**: precisa ser confirmado no runtime e corrigido no contrato do porte sem inventar o comportamento esperado.
- Clique esquerdo confirma window/screen; em region, press-drag-release confirma somente se largura e altura forem pelo menos 4 px. Clique direito cancela. Uma região menor que 4×4 é descartada.
- A barra de opções separa shape (Region/Screen), kind (Photo/Video), destino (Save/Copy e opções condicionais Annotate/Read text) e áudio de sistema para gravação quando há monitor source.
- Opções de ferramenta indisponível são omitidas quando `CaptureService.offers()` não retorna suporte.
- Overlay permanece como instância do shell e só fica visible quando CaptureService está ativo.

**Ainda pendente:** leitura/contrato completo de window picking, drag/crop, coordenadas, ações de confirmação e cancelamento, tooltips e teste isolado de captura/gravação.

## 10. Não confundir AUDITED com VERIFIED

Nesta rodada não foram realizados:
- screenshots de referência ou comparação pixel a pixel;
- cliques/teclas automatizados ou teste de drag;
- teste de hotplug real;
- medição de timing em runtime;
- teste de layer-shell no Niri;
- teste de captura ou lock com efeitos reais.

Nenhum arquivo original em `~/.config/quickshell` foi alterado. Nenhuma preferência, dependência, serviço, módulo, Niri config, monitor virtual ou processo foi modificado.

## 11. Próximas fatias obrigatórias

1. Barra: módulos, workspaces, tray, chips, OSD e notification layer.
2. Dynamic Island: ControlsPanel e todos os detalhes/painéis; Controls blocks e TopRow.
3. Settings: cada uma das 14 seções, tabs, dialogs, import/export e ações cruzadas.
4. Desktop: Widget, Inspector, Picker, Tray, todas as faces e componentes analógicos/sticker.
5. Dock: DockItem e DockMenu.
6. Lock: auxiliares de clock/account/media/power/face.
7. Capture: fluxo window/region, barra completa e geometria.
8. Screenshots comparativos por output/scale/tema, e testes de interação em sessão controlada.

Cada fatia deve registrar arquivos completos, estados, ações, serviços, settings, geometrias/tokens, animações, erros, ciclo de vida e critérios de regressão antes de qualquer implementação correspondente.

## 12. Control Centre — grid e interação AUDITED; faces individuais pendentes

Arquivos lidos integralmente nesta fatia: `IslandState.qml`, `ControlsPanel.qml`, `controls/ControlsTray.qml`, `TopButtons.qml`, `Block.qml`, `BlockFace.qml`, `BlockInspector.qml` e `TopRowInspector.qml`.

### Arbitragem do estado da ilha

`IslandState` define prioridade estrita: **panel > notification > OSD transient > summary > modules**. Só a instância live arbitra; as demais ficam em repouso.

- OSD dura **1800 ms**. Se chega enquanto um painel está aberto, é descartado, não enfileirado.
- Abrir painel encerra transient, descarta/dismiss a notificação atual sem marcá-la como respondida, e remove summary.
- Perder o estado live limpa painel, summary e transient e para o timer de expiração.
- Um painel aberto fica até dismiss; notificação até expirar/ser fechada; OSD até expirar; summary enquanto o ponteiro permanece.

### ControlsPanel e grade

- A linha superior aparece quando há botões configurados; se vazia, aparece como espaço de configuração somente em modo edit.
- Top row tem dois lados e é desabilitada durante edição para que o clique selecione a linha, não acione um botão.
- A grade é publicada em `ControlsService.board`; a tray fica fora do painel, na superfície da barra, e converte coordenadas para as células do grid.
- Botão direito no fundo alterna edição; clique esquerdo no fundo limpa seleção/inspector.
- Escape encerra edição primeiro; se não está editando, o evento sobe para a ilha fechar.
- Destruir ControlsPanel encerra edição e limpa a referência `board`, evitando reabrir em estado que capture input.
- A grade desenha lattice e landing preview; o preview é posicionado diretamente na célula e anima somente entre células.
- Os blocos são keyed por ID estável: reordenar/resize não recria o conteúdo, preservando playback e scroll.

### Blocos, drag e resize

- `Block` lê o row atual no serviço; a posição vinculada é suspensa durante drag e restaurada após.
- Em modo edit, click seleciona e abre inspector; drag move o bloco; o destino usa nearest-free; drop sobre a tray remove o bloco.
- Handle inferior direito redimensiona entre tamanhos suportados; roda do mouse/touchpad também cicla tamanhos em passos de 120 unidades e pausa de 250 ms.
- Badge superior esquerdo remove; handle e badge aparecem em hover/seleção, com fade rápido.
- Durante edição o face interno é desabilitado, impedindo que o drag também acione controles internos.
- Inspector tem 268 px, é colocado à direita, à esquerda ou abaixo conforme espaço, rola se exceder a altura do board e oferece Remove, tamanhos por footprint e configuração de listas para os blocos Toggles/Shortcuts.
- Linhas de toggles/shortcuts podem ser ativadas/desativadas e reordenadas; os arrows consomem o press mesmo quando no limite, para não disparar toggle acidental.

### Tray de blocos

- Agrupa catálogo em Controls, At a glance e Personal.
- Cada item é mostrado em seu menor tamanho suportado; clique adiciona, drag gera ghost na camada host e calcula nearest-free; soltar na grade adiciona na célula; soltar sobre a própria tray cancela.
- Tray publica o card para o serviço e limpa `tray`/landing no destruction.
- O conjunto de block IDs vem do catálogo ControlsService e o mapeamento visual é explícito em BlockFace: toggles, volume, brightness, appearance, media, weather, calendar, notifications, velatura, pet, clock, games, notes, tasks, session e shortcuts.

### Top row e ações de sessão

- TopButtons são catálogo dinâmico. Ação comum executa imediatamente; ação de sessão destrutiva arma primeiro, expande o botão, fica vermelha/rotulada e só executa no segundo clique.
- O estado armado expira em **3000 ms**. Executar ação de sessão fecha o painel depois de chamar SessionService.
- Itens não-session abrem painel, ou solicitam Settings quando não há painel.
- TopRowInspector permite marcar/desmarcar cada item, mover entre lados e reordenar com arrows. O editor de Settings usa outro fluxo (drag via TopRowEditor), que ainda precisa ser auditado.
- O TopRowInspector limita altura ao espaço disponível e usa Flickable para a lista.

**Ainda pendente:** auditoria dos 16 block faces (cada estado/ação e serviço), dos 15 doors/18 toggles apresentados, do conteúdo visual de cards, jogos/pet/media/calendar/notifications e dos fluxos equivalentes na Settings UI. Esta ficha cobre a estrutura e o mecanismo de composição/edição, não cada face.

## 13. Módulos da barra — registry/chip e primeira amostra AUDITED

Arquivos lidos integralmente nesta fatia: `bar/modules/Module.qml`, `ChipFace.qml`, `ClockModule.qml`, `VolumeModule.qml`, `NetworkModule.qml`, `BatteryModule.qml` e `bar/island/DetailFace.qml`.

### Registry e chip comum

- `Module.qml` faz o dispatch por ID via Loader; o catálogo visual lista 22 módulos: clock, calendar, media, timer, claude, codex, battery, volume, brightness, network, bluetooth, weather, github, markets, screentime, stats, updates, pet, games, notes, tasks e notifications.
- Compact face é usada quando o módulo tem ring (`ModuleService.ringed`); caso contrário o chip usa símbolo e valor.
- `ChipFace` é reutilizado na barra, nos tiles de Settings e no preview da barra; portanto tamanho, fonte e figura precisam ser implementados como componente compartilhado no porte.
- Valores usam dígitos tabulares e largura calculada para que o relógio/percentual mudando não empurre os módulos vizinhos. O texto é recortado durante a animação de reveal e só aparece após 20% do reveal.
- Claude, Codex e Pet desenham marca própria em vez de depender de glyph de fonte.
- Ring é usado somente se o módulo estiver no catálogo `ringed`; quando o chip está sozinho, escala/reveal diferem do módulo encaixado na cápsula.

### Módulos inspecionados

- **Clock:** é a face de repouso da ilha, não um chip com slot. `SystemClock` usa precisão Seconds somente quando a preferência de segundos está ativa; o formato de hora acrescenta `ss` sem alterar o formato base. Data é opcional e usa formato abreviado `ddd d MMM`.
- **Volume:** ring mostra volume e ícone do sink; mute zera o progresso e muda glyph/cor. Detalhe mostra sink, mute, volume de saída, microfone, streams por aplicação com controles próprios e escolha de output quando há mais de um. O ring/mark também alterna mute.
- **Network:** chama refresh ao ser instanciado; detalhe mostra nome da conexão, linha de estado, estado Wi-Fi e toggle. Fact Link distingue wired/wireless/none e internet reachability. Seleção de redes fica no painel próprio, não neste cartão.
- **Battery:** chip reutiliza BatteryWidget. Detalhe mostra percentual, state word, estimativa (quando chega), potência, energia/capacidade e health somente quando o hardware reporta health; não inventa valor ausente.

### DetailFace

- Rede e Bluetooth não abrem um ModuleCard genérico: usam NetworkDetail/BluetoothDetail dentro do próprio módulo expandido, sem botão Back interno, e fecham pela ação de retorno do host.
- Calendar reutiliza o CalendarCard e o estado mensal compartilhado com Controls, evitando duas instâncias de calendário.
- Media troca para LyricsDetail somente quando LyricsService tem lyrics disponíveis; o tamanho da ilha cresce para o conteúdo de lyrics.
- Os demais IDs usam `Module(moduleId, compact=false)`, ou seja, o mesmo registry entrega o cartão expandido.

**Ainda pendente:** os outros 18 módulos, IslandRest/IslandSummary e os componentes compartilhados ModuleCard/CardLevel/CardFact/RingIndicator; os painéis de detalhe e as preferências de forma/ordem ainda precisam de fichas por módulo. Esta amostra não permite marcar Bar module UI como AUDITED.

## 14. Bar modules — catálogo completo AUDITED (22/22)

Todos os 22 arquivos do registry foram lidos, além de `Module.qml`, `ChipFace.qml` e `DetailFace.qml`. O contrato do porte não deve reduzir os módulos a um texto genérico: cada um tem compact face, detail face, ciclo de subscription e ações diferentes.

| ID | Serviço/integração | Contrato de UI e ações |
|---|---|---|
| `clock` | SystemClock + SettingsService | Face de repouso da ilha; formato configurável, segundos só quando habilitados, data opcional. |
| `battery` | BatteryService | Ring de carga/estado; detail com percentagem, estimativa e ausência de bateria tratada. |
| `volume` | AudioService | Ring de volume; clique no mark alterna mute; detail controla sink, streams por app e outputs. |
| `brightness` | BrightnessService | Ring de nível; detail atualiza dispositivos dimmable e oferece slider por ecrã. |
| `network` | NetworkService | Estado de conexão; refresh no detail, toggle Wi-Fi; seleção de redes fica em NetworkDetail. |
| `bluetooth` | BluetoothService | Nome/resumo e estado do adapter; toggle no detail; pairing/dispositivos ficam no Control Centre / BluetoothDetail, sem PIN agent. |
| `weather` | WeatherService | Temperatura/glyph e forecast; detail usa horas futuras/amanhã e dados de sensação/alta/baixa. |
| `media` | MediaService, CavaService, LyricsService | Artwork, ring de loudness e spectrum; detail com seek somente se seekable, transport conforme capabilities e lyrics lazy enquanto aberto. |
| `calendar` | CalendarService, TasksService | Mês com semanas começando segunda, navegação mês, seleção de dia, pontos de tarefas/eventos, abrir calendário e criar evento. |
| `timer` | TimerService | Idle configura h/m/s e preview; running exibe progress/endsAt, hold/resume, restart e stop; clicar no chip não cancela. |
| `claude` | ClaudeService | Session/week/model limits, resets, tokens/messages dos transcripts; fallback para contagem local quando limites OAuth não conhecidos. |
| `codex` | CodexService | Janelas de uso e resets; só considera janela corrente para o gauge, dado lido dos logs locais. |
| `github` | GithubService | Total anual, streak, idade do dado e contribution grid; subscribe/release por lifecycle. |
| `markets` | MarketsService | Primeiro par lidera preço/change/sparkline; restantes em linhas; idade do cache/rede explícita. |
| `stats` | StatsService | CPU ring com thresholds 70/90%, memória e sparklines; detalhes completos no StatsPanel. |
| `screentime` | ScreenTimeService | Total do dia, comparação com média, distribuição por grupos/horas e top 3; follow/unfollow por lifecycle. |
| `updates` | PackagesService, UpdatesService | Atualizações disponíveis e check manual; Open muda PackagesService.view para updates antes de abrir packages. |
| `games` | GamesService | Total de rounds, catálogo e melhores recentes; Play abre painel, nenhum jogo roda dentro do chip. |
| `pet` | PetService | Espécie/nível/progresso, mood e shelf; Feed/Play bloqueados pelo cooldown; hop/bob pausa quando asleep/game mode. |
| `notes` | NotesService | Últimas três notas e idade; New cria, Open abre, linha abre note no NotesPanel. |
| `tasks` | TasksService | Pendentes/atrasadas e próximas tarefas; New/Open/linha levam ao BoardPanel na tarefa escolhida. |
| `notifications` | NotificationService | Histórico retido, DND, default action e critical; abre a notificação ainda ativa, remove uma ou limpa histórico. |

### Regras transversais de módulos

- `compact` é uma apresentação distinta, não simplesmente o mesmo card encolhido. Alguns módulos não têm compact face e não são elegíveis para barra (ex.: GitHub é desktop-only).
- `ChipFace` mede figura, ring, forma, reveal e largura; a mesma implementação é usada em Settings preview e na barra real. A largura/reveal deve manter a estabilidade dos vizinhos.
- Serviços que têm `subscribe/release` são ligados pelo lifecycle do componente; não transformar cada widget em polling independente. Media usa também Cava e Lyrics com lifecycles separados.
- Network/Bluetooth e vários módulos de ação encaminham para painel próprio; o detail não deve duplicar a navegação que pertence ao host.
- Os quatro módulos que já tinham fichas específicas (Clock, Volume, Network e Battery) permanecem como referências de maior profundidade; os restantes foram lidos e adicionados ao contrato acima.

## 15. Dynamic Island — painéis e detalhes internos AUDITED

O roteamento do host foi cruzado com os componentes efetivamente existentes. IDs de painel são contratos IPC/UI, não nomes livres.

| Painel/detalhe | Serviço(s) | Comportamento observado |
|---|---|---|
| `appearance` / `palette` | WallpaperService, ThemeService | Três páginas deslizantes (animated, wallpaper, palette); Left/Right percorrem carousel, Up/Down troca página, Enter aplica, Home/End e PageUp/PageDown navegam. Faz scan ao abrir; tiles distantes não decodificam imagem. |
| `auth` | PolkitService, AccountService | Campo de senha com echoMode condicionado por responseVisible; submit limpa o campo; erro anima shake; Caps Lock aparece; Cancel e destruction cancelam a solicitação pendente. Nunca persistir senha. |
| `controls` | ControlsService | Grid editável com top row, tray, inspector, drag/resize e nearest-free; detalhe na seção 12. |
| `board` | TasksService | Kanban de três lanes; foco inicial em New; setas percorrem lanes/cards, Enter abre, Space muda state, drag/drop reordena e Escape volta do editor para o board. |
| `camera` | CameraService, SettingsService | Preview lê o dispositivo virtual de volta; mirror local, shutter snapshot, reações por gesture, um efeito por vez via carousel; FPS/faces/consumidores e status ficam visíveis. |
| `games` | GamesService | Shelf com seleção teclado e pointer; game carrega só quando ilha alcança tamanho-alvo; R reinicia, Enter/Space reiniciam após fim, Escape volta à shelf; record/best e feedback de novo record pertencem ao frame. |
| `launcher` | LauncherService, ClipboardService, EmojiService | Prefixo determina modo; query vive no service para sizing antes do Loader; refresh no open, limpa query no destruction; lista/grid tem navegação própria, Tab/Backtab alterna grupos/tone e Delete pode remover entrada de clipboard. |
| `machines` | VmService | Duas views (machines/new); lista mostra recursos/estado; detalhes incluem recursos e snapshots; catálogo filtrável por OS/release/edition, configura cores/memória/disco; remoção exige armar; lifecycle subscribe/release. |
| `notes` | NotesService, DesktopService | Grid de notas com New, teclado, arquivadas e indicador de placement; editor usa papel full-bleed, grava título/texto por edição, suporta Shift+Tab/Up para título, troca tint, Archive/Restore/Delete. |
| `packages` | PackagesService, UpdatesService | Views find/installed/updates; busca e seleção por teclado, Tab/Backtab muda seleção, install/remove/upgrade e refresh; ação pode ser longa e não pode bloquear GTK. |
| `screentime` | ScreenTimeService | Report day/week/month, barras empilhadas, ribbon de sessões, heatmap weekday×hour, Apps/Groups, hover readout e drill-down de bucket para day. Empty state explica que conta a partir do uso da sessão. |
| `overview` | NiriService | Workspaces por output com janelas; teclado seleciona, Enter foca workspace/janela conforme seleção; click seleciona, double-click ativa; seleção é corrigida quando a lista muda. |
| `stats` | StatsService | CPU, memória, swap, rede, uptime, cores, disks e temperaturas; disponibilidade/valores desconhecidos devem continuar distintos de zero. |
| `servers` | ServersService, OsdService | Lista por port; row abre URL, copy emite OSD, stop desabilita concorrentes e mostra Stopping; status mostra erros e listeners expostos na rede. |
| `session` | SessionService | Layout keyboard-first; arrows movem; destrutivos armam em vermelho por 3 s, mover desarma; fecha a ilha antes de executar lock/reboot/shutdown etc. |
| `tray` | TrayService | Menu DBusMenu com stack de submenus; setas movem, Right entra, Left volta, Enter executa; reset stack no destruction. |
| `keys` | ShortcutService | Catálogo read-only; não editar keybinds no shell porque o original delega a edição ao NiriMod. |
| `wifi` | NetworkService | Scan ao abrir, lista signal, toggle Wi-Fi, senha inline, Escape cancela editor, connect/disconnect; conclusão atualiza status e feedback. |
| `bluetooth` | BluetoothService | Discovering ativo enquanto painel vive; toggle controla adapter/discovery; lista de dispositivos, connect e opção de mostrar unnamed; não há PIN agent. |
| `sound` | AudioService | Slider de sink, mute, streams por aplicação, escolha de output quando há vários; Back retorna ao Controls. |
| `microphone` | AudioService | Slider/mute da source, streams e escolha de input; Back retorna ao Controls. |
| `brightness` | BrightnessService | Um slider por display dimmable; módulo só mostra expansão se há mais de um display. |
| `nightlight` | SunsetService | Temperatura e toggle; presets Kelvin e Back. |
| `lyrics` | LyricsService, MediaService | Lyrics lazy, sincronização com current line e seek por verso; progress seekable e controls transport; scroll segue a linha quando sincronizado. |
| `pet` | PetService | Ações Feed/Play e shelf de espécies; animação reage a played/celebrated/brought. |
| `notes` / `board` | NotesService / TasksService | São editores com lifecycle próprio; não os substituir por simples listas CRUD. |
| `packages`, `machines`, `screentime` | Serviços respectivos | Painéis grandes com estados de loading/error/empty e ações de sistema; manter as mensagens de dependência/falha e o conteúdo scrollable. |

### Host e detalhes compartilhados

- `DetailFace` escolhe NetworkDetail/BluetoothDetail diretamente, CalendarCard compartilhado, LyricsDetail quando há letras, e Module expandido para os demais IDs.
- O host decide dimensões antes de criar o conteúdo; painel e serviço podem precisar ser preparados antes do primeiro frame. Não criar painel em tamanho zero e esperar que o conteúdo se auto-ajuste depois.
- OSD, notificação e painel têm prioridade diferente; o estado do host não pode ser inferido só pela visibilidade de um filho.
- Teclado pertence ao FocusScope ativo; ao criar Loader, alguns painéis chamam forceActiveFocus no item carregado explicitamente.
- Painéis com requests de Settings fecham/assentam a ilha antes de transferir a janela. Polkit cancelado no destruction precisa responder No para não deixar requester pendurado.

## 16. Control Centre — 16 block faces e conteúdo AUDITED

O registry real é: `toggles`, `volume`, `brightness`, `appearance`, `media`, `weather`, `calendar`, `notifications`, `velatura`, `pet`, `clock`, `games`, `notes`, `tasks`, `session`, `shortcuts`.

| Block | Face e contrato |
|---|---|
| Toggles | Tiles derivados de ControlsService; pages = ceil(tiles/(cols×rows)); wheel vertical/horizontal acumula detents de 120, swipe horizontal, chevrons e dots; tile chama activate, abre painel ou dismiss conforme catálogo. |
| Volume | SliderRow, mute no ícone, expande para sound; indisponível quando AudioService não está ready. |
| Brightness | SliderRow, mínimo 1, refresh ao montar; expande só com mais de um display. |
| Appearance | Wallpaper atual + palette swatches; clique pede appearance panel. |
| Media | Card com artwork/metadata e transport controls. |
| Weather | Layout adapta cols/rows e previsão; mantém unknown/no forecast distinto. |
| Calendar | Month grid, events/tasks, dia selecionado, navegação e wheel; abre calendário/dia. |
| Notifications | Histórico e DND, abre ação default se ainda disponível, remove item e limpa. |
| Velatura | Resumo do sistema/máquina (CPU, uptime, disk etc.) a partir de Machine/Stats/Account services. |
| Pet | Face e ações de pet dimensionadas pelo número de rows. |
| Clock | Hora/data com dimensões derivadas de cols/rows e clock format. |
| Games | Últimos jogos e melhor; abre GamesPanel. |
| Notes | Sticky note compartilhada; click abre nota/editor. |
| Tasks | Resumo e fila; abre BoardPanel na tarefa. |
| Session | ButtonsBlock com ações de sessão; destrutivos armam por 3 s, executam e dismiss. |
| Shortcuts | ButtonsBlock dos doors selecionados por bloco; abre painel ou Settings; não inventar binding editing. |

### Block/inspector/tray

- O tamanho da face vem de `ControlsService.parse(size)`; o componente recebe cols/rows e alguns blocos escolhem layout alternativo.
- BlockInspector altera footprint, lista de toggles/shortcuts e ordenação; remove e reconfigura o bloco. Ações de reorder no limite continuam consumindo o press para não acionar toggle.
- ControlsTray lista catálogo por grupos, apresenta cada bloco no menor footprint suportado, clique adiciona, drag mostra ghost/nearest-free e soltar na tray cancela.
- GridSize é o mesmo picker usado em Settings e no painel; hover pré-visualiza colunas/linhas, click chama resize; o mínimo é 2×2.
- TopButtons usa SessionService para ações comuns e destructive com segundo clique, timeout de 3 s; TopRowInspector reordena/move entre lados. Settings usa TopRowEditor separado, ainda registrado como integração equivalente a conferir.

## 17. Settings — 14 seções, tabs e ações cruzadas AUDITED

O menu principal contém exatamente 14 seções: Bar, Island, Control Centre, Launcher, Dock, Desktop, Modules, Appearance, Windows, Displays, Input, Keys, Session e System. A janela e a navegação raiz estão descritas na seção 4; abaixo estão os contratos das páginas e suas diferenças importantes.

| Seção | Subpáginas / conteúdo | Mutação e fluxo |
|---|---|---|
| Bar | Style, Layout, Workspaces | Estilos grouped/spread/island/bezel; corners/sides/every screen; altura/margens; surface/shadow; workspaces count/style; LayoutEditor edita listas left/right. |
| Island | Position, Clock, Beside the time, Notifications | attached/detached; summary on hover; clock format/date/seconds; chips de privacy/network/etc por SettingsService.setBeside; duração de toast e DND. |
| Control Centre | Grid size, edit/default layout, Top row | GridSize chama ControlsService.resize; Edit fecha Settings e abre ilha em edição; Restore usa layout default; TopRowEditor reordena botões. |
| Launcher | Results, Sigils, Clipboard | Ordem recent/alphabetical, 4–14 resultados, fit/fixed; sigils únicos de um caractere não alfanumérico; clipboard history/keep count/images/wipe on lock. |
| Dock | Edge, alignment, screens, icon size, magnify, ground, opacity, shadow, launcher/running/autohide, kept apps | Dock pode estar desligado, mas lista pinned segue editável e alimenta launcher; drag reorder é feito no dock, add/remove na Settings. |
| Desktop / Widgets | Edit, face, ground, opacity, shadow, notes/spectrum visibility | Edit fecha Settings e passa screenName; face modern/analogue/sticker; widgets têm override próprio no Inspector; Deck e Spectrum podem sair quando há janela. |
| Modules | Weather location, GitHub user, markets symbols, pet style, lyrics lookup, notes handwriting | Dados de módulos são configurações da máquina/perfil conforme SettingsService schema; valores são trim/normalizados no set. |
| Appearance | Wallpaper, Terminal, Type, Motion | wallpaper folder, Matugen, transições awww, terminal ground/greeting, fontes fontconfig sans/mono, Nerd Font hint, game mode e motion presets. Wallpaper/palette picker abre painel appearance em vez de duplicar UI. |
| Windows | Capacidades nativas do Niri e NiriMod | Página informativa: scrolling columns, gaps/borders, corners/shadows, blur, animations/gestures, window rules. Sem controles falsos para recursos não suportados; KDL edit via NiriMod. |
| Displays | Display config path, kanshi profiles, Niri output settings | Apenas documentação e ação NiriMod; shell não reescreve output KDL/kanshi por conta própria. |
| Input | Keyboard/pointer, xkb, repeat, touchpad/cursor | Informativa; NiriMod edita KDL preservando includes/comentários. Shake-to-find declarado não suportado. |
| Keys | 32 shell actions + Niri binds | Só mostra "Set in Niri KDL"; ShortcutService não fornece bindings efetivos/editáveis. NiriMod é a rota. |
| Session | Lock, Face unlock, idle | Account name/picture; lock clock stacked/inline/flip; music off/player/lyrics e ground cover/desktop; blur; face setup/enroll/remove/trial; idle lock/screen off/suspend. |
| System | Profiles, machine, updates | Reset só perfil ativo, machine language/hardware info, update checkout e diff/restore de arquivos editados/deletados. |

### LayoutEditor (layout da barra)

- Catálogo é derivado de ModuleService.catalogue e separado em gauges, readings, buttons e workspaces/tray/split. Somente módulos com `entry.bar` entram.
- A mesma peça pode aparecer duas vezes; seleção é por side+index, não por ID.
- O editor desenha a barra real e o catálogo; um MouseArea central faz hit-test, segura uma cópia e grava as listas somente no release. Isso evita destruir o delegate que está sendo arrastado quando o gap se move.
- Drag do catálogo para o próprio catálogo não altera nada; click no catálogo acrescenta ao fim da direita; drop fora mantém listas originais; drop sobre metade da barra decide left/right; move limpa seleção.
- Split, Workspaces e Tray são itens estruturais, não módulos; split quebra cápsulas, workspaces/tray terminam o grupo atual.
- Module look overrides shape/figure/when por instância; vazio significa herdar setting global. Botões/split não têm look customizado.
- Unavailable ou "only while running" aparecem com opacity 0.45; isso não os remove do catálogo.
- Layout editor é uma edição de duas listas e deve preservar ordem, duplicatas e divisão entre cápsulas; não pode normalizar removendo IDs repetidos.

### Perfis e arquivos

- ProfilesPart mostra thumbnail do wallpaper, summary do bar/dock/widgets e status In use; rename, duplicate, export, delete com confirmação; import/export passam por FileDialog e ProfileService.
- KeptApplications guarda desktop IDs, mantém entradas removidas do sistema para que sua posição não desapareça, e oferece busca em LauncherService.application index.
- FontPicker usa fontconfig; não aceitar família arbitrária como se instalada, porque Qt substitui silenciosamente.
- SystemSection update não é um updater de pacotes: opera sobre checkout da Velatura, deixa arquivos deletados/de editados explícitos, guarda .new/backups e bloqueia pull se há commits locais à frente. Atualização abre terminal interativo, não pode ser escondida em background.
- SessionSection face setup pode instalar helper, ligar PAM e fazer add/remove/trial; são operações privilegiadas e não devem ser executadas durante auditoria/port sem confirmação.
- Windows/Displays/Input/Keys são intencionalmente guidance-only; não implementar editor KDL interno onde o original delega ao NiriMod.

## 18. Desktop — widgets, inspector, picker e faces AUDITED

### Widget lifecycle e input

- Um delegate é keyed pela row key e lê `DesktopService.entryOf(key)`; não conhece conteúdo do módulo. ModuleService.watch/unwatch acompanha a troca de moduleId e o destruction; isso preserva playback/scroll durante move/resize.
- Família, geometry, ink, opacity e theme são resolvidos pelo DesktopService. X/Y usam Binding suspenso durante drag e restaurado ao terminar; width/height animam com durationMedium.
- Face fica disabled durante edição; clique seleciona/abre Inspector, botão direito em repouso abre menu, drag move; drop na tray remove, note na borda vira Deck, spectrum na borda vira EdgeSpectrum; demais drops fazem nearest-free no grid da mesma tela.
- Drag nunca atravessa diretamente de um output para outro; para mover entre telas, remove e adiciona na outra. A superfície Desktop pode transferir o card/tray para a tela sob o ponteiro.
- Wheel em edição cicla family por passos de 120 e cooldown de 250 ms. Resize usa handle diagonal; seleção/hover mostra badge remove e handle.
- `DesktopService.inHand` fica true enquanto drag/resize e evita o card mudar de tela durante a ação.

### Inspector

- Card largura 312 px, aparece à direita/esquerda/abaixo conforme espaço, preso ao board, Flickable se excede altura.
- Shape escolhe famílias suportadas por módulo e tema; Modern, Analogue e Sticker têm catálogos diferentes.
- Spectrum não é capsule: edita look, fill, lows orientation, peaks, colors, bar/gap/opacity; strip tem edge e height. Deck não tem shape e edita notes associadas/ordem/toggles. Photo edita picture e caption; o text field sinaliza DesktopService.typing e fecha foco quando surface perde teclado.
- Widget comum pode ter theme override, palette/ink e opacity override; Reset retorna a herança desktop. Remove é ação explícita.
- Notes, photo e spectrum são bare/on-picture; sticker não se aplica a esses tipos. Não aplicar capsule opacity/shadow a eles.

### Picker de fotos

- Pointer-only, sem input de path e sem search. Places: Pictures, Wallpapers, Downloads, Desktop e Home (sem duplicar Home); abre no diretório da foto atual ou Pictures.
- FolderListModel lista dirs primeiro, não mostra hidden, filtra extensões de imagem case-insensitive. Click em diretório navega; click em imagem escolhe e grava path; Cancel fecha sem alterar.
- O card se posiciona ao lado do widget se houver espaço, senão centraliza; o tap handler captura clicks para que o background não feche o picker.

### Faces e compatibilidade

`Face.qml` é o dispatcher:
- `notes` e `spectrum` são faces shared, independentemente do theme.
- `lyrics` sempre usa LyricsFace.
- Modern escolhe registry por family: `2x2 → Squares`, `4x2 → Wides`, `4x4 → Larges`, `8x2 → Bands`.
- Analogue e Sticker usam um registry único, com layout interno por family. Sticker recebe seed para determinismo visual; photo recebe row.
- Wides tem fallback para Module detail no tamanho de catálogo quando não existe face própria. Não assumir que todos os registries têm fallback idêntico.

| Registry | IDs explícitos |
|---|---|
| Squares (2×2) | battery, volume, brightness, network, bluetooth, updates, weather, github, screentime, markets, stats, claude, codex, timer, pet, games, media, clock, calendar, notes, tasks, photo |
| Wides (4×2) | battery, volume, brightness, network, bluetooth, updates, weather, github, screentime, markets, stats, claude, codex, timer, pet, media, clock, calendar, notes, tasks, photo, spectrum |
| Larges (4×4) | calendar, weather, stats, screentime, markets, media, claude, codex, notes, tasks, photo, spectrum |
| Bands (8×2) | clock, weather, github, screentime, markets, notes, photo, spectrum |
| Analogue / Sticker | clock, weather, github, markets, screentime, battery, stats, claude, codex, media, calendar, timer, volume, brightness, tasks, updates, network, bluetooth, pet, games, photo |

- O catálogo de famílias por módulo/tema é a autoridade para add/resize. A tabela acima só lista componentes desenhados; não substitui DesktopService.familiesFor().
- Modern fallback e faces ausentes devem ser tratados como capability, não renderizar um retângulo vazio. Em particular, 8×2 é strip/band, não 4×2 esticado.

### Spectrum no desktop

- SpectrumBars usa um único ShaderEffect + `spectrum.frag.qsb`, com 64 bandas em 16 vector4d uniforms; não criar 64 nós de cena por frame.
- Cava subscribe é contado por consumidor; sample usa valores sintéticos estáticos e não deve abrir Cava. Peaks são opcionais, listening respeita spectrumAwayOn/game mode; frame acompanha a taxa do Cava, não o refresh rate do monitor.
- EdgeSpectrum é uma superfície de ponta a ponta, não uma widget capsule. Em edição pode mover para outra edge, para grid (4×2) ou tray (remove); edge só aceita um spectrum por output.

## 19. Dock — item e menu AUDITED

- DockItem separa launcher, pinned app e running app. Clique primário ativa/abre; middle-click pode criar nova instância; right-click abre menu contextual; indicador de janela usa classes/janelas associadas.
- Drag de item reorganiza apenas pinned order e persiste via DockService; running-only items não se tornam pinned por serem arrastados sem ação de pin.
- Menu é keyed por item ID e consulta janelas atuais; inclui abrir nova janela, janela individual/foco, pin/unpin e fechar janela/app conforme capacidades. Menu se fecha se o item deixa de existir ou o alvo fica inválido.
- DockMenu não pode roubar o lifecycle da Dock surface: sua hit mask expande no menu e clique externo precisa chegar ao menu/host conforme original. Não converter o menu em janela xdg separada sem preservar z-order.
- KeptApplications em Settings compartilha a mesma lista do Dock e Launcher; manter desktop ID estável e entradas desinstaladas.

## 20. Lock — componentes internos AUDITED

- LockAccount é um único pill preto que expande de avatar+nome para password field; TextInput segura keyboard desde o início, passwordMaskDelay=0, submit via Enter/botão, spinner enquanto PAM autentica, refusal shake e mensagem. Escape limpa campo e volta clock; qualquer tecla que acorda a tela não pode ser também a primeira tecla da senha.
- Campo é sincronizado com LockService.typed entre superfícies; ao sair de awake, LockSurface chama account.clear. Senha não pode ser persistida/logada/serializada e deve ser apagada após submit e failure.
- LockClock tem stacked/inline/flip; split usa SettingsService.clockFormat, fonte tabular e date. O flip mantém delegates por count para dobrar o cartão existente.
- LockMedia pode mostrar cover, title/artist, progress/seek, transport e lyrics sincronizadas; lockMusic controla off/player/lyrics e lockMusicGround controla cover/desktop. Lyrics só carregam quando habilitadas e disponíveis.
- LockPower aparece só awake; power actions são confirmação/destrutivas no menu e respeitam lock/logind lifecycle. LockIsland mantém estado de face unlock e não é só decoração.
- LockSurface captura screenshot do output sem cache, desfoca enquanto held, pode usar cover wash de baixa resolução atrás da música, bateria só se disponível; clock/account/player reposicionam por altura real para não sobrepor.
- Testes de lock/auth não podem usar credenciais reais nem invocar PAM durante auditoria.

## 21. Capture — interação e geometria AUDITED; modo Window BLOCKED

- CaptureBar tem três grupos: shape (region/screen), kind (photo/video), destino (file/clipboard + editor/text só se CaptureService.offers), e áudio do sistema para vídeo apenas quando monitor source existe. Não é microfone.
- Pointer handler captura left/right; right cancela. Region começa no press, atualiza rect com min/max e confirma no release somente com width/height ≥4 px; clique curto descarta retângulo. Screen usa rect zero e backend interpreta como tela inteira.
- Window deveria atualizar seleção por hover e confirmar com click, mas a chamada `root.windowUnder(mouse.x, mouse.y)` não tem definição encontrada. **BLOCKED para paridade funcional** até reproduzir no original/recuperar helper; não adivinhar Niri window geometry. Enquanto isso a seleção e click path não são especificáveis.
- Label de dimensões fica acima da seleção, mas passa para dentro quando a seleção toca o topo; clamp horizontal. Wash cobre as quatro regiões externas. CaptureBar desaparece durante drag e tem z acima do pointer para não iniciar crop ao clicar nas opções.
- `take()` não chama backend se region menor que 4×4; screen passa 0,0,0,0; seleção não é normalizada por escala manual aqui, o backend recebe coordenadas da superfície e deve manter o mesmo output/pixel transform.
- Não houve screenshot real, captura, gravação, OCR ou teste de coordenadas; os contratos são estáticos.

## 22. Ilha em repouso, summary, OSD, notifications, workspaces e tray AUDITED

### IslandRest

- Clock mantém centro; activities são distribuídas nos dois lados. Uma atividade só é dividida em mark à esquerda e figure à direita; duas atividades ficam uma por lado; overflow aparece como +N e é mostrado no summary.
- Activities conhecidas: recorder, privacy, timer, media, machine e workspace. Privacy não recebe pointer/click; o restante aciona o serviço/painel. Recorder é um stop affordance: dot pulsa, hover quadrado, click encerra.
- Media mark adquire/release MediaService; spectrum adquire/release CavaService. Timer mark é ring com estado paused; machine mostra VM running count/uptime; workspace anima label de troca.
- Se activities não cabem, clock encolhe para clockCore e overflow ocupa slot trailing. Não truncar removendo atividade silenciosamente.

### IslandSummary

- Uma face de maior prioridade escolhida por ModuleService.glanceFace, faixa de chips para o restante e clock/data no lado oposto.
- Sem atividade, weather vira face; sem weather, mostra semana. Media usa artwork e pode substituir value pela linha de lyrics sincronizada; timer mostra progress/controls; privacy mostra mic/camera/screen e who, sem ação; recorder pode parar; machine abre Machines.
- Só o quadrado, controls e chips são clicáveis; clique no restante abre Controls. LyricsService subscribe/release acompanha summary.
- As ações do summary devem ser consistentes com ModuleService.activate e com a lista de activities; não criar uma segunda regra de prioridade em Rust.

### OSD e notifications

- OsdLayer é transient agregado: ícone/reading/level; battery pode não ter level; eventos não devem criar uma surface por dispositivo. Duração/aggregation/debounce pertence ao OsdService/IslandState.
- NotificationLayer renderiza picture→app icon→stand-in→glyph; critical recebe ring warning; inline actions ficam no row quando há largura, caso contrário em segunda linha. Click principal invoca default action; click em outra área fecha; action button invoca action específica. Styled body respeita Pango markup anunciado pelo servidor.
- Histórico é diferente do toast ativo: histórico pode mostrar texto sem default action viva; não executar action velha. Remoção/clear não deve chamar action.

### WorkspacesWidget

- Runtime mostra somente workspaces reais que Niri reporta para aquele output, ordenados por idx; Niri cria/remove dinamicamente, não existem slots vazios reservados. Settings preview usa 5 slots artificiais.
- Estados focused/occupied/empty são distintos; quatro formas geométricas (dots/bars/rings) e quatro estilos escritos (numbers/roman/kanji/greek); active pill desliza em styles escritos. Hover e click target ocupam o slot inteiro, não só o glyph.
- Click chama focusWorkspaceOn(workspaceId, screenName); não inferir número global entre monitores. O cálculo de número local é NiriService.numberOn.

### TrayWidget / TrayMenuPanel

- TrayWidget mostra até 5 itens sem expandir; acima disso deixa 4 e mostra +N; click alterna fold, click primário chama activate, middle chama secondaryActivate, right abre TrayMenuPanel, wheel encaminha scroll delta normalizado por 120.
- Layout editor preview usa `still=true`, sem input. Ícones são neutralizados/grey para não quebrar o material visual do shell.
- TrayMenuPanel usa DBusMenu stack, foco inicial, teclado Up/Down/Enter/Right/Left, back e reset da stack no destruction. Item disabled/separator não é selecionável; submenu entra no stack e seleção é resetada quando entries mudam.

## 23. Games e pet — UI/contratos de interação AUDITED

### GameBoard / GamesPanel

- GameBoard carrega somente quando o painel alcança o tamanho de catálogo (roomy); o item carregado recebe forceActiveFocus porque Loader é FocusScope. Cada game é FocusScope com score, over, tint, finished(score), restart(). O frame é dono do score strip, best, overlay de fim e botão de restart.
- R reinicia sempre; Enter/Return/Space reiniciam somente após round over. Escape pertence ao GamesPanel e volta para shelf. A melhor pontuação é registrada por GamesService.record quando finished dispara; jogos lower-is-better são marcados como Moves.
- 11 jogos do registry:
  - **Whack:** nove buracos, até dois moles; clique ou teclas 1–9; 45 s, misses custam 2 s e debounce 300 ms.
  - **Snake:** 20×20, setas/WASD, turn queue impede reversão, 10 pontos por apple e tick acelera 3 ms por apple até 55 ms; colisão encerra.
  - **Flood:** 14×14, seis cores, corner flood; score é número de movimentos, sem perder, finished só quando todo tabuleiro vira uma cor.
  - **Mini2048:** 4×4, setas/WASD, merge uma vez por movimento; tile 2 ou 4, fim quando board cheio sem merges; atingir 2048 não encerra.
  - **Lights:** 5×5, click inverte cruz; 6–10 toggles aleatórios iniciam tabuleiro solucionável; score é presses, fim quando tudo off.
  - **Hextris:** seis lanes em hexágono, setas giram, combinações adjacentes limpam; stack 9 termina; chain multiplica score e velocidade aumenta.
  - **Bots:** cinco lanes, click/dígito bash no robô mais baixo; 3 chegam ao chão termina; miss perde vida e há debounce.
  - **Target:** target encolhe, hit vale 1/2/3 pela distância; três vidas; miss perde uma, ritmo acelera com score; mouse-only.
  - **Space:** 5×4 invaders, esquerda/direita e Space; três vidas, até três shots, invulnerability após hit; ondas aceleram.
  - **Tetris:** 10×20, seven-bag, setas/WASD, up gira com wall-kick, down soft-drop, Space hard-drop; score padrão por drop/linhas e gravity acelera por level.
  - **Solitaire:** Klondike draw-one; click pick/drop, stock/waste e foundations; sem drag; score é moves e stuck state permanece até redeal.
- Burst e Pop são efeitos reutilizados, não parte do loop principal: instances persistem e reiniciam animação para evitar alocação por hit.

### PetPanel / PetFace

- PetPanel subscreve PetService; nome editável in-place, egg sem nome, level/XP/progress/mood; Feed/Play só habilitam com canFeed/canPlay. PetFamily troca espécie ativa.
- Idle bob pausa asleep e durante hop; played/celebrated/brought disparam hop. Game mode/lifecycle deve suspender animação e serviço conforme PetService já faz.
- O porte precisa preservar cooldown e gating no backend; não liberar botão apenas por UI. Feed/play/rename/species são comandos com estado confirmado pelo serviço.

## 24. Cobertura desta rodada e limites

- Arquivos raiz e componentes listados nas seções 14–23 foram lidos ou inspecionados por trechos de lógica, registries, handlers, mutações e serviços. A documentação registra contratos de fonte, não uma certificação de paridade visual.
- Não foram capturados screenshots, não houve pixel diff, drag automatizado, interação de teclado real, hotplug, teste de layer-shell, lock/PAM, capture, webcam, gamepad ou audio loopback.
- A chamada `root.windowUnder()` permanece o único bloqueio de fonte explicitamente encontrado para um fluxo de UI nesta rodada; outros recursos externos seguem com seus bloqueios de backend já documentados.
- Antes de implementar cada face, validar a lista de serviços consumidos contra APP_API_CONTRACT e acrescentar qualquer evento/command faltante; a UI não deve inventar estado local que já pertence ao service.
- A comparação visual deverá ser feita por output/scale/tema, com screenshot baseline de cada estado: repouso, hover, expanded, edit, loading, empty, error, notification, OSD e lock/capture em ambiente seguro.

## 25. Catálogo mecânico de componentes

Além desta auditoria semântica, foi gerado `UI_COMPONENT_CATALOG.md` a partir dos arquivos QML ativos. Ele indexa **303 componentes QML** nas famílias de bar/modules/island/controls/games/widgets, Settings, Desktop/faces (modern/analogue/sticker), Dock, Deck, Lock e Capture. Para cada arquivo registra linhas, serviços/estados referenciados, funções, signals e handlers de interação.

O catálogo serve para Codeína/backend localizar a fonte e cruzar dependências; não transforma referência textual em prova de execução e não substitui as fichas semânticas deste documento.

## 26. Contratos das faces individuais do Desktop

A leitura dos componentes compartilhados confirmou que cada tema é um sistema de composição próprio; não é só troca de cores.

### Modern

- `WidgetFace` define a grade comum: mark no canto superior esquerdo, label superior direito, reading ancorado embaixo e note/caption; wide reserva extraShare para sparkline/forecast/transport, large usa body entre mark e reading. Fontes horizontais reduzem até mínimo sem trocar altura do widget.
- `DayTasks` é shared entre famílias e temas: tasks do dia, pending count, scroll, TaskRow abre task no BoardPanel; back restaura calendário. Mantém o dia mostrado durante fade-out.
- `GithubFace` é apenas contribution wall (cells fixos, largura aumenta semanas); empty state distingue username não configurado de GitHub indisponível.
- `LyricsFace` é shared entre temas, subscreve/release LyricsService, current verse fica no centro e linhas próximas esmaecem; instrumental/unavailable/status não são confundidos com lista vazia.
- `NoteFace` é read-only no desktop; click abre NotesPanel porque a superfície desktop não toma keyboard. Usa Sticky/paper, sem capsule.
- `PhotoFace` é edge-to-edge; arquivo perdido é distinto de vazio; click abre no imv. Inspector escolhe o path e caption.
- `SpectrumFace` usa DesktopService.spectrumOf e SpectrumBars; respeita spectrumAwayOn exceto durante edição/tray preview.
- `WidgetFace` é layout base e não possui subscription própria; cada módulo recebe ink/family/row e é responsável pelo próprio serviço.

### Analogue

- `Instrument`, `Dial`, `Gauge`, `Arcs`, `Fader`, `Knob`, `Thermometer`, `Hourglass`, `Joystick`, `Leaf`, `Record`, `Recorder`, `Sky`, `Paper`, `Parcel` e `PhotoFace` formam a linguagem de papel/instrumento: tokens de ink vêm do DesktopService e devem ser preservados como composição, não achatados em cards Modern.
- CalendarFace tem 2×2 leaf do dia, 4×2 semana Monday-first e 4×4 mês; dias com tasks são clicáveis e abrem DayTasks; pointer leave/arrow retorna. Agenda line prioriza pendências de hoje, depois próxima task, depois weekday.
- GamesFace mostra último jogo/best e botão que abre GamesPanel.
- MediaFace tem cover/record, título/artista e transport; só mostrar controles permitidos pelas capabilities do player.
- TasksFace mostra count/overdue, lista de tasks e abre BoardPanel; TimerFace tem ring e start 5 min / toggle / cancel conforme running/paused.
- PhotoFace tem print instantâneo em famílias menores e film strip em 8×2; rotação é determinística por row key e click abre imv.

### Sticker

- `Cut`, `Dot`, `Ring`, `Spread`, `Sticker`, `Tag` e `Tone` são primitivos geométricos. Seed deriva da widget key para que recortes/lean permaneçam estáveis em redraw e dois stickers não repitam aleatoriamente a mesma pose.
- CalendarFace mantém a mesma navegação por dia/tarefa em linguagem de sticker; DayTasks é shared.
- GamesFace abre arcade por Dot Play; MediaFace oferece previous/play/next em dots, com layout diferente em 2×2/4×2/4×4.
- TasksFace mostra selo com pendentes/atrasadas e até 3/7 strips; clicar strip abre tarefa, empty strip abre board. TimerFace oferece start 5 min, pause/resume e cancel.
- PhotoFace e Notes são objetos de papel/sticker, não devem herdar capsule glass. Faces sem handlers são somente leitura e não precisam emitir comandos artificiais.

### Critério de paridade das faces

Cada implementação deve usar o registry correto e capability real do módulo. Estados de unavailable, not configured, no data, loading e stale são distintos; zero não substitui ausência. Faces interativas devem emitir o mesmo comando do serviço/painel original, não duplicar regras de negócio no widget.

## 27. Pet renderer e coleção — componentes AUDITED

- PetFace escolhe quatro styles (plush, pixel, paper, creature) via Loader; todas recebem o mesmo record/species/mood/egg/blink e usam um único relógio de blink. Settings preview pode renderizar os quatro simultaneamente sem depender de estado global.
- Cinco espécies têm silhuetas próprias (dot, sprout, ember, sol, drift). PetCreature varia o corpo por espécie; PetPlush usa corpo comum com crown/ears; PetPaper é corte geométrico de duas cores; PetPixel é sprite 16×16 convertido em runs horizontais para reduzir Rectangle count.
- PetEgg é compartilhado pelas quatro skins e usa coat da espécie futura. Mood altera olhos/boca (content, beaming, peckish, lonely, asleep); asleep fecha olhos e desliga blink/bob; level 15 mostra estrela.
- PetSoftFace é compartilhado pelas duas skins shaded e recebe geometria da face do corpo. Não replicar a lógica de mood em cada renderer.
- PetFamily desenha todos os registros na ordem persistida e mais um slot do próximo egg; nome é editável por membro, bringOut só para o que não está ativo, totalLevel/nextEggAt alimenta eggProgress. A espécie ativa e os outros membros podem ter moods distintos via moodAt(index).
- Os estilos são puramente visuais; XP, cooldown, hatchedAt, activeIndex, species, mood, feed/play e bringOut permanecem no PetService. A UI não concede XP diretamente.

- FaceRing desenha 48 ticks em 360°, head gira uma volta por segundo durante scanning, tail cobre 42%; match preenche a volta inteira em durationMorph, miss muda hue e a LockIsland segura vermelho/shake por duas morph durations.
- Padlock desenha shackle em Shape (não glyph); opened levanta e gira com overshoot. FlipCard mantém previous e dobra a metade antiga e depois a nova; respeita Theme.motion=0 e atualiza previous sem animação nesse caso.
