---
type: AgentMemory
title: "Revisões da orientação de atribuição, andamento e confronto UnB"
description: "Pareceres novos #615/#614/#618; #618 falha Clippy 124/100, #615 verde 7/7 mas main avançou, #614 não tem CI completo no head."
agent: gaviao
timestamp: "2026-11-09T23:44:14Z"
tags: [revisao, bdd, ci, clippy, cassettes, andamento, atribuicao, cafezinho]
---

# Retomar

**Fatos:** [#615](https://github.com/franklinbaldo/ies/pull/615#issuecomment-6754525017), head `5bd7385`, corrige orientação após efeito incerto para **não repetir confirmação**; CI oficial [#42719839960](https://github.com/franklinbaldo/ies/actions/runs/42719839960) **7/7**, mas branch estava 2 commits atrás da main. Mérito aprovado; merge verdadeiro e novo CI antes de integrar.

[#614](https://github.com/franklinbaldo/ies/pull/614#issuecomment-6754525523), head `380deda`: BDD-259-02 traz duas execuções CLI real, POST único e segunda leitura do SQLite; rápido [#42719511990](https://github.com/franklinbaldo/ies/actions/runs/42719511990) verde e inspeção dos cards [#42719511426](https://github.com/franklinbaldo/ies/actions/runs/42719511426) verde, mas sem 7/7 no head e 19 commits atrás da main. Remover manifesto de inspeção temporário; BDD-259-03 ainda não entregue. [#616](https://github.com/franklinbaldo/ies/pull/616) duplicada fechada sem merge; RED dali não é GREEN da #614.

[#618](https://github.com/franklinbaldo/ies/pull/618#issuecomment-6754525972), head `3a88094`: confronto proposto usa versão/autoria da cassette UnB em vez de dados sintéticos; [CI oficial #42722335244](https://github.com/franklinbaldo/ies/actions/runs/42722335244) **6/7**. Clippy concreto `too_many_lines` **124/100** em `src/cassette/anti_ies_documento.rs:50`; corrigir por extração coesa sem suprimir lint. Teste marcado `#[ignore]`: sem execução específica, confronto semântico **NOT_RUN**, mesmo se os sete gates futuros ficarem verdes. Teste não é E2E novo da CLI nem validação live.

[#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft antiga/conflitante; preservar E2E da criação (duas CLIs, POST único) e escolher migração livre após merge da main.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); a conversa recente de 🦫 distingue inspeção de integridade de reprodução do estado. Comparação útil com os metadados da #618: restaurar os créditos de um filme a partir do negativo original não prova que outra cena foi filmada. Analogia, não evidência técnica.

**Provas:** compatibilidade sintética só quando executada no SHA; integridade de cassette não é confronto semântico; live não ocorreu nesta revisão. Sem implementação, merge ou acesso ao SEI pelo Gavião.
