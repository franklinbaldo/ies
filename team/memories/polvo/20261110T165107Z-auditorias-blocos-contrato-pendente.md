---
type: AgentMemory
title: "Auditorias de blocos encerradas; inclusão espera contrato unificado"
description: "PRs #664/#672 integradas e auditorias #591/#592 encerradas; inventário da cassette UnB subsidia #381/#343 sem assumir POST, BDD ou paridade não comprovados."
agent: polvo
timestamp: "2026-11-10T16:51:07Z"
tags: [blocos, documentos, contrato-cli, cassettes, auditoria, anti-ies]
---

# Ponto de retomada

- **Fatos:** PR [#664](https://github.com/franklinbaldo/ies/pull/664) e [#672](https://github.com/franklinbaldo/ies/pull/672) foram integradas, CI oficial de cada head 7/7; fechei as auditorias [#591](https://github.com/franklinbaldo/ies/issues/591) e [#592](https://github.com/franklinbaldo/ies/issues/592). As correções foram documentais, não novas provas de paridade. Confronto semântico bilateral e live dessas auditorias seguem NOT_RUN.
- **Inventário para o próximo trabalho:** comentei na [#381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6773436997) a cassette UnB de 152 interações (9 POST) e as ações de bloco; a contagem agregada **não identifica** qual POST inclui documento/processo nem prova efeito. O proprietário aprovou unificar #381/#343; contrato e BDD canônicos ainda cabem ao 🐘. Sem BDD aprovado, não implementar sintaxe candidata.
- **Cafezinho:** [#388](https://github.com/franklinbaldo/ies/issues/388) reforçou que condição inicial, estado observado e resultado de escrita são três proposições distintas; a decisão [#681](https://github.com/franklinbaldo/ies/issues/681) muda o vocabulário público da falha não confirmada e exige evitar descrições antigas de saída 3 em novos trabalhos.
