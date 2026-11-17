---
type: AgentMemory
title: "Relação bilateral sem inventar observação"
description: "O RED da #491 revelou que o vínculo registrado em uma ponta desaparecia quando a outra era consultada; a leitura agora é bidirecional sem transformar não observado em vazio."
agent: castor
timestamp: "2026-11-08T18:40:57Z"
tags: [bdd, rust, sqlite, anti-ies, causalidade]
---

# Encaixe para a próxima rodada

Na [PR #491](https://github.com/franklinbaldo/ies/pull/491), o [RED #42510592546](https://github.com/franklinbaldo/ies/actions/runs/42510592546) falhou **um teste Rust**: registro 10→22 estava visível em 10, porém a consulta do processo 22, embora observado, retornava `Some([])`. A consulta agora usa uma única relação em ambas as direções, sem exigir duas escritas ou inventar a observação da outra ponta. [GREEN #42510805813](https://github.com/franklinbaldo/ies/actions/runs/42510805813) passou 7/7 no head `2472f897`.

**Retomada:** a PR é Draft **por sequência do esquema**, não por falha de comportamento. Aguardar #487/v13 e #489/v14; depois merge verdadeiro da main, verificar ordem 0012→0013→0014→0015 e CI no SHA final. BDD-258-02/03/04 seguem sem prova de confirmação remota. Inspeção de hashes PEN1 ≠ comparação semântica nem live. Nenhum subcódigo novo.

No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6725474300), a marca de pareamento nas duas metades de uma junta foi uma analogia sobre **evidência bilateral versus cronologia ainda desconhecida** — ficção, não prova sobre SEI.
