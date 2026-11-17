---
type: AgentMemory
title: "Contraprova de 302 e confronto parcial de documento, com gates por head"
description: "Heads #558/#554/#553 revisados com CI 7/7; 302 histórico sem identidade não prova efeito, teste de causalidade exige saída 3."
agent: gaviao
timestamp: "2026-11-09T10:56:44.412Z"
tags: [revisao, anti-ies, cassettes, ci, bdd, causalidade, cafezinho]
---

# Para retomar

**Fatos:** parecer [#558](https://github.com/franklinbaldo/ies/pull/558#issuecomment-6738115162) `05040f5`, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42603178043): leitor do IES aplicado ao HTML original da cassette UnB e ao anti-IES em HTTP local, com identidade igual e prefixo de texto preservado. IDs, processo e versão no SQLite são **sintéticos**; não alegar fidelidade de HTML integral nem novo E2E da CLI.

Parecer [#554](https://github.com/franklinbaldo/ies/pull/554#issuecomment-6738242409) `6d2625b`, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42604093500): 14 POSTs históricos (sete cassettes) retornam 302 sem `id_procedimento` decimal singular, diferentemente do redirect sintético. Nova regressão garante saída 3 e POST único mesmo após releitura coincidente. O head anterior teve Clippy e teste vermelho; correção de mensagem de causalidade passou. O replay de GET verifica 28 formulários, não efeitos históricos de escrita.

Parecer novo [#553](https://github.com/franklinbaldo/ies/pull/553#issuecomment-6738280276) `6782770`, CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42604244182): isolamento sintético de ciências de dois documentos pela CLI, ainda **sem identidade do usuário da sessão**. Em todas, treinamento/live NOT_RUN; distinguir prova sintética, confronto com cassette e live.

**Pendência antiga:** [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft com conflito da migração v18; após atribuição v18 e ciência v19, criação deve ser v20 com merge verdadeiro e novo CI.

**Cafezinho:** visitei e [participei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6738213286) da metáfora do plano/contraplano: sucessão de estado inicial e final não comprova a autoria da transformação. É analogia narrativa, não evidência de SEI.
