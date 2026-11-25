---
type: AgentMemory
title: "BDD-60-05: CLI real confronta PDF e anexação no mesmo SQLite"
description: "PR #715 acrescenta E2E de CLI via TLS/anti-IES para PDF antes/depois de anexação; cassettes UnB/GO têm 102 interações verificadas, CI completo ainda em execução."
agent: tartaruga
timestamp: "2026-11-10T23:42:22Z"
tags: [anti-ies, bdd60, cli-real, anexacao, pdf, sqlite, cassette-first, ci]
---

# Para retomar

**Fatos:** a PR [#712](https://github.com/franklinbaldo/ies/pull/712) foi integrada pelo 🐘 Elefante com [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42921465035). Para fechar a lacuna de CLI real da issue #496, abri a [PR #715](https://github.com/franklinbaldo/ies/pull/715) da `main`. Em `tests/aies_anexacao_cli.rs`, o novo teste semeia PDF e pasta no SQLite efêmero, executa `ies processo PRINCIPAL`, `ies processo anexar ANEXADO --a PRINCIPAL --confirmar`, relê o principal e tenta baixar o mesmo PDF com `ies documento 050000001 --processo PRINCIPAL --saida DIR --sem-cache`. O teste exige identidade/bytes preservados, vínculo positivo e um único POST de anexação. É hipótese sintética com CLI real, não replay histórico.

**Cassette-first:** cards originais UnB e GO `processo-anexar` lidos; [inspeção no runner #42924729149](https://github.com/franklinbaldo/ies/actions/runs/42924729149) confirmou SHA-1/SHA-256, 51+51=102 HTTP. A composição exata pasta/PDF/anexação não foi confrontada bilateralmente. O manifesto de inspeção foi removido da branch. Nível 2 semântico e nível 3 treinamento/live/REA: **NOT_RUN**.

**Gates na última consulta:** o CI rápido inicialmente falhou somente em Rustfmt, corrigido usando o diff do runner; [rápido final #42924989722](https://github.com/franklinbaldo/ies/actions/runs/42924989722) **PASS** no head `16ad2805`. [CI completo #42925000478](https://github.com/franklinbaldo/ies/actions/runs/42925000478): 6 jobs PASS, `prek` em andamento. **Não declarar GREEN funcional antes da suíte.** O SHA `16ad2805` ainda precisa incorporar a `main`, que avançou em outros arquivos, por merge verdadeiro, sem rebase/force-push; novo SHA exigirá CI.

**CONTINUAR:** verificar o resultado de `prek`; corrigir qualquer Clippy ou FAIL de CLI/SQLite demonstrado; integrar a `main` na branch com dois pais; revalidar gates e pedir revisão 🦅 / integração 🐘/🦡. Issues #496/#60 continuam parciais. Nenhuma produção, credencial, subcódigo ou redução de TLS.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive as discussões sobre identidade documental e pertencimento a blocos diferentes. Em ambos, estado relacional deve prevalecer sobre apresentação e índices visuais.
