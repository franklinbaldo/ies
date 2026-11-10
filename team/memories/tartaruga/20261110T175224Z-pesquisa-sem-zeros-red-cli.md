---
type: AgentMemory
title: "BDD-60-05: RED real da CLI na pesquisa de numero SEI sem zeros"
description: "A pesquisa sintética devolvia 503 sem percorrer a arvore para o numero 11519; a PR #691 introduz fallback de localizacao, preservando a conferencia literal da identidade na CLI."
agent: tartaruga
timestamp: "2026-11-10T17:52:24Z"
tags: [bdd60, anti-ies, cassette-first, cli, red, pesquisa-rapida]
---

# Retomar

A [PR #687](https://github.com/franklinbaldo/ies/pull/687) ja foi integrada. Para a lacuna de CLI real, abri a [PR #691](https://github.com/franklinbaldo/ies/pull/691), usando o BDD-60-05 existente: o [ensaio UnB de 04/11](https://github.com/franklinbaldo/ies/blob/main/docs/validacao-live/2026-11-04-documento-obter.md) documenta que pesquisar `11505` abre `0011505`, mas a CLI deve recusar a identidade divergente.

**RED funcional:** o [CI #42867908910](https://github.com/franklinbaldo/ies/actions/runs/42867908910) executou o binario real via TLS local, anti-IES HTTP e SQLite e falhou em `zeros_opacos::bdd_60_05_cli_recusa_numero_sem_zeros_mesmo_com_ocultacao`: trilha `GET procedimento_controlar`, `POST protocolo_pesquisa_rapida`, **sem** `GET procedimento_visualizar`; 21 testes passaram e 1 falhou. Uma falha Rustfmt separada foi corrigida.

**Correcao proposta:** fallback de localizacao numerica apenas na pesquisa rapida do anti-IES/iesei, com resultado potencialmente ambiguo, sem alterar o numero opaco da arvore. A CLI deve rejeitar antes de obter o corpo, inclusive com `--ocultar-inacessiveis`. O teste usa IDs sinteticos; o card original `documento-numero-e-busca` nao contem a variante de zeros. Live novo/REA NOT_RUN. O [CI GREEN #42869032719](https://github.com/franklinbaldo/ies/actions/runs/42869032719) ainda estava em execucao ao registrar isto, nao inferir sucesso.

**Proxima rodada:** conferir logs, corrigir qualquer divergencia, sincronizar a `main` por merge de dois pais e revalidar o SHA final antes de pedir integracao. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) mantive a separacao entre identidade publicada, inferencia sintética e observacao real; a analogia nao acrescenta prova.
