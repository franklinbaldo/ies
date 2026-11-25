---
type: AgentMemory
title: "BDD-60-05: URL antiga não entrega corpo após cancelamento"
description: "PR #533 corrige o endpoint direto de visualização após cancelamento persistido; RED HTTP 200 indevido e GREEN 7/7; paridade de cassettes ainda não demonstrada."
agent: tartaruga
timestamp: "2026-11-09T03:45:26Z"
tags: [anti-ies, documento, cancelamento, http, sqlite, bdd, cassette]
---

# Para retomar

[Fato: PR #533](https://github.com/franklinbaldo/ies/pull/533) Ready para revisão/integração: a árvore não mais publicava `src` para cancelado, mas a URL `documento_visualizar` antiga ainda devolvia 200 com corpo. [RED #42557512081](https://github.com/franklinbaldo/ies/actions/runs/42557512081) reproduziu **1 FAIL, 2 PASS** (Rustfmt também falhou separadamente). A rota passou a consultar `Situacao` no mesmo SQLite e negar com 403. [GREEN #42557776963](https://github.com/franklinbaldo/ies/actions/runs/42557776963) **7/7** e CI rápido aprovado no SHA `367304a9fdb07af734efa9277dbcf31476c199f2`. O teste novo é **HTTP/SQLite**, não novo E2E de CLI, embora os E2E prévios tenham passado.

[Fato: inspeção #42557504762](https://github.com/franklinbaldo/ies/actions/runs/42557504762): cards GO cancelamento e UnB documento, **95 interações**, hashes e estrutura PASS; confronto semântico independente e live **NOT_RUN**. Marcador de inspeção removido. Reusar a URL cancelada é hipótese de coerência sintética, não prova do SEI.

[Conversa no Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388): uma imagem final pode esconder a sequência de operações. A analogia sugere testar o estado posterior com a **mesma URL anterior**, em vez de confiar só no ícone da árvore, mas não constitui evidência externa.

**Próximo:** conferir integração da #533 e procurar apenas a próxima lacuna BDD já prevista, sem refazer a correção. Sem produção, credenciais, subcódigos ou redução de TLS.
