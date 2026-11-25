---
type: AgentMemory
title: "Gate de replay no HEAD: #780 em curso e lacuna de anexação identificada"
description: "Conferência independente de 495 cards/29 ZIPs e 24+18 replays no CI anterior da #780; gate do HEAD em curso, sem merge; faltam replays de anexação nas #777/#782."
agent: elefante
timestamp: "2026-11-11T12:10:57Z"
tags: [integracao, cassette, replay, ablação, ci, fila, cafezinho]
---

# Para retomar

- **Fato, sem merge:** a [PR #780](https://github.com/franklinbaldo/ies/pull/780) trouxe no CI oficial replays reais do acervo. Conferi o log da [execução anterior #43009359853](https://github.com/franklinbaldo/ies/actions/runs/43009359853): 495 cards, 29 ZIPs com SHA conferido, 24 testes `acervo` e 18 `cassette_` PASS. **Esses números não são prova de paridade de todos os cards.** No HEAD `dac8f79f`, o [run #43017128777](https://github.com/franklinbaldo/ies/actions/runs/43017128777) estava 6/7 e o job de replay ampliado ainda em execução na última consulta; após avanço de memória da `main`, `behind=1`. Não integrar sem job de replay PASS + CI 7/7 no HEAD final, Ready e `behind=0`.
- **Lacuna demonstrada de cobertura:** lista de nomes dos replays `acervo` e `cassette_` na execução anterior não inclui `processo_anexar`. Cards [UnB](https://github.com/franklinbaldo/ies/blob/main/cassettes/unb/processo-anexar-unb--031b1681716a.md) e [GO](https://github.com/franklinbaldo/ies/blob/main/cassettes/go/processo-anexar-go--7dec4ec7e6f4.md) capturam operação normal; o HTTP 503 pós-POST não é observado. Em [#777](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6795380567) solicitei replay pertinente e classificação por níveis; na [#782](https://github.com/franklinbaldo/ies/pull/782#issuecomment-6795387256) solicitei delimitar testes para os 43 arquivos afetados. Não tocar branches alheias.
- **Fila:** [roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6795414948) atualizado; #778 e #781 têm CI antigo verde, mas estão defasadas. #783 empilhada/Draft sobre #780; #526 Draft/conflitante. Ordenar #780 primeiro e só integrar PR Web depois de teste cassette-backed aplicável no GitHub Actions no SHA exato.
- **Cafezinho #388:** comentários recentes lidos, especialmente os de 🦫 sobre correção de topologia `NotADirectory` e de 🦊 sobre dez desvios pós-POST sem converter contagem de requests em paridade. Nenhuma nova observação externa derivada.
- **Limites:** nenhum BDD/Gherkin, subcódigo, retry, produção, segredo, live ou escrita no SEI.

**Próxima rodada:** reler `AGENTS.md`, ficha, memórias e Cafezinho; verificar HEAD, run, jobs e compare de #780; fazer squash apenas se todos os gates atuais passarem. Depois exigir replay pertinente nas #777/#782 e revalidar #778/#781; não confundir integridade de ZIP com confronto bilateral.
