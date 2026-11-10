---
type: AgentMemory
title: "Auditoria #592: merge fiel à main e prova limitada ao documento"
description: "PR #672 reconciliada com merge de dois pais; somente dois documentos diferem da main; CI rápido aprovado no SHA reconciliado, completo iniciado; seguir sem confundir hashes com replay."
agent: polvo
timestamp: "2026-11-10T15:51:33Z"
tags: [blocos, auditoria, sincronizacao, ci, anti-ies, cassettes]
---

# Para a próxima rodada

- **Fato:** a [PR #672](https://github.com/franklinbaldo/ies/pull/672) voltou a `behind=0` por merge real [`76c4e7e8`](https://github.com/franklinbaldo/ies/commit/76c4e7e8a182c43c49b130dbe5ac9b0e0247bab3), com pais `39b7a5d` e `5ce7eff`. A árvore preservou a main integral e sobrepôs somente os dois documentos auditados. O 🦅 considerou o conteúdo LGTM, condicionado ao CI final e sincronização.
- **Prova nessa rodada:** [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42848548548) sucesso no head `76c4e7e8`; [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42848601101) estava em andamento na consulta, não declarar 7/7 sem verificação posterior. Nenhum novo teste funcional ou replay de corpos de cassette foi escrito.
- **Limite:** quatro cards UnB/GO, 394 interações, hashes verificados anteriormente não demonstram correspondência semântica; treino/live NOT_RUN. Não converter `sim_bloco_linha` presente em `sim_bloco_detalhe` vazio. A correção da #592 é auditoria documental, não nova capacidade da CLI.
- **Retomada:** reconciliar na PR o commit desta própria memória publicado na main e qualquer avanço paralelo, obter 7/7 no SHA final e entregar para integração 🐘/🦡. Vale a observação da 🐢 no [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6767655539): publicar memória pode alterar a base do PR recém-conciliado.
