---
type: AgentMemory
title: "BDD-60-05: chave documental sem igual pode virar falso campo ausente"
description: "A PR #687 propõe detectar id_documento sem = antes do fallback de navegação; teste HTTP sintético foi escrito antes da correção, mas o CI ainda não executou."
agent: tartaruga
timestamp: "2026-11-10T16:38:23Z"
tags: [bdd60, anti-ies, navegacao, http, sqlite, cassette-first]
---

# Para retomar

**Fato de código:** a análise da `documento_arvore::identidade_documental` encontrou `filter_map(split_once('='))`: uma chave `id_documento` sem sinal de igual desaparecia, permitindo que `id_protocolo` fosse escolhido em seu lugar. Na [PR #687](https://github.com/franklinbaldo/ies/pull/687) há teste HTTP/SQLite novo no commit `ab9878e9` e correção mínima no `af0a803e`.

**Ainda não comprovado:** apesar da transição Draft → Ready, a consulta a Actions não retornou workflow para esses commits. **Não declarar RED nem GREEN**, nem paridade de cassette: os sete cards `documento-numero-e-busca` previamente confrontados não contêm essa forma de URL. Próxima rodada deve verificar os runners, tratar falhas, sincronizar a `main` por merge verdadeiro e revalidar o SHA atual. Não mesclar sem gates.

**Cafezinho:** li a [conversa #388](https://github.com/franklinbaldo/ies/issues/388) sobre a diferença entre ausência de observação e vazio efetivamente observado. A relação com esta chave sem `=` é uma analogia de representação, não evidência externa.
