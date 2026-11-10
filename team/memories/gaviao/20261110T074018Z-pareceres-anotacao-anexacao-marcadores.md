---
type: AgentMemory
title: "Anotação, anexação e retirada: três heads, dois CI falhos"
description: "Pareceres #646/#648/#649: Clippy na anotação, anexação 7/7 porém main avançou, retirada com dois Clippy e regressão em teste legado."
agent: gaviao
timestamp: "2026-11-10T07:40:18Z"
tags: [revisao, bdd, anti-ies, ci, clippy, anexacao, marcadores, anotacao, cafezinho]
---

# Ponto de retomada

**Fatos:** #646 head `12d238f`, [parecer](https://github.com/franklinbaldo/ies/pull/646#issuecomment-6761822740), [CI oficial 6/7](https://github.com/franklinbaldo/ies/actions/runs/42776327646): `src/processo_anotar_cli.rs:87` falha Clippy `needless_pass_by_value`; corrigir para referência e revalidar. O E2E de anotação vazia/zero POST é sintético e os hashes UnB/GO não são replay bilateral.

**Fato:** #648 head `6d2cbd7`, [parecer](https://github.com/franklinbaldo/ies/pull/648#issuecomment-6761823328), [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42775858240). BDD-270-01/03 têm E2E sintéticos, mas a variante com documento não assinado ainda não foi ensaiada; comparação no momento mostrava `main` 7 commits à frente. Confronto semântico/live NOT_RUN.

**Fato:** #649 head `8d05cf4`, [parecer](https://github.com/franklinbaldo/ies/pull/649#issuecomment-6761824013), [CI 6/7](https://github.com/franklinbaldo/ies/actions/runs/42776009146): `too_many_lines` em `src/processo_marcar/escrita.rs:129` (183/100) e `tests/aies_marcadores_cli.rs:224` (105/100); teste antigo `tests/cli_processo_marcar.rs:138` falhou pois retirar confirmado agora busca sessão em vez de reportar escrita fechada. E2E novo passou isoladamente, não salva o gate. Corrigir a distinção entre atribuir e retirar, sem eliminar o teste.

**Cafezinho:** li o [#388](https://github.com/franklinbaldo/ies/issues/388), especialmente a [descoberta da 🐙 na #646](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761649928): dois links iguais podem passar na percepção humana e falhar no contrato observável. É um lembrete útil de continuidade do comportamento, não validação live.

Retomar somente novos heads; a memória em `main` pode mudar o behind. #526 permanece Draft antigo sem head revisável nesta rodada. Nenhuma implementação, merge ou requisição ao SEI realizada pelo Gavião.
