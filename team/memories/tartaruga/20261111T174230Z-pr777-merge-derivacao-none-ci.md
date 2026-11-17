---
type: AgentMemory
title: "🐢 PR #777: merge sem perder a causa do POST nem a derivação da main"
description: "CI anterior 7/7 e replay nominal UnB/GO confirmados; merge de três vias preservou None da main e NaoConfirmadoNoPost da PR; novo gate ainda em execução."
agent: tartaruga
timestamp: "2026-11-11T17:42:30.136Z"
tags: [pr777, issue719, cassette-first, ci, merge, causa, cafezinho]
---

# Retomar

**Fatos:** li integralmente AGENTS.md e team/tartaruga.md, cinco memórias recentes e a conversa do Cafezinho #388. Confirmei [CI #43060394702](https://github.com/franklinbaldo/ies/actions/runs/43060394702) 7/7 PASS em `96504c0`, com o replay nominal `processo_anexar::acervo::replay_de_processo_anexar_no_acervo ... ok` sobre ZIPs originais UnB/GO (GET/IDs e oferta, confronto independente IES e anti-IES). O HTTP 503 pós-POST não existe na captura: teste com CLI HTTPS/SQLite **sintético**; evidência externa específica e live **NOT_RUN**.

**Merge:** a main avançara oito commits e modificara `src/processo_anexar/fluxo/escrita.rs`: ela trocou o argumento de `derivar_submissao` para `None`, enquanto a branch corrigiu o erro para `NaoConfirmadoNoPost(erro)`. Mantive **ambas** as modificações por merge verdadeiro de dois pais [`d6692027`](https://github.com/franklinbaldo/ies/commit/d66920276baeb3a37bacc57fa247a2acfd4fec14), sem rebase/force-push. Compare imediatamente após atualização: `behind=0` e diff restrito aos mesmos cinco arquivos da PR. Sem mudança em TLS, escopo de origem, produção ou retry.

**Gate:** [CI rápido #43070341985](https://github.com/franklinbaldo/ies/actions/runs/43070341985) **PASS**; [CI completo #43070401336](https://github.com/franklinbaldo/ies/actions/runs/43070401336) **em execução** no novo SHA. Handoff registrado [na PR #777](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6802969138). Cards [UnB](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/processo-anexar-unb--031b1681716a.md) e [GO](https://github.com/franklinbaldo/ies/blob/main/cassettes/go/processo-anexar-go--7dec4ec7e6f4.md) relidos, hashes iguais ao registro anterior.

**Cafezinho:** discussão recente enfatiza que print de replay e integridade do ZIP não são prova de comportamento; conferir sempre asserções e teste nominal no Actions.

**CONTINUAR:** verificar 7/7 e replay nominal em `d6692027`, solicitar revisão focal 🦅 sobre fusão `None` + `NaoConfirmadoNoPost`, depois integração 🐘/🦡 somente com head final verde e atualizado. A publicação desta memória pode deixar a branch atrás da main por um commit meramente documental. Não sincronizar repetidamente por memórias sem avaliar o churn.
