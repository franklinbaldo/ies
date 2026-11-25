---
type: AgentMemory
title: "Documento cancelado ainda oferecia link de versões no anti-IES"
description: "RED no SQLite/árvore confirmou oferta indevida de histórico após cancelamento; PR #607 corrige navegação e inclui CLI real, com GREEN inicial 7/7, revalidação pós-merge pendente."
agent: tartaruga
timestamp: "2026-11-09T22:02:48Z"
tags: [bdd60, anti-ies, navegacao, cancelamento, sqlite, tdd, cassette-first]
---

# Para retomar

**Fato:** a [PR #604](https://github.com/franklinbaldo/ies/pull/604) foi integrada. Na nova [PR #607](https://github.com/franklinbaldo/ies/pull/607), o RED [#42706715084](https://github.com/franklinbaldo/ies/actions/runs/42706715084) reproduziu **12 testes OK, 1 falha**: o anti-IES publicava dois links de histórico na árvore mista mesmo depois de cancelar DOC_A no SQLite, quando deveria anunciar apenas DOC_B sob a hipótese adotada. A correção condiciona a ação à disponibilidade em árvores individual e mista e preserva a recusa HTTP 403 para links previamente publicados. Incluí teste de CLI real com `--versao 1`, com e sem ocultação.

**Gates:** [GREEN inicial #42707538525](https://github.com/franklinbaldo/ies/actions/runs/42707538525) 7/7 e [rápido #42707502413](https://github.com/franklinbaldo/ies/actions/runs/42707502413) verde no SHA `859f2fe`. Fiz merge verdadeiro da main em `b823aa2`; **falta revalidar gates no SHA final** após eventual incorporação desta memória.

**Limite:** cards UnB de versões e de mover/cancelar consultados, com verificações anteriores no Actions; não há gravação de remoção de link especificamente após cancelamento. A recusa da CLI e oferta da barra são provas **sintéticas**, não paridade externa. Treinamento/live nesta fatia NOT_RUN. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) li a discussão mais recente sobre estado observado versus atribuição do efeito, que continua útil para distinguir simulação, cassette e live.

**Próximo:** sincronizar esta memória na PR por merge verdadeiro, executar CI no novo head e comunicar à #60, preservando a revisão 🦅 e integração 🐘/🦡. Sem produção, credenciais ou novos subcódigos.
