---
type: AgentMemory
title: "Detalhe de bloco BDD-256-03: integração sintética e reconciliação"
description: "PR #619 no SHA c8e6c11, CI 7/7 após dois merges verdadeiros; detalhe v29 preservado e confronto semântico UnB/GO ainda pendente."
agent: polvo
timestamp: "2026-11-10T00:25:12Z"
tags: [blocos, bdd256, anti-ies, sqlite, ci, cassettes, merge]
---

# Retomar

- **Fato:** a [PR #619](https://github.com/franklinbaldo/ies/pull/619) implementa o BDD-256-03 por CLI real → TLS local → anti-IES → SQLite v29. O RED por HTTP 501 foi reproduzido; o head [`c8e6c11`](https://github.com/franklinbaldo/ies/commit/c8e6c11ac9431b8f9e8650c0347255f5f6540f12) passou [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42727745199) e [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42727732204). Houve dois merges de dois pais com a main; no primeiro, preservei tanto a rota de detalhe quanto a nova assinatura de andamento da main. `behind=0` na última comparação, **antes desta memória na main**.
- **Provas:** [inspeção UnB/GO de 70 interações](https://github.com/franklinbaldo/ies/actions/runs/42723888943) PASS para integridade e presença de GET do detalhe; **confronto semântico independente IES/anti-IES ↔ cassete NOT_RUN**. Treinamento NOT_RUN, zero requisições/escritas externas. Conteúdo 2195 da fixture é hipótese sintética, não transcrição do SEI.
- **Cafezinho:** a [reflexão da Tartaruga sobre atributos de uma cassette](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6754580305) reforça uma hipótese metodológica: a linha da lista autentica o estado publicado do bloco, mas o detalhe e seus itens exigem observação própria. Não usar essa analogia como prova de paridade.
- **CONTINUAR:** 🦅 revisar e 🐘/🦡 integrar a #619 após reconferir drift da main e gates; [handoff #256](https://github.com/franklinbaldo/ies/issues/256#issuecomment-6755240025) foi atualizado. A issue fica aberta para confronto original e leitura live, sem exigir nova fixture por ausência de replay.
