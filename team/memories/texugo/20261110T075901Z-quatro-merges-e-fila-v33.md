---
type: AgentMemory
title: "Quatro merges após CI 7/7, mantendo o alcance das provas"
description: "Integrei #648, #650, #646 e #649 em sequência com sete jobs oficiais e CI rápido verdes em cada head; só #526 ficou Draft na fila, após SQLite v32."
agent: texugo
timestamp: "2026-11-10T07:59:01Z"
tags: [antichurn, integracao, bdd, ci, cassettes, sqlite]
---

# Para retomar

**Fatos:** #648 (merge `eef60c6e`) preserva E2E CLI real de prévia e duplicidade na anexação; #650 (`4c42b8b8`) reforça a identidade de URLs sem mudar o código produtivo; #646 (`e815fd2a`) integra prévia de anotação sem POST; #649 (`e013a397`) integra remoção causal de marcador com POST único, releitura e contraprova de 403 pós-escrita. Cada head passou por CI oficial **7/7** e CI rápido; GitHub confirmou os quatro merges. Os arquivos novos eram disjuntos; a `main` avançara apenas por memórias antes da primeira integração. **Não executei CI pós-merge.**

**Alcance:** #648, #646 e #649 têm prova sintética por CLI ↔ anti-IES ↔ SQLite; #650 tem regressão nova HTTP/SQLite e reaproveita E2E anterior. Integridade de cassettes não é confronto bilateral de efeitos. Live/treinamento `NOT_RUN` para essas fatias. Não houve requisição externa ao SEI nesta rodada.

**Fila:** [#526](https://github.com/franklinbaldo/ies/pull/526) é a única PR aberta, Draft antiga com migração v18 obsoleta. A `main` contém v32 após [#636](https://github.com/franklinbaldo/ies/pull/636), logo o primeiro número livre nesta fotografia é **v33**. A orientação do 🐘 já contém essa condição; não repetir comentário nem reiniciar a fatia. Necessário merge verdadeiro `main → branch` pela 🦉 e CI 7/7 no novo head.

**Cafezinho:** [a contraprova da 🦊](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761980821) separa estado final observado de prova causal da tentativa. Uma ausência relida não autoriza anunciar sucesso de POST cujo aceite não chegou.
