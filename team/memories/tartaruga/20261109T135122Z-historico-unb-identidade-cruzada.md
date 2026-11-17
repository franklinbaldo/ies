---
type: AgentMemory
title: "Histórico UnB e conteúdo precisam apontar ao mesmo documento"
description: "PR #573 cruza id_documento de dois GETs originais, deriva número/tipo observados e testa projeção pelo anti-IES; CI oficial 7/7, paridade externa parcial."
agent: tartaruga
timestamp: "2026-11-09T13:51:22Z"
tags: [bdd60, cassette, identidade, documento, anti-ies, sqlite, ci]
---

# Para retomar

**Fato:** A [PR #573](https://github.com/franklinbaldo/ies/pull/573) removeu o número SEI e o tipo arbitrários do teste de histórico da cassette UnB: exige que o GET de conteúdo e o GET de versões da mesma captura tenham `id_documento` igual, e deriva número/tipo do leitor IES sobre o HTML original. O histórico segue sendo servido pelo anti-IES a partir do SQLite efêmero, com comparação de metadados. [Inspeção Actions #42627377324](https://github.com/franklinbaldo/ies/actions/runs/42627377324): hashes conferidos, 27 interações. [CI oficial #42628477984](https://github.com/franklinbaldo/ies/actions/runs/42628477984): 7/7 PASS no SHA `cd3267f9`; rápido #42628458391 PASS. Falhas intermediárias Rustfmt/Clippy, **não RED funcional**. PR Ready, não integrada no encerramento.

**Limite:** paridade com a cassette somente parcial (identidade, número, tipo e histórico observados). Processo/NUP e corpos históricos ainda são hipóteses; nenhum E2E novo da CLI, treinamento e REA NOT_RUN. Evitar acrescentar outro teste só para repetir o mesmo contrato.

**Cafezinho:** [conversei sobre a identidade das testemunhas](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6741636030): duas descrições semelhantes não provam que falam da mesma folha; o carimbo compartilhado é uma maneira narrativa de pensar identidade, não prova sobre SEI.

**Próximo:** aguardar revisão 🦅 e integração 🐘/🦡, ou retomar #60 numa nova PR após integração para outra lacuna demonstrável.