# Auditoria visual inicial — tema e configurações

**Fonte:** instalação ativa em `/home/px/.config/quickshell` (somente leitura).
**Escopo:** `theme/Theme.qml`, `theme/Palettes.qml`, `theme/Motion.qml`,
`services/SettingsService.qml`, `services/ThemeService.qml`,
`settings/AppearanceSection.qml` e `settings/WindowsSection.qml`.

## Identidade e opções observadas

- **Tema/paleta:** escolha `adaptive` (segue cores extraídas do wallpaper) ou
  nove paletas nomeadas: Catppuccin Mocha/Latte, Tokyo Night, Gruvbox Dark,
  Nord, Rosé Pine, Cyberpunk, Bauhaus e Crimson Moon. Cada paleta define
  background, surface, hover, border, text, muted, accent, accent hover/text
  e cores semânticas. `Theme.qml` inicia com valores escuros neutros e azul
  antes de carregar a paleta adaptativa. Não há seletor geral claro/escuro;
  Catppuccin Latte é uma paleta clara.
- **Superfícies:** `surfaceStyle` da barra/ilha, `desktopGround`, `dockGround`
  e `terminalGround` são configurações distintas. Grounds reconhecidos:
  `classic`, `glass`, `liquid` e `matugen`. Classic é default para barra/ilha,
  desktop e dock; terminal inicia em glass. Game mode força classic no shell.
- **Transparência/blur:** terminal ground usa tint 0.22 e opacity 0.90; liquid
  usa RGBA (0.06, 0.07, 0.09, 0.66); matugen usa alpha 0.96. Desktop e dock
  têm opacidade persistida, ambas 100%. Glass/liquid dependem do blur do
  compositor; CSS GTK não pode fingir que alpha implementa blur.
- **Bordas/raios/sombras:** ilha estrutural preta (#000), interior #141414,
  hover #1f1f1f e borda #262626. Raios compartilhados 8/12/18/pill 999 px;
  desktop/screen radius 22 px e notch 6 px. Sombras de bar/dock/widgets são
  independentes e começam desligadas; game mode as suprime.
- **Tipografia:** stacks persistidos `Inter, Cantarell, SF Pro Text, sans-serif`
  e `JetBrainsMono Nerd Font Mono, monospace`; fontes de display/script/nota
  derivam delas e da disponibilidade local. Tamanhos 10/11/13/14/16,
  widgets 30 e display 68 px.
- **Geometria:** cápsula da barra 32 px, margem superior 16, lateral 18,
  espaçamento 8; reserva é margem + altura e input adiciona 8 px. Grid desktop
  usa células 86–108 px e gutter 18. Dock inicia com ícone 44 px, padding 6,
  gap 8 e margem 18.
- **Movimento:** 140/200/380 ms (fast/medium/morph), escala 100%, curva
  `OutCubic`; opções OutQuint, OutBack e Linear; presets smooth/snappy/springy/
  off ajustam escala e curva. Mudança de paleta usa 260 ms. Game mode fixa
  escala em zero e desativa trabalho animado do shell.
- **Outras opções visuais:** wallpaper folder, Matugen, transição (default
  wipe), greeting, `screenCorners`, `windowShadow`, `windowGlass`,
  `desktopTheme` e `desktopOpacity`. Opções de janelas/blur dependem das
  capacidades e regras Niri; `WindowsSection.qml` explicita limites.

## Implementado nesta fatia

`src/ui/theme.rs` centraliza os tokens do estado inicial, catálogo das nove
paletas, métricas compartilhadas e CSS GTK para componentes com classe
`velatura-root`. O bootstrap existente usa o provider; não há nova janela,
serviço ou lifecycle. O provider representa apenas classic inicial. Não
declara paleta adaptativa, transparência, blur, persistência ou controles como
funcionais.

## Dependências pendentes

1. Este checkout ainda não expõe `SettingsSnapshot`, mutações tipadas de
   settings, nem `ThemeService`/`WallpaperService`; o draft de
   `APP_API_CONTRACT.md` não é contrato executável. Seleção/persistência real
   depende dessa API.
2. Paleta adaptativa requer resultado/evento do serviço nativo de tema e
   wallpaper; a UI não deve duplicar esse cálculo.
3. Glass/liquid requer superfícies Wayland e validação de blur no Niri; CSS
   sozinho não implementa os materiais originais.
4. Fontes devem ser resolvidas contra as fontes locais sem afirmar que uma
   família ausente foi aplicada.

Por essas dependências, controles de Appearance e Dynamic Island ainda não são
declarados implementados. O próximo passo é fechar o contrato nativo de
settings/tema com o backend; a validação de superfícies Niri segue em fatia
separada.
