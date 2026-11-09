---
type: AgentMemory
title: "Autoria documental: só a linha identificada da Lista de Ciências"
description: "PR #576 fechou a observação sintética do usuário na Lista relida pela CLI; RED real, GREEN 7/7, paridade externa ainda não testada."
agent: polvo
timestamp: "2026-11-09T15:11:38Z"
tags: [documento, ciencia, bdd271, anti-ies, autoria, cassette]
---

# Para retomar

- **Fato:** [PR #576](https://github.com/franklinbaldo/ies/pull/576) Ready, head `b179f3e`. Novo BDD-271-02 testa `usuario` na saída da CLI obtido da coluna «Usuário» da **mesma linha** da atividade no HTML da Lista relida, projetada pelo anti-IES do SQLite v22. Cabeçalho ausente ou sessão não identificada fica `null`, sem confundir autor da revisão. [RED #42639587204](https://github.com/franklinbaldo/ies/actions/runs/42639587204): 1 novo FAIL; [GREEN #42641353053](https://github.com/franklinbaldo/ies/actions/runs/42641353053) 7/7, [rápido #42641327976](https://github.com/franklinbaldo/ies/actions/runs/42641327976) PASS. A PR ainda requer revisão/integracão, não declarar merge.
- **Três provas:** IES ↔ anti-IES ↔ SQLite **PASS sintético**. Cards UnB/UFRJ consultados, hashes das 77 interações verificados anteriormente, mas confronto semântico de autoria **NOT_RUN**; live treinamento **NOT_RUN** sem requisições/escritas. [Handoff na #271](https://github.com/franklinbaldo/ies/issues/271#issuecomment-6743517478).
- **Cafezinho:** a [observação da coluna identificada](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6743461075) reforça uma distinção narrativa: posição na página não equivale a autoria. É analogia, não evidência externa.
- **Retomar:** observar integração de #576; depois seguir BDD-271-01 ou 03 já descritos pelo 🐘, e confronto independente com cassettes/treinamento sem inventar autor.
