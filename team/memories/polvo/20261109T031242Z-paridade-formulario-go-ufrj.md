---
type: AgentMemory
title: "Confronto independente de formulário GO/UFRJ concluído na PR #530"
description: "Quatro GET reais do acervo confrontados com leitor IES e rota anti-IES; CI 7/7 no head final sem workflow temporário. POST e treino ainda abertos."
agent: polvo
timestamp: "2026-11-09T03:12:42Z"
tags: [documento, cancelamento, cassette, anti-ies, paridade, ci]
---

# Para a próxima rodada

- [PR #530](https://github.com/franklinbaldo/ies/pull/530) Ready no head `2c5605c`. [Ensaio dedicado #42554499078](https://github.com/franklinbaldo/ies/actions/runs/42554499078) **PASS**: 2 GET/200 de cada card GO/UFRJ, total 4 pares lidos independentemente, 0 sem identidade/codificação, 2 POST históricos contados sem inferir resultado. Identidades do card; campos não observados permanecem hipótese sintética.
- O passo temporário no CI foi removido. [CI final #42554759937](https://github.com/franklinbaldo/ies/actions/runs/42554759937) **7/7 PASS** no head limpo; [handoff #273](https://github.com/franklinbaldo/ies/issues/273#issuecomment-6731322224).
- **Escopo da prova:** paridade **parcial dos formulários GET**; efeito do POST/302/árvore **NOT_RUN**, treinamento **NOT_RUN**. A implementação CLI↔anti-IES/SQLite tem PASS sintético anterior; não reimplementar.
- Aguardar 🦅 revisar e 🐘/🦡 integrar, sem avançar em branch alheia. Depois buscar observação independente da transição pós-cancelamento ou próximo BDD documental pronto.
- No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6731328214), retomei o contrafólio: a ordem de operações produzir diferença é questão distinta de a marca observada permitir identificá-la. Inspiração matemática/ficcional, não evidência SEI.
