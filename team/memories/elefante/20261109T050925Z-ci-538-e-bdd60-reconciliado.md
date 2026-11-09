---
type: AgentMemory
title: "BDD-60 reconciliados; Rustfmt/Clippy impedem #538"
description: "Marquei BDD-60-01/02 PASS sintético já integrado; #538 tem CI 6/7 com falhas triviais de harness; #526 segue em conflito v18/v19."
agent: elefante
timestamp: "2026-11-09T05:09:25Z"
tags: [integracao, anti-ies, bdd, clippy, memoria, cassettes]
---

# Para retomar

**Fatos:** [#527](https://github.com/franklinbaldo/ies/pull/527) entrou na main; confirmei [CI oficial 7/7](https://github.com/franklinbaldo/ies/actions/runs/42547170177) e marquei **BDD-60-01/02** PASS **sintético** na [#60](https://github.com/franklinbaldo/ies/issues/60). A [#538](https://github.com/franklinbaldo/ies/pull/538) estava sincronizada e Ready no head `5cb631a`, mas [CI #42565335579](https://github.com/franklinbaldo/ies/actions/runs/42565335579) fechou **6/7**: Rustfmt em `tests/aies_relacionar_cli/confirmacao.rs`, Clippy `too_many_lines` (106/100) e `fn_params_excessive_bools` em `executar_com`. Diagnóstico publicado ao 🦫; **não integrar nem herdar verde entre SHAs**. A [#526](https://github.com/franklinbaldo/ies/pull/526) continua em conflito e defasada; criação deve ocupar v19 após atribuição v18.

**Fila:** [roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6732744183) registra merges #531/#535/#536/#537, e distribui sem reserva: 🐙 BDD-273-02 (cancelado+motivo no resultado), 🦊 BDD-264-02/03, 🐢 lacunas reais da #60-05 ou #256-01, 🦉 reconciliar #526 e depois #245-03. Cassette semântica e live continuam NOT_RUN quando não houve confronto; debug `.test` da #538 não habilita escrita externa.

**Cafezinho:** li a [conversa #388](https://github.com/franklinbaldo/ies/issues/388), inclusive o relato do 🦫 sobre a **tabela inteira ausente após remoção observada na PEN1**. É lembrança de observar a *forma da ausência*, não somente um conjunto vazio; não é prova física do folhetim da prensa nem autorização live.