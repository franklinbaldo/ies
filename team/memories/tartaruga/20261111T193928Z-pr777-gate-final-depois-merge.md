---
type: AgentMemory
title: "🐢 PR #777: 7/7 no head antigo; novo merge de memórias e replay obrigatório"
description: "Revalidado CI/replay UnB-GO em eb06db9; main documental reconciliada com branch no 878b713 e novo CI oficial ainda pendente."
agent: tartaruga
timestamp: "2026-11-11T19:39:28.278Z"
tags: [pr777, issue719, cassette-first, replay, github-actions, merge, cafezinho]
---

# Retomar

**Fatos:** reli integralmente `AGENTS.md`, `team/tartaruga.md` e as cinco memórias recentes; consultei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), em especial a discussão de hashes versus testes executados. O [CI #43080129451](https://github.com/franklinbaldo/ies/actions/runs/43080129451) encerrou **7/7 PASS** no SHA `eb06db9`; log nominal confirma `processo_anexar::acervo::replay_de_processo_anexar_no_acervo`, usando ZIPs originais UnB/GO com hashes verificados, GET original e confronto independente dos fatos com IES e HTTP anti-IES.

**Merge:** sete commits de `main@ff36162` eram exclusivamente memórias OKF, sem sobreposição nos cinco arquivos da PR. Merge verdadeiro de dois pais em [`878b713`](https://github.com/franklinbaldo/ies/commit/878b71383a78db5bdb025993b416112e4db8804f), sem rebase nem force-push. Compare após a atualização: `behind=0`, cinco arquivos próprios apenas. [CI rápido #43088908032](https://github.com/franklinbaldo/ies/actions/runs/43088908032) **PASS**; [CI oficial #43088953434](https://github.com/franklinbaldo/ies/actions/runs/43088953434) **em execução** no SHA final. [Handoff na PR](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6805495012).

**Limites:** IES↔cassette e anti-IES↔cassette têm PASS apenas no SHA anterior até concluir teste nominal no novo; CLI↔anti-IES↔SQLite com 503 pós-POST tem PASS sintético anterior, não original. A captura original não contém esse 503 e live é NOT_RUN. Não medir paridade por contagem de requests.

**CONTINUAR:** conferir 7/7 e replay nominal no head `878b713`, solicitar revisão focal 🦅 e repassar a 🐘/🦡 para squash. A publicação desta memória na main pode deixar a branch atrás por alteração documental: avaliar sem gerar churn automático. Sem produção, segredos, retry, subcódigos ou relaxamento TLS.
