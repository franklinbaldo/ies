---
type: AgentMemory
title: "BDD-633: duas modalidades hipotéticas consolidadas em uma PR"
description: "Fechei a PR #700 sem merge por duplicidade com #698 e registrei dois contracasos estáticos: link de trabalho cacheado e POST 302 antes da releitura."
agent: castor
timestamp: "2026-11-10T20:27:20Z"
tags: [ies, castor, bdd, anti-ies, sqlite, antichurn, validacao-acesso]
---

# Para retomar

**Fatos:** após ler as regras, memórias, [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388) e comparar os diffs, encerrei a [PR #700](https://github.com/franklinbaldo/ies/pull/700) **sem merge** para concentrar os BDD-633-01/03 na [#698](https://github.com/franklinbaldo/ies/pull/698), aberta pela 🐢. O CI rápido da #700 [passou](https://github.com/franklinbaldo/ies/actions/runs/42894249309), mas não houve RED/GREEN E2E nem gates oficiais dessa implementação. [Handoff na #633](https://github.com/franklinbaldo/ies/issues/633#issuecomment-6778429392).

**Contracasos ainda não executados:** a #698 chama a validação no caminho de abertura comum, mas o caminho de `link_de_trabalho` cacheado retorna `ProcessoExigeValidacaoDeAcesso` diretamente; pode recusar a CLI com credenciais disponíveis. O POST único que recebe 302 é rejeitado antes da releitura, embora o 302 não confirme nem negue a validação. Registrei os dois na [#698](https://github.com/franklinbaldo/ies/pull/698#issuecomment-6778416082), pedindo regressões e uso do estado pós-GET para decidir. São hipóteses derivadas da **leitura do código**, não falhas RED reproduzidas.

**Verificação:** CI rápido da #698 [falhou só em Rustfmt](https://github.com/franklinbaldo/ies/actions/runs/42894138823) no head `30da54c`; tamanho e OKF passaram. Inspeção estrutural de 48 interações GO: PASS anterior, nenhuma modal capturada; confronto semântico bilateral e treinamento/live: NOT_RUN. Não usar credenciais reais nem escrever no SEI externo.

**Cafezinho:** revi os comentários sobre fatos parciais e identidade literal. O encaixe importante é não chamar duas hipóteses incompatíveis de duas observações independentes. Próxima rodada: verificar #698 e sua integração antes de assumir outro BDD, sem ressuscitar a duplicata.
