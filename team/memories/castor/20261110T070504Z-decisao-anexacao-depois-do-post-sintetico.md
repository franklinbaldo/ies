---
type: AgentMemory
title: "BDD-270-02: documentação não pode ficar na versão de prévia"
description: "Corrigi decisão, contrato e evidência para separar confirmação sintética de cassette/live; PR #636 reconciliada, CI final ainda em andamento no registro."
agent: castor
timestamp: "2026-11-10T07:05:04Z"
tags: [ies, anti-ies, anexacao, bdd, documentacao, ci, cassettes, qualidade]
---

# Para retomar

**Fato:** a revisão 🦅 da [PR #636](https://github.com/franklinbaldo/ies/pull/636#issuecomment-6760910740) revelou que `docs/decisoes/processo-anexar.md` descrevia apenas prévia/recusa, embora o contrato e a CLI real já incluíssem POST único e releitura. Corrigi a decisão, marquei a recusa antiga como **histórica** no contrato e atualizei `contracts/evidencias/processo-anexar.md`. Não alterei a implementação, os testes nem a migração de anexação v32.

**Prova:** CI rápido [#42772504310](https://github.com/franklinbaldo/ies/actions/runs/42772504310) PASS; CI oficial [#42772523625](https://github.com/franklinbaldo/ies/actions/runs/42772523625) ainda estava em execução ao registrar esta nota, sobre `ef177b13`. O E2E da CLI ↔ anti-IES ↔ SQLite já tinha PASS **sintético** anterior; a inspeção SHA/estrutura de 102 interações UnB/GO não é confronto semântico, e este confronto e live seguem NOT_RUN. Nova `main` móvel exige comparação de `behind`, merge verdadeiro e CI oficial no head definitivo antes da integração 🐘/🦡.

**Cafezinho:** a [conversa da 🐙 sobre `btnPesquisar`](https://github.com/franklinbaldo/ies/issues/388) reforçou o mesmo hábito de oficina: uma etiqueta antiga não pode afirmar mais nem menos que a peça testada. Ainda é hipótese, não medição do SEI, que a simulação da confirmação reproduza fielmente o protocolo observado.
