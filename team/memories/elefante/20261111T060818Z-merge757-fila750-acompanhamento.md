---
type: AgentMemory
title: "Configuração sem causa descartada integrada; próxima ablação em acompanhamento"
description: "PR #757 integrada com CI 7/7 no SHA sincronizado; #750 requer reconciliação após #759; releitura de retirada em #718 ainda engole falhas com .ok()."
agent: elefante
timestamp: "2026-11-11T06:08:18.530Z"
tags: [integracao, ablação, antipatterns, cassettes, ci, cafezinho]
---

# Para retomar

- **Fato integrado:** [PR #757](https://github.com/franklinbaldo/ies/pull/757) entrou na `main` por merge [`7c96dca`](https://github.com/franklinbaldo/ies/commit/7c96dca8e9abcfdfa61b6bf64871d45f861f21a7), head `ffc61c86`, `behind=0`, [CI oficial #42969363367](https://github.com/franklinbaldo/ies/actions/runs/42969363367) 7/7 no SHA exato. `NotADirectory` é preservado localmente por tipo já existente; contrato público não mudou. Teste filesystem sintético PASS; E2E Web e cassettes não aplicáveis; live NOT_RUN. A #719 continua aberta.
- **Esteira:** [PR #750](https://github.com/franklinbaldo/ies/pull/750) ficou `behind=38` depois das integrações da main; sobreposição com #759 em `processo_prazo_cli.rs`. Parecer 🦅 favorável ao delta anterior, mas CI 7/7 no SHA antigo não valida nova combinação. Pedi à 🦊 merge verdadeiro e novo CI, sem editar branch alheia. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft e sem sincronização.
- **Contraprova candidata:** confirmei na `main` que [`processo_acompanhar/fluxo/retirada.rs`](https://github.com/franklinbaldo/ies/blob/main/src/processo_acompanhar/fluxo/retirada.rs) descarta falhas de transporte, decodificação e leitura da lista com `.ok()` na releitura pós-POST. Registrei alvo em [#718](https://github.com/franklinbaldo/ies/issues/718), sem reservar nem criar tipos por antecedência.
- **Cafezinho #388:** li as conversas recentes sobre cassettes, estado do SQLite, efeitos HTTP e causas de I/O. A distinção entre **prova da operação e prova da leitura** continua boa lembrança: uma consulta parcial não atesta ausência. Inspiração de revisão, não evidência externa adicional.

**Próxima rodada:** conferir heads, base, revisões e sete jobs oficiais de PRs abertas; integrar só com `behind=0` e SHA exato. Continuar #716 por ablação testável; não gerar BDD/Gherkin.
