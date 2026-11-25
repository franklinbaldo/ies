---
type: AgentMemory
title: "Assinaturas com observação ausente e prévia honesta de marcador"
description: "Pareceres #651/#652: SQL triestado corrige assinatura não observada; prévia de retirada não pode negar capacidade de confirmação condicional."
agent: gaviao
timestamp: "2026-11-10T08:42:00Z"
tags: [revisao, bdd, anti-ies, anexacao, marcadores, cassettes, sqlite, ci, cafezinho]
---

# Para retomar

**Fato #651:** [parecer do head `0bdb2b2`](https://github.com/franklinbaldo/ies/pull/651#issuecomment-6762918133). `LEFT JOIN` entre `documento_simulado` e `documento_editor` diferencia assinatura zero, positiva e não observada, sem usar subcódigos novos. Os REDs estão documentados na PR; CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42783676087) no SHA anterior `30e0687`; head novo muda somente memória do 🦫, CI rápido verde e [CI completo novo](https://github.com/franklinbaldo/ies/actions/runs/42784405501) estava em execução na última checagem. Não afirmar aprovação 7/7 do head novo sem conferir.

**Fato #652:** [parecer do head `6d2aadb`](https://github.com/franklinbaldo/ies/pull/652#issuecomment-6762918754). BDD-255-02 já tem confirmação sintética na main via #649; a PR corrige apenas a frase enganosa da prévia, preservando `retirada_possivel=false` antes de conferir o formulário. [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42782312167) no head, mas a comparação ficou `behind=1` e divergente após avanço da main: merge verdadeiro + novo CI antes da integração.

**Provas:** compatibilidade CLI real ↔ anti-IES ↔ SQLite sintética em ambas; cassettes UnB/GO de anexação e marcadores identificadas e disponíveis; inspeção de hashes **não** confirma paridade semântica S/N ou mensagem da CLI. Live NOT_RUN; nenhuma requisição ao SEI pelo Gavião.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente [o relato do 🐘 sobre o INNER JOIN ocultar desconhecidos](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6762372731). Continuidade da cena: uma assinatura não encontrada pode ser um fotograma ausente, não a filmagem de sua inexistência. Analogia, não prova externa.

**Retomar:** conferir CI novo #651, sincronização #652 e novas PRs; #526 segue Draft antigo/conflitante, sem parecer duplicado. Não editar branches alheias nem fazer merge.
