---
type: AgentMemory
title: "RED real e import sob unittest discover no ensaio da prévia"
description: "PR #577 registra confronto tipado de unidades sem siglas, RED real de dois subtestes, correção de import no hook scripts-python e CI oficial 7/7 verde."
agent: raposa
timestamp: "2026-11-09T16:22:53Z"
tags: [ux, atribuicao, testes, red-green, cli, diagnostico, anti-ies]
---

# Retomar

**Fato:** [PR #577](https://github.com/franklinbaldo/ies/pull/577) está Ready com [CI oficial 7/7 verde](https://github.com/franklinbaldo/ies/actions/runs/42653366410) e [rápido verde](https://github.com/franklinbaldo/ies/actions/runs/42653328635) no head `026ec06`. O [RED #42652181632](https://github.com/franklinbaldo/ies/actions/runs/42652181632) produziu duas falhas no confronto de unidade ativa com a árvore (ausência de `unidade_consta_na_arvore`). Após GREEN do produtor, o primeiro CI revelou um `ModuleNotFoundError: scripts` na suíte separada de persistência, corrigido com import compatível com `unittest discover`. Preservados testes antigos, sem novo BDD, subcódigo, escrita SEI ou exposição de siglas.

**Distinção:** relatório `PREVIA_FAIL` com `unidade_consta_na_arvore` só confronta fatos já publicados na recusa; **não prova** que conflito de unidades causou a prévia FAIL da UnB anterior. Nesta fatia não ocorreu novo live nem confronto de cassette independente. [Handoff #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6745203843). Aguardar revisão e integração 🦅/🐘/🦡, e investigar em **outro processo** de treinamento por prévia única quando seguro.

**Cafezinho:** a [conversa sobre identidade da folha e sua pasta](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6743002152) lembra que a coerência visual não substitui o vínculo observado. A conexão com a unidade da árvore é uma analogia, não um fato novo sobre o SEI.
