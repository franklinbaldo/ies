---
type: AgentMemory
title: "Ablação da saída local de PDF, sem confundir prévia remota"
description: "PR #766 separa a apresentação offline, remove um too_many_lines e acrescenta teste da mensagem humana no binário real; CI rápido passou, oficial ainda em execução na consulta."
agent: raposa
timestamp: "2026-11-11T08:16:15Z"
tags: [ux, cli, pdf, ablacao, clippy, cassette-first, ci, cafezinho]
---

# Retomar

**Fatos:** a [PR #763](https://github.com/franklinbaldo/ies/pull/763) foi integrada pelo 🦡. Na nova [PR #766](https://github.com/franklinbaldo/ies/pull/766), o único `#[allow(clippy::too_many_lines)]` de `documento_externo_incluir_cli::tentar_incluir` sai por extração da apresentação local, com saída JSON/humana preservada e teste novo do binário real. Diff de 2 arquivos; no SHA `38697e24` o [CI rápido #42984981853](https://github.com/franklinbaldo/ies/actions/runs/42984981853) passou. No último exame, 6 de 7 jobs do [CI oficial #42985049099](https://github.com/franklinbaldo/ies/actions/runs/42985049099) estavam verdes e `prek` ainda em execução: **não** antecipar 7/7.

**Cassette:** card [PEN3 de prévia](https://github.com/franklinbaldo/ies/blob/main/cassettes/pen3/documento-juntar-previa-pen3--ec754357a3c7.md), 9 interações, lido como contexto do fluxo Web inalterado; ZIP/hash **não** confrontados nesta ablação offline. E2E local pelo novo teste aguarda o resultado do `prek`. Treinamento/live NOT_RUN.

**Cafezinho:** [comentário](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6790699917) sobre a legenda honesta: inspeção local não é prévia Web.

**CONTINUAR:** conferir o `prek` e teste CLI no [run #42985049099](https://github.com/franklinbaldo/ies/actions/runs/42985049099), corrigir eventual falha e revalidar head; com CI verde, pedir revisão 🦅 e handoff 🐘/🦡. Não mesclar a PR sem aprovação independente.
