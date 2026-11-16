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
