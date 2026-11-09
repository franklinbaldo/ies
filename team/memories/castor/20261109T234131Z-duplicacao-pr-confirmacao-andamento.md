---
type: AgentMemory
title: "PR duplicada fechada e trava de escrita pendente no andamento"
description: "BDD-259-02 já tinha PR #614; fechei #616, deixei RED medido e revisão sobre release, atomicidade e manifesto; retomar a PR existente."
agent: castor
timestamp: "2026-11-09T23:41:31Z"
tags: [ies, anti-ies, andamento, bdd, qualidade, coordenacao, sqlite, seguranca, pr]
---

# Para a próxima rodada

**Lição operacional:** antes de escolher tarefa ou criar branch, consultar **todas as PRs abertas e branches do próprio agente**, não só a última PR do turno anterior. A [PR #614](https://github.com/franklinbaldo/ies/pull/614) já implementava BDD-259-02; iniciei por engano uma [PR #616](https://github.com/franklinbaldo/ies/pull/616) sobre os mesmos arquivos. Fechei #616 sem merge, mantendo somente #614 como frente canônica. Não retomar #616 nem editar outra branch paralela. O [RED #42720957415](https://github.com/franklinbaldo/ies/actions/runs/42720957415) da PR duplicada comprovou BDD-259-01 PASS e BDD-259-02/03 FAIL antes de liberar confirmação; commits posteriores não têm CI final confirmado e não constituem entrega.

**Revisão da PR canônica:** no [comentário #6754488451](https://github.com/franklinbaldo/ies/pull/614#issuecomment-6754488451) registrei três pontos com verificação de código: (1) falta restringir `--confirmar` em release, pois a função de origem aceita UnB e GO de treinamento sem `cfg!(debug_assertions)`, contrariando a instrução de não liberar escrita de distribuição nesta fatia; (2) `andamentos::acrescentar` lê fora da transação de `guardar`, possível perda sob conexões concorrentes, ainda **hipótese**, não bug reproduzido; (3) retirar manifesto temporário de cassettes antes de integrar. No head `380dedaf` da #614, CI rápido e inspeção PASS; CI oficial 7/7 **não confirmado**.

**Próxima frente:** BDD-259-03 somente depois da integração da #614; hipótese sintética de POST único que persiste no SQLite mas 503 na releitura, CLI saída 3 sem retry. Cassette UnB/GO: 50 interações previamente conferidas quanto à integridade; confronto semântico específico NOT_RUN e live NOT_RUN. Nenhuma escrita externa foi realizada nesta rodada.

**Cafezinho:** a troca sobre carimbo, autor do efeito e fotografia da observação em [#388](https://github.com/franklinbaldo/ies/issues/388) reforçou que estado posterior por si só não prova autoria do POST. Isto é analogia de discussão, não evidência nova do SEI.
