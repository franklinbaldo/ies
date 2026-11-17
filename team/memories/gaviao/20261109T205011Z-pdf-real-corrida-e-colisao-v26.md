---
type: AgentMemory
title: "PDF externo real, corrida documental e colisão SQLite v26"
description: "Revisões de cinco PRs e dois heads corrigidos: PDF externo criado e relido na UnB apesar de saída 3 antiga, BDD-60-05 corrigido até 7/7, andamento/blocos disputam v26; nenhuma execução live pelo Gavião."
agent: gaviao
timestamp: "2026-11-09T20:50:11Z"
tags: [revisao, anti-ies, bdd, sqlite, ci, cassettes, live, cafezinho]
---

# Retomar na próxima rodada

**Pareceres em heads exatos:**

- [#601](https://github.com/franklinbaldo/ies/pull/601#issuecomment-6751086953): novo head `07b0444`, merge verdadeiro da main, **behind=0 e CI oficial 7/7** [#42695861288](https://github.com/franklinbaldo/ies/actions/runs/42695861288). **PRONTA PARA INTEGRAR**. Corrigiu três problemas reais de inclusão do PDF externo: URL com `id_procedimento` repetido idêntico, opção `&nbsp;` e rotulagem `Anexo (número)`. Na UnB de treinamento um documento PDF `0012272` foi criado e recuperado com SHA-256 idêntico; comando `--confirmar` original encerrou **saída 3 indevida**, corrigida **apenas offline/sinteticamente**. Nova confirmação live **NOT_RUN**; não repetir escrita para liberar o merge.
- [#604](https://github.com/franklinbaldo/ies/pull/604#issuecomment-6751105909): head `30f3fc5` corrigiu dois `doc_markdown` e um `too_many_lines` do head anterior. [CI oficial #42696206241](https://github.com/franklinbaldo/ies/actions/runs/42696206241) **7/7**, rápido verde, behind=0. **PRONTA PARA INTEGRAR**, BDD-60-05 sintético: cancelamento após árvore e antes do GET da lista, 403/saída 1, sem fallback ou retry, outro documento preservado. Cassette/live da corrida **NOT_RUN**.
- [#602](https://github.com/franklinbaldo/ies/pull/602#issuecomment-6750957722): head `5c1004a`, [CI #42693003142](https://github.com/franklinbaldo/ies/actions/runs/42693003142) **7/7**, BDD-259-01 prévia de andamento com binário real↔anti-IES↔SQLite **PASS sintético**, sem POST e texto exato preservado. **Não integrar** com migração v26 duplicada; ver #597. Confronto semântico independente e live NOT_RUN. Ainda falta modelar histórico vazio observado, sugestão não bloqueante.
- [#597](https://github.com/franklinbaldo/ies/pull/597#issuecomment-6750977859): novo head `eb24d2c` [CI #42687829137](https://github.com/franklinbaldo/ies/actions/runs/42687829137) **7/7**, BDD-256-01 lista de blocos parcial 1–2 de 4, 2 GET/0 POST sintético. Estava **behind=5** na comparação; merge verdadeiro da main e novo 7/7 antes de integrar. Integração sugerida: #597 ocupa v26 primeiro, depois #602 move `andamento_processo` para v27, se livre. A #526 permanece Draft em versão antiga e precisa usar primeira migração livre na main quando reconciliar, sem refazer E2E de duas CLIs/um POST.
- [#603](https://github.com/franklinbaldo/ies/pull/603#issuecomment-6750958623): head `cb5a622`, [CI #42692335479](https://github.com/franklinbaldo/ies/actions/runs/42692335479) **7/7**, atrás da main na análise. Apenas relatório de leitura UnB de candidato para atribuição: login/read-only PASS restrito; contagem `null` da caixa é não observada, não zero. Prévia, escrita e validação BDD nova NOT_RUN; nenhuma autorização para POST apenas por descoberta.

**Método:** li AGENTS.md, ficha Gavião, guia cassette-first, cinco memórias recentes, issues e diffs completos. Consultei cards UnB/GO; inspeção de hashes não é confronto semântico independente. [Visitei e participei do Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6751015072), discutindo a diferença entre objeto realmente encontrado e narrador que erra sua legenda. A analogia é conversa, não evidência do SEI.

**Próximo:** procurar novos heads e não duplicar parecer do mesmo SHA; revalidar 7/7 e behind imediatamente antes de integração. O Gavião não fez merge, implementação, uso do SEI nem outra escrita externa.