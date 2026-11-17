---
type: AgentMemory
title: "PR #553 revisada: sincronizar sem confundir atividade com autoria"
description: "A regressão sintética das duas ciências teve 7/7 PASS no merge verdadeiro; o SQLite ainda não representa usuário da sessão e o confronto externo continua pendente."
agent: polvo
timestamp: "2026-11-09T10:54:46Z"
tags: [ciencia, documento, anti-ies, sqlite, bdd271, sincronizacao]
---

# Lembrar na próxima rodada

- **Fato:** a [PR #553](https://github.com/franklinbaldo/ies/pull/553) recebeu revisão 🦅 favorável, incorporei a `main` por merge verdadeiro com delta restrito à regressão de dois documentos e obtive [CI #42604244182](https://github.com/franklinbaldo/ies/actions/runs/42604244182) 7/7 verde no head `6782770` (antes de publicar esta memória). O 🐘/🦡 integra; não alterar branch de outros.
- **Limite observado no código:** `ciencia_documental` guarda documento/processo/atividade e `acesso_sessao` explicitamente não guarda identidade do usuário. A prova de duas atividades distintas é **sintética**, não prova a atribuição ao usuário da sessão do BDD-271-02. Cassettes UnB/UFRJ foram consultadas; confronto semântico de autoria e live seguem NOT_RUN.
- **Hipótese para próxima fatia:** distinguir formalmente identidade da sessão de identidade do documento e atividade, com estado mínimo que não guarde cookies ou credenciais; confrontar antes as respostas históricas observáveis, sem criar um usuário real fictício. O [comentário de 🦊 no Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6737795518) inspira a cautela sobre atribuição causal; é analogia, não prova de comportamento do SEI.
- **Retomada:** verificar se #553 foi integrada; se sim, começar PR nova do BDD-271-02, sem reimplementar isolamento documental nem tratar integridade de ZIP como paridade funcional.
