---
type: AgentMemory
title: "BDDs de vínculo e cancelamento reconciliados; RED de identidade AJAX"
description: "Após #538–#541, BDD-258-02/03/04 e BDD-273-01/02 estão PASS sintético; #542 obteve RED real de NUP×ID, #526 segue conflito v18/v19."
agent: elefante
timestamp: "2026-11-09T06:10:21Z"
tags: [integracao, bdd, anti-ies, cassettes, causalidade, ci]
---

# Próxima rodada

**Fatos:** Atualizei checklists das issues [#258](https://github.com/franklinbaldo/ies/issues/258) e [#273](https://github.com/franklinbaldo/ies/issues/273) após merges #538–#541: PASS **sintético** para confirmação/remoção/efeito incerto de vínculos no binário debug TLS local, e prévia/cancelamento/releitura de documento pela CLI com motivo exato. O release conserva escrita fechada e treinamento continua NOT_RUN. Registrei na [#264](https://github.com/franklinbaldo/ies/issues/264) o confronto restrito da [#539](https://github.com/franklinbaldo/ies/pull/539): formulários GET de atribuição, não POST ou causalidade.

**RED concreto:** [#542](https://github.com/franklinbaldo/ies/pull/542), [CI #42570917024](https://github.com/franklinbaldo/ies/actions/runs/42570917024): 2129 testes PASS, 1 FAIL; ID de destino 22 aparece no formulário com outro NUP, mas a consulta AJAX resultou em `SemEfeito`. Orientação publicada; aguardar GREEN tipado, arquivo de inspeção removido e CI 7/7. [#543](https://github.com/franklinbaldo/ies/pull/543) ainda Draft. [#526](https://github.com/franklinbaldo/ies/pull/526) está 211 commits atrás, com conflito SQLite criação v18 versus atribuição v18 integrada; dono deve reconciliar criação para v19 e repetir CI.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388). A comparação do 🐢 entre dois nós com mesmo número SEI e gravações musicais distintas recorda que rótulo não basta para certificar identidade; é analogia, não evidência externa. Na próxima rodada, conferir heads e CI atuais antes de qualquer integração. Radar da fila publicado em [#56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6733531706).
