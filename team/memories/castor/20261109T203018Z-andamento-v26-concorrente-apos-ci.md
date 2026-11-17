---
type: AgentMemory
title: "Andamento pronto sinteticamente, mas migração v26 concorrente"
description: "PR #602 sincronizada por merge real com a main e CI 7/7; aguardar integração da #597/v26 para renumerar andamento como v27 e revalidar."
agent: castor
timestamp: "2026-11-09T20:30:18Z"
tags: [anti-ies, andamento, sqlite, merge, ci, cassettes]
---

# Encaixe para a próxima rodada

**Fato:** A [PR #602](https://github.com/franklinbaldo/ies/pull/602) implementa a prévia BDD-259-01 com CLI real → anti-IES → SQLite efêmero e zero POST. O head `5c1004a7` incorporou `main@6dd42fb` por merge verdadeiro, somente com duas memórias aditivas. O [CI oficial #42693003142](https://github.com/franklinbaldo/ies/actions/runs/42693003142) passou **7/7** nesse SHA, após Draft → Ready; [rápido](https://github.com/franklinbaldo/ies/actions/runs/42692773200) e [inspeção](https://github.com/franklinbaldo/ies/actions/runs/42692773292) também passaram.

**Atenção:** [#597](https://github.com/franklinbaldo/ies/pull/597) (🐙) ainda estava aberta e possui `0026_blocos_listados.sql`; #602 contém `0026_andamento_processo.sql`. Integrar #597 primeiro, depois sincronizar #602 por merge verdadeiro e migrar somente andamento para v27, incluindo registrador, `PRAGMA user_version`, expectativas e novo CI no head. Não mesclar v26 duplicada.

**Provas:** sintética PASS no BDD-259-01; UnB/GO com 50 interações verificadas em integridade e estrutura, **confronto semântico independente NOT_RUN**; treinamento NOT_RUN. BDD-259-02/03 continuam pendentes, com `--confirmar` público bloqueado. Handoff na [#259](https://github.com/franklinbaldo/ies/issues/259#issuecomment-6750681871).

**Cafezinho:** reli a [conversa #388](https://github.com/franklinbaldo/ies/issues/388), inclusive a distinção da 🦊 entre observação de um candidato e autorização para atuar; também se aplica a distinguir prévia de andamento de escrita. É analogia, não evidência do SEI.
