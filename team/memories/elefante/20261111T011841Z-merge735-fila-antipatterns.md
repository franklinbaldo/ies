---
type: AgentMemory
title: "Prioridade de ablação integrada na norma comum; sete fatias livres"
description: "Integrei #735 (AGENTS.md) com CI oficial 7/7 e preparei #717–#723 para implementadores; PRs #724/#726 seguem sem gate de integração."
agent: elefante
timestamp: "2026-11-11T01:18:41Z"
tags: [antipatterns, prioridade, integracao, qualidade, ci, cafezinho]
---

# Retomar na próxima rodada

- **Decisão do proprietário efetiva na main:** [PR #735](https://github.com/franklinbaldo/ies/pull/735) integrada em [`798f06f`](https://github.com/franklinbaldo/ies/commit/798f06f3c40ea1e6f7f399cfd9c0c5d10849e5bf); dois pais, branch `behind=0`, head `ee52e43`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42937312944). `AGENTS.md` agora prioriza remover antipatterns efetivos sobre novas capacidades; ablação local tem teste técnico, mudança pública exige BDD CLI.
- **Fila distribuída:** #717 (100) erros sem dado; #718 (99) `.ok()` pós-escrita; #719 (98) descarte em pós-POST/I/O; #720 (97) ramos que colapsam; #723 (96) nomenclatura; #721 (95) supressões; #722 (94) defaults. Todas `handoff-chatgpt` e sem avatar. [Coordenação na mãe #716](https://github.com/franklinbaldo/ies/issues/716#issuecomment-6784037462), com sugestões de áreas, sem reservas.
- **Esteira:** [#724](https://github.com/franklinbaldo/ies/pull/724) teve [CI oficial falho](https://github.com/franklinbaldo/ies/actions/runs/42935178443) no head `efaa5e0`: Clippy `too_many_lines` (101/100, `tests/support/documento_interprocessos_estado.rs:180`) e teste existente esperando 302 no anti-IES. Comentário diagnóstico publicado, sem editar branch alheia. [#726](https://github.com/franklinbaldo/ies/pull/726) head `704ca86` só tinha CI rápido verde e GREEN POST/SQLite ainda não comprovado; [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft conflitante. Não integrar.
- **Limites:** sem código de operação alterado por 🐘, nenhum GET/POST ao SEI externo, sem produção/segredos/retry/subcódigos; cassettes e live não foram revalidados nesta rodada.
- **Cafezinho:** visita completa à [#388](https://github.com/franklinbaldo/ies/issues/388); destaque para a conversa sobre lista parcial versus ausência e para a fronteira entre título do teste e evidência real. Sem necessidade de comentar novamente.

**Próxima rodada:** examinar PRs de #717–#723 antes das capacidades novas; exigir ablação real e contraprova. Para merge, verificar Ready, ausência de contraprova, branch atualizada e CI oficial 7/7 no SHA exato; não integrar #724/#726/#526 por gates anteriores.
