---
type: AgentMemory
title: "🐢 PR #777: replay original validado, novo merge aguarda gate"
description: "CI 7/7 e replay UnB/GO confirmados no d6692027; reconciliação com main em eb06db9, novo CI oficial iniciado e pendente."
agent: tartaruga
timestamp: "2026-11-11T18:42:25Z"
tags: [pr777, issue719, cassette-first, github-actions, merge, cafezinho]
---

# Retomar

**Fatos:** reli AGENTS.md, team/tartaruga.md e cinco memórias recentes, e visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Confirmei [Actions #43070401336](https://github.com/franklinbaldo/ies/actions/runs/43070401336) **7/7 PASS** no head `d6692027`; log nominal executou o replay de anexação UnB/GO, confrontando IES e anti-IES/SQLite separadamente contra os GETs originais. A matriz externa cobre formulário, ação, controles e IDs, **não** o POST histórico.

**Merge:** sem sobreposição nos cinco arquivos próprios, incorporei 9 commits da `main@8af0322` por merge de dois pais no [HEAD `eb06db9`](https://github.com/franklinbaldo/ies/commit/eb06db9d94268631f7e037ed2793eddc9f69651f). Compare logo após: `behind=0`, diff restrito aos cinco arquivos. [CI rápido #43080081193](https://github.com/franklinbaldo/ies/actions/runs/43080081193) **PASS**; [CI oficial #43080129451](https://github.com/franklinbaldo/ies/actions/runs/43080129451) acionado e **queued** na consulta. [Handoff da PR](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6804299896).

**Limite:** HTTP 503 após POST com CLI real/HTTPS local/SQLite é **PASS sintético no SHA anterior**; comparação com cassette original dessa transição e live são **NOT_RUN**. No HEAD novo não herdar nenhum PASS de replay até conferir o job pertinente. Não usar contagem fixa de requisições para equivalência.

**Cafezinho:** a discussão sobre base64 corrompido e replay que imprime sucesso sem falhar reforça conferir os asserts e o teste nominal. **CONTINUAR:** confirmar 7/7 + replay de `processo_anexar::acervo::replay_de_processo_anexar_no_acervo` em `eb06db9`, revisão 🦅 e integração 🐘/🦡 somente se head final ainda válido. Não criar nova rodada de merges apenas por memórias, salvo necessidade real. Sem produção, segredos, retry, TLS enfraquecido ou novos subcódigos.
