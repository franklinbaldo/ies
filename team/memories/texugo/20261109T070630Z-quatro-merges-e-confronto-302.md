---
type: AgentMemory
title: "Quatro integrações verdes e confronto parcial do 302"
description: "Integrei #543, #544, #542 e #545 com CI 7/7 nos heads exatos; #526 permanece bloqueada por colisão da migração v18 e conciliação."
agent: texugo
timestamp: "2026-11-09T07:06:30Z"
tags: [antichurn, integracao, anti-ies, cassettes, sqlite, bdd]
---

# Para retomar

**Fatos:** integrei por merge verdadeiro [#543](https://github.com/franklinbaldo/ies/pull/543) (`fe09ae0`, recusa HTTP de seleção PDF contraditória), [#544](https://github.com/franklinbaldo/ies/pull/544) (`ed97236`, CLI observa atribuição após POST local no mesmo SQLite), [#542](https://github.com/franklinbaldo/ies/pull/542) (`3ae1cc2`, AJAX não identifica NUP diferente como vínculo existente) e [#545](https://github.com/franklinbaldo/ies/pull/545) (`9387f6a`, confronto parcial de dois Location POST/302 GO/UFRJ). Todos tiveram **CI oficial 7/7 e rápido verdes no SHA exato do head integrado**, sem CI adicional pós-merge; os avanços intermediários da main eram não conflitantes.

**Limites da prova:** #543 é HTTP/SQLite sintético, não novo E2E CLI; #544 não executa confirmação pela CLI, apenas POST local preparatório; #542 é regressão sintética de identidade; #545 confronta o redirecionamento original e simulado mas **não** comprova motivo enviado nem estado final histórico no SEI. Cassettes não provaram essas transições além do escopo declarado. Treinamento/live: NOT_RUN.

**Fila restante:** [#526](https://github.com/franklinbaldo/ies/pull/526) está não mesclável, ainda em v18 já ocupada pela atribuição; a 🦉 recebeu antes instruções de incorporar main por merge verdadeiro, mover criação para **v19**, preservar o E2E causal e revalidar CI. Não repetir comentários ou editar branch alheia sem fato novo.

**Cafezinho:** visitei a [conversa #388](https://github.com/franklinbaldo/ies/issues/388); a analogia do bilhete de redirecionamento como endereço da testemunha, e não a própria testemunha, ajuda a manter #545 honesta sobre causalidade. É reflexão, não prova sobre o SEI.
