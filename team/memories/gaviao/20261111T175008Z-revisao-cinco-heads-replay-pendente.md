---
type: AgentMemory
title: "Cinco heads novos: quatro CIs com replay e anexação ainda pendente"
description: "Revi #777, #778, #781, #792 e #790 em SHAs atualizados; quatro CIs 7/7, #777 6/7 com replay de anexação em curso. Todas as branches ficaram atrás da main."
agent: gaviao
timestamp: "2026-11-11T17:50:08Z"
tags: [revisao, cassette-first, antipatterns, ci, anti-ies, sincronizacao, cafezinho]
---

# Ponto de retomada

- [#777](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6803140855) head `d6692027`: [CI #43070401336](https://github.com/franklinbaldo/ies/actions/runs/43070401336) tinha 6/7 PASS; `replay unidade.retornos` ainda em execução. Conferir seu log nominal de `processo_anexar::acervo` **neste SHA**. GET original UnB/GO e anti-IES têm oráculo extraído de atributos HTML sem `ler_tela`; HTTP 503 pós-POST continua hipótese sintética sem captura externa. Branch estava behind=1.
- [#778](https://github.com/franklinbaldo/ies/pull/778#issuecomment-6803122499) head `dec3357`: [CI #43061116169](https://github.com/franklinbaldo/ies/actions/runs/43061116169) 7/7; três causas Rust preservadas por `#[source]`. Replay geral 34 acervo/18 cassette; behind=8.
- [#781](https://github.com/franklinbaldo/ies/pull/781#issuecomment-6803123178) head `ec59698`: [CI #43065590260](https://github.com/franklinbaldo/ies/actions/runs/43065590260) 7/7, replay de troca de unidade executado com 32 cassettes, 34 acervo/18 cassette; bilateral de transição continua NOT_RUN; behind=3.
- [#792](https://github.com/franklinbaldo/ies/pull/792#issuecomment-6803123953) head `97ddadd`: [CI #43063138940](https://github.com/franklinbaldo/ies/actions/runs/43063138940) 7/7, replay geral 34+18. Base64 corrompido é contraprova sintética do harness; behind=4.
- [#790](https://github.com/franklinbaldo/ies/pull/790#issuecomment-6803140027) head `55397bc`: [CI #43063066460](https://github.com/franklinbaldo/ies/actions/runs/43063066460) 7/7, rejeição de formulário login sintético substitui teste vazio; não alegar cassette de login; behind=4.
- A [#795](https://github.com/franklinbaldo/ies/pull/795) foi integrada em 11/11/2026 16:51Z; não necessita novo parecer. Em todas as PRs Ready abertas, 🐘/🦡 devem reconciliar e revalidar replay no SHA final.
- Consultei AGENTS.md e ficha integralmente, cinco memórias recentes, contrato e cards de anexação UnB/GO (hashes conferidos nos cards) e visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), com reflexão sobre teste que imprime em vez de falhar. Não houve merge, código funcional, live, produção nem retry pelo Gavião.
