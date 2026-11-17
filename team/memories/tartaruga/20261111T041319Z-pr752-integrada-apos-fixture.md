---
type: AgentMemory
title: "PR #752 integrada: correção de fixture após #749 e GREEN no head"
description: "Adendo à memória 04:04: erro de fixture resolvido, CI oficial 7/7 e rápido PASS no ccc1f89b; PR #752 integrada pelo merge e0a1171."
agent: tartaruga
timestamp: "2026-11-11T04:13:19Z"
tags: [issue719, utf8, confidencialidade, ci, merge, teste, cafezinho]
---

# Atualização aditiva à memória 04:04

A [PR #752](https://github.com/franklinbaldo/ies/pull/752) está **integrada** à `main` no [merge e0a1171](https://github.com/franklinbaldo/ies/commit/e0a117118a037dd897430627d47bf0aa81b99835). O [CI completo #42957115800](https://github.com/franklinbaldo/ies/actions/runs/42957115800) passou **7/7** e o [rápido #42957099097](https://github.com/franklinbaldo/ies/actions/runs/42957099097) também passou, ambos no head `ccc1f89b`. A issue #719 permanece aberta para outras ablações; handoff no [comentário](https://github.com/franklinbaldo/ies/issues/719#issuecomment-6786804863).

O CI intermediário descobriu que o teste da sentinela usava `text/plain` sem `Content-Disposition`; depois da #749, a leitura recusa a ausência do nome **antes** de alcançar o byte inválido. Corrigi o cabeçalho da **fixture**, sem afrouxar o contrato ou mudar produção. A prova é técnica local e sintética, não replay semântico de cassette ou live. Nenhuma credencial, novo subcódigo ou redução de TLS.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6786810087), registrei a observação de que um teste de segurança precisa alcançar a fronteira que pretende medir. **Próxima rodada:** retomar a prioridade de antipatterns #716/#719, consultando PRs ativas e BDDs prontos antes de assumir outra fatia.
