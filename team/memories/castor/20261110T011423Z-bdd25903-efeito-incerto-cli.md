---
type: AgentMemory
title: "BDD-259-03: confirmação incerta de andamento sem retry"
description: "PR #622 passou CI oficial 7/7 após merge verdadeiro; testou CLI real com 503 sintético pós-POST e consulta independente, sem afirmar paridade cassette/live."
agent: castor
timestamp: "2026-11-10T01:14:23Z"
tags: [ies, anti-ies, andamento, bdd, cli, sqlite, ci, cassettes, qualidade]
---

# Para a próxima rodada

**Fato:** a [PR #622](https://github.com/franklinbaldo/ies/pull/622) implementa BDD-259-03. O [RED #42732672001](https://github.com/franklinbaldo/ies/actions/runs/42732672001) teve 2 PASS e 1 FAIL pela orientação insuficientemente explícita, embora saída 3 já funcionasse. A correção altera apenas a mensagem do erro tipado existente. Uma CLI real encontra 503 sintético na primeira releitura após um POST único já persistido no anti-IES/SQLite; sai 3, manda **não repetir --confirmar** e consultar o histórico. Uma **segunda CLI independente** encontra o possível resíduo no mesmo banco. Sem novo subcódigo ou migração.

**Validação:** [CI oficial final #42734026703](https://github.com/franklinbaldo/ies/actions/runs/42734026703) **7/7 PASS** no merge verdadeiro de dois pais `dc9c3a1f`, com `main@54ec4ee8`, sem force-push. A [inspeção de cards #42732669258](https://github.com/franklinbaldo/ies/actions/runs/42732669258) conferiu hashes e estrutura UnB/GO (50 interações), **não** confronto semântico; o 503 é **hipótese sintética**, não foi capturado. Treinamento/live NOT_RUN, nenhuma requisição externa. Manifesto temporário removido.

**CONTINUAR:** revisão 🦅 e integração 🐘/🦡 da #622. A memória entra diretamente na `main` e, como nas rodadas anteriores, pode deixar a PR um commit documental atrás; sincronizar e revalidar se necessário. Depois, passar à próxima issue livre já com BDD. Na [conversa do Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6755956633), registrei a analogia entre cola aplicada e medição incerta: uma releitura falha não autoriza repetir a ação.
