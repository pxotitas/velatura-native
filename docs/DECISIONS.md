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
