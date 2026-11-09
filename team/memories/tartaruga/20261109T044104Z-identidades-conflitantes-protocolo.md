---
type: AgentMemory
title: "Parâmetros de seleção documental contraditórios"
description: "PR #537: RED reproduziu árvore 200 ao misturar id_documento=42 com id_protocolo=43; correção mínima recusa 400; CI final verde, paridade externa pendente."
agent: tartaruga
timestamp: "2026-11-09T04:41:04Z"
tags: [anti-ies, identidade, navegacao, bdd, http, sqlite, cassette]
---

# Para lembrar

**Fato:** a [PR #537](https://github.com/franklinbaldo/ies/pull/537) isola a identidade contraditória no BDD-60-05. [RED #42562601847](https://github.com/franklinbaldo/ies/actions/runs/42562601847) verificou HTTP 200 indevido na árvore; agora há 400 para seleções divergentes e 200 quando concordantes, reutilizando parser existente. [CI final #42562840914](https://github.com/franklinbaldo/ies/actions/runs/42562840914) 7/7 no SHA `664a433`; PR Ready para revisão/integração. É teste HTTP/SQLite, sem E2E novo da CLI nesse subcaso.

**Prova externa ainda ausente:** [inspeção de cassettes UnB/GO #42562598796](https://github.com/franklinbaldo/ies/actions/runs/42562598796) conferiu hashes/estrutura, 30 HTTP; não houve replay semântico independente nem live. A recusa de parâmetros conflitantes é hipótese funcional do anti-IES, não observação comprovada do SEI.

**Retomar:** verificar integração da #537, então procurar outra lacuna do BDD existente sem refazer este caso. No [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), a discussão sobre duas hipóteses com a mesma projeção reforçou a ideia de exigir uma testemunha que discrimine identidades; analogia, não evidência SEI.
