---
type: AgentMemory
title: "Sobrestamento: botão observado só aparece no confronto independente"
description: "PR #640 compara GET real UnB/GO com HTML HTTP do anti-IES; RED encontrou btnPesquisar ausente, GREEN restaurou forma, sem provar efeito do POST."
agent: polvo
timestamp: "2026-11-10T04:55:10Z"
tags: [bdd269, cassettes, anti-ies, replay, formulario, causalidade]
---

# Para a próxima rodada

- **Fato:** [PR #640](https://github.com/franklinbaldo/ies/pull/640) usa ZIPs originais SHA-verificados UnB/GO e o leitor IES para comparar a estrutura do GET `procedimento_sobrestar` com o anti-IES HTTP sustentado por SQLite. [RED #42758081817](https://github.com/franklinbaldo/ies/actions/runs/42758081817) encontrou `btnPesquisar` ausente no anti-IES; [GREEN #42758302661](https://github.com/franklinbaldo/ies/actions/runs/42758302661) passou após restaurar apenas o botão observado. Workflow temporário removido da PR, CI rápido verde no head final.
- **Limite:** a prova externa é **estrutural parcial**: não valida POST/302, efeito ou autoria; treinamento permanece NOT_RUN, zero SEI externo. Não confundir com E2E sintético da PR #627, já mesclada. O CI oficial [#42758557017](https://github.com/franklinbaldo/ies/actions/runs/42758557017) ainda estava em execução ao escrever esta nota; conferir resultado final.
- **Ritual:** no [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6759334884), conversamos sobre a diferença entre um leitor aceitar HTML e o simulador preservar todos os controles observados. Próximo ponto: revisão Gavião, integração Elefante/Texugo e confrontação semântica do efeito sem fabricar corpos POST.
