---
type: AgentMemory
title: "Barra original UnB confrontada antes e depois da conclusão"
description: "PR #599 compara link de versões na cassette UnB com o estado HTTP/SQLite do anti-IES; CI 7/7, paridade limitada à oferta da ação."
agent: tartaruga
timestamp: "2026-11-09T19:17:24Z"
tags: [bdd60, anti-ies, cassette, navegacao, transporte, sqlite, ci]
---

# Retomar

**Fato:** [PR #599](https://github.com/franklinbaldo/ies/pull/599) Ready no head `2e4b3449d56a57c246d7d3a8dcf3f85f7c3af1af`. O teste offline `cassette_unb_arvore_concluida_retira_link_de_versoes_no_http` lê o ZIP UnB original com SHA-1/SHA-256, cruza a mesma identidade DOC/NUP em histórico, corpo e árvores e comprova que `documento_versao_listar` é oferecido antes de `procedimento_concluir` e não oferecido depois. Projeta a mesma variação de oferta por HTTP anti-IES sobre SQLite efêmero, com GET 200, POST 405 e revisão preservada. O leitor da barra é o do próprio IES.

**Gates:** [CI completo #42681286814](https://github.com/franklinbaldo/ies/actions/runs/42681286814) **7/7 PASS** e [rápido #42681277136](https://github.com/franklinbaldo/ies/actions/runs/42681277136) PASS no head exato. Falhas intermediárias: Rustfmt, `clippy::doc_markdown` e `clippy::assert_is_empty`; corrigidas sem supressão. Sem RED funcional: a comparação nova passou na primeira execução; não houve mudança de código produtivo.

**Três provas:** CLI real ↔ anti-IES ↔ SQLite **PASS sintético nas regressões existentes**; ambos ↔ cassette **PASS parcial novo para oferta da barra**; live/treinamento nesta rodada **NOT_RUN**. O snapshot do processo aberto/fechado em `TESTE` e os campos de revisão não presentes na gravação são hipóteses sintéticas, não dados do SEI. Não reivindicar igualdade do HTML integral, acesso por URL adivinhada nem generalização para outras instalações.

**Coordenação:** [handoff #60](https://github.com/franklinbaldo/ies/issues/60#issuecomment-6748982802). [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) visitado; a discussão de oferta de operação diferente de existência do recurso continua relevante e agora tem comparação direta com a observação histórica. A `main` avançou com memória de outra rotina após a criação da branch.

**Próximo:** revisão 🦅 e integração 🐘/🦡 da #599, incorporando a `main` por merge verdadeiro e revalidando gates se SHA mudar. A #60 permanece aberta para paridade de outras formas e validação externa autorizada. Não acessar produção, credenciais ou reduzir TLS.
