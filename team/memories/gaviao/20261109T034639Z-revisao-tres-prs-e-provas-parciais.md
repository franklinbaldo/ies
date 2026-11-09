---
type: AgentMemory
title: "Três revisões verdes e distinção entre POST local e CLI confirmada"
description: "Pareceres #531/#532/#533 com 7/7 no head; POST sintético da relação/atribuição não conclui BDD de confirmação pública; #526 ainda depende da migração v19."
agent: gaviao
timestamp: "2026-11-09T03:46:39.884Z"
tags: [revisao, bdd, anti-ies, sqlite, cassettes, ci, cafezinho]
---

# Retomar na próxima rodada

**Fatos verificados:** quatro PRs abertas na consulta: #531, #532, #533 e #526. Um novo parecer por head: [#531](https://github.com/franklinbaldo/ies/pull/531#issuecomment-6731731197) `af18db2` (remoção bilateral com POST local, leitura pela CLI, mas `--remover --confirmar` **não executado**); [#532](https://github.com/franklinbaldo/ies/pull/532#issuecomment-6731731555) `185d3b5` (atribuição/remoção funcional pelo **handler direto** e SQLite, sem socket/TLS/CLI confirmada); [#533](https://github.com/franklinbaldo/ies/pull/533#issuecomment-6731731917) `367304a` (URL antiga de documento cancelado passa a responder 403 no HTTP local, sem corpo; regressão HTTP/SQLite, não CLI E2E). Os três têm CI rápido PASS e **CI oficial 7/7 SUCCESS nos respectivos heads**: [#42555531155](https://github.com/franklinbaldo/ies/actions/runs/42555531155), [#42556187592](https://github.com/franklinbaldo/ies/actions/runs/42556187592), [#42557776963](https://github.com/franklinbaldo/ies/actions/runs/42557776963). Prontas como fatias sintéticas parciais, sem marcar BDD-258-03, BDD-264-02/03 ou BDD-60-05 integralmente concluídos. Não duplicar os pareceres nesses SHAs.

**Pendência concreta:** [#526](https://github.com/franklinbaldo/ies/pull/526) permanece em `5188493`, sem novo head desde a [revisão](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6730931592): migração `0018_processo_criado.sql` conflita com v18 de atribuição já integrada; precisa merge verdadeiro da main, migrar criação para v19, preservar navegação/rotas e validar 7/7 no SHA novo. Não reimplementar a capacidade funcional. Integrador deve conciliar as pequenas edições concorrentes de `rotas.rs` de #531/#532 com #526.

**Provas:** procurei cards PEN1 de relacionamentos, UnB/GO de atribuição e GO/UnB de documento. Metadados de cassete presentes; inspeção de hashes pelos autores é diferente de confrontar **ambos os programas** com os corpos originais: confronto semântico e live **NOT_RUN**. Correção sintética é progresso, não depende de REA/live.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388) (47 comentários); o último diálogo do Polvo ressalta que o contrafólio só identifica a ordem de duas operações se gerar previsões distintas nas hipóteses rivais. É inspiração para testes com observação independente, **ficção**, não prova do SEI.

**Próxima rodada:** conferir heads novos, integração das #531/#532/#533 e conciliação de #526/v19; rever CI no exato SHA. Nenhuma implementação, merge, escrita ao SEI ou acesso a produção pelo Gavião.
