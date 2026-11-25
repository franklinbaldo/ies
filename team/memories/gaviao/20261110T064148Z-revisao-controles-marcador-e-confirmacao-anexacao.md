---
type: AgentMemory
title: "Revisão de controles de marcador e confirmação de anexação"
description: "Heads #644 e #636 revisados com CI 7/7; estrutural GET distinto de POST causal; decisão de anexação permanece desatualizada."
agent: gaviao
timestamp: "2026-11-10T06:41:48Z"
tags: [revisao, bdd, anti-ies, cassettes, anexacao, marcadores, ci, cafezinho]
---

# Próxima tomada

**Fato #644.** Head `b36bb46`, [parecer](https://github.com/franklinbaldo/ies/pull/644#issuecomment-6760910168), [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42768704998), atrás da `main` em zero commits na checagem anterior à memória. A projeção do anti-IES recupera controles da tela de marcadores no GET e um E2E observa esses sinais via HTTPS local, mas **não** executa POST. BDD-255-01/02 parciais; inspeção de hashes [UnB/GO](https://github.com/franklinbaldo/ies/actions/runs/42767722412) não equivale a confronto semântico independente; live NOT_RUN.

**Fato #636.** Head `c588b06`, [parecer](https://github.com/franklinbaldo/ies/pull/636#issuecomment-6760910740), [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42767244366), `behind=1` na checagem. BDD-270-02 PASS sintético com CLI Rust real/anti-IES/SQLite, POST único, segundo comando de releitura e saída 3 em 503 pós-escrita. Houve correção de causa tipada/Serde sem novo subcódigo. **Divergência demonstrada**: `docs/decisoes/processo-anexar.md:7,49-51,62,73-75` ainda descreve confirmação bloqueada, embora contrato e código a habilitem para treinamento. Atualizar a decisão existente; sincronizar `main`, novo CI 7/7. Cassette UnB/GO tem inspeção de integridade, não replay independente; live NOT_RUN.

**#526** segue Draft, head `5188493`, conflitante; preservar o E2E existente e não repetir parecer antigo.

**Cafezinho:** lido [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente [a observação da 🦊](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6760627379): o botão está na fotografia, mas isso não prova que a ação foi executada; do mesmo modo, uma lista após o POST não substitui prova de resposta e releitura. Analogia de montagem, não evidência externa.

Retomar apenas heads novos; sem implementação, merge ou acesso ao SEI pelo 🦅.
