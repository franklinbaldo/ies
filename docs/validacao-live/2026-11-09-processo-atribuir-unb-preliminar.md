---
type: SEIObservation
title: "UnB de treinamento: login e prévia inconclusiva de atribuição"
description: "Ensaio controlado de 09/11/2026 com login PASS e prévia não aprovada, sem POST de atribuição nem resíduo de escrita"
---

# Processo atribuir — UnB de treinamento, 09/11/2026

**Issue:** [#264](https://github.com/franklinbaldo/ies/issues/264).
**Execução:** [GitHub Actions #42625001594](https://github.com/franklinbaldo/ies/actions/runs/42625001594),
head `7ca267f`, Rust 1.99.1 em build release `--locked`.
Foi usada exclusivamente a origem allowlisted
`https://sei.treinamento.unb.br/sei` (UnB de **treinamento**).
Conta pública da [tabela autorizada](../interfaces/credenciais-de-treinamento.md);
arquivos locais `0600`, sem publicar credenciais nem dados de sessão.

## Sequência efetivamente realizada

1. Compilação da CLI real: **PASS**.
2. `ies --json login` uma vez: **PASS**,
   `login_realizado`, **6 requisições**.
3. Intervalo mínimo de **30 segundos** antes da prévia.
4. `ies --json processo atribuir 23106.001748/2026-94 --usuario aluno2`,
   dentro do executor `scripts/ensaio_processo_atribuir.py`:
   **PREVIA_FAIL**, código **1**, `requisicoes: [null]`.
   O processo foi escolhido entre os já consultados na UnB em
   [04/11](2026-11-04-varredura-unb-go.md); a atribuição atual **não foi
   confirmada** nesta execução. O executor antigo não preservou a causa
   tipada ou o código interno da prévia no resumo; portanto **não é
   possível diagnosticar a causa apenas com o log saneado**.

O executor interrompeu a sequência **antes de criar a trava de escrita**:
`checkpoint: false`. **Nenhum `--confirmar`, nenhum POST de
atribuição e nenhuma tentativa de remoção** foram realizados.
O POST de autenticação do `ies login` é distinto de um POST de atribuição.
Não houve retry de requisição falha, e nenhuma atribuição/resíduo da
operação foi constatado. A ausência de checkpoint não prova ausência de
efeito dos GETs, que podem registrar recebimento no SEI.

## Estado das três provas

- **IES ↔ anti-IES ↔ SQLite:** PASS sintético anterior nos BDD-264-01/02/03;
  não reexecutado neste ensaio externo.
- **IES e anti-IES ↔ cassettes:** 28 formulários GET confrontados e
  14 POSTs históricos caracterizados nos ensaios anteriores. O efeito
  causal dos POSTs históricos **não foi comprovado**.
- **SEI UnB de treinamento:** login PASS, prévia **FAIL** no roteiro;
  atribuição, remoção e releituras após escrita **NOT_RUN**,
  zero POST de atribuição.

## Retomada segura

A [PR #572](https://github.com/franklinbaldo/ies/pull/572)
corrige o relatório de `PREVIA_FAIL` para conservar apenas saída,
`subcodigo` tipado existente e status HTTP válido, sem texto livre,
cookie, URL assinada ou segredo. **Não repetir a requisição falha
deste processo** para tentar produzir diagnóstico retrospectivo.
Em outro processo de treinamento previamente identificado, executar
prévia única, avaliar resultado antes de qualquer confirmação e
registrar o resíduo; jamais usar produção. O workflow temporário foi
excluído da branch de ensaio no commit
[`ca352216`](https://github.com/franklinbaldo/ies/commit/ca352216ae49e4a325b82138e363bc9bc5c05450)
e não faz parte desta PR.
