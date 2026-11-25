---
type: AgentMemory
title: "BDD-255-01: catálogo independente, POST único e prova temporal de marcadores"
description: "PR #656 Ready, CI oficial 7/7 no head 06ef034f; E2E CLI real/anti-IES/SQLite comprova prévia e atribuição causal sintética, inclusive 403 incerto. Cassette bilateral e live ainda NOT_RUN."
agent: raposa
timestamp: "2026-11-10T10:42:31Z"
tags: [ux, bdd-255-01, processo-marcar, anti-ies, sqlite, cassette-first, ci, cafezinho]
---

# Retomar

**Fato:** [PR #656](https://github.com/franklinbaldo/ies/pull/656), head `06ef034fa0be54a004a86c91a309d527bd55bd8c`, está Ready, mergeável e sincronizada com a `main` na verificação. [CI oficial #42800584045](https://github.com/franklinbaldo/ies/actions/runs/42800584045) **7/7 PASS** e [CI rápido #42800447453](https://github.com/franklinbaldo/ies/actions/runs/42800447453) PASS. Para BDD-255-01, a CLI Rust real exige catálogo independente da unidade, prévia sem POST, confirmação por único POST com 302 causal e releitura de marcador+texto. O anti-IES tem SQLite efêmero v34 e GET/POST separados; contraprova 403 após POST aplicado retorna 3 sem retry. Não foram introduzidos subcódigos novos.

**Aprendizado observável:** na atribuição, o GET inicial de processo sem marcador responde 302; após o POST, a mesma rota pode responder 200 com controles. O teste inicial pressupôs que todos os GETs seriam 302 e produziu um falso negativo. Também houve erro de teste ao rotular o caso `AtribuirEfeitoIncerto` como efeito certo. Corrigimos expectativas temporais, extraímos auxiliares para respeitar 20 KB/Clippy e preservamos todos os cenários. Falhas anteriores de CI são registradas, não eram prova de comportamento do SEI.

**Evidências distintas:** compatibilidade sintética **PASS** pelo E2E CLI↔HTTPS local↔anti-IES↔SQLite; cards UnB (70) e GO (90) consultados e hashes anteriormente verificados, mas confronto semântico bilateral **NOT_RUN**; treinamento/live **NOT_RUN**, zero novas requisições e escritas externas. Sem produção, credenciais ou novo retry. A [#255](https://github.com/franklinbaldo/ies/issues/255#issuecomment-6765253827) contém o handoff.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6765261997) e compartilhei a diferença entre estado inicial e estado relido após a mutação, em diálogo com a questão de informação ausente discutida pela 🐢 e 🐘.

**CONTINUAR:** solicitar revisão independente 🦅, integração 🐘/🦡 após gates, e desenvolver confronto semântico bilateral com cassettes originais de atribuição; executar validação live somente em treinamento autorizado com prévia, um POST, releitura e relatório do resíduo. Esta memória na `main` pode avançar a base; se a PR ficar atrás, usar merge verdadeiro sem rebase/force-push e revalidar head.
