---
type: AgentMemory
title: "Link no Controle R exige linha de origem, não só estado derivado"
description: "O BDD-342-02 passou pela CLI real quando o anti-IES projetou href apenas nas linhas originalmente registradas; os testes antigos de atribuição e criação voltaram sem alterações."
agent: castor
timestamp: "2026-11-09T10:09:10Z"
tags: [leitura-parcial, identidade, anti-ies, cli, sqlite, proveniencia]
---

# Junta para retomar

**Fato:** a [PR #551](https://github.com/franklinbaldo/ies/pull/551) introduz E2E TLS local para BDD-342-02: Controle R parcial observa o NUP pedido e a árvore do mesmo ID informa outro NUP; a CLI recusa sem escrita. Tornar *todas* as linhas sintéticas navegáveis quebrou testes preexistentes de atribuição e criação. A correção projeta `href` só nas linhas da fotografia original; linhas acrescentadas pelo SQLite mantêm a busca rápida. Sem mudar as expectativas dos testes antigos, o [CI final](https://github.com/franklinbaldo/ies/actions/runs/42598234993) passou 7/7 no head `e8209c35`.

**Limites:** PASS sintético, mas a contradição não está na cassette UnB e o teste live segue NOT_RUN. Retomar apenas se revisão 🦅 solicitar ajuste; 🐘/🦡 sincronizam a `main` e integram. O [Cafezinho](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6737401915) lembrou que uma seta desenhada para uma tábua não examinada muda o trajeto: analogia, não prova do SEI.
