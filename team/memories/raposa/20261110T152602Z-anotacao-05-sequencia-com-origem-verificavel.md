---
type: AgentMemory
title: "BDD-ANOT-05: a anotação inicial também precisa de origem comprovada"
description: "PR #680 acrescenta E2E de quatro comandos da CLI sobre um SQLite: registro, consulta, envio com retirada e nova consulta; hashes de 2 cassettes UnB/GO PASS, paridade semântica/live NOT_RUN."
agent: raposa
timestamp: "2026-11-10T15:26:02Z"
tags: [ux, bdd-anot-05, anti-ies, sqlite, cassette-first, e2e, cafezinho]
---

# Para lembrar

A [PR #680](https://github.com/franklinbaldo/ies/pull/680) testa o BDD-ANOT-05 da [#638](https://github.com/franklinbaldo/ies/issues/638) com quatro comandos **reais** no mesmo SQLite descartável: registrar texto → consultar presença → enviar retirando → consultar ausência. O ensaio antigo já comprovava envio com anotação **pré-carregada**; esta fatia comprova que a anotação prévia é resultado de uma operação anterior da CLI. Uma hipótese de estado inicial não é prova de sua criação.

**Fatos nesta gravação:** [inspeção Actions #42843191269](https://github.com/franklinbaldo/ies/actions/runs/42843191269) validou hashes UnB/GO (37 + 40 interações; 1 POST `anotacao_registrar` e 1 POST `procedimento_enviar` por instalação). Isso não é confronto bilateral: cassette semântica **NOT_RUN** e live **NOT_RUN**. O [CI rápido #42843663721](https://github.com/franklinbaldo/ies/actions/runs/42843663721) passou após correção de Rustfmt; o [CI oficial #42843693435](https://github.com/franklinbaldo/ies/actions/runs/42843693435) estava em execução. A primeira falha foi **formatação**, não RED comportamental; não reivindicar RED inexistente.

Li o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6771324867) e registrei a diferença entre fonte observada e estado presumido. **Retomar:** verificar CI no SHA final, corrigir eventuais falhas, reconciliar `main` por merge verdadeiro e buscar revisão 🦅 antes de integração 🐘/🦡. Sem SEI externo, segredos ou subcódigos.
