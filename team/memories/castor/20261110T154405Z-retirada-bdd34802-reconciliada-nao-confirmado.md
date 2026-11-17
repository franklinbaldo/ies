---
type: AgentMemory
title: "BDD-348-02: retirada tipada e regra de escrita não confirmada"
description: "PR #678 conciliada com main após #681; RED comportamental, GREEN Rust anterior, retorno complexo refatorado para tipo; CI oficial do SHA final aguardando."
agent: castor
timestamp: "2026-11-10T15:44:05Z"
tags: [ies, anti-ies, acompanhamento, bdd, rust, sqlite, ci, cassettes]
---

# Ponto de retomada

**Fato:** a [PR #678](https://github.com/franklinbaldo/ies/pull/678) implementa a remoção de G por formulário servido, único POST e releitura com H preservado, exercitada pela CLI real e anti-IES/SQLite. [RED #42840796430](https://github.com/franklinbaldo/ies/actions/runs/42840796430) falhou sem POST; no [CI #42844199247](https://github.com/franklinbaldo/ies/actions/runs/42844199247), a suíte Rust passou, mas Clippy acusou tipo de retorno complexo. Substituí a tupla por `LeituraDoGerenciador` sem `#[allow]`. Reconciliei a `main` por merge verdadeiro [`8cb42b4`](https://github.com/franklinbaldo/ies/commit/8cb42b4cededb1cfd2499b3f9c162ff3821d3181), e fiz a refatoração [`62b46b7`](https://github.com/franklinbaldo/ies/commit/62b46b7d7808ee46c8c8f34575b897b42c5b2dd3).

**Mudança normativa importante:** a decisão do proprietário [#681](https://github.com/franklinbaldo/ies/issues/681) aboliu a saída 3 e o código público `efeito_incerto`. Escrita não comprovada agora é saída **1** e código `nao_confirmado`, sem retry. Os testes/CLI da #678 seguem a `main`; o texto do BDD-348-02 na [#348](https://github.com/franklinbaldo/ies/issues/348#issuecomment-6771749126) precisa ser alinhado pelo 🐘, sem alteração unilateral.

**Evidência:** CI rápido [#42846737767](https://github.com/franklinbaldo/ies/actions/runs/42846737767) PASS no head `62b46b7`; CI oficial [#42846965258](https://github.com/franklinbaldo/ies/actions/runs/42846965258) estava em execução ao escrever esta memória, portanto **não** declarar 7/7 ainda. Integridade das 97 interações UnB PASS anterior, confronto bilateral com cassettes e treinamento/live NOT_RUN; o 503 e grupos G/H são hipóteses sintéticas. Nenhuma escrita no SEI externo nem credenciais reais.

**Cafezinho:** visitei o [#388](https://github.com/franklinbaldo/ies/issues/388) e acompanhei o comentário sobre o estado inicial realmente produzido por uma operação anterior. Na retirada de acompanhamento, verificar H preservado e G ausente na segunda CLI é mais forte do que aceitar um 302; não substitui observação externa.

**CONTINUAR:** conferir os 7 gates no SHA atual; após esta memória na `main`, sincronizar a PR por merge verdadeiro e revalidar o novo SHA, sem rebase, force-push nem integrar por conta própria.
