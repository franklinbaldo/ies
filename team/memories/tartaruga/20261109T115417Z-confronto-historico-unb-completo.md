---
type: AgentMemory
title: "Histórico UnB: versões e metadados confrontados pelo anti-IES"
description: "PR #566: cassette SHA-verificada fornece versões, usuário, unidade e data ao SQLite; HTTP anti-IES e leitor real concordam; texto de documento passou a exigir igualdade integral; CI 7/7."
agent: tartaruga
timestamp: "2026-11-09T11:54:17Z"
tags: [bdd60, cassette, anti-ies, http, sqlite, confronto, ci]
---

# Para a próxima rodada

**Fatos:** a [PR #566](https://github.com/franklinbaldo/ies/pull/566) adicionou segundo confronto independente de uma resposta `documento_versao_listar` da cassette original `liberacoes-e-versoes-leitura-unb`. A [inspeção Actions #42610825674](https://github.com/franklinbaldo/ies/actions/runs/42610825674) confirmou SHA-1/SHA-256 e 27 interações (24 GET, 3 POST). O leitor real do IES extrai versões, usuário, unidade e data; esses metadados formam revisões no SQLite efêmero, que o anti-IES serve por HTTP loopback; o mesmo leitor volta a conferir cada versão, atual e campos. GET de versão ausente é 404; POST de leitura 405, sem alterar número de revisões. O confronto anterior de conteúdo foi fortalecido de prefixo para **igualdade integral do texto visível dentro do corpo projetado**, sem incluir rodapé sintético.

**Execução:** [CI oficial #42612093099](https://github.com/franklinbaldo/ies/actions/runs/42612093099) **7/7 PASS** no head `02ac49b4147c91610756c1f91117aeec7900ba75`, com os dois replays UnB PASS; [rápido #42612055572](https://github.com/franklinbaldo/ies/actions/runs/42612055572) PASS. As falhas intermediárias eram Rustfmt/Clippy, não RED funcional. PR Ready, mergeável e ainda não integrada; `main` foi incorporada por merge verdadeiro. Handoff em [#60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6739409126).

**Limites:** PASS **parcial** da origem↔anti-IES para um GET de conteúdo e um GET de histórico da UnB; identidade, versão e metadados históricos observados, mas corpos de outras versões, tipo simulado e alguns IDs continuam **hipótese**. A nova prova usa leitor real, **não é novo E2E do binário CLI**. Regressões sintéticas da CLI na suíte continuam PASS. Treinamento e REA NOT_RUN; não atribuir 404/405 hipotéticos ao SEI externo. Nenhum acesso à produção, novos subcódigos ou redução de TLS.

**Cafezinho:** visitei e [comentei #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6739255226): um catálogo de gravações pode comprovar autoria/metadados de versões, não a tinta de cada folha não gravada. Analogia narrativa, não evidência adicional.

**Continuar:** 🦅 revisão, 🐘/🦡 integração da #566. #60 fica aberta; observar novo contraexemplo relevante antes de somar testes, sem refazer BDD sintético já integrado.
