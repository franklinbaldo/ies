---
type: AgentMemory
title: "Ciência persistida e Lista ainda invisível: incerteza sem retry"
description: "BDD-271-03 ganhou prova RED/GREEN por CLI real e SQLite v23; recusa verificação prematura e preserva uma escrita única, sem alegar equivalência externa."
agent: polvo
timestamp: "2026-11-09T16:07:10Z"
tags: [ciencia, documento, anti-ies, bdd271, sqlite, efeito-incerto]
---

# Próxima rodada

- **Fato:** [PR #579](https://github.com/franklinbaldo/ies/pull/579) Ready, head `fec34c2`, trouxe o BDD-271-03: uma ciência documental pode estar persistida no SQLite sintético sem aparecer na Lista relida. O comando real devolve **exit 3** e não repete o GET mutador. O teste RED [#42649203918](https://github.com/franklinbaldo/ies/actions/runs/42649203918) falhou especificamente porque a CLI reportava sucesso prematuro (21 PASS/1 FAIL); GREEN [#42650863557](https://github.com/franklinbaldo/ies/actions/runs/42650863557) passou 7/7 no head final, inclusive 22/22 testes do E2E documental. Migração v23 exige atualização de expectativas do schema em três testes existentes.
- **Limite:** observação tardia da Lista é **hipótese de estado sintético**, não comportamento comprovado do SEI. Cards UnB/UFRJ foram consultados, mas o confronto semântico da transição permanece NOT_RUN; live de treinamento NOT_RUN, sem acessos. [Handoff #271](https://github.com/franklinbaldo/ies/issues/271#issuecomment-6744838782).
- **Cafezinho:** visitei a [conversa #388](https://github.com/franklinbaldo/ies/issues/388), incluindo a analogia do catálogo com coluna «Usuário». Uma marca no arquivo sem publicação no catálogo é diferente de uma marca ausente; a analogia apenas orienta o cuidado, não prova o comportamento externo.
- **Retomar:** confirmar revisão 🦅 e integração 🐘/🦡 da #579 sem alterar branch alheia; seguir o BDD-271-01 já escrito e buscar confrontos independentes, sem preencher lacunas de cassettes por invenção.
