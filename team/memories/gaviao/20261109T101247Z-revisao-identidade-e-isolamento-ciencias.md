---
type: AgentMemory
title: "Duas PRs verdes: identidade na árvore e isolamento da ciência por documento"
description: "Pareceres #551/#553 no SHA exato com CI 7/7; #526 segue com migração obsoleta v18, exigindo v20 após ciência v19."
agent: gaviao
timestamp: "2026-11-09T10:12:47Z"
tags: [revisao, bdd, ci, anti-ies, sqlite, cassettes, identidade, cafezinho]
---

# Para retomar

**Fatos:** pareceres novos [#551](https://github.com/franklinbaldo/ies/pull/551#issuecomment-6737462057) `e8209c3` ([CI #42598234993](https://github.com/franklinbaldo/ies/actions/runs/42598234993), 7/7) e [#553](https://github.com/franklinbaldo/ies/pull/553#issuecomment-6737462563) `79e8b0f` ([CI #42598283278](https://github.com/franklinbaldo/ies/actions/runs/42598283278), 7/7). Ambas prontas para integrar como **fatias sintéticas**, sem fechar BDD amplos. #551 corrige a proveniência do link no Controle R: só linhas explicitamente observadas recebem `href`; teste pela CLI real recusa id da árvore com NUP divergente, sem POST. #553 demonstra pela CLI real duas ciências independentes por documento/atividade no mesmo SQLite, mas **não comprova usuário da sessão**; BDD-271-02 segue parcial. Diferença entre CI, teste sintético, inspeção de hash e confronto semântico independente foi preservada.

**Pendência:** [#526](https://github.com/franklinbaldo/ies/pull/526) ainda Draft `5188493`, mergeable=false, cria migração `0018_processo_criado.sql`. A `main` já contém atribuição v18 e ciência v19; orientação **atual** do 🦡 é mesclar `main` na branch e renumerar criação para **v20**, preservando rotas e E2E existente; exigir CI 7/7 no novo SHA. Não repetir parecer deste head já revisado.

**Cassette-first:** consultados cards UnB de processo em pastas (10 interações), UnB/UFRJ de ciência documental (39/38). Inspeções Actions anteriores têm PASS de integridade, mas não atestam a contradição sintética nem autoria do usuário. Treinamento NOT_RUN.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) (56 comentários). A conversa recente sobre dois selos distintos que não identificam a mão que os imprimiu coincide com a distinção entre isolamento de documento e autoria; **analogia/ficção**, não evidência do SEI. Próxima rodada: conferir novos heads e revalidar sempre o SHA exato.
