---
type: AgentMemory
title: "Fallback de anotação exige formulário HTTP íntegro"
description: "PR #514 fecha BDD-333-03 em CLI real e SQLite; RED identificou action sem infra_hash no anti-IES, GREEN 7/7 após corrigir o simulador."
agent: coruja
timestamp: "2026-11-08T22:37:30Z"
tags: [processos, anti-ies, anotacoes, bdd, sqlite, reconciliacao]
---

# Retomada

**Fato:** a [PR #514](https://github.com/franklinbaldo/ies/pull/514) exercita, em TLS loopback, a prévia e confirmação de `ies processo enviar --remover-anotacoes` quando a tela de envio não oferece checkbox, mas o formulário próprio permite limpar `txaDescricao`. Duas invocações da CLI compartilham o mesmo SQLite; o teste exige prévia sem POST mutador, um POST de anotação, GET que comprova vazio antes de um POST de envio, GET da árvore e estado final coerente. A hipótese sem checkbox é injetada somente na resposta de teste, não suposta observação do SEI.

**Contraexemplo real:** o [RED no CI #42530684112](https://github.com/franklinbaldo/ies/actions/runs/42530684112) encontrou `acao_do_formulario_sem_infra_hash_unico` na `action` simulada de `anotacao_registrar`, interrompendo a escrita. A correção do anti-IES publica um hash **sintético** de formato válido no mesmo formulário, reutilizando a derivação segura da CLI. [CI final #42530934163](https://github.com/franklinbaldo/ies/actions/runs/42530934163) **7/7**; [CI rápido #42530924326](https://github.com/franklinbaldo/ies/actions/runs/42530924326) verde no head `5a126bdd`.

**Cassettes:** [inspeção #42530191147](https://github.com/franklinbaldo/ies/actions/runs/42530191147) verificou hashes e integridade de **46 interações** dos cards `processo-enviar-retorno` UnB (22) e PEN3 (24). As respostas não validam a alternativa de anotação sem checkbox. Confronto semântico dessa transição e SEI de treinamento **não executados**; REA não executado. Não liberar remoção pública nem alegar paridade externa.

**Retomar:** 🦅 revisar PR #514 e 🐘/🦡 integrar após CI; não repetir teste já verde. Sem migração nem subcódigo novo; sem acesso ao SEI real. A [discussão do Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) sobre sequência de impressão é metáfora: um estado final não substitui as leituras intermediárias, não é evidência SEI.
