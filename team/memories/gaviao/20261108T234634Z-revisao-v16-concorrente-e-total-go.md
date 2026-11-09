---
type: AgentMemory
title: "Quatro revisões e nova concorrência de migração v16"
description: "Heads #517/#518/#491/#519 revisados; CI oficial 7/7 nos quatro, nova colisão v16 entre #491 e #518, total GO corrigido e recusa HTTP duplicada."
agent: gaviao
timestamp: "2026-11-08T23:46:34Z"
tags: [revisao, bdd, anti-ies, sqlite, cassettes, ci, cafezinho]
---

# Para a próxima rodada

**Fatos verificados:** pareceres por head publicados na [#517](https://github.com/franklinbaldo/ies/pull/517#issuecomment-6728850806) (8c44d97, total GO 27.338 em vez de fixture 27), [#518](https://github.com/franklinbaldo/ies/pull/518#issuecomment-6728851155) (9766a66, BDD-245-01 prévia de processo pela CLI com 3 GET/0 POST e SQLite, RED e CI 7/7), [#491](https://github.com/franklinbaldo/ies/pull/491#issuecomment-6728851446) (9dedb29, relacionamento prévio BDD-258-01, CI 7/7) e [#519](https://github.com/franklinbaldo/ies/pull/519#issuecomment-6728851796) (64f5afd, regressão de id_protocolo repetido HTTP/SQLite, CI 7/7 após fim do job prek). Não duplicar pareceres nesses SHAs.

**Nova integração pendente:** #491 reconciliou a antiga colisão v15, mas propõe agora 0016_processo_relacionamentos.sql e #518 também propõe 0016_tipo_processo_simulado.sql. Ordem sugerida: **#491 v16, depois #518 v17** por merge verdadeiro, ajustando include de banco_cassettes, asserts e novo CI. Ordem inversa exige #491 v17. Nenhuma das duas fatias precisa ser reimplementada. As branches diferem da main apenas por commits posteriores de memórias OKF na última consulta, mas a ordem das migrações exige decisão do integrador.

**Evidências:** #517 preserva dados sintéticos e muda só o total declarado com base no relatório de replay GO; não chamar isso de paridade completa. #518 tem inspeção Actions de 2 cards UnB, 18 interações/hash; #491 PEN1 53; integridade não equivale a confronto independente IES e anti-IES versus cassette. #519 é regressão HTTP, não novo BDD da CLI completo. Treinamento/REA não executados, não são gates.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), agora com 44 comentários. A conversa dos colegas sobre contrafólio, gravação multipista e diferença entre observação do vazio e ausência de observação reforça a importância de testemunhas independentes; é uma inspiração do folhetim, não evidência de comportamento do SEI.

**Próximo:** examinar novos heads, confirmar integração ordenada #491/#518, não repetir a cobertura existente do BDD-61-02 ou BDD-60-05.
