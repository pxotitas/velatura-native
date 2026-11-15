# Regras compartilhadas para agentes

Este arquivo é o contrato comum de trabalho. Todos os agentes, inclusive ChatGPT/backend e Codeína/frontend, devem lê-lo antes de alterar o projeto.

## 1. Produto e fonte de verdade

- O produto chama-se Velatura Native durante a migração; a identidade do shell continua sendo Velatura.
- O original em /home/px/.config/quickshell é a referência funcional e visual primária.
- O original deve ser tratado como read-only.
- Não presumir comportamento com base apenas no nome de um componente. Abrir o QML, serviços associados, dependências e estados relevantes.
- Se documentação e implementação original divergirem, registrar a divergência em DECISIONS.md e pedir validação quando afetar produto ou aparência.

## 2. Proibições

A versão final não pode depender de Quickshell, QML / Qt Quick, Electron, WebView / Chromium embutido ou runtime JavaScript para implementar o shell. Não mascarar uma dependência proibida atrás de scripts auxiliares.

## 3. Princípios de implementação

- Rust é a linguagem principal.
- GTK4 é a base preferida da UI.
- Relm4 pode ser usado se simplificar a arquitetura sem criar custo desnecessário.
- gtk4-layer-shell é a opção inicial para superfícies Wayland que exigem layer-shell; validar o suporte real no Niri e as limitações da biblioteca.
- zbus para serviços D-Bus quando aplicável.
- Tokio apenas onde a concorrência assíncrona justificar.
- Preferir APIs nativas do Linux e protocolos documentados.
- Evitar polling contínuo; preferir eventos, subscriptions e caches.
- UI não pode bloquear em I/O, chamadas de sistema ou IPC.
- Carregar painéis e serviços sob demanda quando isso não alterar o comportamento esperado.

## 4. Regras de fidelidade

- Não remover funcionalidade só porque é trabalhosa.
- Não substituir um recurso por um botão decorativo ou placeholder e marcar como concluído.
- Não mudar hierarquia, espaçamento, cores, tipografia, forma ou animação sem evidência no original ou decisão aprovada.
- Preservar configurações e estados persistentes.
- Se uma função não puder ser portada, marcar como bloqueada/pendente, explicar a razão e propor alternativas; nunca omitir silenciosamente.
- design.md é a especificação visual compartilhada. Quando houver dúvida, inspecionar a fonte QML correspondente.

## 5. Segurança operacional

O usuário utiliza o shell original diariamente e depende de uma configuração Niri existente.

É proibido sem autorização explícita:
- editar ~/.config/niri/config.kdl;
- mudar o autostart do Niri;
- matar ou desativar o Quickshell/Velatura atual;
- sobrescrever ~/.config/quickshell;
- apagar configurações ou dados do usuário;
- alterar monitores, resolução, Virtual-1, serviços de captura ou a transmissão do iPad;
- instalar pacotes de sistema ou executar comandos privilegiados.

Desenvolver e testar em diretórios e configurações isolados. Não exigir privilégios administrativos para desenvolvimento normal.

## 6. Desempenho e hardware

Hardware primário: Intel i5-5350U, 2 núcleos físicos/4 threads, 8 GB RAM e Intel HD 6000.

- Medir CPU e RAM em repouso e em interação.
- Não alegar que Rust é eficiente sem medir.
- Evitar loops de atualização, animações contínuas e redraws de tela inteira.
- Desativar/pausar atualizações visuais quando a superfície estiver oculta.
- Evitar múltiplas instâncias de serviços e watchers duplicados.
- Durante builds, limitar concorrência a um núcleo quando possível (taskset -c 0 e jobs=1).
- Monitorar temperatura. Interromper trabalho pesado se a CPU passar de 90 °C.

## 7. Contrato entre backend e frontend

- Backend define APIs de domínio, estado, eventos, persistência e operações do sistema.
- Frontend define widgets, composição, estilos, interação visual e animações.
- Não duplicar lógica de sistema em callbacks da UI.
- Não implementar duas vezes o mesmo serviço.
- Toda API deve ter tipos documentados, erros explícitos e comportamento assíncrono definido.
- Mudanças de contrato devem ser registradas em ARCHITECTURE.md e comunicadas ao outro agente.
- Usar fixtures/mock providers para UI quando o backend ainda não estiver pronto; não inventar formatos incompatíveis.
- Preferir uma interface Rust tipada interna entre módulos; só criar IPC externo se houver necessidade real.

## 8. Git e colaboração

- Commits pequenos, descritivos e funcionais.
- Não fazer force-push nem reescrever histórico compartilhado.
- Não adicionar arquivos gerados, caches, builds ou segredos.
- Não alterar arquivos fora do escopo sem avisar.
- Não sobrescrever mudanças do outro agente.
- Antes de editar, verificar git status.
- Ao terminar, informar arquivos alterados, testes executados, limitações e próximos passos.

## 9. Critério para dizer concluído

Um recurso só pode ser marcado como concluído se foi comparado com o original, tem implementação real (não placeholder), trata erros e ausência de dependências, foi testado, atualizou a matriz de portabilidade e não quebrou recursos relacionados.

## 10. Comunicação

Ser explícito sobre o que foi realmente feito. Nunca dizer que algo foi instalado, testado, integrado ou concluído sem evidência. Registrar decisões e bloqueios em arquivos do repositório para que outro agente possa continuar sem depender do histórico do chat.
