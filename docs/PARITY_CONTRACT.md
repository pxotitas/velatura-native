# Contrato de paridade 1:1 — Velatura Native

**Este documento prevalece sobre qualquer interpretação que permita simplificar o produto.**

## 1. Objetivo inegociável

Velatura Native deve ser uma portabilidade 1:1 do Velatura original. A implementação interna muda; o produto percebido pelo usuário não deve perder nem alterar comportamento.

Não é:
- um redesign;
- uma versão lite;
- uma alternativa inspirada;
- uma reconstrução apenas da barra;
- um protótipo que imita screenshots;
- um conjunto de widgets com controles sem integração;
- uma versão que substitui recursos difíceis por placeholders.

É uma reimplementação integral do mesmo shell, com a mesma identidade, o mesmo conjunto de funcionalidades e comportamento equivalente.

## 2. O que significa 1:1

Para cada recurso, preservar todas as dimensões aplicáveis:

1. **Capacidade:** tudo o que o original permite fazer.
2. **Entrada:** mouse, teclado, atalhos, CLI/IPC, automações e eventos externos.
3. **Saída:** conteúdo, dados, efeitos no sistema, notificações e respostas.
4. **Estado:** estados possíveis, transições, prioridades, concorrência e recuperação.
5. **Persistência:** preferências, histórico, dados, defaults, import/export e migração.
6. **Visual:** dimensões, posição, espaçamento, cor, tipografia, ícones, formas, clipping, sombras e materiais.
7. **Movimento:** duração, easing, sequência, interrupção, reversão, hover, dwell, hide/show e sincronização com input.
8. **Integração:** serviços, protocolos, permissões, dispositivos, daemons e ferramentas externas.
9. **Ciclo de vida:** startup, lazy loading, subscribe/unsubscribe, hotplug, reconnect, shutdown e retomada.
10. **Acessibilidade:** foco, teclado, leitura, motion scale e regiões de input.
11. **Multi-monitor:** instâncias, monitor live, primary/fallback, hotplug e comportamento em outputs virtuais.
12. **Desempenho:** manter o resultado visual sem loops inúteis; otimização não pode remover comportamento observável.

Não basta que a ação principal funcione. Menus contextuais, estados vazios, erros, confirmações, preferências menos usadas e casos extremos também fazem parte da paridade.

## 3. Nenhuma simplificação silenciosa

Se um recurso parecer caro, complexo, redundante ou difícil:
- não removê-lo;
- não substituir por placeholder;
- não mudar o fluxo;
- não reduzir animação ou precisão por conta própria;
- não marcar como concluído parcialmente.

Registrar o bloqueio, dependência, risco e alternativas em docs/DECISIONS.md e docs/TASKS.md. Perguntar ao usuário quando a decisão altera comportamento ou escopo.

Uma limitação técnica pode justificar um bloqueio temporário, nunca uma remoção invisível.

## 4. Fonte de verdade e auditoria

- A fonte primária é a instalação atual em /home/px/.config/quickshell.
- Ler os arquivos ativos completos; não basear implementação em nomes, comentários isolados ou apenas neste inventário.
- Arquivos .new, .bak e .before-* são variantes históricas/rascunhos até que o grafo ativo prove o contrário.
- Seguir imports, singletons, consumidores, scripts, comandos, settings e assets.
- Quando a implementação e design.md divergirem, conferir o QML e registrar a decisão.
- Nunca editar a fonte original durante a portabilidade.

## 5. Contrato por recurso

Antes de começar um recurso, criar/atualizar sua ficha com:

- ID e nome do recurso;
- superfícies/componentes envolvidos;
- arquivos QML e assets originais;
- serviços consumidos;
- scripts e ferramentas externas;
- inputs (mouse/teclado/CLI/eventos);
- outputs e efeitos no sistema;
- estados e máquina de estados;
- dimensões, tokens e regras de layout;
- animações, timings e easing;
- settings/defaults persistidos;
- comportamento multi-monitor;
- erros, permissões e estados indisponíveis;
- comportamento no startup e ao ficar oculto;
- critérios de paridade visual;
- testes de comportamento e regressão.

## 6. Critérios de aceite

Um recurso só pode ser marcado VERIFIED quando:

- todas as ações do original estão presentes;
- todos os estados relevantes foram reproduzidos;
- os efeitos no sistema foram testados;
- preferências e persistência funcionam;
- a comparação visual foi feita com a mesma resolução/scale/tema;
- timings e transições foram comparados;
- atalhos e IPC funcionam;
- estados de erro e dispositivos ausentes foram testados;
- multi-monitor/hotplug foi testado quando aplicável;
- não há placeholders;
- não houve regressão em recursos dependentes;
- evidências e testes estão registrados.

Estados permitidos na matriz:
- NOT AUDITED: ainda não compreendido.
- AUDITED: comportamento e dependências descritos, sem implementação.
- IN PROGRESS: implementação em curso.
- IMPLEMENTED: implementação existe, ainda sem paridade verificada.
- VERIFIED: critérios de aceite comprovados.
- BLOCKED: bloqueio explícito, com motivo e plano.

Nunca usar IMPLEMENTED ou VERIFIED para uma tela que apenas desenha a interface.

## 7. Compatibilidade e migração

- Preservar os targets/verbs do IPC documentados em docs/IPC_API.md.
- Preservar o significado dos settings e defaults.
- Migrar configurações existentes sem sobrescrever a fonte.
- Criar backup antes de importar/migrar.
- Não descartar chaves desconhecidas durante a migração.
- Manter compatibilidade com automações existentes sempre que possível.

## 8. Trabalho entre agentes

Backend e frontend trabalham sobre a mesma especificação 1:1:
- backend implementa o efeito e o estado reais;
- frontend reproduz a apresentação e as interações;
- nenhum dos lados pode declarar o recurso completo isoladamente;
- contratos e mudanças devem ser documentados;
- mocks só são válidos para desenvolvimento temporário e precisam ser substituídos por integração real antes de VERIFIED.

## 9. Desempenho sem perda de fidelidade

O alvo é hardware modesto. Otimizar por eventos, cache, async, lazy loading, redução de duplicação e suspensão de trabalho oculto. Não otimizar removendo módulos, estados, precisão, animações importantes ou recursos.

O resultado deve continuar 1:1, apenas implementado de forma mais eficiente.
