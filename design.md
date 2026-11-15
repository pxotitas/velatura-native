# Velatura — Design System & Visual Specification

**Documento de referência visual obrigatório para a reconstrução nativa.**

Fonte auditada: /home/px/.config/quickshell, principalmente theme/Theme.qml, theme/Palettes.qml, theme/Motion.qml, services/SettingsService.qml, bar/Bar.qml, bar/widgets/DynamicIsland.qml, bar/island/IslandState.qml, dock/Dock.qml e desktop/Desktop.qml.

**Status:** especificação inicial baseada diretamente nos tokens e na lógica QML. Ainda é necessário comparar screenshots reais e inspecionar cada painel/componente durante sua portabilidade. Quando um detalhe não estiver especificado aqui, o QML original é a autoridade.

---

# 1. Identidade visual

Velatura tem uma linguagem visual própria: uma interface de shell compacta, construída em torno de cápsulas arredondadas e de uma Dynamic Island central que se transforma conforme o contexto.

A interface combina:
- uma barra superior com grupos de módulos;
- uma cápsula central que pode expandir para painéis;
- módulos compactos com ícones, indicadores e valores;
- painéis escuros e organizados em cards;
- uma dock flutuante;
- widgets posicionáveis no desktop;
- detalhes de vidro, rim, recortes e notch opcionais;
- temas e paletas personalizáveis.

A aparência deve ser precisa e intencional, não uma coleção genérica de widgets GTK.

## Princípios

1. **Forma antes de decoração:** geometria, alinhamento, espaçamento e hierarquia são essenciais.
2. **Compacto, mas legível:** a barra é pequena; painéis abertos têm espaço e organização.
3. **Cápsulas como linguagem:** módulos e grupos são organizados em superfícies arredondadas.
4. **A ilha é o centro da experiência:** não é apenas um botão ou popup; ela muda de conteúdo e tamanho mantendo continuidade.
5. **Tema consistente:** componentes consomem tokens compartilhados; não espalhar cores hardcoded.
6. **Efeitos são opcionais:** o estilo padrão é sólido/classic, sem depender de blur.
7. **Movimento funcional:** animações comunicam mudança de estado; não devem ser enfeites constantes.

# 2. Hierarquia das superfícies

## 2.1 Barra superior

A barra ocupa a região superior de cada monitor. Ela tem:
- zona esquerda;
- Dynamic Island central;
- zona direita;
- módulos organizados em uma linha;
- reserva de espaço para que janelas não fiquem atrás da barra.

Defaults confirmados em SettingsService:
- altura da cápsula: 32 px;
- margem superior: 16 px;
- margem lateral: 18 px;
- espaçamento entre cápsulas: 8 px;
- faixa de input/reserva visual: margem superior + altura da cápsula; a região de input adiciona 8 px de folga inferior.

A barra alinha-se visualmente às margens do compositor. Não deve parecer uma barra tradicional ocupando toda a largura com fundo retangular, exceto no estilo bezel.

Há uma superfície por monitor. Somente uma ilha fica live: a ilha acompanha o monitor focado. As outras barras continuam desenhadas, mas sem receber o estado global da ilha. Ao mudar foco, a responsabilidade passa de uma instância para outra sem destruir/recriar a ilha visível.

A implementação original mantém a layer surface em tamanho estável e muda a região de input. Não redimensiona a janela da barra a cada frame da animação, para evitar jitter no compositor.

## 2.2 Quatro estilos de barra

### grouped (default)

- Grupos esquerdo e direito ficam próximos da ilha.
- Os grupos se deslocam para abrir espaço quando a ilha cresce.
- É o estilo inicial confirmado.
- Mantém leitura de três zonas: esquerda, centro e direita.

### spread

- Grupo esquerdo encostado na margem esquerda.
- Grupo direito encostado na margem direita.
- Ilha permanece centralizada.
- Os grupos normalmente ficam parados.
- Se um painel aberto for largo demais para o espaço entre os grupos, os lados saem de cena para dar largura ao painel.

### island

- Tudo fica dentro de uma cápsula/faixa unificada.
- O fundo se expande/molda para acomodar a ilha.
- Ao abrir painel, os módulos laterais são ocultados/recortados.
- A faixa acompanha a largura da ilha.

### bezel

- Uma faixa ocupa praticamente toda a largura superior, com margens laterais.
- A ilha aparece como um recorte/queda no centro.
- Os cantos da tela podem participar da silhueta.
- A faixa não se desloca durante o morph.

Esses estilos não são apenas variações de alinhamento: possuem regras próprias de superfície, recorte, largura e input. Portar os quatro.

## 2.3 Dynamic Island

A Dynamic Island é uma cápsula central que troca entre camadas:
- repouso com módulos/relógio/atividades;
- resumo/glance;
- detalhe de módulo;
- OSD transitório;
- notificação;
- painel aberto.

A forma muda de acordo com o conteúdo. A ilha pode expandir em largura e altura, arredondar os cantos e apresentar um painel sem parecer uma janela independente.

O estado do conteúdo é separado do estado geométrico. O painel aberto tem prioridade sobre notificações/OSD; o original descarta transientes recebidos enquanto um painel está aberto, em vez de exibi-los atrasados depois.

O tamanho do painel é calculado antes de carregá-lo para que a cápsula alcance a geometria correta. Os painéis possuem tamanhos próprios, incluindo, por exemplo:
- Appearance: 940 × 196;
- Stats: 940 × 614;
- Wi-Fi: 420 × 500;
- Bluetooth: 420 × 500;
- Session: 720 × 180;
- Auth: 460 × 236;
- Machines: 860 × 580;
- Screen Time: 940 × 614;
- Overview: largura 1560 e altura calculada por linhas;
- outros tamanhos são derivados dos respectivos serviços.

Esses valores são medidas lógicas do QML original e devem ser conferidos em screenshots e escalas reais antes de congelar a UI GTK.

## 2.4 Dock

A dock é uma cápsula flutuante numa borda da tela, exceto a borda superior, reservada à barra.

Defaults:
- borda inferior;
- alinhamento central;
- tamanho de ícone: 44 px;
- padding: 6 px;
- gap: 8 px;
- raio alinhado ao raio desktop (22 px);
- launcher no início da linha;
- mostrar apps em execução;
- magnificação habilitada;
- autohide desabilitado;
- dock por monitor;
- somente desenhar se houver itens.

A magnificação original:
- cresce até 1,55×;
- influencia ícones vizinhos;
- usa uma curva espacial suave centrada no ponteiro;
- mantém o tamanho externo da cápsula estável;
- desativa magnificação durante drag, menu aberto e edição do desktop.

Quando autohide está ativo, existe uma faixa de revelação de 4 px na borda. O movimento de ocultar/revelar deve ser sincronizado com a região de input.

## 2.5 Desktop e widgets

A camada de desktop fica entre wallpaper e janelas, no layer inferior e sem exclusive zone.

O desktop oferece:
- widgets colocados em grade;
- diferentes tamanhos e spans;
- drag para mover;
- resize por canto;
- clique para abrir inspector;
- picker de widgets;
- faces analógicas e sticker;
- edição em modo separado;
- deck de cartões/notas nas bordas.

Tokens de grade confirmados:
- célula base: 86 px;
- célula máxima: 108 px;
- gutter: 18 px;
- stride: célula + gutter;
- widgets ocupam células inteiras.

No modo de organização, a superfície sobe temporariamente para o layer superior e captura input da tela. Escape, clique direito ou clique em outra superfície encerra o modo, conforme o comportamento do componente original.

## 2.6 Settings

A janela principal de configurações é uma janela normal xdg-toplevel, gerenciada pelo compositor, não uma layer surface.

Tamanho inicial:
- 980 × 680 px.

Ela pode permanecer aberta enquanto a Dynamic Island continua visível e reage a alterações de configuração em tempo real.

A organização interna inclui seções para aparência, barra, controles, dock, entrada, ilha, launcher, módulos, monitores, sessão, sistema, widgets, janelas e atalhos. A estrutura exata deve ser conferida em settings/SettingsPanel.qml e nos componentes de seção.

# 3. Superfícies, cores e materiais

## 3.1 Default classic

O valor padrão confirmado é surfaceStyle = classic.

- Ilha e cápsulas são superfícies sólidas.
- A ilha usa preto puro como identidade estrutural (#000000).
- Superfície interna escura: #141414.
- Hover: #1f1f1f.
- Borda: #262626.
- Não depende de blur do compositor.
- Não deve ser transformado em vidro translúcido por padrão.

Este é o caminho visual e de performance prioritário para a primeira implementação nativa.

## 3.2 Glass

Opção configurável, não default:
- permite que o wallpaper apareça por trás;
- usa ground com tint da cor de destaque;
- o terminal recebe tint de accent e opacidade própria;
- tem veil claro, rim e sheen;
- o efeito de blur vem do compositor, não de um WebView ou shader de interface.

Tokens observados:
- terminalTint: 0.22;
- terminalOpacity: 0.90;
- rim glass: branco com alpha 0.20;
- veil base: branco com alpha 0.08;
- veil hover: branco com alpha 0.14;
- sheen branco com alpha 0.10;
- glassEdge: 0.12.

## 3.3 Liquid

Opção configurável:
- frost escuro e translúcido;
- ground aproximado: RGB (0.06, 0.07, 0.09), alpha 0.66;
- rim muito sutil (branco alpha 0.08);
- veil base branco alpha 0.07;
- hover branco alpha 0.13;
- linha branco alpha 0.18;
- iluminação direcional mais forte, com sheen branco alpha 0.20;
- a iluminação é mais intensa em dois cantos opostos e desaparece até um traço nos outros.

Não aplicar liquid em todo lugar automaticamente. Respeitar o material configurado por superfície.

## 3.4 Matugen / wallpaper adaptive

O tema pode derivar cores do wallpaper por Matugen. O token de fundo matugen começa em #0c0c0c e o sistema aplica paleta dinâmica. O customizador deve ler a mesma fonte de paletas que o serviço de tema, evitando divergência entre preview e aplicação.

A troca de cores tem transição de 260 ms nos tokens de fundo, superfície, borda, texto e accent.

## 3.5 Paletas disponíveis

A lista curada em Palettes.qml contém:

| ID | Nome | Accent principal |
|---|---|---|
| catppuccin_mocha | Catppuccin Mocha | #89b4fa |
| catppuccin_latte | Catppuccin Latte | #1e66f5 |
| tokyo_night | Tokyo Night | #7aa2f7 |
| gruvbox_dark | Gruvbox Dark | #d79921 |
| nord | Nord | #88c0d0 |
| rose_pine | Rosé Pine | #ebbcba |
| cyberpunk | Cyberpunk | #ff007f |
| bauhaus | Bauhaus | #e52521 |
| crimson | Crimson Moon | #e63946 |

Cada paleta fornece tokens: background, surface, surfaceHover, border, text, textMuted, accent, accentHover, accentText, red, green, yellow e blue.

Não hardcode uma paleta dentro de cada widget. Todos os componentes precisam consumir o mesmo theme model.

## 3.6 Indicadores e cores funcionais

Cores fixas observadas:
- branco (#ffffff) para indicador ativo;
- cinza escuro (#4d4d4d) para indicador inativo;
- verde (#32d74b) para bom;
- amarelo (#ffd60a) para aviso;
- vermelho (#ff453a) para erro;
- azul claro (#64d2ff) para timer;
- privacidade: microfone laranja (#ff9f0a), câmera verde e tela azul (#0a84ff).

O tema possui cores próprias para pets, papel, stickers, espectro de áudio e estados de contribuição. Consultar Theme.qml para os tokens completos antes de implementar esses elementos.

# 4. Geometria, espaçamento e raios

Tokens confirmados:
- bar/capsule height: 32 px (configurável);
- top margin: 16 px (configurável);
- side margin: 18 px (configurável);
- capsule spacing: 8 px;
- desktop gutter: 18 px;
- panel padding: 20 px;
- control center grid: 6 colunas × 8 linhas;
- control cell: 140 × 64 px;
- control gutter: 12 px;
- botões circulares do control center: 42 px;
- detail header: 38 px;
- detail title: 24 px;
- detail row: 36 px;
- detail row gap: 4 px;
- detail section gap: 14 px;
- detail label column: 128 px;
- module card width: 380 px;
- module card inset: 4 px;
- module card padding: 16 px;
- module card mark: 44 px;
- module card gap: 14 px;
- module card row: 22 px;
- module card row gap: 8 px;
- module card label column: 84 px;
- desktop radius: 22 px;
- small radius: 8 px;
- medium radius: 12 px;
- large radius: 18 px;
- pill radius: 999 px;
- notch radius: 6 px;
- picture corner factor: 0.24 da menor dimensão relevante.

Estes são tokens de layout, não valores arbitrários para replicar isoladamente. Preservar relações e escala quando o monitor ou scale factor mudar.

# 5. Tipografia e ícones

A interface usa tipografia limpa, compacta e com pesos distintos para títulos, valores e labels. A fonte padrão pode ser personalizada por SettingsService; existe também uma família monoespaçada separada.

Regras:
- manter alinhamento óptico;
- usar peso maior em valores e headings;
- usar muted text para metadados;
- não reduzir labels até ficarem ilegíveis;
- respeitar fontes configuradas e fallback;
- manter ícones consistentes em forma e peso;
- os aliases de ícones de aplicações ficam centralizados no tema;
- consultar FontService, Theme.qml e os componentes originais antes de escolher fallback.

Não presumir a fonte ativa sem ler o estado/configuração do usuário.

# 6. Dynamic Island: estados e interação

O estado central tem camadas identificadas:
- modules;
- summary;
- osd;
- notification;
- panel.

A camada resolvida segue prioridade funcional:
1. shell inativo: repouso;
2. painel aberto: panel;
3. notificação ativa: notification;
4. OSD transitório: osd;
5. resumo ativo: summary;
6. caso normal: modules.

Regras:
- abrir painel cancela OSD transitório;
- abrir painel dispensa a notificação atual;
- evento OSD recebido enquanto painel está aberto é descartado;
- OSD expira após 1800 ms;
- ilha live acompanha monitor focado;
- durante painel aberto, a ilha não troca de monitor;
- a ilha deve fechar/limpar seu estado ao perder o papel live;
- primeira frame após startup não deve animar a ilha de zero.

O hover pode mostrar glance/summary quando essa preferência está habilitada. Clique/atalho abre control center ou painel associado. Há tempos de permanência e fechamento por saída do ponteiro; consultar DynamicIsland.qml para regras completas de dwell/leave.

# 7. Movimento e animações

## 7.1 Tokens de tempo

Theme.qml define:
- fast = 140 ms × motion scale;
- medium = 200 ms × motion scale;
- morph = 380 ms × motion scale;
- palette transition = 260 ms.

Motion scale é configurável. O modo game mode zera movimento e força materiais sólidos. Motion presets incluem:
- Smooth: scale 100, OutCubic;
- Snappy: scale 70, OutQuint;
- Springy: scale 110, OutBack;
- Off: scale 0, Linear.

Curvas disponíveis:
- OutCubic;
- OutQuint;
- OutBack;
- Linear.

A curva padrão deve ser confirmada no valor ativo da configuração, mas o fallback em Theme.qml é OutCubic.

## 7.2 Ilha e cápsulas

- largura e altura da ilha mudam com morph de 380 ms;
- radius muda em medium (200 ms);
- cor pode mudar em fast (140 ms);
- bandWidth e islandWidth devem compartilhar progresso/curva para parecer um único movimento;
- a barra não deve redimensionar a layer surface por frame;
- a região de input precisa acompanhar a forma visível sem lag;
- transição de tela live é uma troca de estado, não recriação visual.

## 7.3 Dock

- autohide usa movimento morph;
- magnificação tem entrada/saída fast;
- swell acompanha hover;
- ícones crescem e empurram vizinhos sem alterar o tamanho da cápsula;
- durante drag/menu/edição, magnificação fica desabilitada;
- atraso de retração do peek: 350 ms.

## 7.4 Outros movimentos

- fade de conteúdo e estados de hover usam fast/medium;
- mudanças de paleta: 260 ms;
- wallpaper transitions têm presets próprios em WallpaperService;
- faces de widgets, pets, jogos, spectrum e previews podem ter movimento específico. Inspecionar cada implementação, não aplicar um preset global cegamente.

## 7.5 Regras de performance

- motionScale = 0 deve realmente remover movimento e timers puramente visuais.
- não animar conteúdo invisível;
- evitar animações infinitas;
- evitar blur em tempo real por padrão;
- preferir animações de opacity/transform a redesenhos complexos;
- não usar timers de alta frequência para simular animação;
- manter sincronizados tamanho, conteúdo, input region e estado do compositor.

# 8. Composição dos painéis

Os painéis são apresentados dentro da ilha e devem respeitar a geometria do painel calculada antes do carregamento.

O control center usa uma grade configurável de até 6 × 8 células. Blocos ocupam spans inteiros. Os detalhes de áudio/rede/Bluetooth/brilho/night light usam páginas com cabeçalho, seções e linhas consistentes.

Os cards de módulo usam:
- mark de 44 px;
- heading e figura;
- rows de altura 22 px;
- label column de 84 px;
- padding 16 px;
- width base 380 px.

Preservar as densidades, alinhamentos e layouts específicos dos painéis. Não transformar tudo em uma coluna de cartões padrão.

# 9. Acessibilidade e input

- foco de teclado visível quando o painel recebe keyboard focus;
- Escape fecha painel/encerra edição conforme a superfície;
- clique fora fecha a ilha quando apropriado;
- input region deve corresponder à geometria, não cobrir a tela sem necessidade;
- controles precisam ter labels acessíveis;
- sliders, toggles, tabs e menus devem funcionar com teclado;
- não interceptar teclas globais sem o estado/painel correto;
- não roubar foco ao mostrar OSD ou notificação.

# 10. Regras de fidelidade para a portabilidade

1. O default classic sólido é obrigatório na primeira versão visual.
2. Glass e liquid são opções preservadas, não efeitos impostos.
3. Os quatro estilos de barra devem existir.
4. A ilha precisa morphar, não simplesmente abrir uma janela separada.
5. As superfícies por monitor e a ilha live devem ser preservadas.
6. Settings permanece uma janela normal.
7. Dock e desktop continuam recursos separados.
8. Não simplificar painéis ou eliminar módulos silenciosamente.
9. Cada componente deve ser comparado com o original antes de marcar como portado.
10. Se o QML contradizer este documento, documentar a divergência e atualizar este arquivo após validar.

# 11. Arquivos originais para inspeção visual

- theme/Theme.qml — tokens globais, superfícies, raios, espaçamentos, animações.
- theme/Palettes.qml — paletas.
- theme/Motion.qml — presets de movimento legados/preview.
- services/SettingsService.qml — defaults e opções configuráveis.
- bar/Bar.qml — geometria, composição e comportamento da barra.
- bar/widgets/DynamicIsland.qml — tamanho, camadas, conteúdo e transições da ilha.
- bar/island/IslandState.qml — máquina de estados.
- bar/island/IslandRest.qml — conteúdo em repouso.
- bar/island/IslandSummary.qml — glance.
- bar/modules/* — módulos.
- bar/island/* — painéis.
- components/* — primitives visuais.
- dock/Dock.qml — dock e magnificação.
- desktop/Desktop.qml — desktop e edição.
- settings/SettingsPanel.qml e settings/* — configurações.
- lock/* — lock screen.
- capture/* — captura.

Este documento é um mapa de design e não substitui a leitura dos componentes-fonte. Atualize-o com medidas e screenshots reais durante a auditoria de cada área.
