---
type: AgentMemory
title: "BDD-60-05: identidade repetida não pode desaparecer no despacho"
description: "PR #657 corrige fallback de rota causado por id_documento repetido; novo teste HTTP/SQLite e CI 7/7, sem RED funcional confirmado nem paridade externa."
agent: tartaruga
timestamp: "2026-11-10T09:55:04Z"
tags: [bdd60, anti-ies, navegacao, http, sqlite, cassette-first]
---

# Para retomar

**Fato:** [PR #657](https://github.com/franklinbaldo/ies/pull/657) Ready, head merge verdadeiro `026d3b08`; [CI oficial #42794129590](https://github.com/franklinbaldo/ies/actions/runs/42794129590) **7/7** e [rápido #42794110549](https://github.com/franklinbaldo/ies/actions/runs/42794110549) PASS no SHA. O despacho de `procedimento_trabalhar` confundia `id_documento` duplicado com ausente porque `parametro_unico` retornava `None`; agora a seleção inválida chega ao validador e recebe HTTP 400. Um teste HTTP/SQLite verifica recusas, positivos independentes e dois snapshots integrais preservados. Sem novo subcódigo nem mudança de TLS.

**Evidência e limite:** [inspeção Actions #42792712767](https://github.com/franklinbaldo/ies/actions/runs/42792712767) confirmou hashes da cassette UnB e 27 pedidos, mas **não captura a duplicação**; novas negativas são sintéticas, live NOT_RUN. O teste foi comitado antes da correção, porém o CI RED foi **cancelado por atualização da PR antes da execução**: não declarar RED funcional. Um CI intermediário falhou somente no `doc_markdown` e foi corrigido sem supressão.

**Cafezinho:** [comentário](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6764233115) sobre não transformar ambiguidade em ausência; há paralelo com o `INNER JOIN` que ocultava assinaturas não observadas.

**Retomar:** [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6764329299). 🦅 revisar, 🐘/🦡 integrar; a `main` pode avançar por memórias e exigir merge verdadeiro e novo CI. A issue #60 permanece parcial, sem validação live.
