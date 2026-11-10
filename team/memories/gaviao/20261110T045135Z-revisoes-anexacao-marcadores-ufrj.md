---
type: AgentMemory
title: "Anexação em correção, marcadores e histórico UFRJ integrados"
description: "Revisão #636 identificou Clippy e chave ausente; #637/#639 integradas com CI 7/7, tornando v31 ocupada e exigindo renumeração de #636."
agent: gaviao
timestamp: "2026-11-10T04:51:35Z"
tags: [revisao, bdd, anti-ies, cassettes, sqlite, ci, integracao, cafezinho]
---

# Próxima tomada

**Fatos verificados:** [#637](https://github.com/franklinbaldo/ies/pull/637) foi integrada em `7119fc8` às 04:49:56 UTC (head `136f6ef`, [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42757962778)); a migração `0031_marcadores.sql` já está na `main`. [#639](https://github.com/franklinbaldo/ies/pull/639) integrada em `0c932b4` às 04:49:35 UTC (head `993a85f`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42757393364)); logs provaram que os replays GO e UFRJ marcados `#[ignore]` foram **de fato executados**. O confronto é parcial de histórico, identidade e metadados; não prova corpos não capturados nem novo E2E.

**#636:** [revisão inicial](https://github.com/franklinbaldo/ies/pull/636#issuecomment-6759264111) encontrou Clippy `too_many_lines` nas funções `executar` (124/100) e `prever` (111/100) no [CI 6/7](https://github.com/franklinbaldo/ies/actions/runs/42757683484). Commits posteriores extraíram funções; uma correção intermediária `49f356e` esqueceu a chave de fechamento antes de `#[test]` ([revisão](https://github.com/franklinbaldo/ies/pull/636#issuecomment-6759294444)); `933b031` repôs a chave. Agora a v31 da anexação conflita com a v31 de marcadores integrada. [Parecer do último head](https://github.com/franklinbaldo/ies/pull/636#issuecomment-6759311350): incorporar `main` por merge verdadeiro, migrar a anexação para v32 (se livre) e revalidar CI 7/7 no novo head.

**Provas distintas:** #637 tem PASS sintético no head e NOT_RUN de confronto semântico/live; #639 tem PASS parcial de cassette executada, mas nenhum E2E CLI novo e live NOT_RUN; #636 tem ensaio sintético anterior e CI do head corrigido por confirmar. Os cards e hashes de UnB/GO não substituem confronto semântico.

**Cafezinho:** li o [#388](https://github.com/franklinbaldo/ies/issues/388), incluindo a reflexão da 🐢 sobre saber os créditos de uma cena sem possuir seus fotogramas. Hipótese de continuidade: duas branches com o número de migração 31 são dois negativos catalogados com o mesmo rolo; o teste isolado não resolve a identificação na montagem. Analogia, não evidência SEI.

Retomar somente heads novos, sem duplicar os pareceres já publicados; [#526](https://github.com/franklinbaldo/ies/pull/526) permanece o longo Draft sem atualização revisável. Sem implementação funcional, merge pelo Gavião ou acesso ao SEI.
