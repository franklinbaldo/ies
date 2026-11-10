---
type: AgentMemory
title: "BDD-581-01: limpeza de anotação comprovada não equivale a tramitação"
description: "PR #689 teve RED comportamental e CI oficial 7/7 no head 4c8fad5; falta reconciliar nova memória da main, revalidar SHA final e receber revisão independente."
agent: raposa
timestamp: "2026-11-10T17:42:39Z"
tags: [ux, bdd-581-01, anti-ies, sqlite, cassette-first, ci, cafezinho]
---

# Retomada

**Fato:** a [PR #689](https://github.com/franklinbaldo/ies/pull/689) (issue [#581](https://github.com/franklinbaldo/ies/issues/581)) preserva `anotacao_depois:false` e informa que não houve tramitação quando a anotação foi limpa e relida, mas a montagem do formulário de envio falhou pré-POST. O [RED oficial #42863725613](https://github.com/franklinbaldo/ies/actions/runs/42863725613) mostrou `null` indevido; o [GREEN oficial #42866137555](https://github.com/franklinbaldo/ies/actions/runs/42866137555) aprovou **7/7** no head `4c8fad54`, após ajuste de Rustfmt. A evidência percorre CLI real, TLS local, anti-IES e SQLite. Precisam de revisão do 🦅 e integração do 🐘/🦡.

**Fronteiras:** [inspeção de 77 interações UnB/GO](https://github.com/franklinbaldo/ies/actions/runs/42863441481) PASS somente para hashes e estrutura; confronto bilateral de corpos/efeitos **NOT_RUN**, treinamento **NOT_RUN**, zero SEI externo. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6774341765) relatei a diferença entre duas operações encadeadas e uma só mensagem de falha.

**Próxima ação:** incorporar `main` na branch da #689 por merge de dois pais, revalidar os gates do novo SHA, publicar handoff para BDD-581-01 e deixar BDD-581-02 para a próxima fatia. Não confundir CI do head antigo com CI do head reconciliado.
