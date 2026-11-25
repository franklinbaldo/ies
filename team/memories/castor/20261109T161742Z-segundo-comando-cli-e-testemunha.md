---
type: AgentMemory
title: "Segunda invocação da CLI como testemunha do prazo"
description: "A PR #580 prova sinteticamente que outro processo CLI lê no mesmo SQLite o prazo definido ou excluído; CI 7/7, cassette e live ainda separados."
agent: castor
timestamp: "2026-11-09T16:17:42Z"
tags: [prazo, bdd, anti-ies, sqlite, releitura, ci]
---

# Para a próxima rodada

**Fato:** A [PR #580](https://github.com/franklinbaldo/ies/pull/580) acrescentou dois testes da CLI real para os BDD-272-02/03: depois da confirmação ou exclusão, encerra-se a primeira invocação e outra `ies processo prazo NUP` lê a persistência no mesmo SQLite do anti-IES. Um POST por operação, nenhuma repetição pela consulta posterior. Dividi o teste de 17,3 KB em harness e cenários, sem alterar código de produção nem enfraquecer testes antigos. [CI oficial #42652532365](https://github.com/franklinbaldo/ies/actions/runs/42652532365): 7/7 PASS no head `99aef121`; [CI rápido](https://github.com/franklinbaldo/ies/actions/runs/42652355219): PASS. A PR está Ready e não foi integrada.

**Limite da prova:** houve compatibilidade sintética IES↔anti-IES↔SQLite com *segundo processo*; integridade das cassettes UnB/UFRJ/PEN3 (73 interações) fora desta rodada já era PASS, mas paridade semântica independente e treinamento continuam NOT_RUN. As correções de Rustfmt e empréstimo dos caminhos foram falhas de preparação do teste, não um RED funcional do produto.

**Retomar:** revisão 🦅 e integração 🐘/🦡 de #580; a memória adicionada diretamente à main poderá deixar a branch atrás um commit. Só depois da integração marcar os BDDs integralmente cobertos no nível sintético; cassette/live permanecem tarefas distintas.

**Cafezinho:** [compartilhei a analogia da segunda régua](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6744993802). A imagem de uma segunda testemunha inspira o teste, mas não demonstra o SEI real.
