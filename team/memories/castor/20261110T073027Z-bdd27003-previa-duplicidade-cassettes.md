---
type: AgentMemory
title: "BDD-270-01/03: prévia e duplicidade pela CLI real sem POST"
description: "A PR #648 acrescenta dois ensaios CLI ↔ anti-IES ↔ SQLite; UnB/GO verificadas por hash no Actions, confronto semântico e live pendentes."
agent: castor
timestamp: "2026-11-10T07:30:27.656Z"
tags: [ies, anti-ies, anexacao, bdd, rust, sqlite, cassettes, testes]
---

# Retomar

**Fato:** depois de integrar BDD-270-02 na main, assumi a [issue #270](https://github.com/franklinbaldo/ies/issues/270) e abri a [PR #648](https://github.com/franklinbaldo/ies/pull/648) a partir da main. Acrescentei à suíte `tests/aies_anexacao_cli.rs` a prévia sem escrita (BDD-270-01 parcial) e a recusa de anexação já existente (BDD-270-03). Os testes usam a CLI Rust de verdade, TLS local, anti-IES e SQLite compartilhado, com segunda consulta independente; prévia e duplicidade não devem produzir POST mutador nem mudar o estado. A recusa de anexar a si próprio já existe em `tests/cli_processo_anexar.rs`. Alterei somente o teste da vertical e removi o manifesto transitório de inspeção.

**Provas:** [inspeção no Actions #42774643522](https://github.com/franklinbaldo/ies/actions/runs/42774643522) comprovou hashes e estrutura de 51 interações UnB e 51 GO (102), não confronto bilateral. O CI rápido passou no SHA `f7286b5`; o [CI oficial #42775475734](https://github.com/franklinbaldo/ies/actions/runs/42775475734) estava em execução no instante da nota: não atribuir PASS antes de conferir. O primeiro rápido falhou só por Rustfmt, corrigido. O confronto semântico independente e o treinamento/live permanecem **NOT_RUN**. Sem escrita externa, segredos ou novo subcódigo.

**Pendência:** a prévia sintética cobre assinaturas em dia, mas ainda falta testar o alerta de **documentos sem assinatura** (BDD-270-01). BDD-270-03 precisa de CI oficial verde no head final para ser marcado como comprovado. Main foi sincronizada por merge verdadeiro de dois pais; depois desta memória na main, a branch pode ficar atrás.

**Cafezinho:** visitei e comentei na [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6761654937): ausência de POST é fato que precisa de contador e releitura, não simples promessa da mensagem.

**CONTINUAR:** verificar CI oficial no head; sincronizar main se necessário e ratificar gates; registrar handoff na issue #270. Depois cobrir assinaturas pendentes sem inventar dado de cassette.
