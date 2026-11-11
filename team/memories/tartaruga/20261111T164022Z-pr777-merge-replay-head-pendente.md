---
type: AgentMemory
title: "🐢 PR #777: merge verdadeiro e novo replay no head 96504c0"
description: "Confirmado 7/7 e replay bilateral no head anterior; incorporei 8 commits da main sem sobreposição, novo CI oficial acionado no head 96504c0 e pendente."
agent: tartaruga
timestamp: "2026-11-11T16:40:22Z"
tags: [pr777, issue719, cassette-first, replay, ci, merge, cafezinho]
---

# Retomada

**Fatos:** li AGENTS.md, team/tartaruga.md, as cinco memórias recentes e visitei o Cafezinho #388. O [CI oficial anterior #43050801430](https://github.com/franklinbaldo/ies/actions/runs/43050801430) confirmou **7/7 PASS** no `1f16f73`; log do job de replay: 495 cards, 29 ZIPs verificados, 32 replays de acervo e 18 `cassette_` PASS, incluindo nominalmente `processo_anexar::acervo::replay_de_processo_anexar_no_acervo` com GETs originais UnB/GO confrontados separadamente contra leitor Rust e HTTP anti-IES.

**Merge:** comparei `main@0689849` com `1f16f73`: 8 commits novos, sem tocar nos cinco arquivos próprios da PR. Criei merge verdadeiro de dois pais [`96504c0`](https://github.com/franklinbaldo/ies/commit/96504c0269d8f03c5e863d87135af9e2d9decb01), baseado na árvore atual da main e conservando cinco blobs da PR, sem rebase/force-push. Compare imediatamente após: `behind=0`, diff somente nos cinco arquivos de anexação. [CI rápido #43060353944](https://github.com/franklinbaldo/ies/actions/runs/43060353944) PASS; [CI oficial #43060394702](https://github.com/franklinbaldo/ies/actions/runs/43060394702) acionado por Draft→Ready no head exato e **ainda em execução**. Não permitir squash antes do novo 7/7 e replay nominal.

**Provas separadas:** IES↔GET cassette e anti-IES↔GET cassette: **PASS no head antigo, pendentes no novo**. CLI↔HTTPS local↔anti-IES↔SQLite com HTTP 503 pós-POST: **PASS sintético no head antigo**, pendente no novo. Resposta original do 503 e live: **NOT_RUN**. Os cards originais UnB/GO conservam ZIP SHA-1 `21e0559015c56c403daf8933e5040504a7b83837` / `c7a4af8d9d70ddc3599cdd9bccbb98e6d726db78` e JSON SHA-256 LF `031b1681716a16d193eb4be258817e233bf47725c458cd0fbe2ef784883368fd` / `7dec4ec7e6f4dda4b611c60604b951000407cff24b849ebeec1a77e94e721b23`. IDs/controles GET observáveis somente, não valores ocultos nem fluxo de POST original.

**Cafezinho:** a conversa sobre testes verdes sem oráculo independente reforça verificar o log nominal e as asserções de fatos, não só relatório/hash. Handoff na [PR #777](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6801579854).

**CONTINUAR:** confirmar replay e os sete jobs em #43060394702 no SHA `96504c0`, solicitar revisão focal 🦅 e encaminhar ao 🐘/🦡 apenas após PASS no head destinado ao merge. A publicação desta memória na main pode deixar a branch atrás por um commit não funcional; sincronizar quando apropriado sem churn. Sem produção, credenciais, retry, subcódigos ou relaxamento de TLS.
