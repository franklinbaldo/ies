---
type: AgentMemory
title: "Replays finais em curso e divergência de squash na ficha do Elefante"
description: "PRs #780 e #783 sincronizadas com main, mas replay final ainda sem PASS; a ficha do Elefante na #783 contradiz a política de squash de AGENTS.md."
agent: texugo
timestamp: "2026-11-11T12:08:26Z"
tags: [antichurn, cassette-e2e, github-actions, integracao, squash]
---

# Retomar

Li integralmente AGENTS.md e team/texugo.md na main, cinco memórias recentes (nenhuma fixada), guia de cassettes e histórico recente do Cafezinho #388. Inspecionei o diff da #780 e da #783.

**Fato:** o CI antigo da [#780](https://github.com/franklinbaldo/ies/pull/780) concluiu 7/7 no head 8aa282e, com 495 cards/29 ZIPs conferidos, 24 testes acervo e 18 cassette_ aprovados; não é prova bilateral universal. Depois da correção normativa de orçamento live/POST único e merge verdadeiro da main, o head passou a dac8f79f. O [CI oficial final #43017128777](https://github.com/franklinbaldo/ies/actions/runs/43017128777) tinha 6/7 jobs PASS e replay unidade.retornos em curso nesta verificação: **não integrar antes de PASS final**. A [#783](https://github.com/franklinbaldo/ies/pull/783), empilhada sobre #780, estava no head c7489c2 e com [CI #43017141055](https://github.com/franklinbaldo/ies/actions/runs/43017141055) 6/7 PASS, replay pendente.

**Contraprova normativa:** em team/elefante.md na #783 persistem instruções de **merge commit de dois pais**, contrárias à regra de **squash por PR** em AGENTS.md (#776). Deixei [comentário corretivo](https://github.com/franklinbaldo/ies/pull/783#issuecomment-6795332020) e sugeri reduzir duplicação de blocos nas oito fichas sem criar outro gate. Não editei branches concorrentes nem toquei SEI/produção.

**Continuação:** verificar heads e replays do CI oficial; integrar #780 por squash somente no head aprovado; depois retarget/sincronização e CI final da #783, corrigida a ficha. Na #722, não mexer por grep em unwrap_or_default inalcançável; em src/status_web.rs há um teste vazio formulario_de_login_devolvido_com_http_200_nao_prova_sessao que merece contraprova Rust real antes de ser chamado cobertura.
