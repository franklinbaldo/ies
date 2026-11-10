---
type: AgentMemory
title: "Limpeza comprovada continua verdadeira quando o envio nem começa"
description: "BDD-581-01 PR #689: RED E2E comprovou perda de anotacao_depois=false após falha pré-POST; efeito parcial tipado na CLI, cassettes UnB/GO verificadas apenas em integridade."
agent: raposa
timestamp: "2026-11-10T17:28:30Z"
tags: [ux, bdd-581-01, anti-ies, sqlite, cassette-first, red-green, cafezinho]
---

# Para lembrar

O [RED da PR #689](https://github.com/franklinbaldo/ies/actions/runs/42863725613) observou 13 E2E anteriores PASS e o novo BDD-581-01 FAIL: a limpeza separada havia sido feita no anti-IES/SQLite, mas a CLI publicava `anotacao_depois:null` quando o formulário de envio falhava antes de qualquer POST. A correção mantém a causa Rust tipada, informa limpeza confirmada e envio não iniciado, sem criar novo subcódigo nem repetir escrita. O GREEN do head `36aff57` ainda estava em execução ao registrar esta memória.

[Inspeção de cassettes via Actions](https://github.com/franklinbaldo/ies/actions/runs/42863441481): UnB 37 + GO 40 interações, hashes de ZIP e membro verificados; a falha pré-POST é **hipótese sintética**, e confronto semântico bilateral/live continuam NOT_RUN. Arquivo temporário de inspeção removido da PR.

No [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6774341765), comparei a diferença entre duas escritas encadeadas e uma única etiqueta "falhou": a segunda falhar não desfaz a primeira. Retomar o CI do SHA final, revisão independente e integração do BDD-581-01; depois BDD-581-02 na mesma issue.
