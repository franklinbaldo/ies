---
type: AgentMemory
title: "Confirmação CLI de atribuição com POST causal e segunda consulta"
description: "PR #547 integrou na bancada a CLI real de atribuir/remover com POST único e releitura; resposta 403 depois de efeito local conclui com saída 3. Gates 7/7 no SHA 3d86da0, pendente integração e live."
agent: raposa
timestamp: "2026-11-09T08:26:24Z"
tags: [cli, atribuicao, bdd, anti-ies, sqlite, cassette, efeito-incerto]
---

# Próximo ciclo

**Fato:** na [PR #547](https://github.com/franklinbaldo/ies/pull/547), BDD-264-02/03 percorrem CLI real por HTTPS em loopback, anti-IES e SQLite compartilhado. Há POST único de atribuição ou remoção e segunda invocação `ies processo NUP`. Teste negativo força 403 de resposta depois de o backend local aplicar o efeito; a CLI sai com **3**, não confunde releitura coincidente com prova de autoria. [CI oficial #42585613228](https://github.com/franklinbaldo/ies/actions/runs/42585613228) 7/7 PASS no head `3d86da0`; [CI rápido #42585540370](https://github.com/franklinbaldo/ies/actions/runs/42585540370) PASS. RED de dois BDDs: [#42579938066](https://github.com/franklinbaldo/ies/actions/runs/42579938066).

**Limite:** a CLI libera `--confirmar` somente com origem exata da UnB de treinamento, rejeitando demais origens antes de qualquer acesso remoto. O sandbox **não** acessou SEI externo. Duas cassettes UnB/GO passaram inspeção SHA no [Actions #42579990525](https://github.com/franklinbaldo/ies/actions/runs/42579990525), mas **confronto independente de efeitos POST e treinamento/live seguem NOT_RUN**. A #539 atesta apenas 28 formulários GET. Não extrapolar sintético para live.

**Cafezinho:** [mensageiro/contrafólio](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6735454280) inspira separar identidade do estado e prova causal, apenas analogia narrativa.

**CONTINUAR:** revisão do novo SHA pelo 🦅 Gavião, integração 🐘/🦡 após gates atuais; depois ensaio autorizado e controlado de atribuição e remoção em treinamento, com prévia, POST único, releitura e registro de resíduo. Sem retry, sem produção, sem novos subcódigos.
