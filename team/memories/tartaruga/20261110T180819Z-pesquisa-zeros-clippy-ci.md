---
type: AgentMemory
title: "Pesquisa por numero SEI sem zeros: RED da CLI e corrigir Clippy antes da integracao"
description: "A PR #691 ja tem GREEN da suite Rust, mas o CI completo falhou em doc_markdown; corrigi o teste e sincronizei a main, aguardando revalidacao de 7 gates no head novo."
agent: tartaruga
timestamp: "2026-11-10T18:08:19Z"
tags: [bdd60, anti-ies, cli, red, clippy, cassette-first, ci]
---

# Retomar

**Fatos:** [PR #691](https://github.com/franklinbaldo/ies/pull/691), BDD-60-05. O [RED #42867908910](https://github.com/franklinbaldo/ies/actions/runs/42867908910) demonstrou CLI real via TLS/HTTP/SQLite: pesquisa por `11519` retornava 503 sem consultar a arvore de `0011519`. A correcao faz somente a pesquisa rapida sintetica localizar numeros sem os zeros, mantendo a identidade literal para a CLI recusar antes de baixar o corpo. O [CI #42870156839](https://github.com/franklinbaldo/ies/actions/runs/42870156839) passou Suite Rust e seis jobs, mas falhou em Clippy `doc_markdown` por `UnB` sem crases no comentario do teste. Corrigi sem supressao no commit `fcdb2ffa`, depois de [merge verdadeiro `1d316fac`](https://github.com/franklinbaldo/ies/commit/1d316fac790b66325bd224a74bf267530f948bdd) da main (19 commits, sem conflito de arquivos). [CI completo #42871729894](https://github.com/franklinbaldo/ies/actions/runs/42871729894) ainda estava em execucao quando registrei, rapido #42871601265 PASS; nao declarar 7/7 ainda.

**Fontes:** [ensaio UnB de 04/11](https://github.com/franklinbaldo/ies/blob/main/docs/validacao-live/2026-11-04-documento-obter.md) observou `11505` abrir `0011505` e a CLI recusar; card UnB `documento-numero-e-busca` tem 15 interacoes mas nao documenta essa variante. E2E novo eh **sintetico**; confronto do negativo com cassette **NOT_RUN**, live novo/REA **NOT_RUN**. Nenhum acesso real, credencial, novo subcodigo ou alteracao TLS.

**Cafezinho:** li #388 e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6775227625) sobre busca flexivel versus identidade publicada. Proxima rodada: conferir suite completa no SHA atual, incorporar a main novamente se estiver atras (inclusive esta memoria), revalidar e enviar a revisao 🦅 / integracao 🐘/🦡. Nao declarar sucesso sem runners.
