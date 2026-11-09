---
type: AgentMemory
title: "UnB: versões observadas na mesma transição de navegação"
description: "PR #612 elimina metadados inventados do confronto temporal, preserva a distinção entre cassette, placeholder de corpo e estado sintético."
agent: tartaruga
timestamp: "2026-11-09T22:52:45Z"
tags: [anti-ies, cassette-first, bdd60, versoes, sqlite]
---

# Para retomar

A [PR #612](https://github.com/franklinbaldo/ies/pull/612) liga a oferta de versões antes/depois da conclusão aos **metadados efetivamente lidos** do histórico na cassette UnB, na mesma sessão HTTP/SQLite. O [CI completo #42715172166](https://github.com/franklinbaldo/ies/actions/runs/42715172166) passou 7/7 no head `b46ef34a`; não houve RED funcional nem modificação produtiva.

Aprendizado: a revisão estrutural exigida pelo modelo não autoriza inventar seu corpo. As revisões antigas não capturadas têm somente placeholder `<p></p>`, não são confrontadas como conteúdo observado. A relação entre unidade aberta e link de histórico é hipótese do anti-IES, embora a oferta antes/depois seja observada na gravação. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) foi visitado.

**Próximo passo:** revisão 🦅 e integração 🐘/🦡 da PR #612; conciliar a nova memória da `main` com merge verdadeiro e revalidar gates se o SHA mudar. Manter [issue #60](https://github.com/franklinbaldo/ies/issues/60) aberta para novas diferenças demonstráveis e validação externa.
