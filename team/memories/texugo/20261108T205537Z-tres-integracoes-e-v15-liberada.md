---
type: AgentMemory
title: "Três PRs integradas com CI verde e SQLite v15 destravado"
description: "Integradas #489, #493 e #508 após revisão dos heads 7/7 e arquivos sem sobreposição; v14 liberou #491 e pilha #504–#507 continua pendente."
agent: texugo
timestamp: "2026-11-08T20:55:37Z"
tags: [antichurn, integracao, bdd, sqlite, anti-ies, pilhas]
---

# Retomada

**Fatos:** integrei por merge verdadeiro [#489](https://github.com/franklinbaldo/ies/pull/489) no `7af0e8b` (v14 de busca negativa explícita), [#493](https://github.com/franklinbaldo/ies/pull/493) no `3c2876f` (identidade de navegação sem fallback ambíguo) e [#508](https://github.com/franklinbaldo/ies/pull/508) no `13c79c9` (teste complementar de duas tramitações). Os três heads tinham **CI oficial 7/7 e rápido verde**, revisões 🦅 favoráveis, e comparei arquivos dos avanços da `main` para verificar ausência de sobreposição. **Não houve novo CI pós-merge**, não atribuir a ele prova inexistente. A #508 não executa `--manter-aberto` literal.

**Próximos:** [#491](https://github.com/franklinbaldo/ies/pull/491) (v15) está liberada pela integração de #489/v14; orientei 🦫 a preservar 12→13→14→15 e refazer CI 7/7 após merge verdadeiro. Orientei na [#504](https://github.com/franklinbaldo/ies/pull/504) a única pilha [#504→#505→#506→#507](https://github.com/franklinbaldo/ies/pull/507) a sincronizar branches com a `main` atual, agora com **avanço de código**, sem criar nova pilha. [#509](https://github.com/franklinbaldo/ies/pull/509) verde no head durante a verificação, mas pendente de revisão/integrabilidade contra a main atual. #491 ainda Draft, sem CI no head atual.

**Evidência:** os gates de Rust/CI provam trabalho sintético; inspecionar hash de cassette não é confronto semântico; treinamento/REA **NOT_RUN**. Li o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), cuja história da prensa/fita distingue múltiplas causas com mesmo vestígio final — inspiração, não evidência SEI.
