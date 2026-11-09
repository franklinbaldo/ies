---
type: AgentMemory
title: "Prazo: POST aplicado e confirmação são testemunhas diferentes"
description: "Na PR #568 a CLI real confirma prazo por GET novo; quando o POST altera SQLite mas recebe 403, retorna efeito_incerto sem retry."
agent: castor
timestamp: "2026-11-09T12:20:30Z"
tags: [prazo, bdd, anti-ies, sqlite, clippy, efeito-incerto]
---

# Para a próxima rodada

**Fato:** a [PR #568](https://github.com/franklinbaldo/ies/pull/568) implementou a fatia de `--ate DATA --confirmar` apenas em build debug e origem reservada `https://sei.test/sei`. A CLI real fala com o anti-IES por TLS local e persiste estado na mesma SQLite v20. [RED #42613801109](https://github.com/franklinbaldo/ies/actions/runs/42613801109): um teste falhou; [GREEN #42615589440](https://github.com/franklinbaldo/ies/actions/runs/42615589440): 7/7, com teste de 403 após mutação exigindo saída 3 sem retry. O teste antigo de confirmação sem sessão continuou válido: gate antes da sessão, não enfraquecimento.

**Aprendizado:** modificar o banco não é prova de aceite; um 302 de aceite tampouco substitui a nova leitura que confirma a data. Isso se aproxima do cuidado discutido pelo [Gavião no Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6739171814): observar uma porta fechada difere de não ter filmado a porta. A analogia narrativa não é evidência SEI.

**Retomar:** revisão e integração da #568, sincronizando main por merge verdadeiro; depois `--dias --confirmar` e BDD-272-03 (excluir). Cards UnB/GO consultados; integridade prévia PASS, confronto semântico e treinamento NOT_RUN. Release continua bloqueado, sem escrita externa nem novos subcódigos.
