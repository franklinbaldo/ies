---
type: AgentMemory
title: "Retirada de anotação: RED real e intenção de vazio"
description: "BDD-ANOT-04 na PR #667: RED observado (3 novos E2E falham por --remover ausente), transição CLI/anti-IES/SQLite implementada; inspeção de 2 cassettes PASS de integridade, GREEN do head ainda por validar."
agent: raposa
timestamp: "2026-11-10T13:24:55Z"
tags: [ux, bdd-anot-04, cli, anti-ies, sqlite, cassette-first, red-green, cafezinho]
---

# Retomar

**Fato:** a [PR #662](https://github.com/franklinbaldo/ies/pull/662) foi integrada. A próxima fatia da [#638](https://github.com/franklinbaldo/ies/issues/638) é retirada de anotação. Na [PR #667](https://github.com/franklinbaldo/ies/pull/667) os três novos E2E da CLI real sobre HTTPS local e anti-IES/SQLite produziram **RED comportamental observado** no [CI #42822822360](https://github.com/franklinbaldo/ies/actions/runs/42822822360): 5 testes antigos PASS e os três novos FAIL, porque a CLI sem `--remover` devolvia saída 2 com HTTP vazio. A branch recebeu a implementação de confirmação por POST único, 302 causal e releitura vazia; 403 pós-mutação deverá produzir saída 3, sem retry. **GREEN do novo head ainda não observado**, PR segue Draft.

**Cassette-first:** a [inspeção Actions #42822776545](https://github.com/franklinbaldo/ies/actions/runs/42822776545) validou hashes e estrutura dos dois cards UnB/GO de anotação (37+40 interações; cada qual contém um POST `anotacao_registrar`). **Isso não é confronto semântico bilateral** nem prova da retirada por texto vazio: ambos↔cassette `NOT_RUN`, treinamento/live `NOT_RUN`. Removi o disparador temporário da branch após confirmar o PASS de integridade.

**Distinção:** `--remover` com campo vazio é a intenção de retirar; a ausência observada é a textarea vazia na releitura. Não promover uma à outra por analogia. Registrei essa reflexão no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6768507487).

**CONTINUAR:** sincronizar por merge verdadeiro a branch com a `main` vigente, revalidar CI oficial no SHA final (Draft→Ready), corrigir falhas sem apagar testes, buscar revisão 🦅. BDD-ANOT-05 permanece futuro.
