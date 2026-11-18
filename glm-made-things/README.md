# glm-made-things

Arquivo da primeira fatia de UI nativa (barra + Dynamic Island + launcher),
escrita pelo agente backend (GLM) e **removida da árvore de código em
16/11/2026 por decisão do usuário**: o resultado visual ficou torto e fora de
paridade com o original (barra desalinhada no meio, faixa preta no canto
direito, pills de workspace fora do estilo, transparência incorreta).

## Conteúdo

| Arquivo | O que era |
|---|---|
| `mod.rs` | Raiz do módulo `ui` (feature `gtk-app`) |
| `shell.rs` | Composição por monitor: bombas de eventos, `BarHandle`, `ServiceHandles` |
| `island.rs` | Island: arbitragem, hover, morph, notificações, padrão `TimerSlot` |
| `launcher.rs` | Painel de launcher com modos por sigilo |
| `morph.rs` | `MorphBox` (subclasse de `gtk4::Box`) |
| `layer.rs` | Helpers gtk4-layer-shell |
| `theme.rs` | Tokens + CSS |

Os arquivos estão **fora do build**; o `src/ui/` foi removido e `main.rs`
roda o runtime sem UI. O backend (`src/services/`, sessão Niri, runtime)
continua na árvore e os 56 testes seguem valendo.

## Estado técnico do que está aqui

- Compilava limpo (`check`/`clippy`/`fmt`), 56 testes verdes no projeto.
- A superfície layer-shell chegou a mapear (`niri msg layers` mostrava
  `velatura-bar` na Top) após a correção do `window.present()` ausente —
  esse detalhe (GTK4 esconde janelas sem `present()`) vale para o re-port.
- **Defeitos visuais conhecidos, sem diagnóstico fechado:** faixa preta
  ocupando do módulo de notificações até a borda direita (suspeita: fundo
  opaco da janela/região opaca do GTK sobre transparência do layer-shell);
  island desalinhado do centro; workspaces como botões avulsos em vez do
  estilo do original.

## O que o re-port precisa estudar no original (read-only)

- `bar/Bar.qml` — arquitetura real: superfície **full-screen transparente**
  (`color: "transparent"`, `exclusiveZone: 0`, `ExclusionMode.Ignore`), band
  que morfa com o island, zonas laterais (`BarZone`) com estilos
  `grouped` (default) / `spread` / `island` / `bezel`, máscara de input por
  região, reserve em superfície separada (`BarReserve`).
- `bar/island/IslandRest.qml` — face de repouso: relógio central +
  segmentos de atividades.
- `bar/widgets/BarZone.qml`, `WorkspacesWidget.qml`, `ClockModule.qml` —
  estilo real das cápsulas, fonte (`Theme.fontFamily`), tamanhos.
- `theme/Theme.qml` — tokens definitivos (cores, raios, fontes,
  `capsuleHeight`, `barTopMargin`, `capsuleSpacing`).
- Defaults do `SettingsService`: `barStyle: "grouped"`, `barSides:
  "capsule"`, `barHeight: 32`, `barMargin: 16`, `barSideMargin: 18`,
  `barEverywhere: true`.

Nada daqui deve voltar para `src/` sem refazer a comparação visual contra o
original rodando lado a lado.
