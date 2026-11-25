---
type: AgentMemory
title: "Oráculo independente de anexação refeito; gates de replay pendentes"
description: "HEAD de #777 corrigiu circularidade do oráculo GET por extrator da fonte; #781/#792/#790 esperam replay oficial, #778 tem 7/7 mas está defasada."
agent: elefante
timestamp: "2026-11-11T14:13:08Z"
tags: [integracao, cassette, replay, oraculo, ci, ablação, cafezinho]
---

# Retomar

- **Fato, sem merge:** reli AGENTS.md, team/elefante.md e as últimas cinco memórias, visitei os comentários do [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Li diffs e jobs oficiais. Atualizei o [roadmap #56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6798131672).
- **#777:** HEAD 56cc745b, behind=2 na consulta. Nova função fatos_da_fonte usa atributos do HTML original UnB/GO sem ler_tela como oráculo; semeia SQLite a partir dos IDs da fonte e confronta IES e anti-IES por HTTP separadamente. Corrige circularidade para GET/nomes de controles/IDs, mas não comprova valores de todos os campos hidden, POST integral ou HTTP 503 pós-POST. Delimitei [na PR](https://github.com/franklinbaldo/ies/pull/777#issuecomment-6798122478). [CI #43034478652](https://github.com/franklinbaldo/ies/actions/runs/43034478652): seis jobs verdes, replay amplo em andamento. Aguardar sete, replay nominal e revisão nova do 🦅, depois behind=0.
- **#781:** HEAD 8ddd45ea, behind=1, replay unidade trocar agora assert real (leitura, Controle, ausência de recusa); [CI #43035581862](https://github.com/franklinbaldo/ies/actions/runs/43035581862) com prek/replay em curso. IES↔original PASS no head anterior, novo HEAD ainda não validado.
- **#792:** HEAD 72f89859, behind=3, base64 inválido no harness deixa de virar corpo vazio; [CI #43034175534](https://github.com/franklinbaldo/ies/actions/runs/43034175534) replay em execução. Corrupção é hipótese sintética. **#790:** HEAD 51c287e0, behind=4, teste falso verde de login substituído por asserção; [CI #43033847947](https://github.com/franklinbaldo/ies/actions/runs/43033847947) replay em execução.
- **#778:** HEAD 0577945b, behind=6; [CI #43032216017](https://github.com/franklinbaldo/ies/actions/runs/43032216017) 7/7 no próprio SHA, com scripts/replay acervo, mas falta reconciliação. Ablação local de Error::source() sem alteração Web; E2E específico não aplicável. **#526 e #784** continuam Draft.
- **Cafezinho:** a 🦊 esclareceu que relatório Lido/Recusado não é gate sem assert; 🦫 distinguiu corrupção da cassette de ausência legítima. Nenhum PASS sintético promovido a paridade externa.

**Próximo ciclo:** consultar main e heads, conferir replay nominal no HEAD final, 7/7, revisão e behind=0 antes de squash. Transições não capturadas permanecem NOT_RUN frente à fonte. Sem SEI externo, produção, segredo, retry, subcódigo ou Gherkin.
