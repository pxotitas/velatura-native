# Pesquisa separada: gtk4-layer-shell e Niri

**Data:** 16/11/2026  
**Escopo:** compatibilidade e riscos de abordagem; sem alteração de dependências, superfícies ou configuração.

## Conclusão provisória

Há evidência upstream de que Niri implementa layer-shell, então `gtk4-layer-shell` é uma abordagem tecnicamente plausível. Isso não comprova que a combinação instalada de biblioteca, GTK e Niri atende às necessidades específicas do Velatura. A decisão permanece **não validada em runtime**; a dependência não foi adicionada.

O README upstream de gtk4-layer-shell restringe a biblioteca a compositores Wayland que implementam Layer Shell e lista suporte a compositores Smithay. A wiki oficial do Niri documenta comportamento específico para componentes layer-shell: fullscreen cobre a camada `top`, somente `overlay` aparece acima; camadas `bottom/background` recebem foco on-demand, mas foco exclusivo só quando não há janelas no workspace; durante Overview, `top/overlay` ficam acima enquanto `bottom/background` acompanham o zoom.

Implicações para o Velatura: barra persistente deve avaliar `top`; conteúdo que precisa aparecer acima de fullscreen exige `overlay`; keyboard focus e exclusiva zone precisam ser provados separadamente. Não é seguro tratar a disponibilidade do protocolo como prova de paridade de máscaras/input region, output routing, handover ou exclusive-zone do original.

## Evidência local

- `pkg-config --modversion gtk4-layer-shell-0` encontrou versão `1.3.0` no ambiente atual. Isso prova a presença da biblioteca C local, não compatibilidade operacional nem que o binding Rust está configurado.
- A tentativa read-only de enumerar globals com `wayland-info` falhou antes de conectar ao display (`failed to create display: Operation not permitted` no sandbox). Portanto o protocolo anunciado na sessão não foi confirmado por este processo.
- Não foi iniciada demo nem criado teste de superfície; a sessão diária e a configuração Niri permaneceram intocadas.

## Próximo passo de validação isolada

Executar uma demo mínima controlada em configuração de teste após permitir acesso normal à sessão Wayland; confirmar `zwlr_layer_shell_v1`, criar surfaces `top` e `overlay` em cada output, verificar keyboard modes on-demand/exclusive, exclusive zone, regiões de input, fullscreen, Overview, output hotplug e fechamento/recriação do output. Só então escolher binding/versionamento e registrar evidência prática antes de incluir a dependência no Cargo.

## Fontes primárias

- [gtk4-layer-shell README — suporte a Wayland/compositores](https://github.com/wmww/gtk4-layer-shell)
- [Wiki oficial Niri — Layer-Shell Components](https://github.com/niri-wm/niri/wiki/Layer%E2%80%90Shell-Components)
