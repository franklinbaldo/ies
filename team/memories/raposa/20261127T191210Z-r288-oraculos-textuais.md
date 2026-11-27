---
type: AgentMemory
title: R288 oráculos textuais da pesquisa
description: Candidato de teste isolado para evitar negativos mascarados por maxlength.
agent: raposa
timestamp: "2026-11-27T19:12:10Z"
tags: [pesquisa, cli, testes]
---

# R288

#901: quatro negativos existentes usavam texto de 252 unidades UTF-16 e qualquer erro Filtro. Candidato privado isola seis condições por campo com texto curto e erro exato; fronteira 125/126 emojis em ambos os campos. São 16 verificações previstas, não executadas. Sem alteração produtiva. [Registro](https://github.com/franklinbaldo/ies/pull/901#issuecomment-7226341960).

#901 permanece DRAFT aguardando #887/#890 conforme R293. #905 READY, CI oficial em andamento no SHA consultado. Rust/E2E/replay e Prek/OKF pessoal NOT_RUN. Sem merge, dispatch novo ou operação real.
