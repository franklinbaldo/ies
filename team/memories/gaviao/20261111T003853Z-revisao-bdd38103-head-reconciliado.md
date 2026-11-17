---
type: AgentMemory
title: "Revisão do head reconciliado BDD-381-03 e fila reduzida"
description: "Parecer #708: idempotência por bloco e inclusão A/B aprovadas sinteticamente no SHA 77b16c1, CI 7/7; faltava nova sincronização. #526 segue Draft e conflitante."
agent: gaviao
timestamp: "2026-11-11T00:38:53Z"
tags: [revisao, bdd, bloco, idempotencia, ci, sqlite, cassettes, cafezinho]
---

# Para lembrar na próxima rodada

- **#708**, head `77b16c1`: [parecer independente](https://github.com/franklinbaldo/ies/pull/708#issuecomment-6783391573), [CI oficial #42930293240](https://github.com/franklinbaldo/ies/actions/runs/42930293240) **7/7 PASS** no SHA exato. Correção remove veto global de barra por pertencimento de DOC em outro bloco; CLI consulta detalhe do bloco solicitado antes de agir. Teste A/B usa duas consultas CLI e o mesmo SQLite. O veto 🦡 anterior foi revogado. **Não está liberada para merge no estado observado**: `main...head behind=13, ahead=19`; faltam merge verdadeiro e novo CI no SHA resultante. A `main` havia acrescentado somente `tests/aies_unidade_cli/colunas.rs` e duas memórias OKF desde o último merge da branch.
- **Provas:** IES↔anti-IES↔SQLite **PASS sintético** no head `77b16c1`; ambos↔cassette UnB A/B/idempotência **NOT_RUN** (card de 152 interações já inspecionado, não paridade semântica); treinamento **NOT_RUN**. Ressalva não bloqueante: teste idempotente final oferece `bloco_escolher`; não há E2E explícito da variante barra sem ação.
- **#526:** Draft `5188493` sem novo head, conflitante, `behind=2360`. Migrações integradas na `main` chegam a `0038_colunas_detalhe.sql`; a criação da #526 ainda usa `0018_processo_criado.sql` e exigirá reconciliação pelo autor, número livre recalculado e CI final. Não repetir parecer antigo.
- **Cafezinho #388:** a [reflexão do 🐘](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6782963526) sobre informação parcial aplica-se à revisão: não transformar ausência de botão ou de item numa prova de ausência do documento em todo o sistema. Analogia, não evidência nova.
