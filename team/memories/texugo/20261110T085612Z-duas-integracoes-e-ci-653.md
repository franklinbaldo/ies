---
type: AgentMemory
title: "Duas integrações com prova tipada, e corrida de head na #653"
description: "Integrei #651 e #652 com CI 7/7 e rápido nos heads; #653 mudou de SHA durante tentativa de merge e aguarda CI no novo head. #654 segue Draft e #526, conflitante."
agent: texugo
timestamp: "2026-11-10T08:56:12Z"
tags: [antichurn, integracao, cassettes, bdd, ci, concorrencia]
---

# Retomar sem refazer

**Fatos:** [#651](https://github.com/franklinbaldo/ies/pull/651) integrou em `414602a2` a consulta tipada de assinaturas na prévia da anexação: positivo observado, zero observado e desconhecido (não afirmar assinado). Head `462d783`: [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42785118847) 7/7 e rápido PASS; 🦅 LGTM. [#652](https://github.com/franklinbaldo/ies/pull/652) integrou em `8047b1b1` a orientação honesta da prévia de retirada de marcador; head `6d2aadb`: [CI oficial](https://github.com/franklinbaldo/ies/actions/runs/42782312167) 7/7 e rápido PASS, 🦅 LGTM. A divergência do head de #652 em relação à main eram apenas duas memórias, depois três arquivos funcionais disjuntos de #651, sem colisão com os três da #652. Não foi exigido ciclo extra de CI por mero avanço independente.

**Corrida real de integração:** [#653](https://github.com/franklinbaldo/ies/pull/653) tinha head `b6b8c07`, CI oficial 7/7 e rápido PASS, mas a tentativa de merge com SHA esperado recebeu 409 porque a 🐢 atualizou a branch para `be4296a`, incorporando a #651. O [CI oficial do novo head](https://github.com/franklinbaldo/ies/actions/runs/42786253907) estava em andamento; rápido PASS. **Não mesclar antes de confirmar sete jobs verdes no SHA atual**, inclusive nova checagem de head/mergeability. A #654 está Draft; #526 continua Draft, antiga e conflitante, sem reiniciar E2E.

**Alcance das provas:** #651 e #652 executaram CLI real ↔ anti-IES ↔ SQLite; cassettes UnB/GO consultadas/conferidas quanto à integridade, mas confronto bilateral semântico específico NOT_RUN, live NOT_RUN. Nenhuma requisição ao SEI, nenhum subcódigo novo, nenhum gate novo e nenhum CI pós-merge nesta rodada.

**Cafezinho:** [a 🐢 notou](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6763091453) que uma fixture pode falhar em barreira anterior à transição sob teste. RED que nem alcança o estado-alvo é diagnóstico da pré-condição, não defeito funcional demonstrado. Retomar primeiro a verificação de #653, não reconstruir o teste.
