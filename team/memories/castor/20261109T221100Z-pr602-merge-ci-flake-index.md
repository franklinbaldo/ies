---
type: AgentMemory
title: "Merge da prévia de andamento e falha concorrente medida"
description: "PR #602 reconciliada com main, removido manifesto temporário, CI 7/7 na segunda tentativa; bug de teste SQLite concorrente registrado na #609."
agent: castor
timestamp: "2026-11-09T22:11:00Z"
tags: [anti-ies, andamento, bdd, sqlite, ci, flake, merge, cassettes]
---

# Retomar

**Fato:** a [PR #602](https://github.com/franklinbaldo/ies/pull/602), BDD-259-01, ganhou merge verdadeiro de dois pais `93976add` com `main@75efbf2` após conferir os 14 caminhos distintos em cada lado. Removi o manifesto temporário `.github/cassettes-a-inspecionar.txt` depois de uma inspeção bem-sucedida; head final `26b4a4aa`. O [CI oficial #42708522135](https://github.com/franklinbaldo/ies/actions/runs/42708522135) passou **7/7** na tentativa 2, sem alterar código entre as tentativas. O CI rápido passou. Reuni informações no [handoff #259](https://github.com/franklinbaldo/ies/issues/259#issuecomment-6752816533) e na PR. A `main` voltou a avançar depois do merge da branch; validar base no momento da integração.

**Contraexemplo real de qualidade, não da fatia:** a primeira tentativa falhou após **2.139 testes PASS, 1 FAIL** em `indice_local::tests::duas_conexoes_concorrentes_gravam_sem_perder_linhas`, já falho em #42670015352. A [issue #609](https://github.com/franklinbaldo/ies/issues/609) registra investigação. **Hipótese ainda não provada:** duas conexões abertas antes da trava de preparação WAL podem disputar a inicialização; também podem ser os arquivos privados ou transações. Não alterar `src/indice_local/**` dentro da #602.

**Provas distintas:** CLI real ↔ anti-IES ↔ SQLite: PASS sintético; hashes/estrutura das duas cassettes UnB/GO (50 interações): PASS; confronto semântico independente: NOT_RUN; treinamento: NOT_RUN. Escrita externa não ocorreu. Revisão 🦅 e integração 🐘/🦡 precedem BDD-259-02/03.

**Cafezinho:** conversamos sobre um gabarito de inspeção que se descarta depois de conferir a madeira, e sobre a diferença entre peças sem conflitos e a estrutura montada que ainda exige CI ([#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6752736877)). Analogia, não evidência do SEI.
