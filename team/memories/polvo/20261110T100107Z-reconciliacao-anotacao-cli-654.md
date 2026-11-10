---
type: AgentMemory
title: "BDD-ANOT-02: preservar duas alterações concorrentes em merges verdadeiros"
description: "PR #654 reconciliada duas vezes com a main, preservando erros tipados e comando Comentarios; CI oficial 7/7 no head a6694d8. Prova externa ainda pendente."
agent: polvo
timestamp: "2026-11-10T10:01:07.917Z"
tags: [bdd638, anotacao, anti-ies, cli, merges, erros-tipados, ci, cassette-first]
---

# Ponto de retomada

- **Fato:** [PR #654](https://github.com/franklinbaldo/ies/pull/654) segue Ready, não integrada. No BDD-ANOT-02, a primeira sincronização por merge real [`49ef9d625`](https://github.com/franklinbaldo/ies/commit/49ef9d625133d3489d7ffacf0ee695a0179cce65) conciliou `src/processo_anotar.rs` com a propagação `?` da main. O segundo merge [`a6694d8fb`](https://github.com/franklinbaldo/ies/commit/a6694d8fb801e66f856710b37a0c3edd49e0de15) reteve o novo `Comentarios` da main em `src/cli_processo.rs` e a opção `Anotar --confirmar` desta PR. Nenhum rebase, force-push ou conflito silenciado.
- **Prova verificada:** [CI oficial #42794972559](https://github.com/franklinbaldo/ies/actions/runs/42794972559) **7/7 PASS** e [rápido #42794957190](https://github.com/franklinbaldo/ies/actions/runs/42794957190) PASS em `a6694d8fb`. [Handoff #638](https://github.com/franklinbaldo/ies/issues/638#issuecomment-6764445734). A main voltou a avançar (9 commits atrás na última consulta), mas a PR era mergeável. Conferir novamente SHA e gates no momento da integração pelo 🐘/🦡.
- **Níveis separados:** IES real↔anti-IES↔SQLite PASS sintético para um POST, releitura e 403 com `efeito_incerto`; hashes de 77 interações UnB/GO verificados anteriormente no Actions, **POST e releitura da cassette ainda NOT_RUN**, assim como live/treinamento. Não transformar integridade de ZIP em conformidade causal.
- **Próxima fatia após integração:** BDD-ANOT-03 já definido na #638, leitura direta de anotação com `ies processo anotar NUP` e campo JSON `anotacao` com texto ou `null`. Não antecipar mudanças na branch ainda aberta.
- **Cafezinho #388:** revisitado; a conversa da 🐢 sobre validação de identidade no despacho reforça conferir as condições da *transição realmente exercitada*. Uma barreira que falha antes do POST não prova o pós-efeito. É uma orientação para próximos falsificadores, não nova evidência de cassette.
