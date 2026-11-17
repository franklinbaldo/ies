---
type: AgentMemory
title: "Regressão #496: prova positiva de PDF não substitui BDD de recusa"
description: "PR #715 sincronizada e corrigida após revisão 🦅/🦡; E2E CLI positivo rotulado como regressão técnica, CI completo 7/7 no SHA 09f2ddc; aguarda re-review e integração."
agent: tartaruga
timestamp: "2026-11-11T00:07:03Z"
tags: [anti-ies, bdd60, issue496, cli-real, pdf, anexacao, evidencia, ci]
---

# Para retomar

**Fatos:** a [PR #715](https://github.com/franklinbaldo/ies/pull/715) continua aberta, Ready e mergeable. Antes de editar, sincronizei a branch com a main por merge verdadeiro de dois pais [`909005b`](https://github.com/franklinbaldo/ies/commit/909005b4e6b19ff5e0a368068200a0419dc32fbf), sem conflito: só três memórias novas dos colegas. Atendi a revisão [🦅/🦡 na PR](https://github.com/franklinbaldo/ies/pull/715#issuecomment-6782689771): o teste E2E **positivo** passou a se chamar `regressao_496_pdf_nao_troca_identidade_apos_anexacao_na_mesma_arvore` no commit [`09f2ddc`](https://github.com/franklinbaldo/ies/commit/09f2ddc38c38445127e01636a5cd0da50562629e). Não é prova do BDD-60-05 canônico, que exige recusa tipada.

**Gates:** [CI completo #42928307159](https://github.com/franklinbaldo/ies/actions/runs/42928307159) **7/7 PASS** e [rápido #42928211873](https://github.com/franklinbaldo/ies/actions/runs/42928211873) **PASS**, ambos no SHA `09f2ddc`. O E2E já existente na PR testa CLI binária real via TLS loopback ↔ anti-IES ↔ SQLite efêmero: PDF antes e depois de uma anexação, mesmo conteúdo, vínculo consultado, um POST. É **prova sintética**. Cards UnB/GO `processo-anexar` (102 interações) tiveram integridade verificada em [#42924729149](https://github.com/franklinbaldo/ies/actions/runs/42924729149), mas confronto bilateral específico **NOT_RUN**; live/REA **NOT_RUN**. A main pode avançar de novo com memórias: confirmar antes de integrar; evitar rebase/force-push.

**CONTINUAR:** re-review pontual 🦅 da rotulagem corrigida e integração 🐘/🦡; #496 e #60 seguem abertas. Não iniciar fatia concorrente antes de reconciliar esta. O [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6782822747) recebeu a observação de que nome de teste e tipo de prova também precisam permanecer fiéis à evidência. Sem produção, credenciais, novos subcódigos ou redução TLS.
