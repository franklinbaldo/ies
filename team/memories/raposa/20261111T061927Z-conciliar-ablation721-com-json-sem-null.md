---
type: AgentMemory
title: "Ablação #721: reconciliação sem perder o contrato JSON sem null"
description: "PR #750 recebeu merge verdadeiro de 40 commits da main; preservou TelaServida e as omissões de chaves JSON da #759, sem conflito textual. CI rápido verde; oficial iniciado."
agent: raposa
timestamp: "2026-11-11T06:19:27Z"
tags: [ux, ablação, tela-servida, json, merge, e2e, cassette-first, cafezinho]
---

# Retomar

**Fato:** a [PR #750](https://github.com/franklinbaldo/ies/pull/750) foi reconciliada por merge real no SHA [`eaa1abce`](https://github.com/franklinbaldo/ies/commit/eaa1abcedf1338d1b9331b7d4633e387cd60b7d3), a partir de `main@34f51ab1` (40 commits atrás). O delta continua restrito a quatro arquivos (+38/−45 linhas), removendo duas supressões de argumentos pela reutilização de `TelaServida`; a mudança recente da #759 que omite chaves JSON ausentes permaneceu na árvore. O merge automático baseado nos hunks de base foi conferido sem sobreposições. O [CI rápido #42971310393](https://github.com/franklinbaldo/ies/actions/runs/42971310393) passou; o [CI oficial #42971390198](https://github.com/franklinbaldo/ies/actions/runs/42971390198) estava em execução na consulta, portanto **não** alegar 7/7 para esse SHA antes da conclusão.

**Prova:** não há mudança pública intencional nem novo E2E nesta ablação. Cards UnB de prazo (33) e atribuição (24) revisitados, mas sem confronto semântico específico com os ZIPs; live `NOT_RUN`. A própria leitura do card não prova paridade. Método vigente: E2E cassette-first sem Gherkin. Nenhum SEI externo acessado.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a observação da 🦫 sobre preservar a causa sem fazer afirmações além da prova. **Próximo passo:** conferir sete gates no head exato, revisar `behind` após esta memória e pedir integração 🐘/🦡 sem reiniciar ablação.
