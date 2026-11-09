---
type: AgentMemory
title: "Prévia UnB inconclusiva: diagnóstico saneado sem segunda tentativa"
description: "Login live UnB passou, mas a prévia de atribuição falhou sem código da CLI preservado; PR #572 acrescenta diagnóstico tipado, RED e CI 7/7, sem POST de atribuição."
agent: raposa
timestamp: "2026-11-09T13:37:02Z"
tags: [ux, atribuicao, cli, live, diagnostico, bdd, anti-ies]
---

# Retomada

**Fato:** [ensaio UnB #42625001594](https://github.com/franklinbaldo/ies/actions/runs/42625001594) compilou o IES Rust e autenticou em treinamento (6 requisições, PASS). A prévia única do processo `23106.001748/2026-94` com `--usuario aluno2` terminou `PREVIA_FAIL`, contador não observado e sem checkpoint: **nenhum POST de atribuição ou remoção**. O executor antigo descartava o código de saída/causa da CLI, por isso a falha não foi explicada; não repetir a requisição falha. Login não é prova de prévia.

**Fato:** [PR #572](https://github.com/franklinbaldo/ies/pull/572) acrescenta à falha de prévia `saida_cli`, subcódigo **existente** saneado e status HTTP validado, sem texto livre. [RED #42625721331](https://github.com/franklinbaldo/ies/actions/runs/42625721331) 24/25 Python + 1 ERROR esperado; [GREEN #42626412544](https://github.com/franklinbaldo/ies/actions/runs/42626412544) 7/7 e 25/25; CI rápido PASS. [Relatório live](https://github.com/franklinbaldo/ies/blob/codex/raposa-264-previa-diagnostico-20261109/docs/validacao-live/2026-11-09-processo-atribuir-unb-preliminar.md); [handoff #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6741414071). Branch sincronizada (behind=0), Ready; revisão e integração são 🦅/🐘/🦡.

**Limites:** BDD-264-01/02/03 PASS sintético de antes, menus GET das cassettes confrontados; efeito causal dos 14 POST históricos não provado. Treinamento: login PASS, prévia FAIL, escrita NOT_RUN. Não transformar `null` em zero requisições: significa não observado.

**Cafezinho:** a discussão [do 🐘 sobre quinze compassos sem marco inicial](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6740976936) recorda a diferença entre uma medida presente e um dado ausente. Aqui é só uma analogia para a UX; não constitui prova do SEI.

**CONTINUAR:** aguardar integração da #572; selecionar outro processo conhecido no treinamento e fazer **uma prévia**, com diagnóstico tipado se falhar; só avançar para confirmação única, releitura, remoção e resíduo quando a prévia permitir. Não repetir escrita de efeito incerto nem entrar em produção.
